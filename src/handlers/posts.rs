use axum::{
    Form, Json, extract::{Path, State}, http::{HeaderMap, StatusCode}, response::{Html, IntoResponse, Response}
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_http::cors::AllowHeaders;
use std::sync::Arc;
use time::OffsetDateTime;
use image::GenericImageView;
use base64::prelude::*;
use tracing;

use crate::errors::{AppError, Result, ValidationError};
use crate::middleware::{
    auth::AuthUser,
    csrf::{validate_token, generate_token},
};
use crate::utils::rate_limit::{RateLimitAction, RateLimiter};
use crate::state::AppState;
use crate::utils::validation::{
    ContentValidator, MediaValidator, TagValidator,
    generate_content_hash, generate_post_slug, ALLOWED_IMAGE_TYPES, ALLOWED_VIDEO_TYPES,
};
use crate::markdown::parse_markdown;
use crate::dto::media::{TagResponse, MediaUpload, MediaResponse};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: Option<String>,
    pub content: String,
    pub tags: String,
    #[serde(default)]
    pub media: Vec<MediaUpload>,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PostResponse {
    pub id: i32,
    pub user_id: i32,
    pub avatar_url: Option<String>,
    pub username: String,
    pub title: Option<String>,
    pub content_rendered_html: String,
    pub slug: String,
    pub is_spam: bool,
    pub tags: Vec<TagResponse>,
    pub media: Vec<MediaResponse>,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub refract_count: i32,
    pub comment_count: i32,
    pub created_at: OffsetDateTime,
    pub edited_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    pub cursor: Option<String>,
}

fn default_limit() -> i32 {
    20
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/posts - Create a new post
pub async fn create_post(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    Form(req): Form<CreatePostRequest>,
) -> Result<Response> {
    
    if !validate_token(&state, &req.csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid)
    }
    // Rate limit check
    rate_limiter
        .check_user_limit(auth_user.user_id, RateLimitAction::Post)
        .await?;

    // Validate title if provided
    if let Some(ref title) = req.title {
        if !title.trim().is_empty() {
            ContentValidator::validate_title(title)?;
        }
    }

    // Validate content
    ContentValidator::validate_post(&req.content)?;

    let tag_list: Vec<String> = req.tags
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();

    tracing::info!(tags_raw = %req.tags, "Raw tags received");

    // Validate tags count
    TagValidator::validate_count(tag_list.len())?;

    // Validate and normalize tags
    let mut validated_tags = Vec::new();
    for tag_name in &tag_list {
        let normalized = TagValidator::validate_name(tag_name)?;
        validated_tags.push(normalized);
    }

    // Validate media
    let mut validated_media = Vec::new();
    if !req.media.is_empty() {
        validated_media = validate_media_uploads(&req.media).await?;
    }

    // Check for duplicate content (same user, last 24 hours)
    let content_hash = generate_content_hash(&req.content);
    check_duplicate_content(&state.pool, auth_user.user_id, &content_hash).await?;

    // Parse markdown to HTML
    let content_rendered_html = parse_markdown(&req.content);

    // Generate temporary slug (before we have post ID)
    let temp_slug = generate_post_slug(
        req.title.as_deref(),
        validated_tags.first().map(|s| s.as_str())
    );

    // Start transaction
    let mut tx = state.pool.begin().await?;

    // Insert post with temporary slug
    let post = sqlx::query!(
        r#"
        INSERT INTO posts (user_id, title, content_raw, content_rendered_html, content_hash, slug)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, title, content_rendered_html, 
                  echo_count, refract_count, comment_count, 
                  created_at, edited_at
        "#,
        auth_user.user_id,
        req.title.as_deref(),
        req.content,
        content_rendered_html,
        content_hash,
        temp_slug,
    )
    .fetch_one(&mut *tx)
    .await?;

    // Update slug with final version including post ID
    let final_slug = format!("{}-{}", temp_slug, post.id);
    sqlx::query!(
        "UPDATE posts SET slug = $1 WHERE id = $2",
        final_slug,
        post.id
    )
    .execute(&mut *tx)
    .await?;

    // Handle tags
    let tag_responses = insert_post_tags(&mut tx, post.id, &validated_tags, auth_user.user_id).await?;

    // Apply tag creation rate limit for new tags
    let new_tags_count = tag_responses.iter().filter(|t| t.is_new).count();
    if new_tags_count > 0 {
        rate_limiter
            .check_user_limit(auth_user.user_id, RateLimitAction::TagCreation)
            .await?;
    }
    tracing::info!(tags_raw = %req.tags, "Raw tags received");

    // Handle media uploads
    let media_responses = if !validated_media.is_empty() {
        insert_post_media(&mut tx, post.id, validated_media.clone()).await?
    } else {
        Vec::new()
    };

    // Commit transaction
    tx.commit().await?;

    // Get username for response
    let user = sqlx::query!(
        "SELECT username, avatar_url FROM users WHERE id = $1",
        auth_user.user_id
    )
    .fetch_one(&state.pool)
    .await?;

    // Build response
    let response = PostResponse {
        id: post.id,
        user_id: post.user_id,
        username: user.username,
        avatar_url: user.avatar_url,
        title: post.title,
        content_rendered_html: post.content_rendered_html,
        slug: final_slug.clone(),
        is_spam: false,
        tags: tag_responses.into_iter().map(|t| t.tag).collect(),
        media: media_responses,
        echo_count: post.echo_count.unwrap_or(0),
        has_echoed: false,
        refract_count: post.refract_count.unwrap_or(0),
        comment_count: post.comment_count.unwrap_or(0),
        created_at: post.created_at,
        edited_at: post.edited_at,
    };

    tracing::info!(
        user_id = auth_user.user_id,
        post_id = post.id,
        slug = %final_slug,
        tags = ?validated_tags,
        has_media = !validated_media.is_empty(),
        "Post created"
    );

    // Return HTML fragment (will be rendered by Maud template in routes)
    Ok((
        StatusCode::SEE_OTHER,
        [("Location", format!("/posts/{}", final_slug))]
    ).into_response())
}

/*impl From<TagResponse> for TagInfo {
    fn from(t: TagResponse) -> Self {
        TagInfo { id: t.id, name: t.name, slug: t.slug }
    }
}
    
    and then 
    tags: response.tags.into_iter().map(Into::into).collect(),
*/



pub async fn render_create_post_page_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Html<String>> {

    let csrf_token = generate_token(&state, auth_user.session_id).await?;

    let user = sqlx::query!(
        "SELECT username, avatar_url FROM users WHERE id = $1",
        auth_user.user_id
    )
    .fetch_one(&state.pool)
    .await?;
    
    let user_info = (auth_user.user_id, user.username, user.avatar_url);
    let markup = crate::templates::create::render_create_post_page(csrf_token, user_info);
    Ok(Html(markup.into_string()))
}

/// GET /api/posts/:slug_or_id - Get single post by slug or ID
pub async fn get_post_by_slug_or_id(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    headers: HeaderMap,
    Path(param): Path<String>,
) -> Result<Html<String>> {

    let back_url = headers
        .get("Referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/")
        .to_string();

    let current_user_id = auth_user.as_ref().map(|a| a.user_id);
    // Try parsing as ID first (fast path)
    let post = if let Ok(id) = param.parse::<i32>() {
        fetch_post(&state.pool, id, current_user_id).await?
    } else {
        // Otherwise lookup by slug
        fetch_post_by_slug(&state.pool, &param, current_user_id).await?
    };

    let current_user_id = auth_user.as_ref().map(|a| a.user_id);

    let user = if let Some(ref auth) = auth_user {
        let u = sqlx::query!(
            "SELECT username, avatar_url FROM users WHERE id = $1",
            auth.user_id
        )
        .fetch_one(&state.pool)
        .await?;
        Some((auth.user_id, u.username, u.avatar_url))
    } else {
        None
    };

    let csrf_token = if let Some(auth) = &auth_user {
        generate_token(&state, auth.session_id).await.ok()
    } else {
        None
    };

    let comments = crate::handlers::comments::fetch_parent_comments(
        &state.pool,
        Some(post.id),
        None,
        None,
        11,
        None,
        current_user_id,
    )
    .await?;

    let has_more = comments.len() > 10;
    let comments: Vec<_> = comments.into_iter().take(10).collect();
    let next_cursor = comments.last().map(|c| {
        c.created_at.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default()
    });
    let total_count = post.comment_count;
    
    let markup = crate::templates::posts::render_post_page(post, user, csrf_token.clone(), comments, total_count, has_more, next_cursor, back_url);
    Ok(Html(markup.into_string()))
}

/// PATCH /api/posts/:id - Update post
pub async fn update_post(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(post_id): Path<i32>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Response> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }
    // Verify ownership
    verify_post_ownership(&state.pool, post_id, auth_user.user_id).await?;

    // Build update query dynamically
    let mut update_fields = Vec::new();
    let mut updated_html: Option<String> = None;

    // Validate and update title
    if let Some(ref title) = req.title {
        if !title.trim().is_empty() {
            ContentValidator::validate_title(title)?;
        }
        update_fields.push("title");
    }

    // Validate and update content
    if let Some(ref content) = req.content {
        ContentValidator::validate_post(content)?;
        updated_html = Some(parse_markdown(content));
        update_fields.push("content_raw");
        update_fields.push("content_rendered_html");
    }

    // Update post
    if !update_fields.is_empty() {
        let now = OffsetDateTime::now_utc();
        
        sqlx::query!(
            r#"
            UPDATE posts
            SET title = COALESCE($1, title),
                content_raw = COALESCE($2, content_raw),
                content_rendered_html = COALESCE($3, content_rendered_html),
                edited_at = $4
            WHERE id = $5 AND user_id = $6
            "#,
            req.title.as_deref(),
            req.content.as_deref(),
            updated_html.as_deref(),
            now,
            post_id,
            auth_user.user_id,
        )
        .execute(&state.pool)
        .await?;
    }

    // Update tags if provided
    if let Some(tags) = req.tags {
        TagValidator::validate_count(tags.len())?;
        
        let mut validated_tags = Vec::new();
        for tag_name in &tags {
            let normalized = TagValidator::validate_name(tag_name)?;
            validated_tags.push(normalized);
        }

        // Start transaction for tag updates
        let mut tx = state.pool.begin().await?;

        // Delete existing tags
        sqlx::query!(
            "DELETE FROM post_tags WHERE post_id = $1",
            post_id
        )
        .execute(&mut *tx)
        .await?;

        // Insert new tags
        insert_post_tags(&mut tx, post_id, &validated_tags, auth_user.user_id).await?;

        tx.commit().await?;
    }

    // Fetch updated post
    let post = fetch_post(&state.pool, post_id, Some(auth_user.user_id)).await?;

    tracing::info!(
        user_id = auth_user.user_id,
        post_id,
        "Post updated"
    );

    Ok(Json(post).into_response())
}

