
#[cfg(test)]
mod pagination_tests {
    use crate::utils::pagination::*;
    
    #[test]
    fn test_cursor_pagination_defaults() {
        let pagination = CursorPagination::new(None);
        assert_eq!(pagination.limit, 25);
        assert!(pagination.cursor.is_none());
    }
    
    #[test]
    fn test_cursor_pagination_validate_limit() {
        let mut pagination = CursorPagination {
            cursor: None,
            limit: 200,
        };
        pagination.validate_limit();
        assert_eq!(pagination.limit, 100); // Should clamp to max
        
        let mut pagination = CursorPagination {
            cursor: None,
            limit: -5,
        };
        pagination.validate_limit();
        assert_eq!(pagination.limit, 25); // Should reset to default
    }
    
    #[test]
    fn test_paginated_response_no_more() {
        let data = vec![1, 2, 3];
        let response = PaginatedResponse::new(data, 5, |item| *item);
        
        assert_eq!(response.data.len(), 3);
        assert!(!response.has_more);
        assert!(response.next_cursor.is_none());
    }
    
    #[test]
    fn test_paginated_response_has_more() {
        let data = vec![1, 2, 3, 4, 5, 6]; // 6 items
        let response = PaginatedResponse::new(data, 5, |item| *item); // Limit 5
        
        assert_eq!(response.data.len(), 5); // Should have 5 items (6th removed)
        assert!(response.has_more);
        assert_eq!(response.next_cursor, Some(5)); // Last item's ID
    }
    
    #[test]
    fn test_cursor_encode_decode() {
        let original = Cursor::new(
            0.85,
            100,
            time::OffsetDateTime::now_utc(),
            42,
        );
        
        let encoded = original.encode();
        let decoded = Cursor::decode(&encoded).unwrap();
        
        assert_eq!(original.rank, decoded.rank);
        assert_eq!(original.engagement_count, decoded.engagement_count);
        assert_eq!(original.id, decoded.id);
    }
}