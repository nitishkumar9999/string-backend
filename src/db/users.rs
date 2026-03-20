// db/users.rs
use sqlx::PgPool;
use crate::{
    models::user::{User, CreateUser},
    errors::Result,
};

/// Find user by GitHub ID
pub async fn find_by_github_id(pool: &PgPool, github_id: i64) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, github_id, github_username,
               username, name, avatar_url, bio_raw, bio_rendered_html,
               created_at, updated_at
        FROM users
        WHERE github_id = $1 AND deleted_at IS NULL
        "#,
        github_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}



/// Find user by username
pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, github_id, github_username,
               username, name, avatar_url, bio_raw, bio_rendered_html,
               created_at, updated_at
        FROM users
        WHERE username = $1 AND deleted_at IS NULL
        "#,
        username
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Create new user
pub async fn create_user(pool: &PgPool, new_user: CreateUser) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (
            github_id, github_username,
            username, name, avatar_url
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING 
            id, github_id, github_username,
            username, name, avatar_url, bio_raw, bio_rendered_html,
            created_at, updated_at
        "#,
        new_user.github_id,
        new_user.github_username,
        new_user.username,
        new_user.name,
        new_user.avatar_url
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Generate unique username from base (handles collisions)
pub async fn generate_unique_username(pool: &PgPool, base: &str) -> Result<String> {
    let mut username = base.to_lowercase();
    let mut counter = 1;

    // Try username, username1, username2, etc until we find available one
    loop {
        if find_by_username(pool, &username).await?.is_none() {
            return Ok(username);
        }
        username = format!("{}{}", base, counter);
        counter += 1;

        if counter > 100 {
            // Fallback to random suffix
            use rand::Rng;
            let random: u32 = rand::thread_rng().gen_range(1000..9999);
            username = format!("{}{}", base, random);
            break;
        }
    }

    Ok(username)
}