/// DELETE /api/posts/:id - Soft delete post
pub async fn delete_post(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(post_id): Path<i32>,
) -> Result<StatusCode> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    // Verify ownership
    verify_post_ownership(&state.pool, post_id, auth_user.user_id).await?;

    // Soft delete
    let now = OffsetDateTime::now_utc();
    let result = sqlx::query!(
        "UPDATE posts SET deleted_at = $1 WHERE id = $2 AND user_id = $3",
        now,
        post_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    tracing::info!(
        user_id = auth_user.user_id,
        post_id,
        "Post deleted"
    );

    Ok(StatusCode::OK)
}

// ============================================================================
// Helper Functions
// ============================================================================

struct PostRecord {
    id: i32,
    user_id: i32,
    username: String,
    avatar_url: Option<String>,
    title: Option<String>,
    content_rendered_html: String,
    slug: String,
    is_spam: bool,
    echo_count: Option<i32>,
    has_echoed: Option<bool>,
    refract_count: Option<i32>,
    comment_count: Option<i32>,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
struct ValidatedMedia {
    data: Vec<u8>,
    filename: String,
    mime_type: String,
    media_type: String, // "image" or "video"
    width: Option<u32>,
    height: Option<u32>,
}

struct TagInsertResult {
    tag: TagResponse,
    is_new: bool,
}

async fn validate_media_uploads(uploads: &[MediaUpload]) -> Result<Vec<ValidatedMedia>> {
    let mut validated = Vec::new();
    let mut image_count = 0;
    let mut video_count = 0;

    for upload in uploads {
        // Decode base64
        let data: Vec<u8> = BASE64_STANDARD.decode(&upload.data)
            .map_err(|_| ValidationError::MissingField("Invalid base64 data".to_string()))?;

        // Detect MIME type
        let mime_type = MediaValidator::detect_mime_type(&data)?;

        // Determine media type
        let media_type = if ALLOWED_IMAGE_TYPES.contains(&mime_type.as_str()) {
            image_count += 1;
            MediaValidator::validate_image_count(image_count)?;
            MediaValidator::validate_image_with_type(&data, &mime_type)?;
            "image"
        } else if ALLOWED_VIDEO_TYPES.contains(&mime_type.as_str()) {
            video_count += 1;
            MediaValidator::validate_video_count(video_count)?;
            MediaValidator::validate_video_with_type(&data, &mime_type)?;
            "video"
        } else {
            return Err(ValidationError::MissingField("Unsupported media type".to_string()).into());
        };

        // Get dimensions for images
        let (width, height) = if media_type == "image" {
            let img = image::load_from_memory(&data)
                .map_err(|_| ValidationError::InvalidImageFormat)?;
            let (w, h) = img.dimensions();
            (Some(w), Some(h))
        } else {
            (None, None)
        };

        validated.push(ValidatedMedia {
            data,
            filename: upload.filename.clone(),
            mime_type,
            media_type: media_type.to_string(),
            width,
            height,
        });
    }

    Ok(validated)
}

async fn check_duplicate_content(
    pool: &PgPool,
    user_id: i32,
    content_hash: &str,
) -> Result<()> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM posts
            WHERE user_id = $1
              AND content_hash = $2
              AND deleted_at IS NULL
              AND created_at > NOW() - INTERVAL '24 hours'
        ) as "exists!"
        "#,
        user_id,
        content_hash,
    )
    .fetch_one(pool)
    .await?;

    if exists.exists {
        return Err(ValidationError::DuplicateContent.into());
    }

    Ok(())
}

