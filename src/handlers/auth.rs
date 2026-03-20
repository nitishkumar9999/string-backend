use axum::{
    extract::{Query, State},
    response::{Redirect, IntoResponse},
    http::{header, HeaderValue, StatusCode, HeaderMap},
    Json,
};
use reqwest::Client;
use urlencoding;
use uuid::Uuid;
use time::OffsetDateTime;
use crate::{
    state::AppState,
    dto::auth::*,
    models::user::CreateUser,
    db::users,
    middleware::{
        auth::{create_session, AuthUser, RequestContext},
        csrf::invalidate_session_tokens,
},
    errors::{AppError, Result},
};

// ============================================================================
// GITHUB OAuth
// ============================================================================

/// GET /auth/github - Initiate GitHub OAuth
pub async fn github_login(State(state): State<AppState>, headers: HeaderMap) -> Result<Redirect> {
    tracing::info!("GitHub login initiated");

    let next = headers
        .get("Referer")
        .and_then(|v| v.to_str().ok())
        .and_then(|r| url::Url::parse(r).ok())
        .map(|u| {
            match u.query() {
                Some(q) => format!("{}?{}", u.path(), q),
                None => u.path().to_string(),
            }
        })
        .unwrap_or_else(|| "/".to_string());
        

    tracing::info!("Login from referer: {}", next);
    
    let client_id = match std::env::var("GITHUB_CLIENT_ID") {
        Ok(id) => {
            tracing::info!("GitHub client ID found");
            id
        }
        Err(e) => {
            tracing::error!("GitHub client ID not found: {:?}", e);
            return Err(AppError::OAuth("GitHub client ID not configured".to_string()));
        }
    };
    /*let client_id = std::env::var("GITHUB_CLIENT_ID")
        .map_err(|_| AppError::OAuth("GitHub client ID not configured".to_string()))?; 
    */

    let oauth_state = format!("{}|{}", Uuid::new_v4(), next);
    let redirect_uri = format!("{}/auth/github/callback", state.base_url);

    tracing::info!("Redirect URI: {}", redirect_uri);
    
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&state={}&scope=user:email",
        client_id,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&oauth_state)
    );

    tracing::info!("Auth URL generated: {}", auth_url);

    Ok(Redirect::to(&auth_url))
}


/// GET /auth/github/callback - GitHub OAuth callback

pub async fn github_callback(
    State(state): State<AppState>,
    ctx: RequestContext,
    Query(params): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
    match github_callback_inner(State(state), ctx, Query(params)).await {
        Ok(response) => response.into_response(),
        Err(e) => {
            tracing::error!("OAuth error: {:?}", e);
            Redirect::to("/feed?error=login_failed").into_response()
        }
    }
}

pub async fn github_callback_inner(
    State(state): State<AppState>,
    ctx: RequestContext,
    Query(params): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse> {

    // 1. Exchange code for access token
    let client_id = std::env::var("GITHUB_CLIENT_ID")
        .map_err(|_| AppError::OAuth("GitHub client ID not configured".to_string()))?;
    let client_secret = std::env::var("GITHUB_CLIENT_SECRET")
        .map_err(|_| AppError::OAuth("GitHub client secret not configured".to_string()))?;

    let client = Client::new();
    
    let token_response = client
        .post("https://github.com/login/oauth/access_token")
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": params.code,
        }))
        .send()
        .await
        .map_err(|e| AppError::OAuth(format!("Failed to get token: {}", e)))?
        .json::<GitHubTokenResponse>()
        .await
        .map_err(|e| AppError::OAuth(format!("Failed to parse token: {}", e)))?;

    // 2. Get user info from GitHub
    let github_user = client
        .get("https://api.github.com/user")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token_response.access_token))
        .header(reqwest::header::USER_AGENT, "YourAppName/1.0")
        .send()
        .await
        .map_err(|e| AppError::OAuth(format!("Failed to get user: {}", e)))?
        .json::<GitHubUser>()
        .await
        .map_err(|e| AppError::OAuth(format!("Failed to parse user: {}", e)))?;

    // 3. Find or create user in database
    let user = match users::find_by_github_id(&state.pool, github_user.id).await? {
        Some(existing_user) => {
            tracing::info!(
                user_id = existing_user.id,
                username = %existing_user.username,
                provider = "github",
                "User logged in"
            );
            existing_user            
        },
        None => {

            let existing_deleted = sqlx::query!(
                "SELECT id FROM users WHERE github_id = $1",
                github_user.id
            )
            .fetch_optional(&state.pool)
            .await?;

            if let Some(deleted_user) = existing_deleted {
                sqlx::query!(
                    "UPDATE users SET deleted_at = NULL WHERE id = $1",
                    deleted_user.id
                )
                .execute(&state.pool)
                .await?;
                users::find_by_github_id(&state.pool, github_user.id).await?.ok_or(AppError::AccountNotFound)?
            } else {

                 // Create new user
                let username = users::generate_unique_username(&state.pool, &github_user.login).await?;

                let new_user = CreateUser {
                    github_id: Some(github_user.id),
                    github_username: Some(github_user.login.clone()),
                    username,
                    name: github_user.name.unwrap_or_else(|| github_user.login.clone()),
                    avatar_url: Some(github_user.avatar_url),
                };

                let new_user = users::create_user(&state.pool, new_user).await?;

                tracing::info!(
                    user_id = new_user.id,
                    username = %new_user.username,
                    provider = "github",
                    "New user registered"
                );
                new_user

            }
            
        }
    };

    // 4. Create session (using actual IP and user agent from context)
    let session_id = create_session(
        &state.pool,
        user.id,
        ctx.ip,
        ctx.user_agent,
    ).await?;

    // 5. Set session cookie and redirect
    let cookie = format!(
        "session_id={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        session_id,
        30 * 24 * 60 * 60 // 30 days
    );

    let next = params.state
        .splitn(2, '|')
        .nth(1)
        .filter(|n| n.starts_with('/'))
        .unwrap_or("/")
        .to_string();

    tracing::info!("State param: {}", params.state);
    tracing::info!("Redirecting to next: {}", next);

    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie.parse()?);
    headers.insert(header::LOCATION, next.parse()?);

    Ok((StatusCode::SEE_OTHER, headers))

}

