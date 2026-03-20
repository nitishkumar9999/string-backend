// models/user.rs
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i32,
    pub github_id: Option<i64>,
    pub github_username: Option<String>,
    pub username: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub bio_raw: Option<String>,
    pub bio_rendered_html: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: uuid::Uuid,
    pub user_id: i32,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_used_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

// For creating new users
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub github_id: Option<i64>,
    pub github_username: Option<String>,
    pub username: String,
    pub name: String,
    pub avatar_url: Option<String>,
}