async fn insert_post_tags(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    post_id: i32,
    tag_names: &[String],
    user_id: i32,
) -> Result<Vec<TagInsertResult>> {
    let mut results = Vec::new();

    for tag_name in tag_names {
        let slug = TagValidator::create_slug(tag_name);

        // Try to get existing tag
        let existing_tag = sqlx::query!(
            "SELECT id, name, slug FROM tags WHERE slug = $1",
            slug
        )
        .fetch_optional(&mut **tx)
        .await?;

        let (tag_id, is_new) = if let Some(tag) = existing_tag {
            (tag.id, false)
        } else {
            // Create new tag
            let new_tag = sqlx::query!(
                r#"
                INSERT INTO tags (name, slug, created_by_user_id)
                VALUES ($1, $2, $3)
                RETURNING id
                "#,
                tag_name,
                slug,
                user_id,
            )
            .fetch_one(&mut **tx)
            .await?;

            (new_tag.id, true)
        };

        // Insert into post_tags
        sqlx::query!(
            "INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2)",
            post_id,
            tag_id,
        )
        .execute(&mut **tx)
        .await?;

        results.push(TagInsertResult {
            tag: TagResponse {
                id: tag_id,
                name: tag_name.clone(),
                slug: slug.clone(),
            },
            is_new,
        });
    }

    Ok(results)
}

