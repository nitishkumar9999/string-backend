

use serde::{Deserialize, Serialize};

// ============================================================================
// SHARED RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize, Clone)]
pub struct TagResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MediaResponse {
    pub id: i32,
    pub media_type: String,
    pub file_path: String,
    pub mime_type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MediaUpload {
    pub data: String, // base64 encoded
    pub filename: String,
}


