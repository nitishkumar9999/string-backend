use axum::{
    Json, Form, extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response, Html}
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query};
use std::sync::Arc;
use time::OffsetDateTime;
use tracing;

use crate::{errors::{AppError, Result, ValidationError}, middleware::csrf::generate_token};
use crate::middleware::{
    auth::AuthUser,
    csrf::validate_token,
};
use crate::utils::rate_limit::{RateLimitAction, RateLimiter};
use crate::state::AppState;
use crate::utils::validation::{ContentValidator, generate_content_hash};
use crate::markdown::parse_markdown_with_depth;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    // Exactly one must be provided
    pub post_id: Option<i32>,
    pub question_id: Option<i32>,
    pub answer_id: Option<i32>,
    // Optional for nested comments
    pub parent_comment_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub content_rendered_html: String,
    pub depth_level: i32,
    pub helpful_count: i32,
    pub reply_count: i32,
    pub is_deleted: bool,
    pub has_marked_helpful: bool, // Current user has marked helpful
    pub created_at: OffsetDateTime,
    pub edited_at: Option<OffsetDateTime>,
    // Only for nested views
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<CommentResponse>>,
}

#[derive(Debug, Deserialize)]
pub struct GetCommentsQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    pub cursor: Option<String>,
}

fn default_limit() -> i32 {
    10
}

#[derive(Debug, Deserialize)]
pub struct GetRepliesQuery {
    pub cursor: Option<String>,
}

// ============================================================================
// Comment CRUD Handlers
// ============================================================================

/// POST /api/comments - Create a comment
pub async fn create_comment(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    headers: HeaderMap, 
    Json(req): Json<CreateCommentRequest>,
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
        .check_user_limit(auth_user.user_id, RateLimitAction::Comment)
        .await?;

    // Validate exactly one parent type
    let parent_count = [req.post_id, req.question_id, req.answer_id]
        .iter()
        .filter(|p| p.is_some())
        .count();

    if parent_count != 1 {
        return Err(ValidationError::MissingField(
            "Exactly one of post_id, question_id, or answer_id required".to_string()
        ).into());
    }

    // Determine depth level and validate parent if nested
    let (depth_level, parent_validated) = if let Some(parent_id) = req.parent_comment_id {
        let parent = validate_parent_comment(
            &state.pool,
            parent_id,
            req.post_id,
            req.question_id,
            req.answer_id,
        )
        .await?;

        let parent_depth = parent.depth_level.unwrap_or(0);

        // Check max depth
        if parent_depth >= 3 {
            return Err(ValidationError::MissingField(
                "Maximum comment depth (3) reached".to_string()
            ).into());
        }

        (parent_depth + 1, true)
    } else {
        (0, false)
    };

    // Validate content based on depth
    ContentValidator::validate_comment(&req.content, depth_level)?;

    // Check for duplicate content (same user, last 1 hour)
    let content_hash = generate_content_hash(&req.content);
    check_duplicate_comment(&state.pool, auth_user.user_id, &content_hash).await?;

    // Parse markdown based on depth
    let content_rendered_html = parse_markdown_with_depth(&req.content, depth_level);

    // Insert comment
    let comment = sqlx::query!(
        r#"
        INSERT INTO comments (
            user_id, post_id, question_id, answer_id, parent_comment_id,
            depth_level, content_raw, content_rendered_html, content_hash
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, user_id, depth_level, helpful_count, reply_count,
                  created_at, edited_at, deleted_at
        "#,
        auth_user.user_id,
        req.post_id,
        req.question_id,
        req.answer_id,
        req.parent_comment_id,
        depth_level,
        req.content,
        content_rendered_html,
        content_hash,
    )
    .fetch_one(&state.pool)
    .await?;

    // Get username
    let username = get_username(&state.pool, auth_user.user_id).await?;

    // Build response
    let response = CommentResponse {
        id: comment.id,
        user_id: comment.user_id,
        username,
        content_rendered_html,
        depth_level: comment.depth_level.unwrap_or(0),
        helpful_count: comment.helpful_count.unwrap_or(0),
        reply_count: comment.reply_count.unwrap_or(0),
        is_deleted: comment.deleted_at.is_some(),
        has_marked_helpful: false,
        created_at: comment.created_at,
        edited_at: comment.edited_at,
        replies: None,
    };

    let (parent_type, parent_id) = if let Some(id) = req.post_id {
        ("post", id)
    } else if let Some(id) = req.question_id {
        ("question", id)
    } else if let Some(id) = req.answer_id {
        ("answer", id)
    } else {
        ("post", 0)
    };
 
    let html = crate::templates::comments::render_comment_fragment(
        &response,
        parent_type,
        parent_id,
        None,
    );
    
    Ok((StatusCode::CREATED, Html(html.into_string())).into_response())
}

/// GET /api/comments/:id - Get single comment with nested replies
pub async fn get_comment(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Path(comment_id): Path<i32>,
) -> Result<Response> {
    let comment = fetch_comment_with_replies(
        &state.pool,
        comment_id,
        auth_user.as_ref().map(|u| u.user_id),
    )
    .await?;

    Ok(Json(comment).into_response())
}

/// PATCH /api/comments/:id - Update comment
pub async fn update_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(comment_id): Path<i32>,
    Json(req): Json<UpdateCommentRequest>,
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
    let comment = verify_comment_ownership(&state.pool, comment_id, auth_user.user_id).await?;
    let depth_level = comment.depth_level.unwrap_or(0);

    // Validate content based on depth
    ContentValidator::validate_comment(&req.content, depth_level)?;

    // Parse markdown based on depth
    let content_rendered_html = parse_markdown_with_depth(&req.content, depth_level);

    // Update comment
    let now = OffsetDateTime::now_utc();
    sqlx::query!(
        r#"
        UPDATE comments
        SET content_raw = $1,
            content_rendered_html = $2,
            edited_at = $3
        WHERE id = $4 AND user_id = $5
        "#,
        req.content,
        content_rendered_html,
        now,
        comment_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;

    // Fetch updated comment
    let updated = fetch_comment(&state.pool, comment_id, Some(auth_user.user_id)).await?;
    Ok(Json(updated).into_response())
}

/// DELETE /api/comments/:id - Soft delete comment (keep for replies)
pub async fn delete_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(comment_id): Path<i32>,
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
    verify_comment_ownership(&state.pool, comment_id, auth_user.user_id).await?;

    // Soft delete (keep original content in DB, display as [deleted])
    let now = OffsetDateTime::now_utc();
    let result = sqlx::query!(
        "UPDATE comments SET deleted_at = $1 WHERE id = $2 AND user_id = $3",
        now,
        comment_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::OK)
}

