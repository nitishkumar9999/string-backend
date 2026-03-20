use axum::{
    extract::{Path, Query, State},
    response::Html,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::{
    dto::auth, errors::Result, middleware::auth::AuthUser, state::AppState
};

#[derive(Debug, Deserialize)]
pub struct TagQuery {
    pub cursor: Option<i32>,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    25
}

#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub tag: TagInfo,
    pub data: Vec<TagFeedItem>,
    pub next_cursor: Option<i32>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TagFeedItem {
    Post(PostTagItem),
    Question(QuestionTagItem),
}

#[derive(Debug, Serialize)]
pub struct PostTagItem {
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
pub struct QuestionTagItem {
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

#[derive(Debug, Serialize, Clone)]
pub struct TagInfo {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub post_count: i64,
    pub question_count: i64,
}

pub async fn get_tag(
    State(state): State<AppState>,
    Path(tag_slug): Path<String>,
    auth_user: Option<AuthUser>,
    Query(mut query): Query<TagQuery>,
) -> Result<Html<String>> {

    query.limit = query.limit.clamp(1, 100);
    let fetch_limit = (query.limit + 1) as i64;

    let tag = fetch_tag_info(&state.pool, &tag_slug).await?;

    let user_id = auth_user.as_ref().map(|u| u.user_id);

    let items = fetch_tag_items(&state.pool, tag.id, query.cursor, fetch_limit, user_id).await?;

    let has_more = items.len() > query.limit as usize;
    let mut data = items;
    if has_more {
        data.pop();
    }

    let next_cursor = if has_more {
        data.last().map(|item| get_item_id(item))
    } else {
        None
    };

    let user = if let Some(ref auth_user) = auth_user {
        Some(get_user_info(&state.pool, auth_user.user_id).await?)
    } else {
        None
    };

    let csrf_token = if let Some(ref auth) = auth_user {
        crate::middleware::csrf::generate_token(&state, auth.session_id).await.ok()
    } else {
        None
    };

    let markup = crate::templates::tags::render_tag_page(
        &tag, 
        &data,
        has_more,
        next_cursor,
        user,
        csrf_token,
    );

    Ok(Html(markup.into_string()))
   
}

//Helpers

async fn fetch_tag_info(pool: &PgPool, tag_slug: &str) -> Result<TagInfo> {
    let tag = sqlx::query!(
        r#"
        SELECT 
            id,
            name,
            slug,
            (SELECT COUNT(*) FROM post_tags WHERE tag_id = tags.id) as "post_count!", 
            (SELECT COUNT(*) FROM question_tags WHERE tag_id = tags.id) as "question_count!"
        FROM tags
        WHERE slug = $1
        "#,
        tag_slug
    )
    .fetch_one(pool)
    .await?;

    Ok(TagInfo {
        id: tag.id,
        name: tag.name,
        slug: tag.slug,
        post_count: tag.post_count,
        question_count: tag.question_count,
    })
}

#[derive(Debug)]
struct RawTagItem {
    id: Option<i32>,
    item_type: String,
    created_at: Option<OffsetDateTime>,
}

async fn fetch_tag_items(
    pool: &PgPool,
    tag_id: i32,
    cursor: Option<i32>,
    limit: i64,
    user_id: Option<i32>,
) -> Result<Vec<TagFeedItem>> {
    let raw_items = if let Some(cursor_id) = cursor {
        sqlx::query_as!(
            RawTagItem,
            r#"
            (
                SELECT p.id, 'post' as "item_type!", p.created_at
                FROM posts p
                JOIN post_tags pt ON p.id = pt.post_id
                WHERE pt.tag_id = $1
                  AND p.deleted_at IS NULL 
                  AND p.is_spam = false
                  AND p.id < $2
            )
            UNION ALL
            (
                SELECT q.id, 'question' as "item_type!", q.created_at
                FROM questions q
                JOIN question_tags qt ON q.id = qt.question_id
                WHERE qt.tag_id = $1
                  AND q.deleted_at IS NULL 
                  AND q.is_spam = false
                  AND q.id < $2
            )
            ORDER BY created_at DESC
            LIMIT $3
            "#,
            tag_id,
            cursor_id,
            limit,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            RawTagItem,
            r#"
            (
                SELECT p.id, 'post' as "item_type!", p.created_at
                FROM posts p
                JOIN post_tags pt ON p.id = pt.post_id
                WHERE pt.tag_id = $1
                  AND p.deleted_at IS NULL 
                  AND p.is_spam = false
            )
            UNION ALL
            (
                SELECT q.id, 'question' as "item_type!", q.created_at
                FROM questions q
                JOIN question_tags qt ON q.id = qt.question_id
                WHERE qt.tag_id = $1
                  AND q.deleted_at IS NULL 
                  AND q.is_spam = false
            )
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            tag_id,
            limit,
        )
        .fetch_all(pool)
        .await?
    };

    let mut items = Vec::new();
    for raw in raw_items {
        let id = raw.id.unwrap_or(0);
        match raw.item_type.as_str() {
            "post" => {
                if let Ok(post) = fetch_post_tag_item(pool, id, user_id).await {
                    items.push(TagFeedItem::Post(post));
                }
            }
            "question" => {
                if let Ok(question) = fetch_question_tag_item(pool, id, user_id).await {
                    items.push(TagFeedItem::Question(question));
                }
            }
            _ => {}
        }
    }

    Ok(items)
}

async fn fetch_post_tag_item(pool: &PgPool, post_id: i32, user_id: Option<i32>) -> Result<PostTagItem> {
    let post = sqlx::query!(
        r#"
        SELECT p.id, p.user_id, p.title, p.content_rendered_html, p.slug,
               p.echo_count, p.refract_count, p.comment_count, p.created_at,
               u.username, u.avatar_url,
               CASE WHEN $2::int IS NOT NULL THEN EXISTS (
                   SELECT 1 FROM echos WHERE user_id = $2 AND post_id = p.id
               ) ELSE false END as "has_echoed!"
        FROM posts p
        JOIN users u ON p.user_id = u.id
        WHERE p.id = $1
        "#,
        post_id,
        user_id as Option<i32>,
    )
    .fetch_one(pool)
    .await?;

    let tags = sqlx::query_as!(
        TagInfo,
        r#"
        SELECT t.id, t.name, t.slug,
               0::bigint as "post_count!",
               0::bigint as "question_count!"
        FROM tags t
        JOIN post_tags pt ON t.id = pt.tag_id
        WHERE pt.post_id = $1
        "#,
        post_id
    )
    .fetch_all(pool)
    .await?;

    Ok(PostTagItem {
        id: post.id,
        user_id: post.user_id,
        username: post.username,
        avatar_url: post.avatar_url,
        title: post.title,
        content_rendered_html: post.content_rendered_html,
        slug: post.slug,
        tags,
        echo_count: post.echo_count.unwrap_or(0),
        has_echoed: post.has_echoed,
        refract_count: post.refract_count.unwrap_or(0),
        comment_count: post.comment_count.unwrap_or(0),
        created_at: post.created_at,
    })
}

async fn fetch_question_tag_item(pool: &PgPool, question_id: i32, user_id: Option<i32>) -> Result<QuestionTagItem> {
    let question = sqlx::query!(
        r#"
        SELECT q.id, q.user_id, q.title, q.content_rendered_html, q.slug,
               q.echo_count, q.answer_count, q.comment_count, q.created_at,
               u.username, u.avatar_url,
               CASE WHEN $2::int IS NOT NULL THEN EXISTS (
                   SELECT 1 FROM echos WHERE user_id = $2 AND question_id = q.id
               ) ELSE false END as "has_echoed!"
        FROM questions q
        JOIN users u ON q.user_id = u.id
        WHERE q.id = $1
        "#,
        question_id,
        user_id as Option<i32>,
    )
    .fetch_one(pool)
    .await?;

    let tags = sqlx::query_as!(
        TagInfo,
        r#"
        SELECT t.id, t.name, t.slug,
               0::bigint as "post_count!",
               0::bigint as "question_count!"
        FROM tags t
        JOIN question_tags qt ON t.id = qt.tag_id
        WHERE qt.question_id = $1
        "#,
        question_id
    )
    .fetch_all(pool)
    .await?;

    Ok(QuestionTagItem {
        id: question.id,
        user_id: question.user_id,
        username: question.username,
        avatar_url: question.avatar_url,
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

async fn get_user_info(pool: &PgPool, user_id: i32) -> Result<(i32, String, Option<String>)> {
    let user = sqlx::query!(
        "SELECT id, username, avatar_url FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok((user.id, user.username, user.avatar_url))
}

fn get_item_id(item: &TagFeedItem) -> i32 {
    match item {
        TagFeedItem::Post(p) => p.id,
        TagFeedItem::Question(q) => q.id,
    }
}