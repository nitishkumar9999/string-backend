mod utils; 
mod errors;
mod middleware;
mod state;
mod handlers;
mod db;
mod models;
mod dto;
mod markdown;
mod routes;
mod templates;

use std::net::SocketAddr;
use std::time::Duration;
use anyhow::Ok;
use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::sync::Arc;
use tracing_subscriber::{
    layer::SubscriberExt, 
    util::SubscriberInitExt,
    EnvFilter
};

use crate::{
    utils::rate_limit::{RateLimiter, RateLimitConfig},
    routes::routes::create_router,
};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    println!("✅ Connected to PostgreSQL successfully!");
    
    // Create rate limiter configuration
    let rate_limit_config = RateLimitConfig {
            posts_per_hour: 10,
            posts_per_day: 50,
            questions_per_hour: 5,
            questions_per_day: 20,
            comments_per_minute: 5,
            comments_per_hour: 100,
            comments_per_day: 500,
            answers_per_hour: 20,
            answers_per_day: 100,
            refracts_per_hour: 15,
            refracts_per_day: 100,
            tag_creation_per_hour: 3,
            tag_creation_per_day: 10,
            anonymous_per_minute: 30,
            anonymous_per_hour: 500,
            echo_per_minute: 10,
            update_profile_per_hour: 5,
            update_username_per_day: 1,
            max_requests: 100,
            window_secs: 60,
    };
    
    // Create rate limiter with pool and config
    let rate_limiter = Arc::new(RateLimiter::new(pool.clone(), rate_limit_config));
    
    // Create app state with both pool and rate_limiter
    let state = state::AppState::new(pool.clone(), rate_limiter);

    // Create router with single state parameter
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    println!("🚀 Server starting on http://0.0.0.0:3001");
    axum::serve(
        listener, 
        app.into_make_service_with_connect_info::<SocketAddr>()
    )
    .with_graceful_shutdown(shutdown_signal(pool))
    .await?;

    Ok(())
}

async fn shutdown_signal(pool: sqlx::PgPool) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
             .await
             .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {tracing::info!("Recieved Ctrl+C");},
        _ = sigterm => {tracing::info!("Recieved SIGTERM");},
    }

    tracing::info!("Shutting down - draining in-flight requests...");

    pool.close().await;
    tracing::info!("Database pool closed. Hasta la Vista! 👋")
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    }

    #[cfg(debug_assertions)]
    {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_line_number(true)
            )
            .init();
    }

    tracing::info!("🔍 Tracing initialized");
}