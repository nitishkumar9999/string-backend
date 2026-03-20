// src/state.rs
use sqlx::PgPool;
use std::sync::Arc;

use crate::utils::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub base_url: String,
    pub rate_limiter: Arc<RateLimiter>,
}


impl AppState {
    pub fn new(pool: PgPool, rate_limiter: Arc<RateLimiter>) -> Self {
        
        
        let base_url = std::env::var("BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        Self { pool, base_url, rate_limiter }
    }
    
}

impl axum::extract::FromRef<AppState> for std::sync::Arc<RateLimiter> {
    fn from_ref(state: &AppState) -> Self {
        state.rate_limiter.clone()
    }
}

impl axum::extract::FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}