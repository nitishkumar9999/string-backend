use axum::{
    Json, extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;

use crate::errors::{AppError, Result, ValidationError};
use crate::middleware::{
    auth::AuthUser,
    csrf::validate_token,
};
use crate::utils::rate_limit::{RateLimitAction, RateLimiter};
use crate::state::AppState;
use crate::utils::validation::{ContentValidator, generate_content_hash};
use crate::markdown::parse_markdown;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateRefractRequest {
    pub original_post_id: i32,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRefractRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct RefractResponse {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub content_rendered_html: String,
    pub echo_count: i32,
    pub created_at: String,
    pub edited_at: Option<String>,
    pub is_deleted: bool,
    pub is_spam: bool,
    pub original_post: OriginalPostInRefract,
}

#[derive(Debug, Serialize)]
pub struct OriginalPostInRefract {
    pub id: i32,
    pub slug: String,
    pub user_id: i32,
    pub username: String,
    pub title: Option<String>,
    pub content_rendered_html: String,
    pub tags: Vec<TagResponse>,
    pub created_at: String,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TagResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct GetRefractsQuery {
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

/// POST /api/refracts - Create or update refract (upsert)
pub async fn create_refract(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Json(req): Json<CreateRefractRequest>,
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
        .check_user_limit(auth_user.user_id, RateLimitAction::Refract)
        .await?;

    // Validate content
    ContentValidator::validate_refract(&req.content)?;

    // Check if original post exists and not deleted
    let original_post = sqlx::query!(
        r#"
        SELECT id, user_id FROM posts 
        WHERE id = $1 AND deleted_at IS NULL AND is_spam = false
        "#,
        req.original_post_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Prevent refractin own post
    if original_post.user_id == auth_user.user_id {
        return Err(AppError::CannotActOnOwnContent);
    }

    // Check for duplicate content (same user, last 24 hours)
    let content_hash = generate_content_hash(&req.content);
    check_duplicate_refract(&state.pool, auth_user.user_id, &content_hash).await?;

    // Parse markdown
    let content_rendered_html = parse_markdown(&req.content);

    // Check if user already refracted this post (unique constraint)
    let existing = sqlx::query!(
        "SELECT id, content_hash FROM refracts WHERE user_id = $1 AND original_post_id = $2 AND deleted_at IS NULL",
        auth_user.user_id,
        req.original_post_id
    )
    .fetch_optional(&state.pool)
    .await?;

    let refract_id = if let Some(existing_refract) = existing {
        // User already refracted this post - update it
        // Only set edited_at if content actually changed
        let now = OffsetDateTime::now_utc();
        let edited_at = if existing_refract.content_hash.as_deref() != Some(&content_hash) {
            Some(now)
        } else {
            None
        };

        sqlx::query!(
            r#"
            UPDATE refracts
            SET content_raw = $1,
                content_rendered_html = $2,
                content_hash = $3,
                edited_at = COALESCE($4, edited_at)
            WHERE id = $5
            RETURNING id
            "#,
            req.content,
            content_rendered_html,
            content_hash,
            edited_at,
            existing_refract.id
        )
        .fetch_one(&state.pool)
        .await?
        .id
    } else {
        // Create new refract
        sqlx::query!(
            r#"
            INSERT INTO refracts (user_id, original_post_id, content_raw, content_rendered_html, content_hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            auth_user.user_id,
            req.original_post_id,
            req.content,
            content_rendered_html,
            content_hash,
        )
        .fetch_one(&state.pool)
        .await?
        .id
    };

    // Fetch complete refract
    let refract = fetch_refract(&state.pool, refract_id).await?;

    tracing::info!(
        user_id = auth_user.user_id,
        refract_id,
        original_post_id = req.original_post_id,
        "Refract created"
    );

    Ok(StatusCode::CREATED.into_response())
}

/// GET /api/refracts/:id - Get single refract with full original post
pub async fn get_refract(
    State(state): State<AppState>,
    Path(refract_id): Path<i32>,
) -> Result<Response> {
    let refract = fetch_refract(&state.pool, refract_id).await?;
    Ok(Json(refract).into_response())
}

/// PATCH /api/refracts/:id - Update refract content
pub async fn update_refract(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(refract_id): Path<i32>,
    Json(req): Json<UpdateRefractRequest>,
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
    let refract = verify_refract_ownership(&state.pool, refract_id, auth_user.user_id).await?;

    // Validate content
    ContentValidator::validate_refract(&req.content)?;

    // Parse markdown
    let content_rendered_html = parse_markdown(&req.content);

    // Generate new hash
    let content_hash = generate_content_hash(&req.content);

    // Only set edited_at if content actually changed
    let now = OffsetDateTime::now_utc();
    let edited_at = if refract.content_hash.as_deref() != Some(&content_hash) {
        Some(now)
    } else {
        None
    };

    // Update refract
    sqlx::query!(
        r#"
        UPDATE refracts
        SET content_raw = $1,
            content_rendered_html = $2,
            content_hash = $3,
            edited_at = COALESCE($4, edited_at)
        WHERE id = $5 AND user_id = $6
        "#,
        req.content,
        content_rendered_html,
        content_hash,
        edited_at,
        refract_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;

    // Fetch updated refract
    let updated = fetch_refract(&state.pool, refract_id).await?;

    tracing::info!(
        user_id = auth_user.user_id,
        refract_id, 
        "Refract updated"
    );

    Ok(Json(updated).into_response())
}

/// DELETE /api/refracts/:id - Soft delete refract
pub async fn delete_refract(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(refract_id): Path<i32>,
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
    verify_refract_ownership(&state.pool, refract_id, auth_user.user_id).await?;

    // Soft delete
    let now = OffsetDateTime::now_utc();
    let result = sqlx::query!(
        "UPDATE refracts SET deleted_at = $1 WHERE id = $2 AND user_id = $3",
        now,
        refract_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::OK)
}

/// GET /api/posts/:id/refracts - Get all refracts of a specific post
pub async fn get_post_refracts(
    State(state): State<AppState>,
    Path(post_id): Path<i32>,
    Query(query): Query<GetRefractsQuery>,
) -> Result<Response> {
    // Verify post exists
    let post_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM posts WHERE id = $1 AND deleted_at IS NULL AND is_spam = false) as \"exists!\"",
        post_id
    )
    .fetch_one(&state.pool)
    .await?
    .exists;

    if !post_exists {
        return Err(AppError::NotFound);
    }

    let limit = query.limit.clamp(1, 50);
    let cursor_time = if let Some(cursor) = query.cursor {
        Some(parse_cursor(&cursor)?)
    } else {
        None
    };

    // Fetch refracts
    let refract_ids = if let Some(cursor) = cursor_time {
        sqlx::query_as!(
            RefractIdRecord,
            r#"
            SELECT id
            FROM refracts
            WHERE original_post_id = $1 
              AND deleted_at IS NULL
              AND is_spam = false
              AND created_at < $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
            post_id,
            cursor,
            limit as i64,
        )
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as!(
            RefractIdRecord,
            r#"
            SELECT id
            FROM refracts
            WHERE original_post_id = $1 
              AND deleted_at IS NULL
              AND is_spam = false
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            post_id,
            limit as i64,
        )
        .fetch_all(&state.pool)
        .await?
    };

    // Fetch full refract details
    let mut refracts = Vec::new();
    for record in refract_ids {
        if let Ok(refract) = fetch_refract(&state.pool, record.id).await {
            refracts.push(refract);
        }
    }

    Ok(Json(refracts).into_response())
}

// ============================================================================
// Helper Functions
// ============================================================================

struct RefractRecord {
    id: i32,
    user_id: i32,
    username: String,
    original_post_id: i32,
    content_rendered_html: String,
    echo_count: Option<i32>,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
    is_spam: Option<bool>,
}

struct RefractOwnership {
    content_hash: Option<String>,
}

struct RefractIdRecord {
    id: i32,
}

struct OriginalPostRecord {
    id: i32,
    slug: String,
    user_id: i32,
    username: String,
    title: Option<String>,
    content_rendered_html: String,
    created_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
}

async fn check_duplicate_refract(
    pool: &PgPool,
    user_id: i32,
    content_hash: &str,
) -> Result<()> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM refracts
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

async fn verify_refract_ownership(
    pool: &PgPool,
    refract_id: i32,
    user_id: i32,
) -> Result<RefractOwnership> {
    let refract = sqlx::query_as!(
        RefractOwnership,
        r#"
        SELECT content_hash
        FROM refracts
        WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
        "#,
        refract_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    match refract {
        None => Err(AppError::NotFound),
        Some(r) => Ok(r),
    }
}

async fn fetch_refract(pool: &PgPool, refract_id: i32) -> Result<RefractResponse> {
    // Fetch refract
    let refract = sqlx::query_as!(
        RefractRecord,
        r#"
        SELECT 
            r.id, r.user_id, r.original_post_id, r.content_rendered_html,
            r.echo_count, r.created_at, r.edited_at, r.deleted_at, r.is_spam,
            u.username
        FROM refracts r
        JOIN users u ON r.user_id = u.id
        WHERE r.id = $1 AND r.is_spam = false
        "#,
        refract_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Fetch original post with full details
    let original_post = fetch_original_post(pool, refract.original_post_id).await?;

    let is_deleted = refract.deleted_at.is_some();

    Ok(RefractResponse {
        id: refract.id,
        user_id: refract.user_id,
        username: if is_deleted {
            "[deleted]".to_string()
        } else {
            refract.username
        },
        content_rendered_html: if is_deleted {
            "[deleted]".to_string()
        } else {
            refract.content_rendered_html
        },
        echo_count: refract.echo_count.unwrap_or(0),
        created_at: refract.created_at.to_string(),
        edited_at: refract.edited_at.map(|dt| dt.to_string()),
        is_deleted,
        is_spam: refract.is_spam.unwrap_or(false),
        original_post,
    })
}

async fn fetch_original_post(pool: &PgPool, post_id: i32) -> Result<OriginalPostInRefract> {
    // Fetch post
    let post = sqlx::query_as!(
        OriginalPostRecord,
        r#"
        SELECT 
            p.id, p.slug, p.user_id, p.title, p.content_rendered_html,
            p.created_at, p.deleted_at,
            u.username
        FROM posts p
        JOIN users u ON p.user_id = u.id
        WHERE p.id = $1
        "#,
        post_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let is_deleted = post.deleted_at.is_some();

    // Fetch tags
    let tags = if !is_deleted {
        sqlx::query_as!(
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
        .await?
    } else {
        Vec::new()
    };

    Ok(OriginalPostInRefract {
        id: post.id,
        slug: if is_deleted {
            "[deleted]".to_string()
        } else {
            post.slug
        },
        user_id: post.user_id,
        username: if is_deleted {
            "[deleted]".to_string()
        } else {
            post.username
        },
        title: if is_deleted {
            Some("[deleted]".to_string())
        } else {
            post.title
        },
        content_rendered_html: if is_deleted {
            "[This post has been deleted]".to_string()
        } else {
            post.content_rendered_html
        },
        tags,
        created_at: post.created_at.to_string(),
        is_deleted,
    })
}

fn parse_cursor(cursor: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(cursor, &time::format_description::well_known::Rfc3339)
        .map_err(|_| ValidationError::InvalidCursor("Invalid cursor format".to_string()).into())
}