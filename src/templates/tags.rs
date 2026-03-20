// src/templates/tags.rs
use maud::{html, Markup, PreEscaped, DOCTYPE};
use crate::handlers::tags::{TagInfo, TagFeedItem, PostTagItem, QuestionTagItem};
use crate::templates::feed::{render_header, render_avatar, format_time};

fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

fn truncate_html(html: &str, max_chars: usize) -> String {
    let text_len = strip_tags(html).len();
    if text_len <= max_chars {
        return html.to_string();
    }
    let mut char_count = 0;
    let mut result = String::new();
    let mut in_tag = false;
    let mut pos = 0;
    while pos < html.len() {
        let remaining = &html[pos..];
        if remaining.starts_with(r#"<div class="code-block-wrapper"#) {
            if let Some(end_offset) = remaining.find("</pre>\n</div>") {
                let end = end_offset + "</pre>\n</div>\n".len();
                let block = &remaining[..end.min(remaining.len())];
                if char_count < max_chars {
                    result.push_str(block);
                }
                pos += block.len();
                continue;
            }
        }
        let ch = html[pos..].chars().next().unwrap();
        let ch_len = ch.len_utf8();
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            char_count += 1;
            if char_count > max_chars {
                result.push_str("...");
                return result;
            }
        }
        result.push(ch);
        pos += ch_len;
    }
    result
}

fn should_show_read_more(html: &str, truncate_at: usize) -> bool {
    strip_tags(html).len() > truncate_at
}

const ICON_COMMENT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M20 2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h6.72l4.76 2.86c.16.09.34.14.51.14s.34-.04.49-.13c.31-.18.51-.51.51-.87v-2h3c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2Zm0 14h-4c-.55 0-1 .45-1 1v1.23l-3.49-2.09A1.03 1.03 0 0 0 11 16H4V4h16z"></path></svg>"#;
const ICON_REFRACT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M17 5H6c-1.1 0-2 .9-2 2v5h2V7h11v3l5-4-5-4zm1 12H7v-3l-5 4 5 4v-3h11c1.1 0 2-.9 2-2v-5h-2z"></path></svg>"#;
const ICON_ECHO: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M14 12c0-1.1-.9-2-2-2s-2 .9-2 2a2 2 0 1 0 4 0m-6 0c0-1.07.42-2.07 1.17-2.82L7.76 7.76A5.97 5.97 0 0 0 6 12c0 1.6.62 3.11 1.76 4.25l1.41-1.42A3.96 3.96 0 0 1 8 12m8.24-4.24-1.41 1.41C15.59 9.93 16 10.93 16 12s-.42 2.07-1.17 2.83l1.41 1.41C17.37 15.11 18 13.6 18 12s-.62-3.11-1.76-4.24"></path><path d="M6.34 17.66C4.83 16.15 3.99 14.14 3.99 12s.83-4.14 2.34-5.65L4.92 4.93C3.03 6.82 1.99 9.33 1.99 12s1.04 5.18 2.93 7.07l1.41-1.41ZM19.07 4.93l-1.41 1.41C19.17 7.85 20 9.86 20 12s-.83 4.15-2.34 5.66l1.41 1.41C20.96 17.18 22 14.67 22 12s-1.04-5.18-2.93-7.07"></path></svg>"#;
const ICON_LINK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M9.88 18.36a3 3 0 0 1-4.24 0 3 3 0 0 1 0-4.24l2.83-2.83-1.41-1.41-2.83 2.83a5.003 5.003 0 0 0 0 7.07c.98.97 2.25 1.46 3.54 1.46s2.56-.49 3.54-1.46l2.83-2.83-1.41-1.41-2.83 2.83Zm2.83-14.14L9.88 7.05l1.41 1.41 2.83-2.83a3 3 0 0 1 4.24 0 3 3 0 0 1 0 4.24l-2.83 2.83 1.41 1.41 2.83-2.83a5.003 5.003 0 0 0 0-7.07 5.003 5.003 0 0 0-7.07 0Z"></path><path d="m16.95 8.46-.71-.7-.7-.71-4.25 4.24-4.24 4.25.71.7.7.71 4.25-4.24z"></path></svg>"#;
const ICON_ANSWER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M12 2C6.49 2 2 6.49 2 12s4.49 10 10 10h9c.37 0 .71-.21.89-.54.17-.33.15-.73-.06-1.03l-1.75-2.53a10 10 0 0 0 1.93-5.9c0-5.51-4.49-10-10-10Zm6 16.43L19.09 20H12c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8c0 1.91-.69 3.75-1.93 5.21-.3.34-.32.85-.06 1.22Z"></path></svg>"#;
const ICON_UP_ARROW: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="m6.293 11.293 1.414 1.414L12 8.414l4.293 4.293 1.414-1.414L12 5.586z"></path><path d="m6.293 16.293 1.414 1.414L12 13.414l4.293 4.293 1.414-1.414L12 10.586z"></path></svg>"#;
const ICON_TAG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M21.41 11.58l-9-9C12.05 2.22 11.55 2 11 2H4c-1.1 0-2 .9-2 2v7c0 .55.22 1.05.59 1.42l9 9c.36.36.86.58 1.41.58s1.05-.22 1.41-.59l7-7c.37-.36.59-.86.59-1.41s-.23-1.06-.59-1.42M5.5 7C4.67 7 4 6.33 4 5.5S4.67 4 5.5 4 7 4.67 7 5.5 6.33 7 5.5 7"></path></svg>"#;

