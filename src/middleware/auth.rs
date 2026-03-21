use axum::{
    async_trait,
    extract::{ConnectInfo, FromRequestParts, Request, State},
    http::{HeaderMap, header::COOKIE, request::Parts, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use sqlx::PgPool;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, Ipv6Addr};
use time::OffsetDateTime;
use uuid::Uuid;
use crate::{
    errors::{AppError, Result},
    state::AppState,
    utils::rate_limit::RateLimiter,
};

// ============================================================================
// REQUEST CONTEXT
// ============================================================================

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub auth: Option<AuthUser>,
    pub ip: IpAddr,
    pub user_agent: Option<String>,
    pub request_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i32,
    pub session_id: Uuid,
    pub is_fresh: bool,
}

impl RequestContext {
    pub fn user_id(&self) -> Option<i32> {
        self.auth.as_ref().map(|a| a.user_id)
    }

    pub fn session_id(&self) -> Option<Uuid> {
        self.auth.as_ref().map(|a| a.session_id)
    }
}

// ============================================================================
// EXTRACTION HELPERS
// ============================================================================

fn extract_ip(headers: &HeaderMap, socket: Option<SocketAddr>) -> IpAddr {
    // 1. CF-Connecting-IP — most trusted when behind Cloudflare
    headers
        .get("cf-connecting-ip")
        .and_then(|h| h.to_str().ok())
        .and_then(|ip| ip.trim().parse().ok())
    // 2. X-Real-IP — set by Nginx
    .or_else(|| headers
        .get("x-real-ip")
        .and_then(|h| h.to_str().ok())
        .and_then(|ip| ip.parse().ok()))
    // 3. X-Forwarded-For — least trusted, can be spoofed
    .or_else(|| headers 
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|ip| ip.trim().parse().ok()))
    // 4. Fallback to socket
    .unwrap_or_else(|| socket
        .map(|s| s.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)))
}

fn anonymize_ip(ip: IpAddr) -> Option<ipnetwork::IpNetwork> {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            let truncated = Ipv4Addr::new(o[0], o[1], o[2], 0);
            format!("{}/24", truncated).parse().ok()
        }
        IpAddr::V6(v6) => {
            let mut seg = v6.segments();
            seg[3] = 0; seg[4] = 0; seg[5] = 0; seg[6] = 0; seg[7] = 0;
            let truncated = Ipv6Addr::from(seg);
            format!("{}/48", truncated).parse().ok()
        }
    }
}

fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

fn extract_session_from_cookies(cookies: &str) -> Option<Uuid> {
    cookies
        .split(';')
        .find_map(|cookie| {
            let mut parts = cookie.trim().splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some("session_id"), Some(value)) => Uuid::parse_str(value).ok(),
                _ => None,
            }
        })
}


// ============================================================================
// SESSION VALIDATION
// ============================================================================

async fn validate_session(
    pool: &PgPool,
    session_id: Uuid,
    ip: IpAddr,
    user_agent: Option<&str>,
) -> Result<AuthUser> {
    let now = OffsetDateTime::now_utc();
    
    let session = sqlx::query!(
        r#"
        SELECT user_id, expires_at, last_used_at
        FROM sessions
        WHERE id = $1
        "#,
        session_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::SessionNotFound)?;

    // Check expiry
    if session.expires_at < now {

        tracing::warn!(
            session_id = %session_id,
            user_id = session.user_id,
            "Session expired"
        );

        sqlx::query!("DELETE FROM sessions WHERE id = $1", session_id)
            .execute(pool)
            .await?;
        return Err(AppError::SessionExpired);
    }

    // Check session freshness (last 15 minutes)
    let is_fresh = session.last_used_at
        .map(|t: OffsetDateTime| (now - t).whole_minutes() < 15)
        .unwrap_or(false);

    // Update session telemetry (async, non-blocking)
    let pool_clone = pool.clone();
    let ua = user_agent.map(|s| s.to_string());
    tokio::spawn(async move {
        let _ = sqlx::query!(
            "UPDATE sessions SET last_used_at = $1, ip_address = $2, user_agent = $3 WHERE id = $4",
            now,
            anonymize_ip(ip),
            ua,
            session_id
        )
        .execute(&pool_clone)
        .await;
    });

    Ok(AuthUser {
        user_id: session.user_id,
        session_id,
        is_fresh,
    })
}

// ============================================================================
// MIDDLEWARE: REQUIRED AUTH
// ============================================================================

pub async fn auth_middleware(
    State(state): State<AppState>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
    mut request: Request,
    next: Next,
) -> Response {
    let pool = &state.pool;
    let headers = request.headers();
    let ip = extract_ip(headers, Some(socket));
    let user_agent = extract_user_agent(headers);
    let request_id = Uuid::new_v4();

    let cookies = match request
        .headers()
        .get(COOKIE)
        .and_then(|v| v.to_str().ok())
    {
        Some(cookies) => cookies,
        None => {
            tracing::warn!(
                ip = %ip,
                path = %request.uri().path(),
                "Authentication required - no session cookie"
            );
            return AppError::AuthenticationRequired.into_response()
        }
    };

    let session_id = match extract_session_from_cookies(cookies) {
        Some(session_id) => session_id,
        None => {
            tracing::warn!(
                ip = %ip,
                "Invalid session cookie format"
            );
            return AppError::AuthenticationRequired.into_response()
        }
    };

    let auth = match validate_session(&pool, session_id, ip, user_agent.as_deref()).await {
        Ok(auth) => auth,
        Err(err) => return err.into_response(),
    };

    request.extensions_mut().insert(RequestContext {
        auth: Some(auth),
        ip,
        user_agent,
        request_id,
    });

    next.run(request).await
}


