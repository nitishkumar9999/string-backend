// dto/auth.rs - MATCHING YOUR MVP SCHEMA
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// OAuth callback query params
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

// GitHub OAuth token response
#[derive(Debug, Deserialize)]
pub struct GitHubTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

// GitHub user response
#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String, // username
    pub name: Option<String>,
    pub avatar_url: String,
    pub bio: Option<String>,
}

// User info for responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}