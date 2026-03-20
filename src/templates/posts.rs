use maud::{html, Markup, PreEscaped, DOCTYPE};
use crate::handlers::{comments::{self, CommentResponse}, posts::PostResponse, questions::QuestionResponse};
use time::OffsetDateTime;
// Import icons (you'll fill these)
const ICON_BACK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24">
                <path d="M21 11H6.83l3.58-3.59L9 6l-6 6 6 6 1.41-1.41L6.83 13H21z"></path>
            </svg>"#;
const ICON_COMMENT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M20 2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h6.72l4.76 2.86c.16.09.34.14.51.14s.34-.04.49-.13c.31-.18.51-.51.51-.87v-2h3c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2Zm0 14h-4c-.55 0-1 .45-1 1v1.23l-3.49-2.09A1.03 1.03 0 0 0 11 16H4V4h16z"></path></svg>"#;
const ICON_ANSWER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M12 2C6.49 2 2 6.49 2 12s4.49 10 10 10h9c.37 0 .71-.21.89-.54.17-.33.15-.73-.06-1.03l-1.75-2.53a10 10 0 0 0 1.93-5.9c0-5.51-4.49-10-10-10Zm6 16.43L19.09 20H12c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8c0 1.91-.69 3.75-1.93 5.21-.3.34-.32.85-.06 1.22Z"></path>
                </svg>"#;
