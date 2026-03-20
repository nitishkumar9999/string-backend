
// ============================================================================
// tests/integration/echo_tests.rs
// ============================================================================

#[cfg(test)]
mod echo_tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_echo_post() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("Post")).await;
        
        let body = serde_json::json!({
            "post_id": post.id
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/echo",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["success"], true);
        assert_json_has_field!(json, "echo_count");
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_echo_idempotent() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("Post")).await;
        
        let body = serde_json::json!({
            "post_id": post.id
        });
        
        // Echo twice
        let request1 = authenticated_csrf_request(
            "POST",
            "/api/echo",
            user.session_id,
            &csrf_token,
            body.clone(),
        );
        app.clone().oneshot(request1).await.unwrap();
        
        let request2 = authenticated_csrf_request(
            "POST",
            "/api/echo",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request2).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        // Verify only one echo exists
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM echoes WHERE post_id = $1 AND user_id = $2",
            post.id,
            user.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(count.unwrap(), 1);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_cannot_echo_own_post() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("Own Post")).await;
        
        let body = serde_json::json!({
            "post_id": post.id
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/echo",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::FORBIDDEN);
        
        cleanup_test_db(&db_name).await;
    }
}