// ============================================================================
// LOGOUT
// ============================================================================

/// POST /auth/logout - Logout current session
pub async fn logout(
    State(state): State<AppState>,
    auth: Option<AuthUser>,
    headers_in: HeaderMap,
) -> impl IntoResponse {
    tracing::info!("Logout handler started");

    if let Some(auth) = auth {
        if let Err(e) =
            invalidate_session_tokens(&state.pool, auth.session_id).await
        {
            tracing::warn!("Failed to invalidate CSRF tokens: {:?}", e);
        }

        if let Err(e) = sqlx::query!(
            "DELETE FROM sessions WHERE id = $1",
            auth.session_id
        )
        .execute(&state.pool)
        .await
        {
            tracing::warn!("Failed to delete session: {:?}", e);
        }
    }

    let mut headers = HeaderMap::new();

    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_static("session_id=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
    );

    let is_htmx = headers_in.contains_key("hx-request");

    if is_htmx {
        headers.insert("hx-redirect", "/?logout=true".parse().unwrap());
        tracing::info!("Logout completed");
        (StatusCode::OK, headers)
    } else {
        headers.insert(header::LOCATION, HeaderValue::from_static("/?logout=true"));
        tracing::info!("Logout completed");
        (StatusCode::SEE_OTHER, headers)
    }

}

