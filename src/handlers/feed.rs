use axum::{
    Json, extract::{Query, State}, http::HeaderMap, response::{Html, IntoResponse, Response}
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;

use crate::errors::{AppError, Result};
use crate::middleware::{
            auth::AuthUser, 
            csrf::generate_token
};
use crate::state::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub cursor: Option<i32>,
    #[serde(default = "default_limit")]
    pub limit: i32,
    pub logout: Option<bool>,
    pub deleted: Option<bool>,
    pub error: Option<String>,

}

fn default_limit() -> i32 {
    25
}

#[derive(Debug, Serialize)]
pub struct FeedResponse {
    pub data: Vec<FeedItem>,
    pub next_cursor: Option<i32>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FeedItem {
    Post(PostFeedItem),
    Question(QuestionFeedItem),
    Refract(RefractFeedItem),
}

#[derive(Debug, Serialize)]
pub struct PostFeedItem {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub avatar_url: Option<String>,
    pub title: Option<String>,
    pub content_rendered_html: String,
    pub slug: String,
    pub tags: Vec<TagInfo>,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub refract_count: i32,
    pub comment_count: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct QuestionFeedItem {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub avatar_url: Option<String>,
    pub title: String,
    pub content_rendered_html: String,
    pub slug: String,
    pub tags: Vec<TagInfo>,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub answer_count: i32,
    pub comment_count: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct RefractFeedItem {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub avatar_url: Option<String>,
    pub content_rendered_html: String,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub created_at: OffsetDateTime,
    pub original_post: OriginalPostPreview,
}

#[derive(Debug, Serialize)]
pub struct OriginalPostPreview {
    pub id: i32,
    pub slug: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub title: Option<String>,
    pub content_rendered_html: String,
    pub created_at: OffsetDateTime,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TagInfo {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

// ============================================================================
// Feed Handler
// ============================================================================

/// GET /api/feed - Unified chronological feed
pub async fn get_feed(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Query(mut query): Query<FeedQuery>,
) -> Result<impl IntoResponse> {
    // Validate and clamp limit
    query.limit = query.limit.clamp(1, 100);
    
    // Fetch one extra to determine if there's more
    let fetch_limit = (query.limit + 1) as i64;

    let current_user_id = auth_user.as_ref().map(|u| u.user_id);

    // Fetch mixed feed items chronologically
    let feed_items = fetch_feed_items(&state.pool, query.cursor, fetch_limit, current_user_id).await?;

    // Build response with pagination
    let has_more = feed_items.len() > query.limit as usize;
    let mut data = feed_items;
    
    if has_more {
        data.pop();
    }

    let next_cursor = if has_more {
        data.last().map(|item| get_item_id(item))
    } else {
        None
    };

    let feed_response = FeedResponse {
        data,
        next_cursor,
        has_more,
    };

    let markup = {
        let (user, csrf_token) = if let Some(auth_user) = auth_user {
            let u = get_user_info(&state.pool, auth_user.user_id).await.ok();
            let t = match generate_token(&state, auth_user.session_id.clone()).await {
                Ok(token) => {
                    tracing::info!("CSRF token generated successfully: {}", token);
                    Some(token)
                }
                Err(e) => {
                    tracing::error!("Failed to generate CSRF token: {:?}", e);
                    None
                }
            };
                
            (u, t)
        } else {
            (None, None)
        };

        crate::templates::feed::render_feed_page(
            feed_response, 
            user, 
            csrf_token, 
            query.logout.unwrap_or(false), 
            query.deleted.unwrap_or(false),
            query.error.as_deref(),
        )
    };

    let mut headers = HeaderMap::new();
    headers.insert("Cache-Control", "no-store".parse().unwrap());

    Ok((headers, Html(markup.into_string())))
}

async fn get_user_info(pool: &PgPool, user_id: i32) -> Result<(i32, String, Option<String>)> {
    let user = sqlx::query!(
        "SELECT id, username, avatar_url FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok((user.id, user.username, user.avatar_url))
}

// ============================================================================
// Helper Functions
// ============================================================================

#[derive(Debug)]
struct RawFeedItem {
    id: Option<i32>,
    item_type: String, // "post", "question", "refract"
    created_at: Option<OffsetDateTime>,
}

async fn fetch_feed_items(
    pool: &PgPool,
    cursor: Option<i32>,
    limit: i64,
    current_user_id: Option<i32>,
) -> Result<Vec<FeedItem>> {
    // First, get mixed IDs and types chronologically
    let raw_items = if let Some(cursor_id) = cursor {
        sqlx::query_as!(
            RawFeedItem,
            r#"
            (
                SELECT id, 'post' as "item_type!", created_at
                FROM posts
                WHERE deleted_at IS NULL 
                  AND is_spam = false
                  AND id < $1
            )
            UNION ALL
            (
                SELECT id, 'question' as "item_type!", created_at
                FROM questions
                WHERE deleted_at IS NULL 
                  AND is_spam = false
                  AND id < $1
            )
            UNION ALL
            (
                SELECT id, 'refract' as "item_type!", created_at
                FROM refracts
                WHERE deleted_at IS NULL 
                  AND is_spam = false
                  AND id < $1
            )
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            cursor_id,
            limit,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            RawFeedItem,
            r#"
            (
                SELECT id, 'post' as "item_type!", created_at
                FROM posts
                WHERE deleted_at IS NULL AND is_spam = false
            )
            UNION ALL
            (
                SELECT id, 'question' as "item_type!", created_at
                FROM questions
                WHERE deleted_at IS NULL AND is_spam = false
            )
            UNION ALL
            (
                SELECT id, 'refract' as "item_type!", created_at
                FROM refracts
                WHERE deleted_at IS NULL AND is_spam = false
            )
            ORDER BY created_at DESC
            LIMIT $1
            "#,
            limit,
        )
        .fetch_all(pool)
        .await?
    };

    // Now fetch full details for each item
    let mut feed_items = Vec::new();
    
    for raw in raw_items {
        let id = match raw.id {
            Some(id) => id,
            None => continue,
        };

        match raw.item_type.as_str() {
            "post" => {
                if let Ok(post) = fetch_post_feed_item(pool, id, current_user_id).await {
                    feed_items.push(FeedItem::Post(post));
                }
            }
            "question" => {
                if let Ok(question) = fetch_question_feed_item(pool, id, current_user_id).await {
                    feed_items.push(FeedItem::Question(question));
                }
            }
            "refract" => {
                if let Ok(refract) = fetch_refract_feed_item(pool, id, current_user_id).await {
                    feed_items.push(FeedItem::Refract(refract));
                }
            }
            _ => {}
        }
    }

    Ok(feed_items)
}

async fn fetch_post_feed_item(pool: &PgPool, post_id: i32, current_user_id: Option<i32>) -> Result<PostFeedItem> {
    let post = sqlx::query!(
        r#"
        SELECT p.id, p.user_id, p.title, p.content_rendered_html, p.slug,
               p.echo_count, p.refract_count, p.comment_count, p.created_at,
               u.username, u.avatar_url,
               COALESCE((SELECT true FROM echos WHERE post_id = p.id AND user_id = $2), false) as "has_echoed!"
        FROM posts p
        JOIN users u ON p.user_id = u.id
        WHERE p.id = $1
        "#,
        post_id,
        current_user_id,
    )
    .fetch_one(pool)
    .await?;

    // Fetch tags
    let tags = sqlx::query_as!(
        TagInfo,
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

    Ok(PostFeedItem {
        id: post.id,
        user_id: post.user_id,
        username: post.username,
        avatar_url: post.avatar_url,
        title: post.title,
        content_rendered_html: post.content_rendered_html,
        slug: post.slug,  // Already a String, not Option<String>
        tags,
        echo_count: post.echo_count.unwrap_or(0),
        has_echoed: post.has_echoed,
        refract_count: post.refract_count.unwrap_or(0),
        comment_count: post.comment_count.unwrap_or(0),
        created_at: post.created_at,
    })
}

async fn fetch_question_feed_item(pool: &PgPool, question_id: i32, current_user_id: Option<i32>) -> Result<QuestionFeedItem> {
    let question = sqlx::query!(
        r#"
        SELECT q.id, q.user_id, q.title, q.content_rendered_html, q.slug,
               q.echo_count, q.answer_count, q.comment_count, q.created_at,
               u.username, u.avatar_url,
               COALESCE((SELECT true FROM echos WHERE question_id = q.id AND user_id = $2), false) as "has_echoed!"
        FROM questions q
        JOIN users u ON q.user_id = u.id
        WHERE q.id = $1
        "#,
        question_id,
        current_user_id,
    )
    .fetch_one(pool)
    .await?;

    // Fetch tags
    let tags = sqlx::query_as!(
        TagInfo,
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

    Ok(QuestionFeedItem {
        id: question.id,
        user_id: question.user_id,
        username: question.username,
        avatar_url: question.avatar_url,
        title: question.title,
        content_rendered_html: question.content_rendered_html,
        slug: question.slug,  // Already a String, not Option<String>
        tags,
        echo_count: question.echo_count.unwrap_or(0),
        has_echoed: question.has_echoed,
        answer_count: question.answer_count.unwrap_or(0),
        comment_count: question.comment_count.unwrap_or(0),
        created_at: question.created_at,
    })
}

pub async fn fetch_refract_feed_item(pool: &PgPool, refract_id: i32, current_user_id: Option<i32>) -> Result<RefractFeedItem> {
    let refract = sqlx::query!(
        r#"
        SELECT r.id, r.user_id, r.original_post_id, r.content_rendered_html,
               r.echo_count, r.created_at, u.username, u.avatar_url,
               COALESCE((SELECT true FROM echos WHERE refract_id = r.id AND user_id = $2), false) as "has_echoed!"
        FROM refracts r
        JOIN users u ON r.user_id = u.id
        WHERE r.id = $1
        "#,
        refract_id,
        current_user_id,
    )
    .fetch_one(pool)
    .await?;

    // Fetch original post preview
    let original = sqlx::query!(
        r#"
        SELECT p.id, p.slug, p.title, p.content_rendered_html, p.created_at, p.deleted_at, u.username, u.avatar_url
        FROM posts p
        JOIN users u ON p.user_id = u.id
        WHERE p.id = $1
        "#,
        refract.original_post_id
    )
    .fetch_one(pool)
    .await?;

    let is_deleted = original.deleted_at.is_some();

    Ok(RefractFeedItem {
        id: refract.id,
        user_id: refract.user_id,
        username: refract.username,
        avatar_url: refract.avatar_url,
        content_rendered_html: refract.content_rendered_html,
        echo_count: refract.echo_count.unwrap_or(0),
        has_echoed: refract.has_echoed,
        created_at: refract.created_at,
        original_post: OriginalPostPreview {
            id: original.id,
            slug: if is_deleted {
                "[deleted]".to_string()
            } else {
                original.slug  // Already a String, not Option<String>
            },
            username: if is_deleted {
                "[deleted]".to_string()
            } else {
                original.username
            },
            avatar_url: if is_deleted { 
                None 
            } else { 
                original.avatar_url 
            },
            title: if is_deleted {
                Some("[deleted]".to_string())
            } else {
                original.title
            },
            content_rendered_html: if is_deleted {
                "[deleted]".to_string()
            } else {
                original.content_rendered_html
            },
            created_at: original.created_at,
            is_deleted,
        },
    })
}

fn get_item_id(item: &FeedItem) -> i32 {
    match item {
        FeedItem::Post(p) => p.id,
        FeedItem::Question(q) => q.id,
        FeedItem::Refract(r) => r.id,
    }
}