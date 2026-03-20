use axum::{
    extract::State,
    Json,
};
use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    errors::{AppError, Result},
    middleware::auth::AuthUser,
    state::AppState,
};

// ============================================================================
// TOKEN GENERATION
// ============================================================================

/// Generate CSRF token for a session
pub async fn generate_token(
    state: &AppState,
    session_id: Uuid,
) -> Result<String> {
    let token = Uuid::new_v4().to_string();
    let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(2);
    
    sqlx::query!(
        r#"
        INSERT INTO csrf_tokens (session_id, token, expires_at)
        VALUES ($1, $2, $3)
        "#,
        session_id,
        token,
        expires_at
    )
    .execute(&state.pool)
    .await?;
    
    Ok(token)
}

// ============================================================================
// TOKEN VALIDATION
// ============================================================================

/// Validate CSRF token for a session
pub async fn validate_token(
    state: &AppState,
    token: &str,
    session_id: Uuid,
) -> Result<bool> {
    let valid = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM csrf_tokens
            WHERE token = $1
              AND session_id = $2
              AND expires_at > NOW()
        ) as "exists!"
        "#,
        token,
        session_id
    )
    .fetch_one(&state.pool)
    .await?;
    
    Ok(valid)
}

// ============================================================================
// TOKEN CLEANUP
// ============================================================================

/// Clean up expired CSRF tokens
pub async fn cleanup_expired_tokens(state: &AppState) -> Result<i32> {
    let count = sqlx::query_scalar!(
        "SELECT cleanup_expired_csrf_tokens() as count"
    )
    .fetch_one(&state.pool)
    .await?
    .ok_or_else(|| AppError::InternalError)?;
    
    Ok(count)
}

/// Delete all tokens for a specific session
pub async fn invalidate_session_tokens(pool: &PgPool, session_id: Uuid) -> Result<u64> {
    let result = sqlx::query!(
        "DELETE FROM csrf_tokens WHERE session_id = $1",
        session_id
    )
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected())
}


// ============================================================================
// API RESPONSES
// ============================================================================

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in_seconds: i64,
}

// ============================================================================
// HTTP HANDLERS
// ============================================================================

/// GET /csrf/token
/// Generate a CSRF token for current session
pub async fn get_token(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<TokenResponse>> {
    let token = generate_token(&state, auth.session_id).await?;
    
    Ok(Json(TokenResponse {
        token,
        expires_in_seconds: 7200, // 2 hours
    }))
}