/// POST /auth/logout-all - Logout all sessions
pub async fn logout_all(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<LogoutResponse>> {

    let session_ids = sqlx::query_scalar!(
        "SELECT id FROM sessions WHERE user_id = $1",
        auth.user_id
    )
    .fetch_all(&state.pool)
    .await?;

    for session_id in session_ids {
        let _ = invalidate_session_tokens(&state.pool, session_id).await;
    }

    let result = sqlx::query!(
        "DELETE FROM sessions WHERE user_id = $1",
        auth.user_id
    )
    .execute(&state.pool)
    .await?;

    tracing::warn!(
        user_id = auth.user_id,
        sessions_deleted = result.rows_affected(),
        "All sessions terminated"
    );

    Ok(Json(LogoutResponse {
        success: true,
        sessions_deleted: result.rows_affected(),
    }))
}

// ============================================================================
// CURRENT USER
// ============================================================================

/// GET /api/auth/me - Get current user info with stats
pub async fn me(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> Result<Json<Option<MeResponse>>> {
    let user_info = match ctx.auth {
        Some(auth) => {
            // Fetch user
            let user = sqlx::query!(
                r#"
                SELECT id, username, name, avatar_url, bio_rendered_html, created_at
                FROM users
                WHERE id = $1 AND deleted_at IS NULL
                "#,
                auth.user_id
            )
            .fetch_optional(&state.pool)
            .await?
            .ok_or(AppError::AccountNotFound)?;

            // Fetch stats based on YOUR schema
            let stats = sqlx::query!(
                r#"
                SELECT
                    (SELECT COUNT(*) FROM posts WHERE user_id = $1 AND deleted_at IS NULL) as "post_count!",
                    (SELECT COUNT(*) FROM questions WHERE user_id = $1 AND deleted_at IS NULL) as "question_count!",
                    (SELECT COUNT(*) FROM answers WHERE user_id = $1 AND deleted_at IS NULL) as "answer_count!",
                    (SELECT COUNT(*) FROM refracts WHERE user_id = $1 AND deleted_at IS NULL) as "refract_count!",
                    (SELECT COALESCE(SUM(echo_count), 0) FROM posts WHERE user_id = $1 AND deleted_at IS NULL) +
                    (SELECT COALESCE(SUM(echo_count), 0) FROM questions WHERE user_id = $1 AND deleted_at IS NULL) +
                    (SELECT COALESCE(SUM(echo_count), 0) FROM answers WHERE user_id = $1 AND deleted_at IS NULL) as "total_echoes!"
                "#,
                auth.user_id
            )
            .fetch_one(&state.pool)
            .await?;

            Some(MeResponse {
                user: UserInfo {
                    id: user.id,
                    username: user.username,
                    name: user.name,
                    avatar_url: user.avatar_url,
                    bio: user.bio_rendered_html,
                    created_at: user.created_at,
                },
                stats: UserStats {
                    post_count: stats.post_count,
                    question_count: stats.question_count,
                    answer_count: stats.answer_count,
                    refract_count: stats.refract_count,
                    total_echoes_received: Some(stats.total_echoes),
                },
            })
        }
        None => None,
    };

    Ok(Json(user_info))
}

// ============================================================================
// SESSION MANAGEMENT
// ============================================================================

/// GET /api/auth/sessions - List all user sessions
pub async fn list_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<SessionsResponse>> {
    let sessions = sqlx::query!(
        r#"
        SELECT id, ip_address::text as ip_text, user_agent, last_used_at, created_at
        FROM sessions
        WHERE user_id = $1 AND expires_at > NOW()
        ORDER BY last_used_at DESC NULLS LAST
        "#,
        auth.user_id
    )
    .fetch_all(&state.pool)
    .await?;

    let session_infos: Vec<SessionInfo> = sessions
        .into_iter()
        .map(|s| SessionInfo {
            id: s.id,
            ip_address: s.ip_text,
            user_agent: s.user_agent,
            last_used_at: s.last_used_at.unwrap_or(s.created_at),
            created_at: s.created_at,
            is_current: s.id == auth.session_id,
        })
        .collect();

    Ok(Json(SessionsResponse {
        sessions: session_infos,
    }))
}

/// DELETE /api/auth/sessions/:session_id - Delete specific session
pub async fn delete_session(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> Result<Json<LogoutResponse>> {
    // Prevent deleting current session (use logout instead)
    if session_id == auth.session_id {
        return Err(AppError::Forbidden);
    }

    let _ = invalidate_session_tokens(&state.pool, session_id).await;

    let result = sqlx::query!(
        "DELETE FROM sessions WHERE id = $1 AND user_id = $2",
        session_id,
        auth.user_id
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::SessionNotFound);
    }

    Ok(Json(LogoutResponse {
        success: true,
        sessions_deleted: 1,
    }))
}


/// DELETE /api/auth/account - Delete user account (DANGEROUS)
pub async fn delete_account(
    State(state): State<AppState>,
    auth: Option<AuthUser>,
    headers_in: HeaderMap
) -> impl IntoResponse {
    tracing::info!("Delete account handler started");
    // This will cascade delete everything due to ON DELETE CASCADE in schema:
    // - sessions
    // - posts
    // - questions
    // - answers
    // - refracts
    // - comments
    // - echos
    // - comment_helpful
    // - collections
    // etc.

    if let Some(auth) = auth {
        match sqlx::query_scalar!(
            "SELECT id FROM sessions WHERE user_id = $1",
            auth.user_id
        )
        .fetch_all(&state.pool)
        .await
        {
            Ok(session_ids) => {
                for session_id in session_ids {
                    if let Err(e) = 
                        invalidate_session_tokens(&state.pool, session_id).await 
                    {
                        tracing::warn!(
                            sessions_id = %session_id,
                            "Failed to invalidate CSRF tokens: {:?}", e
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    user_id = auth.user_id,
                    "Failed to fetch sessions: {:?}", e
                );
            }
        }

        if let Err(e) = sqlx::query!(
            r#"
            UPDATE users
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            auth.user_id
        )
        .execute(&state.pool)
        .await
        {
            tracing::warn!(
                user_id = auth.user_id,
                "Failed to delete account: {:?}", e
            );
        } else {
            tracing::warn!(
                user_id = auth.user_id,
                "Account deleted"
            );
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_static("session_id=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",),
    );

    let is_htmx = headers_in.contains_key("hx-request");

    if is_htmx {
        headers.insert("hx-redirect", "/?deleted=true".parse().unwrap());
        (StatusCode::OK, headers)
    } else {
        headers.insert(
            header::LOCATION,
            HeaderValue::from_static("/?deleted=true"),
        );
        (StatusCode::SEE_OTHER, headers)
    }
}


// Response structs
#[derive(Debug, serde::Serialize)]
pub struct MeResponse {
    pub user: UserInfo,
    pub stats: UserStats,
}

#[derive(Debug, serde::Serialize)]
pub struct UserStats {
    pub post_count: i64,
    pub question_count: i64,
    pub answer_count: i64,
    pub refract_count: i64,
    pub total_echoes_received: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub sessions_deleted: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionInfo>,
}

#[derive(Debug, serde::Serialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_used_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub is_current: bool,
}