// ============================================================================
// Comment Listing Handlers
// ============================================================================

/// GET /api/posts/:id/comments - Get top-level comments for a post
pub async fn get_post_comments(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Path(post_id): Path<i32>,
    Query(query): Query<GetCommentsQuery>,
) -> Result<Response> {

    let page_size = 10;
    let comments = fetch_parent_comments(
        &state.pool,
        Some(post_id),
        None,
        None,
        page_size + 1,
        query.cursor,
        auth_user.as_ref().map(|u| u.user_id),
    )
    .await?;

    let has_more = comments.len() > page_size as usize;
    let comments: Vec<_> = comments.into_iter().take(page_size as usize).collect();
    let next_cursor = comments.last().map(|c| {
        c.created_at.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default()
    });

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
        Some(crate::middleware::csrf::generate_token(
            &state, auth.session_id
        ).await.unwrap_or_default())
    } else {
        None
    };

    let html = crate::templates::comments::render_comment_list_fragment(
        &comments,
        "post",
        post_id,
        current_user.as_ref(),
        csrf_token.as_deref(),
        has_more,
        next_cursor,
    );

    Ok(Html(html.into_string()).into_response())
}

/// GET /api/questions/:id/comments - Get top-level comments for a question
pub async fn get_question_comments(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Path(question_id): Path<i32>,
    Query(query): Query<GetCommentsQuery>,
) -> Result<Response> {

    let page_size = 10;
    let comments = fetch_parent_comments(
        &state.pool,
        None,
        Some(question_id),
        None,
        page_size + 1, 
        query.cursor,
        auth_user.as_ref().map(|u| u.user_id),
    )
    .await?;

    let has_more = comments.len() > page_size as usize;
    let comments: Vec<_> = comments.into_iter().take(page_size as usize).collect();
    let next_cursor = comments.last().map(|c| {
        c.created_at.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default()
    });

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
        Some(crate::middleware::csrf::generate_token(
            &state, auth.session_id
        ).await.unwrap_or_default())
    } else {
        None
    };

    let html = crate::templates::comments::render_comment_list_fragment(
        &comments,
        "question",
        question_id,
        current_user.as_ref(),
        csrf_token.as_deref(),
        has_more,
        next_cursor,
    );

    Ok(Html(html.into_string()).into_response())

}

