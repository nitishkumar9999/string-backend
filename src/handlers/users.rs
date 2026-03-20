use axum::{
    Json, extract::{Path, Query, State, Form}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response, Html}
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;
use sha2::{Sha256, Digest};
use tracing;
use maud::html;

use crate::{errors::{AppError, Result, ValidationError}, templates::profile::ProfileFeedItem};
use crate::middleware::{
    auth::AuthUser,
    csrf::{validate_token, generate_token},
};
use crate::utils::rate_limit::{RateLimitAction, RateLimiter};
use crate::state::AppState;
use crate::markdown::parse_markdown;
use crate::templates::profile::{ProfileData, ActivityItem};
use crate::handlers::feed::RefractFeedItem;
use crate::templates::feed::render_refract_card;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUsernameRequest {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub platform: String,
    pub url: String,
    pub display_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkRequest {
    pub url: Option<String>,
    pub display_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserFeedQuery {
    pub cursor: Option<i32>,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    25
}

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub bio_rendered_html: Option<String>,
    pub created_at: OffsetDateTime,
    pub links: Vec<UserLinkResponse>,
    pub stats: UserStatsResponse,
}

#[derive(Debug, Serialize)]
pub struct UserStatsResponse {
    pub post_count: i64,
    pub question_count: i64,
    pub answer_count: i64,
    pub refract_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UserLinkResponse {
    pub id: i32,
    pub platform: String,
    pub url: String,
    pub display_text: Option<String>,
    pub display_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct BulkUpdateLinksRequest {
    pub link_github: Option<String>,
    pub link_website: Option<String>,
    pub link_twitter: Option<String>,
    pub link_youtube: Option<String>,
    pub link_email: Option<String>,
    // No display_text needed
}

#[derive(Debug, Serialize)]
pub struct UserFeedResponse {
    pub data: Vec<UserFeedItem>,
    pub next_cursor: Option<i32>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum UserFeedItem {
    Post(UserPostItem),
    Question(UserQuestionItem),
    Answer(UserAnswerItem),
    Refract(RefractFeedItem),
}

#[derive(Debug, Serialize)]
pub struct UserPostItem {
    pub id: i32,
    pub title: Option<String>,
    pub content_rendered_html: String,
    pub slug: String,
    pub tags: Vec<String>,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub comment_count: i32,
    pub refract_count: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct UserQuestionItem {
    pub id: i32,
    pub title: String,
    pub content_rendered_html: String,
    pub slug: String,
    pub tags: Vec<String>,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub answer_count: i32,
    pub comment_count: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct UserAnswerItem {
    pub id: i32,
    pub question_id: i32,
    pub question_title: String,
    pub question_slug: String,
    pub content_rendered_html: String,
    pub slug: String,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub comment_count: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct UserRefractItem {
    pub id: i32,
    pub original_post_id: i32,
    pub content_rendered_html: String,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub created_at: OffsetDateTime,
}

// ============================================================================
// User Profile Handlers
// ============================================================================

/// GET /api/users/:username - Get user profile with stats
pub async fn get_user_profile(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Path(username): Path<String>,
) -> Result<Html<String>> {
    // Note: No rate limiting for profile views (public data)

    // Fetch user profile with stats
    let user = sqlx::query!(
        r#"
        SELECT 
            u.id, u.username, u.name, u.avatar_url, u.bio_rendered_html, u.created_at,
            (SELECT COUNT(*) FROM posts WHERE user_id = u.id AND deleted_at IS NULL) as "post_count!",
            (SELECT COUNT(*) FROM questions WHERE user_id = u.id AND deleted_at IS NULL) as "question_count!",
            (SELECT COUNT(*) FROM answers WHERE user_id = u.id AND deleted_at IS NULL) as "answer_count!",
            (SELECT COUNT(*) FROM refracts WHERE user_id = u.id AND deleted_at IS NULL) as "refract_count!"
        FROM users u
        WHERE u.username = $1 AND u.deleted_at IS NULL
        "#,
        username
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Fetch user links
    let links = sqlx::query!(
        r#"
        SELECT id, platform, url, display_text, display_order
        FROM user_links
        WHERE user_id = $1
        ORDER BY display_order ASC
        "#,
        user.id
    )
    .fetch_all(&state.pool)
    .await?;

    let profile = ProfileData {
        user_id: user.id,
        username: user.username.clone(),
        display_name: Some(user.name),  // or user.name if it's Option<String>
        bio: user.bio_rendered_html,
        avatar_url: user.avatar_url,
        joined_at: user.created_at,
        post_count: user.post_count as i32,
        question_count: user.question_count as i32,
        answer_count: user.answer_count as i32,
        refract_count: user.refract_count as i32,
        links: links.into_iter().map(|link| crate::templates::profile::UserLink {
            id: link.id,
            platform: link.platform,
            url: link.url,
        }).collect(),
        is_own_profile: auth_user.as_ref().map(|a| a.user_id == user.id).unwrap_or(false),
    };

    let current_user_id = auth_user.as_ref().map(|a| a.user_id);

    let feed_items = fetch_user_feed(&state.pool, profile.user_id, None, 15, current_user_id).await?;
    let activities = convert_to_activities(feed_items);

    let csrf_token = if let Some(ref auth) = auth_user {
        generate_token(&state, auth.session_id).await.ok()
    } else {
        None
    };

    let current_user = if let Some(auth) = auth_user {
        let username = sqlx::query_scalar!("SELECT username FROM users WHERE id = $1", auth.user_id)
            .fetch_one(&state.pool)
            .await?;
        Some((auth.user_id, username, None))
    } else {
        None
    };



    let markup = crate::templates::profile::render_profile_page(profile, activities, current_user, csrf_token);

    Ok(Html(markup.into_string()))
}

/// GET /api/users/:username/feed - Get user's content feed
pub async fn get_user_feed(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Path(username): Path<String>,
    Query(mut query): Query<UserFeedQuery>,
) -> Result<Html<String>> {
    // Note: No rate limiting for public feed views

    // Validate limit
    query.limit = query.limit.clamp(1, 100);

    // Get user ID
    let user = sqlx::query!("SELECT id FROM users WHERE username = $1 AND deleted_at IS NULL", username)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let current_user_id = auth_user.as_ref().map(|a| a.user_id);

    // Fetch user feed items
    let feed_items = fetch_user_feed(&state.pool, user.id, query.cursor, query.limit + 1, current_user_id).await?;

    // Build paginated response
    let has_more = feed_items.len() > query.limit as usize;
    let mut data = feed_items;
    
    if has_more {
        data.pop();
    }

    let next_cursor = if has_more {
        data.last().map(|item| get_feed_item_id(item))
    } else {
        None
    };

    let activities = convert_to_activities(data);
    let markup = html! {
        @for item in &activities {
            @match item {
                crate::templates::profile::ProfileFeedItem::Refract(r) => {
                    (crate::templates::feed::render_refract_card(r))
                },
                crate::templates::profile::ProfileFeedItem::Activity(a) => {
                    (crate::templates::profile::render_activity_item(a))
                },
            }
        }
    };
    Ok(Html(markup.into_string()))
}

/// PATCH /api/users/profile - Update current user's profile
pub async fn update_profile(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Form(req): Form<UpdateProfileRequest>,
) -> Result<Response> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    // Rate limit
    rate_limiter
        .check_user_limit(auth_user.user_id, RateLimitAction::UpdateProfile)
        .await?;

    // Validate name if provided
    if let Some(ref name) = req.name {
        if name.trim().is_empty() || name.len() > 100 {
            return Err(ValidationError::MissingField("Name must be 1-100 characters".to_string()).into());
        }
    }

    // Process bio if provided
    let (bio_raw, bio_rendered_html, bio_hash) = if let Some(bio) = req.bio {
        // Validate bio length
        if bio.len() > 1000 {
            return Err(ValidationError::MissingField("Bio must be under 1000 characters".to_string()).into());
        }
        
        let rendered = parse_markdown(&bio);
        let hash = calculate_hash(&bio);
        (Some(bio), Some(rendered), Some(hash))
    } else {
        (None, None, None)
    };

    // Validate avatar URL if provided
    if let Some(ref url) = req.avatar_url {
        validate_url(url)?;
    }

    // Update profile
    sqlx::query!(
        r#"
        UPDATE users
        SET 
            name = COALESCE($1, name),
            bio_raw = COALESCE($2, bio_raw),
            bio_rendered_html = COALESCE($3, bio_rendered_html),
            bio_content_hash = COALESCE($4, bio_content_hash),
            avatar_url = COALESCE($5, avatar_url),
            updated_at = NOW()
        WHERE id = $6
        "#,
        req.name,
        bio_raw,
        bio_rendered_html,
        bio_hash,
        req.avatar_url,
        auth_user.user_id
    )
    .execute(&state.pool)
    .await?;

    Ok(Html(r#"<div class="toast toast-success">Saved!</div>"#.to_string()).into_response())
}

/// PATCH /api/users/username - Update current user's username
pub async fn update_username(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Form(req): Form<UpdateUsernameRequest>,
) -> Result<Response> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    // Rate limit (strict - username changes are sensitive)
    rate_limiter
        .check_user_limit(auth_user.user_id, RateLimitAction::UpdateUsername)
        .await?;

    // Validate username format
    validate_username(&req.username)?;

    let current = sqlx::query_scalar!(
        "SELECT username FROM users WHERE id = $1",
        auth_user.user_id
    )
    .fetch_one(&state.pool)
    .await?;

    // Check if username is available
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND id != $2)",
        req.username,
        auth_user.user_id
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(false);

    if exists {
        return Err(ValidationError::MissingField("Username already taken".to_string()).into());
    }

    // Check 30-day cooldown
    let last_changed = sqlx::query_scalar!(
        "SELECT last_username_changed_at FROM users WHERE id = $1",
        auth_user.user_id
    )
    .fetch_one(&state.pool)
    .await?;

    if let Some(last_changed) = last_changed {
        let days_since = (OffsetDateTime::now_utc() - last_changed).whole_days();
        if days_since < 30 {
            return Err(ValidationError::MissingField(
                format!("Can only change username every 30 days. {} days remaining.", 30 - days_since)
            ).into());
        }
    }

    // Update username
    sqlx::query!(
        "UPDATE users SET username = $1, last_username_changed_at = NOW() WHERE id = $2",
        req.username,
        auth_user.user_id
    )
    .execute(&state.pool)
    .await?;

    tracing::info!(
        user_id = auth_user.user_id,
        old_username = %current,
        new_username = %req.username,
        "Username changed"
    );

    Ok(Html(r#"<div class="toast toast-success">Saved!</div>"#.to_string()).into_response())
}

// ============================================================================
// User Links Handlers
// ============================================================================

/// POST /api/users/links - Add a new link
pub async fn add_user_link(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Form(req): Form<CreateLinkRequest>,
) -> Result<Response> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    // Rate limit
    rate_limiter
        .check_user_limit(auth_user.user_id, RateLimitAction::UpdateProfile)
        .await?;

    // Validate platform
    let allowed_platforms = ["github", "youtube", "website", "email", "twitter"];
    if !allowed_platforms.contains(&req.platform.as_str()) {
        return Err(ValidationError::MissingField("Invalid platform".to_string()).into());
    }

    // Validate URL
    validate_url(&req.url)?;

    // Get max display order
    let max_order = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(display_order), -1) FROM user_links WHERE user_id = $1",
        auth_user.user_id
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(-1);

    // Insert link
    let link = sqlx::query!(
        r#"
        INSERT INTO user_links (user_id, platform, url, display_text, display_order)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, platform, url, display_text, display_order
        "#,
        auth_user.user_id,
        req.platform,
        req.url,
        None as Option<String>,
        max_order + 1
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Html(r#"<div class="toast toast-success">Saved!</div>"#.to_string()).into_response())
}

/// PATCH /api/users/links/:id - Update a link
pub async fn update_user_link(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(link_id): Path<i32>,
    Form(req): Form<UpdateLinkRequest>,
) -> Result<Response> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    // Rate limit
    rate_limiter
        .check_user_limit(auth_user.user_id, RateLimitAction::UpdateProfile)
        .await?;

    // Verify ownership
    let link_user_id = sqlx::query_scalar!(
        "SELECT user_id FROM user_links WHERE id = $1",
        link_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if link_user_id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    // Validate URL if provided
    if let Some(ref url) = req.url {
        validate_url(url)?;
    }

    // Update link
    let link = sqlx::query!(
        r#"
        UPDATE user_links
        SET url = COALESCE($1, url), 
            display_text = COALESCE($2, display_text)
        WHERE id = $3
        RETURNING id, platform, url, display_text, display_order
        "#,
        req.url,
        req.display_text,
        link_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Html(r#"<div class="toast toast-success">Saved!</div>"#.to_string()).into_response())
}

pub async fn update_links_bulk(
    State(state): State<AppState>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Form(req): Form<BulkUpdateLinksRequest>,
) -> Result<Response>{

    tracing::info!("update_links_bulk called, req: {:?}", req);

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }

    let links = vec![
        ("github", req.link_github),
        ("website", req.link_website),
        ("twitter", req.link_twitter),
        ("youtube", req.link_youtube),
        ("email", req.link_email),
    ];

    for (platform, url_opt) in links {
        if let Some(url) = url_opt {
            if !url.is_empty() {
                if platform != "email" {
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        continue;
                    }
                }

                let stored_url = if platform == "email" && !url.starts_with("mailto:") {
                    format!("mailto:{}", url)
                } else {
                    url
                };

                sqlx::query!(
                    r#"
                    INSERT INTO user_links (user_id, platform, url, display_text, display_order)
                    VALUES ($1, $2, $3, $4, 0)
                    ON CONFLICT (user_id, platform) 
                    DO UPDATE SET url = $3, display_text = $4
                    "#,
                    auth_user.user_id,
                    platform,
                    stored_url,
                    None as Option<String>
                )
                .execute(&state.pool)
                .await?;
            } else {
                sqlx::query!(
                    "DELETE FROM user_links WHERE user_id = $1 AND platform = $2",
                    auth_user.user_id,
                    platform
                )
                .execute(&state.pool)
                .await?;
            }
        }
    }

    Ok(Html(r#"<div class="toast toast-success">Links saved!</div>"#.to_string()).into_response())
}

/// DELETE /api/users/links/:id - Delete a link
pub async fn delete_user_link(
    State(state): State<AppState>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    auth_user: AuthUser,
    headers: HeaderMap,
    Path(link_id): Path<i32>,
) -> Result<StatusCode> {

    let csrf_token = headers
        .get("X-CSRF-Token")
        .ok_or(AppError::CsrfTokenMissing)?
        .to_str()
        .map_err(|_| AppError::CsrfTokenInvalid)?;

    if !validate_token(&state, csrf_token, auth_user.session_id).await? {
        return Err(AppError::CsrfTokenInvalid);
    }
    
    // Rate limit
    rate_limiter
        .check_user_limit(auth_user.user_id, RateLimitAction::UpdateProfile)
        .await?;

    // Verify ownership
    let link_user_id = sqlx::query_scalar!(
        "SELECT user_id FROM user_links WHERE id = $1",
        link_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if link_user_id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    // Delete link
    sqlx::query!("DELETE FROM user_links WHERE id = $1", link_id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Helper Functions
// ============================================================================

struct RawFeedItem {
    r#type: Option<String>,
    id: Option<i32>,
    created_at: Option<OffsetDateTime>,
}

async fn fetch_user_feed(
    pool: &PgPool,
    user_id: i32,
    cursor: Option<i32>,
    limit: i32,
    current_user_id: Option<i32>,
) -> Result<Vec<UserFeedItem>> {
    let raw_items = if let Some(cursor_id) = cursor {
        sqlx::query_as!(
            RawFeedItem,
            r#"
            (
                SELECT 'post' as type, id, created_at
                FROM posts 
                WHERE user_id = $1 AND deleted_at IS NULL AND is_spam = false AND id < $2
            )
            UNION ALL
            (
                SELECT 'question', id, created_at
                FROM questions 
                WHERE user_id = $1 AND deleted_at IS NULL AND is_spam = false AND id < $2
            )
            UNION ALL
            (
                SELECT 'answer', id, created_at
                FROM answers 
                WHERE user_id = $1 AND deleted_at IS NULL AND is_spam = false AND id < $2
            )
            UNION ALL
            (
                SELECT 'refract', id, created_at
                FROM refracts 
                WHERE user_id = $1 AND deleted_at IS NULL AND is_spam = false AND id < $2
            )
            ORDER BY created_at DESC
            LIMIT $3
            "#,
            user_id,
            cursor_id,
            limit as i64,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            RawFeedItem,
            r#"
            (
                SELECT 'post' as type, id, created_at
                FROM posts 
                WHERE user_id = $1 AND deleted_at IS NULL AND is_spam = false
            )
            UNION ALL
            (
                SELECT 'question', id, created_at
                FROM questions 
                WHERE user_id = $1 AND deleted_at IS NULL AND is_spam = false
            )
            UNION ALL
            (
                SELECT 'answer', id, created_at
                FROM answers 
                WHERE user_id = $1 AND deleted_at IS NULL AND is_spam = false
            )
            UNION ALL
            (
                SELECT 'refract', id, created_at
                FROM refracts 
                WHERE user_id = $1 AND deleted_at IS NULL AND is_spam = false
            )
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            user_id,
            limit as i64,
        )
        .fetch_all(pool)
        .await?
    };

    let mut feed_items = Vec::new();
    
    for raw in raw_items {
        let id = raw.id.unwrap_or(0);
        
        match raw.r#type.as_deref() {
            Some("post") => {
                if let Ok(item) = fetch_user_post_item(pool, id, current_user_id).await {
                    feed_items.push(UserFeedItem::Post(item));
                }
            }
            Some("question") => {
                if let Ok(item) = fetch_user_question_item(pool, id, current_user_id).await {
                    feed_items.push(UserFeedItem::Question(item));
                }
            }
            Some("answer") => {
                if let Ok(item) = fetch_user_answer_item(pool, id, current_user_id).await {
                    feed_items.push(UserFeedItem::Answer(item));
                }
            }
            Some("refract") => {
                if let Ok(item) = crate::handlers::feed::fetch_refract_feed_item(pool, id, current_user_id).await {
                    feed_items.push(UserFeedItem::Refract((item)));
                }
            }
            _ => {}
        }
    }

    Ok(feed_items)
}

async fn fetch_user_post_item(pool: &PgPool, post_id: i32, current_user_id: Option<i32>) -> Result<UserPostItem> {
    let post = sqlx::query!(
        r#"SELECT id, title, content_rendered_html, slug, echo_count, comment_count, refract_count, created_at,
        COALESCE((SELECT true FROM echos WHERE post_id = $1 AND user_id = $2), false) as "has_echoed!"
        FROM posts WHERE id = $1"#,
        post_id,
        current_user_id,
    )
    .fetch_one(pool)
    .await?;

    let tags = sqlx::query_scalar!(
        "SELECT t.slug FROM tags t JOIN post_tags pt ON pt.tag_id = t.id WHERE pt.post_id = $1",
        post_id
    )
    .fetch_all(pool)
    .await?;

    Ok(UserPostItem {
        id: post.id,
        title: post.title,
        content_rendered_html: post.content_rendered_html,
        slug: post.slug,
        tags,
        echo_count: post.echo_count.unwrap_or(0),
        has_echoed: post.has_echoed,
        comment_count: post.comment_count.unwrap_or(0),
        refract_count: post.refract_count.unwrap_or(0),
        created_at: post.created_at,
    })
}

async fn fetch_user_question_item(pool: &PgPool, question_id: i32, current_user_id: Option<i32>) -> Result<UserQuestionItem> {
    let question = sqlx::query!(
        r#"SELECT id, title, content_rendered_html, slug, echo_count, comment_count, answer_count, created_at,
        COALESCE((SELECT true FROM echos WHERE question_id = $1 AND user_id = $2), false) as "has_echoed!"
        FROM questions WHERE id = $1"#,
        question_id,
        current_user_id,
    )
    .fetch_one(pool)
    .await?;

    let tags = sqlx::query_scalar!(
        "SELECT t.slug FROM tags t JOIN question_tags qt ON qt.tag_id = t.id WHERE qt.question_id = $1",
        question_id
    )
    .fetch_all(pool)
    .await?;

    Ok(UserQuestionItem {
        id: question.id,
        title: question.title,
        content_rendered_html: question.content_rendered_html,
        slug: question.slug,
        tags,
        echo_count: question.echo_count.unwrap_or(0),
        has_echoed: question.has_echoed,
        answer_count: question.answer_count.unwrap_or(0),
        comment_count: question.comment_count.unwrap_or(0),
        created_at: question.created_at,
    })
}

async fn fetch_user_answer_item(pool: &PgPool, answer_id: i32, current_user_id: Option<i32>) -> Result<UserAnswerItem> {
    let answer = sqlx::query!(
        r#"SELECT a.id, a.question_id, a.content_rendered_html, a.slug, a.echo_count, a.comment_count, a.created_at,
        q.title as question_title, q.slug as question_slug,
        COALESCE((SELECT true FROM echos WHERE answer_id = $1 AND user_id = $2), false) as "has_echoed!"
        FROM answers a 
        JOIN questions q ON q.id = a.question_id
        WHERE a.id = $1"#,
        answer_id,
        current_user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(UserAnswerItem {
        id: answer.id,
        question_id: answer.question_id,
        question_title: answer.question_title,
        question_slug: answer.question_slug,
        content_rendered_html: answer.content_rendered_html,
        slug: answer.slug,
        echo_count: answer.echo_count.unwrap_or(0),
        has_echoed: answer.has_echoed,
        comment_count: answer.comment_count.unwrap_or(0),
        created_at: answer.created_at,
    })
}

async fn fetch_user_refract_item(pool: &PgPool, refract_id: i32, current_user_id: Option<i32>) -> Result<UserRefractItem> {
    let refract = sqlx::query!(
        r#"SELECT id, original_post_id, content_rendered_html, echo_count, created_at,
        COALESCE((SELECT true FROM echos WHERE refract_id = $1 AND user_id = $2), false) as "has_echoed!"
        FROM refracts WHERE id = $1"#,
        refract_id,
        current_user_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(UserRefractItem {
        id: refract.id,
        original_post_id: refract.original_post_id,
        content_rendered_html: refract.content_rendered_html,
        echo_count: refract.echo_count.unwrap_or(0),
        has_echoed: refract.has_echoed,
        created_at: refract.created_at,
    })
}

fn get_feed_item_id(item: &UserFeedItem) -> i32 {
    match item {
        UserFeedItem::Post(p) => p.id,
        UserFeedItem::Question(q) => q.id,
        UserFeedItem::Answer(a) => a.id,
        UserFeedItem::Refract(r) => r.id,
    }
}

fn validate_url(url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ValidationError::MissingField("URL must start with http:// or https://".to_string()).into());
    }
    if url.len() > 500 {
        return Err(ValidationError::MissingField("URL too long".to_string()).into());
    }
    Ok(())
}

fn validate_username(username: &str) -> Result<()> {
    if username.len() < 3 || username.len() > 30 {
        return Err(ValidationError::MissingField("Username must be 3-30 characters".to_string()).into());
    }
    
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ValidationError::MissingField("Username can only contain letters, numbers, _ and -".to_string()).into());
    }
    
    Ok(())
}

fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

// HELPER FUNCTION
fn convert_to_activities(feed_items: Vec<UserFeedItem>) -> Vec<ProfileFeedItem> {
    feed_items.into_iter().map(|item| {
        match item {
            UserFeedItem::Post(p) => ProfileFeedItem::Activity(ActivityItem {
                activity_type: "post".to_string(),
                id: p.id,
                title: p.title.unwrap_or_else(|| "Unititled Post".to_string()),
                content_rendered_html: p.content_rendered_html,
                slug: Some(p.slug),
                created_at: p.created_at,
                tags: p.tags,
                comment_count: p.comment_count,
                echo_count: p.echo_count,
                has_echoed: p.has_echoed,
                refract_count: Some(p.refract_count),
                answer_count: None,
                question_title: None,
                question_slug: None,
                refract_content: None,
            }),
            UserFeedItem::Question(q) => ProfileFeedItem::Activity(ActivityItem {
                activity_type: "question".to_string(),
                id: q.id,
                title: q.title,
                content_rendered_html: q.content_rendered_html,
                slug: Some(q.slug),
                created_at: q.created_at,
                tags: q.tags,
                comment_count: q.comment_count,
                echo_count: q.echo_count,
                has_echoed: q.has_echoed,
                refract_count: None,
                answer_count: Some(q.answer_count),
                question_title: None,
                question_slug: None,
                refract_content: None,
            }),
            UserFeedItem::Answer(a) => ProfileFeedItem::Activity(ActivityItem {
                activity_type: "answer".to_string(),
                id: a.id,
                title: format!("Answered: {}", a.question_title),
                content_rendered_html: a.content_rendered_html,
                slug: Some(a.slug),
                created_at: a.created_at,
                tags: vec![],
                comment_count: a.comment_count,
                echo_count: a.echo_count,
                has_echoed: a.has_echoed,
                refract_count: None,
                answer_count: None,
                question_title: None,
                question_slug: None,
                refract_content: None,
            }),
            UserFeedItem::Refract(r) => ProfileFeedItem::Refract(r),
        }
    }).collect()
}

pub async fn render_edit_profile_page_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Html<String>> {
    let user = sqlx::query!(
        "SELECT id, username, name, bio_raw, avatar_url FROM users WHERE id = $1",
        auth_user.user_id
    )
    .fetch_one(&state.pool)
    .await?;

    let links = sqlx::query!(
        "SELECT id, platform, url FROM user_links WHERE user_id = $1 ORDER BY display_order",
        auth_user.user_id
    )
    .fetch_all(&state.pool)
    .await?;

    let csrf_token = generate_token(&state, auth_user.session_id)
        .await
        .map_err(|_| AppError::Internal(("Failed to generate CSRF token".to_string())))?;

    let data = crate::templates::profile_edit::EditProfileData {
        user_id: user.id,
        username: user.username.clone(),
        display_name: Some(user.name),
        bio: user.bio_raw,
        avatar_url: user.avatar_url,
        links: links.into_iter().map(|l| crate::templates::profile_edit::EditUserLink {
            id: Some(l.id),
            platform: l.platform,
            url: Some(l.url),
        }).collect(),
    };

    let current_user = (auth_user.user_id, user.username, None);

    let markup = crate::templates::profile_edit::render_edit_profile_page(data, csrf_token, current_user, None,);

    Ok(Html(markup.into_string()))
}