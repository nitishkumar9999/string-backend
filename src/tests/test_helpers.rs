use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
use serde_json::Value;

use crate::{
    state::AppState,
    rate_limiter::RateLimiter,
    routes::create_router,
};

// ============================================================================
// TEST DATABASE SETUP
// ============================================================================

/// Create a test database pool
/// Each test gets its own isolated database
pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/refract_test".to_string());
    
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Create a test database with a unique name
pub async fn create_unique_test_db() -> (PgPool, String) {
    let base_url = std::env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgresql://localhost".to_string());
    
    let db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace("-", ""));
    
    // Connect to postgres database to create test db
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&base_url)
        .await
        .expect("Failed to connect to postgres");
    
    // Create test database
    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&pool)
        .await
        .expect("Failed to create test database");
    
    // Connect to the new test database
    let test_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!("{}/{}", base_url, db_name))
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&test_pool)
        .await
        .expect("Failed to run migrations");
    
    (test_pool, db_name)
}

/// Clean up test database
pub async fn cleanup_test_db(db_name: &str) {
    let base_url = std::env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgresql://localhost".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&base_url)
        .await
        .expect("Failed to connect to postgres");
    
    // Force disconnect all connections
    sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
        db_name
    ))
    .execute(&pool)
    .await
    .ok();
    
    // Drop database
    sqlx::query(&format!("DROP DATABASE IF EXISTS {}", db_name))
        .execute(&pool)
        .await
        .ok();
}

// ============================================================================
// TEST APP SETUP
// ============================================================================

/// Create a test app with all routes configured
pub fn create_test_app(pool: PgPool) -> Router {
    let state = Arc::new(AppState {
        pool,
        base_url: "http://localhost:3000".to_string(),
    });
    
    let rate_limiter = Arc::new(RateLimiter::new());
    
    create_router(state, rate_limiter)
}

// ============================================================================
// TEST FIXTURES - USER
// ============================================================================

#[derive(Debug)]
pub struct TestUser {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub session_id: Uuid,
}

/// Create a test user and session
pub async fn create_test_user(pool: &PgPool, username: &str) -> TestUser {
    let user = sqlx::query!(
        r#"
        INSERT INTO users (username, name, avatar_url)
        VALUES ($1, $2, $3)
        RETURNING id, username, name
        "#,
        username,
        format!("{} Name", username),
        Some(format!("https://avatar.com/{}.png", username))
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test user");
    
    let session_id = Uuid::new_v4();
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);
    
    sqlx::query!(
        r#"
        INSERT INTO sessions (id, user_id, expires_at)
        VALUES ($1, $2, $3)
        "#,
        session_id,
        user.id,
        expires_at
    )
    .execute(pool)
    .await
    .expect("Failed to create test session");
    
    TestUser {
        id: user.id,
        username: user.username,
        name: user.name,
        session_id,
    }
}

// ============================================================================
// TEST FIXTURES - CONTENT
// ============================================================================

#[derive(Debug)]
pub struct TestPost {
    pub id: i32,
    pub user_id: i32,
    pub slug: String,
    pub title: Option<String>,
}

