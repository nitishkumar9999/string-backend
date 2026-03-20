use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(default = "default_query")]
    pub q: String,                    // search query (required)

    pub content_type: Option<String>, // "posts", "questions", or "all" (default: "all")
    #[serde(default = "default_limit")]
    pub limit: i32,
    pub cursor: Option<String>,       // base64 encoded cursor for pagination
    #[serde(default = "default_mode")]
    pub mode: String,
}
fn default_query() -> String {
    String::new()
}
fn default_mode() -> String {
    "precise".to_string()
}

fn default_limit() -> i32 { 20 }


#[derive(Serialize, sqlx::FromRow)]
pub struct SearchResult {
    pub content_type: String,         // "post" or "question"
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub avatar_url: Option<String>,
    pub title: Option<String>,
    pub content_raw: String,
    pub content_rendered_html: String,
    pub created_at: time::OffsetDateTime,
    pub echo_count: i32,
    pub refract_count: i32,
    pub comment_count: i32,
    pub answer_count: i32,
    pub engagement_count: i32,        // total engagement
    pub rank: f32,                    // relevance score
    pub slug: String,
    pub is_spam: bool,
    pub matched_tags: i32,
    pub tag_names: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub success: bool,
    pub data: Vec<SearchResult>,
    pub meta: SearchMeta,
}

#[derive(Serialize)]
pub struct SearchMeta {
    pub query: String,
    pub parsed_tags: Vec<String>,     // tags found in query
    pub parsed_text: String,          // remaining text
    pub limit: i32,
    pub count: usize,
    pub next_cursor: Option<String>,  // cursor for next page
    pub has_more: bool,               // whether there are more results
    pub content_type: String,
    pub mode: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}