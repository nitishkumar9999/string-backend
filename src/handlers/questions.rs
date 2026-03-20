use axum::{
    Json, Form, extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, response::{Html, IntoResponse, Response}
};
use base64::prelude::BASE64_STANDARD;
use maud::Markup;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;
use base64::prelude::*;
use image::GenericImageView;
use tracing;

use crate::{errors::{AppError, Result, ValidationError}, handlers::answers::AnswerResponse};
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
use crate::dto::media::{MediaUpload, MediaResponse, TagResponse};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateQuestionRequest {
    pub title: String,
    pub content: String,
    pub tags: String,
    #[serde(default)]
    pub media: Vec<MediaUpload>,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateQuestionRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct QuestionResponse {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub avatar_url: Option<String>,
    pub title: String,
    pub content_rendered_html: String,
    pub slug: String,
    pub is_spam: bool,
    pub tags: Vec<TagResponse>,
    pub media: Vec<MediaResponse>,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub answer_count: i32,
    pub comment_count: i32,
    pub created_at: OffsetDateTime,
    pub edited_at: Option<OffsetDateTime>,
    // Top 3 answers included when fetching single question
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_answers: Option<Vec<AnswerResponse>>,
}

#[derive(Debug, Deserialize)]
pub struct GetAnswersQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    pub cursor: Option<i32>,
}

fn default_limit() -> i32 {
    20
}

// ============================================================================
// Question Handlers
// ============================================================================

