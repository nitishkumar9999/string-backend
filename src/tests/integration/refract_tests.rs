
// ============================================================================
// tests/integration/refract_tests.rs
// ============================================================================

#[cfg(test)]
mod refract_tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_create_refract() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user1 = create_test_user(&pool, "user1").await;
        let user2 = create_test_user(&pool, "user2").await;
        let csrf_token = create_test_csrf_token(&pool, user2.session_id).await;
        let post = create_test_post(&pool, user1.id, Some("Original Post")).await;
        
        let body = serde_json::json!({
            "original_post_id": post.id,
            "content": "This is an insightful post worth sharing with my thoughts."
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/refracts",
            user2.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::CREATED);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "id");
        assert_json_field!(json["original_post_id"], post.id);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_refract_upsert() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user1 = create_test_user(&pool, "user1").await;
        let user2 = create_test_user(&pool, "user2").await;
        let csrf_token = create_test_csrf_token(&pool, user2.session_id).await;
        let post = create_test_post(&pool, user1.id, Some("Post")).await;
        
        // First refract
        let body1 = serde_json::json!({
            "original_post_id": post.id,
            "content": "First refract comment."
        });
        
        let request1 = authenticated_csrf_request(
            "POST",
            "/api/refracts",
            user2.session_id,
            &csrf_token,
            body1,
        );
        app.clone().oneshot(request1).await.unwrap();
        
        // Second refract (should update)
        let body2 = serde_json::json!({
            "original_post_id": post.id,
            "content": "Updated refract comment with new thoughts."
        });
        
        let request2 = authenticated_csrf_request(
            "POST",
            "/api/refracts",
            user2.session_id,
            &csrf_token,
            body2,
        );
        let response = app.oneshot(request2).await.unwrap();
        
        assert_status!(response, StatusCode::CREATED);
        
        // Verify only one refract exists
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM refracts WHERE user_id = $1 AND original_post_id = $2",
            user2.id,
            post.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(count.unwrap(), 1);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_cannot_refract_own_post() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let post = create_test_post(&pool, user.id, Some("Own Post")).await;
        
        let body = serde_json::json!({
            "original_post_id": post.id,
            "content": "Trying to refract my own post."
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/refracts",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::FORBIDDEN);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_get_post_refracts() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user1 = create_test_user(&pool, "user1").await;
        let user2 = create_test_user(&pool, "user2").await;
        let post = create_test_post(&pool, user1.id, Some("Post")).await;
        
        // Create refracts
        for _ in 0..3 {
            sqlx::query!(
                "INSERT INTO refracts (user_id, original_post_id, content_raw, content_rendered_html) VALUES ($1, $2, $3, $4)",
                user2.id,
                post.id,
                "Refract content",
                "<p>Refract content</p>"
            )
            .execute(&pool)
            .await
            .unwrap();
        }
        
        let request = unauthenticated_request(
            "GET",
            &format!("/api/posts/{}/refracts", post.id),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "data");
        assert_eq!(json["data"].as_array().unwrap().len(), 3);
        
        cleanup_test_db(&db_name).await;
    }
}