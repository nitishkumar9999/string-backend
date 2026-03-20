use axum::{
    Json, extract::State, http::HeaderMap, response::{Html, IntoResponse, Response}
};
use pulldown_cmark::html;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing;

use crate::errors::{AppError, Result, ValidationError};
use crate::middleware::{
    auth::AuthUser,
    csrf::validate_token,
};
use crate::utils::rate_limit::{RateLimitAction, RateLimiter};
use crate::state::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct EchoRequest {
    // Exactly one must be provided
    pub post_id: Option<i32>,
    pub question_id: Option<i32>,
    pub answer_id: Option<i32>,
    pub refract_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct EchoResponse {
    pub success: bool,
    pub echo_count: i32,
}

// ============================================================================
// Handler
// ============================================================================
const ICON_ECHO: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M14 12c0-1.1-.9-2-2-2s-2 .9-2 2a2 2 0 1 0 4 0m-6 0c0-1.07.42-2.07 1.17-2.82L7.76 7.76A5.97 5.97 0 0 0 6 12c0 1.6.62 3.11 1.76 4.25l1.41-1.42A3.96 3.96 0 0 1 8 12m8.24-4.24-1.41 1.41C15.59 9.93 16 10.93 16 12s-.42 2.07-1.17 2.83l1.41 1.41C17.37 15.11 18 13.6 18 12s-.62-3.11-1.76-4.24"></path><path d="M6.34 17.66C4.83 16.15 3.99 14.14 3.99 12s.83-4.14 2.34-5.65L4.92 4.93C3.03 6.82 1.99 9.33 1.99 12s1.04 5.18 2.93 7.07l1.41-1.41ZM19.07 4.93l-1.41 1.41C19.17 7.85 20 9.86 20 12s-.83 4.15-2.34 5.66l1.41 1.41C20.96 17.18 22 14.67 22 12s-1.04-5.18-2.93-7.07"></path></svg>"#;
/// POST /api/echo - Echo (like/upvote) a post/question/answer/refract
pub async fn create_echo(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Json(req): Json<EchoRequest>,
) -> Result<Response> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }
    // Light rate limit (10/min to prevent spam)
    rate_limiter
        .check_user_limit(auth_user.user_id, RateLimitAction::Echo)
        .await?;

    // Validate exactly one target
    let target_count = [req.post_id, req.question_id, req.answer_id, req.refract_id]
        .iter()
        .filter(|t| t.is_some())
        .count();

    if target_count != 1 {
        return Err(ValidationError::MissingField(
            "Exactly one of post_id, question_id, answer_id, or refract_id required".to_string()
        ).into());
    }

    // Determine target type and verify ownership (prevent echoing own content)
    if let Some(post_id) = req.post_id {
        verify_not_own_post(&state.pool, post_id, auth_user.user_id).await?;
        let echo_count = insert_echo(
            &state.pool,
            auth_user.user_id,
            Some(post_id),
            None,
            None,
            None,
        )
        .await?;
        
        let count = get_post_echo_count(&state.pool, post_id).await?;
        let html = format!(
            r#"<span class="echo-btn echoed">
                {}
                <span class="count">{}</span>
                
            </span>"#,
            ICON_ECHO,
            count
        );
        return Ok(Html(html).into_response());
    }

    if let Some(question_id) = req.question_id {
        verify_not_own_question(&state.pool, question_id, auth_user.user_id).await?;
        let echo_count = insert_echo(
            &state.pool,
            auth_user.user_id,
            None,
            Some(question_id),
            None,
            None,
        )
        .await?;
        
        let count = get_question_echo_count(&state.pool, question_id).await?;
        let html = format!(
            r#"<span class="echo-btn echoed">
                {}
                <span class="count">{}</span>
                
            </span>"#,
            ICON_ECHO,
            count
        );
        return Ok(Html(html).into_response());
    }

    if let Some(answer_id) = req.answer_id {
        verify_not_own_answer(&state.pool, answer_id, auth_user.user_id).await?;
        let echo_count = insert_echo(
            &state.pool,
            auth_user.user_id,
            None,
            None,
            Some(answer_id),
            None,
        )
        .await?;
        
        let count = get_answer_echo_count(&state.pool, answer_id).await?;
        let html = format!(
            r#"<span class="echo-btn echoed">
                {}
                <span class="count">{}</span>
                
            </span>"#,
            ICON_ECHO,
            count
        );
        return Ok(Html(html).into_response());
    }

    if let Some(refract_id) = req.refract_id {
        verify_not_own_refract(&state.pool, refract_id, auth_user.user_id).await?;
        let echo_count = insert_echo(
            &state.pool,
            auth_user.user_id,
            None,
            None,
            None,
            Some(refract_id),
        )
        .await?;
        
        let count = get_refract_echo_count(&state.pool, refract_id).await?;

        let html = format!(
            r#"<span class="echo-btn echoed">
                {}
                <span class="count">{}</span>
                
            </span>"#,
            ICON_ECHO,
            count
        );
        return Ok(Html(html).into_response());
    }

    // Should never reach here due to validation above
    Err(ValidationError::MissingField("Invalid echo target".to_string()).into())
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn insert_echo(
    pool: &PgPool,
    user_id: i32,
    post_id: Option<i32>,
    question_id: Option<i32>,
    answer_id: Option<i32>,
    refract_id: Option<i32>,
) -> Result<i32> {
    // Insert echo (ON CONFLICT DO NOTHING for idempotency)
    // Returns the ID if inserted, or existing ID if already echoed
    let result = sqlx::query!(
        r#"
        INSERT INTO echos (user_id, post_id, question_id, answer_id, refract_id)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT DO NOTHING
        RETURNING id
        "#,
        user_id,
        post_id,
        question_id,
        answer_id,
        refract_id,
    )
    .fetch_optional(pool)
    .await?;

    // If no row returned, echo already existed (idempotent)
    Ok(result.map(|r| r.id).unwrap_or(0))
}

