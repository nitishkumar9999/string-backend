use axum::{
    extract::{Query, State},
    Json,
    response::Html,
};
use reqwest::header::HeaderMap;
use serde::ser::Impossible;
use sqlx::PgPool;
use ammonia::{Builder, UrlRelative};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{dto::search::{SearchMeta, SearchQuery, SearchResponse, SearchResult}, errors::AppError};
use crate::utils::pagination::Cursor;
use crate::state::AppState;
use crate::errors::{ValidationError, Result};
use crate::middleware::{auth::AuthUser, csrf::generate_token};
use crate::templates::search::{SearchPostResult, SearchQuestionResult, SearchResults};

// =====================================================
// 1. SEARCH SANITIZATION
// =====================================================

static TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9+#.\-_]+$").expect("TAG_REGEX compilation failed - invalid pattern")
});

pub fn sanitize_search_query(input: &str) -> String {
    let cleaned = Builder::default()
        .clean_content_tags(
            ["script", "style", "iframe", "object", "embed"]
                .iter()
                .cloned()
                .collect()
        )
        .url_relative(UrlRelative::PassThrough)
        .clean(input);

    cleaned.to_string().trim().to_string()
}

pub fn sanitize_tag_name(input: &str) -> Option<String> {
    let cleaned = sanitize_search_query(input);
    let tag = cleaned.trim().to_lowercase();
    
    if tag.len() < 2 || tag.len() > 30 {
        return None;
    }

    if TAG_REGEX.is_match(&tag) {
        Some(tag)
    } else {
        None
    }
}

pub fn sanitize_display_text(input: &str) -> String {
    Builder::default()
        .link_rel(Some("nofollow noopener noreferrer".into()))
        .clean(input).to_string()
}

pub fn validate_search_query_length(query: &str) -> Result<()> {
    const MAX_QUERY_LENGTH: usize = 200;
    if query.len() > MAX_QUERY_LENGTH {
        return Err(AppError::Validation(ValidationError::SearchQueryTooLong { max :MAX_QUERY_LENGTH }));
    }
    Ok(())
}

pub fn validate_tag_count(tags: &[String]) -> Result<()> {
    const MAX_TAGS: usize = 5;

    if tags.len() > MAX_TAGS {
        return Err(AppError::Validation(ValidationError::TagLimitExceeding { max: MAX_TAGS }));
    }

    Ok(())
}

// =====================================================
// 4. QUERY PARSER - Extract tags from search query
// =====================================================

#[derive(Debug)]
struct ParsedQuery {
    tags: Vec<String>,
    text: String,
}

fn parse_search_query(query: &str) -> ParsedQuery {
    let sanitized = sanitize_search_query(query);
    let mut tags = Vec::new();
    let mut text_parts = Vec::new();

    // Split by whitespace
    for word in sanitized.split_whitespace() {
        // Check if word starts with / (tag prefix like /rust /javascript)
        if word.starts_with('/') {
            let tag = word.trim_start_matches('/');

            if let Some(clean_tag) = sanitize_tag_name(tag) {
                if !tags.contains(&clean_tag) {
                    tags.push(clean_tag);
                }
            } 
        }else {
                text_parts.push(word);
            }
    }

    ParsedQuery {
        tags,
        text: text_parts.join(" "),
    }
}

// =====================================================
// 5. TAG RESOLVER - Convert tag names to IDs
// =====================================================

async fn resolve_tag_ids(pool: &PgPool, tag_names: &[String]) -> Result<Vec<i32>> {
    if tag_names.is_empty() {
        return Ok(Vec::new());
    }

    let result = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM tags WHERE slug = ANY($1)"
    )
    .bind(tag_names)
    .fetch_all(pool)
    .await?;

    Ok(result)
}


// =====================================================
// 6. MAIN SEARCH HANDLER WITH CURSOR PAGINATION
// =====================================================

