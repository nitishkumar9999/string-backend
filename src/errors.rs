use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;
use time::OffsetDateTime;

pub type Result<T> = std::result::Result<T, AppError>;

/// Main application error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(#[from] RateLimitError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account not found")]
    AccountNotFound,

    // CSRF Protection
    #[error("CSRF token missing")]
    CsrfTokenMissing,

    #[error("CSRF token invalid or expired")]
    CsrfTokenInvalid,

    #[error("Too many active CSRF tokens")]
    TooManyActiveTokens,

    // Permissions
    #[error("Forbidden: you don't have permission to perform this action")]
    Forbidden,

    #[error("Cannot perform action on own content")]
    CannotActOnOwnContent, // For echo/refract restrictions

    // OAuth errors
    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("OAuth state mismatch")]
    OAuthStateMismatch,

    //Search Errors
    #[error("Search Failed")]
    Search,
    
    // General
    #[error("Internal server error")]
    InternalError, // Simpler version for cases where you don't need message

    #[error("Invalid header value")]
    InvalidHeader(#[from] axum::http::header::InvalidHeaderValue),

    #[error("Not found")]
    NotFound, 

}

/// Validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    // Content validation
    #[error("Content too short: minimum {min} characters, got {actual}")]
    ContentTooShort { min: usize, actual: usize },

    #[error("Content too long: maximum {max} characters, got {actual}")]
    ContentTooLong { max: usize, actual: usize },

    #[error("Title too short: minimum {min} characters, got {actual}")]
    TitleTooShort { min: usize, actual: usize },

    #[error("Title too long: maximum {max} characters, got {actual}")]
    TitleTooLong { max: usize, actual: usize },

    // Username validation
    #[error("Username too short: minimum {min} characters")]
    UsernameTooShort { min: usize },

    #[error("Username too long: maximum {max} characters")]
    UsernameTooLong { max: usize },

    #[error("Invalid username format: must start and end with alphanumeric, can contain letters, numbers, underscore, and dots")]
    InvalidUsernameFormat,

    #[error("Username contains consecutive dots")]
    UsernameConsecutiveDots,

    #[error("Username is reserved: {0}")]
    UsernameReserved(String),

    // Tag validation
    #[error("Minimum {min} tag required, got {actual}")]
    NotEnoughTags { min: usize, actual: usize },

    #[error("Maximum {max} tags allowed, got {actual}")]
    TooManyTags { max: usize, actual: usize },

    #[error("Invalid tag format: {0}")]
    InvalidTagFormat(String),

    // Media validation
    #[error("File size too large: maximum {max_mb}MB, got {actual_mb}MB")]
    FileSizeTooLarge { max_mb: usize, actual_mb: usize },

    #[error("Invalid image format")]
    InvalidImageFormat,

    #[error("Image dimensions too large: maximum {max}x{max} pixels, got {width}x{height}")]
    ImageDimensionsTooLarge {
        max: u32,
        width: u32,
        height: u32,
    },

    #[error("Too many images: maximum {max} allowed, got {actual}")]
    TooManyImages { max: usize, actual: usize },

    #[error("Too many videos: maximum {max} allowed, got {actual}")]
    TooManyVideos { max: usize, actual: usize },

    #[error("Invalid video format")]
    InvalidVideoFormat,

    #[error("Video duration too long: maximum {max_minutes} minutes")]
    VideoDurationTooLong { max_minutes: usize },

    #[error("Video resolution too high: maximum 1080p")]
    VideoResolutionTooHigh,

    // URL validation
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    #[error("Blocked domain: {0}")]
    BlockedDomain(String),

    #[error("Too many links: maximum {max} links allowed in short content")]
    TooManyLinks { max: usize },

    // Spam detection
    #[error("Excessive uppercase: content appears to be shouting")]
    ExcessiveCaps,

    #[error("Excessive emoji usage")]
    ExcessiveEmoji,

    #[error("Excessive character repetition detected")]
    ExcessiveRepetition,

    // Duplicate content
    #[error("Duplicate content: you posted this recently")]
    DuplicateContent,

    // Bio validation
    #[error("Bio too long: maximum {max} characters")]
    BioTooLong { max: usize },

    // General
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid cursor: {0}")]
    InvalidCursor(String),
    
    #[error("Unknown File Type")]
    UnknownFileType,

    #[error("Unsupported Media Type: {0}")]
    UnSupportedMediaType(String),
    
    #[error("Search query is too long. Maximum {max} characters allowed.")]
    SearchQueryTooLong {max: usize },

    #[error("Too many tags specified. Maximum {max} tags allowed.")]
    TagLimitExceeding { max: usize },

}

/// Rate limit errors
#[derive(Debug, Clone)]
pub struct RateLimitError {
    pub action: String,
    pub limit: String,
    pub current_count: i32,
    pub retry_after_seconds: i64,
    pub window_resets_at: OffsetDateTime,
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rate limit exceeded for {}: {} (current: {})",
            self.action, self.limit, self.current_count
        )
    }
}

impl std::error::Error for RateLimitError {}

/// Error response for API
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Rate limit error response
#[derive(Serialize)]
pub struct RateLimitResponse {
    pub error: String,
    pub message: String,
    pub limit: String,
    pub current_count: i32,
    pub retry_after_seconds: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub window_resets_at: OffsetDateTime,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            AppError::Validation(e) => {
                let response = ErrorResponse {
                    error: "validation_error".to_string(),
                    message: e.to_string(),
                    details: None,
                };
                (StatusCode::BAD_REQUEST, Json(response))
            }