/// GET /api/answers/:id/comments - Get top-level comments for an answer
pub async fn get_answer_comments(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Path(answer_id): Path<i32>,
    Query(query): Query<GetCommentsQuery>,
) -> Result<Response> {

    let page_size = 10;
    let comments = fetch_parent_comments(
        &state.pool,
        None,
        None,
        Some(answer_id),
        query.limit,
        query.cursor,
        auth_user.as_ref().map(|u| u.user_id),
    )
    .await?;

    let has_more = comments.len() > page_size as usize;
    let comments: Vec<_> = comments.into_iter().take(page_size as usize).collect();
    let next_cursor = comments.last().map(|c| {
        c.created_at.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default()
    });

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

    let csrf_token = if let Some(ref auth ) = auth_user {
        Some(generate_token(&state, auth.session_id)
            .await.unwrap_or_default())
    } else {
        None
    };

    let html = crate::templates::comments::render_comment_list_fragment(
        &comments, 
        "answer", 
        answer_id, 
        current_user.as_ref(), 
        csrf_token.as_deref(), 
        has_more, 
        next_cursor,
    );

    Ok(Html(html.into_string()).into_response())
}

/// GET /api/comments/:id/replies - Lazy load nested replies
pub async fn get_comment_replies(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Path(comment_id): Path<i32>,
    Query(query): Query<GetRepliesQuery>,
) -> Result<Response> {
    tracing::info!(comment_id, "get_comment_replies called");

    let page_size = 5;

    let parent = sqlx::query!(
        "SELECT post_id, question_id, answer_id FROM comments WHERE id = $1",
        comment_id
    )
    .fetch_one(&state.pool)
    .await?;

    let (parent_type, parent_id) = if let Some(pid) = parent.post_id {
        ("post", pid)
    } else if let Some(qid) = parent.question_id {
        ("question", qid)
    } else {
        ("answer", parent.answer_id.unwrap_or(0))
    };

    tracing::info!(
        post_id = ?parent.post_id,
        question_id = ?parent.question_id,
        "parent comment fetched"
    );

    let replies = fetch_comment_replies(
        &state.pool,
        comment_id,
        page_size + 1,
        query.cursor,
        auth_user.as_ref().map(|u| u.user_id),
    )
    .await?;

    tracing::info!(reply_count = replies.len(), "replies fetched");

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
        Some(crate::middleware::csrf::generate_token(
            &state, auth.session_id
        ).await.unwrap_or_default())
    } else {
        None
    };

    let has_more = replies.len() > page_size;
    let replies: Vec<_> = replies.into_iter().take(page_size).collect();
    let next_cursor = replies.last().map(|c| {
        c.created_at.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default()
    });

    let html = crate::templates::comments::render_replies_fragment(
        &replies, 
        parent_type, 
        parent_id, 
        current_user.as_ref(), 
        csrf_token.as_deref(),
        has_more,
        next_cursor,
        comment_id,
    );

    Ok(Html(html.into_string()).into_response())
}

// ============================================================================
// Helpful (Upvote) Handlers
// ============================================================================

