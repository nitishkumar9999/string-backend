use axum::body::Body; // Alias for clarity
use axum::{
    extract::State,
    http::{header, HeaderValue, Request},
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::state::AppState;

// Generic <B> allows this to work with standard and streaming bodies
pub async fn security_headers_middleware(
    State(_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let mut response = next.run(request).await;
    inject_security_headers(response.headers_mut());
    response
}

fn inject_security_headers(headers: &mut axum::http::HeaderMap) {
    // 1. Prevent MIME sniffing
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    // 2. Clickjacking Protection
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // 3. Referrer privacy
    headers.insert(header::REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin"));
    
    // 4. HSTS (Production only)
    #[cfg(not(debug_assertions))]
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains")
    );
    
    // 5. Modern CSP 
    // Added 'wss:' for WebSockets and 'https:' for external image loading
    let csp = "default-src 'self'; \
               script-src 'self' 'wasm-unsafe-eval'; \
               style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
               font-src 'self' https://fonts.gstatic.com; \
               img-src 'self' data: https:; \
               connect-src 'self' wss:;";
               
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(csp));
}