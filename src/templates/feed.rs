use maud::{html, Markup, PreEscaped, DOCTYPE};
use time::{OffsetDateTime, macros::format_description};
use crate::{handlers::feed::{FeedItem, FeedResponse}};


const ICON_SEARCH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M18 10c0-4.41-3.59-8-8-8s-8 3.59-8 8 3.59 8 8 8c1.85 0 3.54-.63 4.9-1.69l5.1 5.1L21.41 20l-5.1-5.1A8 8 0 0 0 18 10M4 10c0-3.31 2.69-6 6-6s6 2.69 6 6-2.69 6-6 6-6-2.69-6-6"></path></svg>"#;
const ICON_POST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="m17.71 7.29-3-3a.996.996 0 0 0-1.41 0l-11.01 11A1 1 0 0 0 2 16v3c0 .55.45 1 1 1h3c.27 0 .52-.11.71-.29l11-11a.996.996 0 0 0 0-1.41ZM5.59 18H4v-1.59l7.5-7.5 1.59 1.59zm8.91-8.91L12.91 7.5 14 6.41 15.59 8zM11 18h11v2H11z"></path></svg>"#;
const ICON_QUESTION: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M12 2C6.49 2 2 6.49 2 12s4.49 10 10 10 10-4.49 10-10S17.51 2 12 2m0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8"></path><path d="M11 16h2v2h-2zm2.27-9.75c-2.08-.75-4.47.35-5.21 2.41l1.88.68c.18-.5.56-.9 1.07-1.13s1.08-.26 1.58-.08a2.01 2.01 0 0 1 1.32 1.86c0 1.04-1.66 1.86-2.24 2.07-.4.14-.67.52-.67.94v1h2v-.34c1.04-.51 2.91-1.69 2.91-3.68a4.015 4.015 0 0 0-2.64-3.73"></path></svg>"#;
const ICON_PROFILE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M12 7c-2 0-3.5 1.5-3.5 3.5S10 14 12 14s3.5-1.5 3.5-3.5S14 7 12 7m0 5c-.88 0-1.5-.62-1.5-1.5S11.12 9 12 9s1.5.62 1.5 1.5S12.88 12 12 12"></path><path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2M8.18 19c.41-1.16 1.51-2 2.82-2h2c1.3 0 2.4.84 2.82 2H8.19Zm9.71 0a5 5 0 0 0-4.9-4h-2c-2.41 0-4.43 1.72-4.9 4h-1.1V5h14v14z"></path></svg>"#;
const ICON_SETTINGS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M12 8c-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4-1.79-4-4-4m0 6c-1.08 0-2-.92-2-2s.92-2 2-2 2 .92 2 2-.92 2-2 2"></path><path d="m20.42 13.4-.51-.29c.05-.37.08-.74.08-1.11s-.03-.74-.08-1.11l.51-.29c.96-.55 1.28-1.78.73-2.73l-1-1.73a2.006 2.006 0 0 0-2.73-.73l-.53.31c-.58-.46-1.22-.83-1.9-1.11v-.6c0-1.1-.9-2-2-2h-2c-1.1 0-2 .9-2 2v.6c-.67.28-1.31.66-1.9 1.11l-.53-.31c-.96-.55-2.18-.22-2.73.73l-1 1.73c-.55.96-.22 2.18.73 2.73l.51.29c-.05.37-.08.74-.08 1.11s.03.74.08 1.11l-.51.29c-.96.55-1.28 1.78-.73 2.73l1 1.73c.55.95 1.77 1.28 2.73.73l.53-.31c.58.46 1.22.83 1.9 1.11v.6c0 1.1.9 2 2 2h2c1.1 0 2-.9 2-2v-.6a8.7 8.7 0 0 0 1.9-1.11l.53.31c.95.55 2.18.22 2.73-.73l1-1.73c.55-.96.22-2.18-.73-2.73m-2.59-2.78c.11.45.17.92.17 1.38s-.06.92-.17 1.38a1 1 0 0 0 .47 1.11l1.12.65-1 1.73-1.14-.66c-.38-.22-.87-.16-1.19.14-.68.65-1.51 1.13-2.38 1.4-.42.13-.71.52-.71.96v1.3h-2v-1.3c0-.44-.29-.83-.71-.96-.88-.27-1.7-.75-2.38-1.4a1.01 1.01 0 0 0-1.19-.15l-1.14.66-1-1.73 1.12-.65c.39-.22.58-.68.47-1.11-.11-.45-.17-.92-.17-1.38s.06-.93.17-1.38A1 1 0 0 0 5.7 9.5l-1.12-.65 1-1.73 1.14.66c.38.22.87.16 1.19-.14.68-.65 1.51-1.13 2.38-1.4.42-.13.71-.52.71-.96v-1.3h2v1.3c0 .44.29.83.71.96.88.27 1.7.75 2.38 1.4.32.31.81.36 1.19.14l1.14-.66 1 1.73-1.12.65c-.39.22-.58.68-.47 1.11Z"></path></svg>"#;
const ICON_LOGOUT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M9 13h7v-2H9V7l-6 5 6 5z"></path><path d="M19 3h-7v2h7v14h-7v2h7c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2"></path></svg>"#;
const ICON_DELETE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M17 6V4c0-1.1-.9-2-2-2H9c-1.1 0-2 .9-2 2v2H2v2h2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8h2V6zM9 4h6v2H9zM6 20V8h12v12z"></path><path d="M9 10h2v8H9zm4 0h2v8h-2z"></path></svg>"#;
const ICON_IMAGE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M5 21h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2m0-2v-1.59l3-3 1.29 1.29c.39.39 1.02.39 1.41 0l5.29-5.29 3 3V19h-14ZM19 5v5.59L16.71 8.3a.996.996 0 0 0-1.41 0l-5.29 5.29-1.29-1.29a.996.996 0 0 0-1.41 0l-2.29 2.29V5h14Z"></path><path d="M8.5 7a1.5 1.5 0 1 0 0 3 1.5 1.5 0 1 0 0-3"></path></svg>"#;
const ICON_CODE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="m16.71 16.71 4.7-4.71-4.7-4.71-1.42 1.42 3.3 3.29-3.3 3.29zm-8-1.42L5.41 12l3.3-3.29-1.42-1.42L2.59 12l4.7 4.71zM14 3l-.98-.22-2 9-2 9L10 21l.98.22 2-9 2-9z"></path></svg>"#;
const ICON_COMMENT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M20 2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h6.72l4.76 2.86c.16.09.34.14.51.14s.34-.04.49-.13c.31-.18.51-.51.51-.87v-2h3c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2Zm0 14h-4c-.55 0-1 .45-1 1v1.23l-3.49-2.09A1.03 1.03 0 0 0 11 16H4V4h16z"></path></svg>"#;
const ICON_REFRACT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M17 5H6c-1.1 0-2 .9-2 2v5h2V7h11v3l5-4-5-4zm1 12H7v-3l-5 4 5 4v-3h11c1.1 0 2-.9 2-2v-5h-2z"></path></svg>"#;
const ICON_ECHO: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M14 12c0-1.1-.9-2-2-2s-2 .9-2 2a2 2 0 1 0 4 0m-6 0c0-1.07.42-2.07 1.17-2.82L7.76 7.76A5.97 5.97 0 0 0 6 12c0 1.6.62 3.11 1.76 4.25l1.41-1.42A3.96 3.96 0 0 1 8 12m8.24-4.24-1.41 1.41C15.59 9.93 16 10.93 16 12s-.42 2.07-1.17 2.83l1.41 1.41C17.37 15.11 18 13.6 18 12s-.62-3.11-1.76-4.24"></path><path d="M6.34 17.66C4.83 16.15 3.99 14.14 3.99 12s.83-4.14 2.34-5.65L4.92 4.93C3.03 6.82 1.99 9.33 1.99 12s1.04 5.18 2.93 7.07l1.41-1.41ZM19.07 4.93l-1.41 1.41C19.17 7.85 20 9.86 20 12s-.83 4.15-2.34 5.66l1.41 1.41C20.96 17.18 22 14.67 22 12s-1.04-5.18-2.93-7.07"></path></svg>"#;
const ICON_LINK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M9.88 18.36a3 3 0 0 1-4.24 0 3 3 0 0 1 0-4.24l2.83-2.83-1.41-1.41-2.83 2.83a5.003 5.003 0 0 0 0 7.07c.98.97 2.25 1.46 3.54 1.46s2.56-.49 3.54-1.46l2.83-2.83-1.41-1.41-2.83 2.83Zm2.83-14.14L9.88 7.05l1.41 1.41 2.83-2.83a3 3 0 0 1 4.24 0 3 3 0 0 1 0 4.24l-2.83 2.83 1.41 1.41 2.83-2.83a5.003 5.003 0 0 0 0-7.07 5.003 5.003 0 0 0-7.07 0Z"></path><path d="m16.95 8.46-.71-.7-.7-.71-4.25 4.24-4.24 4.25.71.7.7.71 4.25-4.24z"></path></svg>"#;
const ICON_UP_ARROW: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="m6.293 11.293 1.414 1.414L12 8.414l4.293 4.293 1.414-1.414L12 5.586z"></path><path d="m6.293 16.293 1.414 1.414L12 13.414l4.293 4.293 1.414-1.414L12 10.586z"></path></svg>"#;
const ICON_EXPAND: &str = r#"<svg class="icon-expand" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg>"#;
const ICON_COLLAPSE: &str = r#"<svg class="icon-collapse" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="display:none"><polyline points="18 15 12 9 6 15"></polyline></svg>"#;
const ICON_ANSWER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M12 2C6.49 2 2 6.49 2 12s4.49 10 10 10h9c.37 0 .71-.21.89-.54.17-.33.15-.73-.06-1.03l-1.75-2.53a10 10 0 0 0 1.93-5.9c0-5.51-4.49-10-10-10Zm6 16.43L19.09 20H12c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8c0 1.91-.69 3.75-1.93 5.21-.3.34-.32.85-.06 1.22Z"></path>
                </svg>"#;