/// POST /api/questions - Create a new question
pub async fn create_question(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    Form(req): Form<CreateQuestionRequest>,
) -> Result<Response> {

    if !validate_token(&state, &req.csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    // Rate limit check
    rate_limiter
        .check_user_limit(auth_user.user_id, RateLimitAction::Question)
        .await?;

    // Validate title (required for questions)
    ContentValidator::validate_title(&req.title)?;

    // Validate content
    ContentValidator::validate_question(&req.content, &req.title)?;

    let tag_list: Vec<String> = req.tags
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();

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
    check_duplicate_question(&state.pool, auth_user.user_id, &content_hash).await?;

    // Parse markdown to HTML
    let content_rendered_html = parse_markdown(&req.content);

    // Generate temporary slug (before we have question ID)
    let temp_slug = generate_post_slug(
        Some(&req.title),
        validated_tags.first().map(|s| s.as_str())
    );

    // Start transaction
    let mut tx = state.pool.begin().await?;

    // Insert question with temporary slug
    let question = sqlx::query!(
        r#"
        INSERT INTO questions (user_id, title, content_raw, content_rendered_html, content_hash, slug)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, title, content_rendered_html, 
                  echo_count, answer_count, comment_count, 
                  created_at, edited_at
        "#,
        auth_user.user_id,
        req.title,
        req.content,
        content_rendered_html,
        content_hash,
        temp_slug,
    )
    .fetch_one(&mut *tx)
    .await?;

    // Update slug with final version including question ID
    let final_slug = format!("{}-{}", temp_slug, question.id);
    sqlx::query!(
        "UPDATE questions SET slug = $1 WHERE id = $2",
        final_slug,
        question.id
    )
    .execute(&mut *tx)
    .await?;

    // Handle tags
    let tag_responses = insert_question_tags(&mut tx, question.id, &validated_tags, auth_user.user_id).await?;

    // Apply tag creation rate limit for new tags
    let new_tags_count = tag_responses.iter().filter(|t| t.is_new).count();
    if new_tags_count > 0 {
        rate_limiter
            .check_user_limit(auth_user.user_id, RateLimitAction::TagCreation)
            .await?;
    }

    // Handle media uploads
    let media_responses = if !validated_media.is_empty() {
        insert_question_media(&mut tx, question.id, validated_media.clone()).await?
    } else {
        Vec::new()
    };

    // Commit transaction
    tx.commit().await?;

    let user = sqlx::query!(
        "SELECT username, avatar_url FROM users WHERE id = $1",
        auth_user.user_id
    )
    .fetch_one(&state.pool)
    .await?;

    // Build response
    let response = QuestionResponse {
        id: question.id,
        user_id: question.user_id,
        username: user.username,
        avatar_url: user.avatar_url,
        title: question.title,
        content_rendered_html: question.content_rendered_html,
        slug: final_slug.clone(),
        is_spam: false,
        tags: tag_responses.into_iter().map(|t| t.tag).collect(),
        media: media_responses,
        echo_count: question.echo_count.unwrap_or(0),
        has_echoed: false,
        answer_count: question.answer_count.unwrap_or(0),
        comment_count: question.comment_count.unwrap_or(0),
        created_at: question.created_at,
        edited_at: question.edited_at,
        top_answers: None,
    };

    tracing::info!(
        user_id = auth_user.user_id,
        question_id = question.id,
        slug = %final_slug,
        tags = ?validated_tags,
        has_media = !validated_media.is_empty(),
        "Question created"
    );

    Ok((
        StatusCode::SEE_OTHER,
        [("Location", format!("/questions/{}", final_slug))]
    ).into_response())
}

pub async fn render_create_question_page_handler(
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

    let markup = crate::templates::create::render_create_question_page(csrf_token, user_info);
    Ok(Html(markup.into_string()))
}

/// GET /api/questions/:slug - Get single question with top 3 answers
pub async fn get_question_by_slug(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Html<String>> {

    let back_url = headers
        .get("Referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/")
        .to_string();

    let current_user_id = auth_user.as_ref().map(|a| a.user_id);
    let mut question = fetch_question_by_slug(&state.pool, &slug, current_user_id).await?;
    
    // Fetch top 3 answers by echo_count DESC
    let top_answers = fetch_top_answers(&state.pool, question.id, 3, current_user_id).await?;
    question.top_answers = Some(top_answers);

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

    let current_user_id = auth_user.as_ref().map(|a| a.user_id);

    let answers = fetch_top_answers(&state.pool, question.id, 3, current_user_id).await?;
    let total_answer_count = question.answer_count;
    let answer_has_more = total_answer_count > 3;
    let answer_next_cursor = if answer_has_more {
        Some(3)
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
        None, 
        Some(question.id), 
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
    let total_count = question.comment_count;

    let markup = crate::templates::posts::render_question_page(
        question, 
        user, 
        csrf_token.clone(), 
        comments, 
        total_count, 
        has_more, 
        next_cursor, 
        answers,
        total_answer_count, 
        answer_has_more, 
        answer_next_cursor,
        back_url,
    );

    Ok(Html(markup.into_string()))
}

/// PATCH /api/questions/:id - Update question
pub async fn update_question(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(question_id): Path<i32>,
    Json(req): Json<UpdateQuestionRequest>,
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
    verify_question_ownership(&state.pool, question_id, auth_user.user_id).await?;

    // Build update query dynamically
    let mut update_fields = Vec::new();
    let mut updated_html: Option<String> = None;

    // Validate and update title
    if let Some(ref title) = req.title {
        ContentValidator::validate_title(title)?;
        update_fields.push("title");
    }

    // Validate and update content
    if let Some(ref content) = req.content {
        let title = if let Some(ref t) = req.title {
            t.clone()
        } else {
            // Get existing title for validation
            let existing = sqlx::query!("SELECT title FROM questions WHERE id = $1", question_id)
                .fetch_one(&state.pool)
                .await?;
            existing.title
        };
        
        ContentValidator::validate_question(content, &title)?;
        updated_html = Some(parse_markdown(content));
        update_fields.push("content_raw");
        update_fields.push("content_rendered_html");
    }

    // Update question
    if !update_fields.is_empty() {
        let now = OffsetDateTime::now_utc();
        
        sqlx::query!(
            r#"
            UPDATE questions
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
            question_id,
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
            "DELETE FROM question_tags WHERE question_id = $1",
            question_id
        )
        .execute(&mut *tx)
        .await?;

        // Insert new tags
        insert_question_tags(&mut tx, question_id, &validated_tags, auth_user.user_id).await?;

        tx.commit().await?;
    }

    let current_user_id = Some(auth_user.user_id);
    // Fetch updated question
    let question = fetch_question(&state.pool, question_id, current_user_id).await?;

    tracing::info!(
        user_id = auth_user.user_id,
        question_id,
        "Question updated"
    );

    Ok(Json(question).into_response())
}

/// DELETE /api/questions/:id - Soft delete question
pub async fn delete_question(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(question_id): Path<i32>,
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
    verify_question_ownership(&state.pool, question_id, auth_user.user_id).await?;

    // Soft delete
    let now = OffsetDateTime::now_utc();
    let result = sqlx::query!(
        "UPDATE questions SET deleted_at = $1 WHERE id = $2 AND user_id = $3",
        now,
        question_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    tracing::info!(
        user_id = auth_user.user_id,
        question_id,
        "Question deleted"
    );

    Ok(StatusCode::OK)
}

/// GET /api/questions/:id/answers - Lazy load remaining answers (after top 3)
pub async fn get_question_answers(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Path(slug): Path<String>,
    Query(query): Query<GetAnswersQuery>,
) -> Result<Response> {
    // Verify question exists

    let question = sqlx::query!(
        "SELECT id, slug FROM questions WHERE slug = $1 AND deleted_at IS NULL AND is_spam = false",
        slug,
        
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let question_id = question.id;
    let question_slug = question.slug;


    let limit = query.limit.clamp(1, 50);

    let current_user_id = auth_user.as_ref().map(|a| a.user_id);
    // Fetch answers (skipping top 3 if no cursor - those are in question response)
    let answers = fetch_answers_paginated(
        &state.pool, 
        question_id, 
        11, 
        query.cursor,
        current_user_id,
    ).await?;

    let has_more = answers.len() > 10;
    let answers: Vec<_> =answers.into_iter().take(10).collect();
    let next_cursor = if has_more {
        let current_offset = query.cursor.unwrap_or(3);
        Some(current_offset + 10)
    } else {
        None
    };

    let current_user = if let Some(ref auth) = auth_user {
        let u = sqlx::query!(
            "SELECT username, avatar_url FROM users WHERE id = $1",
            auth.user_id
        )
        .fetch_optional(&state.pool)
        .await?;
        u.map(|u| (auth.user_id, u.username, u.avatar_url))
    } else {
        None
    };

    let csrf_token = if let Some(ref auth) = auth_user {
        Some(generate_token(&state, auth.session_id).await.unwrap_or_default())
    } else {
        None
    };

    let html = crate::templates::answer::render_answer_list_fragment(
        &answers,  // Vec is fine for list fragment
        &question_slug,
        current_user.as_ref(),
        csrf_token.as_deref(),
        has_more,
        next_cursor,
        question_id,
    );

    Ok(Html(html.into_string()).into_response())
}

// ============================================================================
// Helper Functions - Questions
// ============================================================================

struct QuestionRecord {
    id: i32,
    user_id: i32,
    username: String,
    avatar_url: Option<String>,
    title: String,
    content_rendered_html: String,
    slug: String,
    is_spam: Option<bool>,
    echo_count: Option<i32>,
    has_echoed: Option<bool>,
    answer_count: Option<i32>,
    comment_count: Option<i32>,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
struct ValidatedMedia {
    data: Vec<u8>,
    filename: String,
    mime_type: String,
    media_type: String,
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

async fn check_duplicate_question(
    pool: &PgPool,
    user_id: i32,
    content_hash: &str,
) -> Result<()> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM questions
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

async fn insert_question_tags(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    question_id: i32,
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

        // Insert into question_tags
        sqlx::query!(
            "INSERT INTO question_tags (question_id, tag_id) VALUES ($1, $2)",
            question_id,
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

async fn insert_question_media(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    question_id: i32,
    media: Vec<ValidatedMedia>,
) -> Result<Vec<MediaResponse>> {
    let mut responses = Vec::new();

    for (index, item) in media.into_iter().enumerate() {
        // TODO: Upload file to storage
        let file_path = format!("/uploads/questions/{}/{}_{}", question_id, index, item.filename);

        let record = sqlx::query!(
            r#"
            INSERT INTO question_media 
            (question_id, media_type, file_path, original_filename, mime_type, 
             file_size, width, height, display_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, media_type, file_path, mime_type, width, height
            "#,
            question_id,
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

async fn verify_question_ownership(pool: &PgPool, question_id: i32, user_id: i32) -> Result<()> {
    let question = sqlx::query!(
        "SELECT user_id FROM questions WHERE id = $1 AND deleted_at IS NULL",
        question_id
    )
    .fetch_optional(pool)
    .await?;

    match question {
        None => Err(AppError::NotFound),
        Some(q) if q.user_id != user_id => Err(AppError::Forbidden),
        Some(_) => Ok(()),
    }
}

pub async fn fetch_question(pool: &PgPool, question_id: i32, current_user_id: Option<i32>) -> Result<QuestionResponse> {
    let question = sqlx::query_as!(
        QuestionRecord,
        r#"
        SELECT q.id, q.user_id, q.title, q.content_rendered_html, q.slug, q.is_spam,
               q.echo_count, q.answer_count, q.comment_count,
               q.created_at, q.edited_at, u.username, u.avatar_url,
               COALESCE((SELECT true FROM echos WHERE question_id = q.id AND user_id = $2), false) as "has_echoed!" 
        FROM questions q
        JOIN users u ON q.user_id = u.id
        WHERE q.id = $1 AND q.deleted_at IS NULL AND q.is_spam = false
        "#,
        question_id,
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
        JOIN question_tags qt ON t.id = qt.tag_id
        WHERE qt.question_id = $1
        "#,
        question_id
    )
    .fetch_all(pool)
    .await?;

    // Fetch media
    let media = sqlx::query_as!(
        MediaResponse,
        r#"
        SELECT id, media_type, file_path, mime_type, width, height
        FROM question_media
        WHERE question_id = $1
        ORDER BY display_order
        "#,
        question_id
    )
    .fetch_all(pool)
    .await?;

    Ok(QuestionResponse {
        id: question.id,
        user_id: question.user_id,
        username: question.username,
        avatar_url: question.avatar_url,
        title: question.title,
        content_rendered_html: question.content_rendered_html,
        slug: question.slug,
        is_spam: question.is_spam.unwrap_or(false),
        tags,
        media,
        echo_count: question.echo_count.unwrap_or(0),
        has_echoed: question.has_echoed.unwrap_or(false),
        answer_count: question.answer_count.unwrap_or(0),
        comment_count: question.comment_count.unwrap_or(0),
        created_at: question.created_at,
        edited_at: question.edited_at,
        top_answers: None,
    })
}

async fn fetch_question_by_slug(pool: &PgPool, slug: &str, current_user_id: Option<i32>) -> Result<QuestionResponse> {
    let question = sqlx::query_as!(
        QuestionRecord,
        r#"
        SELECT q.id, q.user_id, q.title, q.content_rendered_html, q.slug, q.is_spam,
               q.echo_count, q.answer_count, q.comment_count,
               q.created_at, q.edited_at, u.username, u.avatar_url,
               COALESCE((SELECT true FROM echos WHERE question_id = q.id AND user_id = $2), false) as "has_echoed!"
        FROM questions q
        JOIN users u ON q.user_id = u.id
        WHERE q.slug = $1 AND q.deleted_at IS NULL AND q.is_spam = false
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
        JOIN question_tags qt ON t.id = qt.tag_id
        WHERE qt.question_id = $1
        "#,
        question.id
    )
    .fetch_all(pool)
    .await?;

    // Fetch media
    let media = sqlx::query_as!(
        MediaResponse,
        r#"
        SELECT id, media_type, file_path, mime_type, width, height
        FROM question_media
        WHERE question_id = $1
        ORDER BY display_order
        "#,
        question.id
    )
    .fetch_all(pool)
    .await?;

    Ok(QuestionResponse {
        id: question.id,
        user_id: question.user_id,
        username: question.username,
        avatar_url: question.avatar_url,
        title: question.title,
        content_rendered_html: question.content_rendered_html,
        slug: question.slug,
        is_spam: question.is_spam.unwrap_or(false),
        tags,
        media,
        echo_count: question.echo_count.unwrap_or(0),
        has_echoed: question.has_echoed.unwrap_or(false),
        answer_count: question.answer_count.unwrap_or(0),
        comment_count: question.comment_count.unwrap_or(0),
        created_at: question.created_at,
        edited_at: question.edited_at,
        top_answers: None,
    })
}

async fn fetch_top_answers(pool: &PgPool, question_id: i32, limit: i32, current_user_id: Option<i32>) -> Result<Vec<AnswerResponse>> {
    let answer_ids = sqlx::query!(
        r#"
        SELECT id
        FROM answers
        WHERE question_id = $1 AND deleted_at IS NULL AND is_spam = false
        ORDER BY echo_count DESC, created_at ASC
        LIMIT $2
        "#,
        question_id,
        limit as i64,
    )
    .fetch_all(pool)
    .await?;

    let mut answers = Vec::new();
    for record in answer_ids {
        if let Ok(answer) = fetch_answer(pool, record.id, current_user_id).await {
            answers.push(answer);
        }
    }

    Ok(answers)
}

pub async fn fetch_answers_paginated(
    pool: &PgPool,
    question_id: i32,
    limit: i32,
    cursor: Option<i32>,
    current_user_id: Option<i32>,
) -> Result<Vec<AnswerResponse>> {
    let answer_ids = if let Some(cursor_offset) = cursor {
        sqlx::query_as!(
            AnswerIdRecord,
            r#"
            SELECT id
            FROM answers
            WHERE question_id = $1 
              AND deleted_at IS NULL 
              AND is_spam = false
            ORDER BY echo_count DESC, created_at ASC
            LIMIT $2 OFFSET $3
            "#,
            question_id,
            limit as i64,
            cursor_offset as i64,
        )
        .fetch_all(pool)
        .await?
    } else {
        // Skip top 3 when no cursor (those are already in question response)
        sqlx::query_as!(
            AnswerIdRecord,
            r#"
            SELECT id
            FROM answers
            WHERE question_id = $1 AND deleted_at IS NULL AND is_spam = false
            ORDER BY echo_count DESC, created_at ASC
            LIMIT $2 OFFSET 3
            "#,
            question_id,
            limit as i64,
        )
        .fetch_all(pool)
        .await?
    };

    let mut answers = Vec::new();
    for record in answer_ids {
        if let Ok(answer) = fetch_answer(pool, record.id, current_user_id).await {
            answers.push(answer);
        }
    }

    Ok(answers)
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

// ============================================================================
// Answer Helper Functions (used by both question and answer handlers)
// ============================================================================

struct AnswerRecord {
    id: i32,
    question_id: i32,
    avatar_url: Option<String>,
    user_id: i32,
    username: String,
    content_rendered_html: String,
    slug: String,
    is_spam: Option<bool>,
    echo_count: Option<i32>,
    has_echoed: Option<bool>,
    comment_count: Option<i32>,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
}

struct AnswerIdRecord {
    id: i32,
}

async fn fetch_answer(pool: &PgPool, answer_id: i32, current_user_id: Option<i32>) -> Result<AnswerResponse> {
    let answer = sqlx::query_as!(
        AnswerRecord,
        r#"
        SELECT a.id, a.question_id, a.user_id, a.content_rendered_html, 
               a.slug, a.is_spam, a.echo_count, a.comment_count,
               a.created_at, a.edited_at, u.username, u.avatar_url,
               COALESCE((SELECT true FROM echos WHERE answer_id = a.id AND user_id = $2), false) as "has_echoed!"
        FROM answers a
        JOIN users u ON a.user_id = u.id
        WHERE a.id = $1 AND a.deleted_at IS NULL AND a.is_spam = false
        "#,
        answer_id,
        current_user_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Fetch media
    let media = sqlx::query_as!(
        MediaResponse,
        r#"
        SELECT id, media_type, file_path, mime_type, width, height
        FROM answer_media
        WHERE answer_id = $1
        ORDER BY display_order
        "#,
        answer_id
    )
    .fetch_all(pool)
    .await?;

    Ok(AnswerResponse {
        id: answer.id,
        question_id: answer.question_id,
        avatar_url: answer.avatar_url,
        user_id: answer.user_id,
        username: answer.username,
        content_rendered_html: answer.content_rendered_html,
        slug: answer.slug,
        is_spam: answer.is_spam.unwrap_or(false),
        media,
        echo_count: answer.echo_count.unwrap_or(0),
        has_echoed: answer.has_echoed.unwrap_or(false),
        comment_count: answer.comment_count.unwrap_or(0),
        created_at: answer.created_at,
        edited_at: answer.edited_at,
    })
}