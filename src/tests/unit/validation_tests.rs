
#[cfg(test)]
mod validation_tests {
    use crate::validation::*;
    
    // ========== Content Validation ==========
    
    #[test]
    fn test_validate_post_content_success() {
        let content = "This is a valid post with enough content to pass validation.";
        let result = ContentValidator::validate_post(content, Some("Valid Title"));
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_post_content_too_short() {
        let content = "Short";
        let result = ContentValidator::validate_post(content, Some("Title"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_post_content_too_long() {
        let content = "a".repeat(31000);
        let result = ContentValidator::validate_post(&content, Some("Title"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_post_title_too_short() {
        let content = "Valid content that is long enough for validation.";
        let result = ContentValidator::validate_post(content, Some("Ab"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_post_title_too_long() {
        let content = "Valid content that is long enough for validation.";
        let title = "a".repeat(301);
        let result = ContentValidator::validate_post(content, Some(&title));
        assert!(result.is_err());
    }
    
    // ========== Question Validation ==========
    
    #[test]
    fn test_validate_question_success() {
        let content = "This is a valid question with enough content.";
        let title = "How do I write unit tests?";
        let result = ContentValidator::validate_question(content, title);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_question_title_required() {
        let content = "Valid content";
        let result = ContentValidator::validate_question(content, "");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_question_content_length() {
        let content = "Short";
        let title = "Valid Title Question?";
        let result = ContentValidator::validate_question(content, title);
        assert!(result.is_err());
        
        let long_content = "a".repeat(16000);
        let result = ContentValidator::validate_question(&long_content, title);
        assert!(result.is_err());
    }
    
    // ========== Answer Validation ==========
    
    #[test]
    fn test_validate_answer_success() {
        let content = "This is a valid answer with enough content to help.";
        let result = ContentValidator::validate_answer(content);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_answer_too_short() {
        let content = "Short";
        let result = ContentValidator::validate_answer(content);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_answer_too_long() {
        let content = "a".repeat(31000);
        let result = ContentValidator::validate_answer(&content);
        assert!(result.is_err());
    }
    
    // ========== Comment Validation ==========
    
    #[test]
    fn test_validate_comment_depth_0() {
        let content = "Valid comment content.";
        let result = ContentValidator::validate_comment(content, 0);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_comment_depth_1() {
        let content = "Valid reply content.";
        let result = ContentValidator::validate_comment(content, 1);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_comment_depth_2() {
        let content = "Valid nested reply.";
        let result = ContentValidator::validate_comment(content, 2);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_comment_depth_3() {
        let content = "Deep reply.";
        let result = ContentValidator::validate_comment(content, 3);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_comment_exceeds_depth_limit() {
        let content = "This should fail.";
        let result = ContentValidator::validate_comment(content, 4);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_comment_too_long_for_depth() {
        // Depth 0: max 1500 chars
        let content = "a".repeat(1501);
        let result = ContentValidator::validate_comment(&content, 0);
        assert!(result.is_err());
        
        // Depth 3: max 250 chars
        let content = "a".repeat(251);
        let result = ContentValidator::validate_comment(&content, 3);
        assert!(result.is_err());
    }
    
    // ========== Tag Validation ==========
    
    #[test]
    fn test_validate_tags_success() {
        let tags = vec!["rust".to_string(), "axum".to_string()];
        let result = ContentValidator::validate_tags(&tags);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_tags_empty() {
        let tags: Vec<String> = vec![];
        let result = ContentValidator::validate_tags(&tags);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_tags_too_many() {
        let tags = vec![
            "tag1".to_string(),
            "tag2".to_string(),
            "tag3".to_string(),
            "tag4".to_string(),
            "tag5".to_string(),
            "tag6".to_string(),
        ];
        let result = ContentValidator::validate_tags(&tags);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_tag_name_invalid_chars() {
        let tag = "rust$$$".to_string();
        let result = ContentValidator::validate_tag_name(&tag);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_tag_name_too_short() {
        let tag = "a".to_string();
        let result = ContentValidator::validate_tag_name(&tag);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_tag_name_too_long() {
        let tag = "a".repeat(31);
        let result = ContentValidator::validate_tag_name(&tag);
        assert!(result.is_err());
    }
    
    // ========== Username Validation ==========
    
    #[test]
    fn test_validate_username_success() {
        let usernames = vec!["john_doe", "alice-123", "user_name_123"];
        for username in usernames {
            let result = validate_username(username);
            assert!(result.is_ok(), "Username '{}' should be valid", username);
        }
    }
    
    #[test]
    fn test_validate_username_too_short() {
        let result = validate_username("ab");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_username_too_long() {
        let username = "a".repeat(31);
        let result = validate_username(&username);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_username_invalid_chars() {
        let invalid = vec!["john$doe", "alice@test", "user name", "test!", "user#123"];
        for username in invalid {
            let result = validate_username(username);
            assert!(result.is_err(), "Username '{}' should be invalid", username);
        }
    }
}