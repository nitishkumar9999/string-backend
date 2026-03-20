
// ============================================================================
// tests/integration/post_tests.rs
// ============================================================================

#[cfg(test)]
mod post_tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_create_post_success() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        
        let body = serde_json::json!({
            "content": "This is a test post with enough content to pass validation.",
            "title": "Test Post Title",
            "tags": ["rust", "testing"]
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/posts",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::CREATED);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "id");
        assert_json_has_field!(json, "slug");
        assert_json_field!(json["title"], "Test Post Title");
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_create_post_without_auth() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let body = serde_json::json!({
            "content": "Test content",
            "tags": ["rust"]
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/posts")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_status!(response, StatusCode::UNAUTHORIZED);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_create_post_without_csrf() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        
        let body = serde_json::json!({
            "content": "Test content with enough text to pass validation.",
            "tags": ["rust"]
        });
        
        let request = authenticated_json_request(
            "POST",
            "/api/posts",
            user.session_id,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::FORBIDDEN);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_create_post_validation_failure() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        
        let body = serde_json::json!({
            "content": "Short",
            "tags": ["rust"]
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/posts",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::BAD_REQUEST);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_get_post_by_slug() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let post = create_test_post(&pool, user.id, Some("Test Post")).await;
        
        let request = unauthenticated_request(
            "GET",
            &format!("/api/posts/{}", post.slug),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["id"], post.id);
        assert_json_field!(json["username"], user.username);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_update_post() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("Original Title")).await;
        
        let body = serde_json::json!({
            "content": "Updated content with enough text to pass validation.",
            "title": "Updated Title"
        });
        
        let request = authenticated_csrf_request(
            "PATCH",
            &format!("/api/posts/{}", post.id),
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["title"], "Updated Title");
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_update_post_not_owner() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user1 = create_test_user(&pool, "user1").await;
        let user2 = create_test_user(&pool, "user2").await;
        let csrf_token = create_test_csrf_token(&pool, user2.session_id).await;
        let post = create_test_post(&pool, user1.id, Some("User1's Post")).await;
        
        let body = serde_json::json!({
            "content": "Trying to update someone else's post."
        });
        
        let request = authenticated_csrf_request(
            "PATCH",
            &format!("/api/posts/{}", post.id),
            user2.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::FORBIDDEN);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_delete_post() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("To Delete")).await;
        
        let request = authenticated_csrf_request(
            "DELETE",
            &format!("/api/posts/{}", post.id),
            user.session_id,
            &csrf_token,
            serde_json::json!({}),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        // Verify soft delete
        let deleted_at = sqlx::query_scalar!(
            "SELECT deleted_at FROM posts WHERE id = $1",
            post.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert!(deleted_at.is_some());
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_get_posts_feed() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        create_test_post(&pool, user.id, Some("Post 1")).await;
        create_test_post(&pool, user.id, Some("Post 2")).await;
        create_test_post(&pool, user.id, Some("Post 3")).await;
        
        let request = unauthenticated_request("GET", "/api/posts/feed?limit=10");
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "data");
        assert_json_has_field!(json, "has_more");
        assert_eq!(json["data"].as_array().unwrap().len(), 3);
        
        cleanup_test_db(&db_name).await;
    }
}