const ICON_MAIL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4-8 5-8-5V6l8 5 8-5v2z"/></svg>"#;


pub fn render_feed_page(
    feed: FeedResponse, 
    user: Option<(i32, String, Option<String>)>, 
    csrf_token: Option<String>, 
    show_logout: bool, 
    show_deleted: bool,
    auth_error: Option<&str>,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "StringTechHub - Developer Community" }
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
                script src="/static/create.js" defer {}

                @if let Some(ref token) = csrf_token {
                    meta name="csrf-token" content=(token);
                }
            }
            body {
                span id="top" {}
                (render_header(user.as_ref(), csrf_token.as_deref()))

                main class="feed-container" {

                    div class="feed-items" id="feed-items" {
                        @for item in &feed.data {
                            (render_feed_item(item))
                        }
                    }

                    @if feed.has_more {
                        div
                            class="load-more-trigger"
                            hx-get=(format!("/api/feed?cursor={}&limit=25", feed.next_cursor.unwrap_or(0)))
                            hx-trigger="intersect once"
                            hx-target="#feed-items"
                            hx-swap="beforehand"
                        {
                            div class="loading-spinner" {
                                i class="bx bx-loader-alt bx-spin" {}
                            }
                        }
                    }
                }

                button class="scroll-to-top" id="scroll-to-top" {
                    (PreEscaped(ICON_UP_ARROW))
                }

                @if show_logout {
                    div class="toast toast-success" {
                        div class="toast-icon" {
                            (PreEscaped(r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>"#))
                        }
                        "Logged out successfully"
                    }
                }

                @if show_deleted {
                    div class="toast toast-danger" {
                        div class="toast-icon" {
                            (PreEscaped(r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>"#))
                        }
                        "Account deleted"
                    }
                }

                @if let Some(_) = auth_error {
                    div class="toast toast-danger" {
                        div class="toast-icon" {
                            (PreEscaped(r#"<<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>"#))
                        }
                        "Login failed. Please try again."
                    }
                }

                @if user.is_some() {
                    (crate::templates::refract::render_refract_modal_empty(csrf_token.as_deref()))
                }
            }
        }
    }
}

pub fn render_header(user: Option<&(i32, String, Option<String>)>, csrf_token: Option<&str>) -> Markup {
    html! {
        header class="header" {
            div class="topbar" {
                a href="/" class="logo" { "StringTechHub" }

                div class="search-container" {
                    form class="search-bar" action="/search" method="get" id="search-form" {
                        (PreEscaped(ICON_SEARCH))

                        div class="search-tags-container" id="search-tags" {}

                        input 
                            type="text" 
                            id="search-input" 
                            placeholder="Search... (type /tag for tags)" 
                            autocomplete="off";

                        input
                            type="hidden"
                            name="q"
                            id="search-query-hidden"
                            value="";
                    }
                }

                div class="actions" {
                    @if let Some((_, username, _)) = user {
                        a href="/create/post" class="btn btn-post" {
                            (PreEscaped(ICON_POST))
                            span { "Post" }
                        }
                        a href="/create/question" class="btn btn-ask" {
                            (PreEscaped(ICON_QUESTION))
                            span { "Ask Question" }
                        }
                        a href=(format!("/@{}", username)) class="btn-profile" {
                            (PreEscaped(ICON_PROFILE))
                        }
                        div class="settings-dropdown" {
                            button class="btn-settings" id="settings-btn" aria-haspopup="true" aria-expanded="false" {
                                (PreEscaped(ICON_SETTINGS))
                            }
                            div class="settings-menu" role="menu" {

                                a href="mailto:support@stringtechhub.com" 
                                    class="settings-item settings-item-divider"
                                    target="_blank"
                                {
                                    (PreEscaped(ICON_MAIL))
                                    "support@stringtechhub.com"
                                }

                                    // Logout Button
                                button 
                                    hx-post="/auth/logout" 
                                    hx-headers=(format!(r#"{{"X-CSRF-Token": "{}"}}"#, csrf_token.unwrap_or(" ")))
                                    class="settings-item settings-item-divider" 
                                {
                                    (PreEscaped(ICON_LOGOUT))
                                    "Logout"
                                }

                                    // Delete Account Button
                                button 
                                    hx-delete="/auth/delete-account" 
                                    hx-headers=(format!(r#"{{"X-CSRF-Token": "{}"}}"#, csrf_token.unwrap_or(" ")))
                                    class="settings-item danger" 
                                {
                                    (PreEscaped(ICON_DELETE))
                                    "Delete account"
                                }
                                
                            }
                        }
                    } @else {
                        a href="/auth/github" class="btn btn-post" { span { "Sign Up" } }
                        a href="/auth/github" class="btn btn-ask" { span { "Login" } }
                    }
                }
            }
        }
    }
}


// ============================================================================
// Feed Item Dispatcher
// ============================================================================

pub fn render_feed_item(item: &FeedItem) -> Markup {
    match item {
        FeedItem::Post(post) => render_post_card(post),
        FeedItem::Question(question) => render_question_card(question),
        FeedItem::Refract(refract) => render_refract_card(refract),
    }
}

// ============================================================================
// Post Card
// ============================================================================

pub fn render_post_card(post: &crate::handlers::feed::PostFeedItem) -> Markup {
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
// Question Card
// ============================================================================

fn render_question_card(question: &crate::handlers::feed::QuestionFeedItem) -> Markup {
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

// ============================================================================
// Refract Card
// ============================================================================

pub fn render_refract_card(refract: &crate::handlers::feed::RefractFeedItem) -> Markup {
    let refract_id = refract.id;
    let original_slug = refract.original_post.slug.clone();
    html! {
        article class="feed-card refract-card" data-id=(refract.id) {
            span class="type-badge type-refract" { "REFRACT" }
            div class="card-header" {
                (render_avatar(refract.avatar_url.as_deref(), &refract.username))
                div class="author-info" {
                    a href=(format!("/@{}", refract.username)) class="author-name" { (refract.username) }
                    div class="meta-info" {
                        time { (format_time(&refract.created_at)) }
                    }
                }
            }

            // Commentary with expand/collapse
            div class="card-content" {
                div class="refract-commentary" {
                    div class="content-preview" id=(format!("refract-preview-{}", refract_id)) {
                        (PreEscaped(truncate_html(&refract.content_rendered_html, 300)))
                    }
                    @if should_show_read_more(&refract.content_rendered_html, 300) {
                        div class="content-hidden" id=(format!("refract-full-{}", refract_id)) style="display:none" {
                            (PreEscaped(&refract.content_rendered_html))
                        }
                        button
                            class="expand-btn"
                            onclick=(format!("toggleRefractExpand({})", refract_id))
                        {
                            (PreEscaped(ICON_EXPAND))
                            (PreEscaped(ICON_COLLAPSE))
                            span class="expand-text" { "See more" }
                        }
                    }
                }
            }

            // Embedded original post (shown after expand)
            @if !refract.original_post.is_deleted {
                div
                    class="embedded-post"
                    id=(format!("embedded-post-{}", refract_id))
                    data-href=(format!("/posts/{}", original_slug))
                {
                    div class="embedded-header" {
                        (render_avatar(refract.original_post.avatar_url.as_deref() , &refract.original_post.username))
                        div class="author-info" {
                            span class="author-name" { (refract.original_post.username) }
                            div class="meta-info" {
                                time { (format_time(&refract.original_post.created_at)) }
                            }
                        }
                    }
                    @if let Some(ref title) = refract.original_post.title {
                        h3 class="embedded-title" { (title) }
                    }
                    div class="embedded-preview" {
                        (PreEscaped(truncate_html(&refract.original_post.content_rendered_html, 200)))
                    }
                }
            }

            div class="action-bar" {
                // Post card
                @if refract.has_echoed {
                    span class="action-btn echoed" {
                        (PreEscaped(ICON_ECHO))
                        span class="count" { (refract.echo_count) }
                    }
                } @else {
                    button class="action-btn action-echo"
                        data-echo-type="refract"
                        data-echo-id=(refract.id) {
                            (PreEscaped(ICON_ECHO))
                            
                        }
                }

                button
                    class="action-btn"
                    onclick=(format!("navigator.clipboard.writeText(window.location.origin+'/refracts/{}')", refract.id))
                    title="Copy link"
                {
                    (PreEscaped(ICON_LINK))
                }
            }
        }
    }
}

// ============================================================================
// Avatar
// ============================================================================

pub fn render_avatar(avatar_url: Option<&str>, username: &str) -> Markup {
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

// ============================================================================
// Helpers
// ============================================================================

pub fn format_time(created_at: &OffsetDateTime) -> String {
    let now = OffsetDateTime::now_utc();
    let seconds = (now - *created_at).as_seconds_f64();
    match seconds {
        s if s < 60.0 => "just now".to_string(),
        s if s < 3600.0 => format!("{}m ago", (s / 60.0) as u64),
        s if s < 86400.0 => format!("{}h ago", (s / 3600.0) as u64),
        s if s < 2592000.0 => format!("{}d ago", (s / 86400.0) as u64),
        _ => created_at
            .format(&format_description!("[month repr:short] [day], [year]"))
            .unwrap_or_else(|_| "Unknown".to_string()),
    }
}

pub fn strip_tags(html: &str) -> String {
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

pub fn truncate_html(html: &str, max_chars: usize) -> String {
    let text_len = strip_tags(html).len();
    if text_len <= max_chars {
        return html.to_string();
    }

    let mut char_count = 0;
    let mut result = String::new();
    let mut in_tag = false;
    let mut pos = 0;
    let chars: Vec<char> = html.chars().collect();


   while pos < html.len() {
            let remaining = &html[pos..];
            if remaining.starts_with(r#"<div class="code-block-wrapper"#) {
                // Find the end of this code block
                if let Some(end_offset) = remaining.find("</pre>\n</div>") {
                    let end = end_offset + "</pre>\n</div>\n".len();
                    let block = &remaining[..end.min(remaining.len())];
                    if char_count < max_chars {
                        // Include the full code block
                        result.push_str(block);

                    } 
                    pos += block.len();
                    continue;
                }
            }
            let ch = &html[pos..].chars().next().unwrap();
            let ch_len = ch.len_utf8();

            if *ch == '<' {
                in_tag = true;
            } else if *ch == '>' {
                in_tag = false;
            } else if !in_tag {
                char_count += 1;
                if char_count > max_chars {
                    result.push_str("...");
                    return result;
                }
            }
            result.push(*ch);
            pos += ch_len;
    }
    result
}

pub fn should_show_read_more(html: &str, truncate_at: usize) -> bool {
    strip_tags(html).len() > truncate_at
}