// ============================================================================
// Main tag page
// ============================================================================

pub fn render_tag_page(
    tag: &TagInfo,
    items: &[TagFeedItem],
    has_more: bool,
    next_cursor: Option<i32>,
    user: Option<(i32, String, Option<String>)>,
    csrf_token: Option<String>,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "#" (tag.name) " - StringTechHub" }
                link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@500;600&family=Source+Serif+4:wght@400;500&family=IBM+Plex+Sans:wght@500&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/feed.css";
                link rel="stylesheet" href="/static/search.css";
                link rel="stylesheet" href="/static/refract.css";
                link rel="icon" type="image/x-icon" href="/static/favicon.ico";
                link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png";
                link rel="apple-touch-icon" sizes="180x180" href="/static/apple-touch-icon.png";
                link rel="manifest" href="/static/site.webmanifest";
                
                script src="/static/htmx.min.js" {}
                script src="/static/script.js" defer {}
                script src="/static/refract.js" defer {}
            }
            body {
                (render_header(user.as_ref(), csrf_token.as_deref()))

                main class="feed-container" {

                    // ── Tag header ──────────────────────────────────
                    div class="tag-page-header" {
                        div class="tag-page-title" {
                            (PreEscaped(ICON_TAG))
                            h1 { (tag.name) }
                        }
                        
                        div class="tag-page-stats" {
                            span {
                                strong { (tag.post_count) }
                                " post" (if tag.post_count == 1 { "" } else { "s" })
                            }
                            span class="tag-stats-sep" { "·" }
                            span {
                                strong { (tag.question_count) }
                                " question" (if tag.question_count == 1 { "" } else { "s" })
                            }
                        }
                    }

                    // ── Content ─────────────────────────────────────
                    @if items.is_empty() {
                        div class="search-empty-state" {
                            h2 class="empty-state-title" { "No content yet" }
                            p class="empty-state-text" {
                                "Nothing has been tagged with "
                                strong { "#" (tag.name) }
                                " yet."
                            }
                        }
                    } @else {
                        div class="feed-items" id="tag-items" {
                            @for item in items {
                                @match item {
                                    TagFeedItem::Post(post) => (render_tag_post_card(post)),
                                    TagFeedItem::Question(question) => (render_tag_question_card(question)),
                                }
                            }
                        }

                        @if has_more {
                            div
                                class="load-more-trigger"
                                hx-get=(format!("/tags/{}?cursor={}&limit=20", tag.slug, next_cursor.unwrap_or(0)))
                                hx-trigger="intersect once"
                                hx-target="#tag-items"
                                hx-swap="beforeend"
                            {
                                div class="loading-spinner" {}
                            }
                        }
                    }
                }

                button class="scroll-to-top" id="scroll-to-top" {
                    (PreEscaped(ICON_UP_ARROW))
                }

                @if user.is_some() {
                    (crate::templates::refract::render_refract_modal_empty(csrf_token.as_deref()))
                }
            }
        }
    }
}

// ============================================================================
// Post card — identical structure to feed/search cards
// ============================================================================

