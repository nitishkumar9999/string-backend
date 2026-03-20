mod renderer;

pub use renderer::{MarkdownRenderer, TocEntry};
use ammonia::Builder;
use std::collections::HashSet;
use regex::Regex;
use lazy_static::lazy_static;

pub fn parse_markdown(markdown_input: &str) -> String {
    
    let renderer = MarkdownRenderer::new();
    let (html, _toc) = renderer.render(markdown_input);

    let clean_html = sanitize_html(&html);
    convert_emojis(&clean_html)
}

pub fn parse_markdown_with_toc(markdown_input: &str) -> (String, Vec<TocEntry>) {
    let renderer = MarkdownRenderer::new();
    let (html, toc) = renderer.render(markdown_input);
    
    (sanitize_html(&html), toc)
}

pub fn convert_emojis(text: &str) -> String {
    lazy_static::lazy_static! {
        static ref EMOJI_REGEX: Regex = Regex::new(r":([a-zA-Z0-9_+-]+):").expect("Failed to compile emoji regex - check pattern");
    }

    EMOJI_REGEX.replace_all(text, |caps: &regex::Captures| {
        let shortcode = &caps[1];
        if let Some(emoji) = emojis::get_by_shortcode(shortcode) {
            emoji.to_string()
        } else {
            format!(":{shortcode}:")
        }
    }).to_string()
}

fn sanitize_html(html: &str) -> String {
    let schemes: HashSet<&str> = ["http", "https", "mailto"].iter().cloned().collect();

    Builder::default()

        .add_tags(&["details", "summary", "button"])
        .add_tag_attributes("div", &["class", "data-language"])
        .add_tag_attributes("pre", &["class"])
        .add_tag_attributes("span", &["class"])
        .add_tag_attributes("button", &["class", "data-action"])
        .add_tag_attributes("input", &["type", "disabled", "checked"])
        .add_tag_attributes("h1", &["id"])
        .add_tag_attributes("h2", &["id"])
        .add_tag_attributes("h3", &["id"])
        .add_tag_attributes("h4", &["id"])
        .add_tag_attributes("h5", &["id"])
        .add_tag_attributes("h6", &["id"])
        .add_tag_attributes("a", &["href", "title", "class"])
        .add_tag_attributes("img", &["src", "alt", "title"])
        .add_tag_attributes("sup", &["class"])
        .add_tag_attributes("table", &["class"])
        .add_tag_attributes("thead", &["class"])
        .add_tag_attributes("tbody", &["class"])
        .add_tag_attributes("tr", &["class"])
        .add_tag_attributes("th", &["class"])
        .add_tag_attributes("td", &["class"])
        // Allow inline styles for syntax highlighting
        .add_allowed_classes("code-block-wrapper", &["code-block-wrapper"])
        .add_allowed_classes("code-block-header", &["code-block-header"])
        .add_allowed_classes("code-language", &["code-language"])
        .add_allowed_classes("copy-button", &["copy-button"])
        .add_allowed_classes("code-block", &["code-block"])
        .add_allowed_classes("line-number", &["line-number"])
        .add_allowed_classes("footnote-reference", &["footnote-reference"])
        .add_allowed_classes("footnote-definition", &["footnote-definition"])
        .add_allowed_classes("footnote-definition-label", &["footnote-definition-label"])
        // Allow URL schemes
        .link_rel(Some("noopener noreferrer nofollow")) // Don't add rel="noopener noreferrer" automatically
        .url_schemes(schemes)
        .strip_comments(true)
        .clean(html)
        .to_string()
}



use pulldown_cmark::{Parser, Options, Event, Tag, TagEnd, CowStr, HeadingLevel};
use crate::markdown::renderer::escape_html;

/// Parse markdown with depth-based feature restrictions
/// - Depth 0: Full markdown (bold, italic, links, lists, code, quotes)
/// - Depth 1: Bold, italic, links only
/// - Depth 2: Links only
/// - Depth 3: Plain text only (HTML escaped)
pub fn parse_markdown_with_depth(markdown_input: &str, depth: i32) -> String {
    match depth {
        0 => parse_full_markdown(markdown_input),
        1 => parse_limited_markdown(markdown_input, true, true),  // bold, italic, links
        2 => parse_limited_markdown(markdown_input, false, false), // links only
        3 => escape_html(markdown_input), // plain text
        _ => escape_html(markdown_input), // fallback
    }
}