const ICON_REFRACT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M17 5H6c-1.1 0-2 .9-2 2v5h2V7h11v3l5-4-5-4zm1 12H7v-3l-5 4 5 4v-3h11c1.1 0 2-.9 2-2v-5h-2z"></path></svg>"#;
const ICON_ECHO: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M14 12c0-1.1-.9-2-2-2s-2 .9-2 2a2 2 0 1 0 4 0m-6 0c0-1.07.42-2.07 1.17-2.82L7.76 7.76A5.97 5.97 0 0 0 6 12c0 1.6.62 3.11 1.76 4.25l1.41-1.42A3.96 3.96 0 0 1 8 12m8.24-4.24-1.41 1.41C15.59 9.93 16 10.93 16 12s-.42 2.07-1.17 2.83l1.41 1.41C17.37 15.11 18 13.6 18 12s-.62-3.11-1.76-4.24"></path><path d="M6.34 17.66C4.83 16.15 3.99 14.14 3.99 12s.83-4.14 2.34-5.65L4.92 4.93C3.03 6.82 1.99 9.33 1.99 12s1.04 5.18 2.93 7.07l1.41-1.41ZM19.07 4.93l-1.41 1.41C19.17 7.85 20 9.86 20 12s-.83 4.15-2.34 5.66l1.41 1.41C20.96 17.18 22 14.67 22 12s-1.04-5.18-2.93-7.07"></path></svg>"#;
const ICON_LINK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M9.88 18.36a3 3 0 0 1-4.24 0 3 3 0 0 1 0-4.24l2.83-2.83-1.41-1.41-2.83 2.83a5.003 5.003 0 0 0 0 7.07c.98.97 2.25 1.46 3.54 1.46s2.56-.49 3.54-1.46l2.83-2.83-1.41-1.41-2.83 2.83Zm2.83-14.14L9.88 7.05l1.41 1.41 2.83-2.83a3 3 0 0 1 4.24 0 3 3 0 0 1 0 4.24l-2.83 2.83 1.41 1.41 2.83-2.83a5.003 5.003 0 0 0 0-7.07 5.003 5.003 0 0 0-7.07 0Z"></path><path d="m16.95 8.46-.71-.7-.7-.71-4.25 4.24-4.24 4.25.71.7.7.71 4.25-4.24z"></path></svg>"#;
/// Full post page
pub fn render_post_page(
    post: PostResponse, 
    user: Option<(i32, String, Option<String>)>, 
    csrf_token: Option<String>, 
    comments: Vec<CommentResponse>, 
    total_count: i32, 
    has_more: bool, 
    next_cursor: Option<String>,
    back_url: String,
) -> Markup {

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (post.title.as_deref().unwrap_or("Post")) " - StringTechHub" }
                link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@500;600&family=Source+Serif+4:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=IBM+Plex+Mono:wght@400;500&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/feed.css";
                link rel="stylesheet" href="/static/search.css";
                link rel="stylesheet" href="/static/post.css";
                link rel="stylesheet" href="/static/comments.css";
                link rel="stylesheet" href="/static/refract.css";
                script src="/static/htmx.min.js" {}
                script src="/static/marked.min.js" {}
                script src="/static/script.js" defer {}
                script src="/static/comment.js" defer {}
                script src="/static/refract.js" defer {}

                @if let Some(ref token) = csrf_token {
                    meta name="csrf-token" content=(token);
                }
            }
            body {
            (crate::templates::feed::render_header(user.as_ref(), csrf_token.as_deref()))
                        div class="full-question-page" {
                            div class="container" {
                                a href=(back_url) class="back-button" {
                                    (PreEscaped(ICON_BACK))
                                    "Back"
                                }
                                
                                div class="question-content" {
                                    div class="question-header" {
                                        span class="type-badge" { "POST" }
                                        
                                        @if let Some(ref title) = post.title {
                                            h1 { (title) }
                                        }
                                        
                                        div class="card-header" {
                                            (render_avatar(post.avatar_url.as_deref(), &post.username))
                                            div class="author-info" {
                                                a href=(format!("/@{}", post.username)) class="author-name" { (post.username) }
                                                div class="meta-info" {
                                                    (format_time(&post.created_at))
                                                    @if let Some(ref edited_at) = post.edited_at {
                                                        " · Edited " (format_time(edited_at))
                                                    }
                                                }
                                            }
                                        }
                                        
                                        @if !post.tags.is_empty() {
                                            div class="tags" {
                                                @for tag in &post.tags {
                                                    a href=(format!("/tags/{}", tag.slug)) class="tag" { (tag.name) }
                                                }
                                            }
                                        }
                                    }
                                    
                                    div class="question-body" {
                                        div class="markdown-content" {
                                            (PreEscaped(&post.content_rendered_html))
                                        }
                                    }
                                    
                                    div class="question-actions" {
                                        div class="action-bar" {
                                            span class="action-btn" {
                                                (PreEscaped(ICON_COMMENT))
                                                span { (total_count) " comments" }
                                            }
                                            button class="action-btn refract-btn" data-post-id=(post.id) {
                                                (PreEscaped(ICON_REFRACT))
                                                span { (post.refract_count) " refracts" }
                                            }
                                            
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
                                            data-copy-link=(format!("/posts/{}", post.slug))
                                            title="Copy link"
                                            
                                        {
                                            (PreEscaped(ICON_LINK))
                                            span { "Copy link" }
                                        }
                                    }
                                }
                                (crate::templates::comments::render_comments_section(
                                    "post",
                                    post.id,
                                    post.user_id,
                                    comments,
                                    total_count,
                                    has_more,
                                    next_cursor,
                                    user.clone(),
                                    csrf_token.as_deref(),
                                ))
                            }
                        }
                    }
                    @if user.is_some() {
                        (crate::templates::refract::render_refract_modal_empty(csrf_token.as_deref()))
                    }
                }
            }
    }
}
/// Full question page
pub fn render_question_page(
    question: QuestionResponse, 
    user: Option<(i32, String, Option<String>)>, 
    csrf_token: Option<String>, 
    comments: Vec<CommentResponse>, 
    total_count: i32,
    has_more: bool, 
    next_cursor: Option<String>,
    answers: Vec<crate::handlers::answers::AnswerResponse>,
    answer_count: i32,
    answers_has_more: bool,
    answers_next_cursor: Option<i32>,
    back_url: String,
) -> Markup {
html! {
    (DOCTYPE)
    html lang="en" {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title { (question.title) " - StringTechHub" }
            link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@500;600&family=Source+Serif+4:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=IBM+Plex+Mono:wght@400;500&display=swap" rel="stylesheet";
            link rel="stylesheet" href="/static/feed.css";
            link rel="stylesheet" href="/static/search.css";
            link rel="stylesheet" href="/static/post.css";
            link rel="stylesheet" href="/static/comments.css";
            link rel="stylesheet" href="/static/answer.css";
            link rel="icon" type="image/x-icon" href="/static/favicon.ico";
            link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png";
            link rel="icon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png";
            link rel="apple-touch-icon" sizes="180x180" href="/static/apple-touch-icon.png";
            link rel="manifest" href="/static/site.webmanifest";


            script src="/static/htmx.min.js" {}
            script src="/static/marked.min.js" {}
            script src="/static/script.js" defer {}
            script src="/static/comment.js" defer {}
            script src="/static/answers.js" defer {}

            @if let Some(ref token) = csrf_token {
                meta name="csrf-token" content=(token);
            }
        }
        body {
        (crate::templates::feed::render_header(user.as_ref(), csrf_token.as_deref()))
                div class="full-question-page" {
                    div class="container" {
                        a href=(back_url) class="back-button" {
                            (PreEscaped(ICON_BACK))
                            "Back"
                        }
                        
                        div class="question-content" {
                            div class="question-header" {
                                span class="type-badge" { "QUESTION" }
                                h1 { (question.title) }
                                
                                div class="card-header" {
                                    (render_avatar(None, &question.username))
                                    div class="author-info" {
                                        a href=(format!("/@{}", question.username)) class="author-name" { (question.username) }
                                        div class="meta-info" {
                                            (format_time(&question.created_at))
                                            @if let Some(ref edited_at) = question.edited_at {
                                                " · Edited " (format_time(edited_at))
                                            }
                                        }
                                    }
                                }
                                
                                @if !question.tags.is_empty() {
                                    div class="tags" {
                                        @for tag in &question.tags {
                                            a href=(format!("/tags/{}", tag.slug)) class="tag" { (tag.name) }
                                        }
                                    }
                                }
                            }
                            
                            div class="question-body" {
                                div class="markdown-content" {
                                    (PreEscaped(&question.content_rendered_html))
                                }
                            }
                            
                            div class="question-actions" {
                                div class="action-bar" { 
                                    button class="action-btn tab-btn active" id="comments-tab" {
                                        (PreEscaped(ICON_COMMENT))
                                        span { (total_count) " comments" }
                                    }
                                    button class="action-btn tab-btn active" id="answers-tab" {
                                        (PreEscaped(ICON_ANSWER))
                                        span { (question.answer_count) " answers" }
                                    }

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
                                                
                                            }
                                    }

                                    button 
                                        class="action-btn copy-link-btn" 
                                        data-copy-link=(format!("/questions/{}", question.slug))
                                        title="Copy link"
                                        
                                    {
                                        (PreEscaped(ICON_LINK))
                                        span { "Copy link" }
                                    }
                                }
                            }

                            div id="answers-section" class="tab-content" {
                                (crate::templates::answer::render_answers_section(
                                    question.id, 
                                    question.user_id, 
                                    &question.slug, 
                                    answers, 
                                    answer_count, 
                                    answers_has_more, 
                                    answers_next_cursor, 
                                    user.clone(),
                                    csrf_token.as_deref()
                                ))
                            }
                            div id="comments-section" class="tab-content" style="display:none" {
                                (crate::templates::comments::render_comments_section(
                                    "question",
                                    question.id,
                                    question.user_id,
                                    comments,
                                    total_count,
                                    has_more,
                                    next_cursor,
                                    user.clone(),
                                    csrf_token.as_deref(),
                                ))
                            }
                        }
                    }
                }
            }
        }
    }
}
// Helper functions
fn render_avatar(avatar_url: Option<&str>, username: &str) -> Markup {
    let initial = username.chars().next().unwrap_or('?').to_uppercase().to_string();
    html! {
        div class="avatar" {
            @if let Some(url) = avatar_url {
            img src=(url) alt=(username);
            } @else {
            span class="avatar-placeholder" { (initial) }
            }
        }
    }
}
fn format_time(created_at: &OffsetDateTime) -> String {
    let now = OffsetDateTime::now_utc();
    let seconds = (now - *created_at).as_seconds_f64();
    match seconds {
        s if s < 60.0 => "just now".to_string(),
        s if s < 3600.0 => format!("{}m ago", (s / 60.0) as u64),
        s if s < 86400.0 => format!("{}h ago", (s / 3600.0) as u64),
        s if s < 2592000.0 => format!("{}d ago", (s / 86400.0) as u64),
        _ => {
        use time::macros::format_description;
        created_at.format(&format_description!("[month repr:short] [day], [year]"))
        .unwrap_or_else(|_| "Unknown".to_string())
        }
    }
}