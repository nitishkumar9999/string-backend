use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};

/// Cursor-based pagination (best for infinite scroll)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPagination {
    /// Cursor (typically the ID of the last item)
    pub cursor: Option<i32>,
    /// Number of items to fetch
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    25
}

impl CursorPagination {
    pub fn new(cursor: Option<i32>) -> Self {
        Self {
            cursor,
            limit: 25,
        }
    }

    /// Validate limit (prevent abuse)
    pub fn validate_limit(&mut self) {
        if self.limit < 1 {
            self.limit = 25;
        }
        if self.limit > 100 {
            self.limit = 100; // Max 100 items per request
        }
    }
}

/// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<i32>,
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(mut data: Vec<T>, limit: usize, get_id: impl Fn(&T) -> i32) -> Self {
        let has_more = data.len() > limit;
        
        // Remove extra item if we have more
        if has_more {
            data.pop();
        }

        let next_cursor = if has_more {
            data.last().map(|item| get_id(item))
        } else {
            None
        };

        Self {
            data,
            next_cursor,
            has_more,
        }
    }
}

// Search Pagination
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cursor {
    pub rank: f32,
    pub matched_tags: i32,
    pub engagement_count: i32,
    pub created_at: time::OffsetDateTime, // Unix timestamp
    pub id: i32,
}

impl Cursor {
    pub fn new(rank: f32, matched_tags: i32, engagement_count: i32, created_at: time::OffsetDateTime, id: i32) -> Self {
        Self {
            rank,
            matched_tags,
            engagement_count,
            created_at,
            id,
        }
    }

    pub fn encode(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        general_purpose::STANDARD.encode(json.as_bytes())
    }

    pub fn decode(cursor: &str) -> Result<Self, String> {
        let bytes = general_purpose::STANDARD
            .decode(cursor)
            .map_err(|e| format!("Invalid base64: {}", e))?;
        let json = String::from_utf8(bytes)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;
        serde_json::from_str(&json)
            .map_err(|e| format!("Invalid JSON: {}", e))
    }
}




