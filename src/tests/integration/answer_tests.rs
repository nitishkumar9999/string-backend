// ============================================================================
// tests/integration/answer_tests.rs
// ============================================================================

#[cfg(test)]
mod answer_tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_create_answer_success() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let question = create_test_question(&pool, user.id, "Test Question?").await;
        
        let body = serde_json::json!({
            "content": "This is a comprehensive answer with enough detail to help."
        });
        
        let request = authenticated_csrf_request(
            "POST",
            &format!("/api/questions/{}/answers", question.id),
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::CREATED);
        
        let json = extract_json(response.into_body()).await;
        assert_json_has_field!(json, "id");
        assert_json_field!(json["question_id"], question.id);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_get_answer_by_slug() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let question = create_test_question(&pool, user.id, "Question").await;
        
        let answer = sqlx::query!(
            r#"
            INSERT INTO answers (question_id, user_id, content_raw, content_rendered_html, content_hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, slug
            "#,
            question.id,
            user.id,
            "Answer content",
            "<p>Answer content</p>",
            format!("{:x}", md5::compute("Answer content"))
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        let request = unauthenticated_request(
            "GET",
            &format!("/api/answers/{}", answer.slug.unwrap_or_default()),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        let json = extract_json(response.into_body()).await;
        assert_json_field!(json["id"], answer.id);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_update_answer() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let question = create_test_question(&pool, user.id, "Question").await;
        
        let answer = sqlx::query_scalar!(
            "INSERT INTO answers (question_id, user_id, content_raw, content_rendered_html, content_hash) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            question.id,
            user.id,
            "Original answer",
            "<p>Original answer</p>",
            format!("{:x}", md5::compute("Original answer"))
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        let body = serde_json::json!({
            "content": "Updated answer with more comprehensive details and explanations."
        });
        
        let request = authenticated_csrf_request(
            "PATCH",
            &format!("/api/answers/{}", answer),
            user.session_id,
            &csrf_token,
            body,
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        cleanup_test_db(&db_name).await;
    }
    
    #[tokio::test]
    async fn test_delete_answer() {
        let (pool, db_name) = create_unique_test_db().await;
        let app = create_test_app(pool.clone());
        
        let user = create_test_user(&pool, "testuser").await;
        let csrf_token = create_test_csrf_token(&pool, user.session_id).await;
        let question = create_test_question(&pool, user.id, "Question").await;
        
        let answer = sqlx::query_scalar!(
            "INSERT INTO answers (question_id, user_id, content_raw, content_rendered_html, content_hash) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            question.id,
            user.id,
            "Answer to delete",
            "<p>Answer to delete</p>",
            format!("{:x}", md5::compute("Answer to delete"))
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        let request = authenticated_csrf_request(
            "DELETE",
            &format!("/api/answers/{}", answer),
            user.session_id,
            &csrf_token,
            serde_json::json!({}),
        );
        let response = app.oneshot(request).await.unwrap();
        
        assert_status!(response, StatusCode::OK);
        
        // Verify soft delete
        let deleted_at = sqlx::query_scalar!(
            "SELECT deleted_at FROM answers WHERE id = $1",
            answer
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert!(deleted_at.is_some());
        
        cleanup_test_db(&db_name).await;
    }
}
