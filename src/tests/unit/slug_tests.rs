
#[cfg(test)]
mod slug_tests {
    use crate::utils::slug::*;
    
    #[test]
    fn test_generate_slug_basic() {
        let slug = generate_slug("Hello World", 123);
        assert!(slug.contains("hello-world"));
        assert!(slug.contains("123"));
    }
    
    #[test]
    fn test_generate_slug_special_chars() {
        let slug = generate_slug("Rust & C++: Best Practices!", 456);
        assert!(!slug.contains("&"));
        assert!(!slug.contains(":"));
        assert!(!slug.contains("!"));
        assert!(slug.contains("456"));
    }
    
    #[test]
    fn test_generate_slug_unicode() {
        let slug = generate_slug("Hello 世界", 789);
        assert!(slug.contains("hello"));
        assert!(slug.contains("789"));
    }
    
    #[test]
    fn test_generate_slug_with_tag() {
        let slug = generate_slug_with_tag("My Post Title", "rust", 100);
        assert!(slug.contains("my-post-title"));
        assert!(slug.contains("rust"));
        assert!(slug.contains("100"));
    }
    
    #[test]
    fn test_generate_slug_truncation() {
        let long_title = "a".repeat(200);
        let slug = generate_slug(&long_title, 1);
        assert!(slug.len() < 200);
    }
    
    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Rust & Go"), "rust-go");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(slugify("CamelCase"), "camelcase");
    }
}