
// ============================================================================
// tests/integration/auth_tests.rs
// ============================================================================

#[cfg(test)]
mod auth_tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_get_me_authenticated() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        
        let request = authenticated_request("GET", "/api/auth/me", user.session_id);
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "user");
        assert_json_has_field!(json, "stats");
        assert_json_field!(json["user"]["username"], user.username);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_get_me_unauthenticated() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let request = unauthenticated_request("GET", "/api/auth/me");
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert!(json.is_null());
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_logout() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        
        let request = authenticated_request("POST", "/auth/logout", user.session_id);
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::SEE_OTHER);
        
        // Verify session is deleted
        let session_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id = $1)",
            user.session_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert!(!session_exists.unwrap());
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_logout_all() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        
        // Create additional session
        let session_id_2 = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO sessions (id, user_id, expires_at) VALUES ($1, $2, NOW() + INTERVAL '30 days')",
            session_id_2,
            user.id
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let request = authenticated_json_request(
            "POST",
            "/api/auth/logout-all",
            user.session_id,
            serde_json::json!({}),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["sessions_deleted"], 2u64);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_list_sessions() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        
        let request = authenticated_request("GET", "/api/auth/sessions", user.session_id);
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "sessions");
        assert_eq!(json["sessions"].as_array().unwrap().len(), 1);
        assert_json_field!(json["sessions"][0]["is_current"], true);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_delete_account() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        
        let request = authenticated_request("DELETE", "/api/auth/account", user.session_id);
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::SEE_OTHER);
        
        // Verify user is soft deleted
        let deleted_at = sqlx::query_scalar!(
            "SELECT deleted_at FROM users WHERE id = $1",
            user.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert!(deleted_at.is_some());
        
        cleanup_test_db(&db_name).await;
    }
}