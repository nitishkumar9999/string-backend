
// ============================================================================
// tests/integration/user_tests.rs
// ============================================================================

#[cfg(test)]
mod user_tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_get_user_profile() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        create_test_post(&pool, user.id, Some("Post")).await;
        create_test_question(&pool, user.id, "Question?").await;
        
        let request = unauthenticated_request(
            "GET",
            &format!("/api/users/{}", user.username),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["username"], user.username);
        assert_json_has_field!(json, "stats");
        assert_json_field!(json["stats"]["post_count"], 1i64);
        assert_json_field!(json["stats"]["question_count"], 1i64);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_get_user_feed() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        create_test_post(&pool, user.id, Some("Post 1")).await;
        create_test_post(&pool, user.id, Some("Post 2")).await;
        create_test_question(&pool, user.id, "Question?").await;
        
        let request = unauthenticated_request(
            "GET",
            &format!("/api/users/{}/feed", user.username),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "data");
        assert_eq!(json["data"].as_array().unwrap().len(), 3);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_update_profile() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        
        let body = serde_json::json!({
            "name": "Updated Name",
            "bio": "This is my updated bio."
        });
        
        let request = authenticated_csrf_request(
            "PATCH",
            "/api/users/profile",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        // Verify update
        let updated_name = sqlx::query_scalar!(
            "SELECT name FROM users WHERE id = $1",
            user.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(updated_name, "Updated Name");
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_update_username() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "oldusername").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        
        let body = serde_json::json!({
            "username": "newusername"
        });
        
        let request = authenticated_csrf_request(
            "PATCH",
            "/api/users/username",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        // Verify update
        let updated_username = sqlx::query_scalar!(
            "SELECT username FROM users WHERE id = $1",
            user.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(updated_username, "newusername");
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_add_user_link() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        
        let body = serde_json::json!({
            "platform": "github",
            "url": "https://github.com/testuser",
            "display_text": "My GitHub"
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/users/links",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::CREATED);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "id");
        assert_json_field!(json["platform"], "github");
        
        cleanup_test_db(&db_name).await;
    }
}