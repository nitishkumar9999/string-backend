
// ============================================================================
// tests/integration/question_tests.rs
// ============================================================================

#[cfg(test)]
mod question_tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_create_question_success() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        
        let body = serde_json::json!({
            "title": "How do I write integration tests in Rust?",
            "content": "I'm trying to write integration tests for my Axum application.",
            "tags": ["rust", "testing", "axum"]
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/questions",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::CREATED);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "id");
        assert_json_has_field!(json, "slug");
        assert_json_field!(json["title"], "How do I write integration tests in Rust?");
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_create_question_missing_title() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        
        let body = serde_json::json!({
            "content": "Question content without title.",
            "tags": ["rust"]
        });
        
        let request = authenticated_csrf_request(
            "POST",
            "/api/questions",
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::BAD_REQUEST);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_get_question_with_answers() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let question = create_test_question(&pool, user.id, "Test Question?").await;
        
        // Create answers
        for i in 1..=5 {
            sqlx::query!(
                "INSERT INTO answers (question_id, user_id, content_raw, content_rendered_html, content_hash) VALUES ($1, $2, $3, $4, $5)",
                question.id,
                user.id,
                format!("Answer {}", i),
                format!("<p>Answer {}</p>", i),
                format!("{:x}", md5::compute(format!("Answer {}", i)))
            )
            .execute(&pool)
            .await
            .unwrap();
        }
        
        let request = unauthenticated_request(
            "GET",
            &format!("/api/questions/{}", question.slug),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["id"], question.id);
        assert_json_has_field!(json, "top_answers");
        assert_eq!(json["top_answers"].as_array().unwrap().len(), 3); // Top 3 only
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_update_question() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let question = create_test_question(&pool, user.id, "Original Question?").await;
        
        let body = serde_json::json!({
            "title": "Updated Question Title?",
            "content": "Updated content with more details about the question."
        });
        
        let request = authenticated_csrf_request(
            "PATCH",
            &format!("/api/questions/{}", question.id),
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["title"], "Updated Question Title?");
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_delete_question() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let question = create_test_question(&pool, user.id, "Question to Delete?").await;
        
        let request = authenticated_csrf_request(
            "DELETE",
            &format!("/api/questions/{}", question.id),
            user.session_id,
            &csrf_token,
            serde_json::json!({}),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        // Verify soft delete
        let deleted_at = sqlx::query_scalar!(
            "SELECT deleted_at FROM questions WHERE id = $1",
            question.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert!(deleted_at.is_some());
        
        cleanup_test_db(&db_name).await;
    }
}