async fn insert_post_media(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    post_id: i32,
    media: Vec<ValidatedMedia>,
) -> Result<Vec<MediaResponse>> {
    let mut responses = Vec::new();

    for (index, item) in media.into_iter().enumerate() {
        // TODO: Upload file to storage (S3, local filesystem, etc.)
        // For now, using placeholder path
        let file_path = format!("/uploads/posts/{}/{}_{}", post_id, index, item.filename);

        let record = sqlx::query!(
            r#"
            INSERT INTO post_media 
            (post_id, media_type, file_path, original_filename, mime_type, 
             file_size, width, height, display_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, media_type, file_path, mime_type, width, height
            "#,
            post_id,
            item.media_type,
            file_path,
            item.filename,
            item.mime_type,
            item.data.len() as i64,
            item.width.map(|w| w as i32),
            item.height.map(|h| h as i32),
            index as i32,
        )
        .fetch_one(&mut **tx)
        .await?;

        responses.push(MediaResponse {
            id: record.id,
            media_type: record.media_type,
            file_path: record.file_path,
            mime_type: record.mime_type,
            width: record.width,
            height: record.height,
        });
    }

    Ok(responses)
}

async fn verify_post_ownership(pool: &PgPool, post_id: i32, user_id: i32) -> Result<()> {
    let post = sqlx::query!(
        "SELECT user_id FROM posts WHERE id = $1 AND deleted_at IS NULL",
        post_id
    )
    .fetch_optional(pool)
    .await?;

    match post {
        None => Err(AppError::NotFound),
        Some(p) if p.user_id != user_id => Err(AppError::Forbidden),
        Some(_) => Ok(()),
    }
}

async fn fetch_post(pool: &PgPool, post_id: i32, current_user_id: Option<i32>) -> Result<PostResponse> {
    let post = sqlx::query_as!(
        PostRecord,
        r#"
        SELECT p.id, p.user_id, p.title, p.content_rendered_html, p.slug, p.is_spam,
               p.echo_count, p.refract_count, p.comment_count,
               p.created_at, p.edited_at, u.username, u.avatar_url,
               COALESCE((SELECT true FROM echos WHERE post_id = p.id AND user_id = $2), false) as "has_echoed!"
        FROM posts p
        JOIN users u ON p.user_id = u.id
        WHERE p.id = $1 AND p.deleted_at IS NULL AND p.is_spam = false
        "#,
        post_id,
        current_user_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Fetch tags
    let tags = sqlx::query_as!(
        TagResponse,
        r#"
        SELECT t.id, t.name, t.slug
        FROM tags t
        JOIN post_tags pt ON t.id = pt.tag_id
        WHERE pt.post_id = $1
        "#,
        post_id
    )
    .fetch_all(pool)
    .await?;

    // Fetch media
    let media = sqlx::query_as!(
        MediaResponse,
        r#"
        SELECT id, media_type, file_path, mime_type, width, height
        FROM post_media
        WHERE post_id = $1
        ORDER BY display_order
        "#,
        post_id
    )
    .fetch_all(pool)
    .await?;

    Ok(PostResponse {
        id: post.id,
        user_id: post.user_id,
        username: post.username,
        avatar_url: post.avatar_url,
        title: post.title,
        content_rendered_html: post.content_rendered_html,
        slug: post.slug,
        is_spam: post.is_spam,
        tags,
        media,
        echo_count: post.echo_count.unwrap_or(0),
        has_echoed: post.has_echoed.unwrap_or(false),
        refract_count: post.refract_count.unwrap_or(0),
        comment_count: post.comment_count.unwrap_or(0),
        created_at: post.created_at,
        edited_at: post.edited_at,
    })
}

async fn fetch_post_by_slug(pool: &PgPool, slug: &str, current_user_id: Option<i32>) -> Result<PostResponse> {
    let post = sqlx::query_as!(
        PostRecord,
        r#"
        SELECT p.id, p.user_id, p.title, p.content_rendered_html, p.slug, p.is_spam,
               p.echo_count, p.refract_count, p.comment_count,
               p.created_at, p.edited_at, u.username, u.avatar_url,
               COALESCE((SELECT true FROM echos WHERE post_id = p.id AND user_id = $2), false) as "has_echoed!"
        FROM posts p
        JOIN users u ON p.user_id = u.id
        WHERE p.slug = $1 AND p.deleted_at IS NULL AND p.is_spam = false
        "#,
        slug,
        current_user_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Fetch tags
    let tags = sqlx::query_as!(
        TagResponse,
        r#"
        SELECT t.id, t.name, t.slug
        FROM tags t
        JOIN post_tags pt ON t.id = pt.tag_id
        WHERE pt.post_id = $1
        "#,
        post.id
    )
    .fetch_all(pool)
    .await?;

    // Fetch media
    let media = sqlx::query_as!(
        MediaResponse,
        r#"
        SELECT id, media_type, file_path, mime_type, width, height
        FROM post_media
        WHERE post_id = $1
        ORDER BY display_order
        "#,
        post.id
    )
    .fetch_all(pool)
    .await?;

    Ok(PostResponse {
        id: post.id,
        user_id: post.user_id,
        username: post.username,
        avatar_url: post.avatar_url,
        title: post.title,
        content_rendered_html: post.content_rendered_html,
        slug: post.slug,
        is_spam: post.is_spam,
        tags,
        media,
        echo_count: post.echo_count.unwrap_or(0),
        has_echoed: post.has_echoed.unwrap_or(false),
        refract_count: post.refract_count.unwrap_or(0),
        comment_count: post.comment_count.unwrap_or(0),
        created_at: post.created_at,
        edited_at: post.edited_at,
    })
}

async fn fetch_tags_for_posts(
    pool: &PgPool,
    post_ids: &[i32],
) -> Result<std::collections::HashMap<i32, Vec<TagResponse>>> {
    if post_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let tags = sqlx::query!(
        r#"
        SELECT pt.post_id, t.id, t.name, t.slug
        FROM post_tags pt
        JOIN tags t ON pt.tag_id = t.id
        WHERE pt.post_id = ANY($1)
        "#,
        post_ids
    )
    .fetch_all(pool)
    .await?;

    let mut map = std::collections::HashMap::new();
    for tag in tags {
        map.entry(tag.post_id)
            .or_insert_with(Vec::new)
            .push(TagResponse {
                id: tag.id,
                name: tag.name,
                slug: tag.slug,
            });
    }

    Ok(map)
}

async fn fetch_media_for_posts(
    pool: &PgPool,
    post_ids: &[i32],
) -> Result<std::collections::HashMap<i32, Vec<MediaResponse>>> {
    if post_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let media = sqlx::query!(
        r#"
        SELECT post_id, id, media_type, file_path, mime_type, width, height
        FROM post_media
        WHERE post_id = ANY($1)
        ORDER BY post_id, display_order
        "#,
        post_ids
    )
    .fetch_all(pool)
    .await?;

    let mut map = std::collections::HashMap::new();
    for item in media {
        map.entry(item.post_id)
            .or_insert_with(Vec::new)
            .push(MediaResponse {
                id: item.id,
                media_type: item.media_type,
                file_path: item.file_path,
                mime_type: item.mime_type,
                width: item.width,
                height: item.height,
            });
    }

    Ok(map)
}

async fn get_username(pool: &PgPool, user_id: i32) -> Result<String> {
    let user = sqlx::query!("SELECT username FROM users WHERE id = $1", user_id)
        .fetch_one(pool)
        .await?;

    Ok(user.username)
}

fn parse_cursor(cursor: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(cursor, &time::format_description::well_known::Rfc3339)
        .map_err(|_| ValidationError::InvalidCursor("Invalid cursor format".to_string()).into())
}