            AppError::RateLimit(e) => {
                let response = RateLimitResponse {
                    error: "rate_limit_exceeded".to_string(),
                    message: format!("You've exceeded the limit for {}", e.action),
                    limit: e.limit,
                    current_count: e.current_count,
                    retry_after_seconds: e.retry_after_seconds,
                    window_resets_at: e.window_resets_at,
                };
                return (StatusCode::TOO_MANY_REQUESTS, Json(response)).into_response();
            }

            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                let response = ErrorResponse {
                    error: "database_error".to_string(),
                    message: "A database error occurred".to_string(),
                    details: None,
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            }

            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                let response = ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "An internal error occurred".to_string(),
                    details: None,
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            }

            AppError::AuthenticationRequired => {
                let response = ErrorResponse {
                    error: "authentication_required".to_string(),
                    message: "You must be logged in to perform this action".to_string(),
                    details: None,
                };
                (StatusCode::UNAUTHORIZED, Json(response))
            }

            AppError::SessionNotFound => {
                let response = ErrorResponse {
                    error: "session_not_found".to_string(),
                    message: "Session not found or invalid".to_string(),
                    details: None,
                };
                (StatusCode::UNAUTHORIZED, Json(response))
            }

            AppError::SessionExpired => {
                let response = ErrorResponse {
                    error: "session_expired".to_string(),
                    message: "Your session has expired. Please log in again".to_string(),
                    details: None,
                };
                (StatusCode::UNAUTHORIZED, Json(response))
            }

            AppError::InvalidCredentials => {
                let response = ErrorResponse {
                    error: "invalid_credentials".to_string(),
                    message: "Invalid username or password".to_string(),
                    details: None,
                };
                (StatusCode::UNAUTHORIZED, Json(response))
            }

            AppError::AccountNotFound => {
                let response = ErrorResponse {
                    error: "account_not_found".to_string(),
                    message: "Account not found".to_string(),
                    details: None,
                };
                (StatusCode::NOT_FOUND, Json(response))
            }

            // CSRF errors (403)
            AppError::CsrfTokenMissing => {
                let response = ErrorResponse {
                    error: "csrf_token_missing".to_string(),
                    message: "CSRF token is required for this request".to_string(),
                    details: None,
                };
                (StatusCode::FORBIDDEN, Json(response))
            }

            AppError::CsrfTokenInvalid => {
                let response = ErrorResponse {
                    error: "csrf_token_invalid".to_string(),
                    message: "CSRF token is invalid or has expired".to_string(),
                    details: None,
                };
                (StatusCode::FORBIDDEN, Json(response))
            }

            AppError::TooManyActiveTokens => {
                let response = ErrorResponse {
                    error: "too_many_active_tokens".to_string(),
                    message: "Too many active CSRF tokens. Please try again later".to_string(),
                    details: None,
                };
                (StatusCode::TOO_MANY_REQUESTS, Json(response))
            }

            // Permission errors (403)
            AppError::Forbidden => {
                let response = ErrorResponse {
                    error: "forbidden".to_string(),
                    message: "You don't have permission to perform this action".to_string(),
                    details: None,
                };
                (StatusCode::FORBIDDEN, Json(response))
            }

            AppError::CannotActOnOwnContent => {
                let response = ErrorResponse {
                    error: "cannot_act_on_own_content".to_string(),
                    message: "You cannot echo or refract your own content".to_string(),
                    details: None,
                };
                (StatusCode::FORBIDDEN, Json(response))
            }

            // OAuth errors (400/500)
            AppError::OAuth(msg) => {
                tracing::error!("OAuth error: {}", msg);
                let response = ErrorResponse {
                    error: "oauth_error".to_string(),
                    message: "OAuth authentication failed".to_string(),
                    details: Some(serde_json::json!({ "detail": msg })),
                };
                (StatusCode::UNAUTHORIZED, Json(response))
            }

            AppError::OAuthStateMismatch => {
                let response = ErrorResponse {
                    error: "oauth_state_mismatch".to_string(),
                    message: "OAuth state validation failed. Please try again".to_string(),
                    details: None,
                };
                (StatusCode::UNAUTHORIZED, Json(response))
            }

            AppError::InternalError => {
                tracing::error!("Internal error occurred");
                let response = ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "An internal error occurred".to_string(),
                    details: None,
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            }

            AppError::Search => {
                let response = ErrorResponse {
                    error: "Search_error".to_string(),
                    message: "Search failed. Please try again.".to_string(),
                    details: None,
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            },

            AppError::InvalidHeader(InvalidHeaderValue) => {
                let response = ErrorResponse {
                    error: "Invalid_header".to_string(),
                    message: "Invalid or missing required header. Please check your request.".to_string(),
                    details: None,
                };
                (StatusCode::BAD_REQUEST, Json(response))
            },

            AppError::NotFound => {
                let response = ErrorResponse {
                    error: "not_found".to_string(),
                    message: "The requested resource was not found".to_string(),
                    details: None,
                };
                (StatusCode::NOT_FOUND, Json(response))
            }
        };
        (status, error_response).into_response()
    }
    
}