use axum::{
    BoxError, Router, error_handling::HandleErrorLayer, http::{Method, StatusCode}, middleware, response::Redirect, routing::{delete, get, patch, post},
    extract::Query,
};
use tower::{ServiceBuilder, limit::rate, timeout::TimeoutLayer};
use std::{sync::Arc, time::Duration};
use tower_http::{
    cors::{CorsLayer, Any, AllowOrigin},
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse},
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile}, 
};
use std::collections::HashMap;
use mime_guess;
use tracing::Level;
use tracing_subscriber::{
    layer::SubscriberExt, 
    util::SubscriberInitExt,
    EnvFilter
};
use axum::http::header;

use crate::{state::AppState, utils::rate_limit::RateLimiter};
use crate::middleware::{
    auth::{auth_middleware, optional_auth_middleware, csrf_middleware, rate_limit_middleware},
    csrf::get_token,
    security::security_headers_middleware,
};
use crate::handlers::{
    auth::{
        github_login, github_callback, logout, logout_all, me,
        list_sessions, delete_session, delete_account,
    },
    posts::{
        create_post, get_post_by_slug_or_id, update_post, delete_post,
        render_create_post_page_handler,
    },
    questions::{
        create_question, get_question_by_slug, update_question, delete_question,
        get_question_answers, render_create_question_page_handler
    },
    answers::{
        create_answer, get_answer_by_slug, update_answer, delete_answer,
    },
    comments::{
        create_comment, get_comment, update_comment, delete_comment, 
        get_post_comments, get_question_comments, get_answer_comments, 
        get_comment_replies, mark_helpful, remove_helpful,
    },
    refracts::{
        create_refract, get_refract, update_refract, delete_refract,
        get_post_refracts,
    },
    echo::create_echo,
    feed::get_feed,
    users::{
        get_user_profile, get_user_feed, update_profile, update_username,
        add_user_link, update_user_link, delete_user_link, render_edit_profile_page_handler,
        update_links_bulk,
    },
    search::search,
    tags::get_tag,
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(public_routes(state.clone()))
        .merge(optional_auth_routes(state.clone()))
        .merge(protected_routes(state.clone()))
                .layer(create_trace_layer())

                .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))

                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    rate_limit_middleware,
                ))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    security_headers_middleware,
                ))
                .layer(create_cors_layer())
        .with_state(state)
}

fn public_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest_service("/static", ServeDir::new("./static")
            .append_index_html_on_directories(true)
        )
        .route("/", get(home_page))
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
        .route("/auth/github", get(github_login))
        
}

fn optional_auth_routes(state: AppState) -> Router<AppState> {
    Router::new()

        .route("/auth/github/callback", get(github_callback))
        // ========== FEEDS ==========
        .route("/feed", get(get_feed))

        .route("/tags/:slug", get(get_tag))
        
        // ========== SEARCH ==========
        .route("/search", get(search))
        
        // ========== CURRENT USER ==========
        .route("/me", get(me))
        
        // ========== POSTS ==========
        .route("/posts/:slug_or_id/comments", get(get_post_comments))
        .route("/posts/:slug_or_id/refracts", get(get_post_refracts))
        .route("/posts/:slug_or_id", get(get_post_by_slug_or_id))
        
        // ========== QUESTIONS & ANSWERS ==========
        .route("/questions/:slug/answers", get(get_question_answers))
        .route("/questions/:slug/comments", get(get_question_comments))
        .route("/questions/:slug", get(get_question_by_slug))
        .route("/answers/:slug/comments", get(get_answer_comments))
        .route("/answers/:slug", get(get_answer_by_slug))
        
        // ========== COMMENTS ==========
        .route("/comments/:id/replies", get(get_comment_replies))
        .route("/comments/:id", get(get_comment))
        
        // ========== REFRACTS ==========
        .route("/refracts/:id", get(get_refract))
        
        // ========== USER PROFILES ==========
        .route("/users/:username/feed", get(get_user_feed))
        .route("/@:username", get(get_user_profile))

        .route("/auth/logout", post(logout))
        .route("/auth/logout-all", post(logout_all))
        .route("/auth/sessions", get(list_sessions))
        .route("/auth/sessions/:session_id", delete(delete_session))
        .route("/auth/delete-account", delete(delete_account))

        
        .route_layer(middleware::from_fn_with_state(state.clone(), optional_auth_middleware))
}

fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // ========== CSRF TOKEN ==========
        .route("/csrf-token", get(get_token))

        .route("/create/post", get(render_create_post_page_handler))
        .route("/create/question", get(render_create_question_page_handler))
        
        // ========== AUTH MANAGEMENT =========
        
        // ========== CONTENT CREATION ==========
        .route("/posts/create", post(create_post))
        .route("/posts/:id/edit", patch(update_post))
        .route("/posts/:id/delete", delete(delete_post))
        
        .route("/questions/create", post(create_question))
        .route("/questions/:id/edit", patch(update_question))
        .route("/questions/:id/delete", delete(delete_question))
        
        .route("/questions/:question_id/answer", post(create_answer))
        .route("/answers/:id/edit", patch(update_answer))
        .route("/answers/:id/delete", delete(delete_answer))
        
        .route("/comments/create", post(create_comment))
        .route("/comments/:id/edit", patch(update_comment))
        .route("/comments/:id/delete", delete(delete_comment))
        .route("/comments/:id/helpful", post(mark_helpful))
        .route("/comments/:id/unhelpful", delete(remove_helpful))
        
        .route("/refracts/create", post(create_refract))
        .route("/refracts/:id/edit", patch(update_refract))
        .route("/refracts/:id/delete", delete(delete_refract))
        
        .route("/echo", post(create_echo))
        
        // ========== PROFILE MANAGEMENT ==========
        .route("/profile/update", patch(update_profile))
        .route("/profile/username", patch(update_username))
        .route("/profile/links/add", post(add_user_link))
        .route("/profile/links/:id/edit", patch(update_user_link))
        .route("/profile/links/:id/delete", delete(delete_user_link))
        .route("/profile/update_links_bulk", patch(update_links_bulk))

        /*.route("/api/char-count", post(get_char_count))
        .route("/api/add-tag", post(add_tag))
        .route("/api/remove-tag", post(remove_tag))
        .route("/api/preview", post(preview_markdown))
        .route("/api/close-preview", get(close_preview)) */
        .route("/profile/edit", get(render_edit_profile_page_handler)) 
        
        .route_layer(middleware::from_fn_with_state(state.clone(), csrf_middleware))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

fn create_cors_layer() -> CorsLayer {
    // In production, replace Any with specific origins
    let origins: AllowOrigin = if cfg!(debug_assertions) {
        // Development: allow any origin
        Any.into()
    } else {
        // Production: specific origins only
        // Example: ["https://yourdomain.com".parse().unwrap()]
        Any.into() // TODO: Replace with actual production origins
    };

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET, 
            Method::POST, 
            Method::PATCH, 
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::COOKIE,
            header::HeaderName::from_static("x-csrf-token"),
        ])
        .max_age(std::time::Duration::from_secs(3600))
}

async fn health_check() -> &'static str {
    "OK"
}

pub fn create_trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
> {
    TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .level(Level::INFO)
                .include_headers(true)
        )
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(tower_http::LatencyUnit::Millis)
        )
        .on_failure(
            tower_http::trace::DefaultOnFailure::new()
                .level(Level::ERROR)
        )
}

async fn home_page(Query(params): Query<HashMap<String, String>>) -> Redirect {
    if params.contains_key("logout") {
        Redirect::to("/feed?logout=true")
    } else if params.contains_key("deleted") {
        Redirect::to("/feed?deleted=true")
    } else {
        Redirect::to("/feed")
    }
}


// ============================================================================
// TO ADD TIMEOUT + COMPRESSION LATER
// ============================================================================

/*
Add to Cargo.toml:
tower-http = { version = "0.6", features = ["compression-gzip", "timeout"] }

Then add these layers:
.layer(CompressionLayer::new())
.layer(TimeoutLayer::new(Duration::from_secs(30)))
*/

/* FOR PRODUCTION 

fn create_cors_layer() -> CorsLayer {
    // In production, replace Any with specific origins
    let origins: AllowOrigin = if cfg!(debug_assertions) {
        // Development: allow any origin
        Any.into()
    } else {
        // Production: specific origins only
        // Example: ["https://yourdomain.com".parse().unwrap()]
        Any.into() // TODO: Replace with actual production origins
    };

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET, 
            Method::POST, 
            Method::PATCH, 
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::COOKIE,
            header::HeaderName::from_static("x-csrf-token"),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}
 */