/// POST /api/comments/:id/helpful - Mark comment as helpful
pub async fn mark_helpful(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(comment_id): Path<i32>,
) -> Result<StatusCode> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    // Check if comment exists
    let comment = sqlx::query!(
        "SELECT user_id FROM comments WHERE id = $1 AND deleted_at IS NULL",
        comment_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Prevent marking own comment as helpful
    if comment.user_id == auth_user.user_id {
        return Err(AppError::CannotActOnOwnContent);
    }

    // Insert helpful vote (ON CONFLICT do nothing for idempotency)
    sqlx::query!(
        r#"
        INSERT INTO comment_helpful (comment_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (comment_id, user_id) DO NOTHING
        "#,
        comment_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;

    tracing::info!(
        user_id = auth_user.user_id,
        comment_id,
        "Comment marked helpful"
    );

    Ok(StatusCode::OK)
}

/// DELETE /api/comments/:id/helpful - Remove helpful mark
pub async fn remove_helpful(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(comment_id): Path<i32>,
) -> Result<StatusCode> {
    
    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    sqlx::query!(
        "DELETE FROM comment_helpful WHERE comment_id = $1 AND user_id = $2",
        comment_id,
        auth_user.user_id,
    )
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Helper Functions
// ============================================================================

struct ParentCommentInfo {
    depth_level: Option<i32>,
    post_id: Option<i32>,
    question_id: Option<i32>,
    answer_id: Option<i32>,
}

struct CommentRecord {
    id: i32,
    user_id: i32,
    username: String,
    content_rendered_html: String,
    depth_level: Option<i32>,
    helpful_count: Option<i32>,
    reply_count: Option<i32>,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
    has_marked_helpful: Option<bool>,
}

struct CommentOwnership {
    depth_level: Option<i32>,
}

async fn validate_parent_comment(
    pool: &PgPool,
    parent_id: i32,
    post_id: Option<i32>,
    question_id: Option<i32>,
    answer_id: Option<i32>,
) -> Result<ParentCommentInfo> {
    let parent = sqlx::query_as!(
        ParentCommentInfo,
        r#"
        SELECT depth_level, post_id, question_id, answer_id
        FROM comments
        WHERE id = $1 AND deleted_at IS NULL
        "#,
        parent_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Verify parent belongs to same post/question/answer
    if parent.post_id != post_id
        || parent.question_id != question_id
        || parent.answer_id != answer_id
    {
        return Err(ValidationError::MissingField(
            "Parent comment must belong to same parent entity".to_string()
        ).into());
    }

    Ok(parent)
}

async fn check_duplicate_comment(
    pool: &PgPool,
    user_id: i32,
    content_hash: &str,
) -> Result<()> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM comments
            WHERE user_id = $1
              AND content_hash = $2
              AND deleted_at IS NULL
              AND created_at > NOW() - INTERVAL '1 hour'
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

async fn verify_comment_ownership(
    pool: &PgPool,
    comment_id: i32,
    user_id: i32,
) -> Result<CommentOwnership> {
    let comment = sqlx::query_as!(
        CommentOwnership,
        r#"
        SELECT depth_level
        FROM comments
        WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
        "#,
        comment_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    match comment {
        None => Err(AppError::NotFound),
        Some(c) => Ok(c),
    }
}

async fn fetch_comment(
    pool: &PgPool,
    comment_id: i32,
    current_user_id: Option<i32>,
) -> Result<CommentResponse> {
    let comment = sqlx::query_as!(
        CommentRecord,
        r#"
        SELECT 
            c.id, c.user_id, u.username, c.content_rendered_html,
            c.depth_level, c.helpful_count, c.reply_count,
            c.created_at, c.edited_at, c.deleted_at,
            COALESCE(
                (SELECT true FROM comment_helpful 
                 WHERE comment_id = c.id AND user_id = $2),
                false
            ) as "has_marked_helpful?"
        FROM comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.id = $1
        "#,
        comment_id,
        current_user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(build_comment_response(comment))
}

async fn fetch_comment_with_replies(
    pool: &PgPool,
    comment_id: i32,
    current_user_id: Option<i32>,
) -> Result<CommentResponse> {
    let mut comment = fetch_comment(pool, comment_id, current_user_id).await?;
    
    // Fetch replies recursively
    if comment.reply_count > 0 {
        let replies = fetch_comment_replies(pool, comment_id, 10, None, current_user_id).await?;
        comment.replies = Some(replies);
    }

    Ok(comment)
}

async fn fetch_comment_replies(
    pool: &PgPool,
    parent_id: i32,
    limit: usize,
    cursor: Option<String>,
    current_user_id: Option<i32>,
) -> Result<Vec<CommentResponse>> {
    let cursor_time = if let Some(c) = cursor {
        Some(parse_cursor(&c)?)
    } else {
        None
    };

    let replies = if let Some(cursor) = cursor_time {
        sqlx::query_as!(
            CommentRecord,
            r#"
            SELECT 
                c.id, c.user_id, u.username, c.content_rendered_html,
                c.depth_level, c.helpful_count, c.reply_count,
                c.created_at, c.edited_at, c.deleted_at,
                COALESCE(
                    (SELECT true FROM comment_helpful 
                    WHERE comment_id = c.id AND user_id = $3),
                    false
                ) as "has_marked_helpful?"
            FROM comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.parent_comment_id = $1
              AND c.created_at > $2
            ORDER BY c.helpful_count DESC, c.created_at ASC
            LIMIT $4
            "#,
            parent_id,
            cursor,
            current_user_id,
            limit as i64,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            CommentRecord,
            r#"
            SELECT
                c.id, c.user_id, u.username, c.content_rendered_html,
                c.depth_level, c.helpful_count, c.reply_count,
                c.created_at, c.edited_at, c.deleted_at,
                COALESCE(
                    (SELECT true FROM comment_helpful
                     WHERE comment_id = c.id AND user_id = $2),
                    false
                ) as "has_marked_helpful?"
            FROM comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.parent_comment_id = $1
            ORDER BY c.helpful_count DESC, c.created_at ASC
            LIMIT $3
            "#,
            parent_id,
            current_user_id,
            limit as i64,
        )
        .fetch_all(pool)
        .await?
    };

    Ok(replies.into_iter().map(build_comment_response).collect())
}

pub async fn fetch_parent_comments(
    pool: &PgPool,
    post_id: Option<i32>,
    question_id: Option<i32>,
    answer_id: Option<i32>,
    limit: i32,
    cursor: Option<String>,
    current_user_id: Option<i32>,
) -> Result<Vec<CommentResponse>> {
    let limit = limit.clamp(1, 50);
    let cursor_time = if let Some(cursor) = cursor {
        Some(parse_cursor(&cursor)?)
    } else {
        None
    };

    let comments = if let Some(cursor) = cursor_time {
        sqlx::query_as!(
            CommentRecord,
            r#"
            SELECT 
                c.id, c.user_id, u.username, c.content_rendered_html,
                c.depth_level, c.helpful_count, c.reply_count,
                c.created_at, c.edited_at, c.deleted_at,
                COALESCE(
                    (SELECT true FROM comment_helpful 
                     WHERE comment_id = c.id AND user_id = $5),
                    false
                ) as "has_marked_helpful?"
            FROM comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.parent_comment_id IS NULL
              AND c.post_id IS NOT DISTINCT FROM $1
              AND c.question_id IS NOT DISTINCT FROM $2
              AND c.answer_id IS NOT DISTINCT FROM $3
              AND c.created_at < $4
            ORDER BY c.helpful_count DESC, c.created_at DESC
            LIMIT $6
            "#,
            post_id,
            question_id,
            answer_id,
            cursor,
            current_user_id,
            limit as i64,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            CommentRecord,
            r#"
            SELECT 
                c.id, c.user_id, u.username, c.content_rendered_html,
                c.depth_level, c.helpful_count, c.reply_count,
                c.created_at, c.edited_at, c.deleted_at,
                COALESCE(
                    (SELECT true FROM comment_helpful 
                     WHERE comment_id = c.id AND user_id = $4),
                    false
                ) as "has_marked_helpful?"
            FROM comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.parent_comment_id IS NULL
              AND c.post_id IS NOT DISTINCT FROM $1
              AND c.question_id IS NOT DISTINCT FROM $2
              AND c.answer_id IS NOT DISTINCT FROM $3
            ORDER BY c.helpful_count DESC, c.created_at DESC
            LIMIT $5
            "#,
            post_id,
            question_id,
            answer_id,
            current_user_id,
            limit as i64,
        )
        .fetch_all(pool)
        .await?
    };

    Ok(comments.into_iter().map(build_comment_response).collect())
}

fn build_comment_response(comment: CommentRecord) -> CommentResponse {
    let is_deleted = comment.deleted_at.is_some();
    
    CommentResponse {
        id: comment.id,
        user_id: comment.user_id,
        username: if is_deleted {
            "[deleted]".to_string()
        } else {
            comment.username
        },
        content_rendered_html: if is_deleted {
            "[deleted]".to_string()
        } else {
            comment.content_rendered_html
        },
        depth_level: comment.depth_level.unwrap_or(0),
        helpful_count: comment.helpful_count.unwrap_or(0),
        reply_count: comment.reply_count.unwrap_or(0),
        is_deleted,
        has_marked_helpful: comment.has_marked_helpful.unwrap_or(false),
        created_at: comment.created_at,
        edited_at: comment.edited_at,
        replies: None,
    }
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