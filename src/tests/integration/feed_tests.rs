
// ============================================================================
// tests/integration/feed_tests.rs
// ============================================================================

#[cfg(test)]
mod feed_tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_unified_feed() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        
        // Create mixed content
        create_test_post(&pool, user.id, Some("Post 1")).await;
        create_test_question(&pool, user.id, "Question 1?").await;
        create_test_post(&pool, user.id, Some("Post 2")).await;
        
        let request = unauthenticated_request("GET", "/api/feed?limit=10");
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "data");
        assert_json_has_field!(json, "has_more");
        assert_json_has_field!(json, "next_cursor");
        
        let data = json["data"].as_array().unwrap();
        assert_eq!(data.len(), 3);
        
        // Verify mixed types
        let types: Vec<_> = data.iter().map(|item| item["type"].as_str().unwrap()).collect();
        assert!(types.contains(&"post"));
        assert!(types.contains(&"question"));
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_feed_pagination() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        
        // Create 10 posts
        for i in 1..=10 {
            create_test_post(&pool, user.id, Some(&format!("Post {}", i))).await;
        }
        
        // Get first page (limit 5)
        let request = unauthenticated_request("GET", "/api/feed?limit=5");
        let response = app.clone().oneshot(request).await.unwrap();
        
        let json = extract_json(response.into_body()).await;
        assert_eq!(json["data"].as_array().unwrap().len(), 5);
        assert_json_field!(json["has_more"], true);
        
        let cursor = json["next_cursor"].as_i64().unwrap();
        
        // Get second page
        let request = unauthenticated_request(
            "GET",
            &format!("/api/feed?limit=5&cursor={}", cursor),
        );
        let response = app.oneshot(request).await.unwrap();
        
        let json = extract_json(response.into_body()).await;
        assert_eq!(json["data"].as_array().unwrap().len(), 5);
        
        cleanup_test_db(&db_name).await;
    }
}