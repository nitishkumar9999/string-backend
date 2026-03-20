
// ============================================================================
// tests/integration/comment_tests.rs
// ============================================================================

#[cfg(test)]
mod comment_tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_create_comment_on_post() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("Post")).await;
        
        let body = serde_json::json!({
            "post_id": post.id,
            "content": "Great post! This is a helpful comment."
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/comments",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::CREATED);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "id");
        assert_json_field!(json["post_id"], post.id);
        assert_json_field!(json["depth_level"], 0);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_create_nested_comment() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("Post")).await;
        let parent = create_test_comment(&pool, user.id, Some(post.id), None, 0).await;
        
        let body = serde_json::json!({
            "post_id": post.id,
            "parent_comment_id": parent.id,
            "content": "Reply to comment."
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/comments",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::CREATED);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["parent_comment_id"], parent.id);
        assert_json_field!(json["depth_level"], 1);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_mark_comment_helpful() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("Post")).await;
        let comment = create_test_comment(&pool, user.id, Some(post.id), None, 0).await;
        
        let request = authenticated_csrf_request(
            "POST",
            &format!("/api/comments/{}/helpful", comment.id),
            user.session_id,
            &csrf_token,
            serde_json::json!({}),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        // Verify helpful mark exists
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM comment_helpful WHERE comment_id = $1 AND user_id = $2)",
            comment.id,
            user.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert!(exists.unwrap());
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_unmark_comment_helpful() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("Post")).await;
        let comment = create_test_comment(&pool, user.id, Some(post.id), None, 0).await;
        
        // First mark as helpful
        sqlx::query!(
            "INSERT INTO comment_helpful (comment_id, user_id) VALUES ($1, $2)",
            comment.id,
            user.id
        )
        .execute(&pool)
        .await
        .unwrap();
        
        // Then unmark
        let request = authenticated_csrf_request(
            "DELETE",
            &format!("/api/comments/{}/helpful", comment.id),
            user.session_id,
            &csrf_token,
            serde_json::json!({}),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        // Verify helpful mark removed
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM comment_helpful WHERE comment_id = $1 AND user_id = $2)",
            comment.id,
            user.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert!(!exists.unwrap());
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_get_comment_with_replies() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let post = create_test_post(&pool, user.id, Some("Post")).await;
        let parent = create_test_comment(&pool, user.id, Some(post.id), None, 0).await;
        
        // Create replies
        for _ in 0..3 {
            create_test_comment(&pool, user.id, Some(post.id), Some(parent.id), 1).await;
        }
        
        let request = unauthenticated_request(
            "GET",
            &format!("/api/comments/{}", parent.id),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["id"], parent.id);
        assert_json_has_field!(json, "replies");
        
        cleanup_test_db(&db_name).await;
    }
}