fn render_tag_post_card(post: &PostTagItem) -> Markup {
    let slug = post.slug.clone();
    html! {
        article class="feed-card"
            data-id=(post.id)
            data-slug=(post.slug)
            data-type="post"
        {
            span class="type-badge type-post" { "POST" }
            div class="card-header" {
                (render_avatar(post.avatar_url.as_deref(), &post.username))
                div class="author-info" {
                    a href=(format!("/@{}", post.username)) class="author-name" { (post.username) }
                    div class="meta-info" {
                        time { (format_time(&post.created_at)) }
                    }
                }
            }
            div class="card-content" {
                @if let Some(ref title) = post.title {
                    h2 class="content-title" {
                        a href=(format!("/posts/{}", post.slug)) { (title) }
                    }
                }
                div class="content-preview" {
                    (PreEscaped(truncate_html(&post.content_rendered_html, 500)))
                }
                @if should_show_read_more(&post.content_rendered_html, 500) {
                    a href=(format!("/posts/{}", post.slug)) class="read-more" { "Read more →" }
                }
            }
            @if !post.tags.is_empty() {
                div class="tags" {
                    @for tag in &post.tags {
                        // Tags link to /tags/{slug}
                        a href=(format!("/tags/{}", tag.slug)) class="tag" { (tag.name) }
                    }
                }
            }
            div class="action-bar" {
                a href=(format!("/posts/{}#comments", post.slug)) class="action-btn" title="Comments" {
                    (PreEscaped(ICON_COMMENT))
                    span class="count" { (post.comment_count) }
                }
                button class="action-btn refract-btn" title="Refracts" data-post-id=(post.id) {
                    (PreEscaped(ICON_REFRACT))
                    span class="count" { (post.refract_count) }
                }
                // Post card
                @if post.has_echoed {
                    span class="action-btn echoed" {
                        (PreEscaped(ICON_ECHO))
                            span class="count" { (post.echo_count) }
                    }
                } @else {
                    button class="action-btn action-echo"
                        data-echo-type="post"
                        data-echo-id=(post.id) {
                            (PreEscaped(ICON_ECHO))
                            
                        }
                }
                
                button
                    class="action-btn copy-link-btn"
                    data-copy-link=(format!("/posts/{}", slug))
                    title="Copy link"
                {
                    (PreEscaped(ICON_LINK))
                }
            }
        }
    }
}

// ============================================================================
// Question card
// ============================================================================

fn render_tag_question_card(question: &QuestionTagItem) -> Markup {
    let slug = question.slug.clone();
    html! {
        article class="feed-card"
            data-id=(question.id)
            data-slug=(question.slug)
            data-type="question"
        {
            span class="type-badge type-question" { "QUESTION" }
            div class="card-header" {
                (render_avatar(question.avatar_url.as_deref(), &question.username))
                div class="author-info" {
                    a href=(format!("/@{}", question.username)) class="author-name" { (question.username) }
                    div class="meta-info" {
                        time { (format_time(&question.created_at)) }
                    }
                }
            }
            div class="card-content" {
                h2 class="content-title" {
                    a href=(format!("/questions/{}", question.slug)) { (question.title) }
                }
                div class="content-preview" {
                    (PreEscaped(truncate_html(&question.content_rendered_html, 500)))
                }
                @if should_show_read_more(&question.content_rendered_html, 500) {
                    a href=(format!("/questions/{}", question.slug)) class="read-more" { "Read more →" }
                }
            }
            @if !question.tags.is_empty() {
                div class="tags" {
                    @for tag in &question.tags {
                        a href=(format!("/tags/{}", tag.slug)) class="tag" { (tag.name) }
                    }
                }
            }
            div class="action-bar" {
                a href=(format!("/questions/{}#comments", question.slug)) class="action-btn" title="Comments" {
                    (PreEscaped(ICON_COMMENT))
                    span class="count" { (question.comment_count) }
                }
                a href=(format!("/questions/{}#answers", question.slug)) class="action-btn" title="Answers" {
                    (PreEscaped(ICON_ANSWER))
                    span class="count" { (question.answer_count) }
                }
                // Post card
                @if question.has_echoed {
                    span class="action-btn echoed" {
                        (PreEscaped(ICON_ECHO))
                        span class="count" { (question.echo_count) }
                    }
                } @else {
                    button class="action-btn action-echo"
                        data-echo-type="question"
                        data-echo-id=(question.id) {
                            (PreEscaped(ICON_ECHO))
                            span class="count" { (question.echo_count) }
                        }
                }
                
                button
                    class="action-btn copy-link-btn"
                    data-copy-link=(format!("/questions/{}", slug))
                    title="Copy link"
                {
                    (PreEscaped(ICON_LINK))
                }
            }
        }
    }
}