// GET /api/search?q=/rust+async+programming&limit=20&cursor=xyz123
// Updated search handler - Custom AppError handling
pub async fn search(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Query(params): Query<SearchQuery>,
) -> Result<Html<String>> {

    if params.q.trim().is_empty() {
        let current_user = if let Some(ref auth) = auth_user {
            let username = sqlx::query_scalar!("SELECT username FROM users WHERE id = $1", auth.user_id)
                .fetch_one(&state.pool)
                .await?;
            Some((auth.user_id, username, None))
        } else {
            None
        };

        let csrf_token = if let Some(auth) = &auth_user {
            generate_token(&state, auth.session_id).await.ok()
        } else {
            None
        };

        let markup = crate::templates::search::render_search_page(None, None, current_user, None);
        return Ok(Html(markup.into_string()));
    }

    validate_search_query_length(&params.q)?;
    let parsed = parse_search_query(&params.q);
    validate_tag_count(&parsed.tags)?;

    let search_mode = match params.mode.as_str() {
        "precise" => "precise",
        "all" => "all",
        _ => "precise",
    };

    let tag_ids = resolve_tag_ids(&state.pool, &parsed.tags)
        .await
        .map_err(|_| AppError::Internal("Failed to resolve tags".to_string()))?;

    let impossible = (!parsed.tags.is_empty() && tag_ids.is_empty()) 
        || (search_mode == "precise" && tag_ids.len() != parsed.tags.len());

    if impossible {
        let current_user = get_current_user(&state, &auth_user).await?;
        let csrf_token = get_csrf_token(&state, &auth_user).await;
        let markup = crate::templates::search::render_search_page(
            Some(&params.q), 
            Some(SearchResults { posts: vec![], questions: vec![], total: 0, mode: search_mode.to_string() }), 
            current_user, 
            csrf_token,
        );
        return Ok(Html(markup.into_string()));
    }

    // Determine content types to search
    let content_types = match params.content_type.as_deref() {
        Some("post") => vec!["posts".to_string()],
        Some("question") => vec!["questions".to_string()],
        _ => vec!["posts".to_string(), "questions".to_string()],
    };

    // Prepare search text (None if empty, only tags)
    let search_text = if parsed.text.is_empty() {
        None
    } else {
        Some(parsed.text.clone())
    };

    // Decode cursor if provided
    let (cursor_rank, cursor_matched_tags, cursor_engagement, cursor_created_at, cursor_id) = 
        if let Some(cursor_str) = &params.cursor {
            let cursor = Cursor::decode(cursor_str)
                .map_err(|_| AppError::Internal(cursor_str.to_string()))?;
                
            (
                Some(cursor.rank),
                Some(cursor.matched_tags),
                Some(cursor.engagement_count),
                Some(cursor.created_at),
                Some(cursor.id),
            )
        } else {
            (None, None, None, None, None)
        };

    // Execute unified search using DB function
    let mut results: Vec<SearchResult> = sqlx::query_as::<_, SearchResult>(
        "SELECT * FROM search_all($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(&search_text)
    .bind(if tag_ids.is_empty() { None } else { Some(tag_ids.as_slice()) })
    .bind(&content_types)
    .bind(search_mode)
    .bind(params.limit)
    .bind(cursor_rank)
    .bind(cursor_matched_tags)
    .bind(cursor_engagement)
    .bind(cursor_created_at)
    .bind(cursor_id)
    .fetch_all(&state.pool)  // Fixed: use state.pool
    .await
    .map_err(|e| {
        eprintln!("SQL ERROR: {:?}", e);    
        AppError::Internal(format!("Search failed: {}", e))
    })?;  // Custom Search error

    let mut actual_mode = search_mode;

    if search_mode == "precise" && results.is_empty() {
        results = sqlx::query_as::<_, SearchResult>(
            "SELECT * FROM search_all($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
            )
            .bind(&search_text)
            .bind(if tag_ids.is_empty() { None } else { Some(tag_ids.as_slice()) })
            .bind(&content_types)
            .bind("all")  // switch to OR
            .bind(params.limit)
            .bind(None::<f32>)
            .bind(None::<i32>)
            .bind(None::<i32>)
            .bind(None::<time::OffsetDateTime>)
            .bind(None::<i32>)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Search failed: {}", e)))?;

        actual_mode = "all";
    }

    for result in &mut results {
        if let Some(title) = result.title.clone() {
            result.title = Some(sanitize_display_text(&title));
        }
        result.content_raw = sanitize_display_text(&result.content_raw);
        
    }

    let current_user = if let Some(ref auth) = auth_user {
        let username = sqlx::query_scalar!("SELECT username FROM users WHERE id = $1", auth.user_id)
            .fetch_one(&state.pool)
            .await?;
        Some((auth.user_id, username, None))
    } else {
        None
    };

    let mut search_results = convert_to_template_results(&state.pool, &results, auth_user.as_ref().map(|a| a.user_id)).await?;
    search_results.mode = actual_mode.to_string();

    let csrf_token = if let Some(auth) = &auth_user {
        generate_token(&state, auth.session_id).await.ok()
    } else {
        None
    };

    tracing::info!(
        search_mode = search_mode,
        actual_mode = actual_mode,
        results_count = results.len(),
        "Search mode debug"
    );

    tracing::info!(
        search_text = ?search_text,
        tag_ids = ?tag_ids,
        parsed_tags = ?parsed.tags,
        "Search params"
    );

    let markup = crate::templates::search::render_search_page(Some(&params.q), Some(search_results), current_user, csrf_token);
    Ok(Html(markup.into_string()))
}


async fn convert_to_template_results(pool: &PgPool, results: &[SearchResult], current_user_id: Option<i32>) -> Result<SearchResults> {

    let post_ids: Vec<i32> = results.iter()
        .filter(|r| r.content_type == "post")
        .map(|r| r.id)
        .collect();

    let question_ids: Vec<i32> = results.iter()
        .filter(|r| r.content_type == "question")
        .map(|r| r.id)
        .collect();

    let echoed_post_ids: Vec<i32> = if let Some(uid) = current_user_id {
        sqlx::query_scalar!(
            "SELECT post_id FROM echos WHERE post_id = ANY($1) AND user_id = $2",
            &post_ids,
            uid
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .collect()
    } else {
        vec![]
    };

    let echoed_question_ids: Vec<i32> = if let Some(uid) = current_user_id {
        sqlx::query_scalar!(
            "SELECT question_id FROM echos WHERE question_id = ANY($1) AND user_id = $2",
            &question_ids,
            uid
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .collect()
    } else {
        vec![]
    };

    let mut posts = Vec::new();
    let mut questions = Vec::new();

    for result in results {
        match result.content_type.as_str() {
            "post" => posts.push(SearchPostResult {
                id: result.id,
                title: result.title.clone(),
                slug: result.slug.clone(),
                username: result.username.clone(),
                avatar_url: result.avatar_url.clone(), 
                content_rendered_html: result.content_rendered_html.clone(),
                echo_count: result.engagement_count,
                comment_count: result.comment_count,
                created_at: result.created_at,
                refract_count: result.refract_count,
                tags: result.tag_names.clone().unwrap_or_default(),
                has_echoed: echoed_post_ids.contains(&result.id),
            }),
            "question" => questions.push(SearchQuestionResult {
                id: result.id,
                title: result.title.clone().unwrap_or_default(),
                slug: result.slug.clone(),
                username: result.username.clone(),
                avatar_url: result.avatar_url.clone(), 
                content_rendered_html: result.content_rendered_html.clone(),
                echo_count: result.engagement_count,
                answer_count: result.answer_count,
                comment_count: result.comment_count,
                created_at: result.created_at,
                tags: result.tag_names.clone().unwrap_or_default(),
                has_echoed: echoed_question_ids.contains(&result.id),
            }),
            _ => {}
        }
    }

    Ok(SearchResults {
        posts,
        questions,
        total: results.len(),
        mode: String::new(),
    })
}

async fn get_current_user(state: &AppState, auth_user: &Option<AuthUser>) -> Result<Option<(i32, String, Option<String>)>> {
    if let Some(ref auth) = auth_user {
        let username = sqlx::query_scalar!("SELECT username FROM users WHERE id = $1", auth.user_id)
            .fetch_one(&state.pool)
            .await?;
        Ok(Some((auth.user_id, username, None)))
    } else {
        Ok(None)
    }
}

async fn get_csrf_token(state: &AppState, auth_user: &Option<AuthUser>) -> Option<String> {
    if let Some(auth) = auth_user {
        generate_token(state, auth.session_id).await.ok()
    } else {
        None
    }
}