/// Create a test post
pub async fn create_test_post(pool: &PgPool, user_id: i32, title: Option<&str>) -> TestPost {
    let content = "This is test post content with enough text to pass validation.";
    
    let post = sqlx::query!(
        r#"
        INSERT INTO posts (user_id, title, content_raw, content_rendered_html, content_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, slug, title
        "#,
        user_id,
        title,
        content,
        format!("<p>{}</p>", content),
        format!("{:x}", md5::compute(content))
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test post");
    
    TestPost {
        id: post.id,
        user_id: post.user_id,
        slug: post.slug.unwrap_or_default(),
        title: post.title,
    }
}

#[derive(Debug)]
pub struct TestQuestion {
    pub id: i32,
    pub user_id: i32,
    pub slug: String,
    pub title: String,
}

/// Create a test question
pub async fn create_test_question(pool: &PgPool, user_id: i32, title: &str) -> TestQuestion {
    let content = "This is test question content with enough text to pass validation.";
    
    let question = sqlx::query!(
        r#"
        INSERT INTO questions (user_id, title, content_raw, content_rendered_html, content_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, slug, title
        "#,
        user_id,
        title,
        content,
        format!("<p>{}</p>", content),
        format!("{:x}", md5::compute(content))
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test question");
    
    TestQuestion {
        id: question.id,
        user_id: question.user_id,
        slug: question.slug.unwrap_or_default(),
        title: question.title,
    }
}

#[derive(Debug)]
pub struct TestComment {
    pub id: i32,
    pub user_id: i32,
    pub post_id: Option<i32>,
    pub depth_level: i32,
}

/// Create a test comment
pub async fn create_test_comment(
    pool: &PgPool,
    user_id: i32,
    post_id: Option<i32>,
    parent_comment_id: Option<i32>,
    depth_level: i32,
) -> TestComment {
    let content = "This is a test comment.";
    
    let comment = sqlx::query!(
        r#"
        INSERT INTO comments (user_id, post_id, parent_comment_id, content_raw, content_rendered_html, depth_level)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, post_id, depth_level
        "#,
        user_id,
        post_id,
        parent_comment_id,
        content,
        format!("<p>{}</p>", content),
        depth_level
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test comment");
    
    TestComment {
        id: comment.id,
        user_id: comment.user_id,
        post_id: comment.post_id,
        depth_level: comment.depth_level.unwrap_or(0),
    }
}

// ============================================================================
// REQUEST HELPERS
// ============================================================================

/// Create an authenticated request with session cookie
pub fn authenticated_request(method: &str, uri: &str, session_id: Uuid) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("cookie", format!("session_id={}", session_id))
        .body(Body::empty())
        .unwrap()
}

/// Create an authenticated request with JSON body
pub fn authenticated_json_request(
    method: &str,
    uri: &str,
    session_id: Uuid,
    body: Value,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("cookie", format!("session_id={}", session_id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Create an authenticated request with CSRF token
pub fn authenticated_csrf_request(
    method: &str,
    uri: &str,
    session_id: Uuid,
    csrf_token: &str,
    body: Value,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("cookie", format!("session_id={}", session_id))
        .header("x-csrf-token", csrf_token)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Create an unauthenticated request
pub fn unauthenticated_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

// ============================================================================
// RESPONSE HELPERS
// ============================================================================

/// Extract JSON from response body
pub async fn extract_json(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .expect("Failed to read body");
    serde_json::from_slice(&bytes).expect("Failed to parse JSON")
}

/// Extract status code from response
pub fn extract_status(response: &axum::response::Response) -> StatusCode {
    response.status()
}

// ============================================================================
// ASSERTION HELPERS
// ============================================================================

/// Assert that a response has the expected status code
#[macro_export]
macro_rules! assert_status {
    ($response:expr, $expected:expr) => {
        assert_eq!(
            $response.status(),
            $expected,
            "Expected status {}, got {}",
            $expected,
            $response.status()
        );
    };
}

/// Assert that JSON contains a field with a specific value
#[macro_export]
macro_rules! assert_json_field {
    ($json:expr, $field:expr, $expected:expr) => {
        assert_eq!(
            $json[$field],
            $expected,
            "Expected field '{}' to be {:?}, got {:?}",
            $field,
            $expected,
            $json[$field]
        );
    };
}

/// Assert that JSON contains a field
#[macro_export]
macro_rules! assert_json_has_field {
    ($json:expr, $field:expr) => {
        assert!(
            $json.get($field).is_some(),
            "Expected JSON to have field '{}'",
            $field
        );
    };
}

// ============================================================================
// CLEANUP HELPERS
// ============================================================================

/// Clear all data from test database
pub async fn clear_database(pool: &PgPool) {
    sqlx::query!("TRUNCATE users, posts, questions, answers, comments, refracts, echoes, sessions, csrf_tokens CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clear database");
}

/// Create a CSRF token for testing
pub async fn create_test_csrf_token(pool: &PgPool, session_id: Uuid) -> String {
    let token = Uuid::new_v4().to_string();
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::hours(2);
    
    sqlx::query!(
        "INSERT INTO csrf_tokens (session_id, token, expires_at) VALUES ($1, $2, $3)",
        session_id,
        token,
        expires_at
    )
    .execute(pool)
    .await
    .expect("Failed to create CSRF token");
    
    token
}
