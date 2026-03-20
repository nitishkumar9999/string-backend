use axum::{
    Form, Json, extract::{Path, State}, http::{HeaderMap, StatusCode}, response::{Html, IntoResponse, Response}
};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;
use image::GenericImageView;
use tracing;

use crate::errors::{AppError, Result, ValidationError};
use crate::middleware::{
    auth::AuthUser,
    csrf::{validate_token, generate_token},
};
use crate::utils::rate_limit::{RateLimitAction, RateLimiter};
use crate::state::AppState;
use crate::utils::validation::{
    ContentValidator, MediaValidator, generate_content_hash, 
    ALLOWED_IMAGE_TYPES, ALLOWED_VIDEO_TYPES,
};
use crate::markdown::parse_markdown;
use crate::dto::media::{MediaResponse, MediaUpload};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAnswerRequest {
    pub content: String,
    #[serde(default)]
    pub media: Vec<MediaUpload>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAnswerRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct AnswerResponse {
    pub id: i32,
    pub question_id: i32,
    pub avatar_url: Option<String>,
    pub user_id: i32,
    pub username: String,
    pub content_rendered_html: String,
    pub slug: String,
    pub is_spam: bool,
    pub media: Vec<MediaResponse>,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub comment_count: i32,
    pub created_at: OffsetDateTime,
    pub edited_at: Option<OffsetDateTime>,
}

// ============================================================================
// Answer Handlers
// ============================================================================

/// POST /api/questions/:id/answers - Create an answer to a question
pub async fn create_answer(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(question_id): Path<i32>,
    Json(req): Json<CreateAnswerRequest>,
) -> Result<Response> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    // Rate limit check
    rate_limiter
        .check_user_limit(auth_user.user_id, RateLimitAction::Answer)
        .await?;

    // Verify question exists and not deleted/spam
    let question = sqlx::query!(
        "SELECT id, slug FROM questions WHERE id = $1 AND deleted_at IS NULL AND is_spam = false",
        question_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let question_slug = question.slug;

    // Validate content
    ContentValidator::validate_answer(&req.content)?;

    // Validate media
    let mut validated_media = Vec::new();
    if !req.media.is_empty() {
        validated_media = validate_media_uploads(&req.media).await?;
    }

    // Check for duplicate content (same user, last 24 hours)
    let content_hash = generate_content_hash(&req.content);
    check_duplicate_answer(&state.pool, auth_user.user_id, &content_hash).await?;

    // Parse markdown to HTML
    let content_rendered_html = parse_markdown(&req.content);

    // Generate slug: question-slug-{answer_id}
    // We'll update it after insert to include answer ID
    let temp_slug = format!("{}-answer", question_slug);

    // Start transaction
    let mut tx = state.pool.begin().await?;

    // Insert answer
    let answer = sqlx::query!(
        r#"
        INSERT INTO answers (question_id, user_id, content_raw, content_rendered_html, content_hash, slug)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, question_id, user_id, content_rendered_html,
                  echo_count, comment_count, created_at, edited_at
        "#,
        question_id,
        auth_user.user_id,
        req.content,
        content_rendered_html,
        content_hash,
        temp_slug,
    )
    .fetch_one(&mut *tx)
    .await?;

    // Update slug with final version: question-slug-answer-{id}
    let final_slug = format!("{}-{}", question_slug, answer.id);
    sqlx::query!(
        "UPDATE answers SET slug = $1 WHERE id = $2",
        final_slug,
        answer.id
    )
    .execute(&mut *tx)
    .await?;

    // Handle media uploads
    let media_responses = if !validated_media.is_empty() {
        insert_answer_media(&mut tx, answer.id, validated_media).await?
    } else {
        Vec::new()
    };

    // Commit transaction
    tx.commit().await?;

    // Get username
    let user = sqlx::query!(
        "SELECT username, avatar_url FROM users WHERE id = $1",
        auth_user.user_id
    )
    .fetch_one(&state.pool)
    .await?;

    // Build response
    let response = AnswerResponse {
        id: answer.id,
        question_id: answer.question_id,
        user_id: answer.user_id,
        username: user.username.clone(),
        avatar_url: user.avatar_url,
        content_rendered_html: answer.content_rendered_html,
        slug: final_slug,
        is_spam: false,
        media: media_responses,
        echo_count: answer.echo_count.unwrap_or(0),
        has_echoed: false,
        comment_count: answer.comment_count.unwrap_or(0),
        created_at: answer.created_at,
        edited_at: answer.edited_at,
    };

    tracing::info!(
        user_id = auth_user.user_id,
        answer_id = answer.id,
        question_id,
        "Answer created"
    );

    let current_user = Some((auth_user.user_id, user.username, None));

    let html = crate::templates::answer::render_answer_fragment(
        &response, 
        &question_slug, 
        current_user.as_ref(), 
        Some(csrf_token), 
    );

    Ok((StatusCode::CREATED,
         Html(html.into_string())).into_response())
}

/// GET /api/answers/:slug - Get single answer (for direct sharing)
pub async fn get_answer_by_slug(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Html<String>> {

    let current_user_id = auth_user.as_ref().map(|a| a.user_id);
    let answer = fetch_answer_by_slug(&state.pool, &slug, current_user_id).await?;

    let back_url = headers
        .get("Referer")
        .and_then(|v| v.to_str().ok())
        .map(|referer| {
            let base = referer.split('#').next().unwrap_or(referer);
            format!("{}#answer-{}", referer, answer.id)
        })
        .unwrap_or("/".to_string());

    let question = crate::handlers::questions::fetch_question(
        &state.pool,
        answer.question_id,
        current_user_id
    ).await?;

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

    let csrf_token = if let Some(ref auth) = auth_user {
        generate_token(&state, auth.session_id).await.ok()
    } else {
        None
    };

    let comments = crate::handlers::comments::fetch_parent_comments(
        &state.pool,
        None,
        None,
        Some(answer.id),
        11,
        None,
        auth_user.as_ref().map(|a| a.user_id),
    )
    .await?;

    let has_more = comments.len() > 10;
    let comments: Vec<_> = comments.into_iter().take(10).collect();
    let next_cursor = comments.last().map(|c| {
        c.created_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default()
    });
    let total_comment_count = answer.comment_count;

    let markup = crate::templates::answer::render_answer_page(
        &answer,
        &question,
        comments,
        total_comment_count,
        has_more,
        next_cursor,
        user,
        csrf_token.as_deref(),
        back_url,
    );

    Ok(Html(markup.into_string()))
}

/// PATCH /api/answers/:id - Update answer
pub async fn update_answer(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(answer_id): Path<i32>,
    Json(req): Json<UpdateAnswerRequest>,
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
    verify_answer_ownership(&state.pool, answer_id, auth_user.user_id).await?;

    // Validate content
    ContentValidator::validate_answer(&req.content)?;

    // Parse markdown
    let content_rendered_html = parse_markdown(&req.content);

    // Update answer
    let now = OffsetDateTime::now_utc();
    sqlx::query!(
        r#"
        UPDATE answers
        SET content_raw = $1,
            content_rendered_html = $2,
            edited_at = $3
        WHERE id = $4 AND user_id = $5
        "#,
        req.content,
        content_rendered_html,
        now,
        answer_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;


    let current_user_id = Some(auth_user.user_id);

    // Fetch updated answer
    let updated = fetch_answer(&state.pool, answer_id, current_user_id).await?;

    tracing::info!(
        user_id = auth_user.user_id,
        answer_id,
        "Answer updated"
    );

    Ok(Json(updated).into_response())
}

/// DELETE /api/answers/:id - Soft delete answer
pub async fn delete_answer(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(answer_id): Path<i32>,
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
    verify_answer_ownership(&state.pool, answer_id, auth_user.user_id).await?;

    // Soft delete
    let now = OffsetDateTime::now_utc();
    let result = sqlx::query!(
        "UPDATE answers SET deleted_at = $1 WHERE id = $2 AND user_id = $3",
        now,
        answer_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    tracing::info!(
        user_id = auth_user.user_id,
        answer_id,
        "Answer deleted"
    );

    Ok(StatusCode::OK)
}

// ============================================================================
// Helper Functions
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

#[derive(Debug)]
struct ValidatedMedia {
    data: Vec<u8>,
    filename: String,
    mime_type: String,
    media_type: String,
    width: Option<u32>,
    height: Option<u32>,
}

async fn validate_media_uploads(uploads: &[MediaUpload]) -> Result<Vec<ValidatedMedia>> {
    let mut validated = Vec::new();
    let mut image_count = 0;
    let mut video_count = 0;

    for upload in uploads {
        let data: Vec<u8> = BASE64_STANDARD.decode(&upload.data)
            .map_err(|_| ValidationError::MissingField("Invalid base64 data".to_string()))?;

        let mime_type = MediaValidator::detect_mime_type(&data)?;

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

async fn check_duplicate_answer(
    pool: &PgPool,
    user_id: i32,
    content_hash: &str,
) -> Result<()> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM answers
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

async fn insert_answer_media(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    answer_id: i32,
    media: Vec<ValidatedMedia>,
) -> Result<Vec<MediaResponse>> {
    let mut responses = Vec::new();

    for (index, item) in media.into_iter().enumerate() {
        // TODO: Upload file to storage
        let file_path = format!("/uploads/answers/{}/{}_{}", answer_id, index, item.filename);

        let record = sqlx::query!(
            r#"
            INSERT INTO answer_media 
            (answer_id, media_type, file_path, original_filename, mime_type, 
             file_size, width, height, display_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, media_type, file_path, mime_type, width, height
            "#,
            answer_id,
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

async fn verify_answer_ownership(pool: &PgPool, answer_id: i32, user_id: i32) -> Result<()> {
    let answer = sqlx::query!(
        "SELECT user_id FROM answers WHERE id = $1 AND deleted_at IS NULL",
        answer_id
    )
    .fetch_optional(pool)
    .await?;

    match answer {
        None => Err(AppError::NotFound),
        Some(a) if a.user_id != user_id => Err(AppError::Forbidden),
        Some(_) => Ok(()),
    }
}

async fn fetch_answer(pool: &PgPool, answer_id: i32, current_user_id: Option<i32>) -> Result<AnswerResponse> {
    let answer = sqlx::query_as!(
        AnswerRecord,
        r#"
        SELECT a.id, a.question_id, a.user_id, a.content_rendered_html, 
               a.slug, a.is_spam, a.echo_count, a.comment_count,
               a.created_at, a.edited_at, u.username, u.avatar_url,
               COALESCE ((SELECT true FROM echos WHERE answer_id = a.id AND user_id = $2), false) as "has_echoed!"
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
        user_id: answer.user_id,
        username: answer.username,
        avatar_url: answer.avatar_url,
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

async fn fetch_answer_by_slug(pool: &PgPool, slug: &str, current_user_id: Option<i32>) -> Result<AnswerResponse> {
    let answer = sqlx::query_as!(
        AnswerRecord,
        r#"
        SELECT a.id, a.question_id, a.user_id, a.content_rendered_html, 
               a.slug, a.is_spam, a.echo_count, a.comment_count,
               a.created_at, a.edited_at, u.username, u.avatar_url,
               COALESCE((SELECT true FROM echos WHERE answer_id = a.id AND user_id = $2), false) as "has_echoed!"
        FROM answers a
        JOIN users u ON a.user_id = u.id
        WHERE a.slug = $1 AND a.deleted_at IS NULL AND a.is_spam = false
        "#,
        slug,
        current_user_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let media = sqlx::query_as!(
        MediaResponse,
        r#"
        SELECT id, media_type, file_path, mime_type, width, height
        FROM answer_media
        WHERE answer_id = $1
        ORDER BY display_order
        "#,
        answer.id
    )
    .fetch_all(pool)
    .await?;

    Ok(AnswerResponse {
        id: answer.id,
        question_id: answer.question_id,
        user_id: answer.user_id,
        username: answer.username,
        avatar_url: answer.avatar_url,
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

async fn get_username(pool: &PgPool, user_id: i32) -> Result<String> {
    let user = sqlx::query!("SELECT username FROM users WHERE id = $1", user_id)
        .fetch_one(pool)
        .await?;

    Ok(user.username)
}