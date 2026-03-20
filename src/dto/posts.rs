use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use validator::Validate;

/// Request to create a new post
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostRequest {
    /// Optional title (15-300 chars if provided)
    #[validate(length(min = 15, max = 300, message = "Title must be 15-300 characters"))]
    pub title: Option<String>,
    
    /// Raw markdown content (10-30,000 chars)
    #[validate(length(min = 10, max = 30000, message = "Content must be 10-30,000 characters"))]
    pub content_raw: String,
    
    /// Tag names/slugs to attach (1-5 tags)
    #[validate(length(min = 1, max = 5, message = "Must provide 1-5 tags"))]
    pub tags: Vec<String>,
    
    /// Media IDs uploaded via /api/media/upload (max 5 images + 2 videos)
    #[serde(default)]
    pub media_ids: Vec<i32>,
}

/// Request to update an existing post
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePostRequest {
    /// Optional title (15-300 chars if provided)
    #[validate(length(min = 15, max = 300, message = "Title must be 15-300 characters"))]
    pub title: Option<String>,
    
    /// Raw markdown content (10-30,000 chars)
    #[validate(length(min = 10, max = 30000, message = "Content must be 10-30,000 characters"))]
    pub content_raw: Option<String>,
    
    /// Tag names/slugs to attach (1-5 tags)
    #[validate(length(min = 1, max = 5, message = "Must provide 1-5 tags"))]
    pub tags: Option<Vec<String>>,
    
    /// Media IDs to attach (replaces existing media)
    pub media_ids: Option<Vec<i32>>,
}

/// Response for a single post
#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: i32,
    pub title: Option<String>,
    pub content_rendered_html: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub edited_at: Option<OffsetDateTime>,
    pub author: PostAuthor,
    pub tags: Vec<PostTag>,
    pub media: Vec<PostMedia>,
    pub counts: PostCounts,
}

/// Author information in post response
#[derive(Debug, Serialize)]
pub struct PostAuthor {
    pub id: i32,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Tag information in post response
#[derive(Debug, Serialize)]
pub struct PostTag {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

/// Media information in post response
#[derive(Debug, Serialize)]
pub struct PostMedia {
    pub id: i32,
    pub media_type: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub mime_type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<i32>,
    pub display_order: i32,
}

/// Post interaction counts
#[derive(Debug, Serialize)]
pub struct PostCounts {
    pub views: i32,
    pub echoes: i32,
    pub refracts: i32,
    pub comments: i32,
}

/// List posts response
#[derive(Debug, Serialize)]
pub struct PostListResponse {
    pub posts: Vec<PostResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_more: bool,
}

/// Query parameters for listing posts
#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    
    #[serde(default = "default_page_size")]
    pub page_size: i32,
    
    /// Filter by author username
    pub author: Option<String>,
    
    /// Filter by tag slug
    pub tag: Option<String>,
    
    /// Sort order: "recent", "popular", "trending"
    #[serde(default = "default_sort")]
    pub sort: String,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

fn default_sort() -> String {
    "recent".to_string()
}

impl ListPostsQuery {
    pub fn validate(&self) -> Result<(), String> {
        if self.page < 1 {
            return Err("Page must be >= 1".to_string());
        }
        
        if !(1..=100).contains(&self.page_size) {
            return Err("Page size must be 1-100".to_string());
        }
        
        if !["recent", "popular", "trending"].contains(&self.sort.as_str()) {
            return Err("Sort must be: recent, popular, or trending".to_string());
        }
        
        Ok(())
    }
    
    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.page_size) as i64
    }
    
    pub fn limit(&self) -> i64 {
        self.page_size as i64
    }
}

/// Database row for post with author
#[derive(Debug, sqlx::FromRow)]
pub struct PostWithAuthor {
    // Post fields
    pub id: i32,
    pub user_id: i32,
    pub title: Option<String>,
    pub content_raw: String,
    pub content_rendered_html: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub edited_at: Option<OffsetDateTime>,
    pub view_count: i32,
    pub echo_count: i32,
    pub refract_count: i32,
    pub comment_count: i32,
    
    // Author fields
    pub author_username: String,
    pub author_display_name: Option<String>,
    pub author_avatar_url: Option<String>,
}

impl PostWithAuthor {
    pub fn to_response(
        self,
        tags: Vec<PostTag>,
        media: Vec<PostMedia>,
    ) -> PostResponse {
        PostResponse {
            id: self.id,
            title: self.title,
            content_rendered_html: self.content_rendered_html,
            created_at: self.created_at,
            updated_at: self.updated_at,
            edited_at: self.edited_at,
            author: PostAuthor {
                id: self.user_id,
                username: self.author_username,
                display_name: self.author_display_name,
                avatar_url: self.author_avatar_url,
            },
            tags,
            media,
            counts: PostCounts {
                views: self.view_count,
                echoes: self.echo_count,
                refracts: self.refract_count,
                comments: self.comment_count,
            },
        }
    }
}