// ============================================================================
// MIDDLEWARE: OPTIONAL AUTH
// ============================================================================

pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
    mut request: Request,
    next: Next,
) -> Response {
    let pool = &state.pool;
    let headers = request.headers();
    let ip = extract_ip(headers, Some(socket));
    let user_agent = extract_user_agent(headers);
    let request_id = Uuid::new_v4();

    let auth: Option<AuthUser> = if let Some(cookies) = request.headers().get(COOKIE).and_then(|v| v.to_str().ok()) {
        if let Some(session_id) = extract_session_from_cookies(cookies) {
            validate_session(&pool, session_id, ip, user_agent.as_deref()).await.ok()
        } else {
            None
        }
    } else {
        None
    };

    request.extensions_mut().insert(RequestContext {
        auth,
        ip,
        user_agent,
        request_id,
    });

    next.run(request).await
}

// ============================================================================
// MIDDLEWARE: CSRF VALIDATION
// ============================================================================

pub async fn csrf_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let pool = &state.pool;

    // Skip CSRF for safe methods
    if request.method().is_safe() {
        return next.run(request).await;
    }

    tracing::info!("CSRF check - token from header: {:?}", request.headers().get("X-csrf-token"));

    // Extract token from header
    let token = match request
        .headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
    {
        Some(token) => token,
        None => {
            let content_type = request.headers()
                .get("content-type")
                .and_then(|v: &HeaderValue| v.to_str().ok())
                .unwrap_or("");
            if content_type.contains("application/x-www-form-urlencoded") {
                return next.run(request).await;
            }
            tracing::warn!(
                path = %request.uri().path(),
                method = %request.method(),
                "CSRF token missing"
            );
            return AppError::CsrfTokenMissing.into_response()
        },
    };

    // Get session from request context
    let ctx = match request.extensions().get::<RequestContext>() {
        Some(ctx) => ctx,
        None => return AppError::InternalError.into_response(),
    };

    let session_id = match ctx.session_id() {
        Some(id) => id,
        None => return AppError::AuthenticationRequired.into_response(),
    };

    tracing::info!("About to validate CSRF token in DB");
    // Validate token
    let valid = match sqlx::query_scalar!(
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
    .fetch_one(pool)
    .await
    {
        Ok(valid) => valid,
        Err(_) => return AppError::InternalError.into_response(),
    };

    tracing::info!("CSRF validation result: {}", valid);

    if !valid {
        tracing::warn!(
            session_id = %session_id,
            path = %request.uri().path(),
            "CSRF token invalid or expired"
        );
        return AppError::CsrfTokenInvalid.into_response();
    }

    next.run(request).await
}

pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response> {
    // Extract IP address from request
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .or_else(|| {
            request
                .extensions()
                .get::<std::net::SocketAddr>()
                .map(|addr| addr.ip())
        });

    // For anonymous requests, check IP-based limit
    if let Some(ip_addr) = ip {
        limiter.check_ip_limit(ip_addr).await?;
    }

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

// ============================================================================
// EXTRACTORS
// ============================================================================

#[async_trait]
impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        parts
            .extensions
            .get::<RequestContext>()
            .cloned()
            .ok_or(AppError::InternalError)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        parts
            .extensions
            .get::<RequestContext>()
            .and_then(|ctx| ctx.auth.clone())
            .ok_or(AppError::AuthenticationRequired)
    }
}

// ============================================================================
// SESSION MANAGEMENT
// ============================================================================

pub async fn create_session(
    pool: &PgPool,
    user_id: i32,
    ip: IpAddr,
    user_agent: Option<String>,
) -> Result<Uuid> {
    let expires_at = OffsetDateTime::now_utc() + time::Duration::days(30);
    
    let session = sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, ip_address, user_agent, expires_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        user_id,
        anonymize_ip(ip),
        user_agent,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(session.id)
}

pub async fn delete_session(pool: &PgPool, session_id: Uuid) -> Result<()> {
    sqlx::query!("DELETE FROM sessions WHERE id = $1", session_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_all_user_sessions(pool: &PgPool, user_id: i32) -> Result<()> {
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn rotate_session(pool: &PgPool, old_session_id: Uuid, user_id: i32) -> Result<Uuid> {
    let result = sqlx::query!(
        r#"
        SELECT rotate_session($1, $2) as new_session_id
        "#,
        old_session_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    result.new_session_id.ok_or(AppError::SessionNotFound)
}

pub async fn cleanup_expired_sessions(pool: &PgPool) -> Result<u64> {
    let result = sqlx::query!(
        "DELETE FROM sessions WHERE expires_at < $1",
        OffsetDateTime::now_utc()
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

