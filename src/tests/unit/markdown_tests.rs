
#[cfg(test)]
mod markdown_tests {
    use crate::markdown::*;
    
    #[test]
    fn test_parse_markdown_basic() {
        let input = "# Hello\n\nThis is **bold** text.";
        let output = parse_markdown(input);
        assert!(output.contains("<h1>"));
        assert!(output.contains("<strong>"));
    }
    
    #[test]
    fn test_parse_markdown_code_block() {
        let input = "```rust\nfn main() {}\n```";
        let output = parse_markdown(input);
        assert!(output.contains("<code"));
        assert!(output.contains("rust"));
    }
    
    #[test]
    fn test_parse_markdown_links() {
        let input = "[Rust](https://rust-lang.org)";
        let output = parse_markdown(input);
        assert!(output.contains("<a"));
        assert!(output.contains("href=\"https://rust-lang.org\""));
        assert!(output.contains("rel=\"noopener noreferrer\""));
    }
    
    #[test]
    fn test_depth_aware_markdown_depth_0() {
        let input = "# Title\n\n**Bold** and *italic* and [link](http://example.com)";
        let output = render_depth_aware_markdown(input, 0);
        assert!(output.contains("<h1>"));
        assert!(output.contains("<strong>"));
        assert!(output.contains("<em>"));
        assert!(output.contains("<a"));
    }
    
    #[test]
    fn test_depth_aware_markdown_depth_1() {
        let input = "# Title\n\n**Bold** and *italic* and [link](http://example.com)";
        let output = render_depth_aware_markdown(input, 1);
        assert!(!output.contains("<h1>"));  // No headers
        assert!(output.contains("<strong>"));  // Bold allowed
        assert!(output.contains("<em>"));      // Italic allowed
        assert!(output.contains("<a"));        // Links allowed
    }
    
    #[test]
    fn test_depth_aware_markdown_depth_2() {
        let input = "**Bold** and *italic* and [link](http://example.com)";
        let output = render_depth_aware_markdown(input, 2);
        assert!(!output.contains("<strong>"));  // No bold
        assert!(!output.contains("<em>"));      // No italic
        assert!(output.contains("<a"));         // Links allowed only
    }
    
    #[test]
    fn test_depth_aware_markdown_depth_3() {
        let input = "**Bold** and [link](http://example.com) <script>alert('xss')</script>";
        let output = render_depth_aware_markdown(input, 3);
        assert!(!output.contains("<strong>"));  // No formatting
        assert!(!output.contains("<a"));        // No links
        assert!(!output.contains("<script"));   // XSS sanitized
        assert!(output.contains("Bold"));       // Plain text only
    }
    
    #[test]
    fn test_sanitize_markdown_removes_script() {
        let input = "<script>alert('xss')</script>Safe content";
        let output = sanitize_markdown(input);
        assert!(!output.contains("<script"));
        assert!(output.contains("Safe content"));
    }
    
    #[test]
    fn test_sanitize_markdown_allows_safe_tags() {
        let input = "<p>Paragraph</p><strong>Bold</strong><a href='test'>Link</a>";
        let output = sanitize_markdown(input);
        assert!(output.contains("<p>"));
        assert!(output.contains("<strong>"));
        assert!(output.contains("<a"));
    }
}