/// Full markdown parsing for depth 0
fn parse_full_markdown(markdown_input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    
    let parser = Parser::new_ext(markdown_input, options);
    let mut html = String::new();
    
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => html.push_str("<p>"),
                Tag::Strong => html.push_str("<strong>"),
                Tag::Emphasis => html.push_str("<em>"),
                Tag::Strikethrough => html.push_str("<del>"),
                Tag::Link { dest_url, title, .. } => {
                    let title_attr = if title.is_empty() {
                        String::new()
                    } else {
                        format!(r#" title="{title}""#)
                    };
                    html.push_str(&format!(r#"<a href="{dest_url}"{title_attr}>"#));
                }
                Tag::List(None) => html.push_str("<ul>"),
                Tag::List(Some(_)) => html.push_str("<ol>"),
                Tag::Item => html.push_str("<li>"),
                Tag::CodeBlock(_) => html.push_str("<pre><code>"),
                Tag::BlockQuote(_) => html.push_str("<blockquote>"),
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph => html.push_str("</p>"),
                TagEnd::Strong => html.push_str("</strong>"),
                TagEnd::Emphasis => html.push_str("</em>"),
                TagEnd::Strikethrough => html.push_str("</del>"),
                TagEnd::Link => html.push_str("</a>"),
                TagEnd::List(false) => html.push_str("</ul>"),
                TagEnd::List(true) => html.push_str("</ol>"),
                TagEnd::Item => html.push_str("</li>"),
                TagEnd::CodeBlock => html.push_str("</code></pre>"),
                TagEnd::BlockQuote(_) => html.push_str("</blockquote>"),
                _ => {}
            },
            Event::Text(text) => html.push_str(&escape_html(&text)),
            Event::Code(code) => html.push_str(&format!("<code>{}</code>", escape_html(&code))),
            Event::SoftBreak => html.push(' '),
            Event::HardBreak => html.push_str("<br>"),
            _ => {}
        }
    }
    
    sanitize_comment_html(&html)
}

/// Limited markdown parsing for depth 1 and 2
fn parse_limited_markdown(markdown_input: &str, allow_bold: bool, allow_italic: bool) -> String {
    let parser = Parser::new(markdown_input);
    let mut html = String::new();
    
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => html.push_str("<p>"),
                Tag::Strong if allow_bold => html.push_str("<strong>"),
                Tag::Emphasis if allow_italic => html.push_str("<em>"),
                Tag::Link { dest_url, title, .. } => {
                    let title_attr = if title.is_empty() {
                        String::new()
                    } else {
                        format!(r#" title="{title}""#)
                    };
                    html.push_str(&format!(r#"<a href="{dest_url}"{title_attr}>"#));
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph => html.push_str("</p>"),
                TagEnd::Strong if allow_bold => html.push_str("</strong>"),
                TagEnd::Emphasis if allow_italic => html.push_str("</em>"),
                TagEnd::Link => html.push_str("</a>"),
                _ => {}
            },
            Event::Text(text) => html.push_str(&escape_html(&text)),
            Event::Code(code) => html.push_str(&escape_html(&code)), // No code formatting
            Event::SoftBreak => html.push(' '),
            Event::HardBreak => html.push_str("<br>"),
            _ => {}
        }
    }
    
    sanitize_comment_html(&html)
}

/// Sanitize comment HTML (more restrictive than full posts)
fn sanitize_comment_html(html: &str) -> String {
    use ammonia::Builder;
    use std::collections::HashSet;

    let schemes: HashSet<&str> = ["http", "https"].iter().cloned().collect();

    Builder::default()
        .add_tag_attributes("a", &["href", "title"])
        .link_rel(None)
        .url_schemes(schemes)
        .strip_comments(true)
        .clean(html)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_0_full_markdown() {
        let input = "**Bold** *italic* [link](https://example.com) `code`\n- list";
        let output = parse_markdown_with_depth(input, 0);
        assert!(output.contains("<strong>"));
        assert!(output.contains("<em>"));
        assert!(output.contains("<a href="));
        assert!(output.contains("<code>"));
        assert!(output.contains("<ul>"));
    }

    #[test]
    fn test_depth_1_limited() {
        let input = "**Bold** *italic* [link](https://example.com) `code`\n- list";
        let output = parse_markdown_with_depth(input, 1);
        assert!(output.contains("<strong>"));
        assert!(output.contains("<em>"));
        assert!(output.contains("<a href="));
        assert!(!output.contains("<code>"));
        assert!(!output.contains("<ul>"));
    }

    #[test]
    fn test_depth_2_links_only() {
        let input = "**Bold** *italic* [link](https://example.com)";
        let output = parse_markdown_with_depth(input, 2);
        assert!(!output.contains("<strong>"));
        assert!(!output.contains("<em>"));
        assert!(output.contains("<a href="));
    }

    #[test]
    fn test_depth_3_plain_text() {
        let input = "**Bold** *italic* [link](https://example.com) <script>alert('xss')</script>";
        let output = parse_markdown_with_depth(input, 3);
        assert!(!output.contains("<strong>"));
        assert!(!output.contains("<a href="));
        assert!(!output.contains("<script>"));
        assert!(output.contains("&lt;script&gt;"));
    }
}