async fn verify_not_own_post(pool: &PgPool, post_id: i32, user_id: i32) -> Result<()> {
    let post = sqlx::query!(
        "SELECT user_id FROM posts WHERE id = $1 AND deleted_at IS NULL AND is_spam = false",
        post_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if post.user_id == user_id {
        return Err(AppError::CannotActOnOwnContent);
    }

    Ok(())
}

async fn verify_not_own_question(pool: &PgPool, question_id: i32, user_id: i32) -> Result<()> {
    let question = sqlx::query!(
        "SELECT user_id FROM questions WHERE id = $1 AND deleted_at IS NULL AND is_spam = false",
        question_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if question.user_id == user_id {
        return Err(AppError::CannotActOnOwnContent);
    }

    Ok(())
}

async fn verify_not_own_answer(pool: &PgPool, answer_id: i32, user_id: i32) -> Result<()> {
    let answer = sqlx::query!(
        "SELECT user_id FROM answers WHERE id = $1 AND deleted_at IS NULL AND is_spam = false",
        answer_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if answer.user_id == user_id {
        return Err(AppError::CannotActOnOwnContent);
    }

    Ok(())
}

async fn verify_not_own_refract(pool: &PgPool, refract_id: i32, user_id: i32) -> Result<()> {
    let refract = sqlx::query!(
        "SELECT user_id FROM refracts WHERE id = $1 AND deleted_at IS NULL AND is_spam = false",
        refract_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if refract.user_id == user_id {
        return Err(AppError::CannotActOnOwnContent);
    }

    Ok(())
}

async fn get_post_echo_count(pool: &PgPool, post_id: i32) -> Result<i32> {
    let post = sqlx::query!(
        "SELECT echo_count FROM posts WHERE id = $1",
        post_id
    )
    .fetch_one(pool)
    .await?;

    Ok(post.echo_count.unwrap_or(0))
}

async fn get_question_echo_count(pool: &PgPool, question_id: i32) -> Result<i32> {
    let question = sqlx::query!(
        "SELECT echo_count FROM questions WHERE id = $1",
        question_id
    )
    .fetch_one(pool)
    .await?;

    Ok(question.echo_count.unwrap_or(0))
}

async fn get_answer_echo_count(pool: &PgPool, answer_id: i32) -> Result<i32> {
    let answer = sqlx::query!(
        "SELECT echo_count FROM answers WHERE id = $1",
        answer_id
    )
    .fetch_one(pool)
    .await?;

    Ok(answer.echo_count.unwrap_or(0))
}

async fn get_refract_echo_count(pool: &PgPool, refract_id: i32) -> Result<i32> {
    let refract = sqlx::query!(
        "SELECT echo_count FROM refracts WHERE id = $1",
        refract_id
    )
    .fetch_one(pool)
    .await?;

    Ok(refract.echo_count.unwrap_or(0))
}