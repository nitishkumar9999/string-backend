use maud::{html, Markup, PreEscaped, DOCTYPE};
use time::macros::format_description;

use crate::handlers::feed::RefractFeedItem;
use crate::templates::feed::{
    strip_tags,
    truncate_html,
    should_show_read_more,
    render_avatar,
    render_refract_card,
};

// Reuse SVG icons from feed.rs
const ICON_SEARCH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M18 10c0-4.41-3.59-8-8-8s-8 3.59-8 8 3.59 8 8 8c1.85 0 3.54-.63 4.9-1.69l5.1 5.1L21.41 20l-5.1-5.1A8 8 0 0 0 18 10M4 10c0-3.31 2.69-6 6-6s6 2.69 6 6-2.69 6-6 6-6-2.69-6-6"></path></svg>"#;
const ICON_POST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="m17.71 7.29-3-3a.996.996 0 0 0-1.41 0l-11.01 11A1 1 0 0 0 2 16v3c0 .55.45 1 1 1h3c.27 0 .52-.11.71-.29l11-11a.996.996 0 0 0 0-1.41ZM5.59 18H4v-1.59l7.5-7.5 1.59 1.59zm8.91-8.91L12.91 7.5 14 6.41 15.59 8zM11 18h11v2H11z"></path></svg>"#;
const ICON_QUESTION: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M12 2C6.49 2 2 6.49 2 12s4.49 10 10 10 10-4.49 10-10S17.51 2 12 2m0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8"></path><path d="M11 16h2v2h-2zm2.27-9.75c-2.08-.75-4.47.35-5.21 2.41l1.88.68c.18-.5.56-.9 1.07-1.13s1.08-.26 1.58-.08a2.01 2.01 0 0 1 1.32 1.86c0 1.04-1.66 1.86-2.24 2.07-.4.14-.67.52-.67.94v1h2v-.34c1.04-.51 2.91-1.69 2.91-3.68a4.015 4.015 0 0 0-2.64-3.73"></path></svg>"#;
const ICON_PROFILE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M12 7c-2 0-3.5 1.5-3.5 3.5S10 14 12 14s3.5-1.5 3.5-3.5S14 7 12 7m0 5c-.88 0-1.5-.62-1.5-1.5S11.12 9 12 9s1.5.62 1.5 1.5S12.88 12 12 12"></path><path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2M8.18 19c.41-1.16 1.51-2 2.82-2h2c1.3 0 2.4.84 2.82 2H8.19Zm9.71 0a5 5 0 0 0-4.9-4h-2c-2.41 0-4.43 1.72-4.9 4h-1.1V5h14v14z"></path></svg>"#;
const ICON_SETTINGS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M12 8c-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4-1.79-4-4-4m0 6c-1.08 0-2-.92-2-2s.92-2 2-2 2 .92 2 2-.92 2-2 2"></path><path d="m20.42 13.4-.51-.29c.05-.37.08-.74.08-1.11s-.03-.74-.08-1.11l.51-.29c.96-.55 1.28-1.78.73-2.73l-1-1.73a2.006 2.006 0 0 0-2.73-.73l-.53.31c-.58-.46-1.22-.83-1.9-1.11v-.6c0-1.1-.9-2-2-2h-2c-1.1 0-2 .9-2 2v.6c-.67.28-1.31.66-1.9 1.11l-.53-.31c-.96-.55-2.18-.22-2.73.73l-1 1.73c-.55.96-.22 2.18.73 2.73l.51.29c-.05.37-.08.74-.08 1.11s.03.74.08 1.11l-.51.29c-.96.55-1.28 1.78-.73 2.73l1 1.73c.55.95 1.77 1.28 2.73.73l.53-.31c.58.46 1.22.83 1.9 1.11v.6c0 1.1.9 2 2 2h2c1.1 0 2-.9 2-2v-.6a8.7 8.7 0 0 0 1.9-1.11l.53.31c.95.55 2.18.22 2.73-.73l1-1.73c.55-.96.22-2.18-.73-2.73m-2.59-2.78c.11.45.17.92.17 1.38s-.06.92-.17 1.38a1 1 0 0 0 .47 1.11l1.12.65-1 1.73-1.14-.66c-.38-.22-.87-.16-1.19.14-.68.65-1.51 1.13-2.38 1.4-.42.13-.71.52-.71.96v1.3h-2v-1.3c0-.44-.29-.83-.71-.96-.88-.27-1.7-.75-2.38-1.4a1.01 1.01 0 0 0-1.19-.15l-1.14.66-1-1.73 1.12-.65c.39-.22.58-.68.47-1.11-.11-.45-.17-.92-.17-1.38s.06-.93.17-1.38A1 1 0 0 0 5.7 9.5l-1.12-.65 1-1.73 1.14.66c.38.22.87.16 1.19-.14.68-.65 1.51-1.13 2.38-1.4.42-.13.71-.52.71-.96v-1.3h2v1.3c0 .44.29.83.71.96.88.27 1.7.75 2.38 1.4.32.31.81.36 1.19.14l1.14-.66 1 1.73-1.12.65c-.39.22-.58.68-.47 1.11Z"></path></svg>"#;
const ICON_LOGOUT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M9 13h7v-2H9V7l-6 5 6 5z"></path><path d="M19 3h-7v2h7v14h-7v2h7c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2"></path></svg>"#;
const ICON_DELETE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M17 6V4c0-1.1-.9-2-2-2H9c-1.1 0-2 .9-2 2v2H2v2h2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8h2V6zM9 4h6v2H9zM6 20V8h12v12z"></path><path d="M9 10h2v8H9zm4 0h2v8h-2z"></path></svg>"#;
const ICON_COMMENT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M20 2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h6.72l4.76 2.86c.16.09.34.14.51.14s.34-.04.49-.13c.31-.18.51-.51.51-.87v-2h3c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2Zm0 14h-4c-.55 0-1 .45-1 1v1.23l-3.49-2.09A1.03 1.03 0 0 0 11 16H4V4h16z"></path></svg>"#;
const ICON_ECHO: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M14 12c0-1.1-.9-2-2-2s-2 .9-2 2a2 2 0 1 0 4 0m-6 0c0-1.07.42-2.07 1.17-2.82L7.76 7.76A5.97 5.97 0 0 0 6 12c0 1.6.62 3.11 1.76 4.25l1.41-1.42A3.96 3.96 0 0 1 8 12m8.24-4.24-1.41 1.41C15.59 9.93 16 10.93 16 12s-.42 2.07-1.17 2.83l1.41 1.41C17.37 15.11 18 13.6 18 12s-.62-3.11-1.76-4.24"></path><path d="M6.34 17.66C4.83 16.15 3.99 14.14 3.99 12s.83-4.14 2.34-5.65L4.92 4.93C3.03 6.82 1.99 9.33 1.99 12s1.04 5.18 2.93 7.07l1.41-1.41ZM19.07 4.93l-1.41 1.41C19.17 7.85 20 9.86 20 12s-.83 4.15-2.34 5.66l1.41 1.41C20.96 17.18 22 14.67 22 12s-1.04-5.18-2.93-7.07"></path></svg>"#;
const ICON_SHARE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M5.5 15.5c1.07 0 2.02-.5 2.67-1.26l6.87 3.87c-.01.13-.04.26-.04.39 0 1.93 1.57 3.5 3.5 3.5s3.5-1.57 3.5-3.5-1.57-3.5-3.5-3.5c-1.07 0-2.02.5-2.67 1.26l-6.87-3.87c.01-.13.04-.26.04-.39s-.02-.26-.04-.39l6.87-3.87C16.47 8.5 17.42 9 18.5 9 20.43 9 22 7.43 22 5.5S20.43 2 18.5 2 15 3.57 15 5.5c0 .13.02.26.04.39L8.17 9.76A3.48 3.48 0 0 0 5.5 8.5C3.57 8.5 2 10.07 2 12s1.57 3.5 3.5 3.5m13 1.5c.83 0 1.5.67 1.5 1.5s-.67 1.5-1.5 1.5-1.5-.67-1.5-1.5.67-1.5 1.5-1.5m0-13c.83 0 1.5.67 1.5 1.5S19.33 7 18.5 7 17 6.33 17 5.5 17.67 4 18.5 4m-13 6.5c.83 0 1.5.67 1.5 1.5s-.67 1.5-1.5 1.5S4 12.83 4 12s.67-1.5 1.5-1.5"></path></svg>"#;
const ICON_LINK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M9.88 18.36a3 3 0 0 1-4.24 0 3 3 0 0 1 0-4.24l2.83-2.83-1.41-1.41-2.83 2.83a5.003 5.003 0 0 0 0 7.07c.98.97 2.25 1.46 3.54 1.46s2.56-.49 3.54-1.46l2.83-2.83-1.41-1.41-2.83 2.83Zm2.83-14.14L9.88 7.05l1.41 1.41 2.83-2.83a3 3 0 0 1 4.24 0 3 3 0 0 1 0 4.24l-2.83 2.83 1.41 1.41 2.83-2.83a5.003 5.003 0 0 0 0-7.07 5.003 5.003 0 0 0-7.07 0Z"></path><path d="m16.95 8.46-.71-.7-.7-.71-4.25 4.24-4.24 4.25.71.7.7.71 4.25-4.24z"></path></svg>"#;
const ICON_ANSWER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M12 2C6.49 2 2 6.49 2 12s4.49 10 10 10h9c.37 0 .71-.21.89-.54.17-.33.15-.73-.06-1.03l-1.75-2.53a10 10 0 0 0 1.93-5.9c0-5.51-4.49-10-10-10Zm6 16.43L19.09 20H12c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8c0 1.91-.69 3.75-1.93 5.21-.3.34-.32.85-.06 1.22Z"></path></svg>"#;
const ICON_REFRACT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M17 5H6c-1.1 0-2 .9-2 2v5h2V7h11v3l5-4-5-4zm1 12H7v-3l-5 4 5 4v-3h11c1.1 0 2-.9 2-2v-5h-2z"></path></svg>"#;

// Social platform icons
const ICON_GITHUB: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8"/></svg>"#;
const ICON_WEBSITE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 48 48"><g fill="none" stroke="currentColor" stroke-width="3"><path stroke-linejoin="round" d="M3.539 39.743c.208 2.555 2.163 4.51 4.718 4.718C11.485 44.723 16.636 45 24 45s12.515-.277 15.743-.539c2.555-.208 4.51-2.163 4.718-4.718C44.723 36.515 45 31.364 45 24s-.277-12.515-.539-15.743c-.208-2.555-2.163-4.51-4.718-4.718C36.515 3.277 31.364 3 24 3s-12.515.277-15.743.539c-2.555.208-4.51 2.163-4.718 4.718C3.277 11.485 3 16.636 3 24s.277 12.515.539 15.743Z"/><path stroke-linecap="round" d="M3.5 13.5h41"/><path stroke-linecap="round" stroke-linejoin="round" d="M10 8.5h2m6 0h2"/></g></svg>"#;
const ICON_TWITTER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 14 14"><g fill="none"><g clip-path="url(#SVGG1Ot4cAD)"><path fill="currentColor" d="M11.025.656h2.147L8.482 6.03L14 13.344H9.68L6.294 8.909l-3.87 4.435H.275l5.016-5.75L0 .657h4.43L7.486 4.71zm-.755 11.4h1.19L3.78 1.877H2.504z"/></g><defs><clipPath id="SVGG1Ot4cAD"><path fill="currentColor" d="M0 0h14v14H0z"/></clipPath></defs></g></svg>"#;
const ICON_YOUTUBE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="m10 15l5.19-3L10 9zm11.56-7.83c.13.47.22 1.1.28 1.9c.07.8.1 1.49.1 2.09L22 12c0 2.19-.16 3.8-.44 4.83c-.25.9-.83 1.48-1.73 1.73c-.47.13-1.33.22-2.65.28c-1.3.07-2.49.1-3.59.1L12 19c-4.19 0-6.8-.16-7.83-.44c-.9-.25-1.48-.83-1.73-1.73c-.13-.47-.22-1.1-.28-1.9c-.07-.8-.1-1.49-.1-2.09L2 12c0-2.19.16-3.8.44-4.83c.25-.9.83-1.48 1.73-1.73c.47-.13 1.33-.22 2.65-.28c1.3-.07 2.49-.1 3.59-.1L12 5c4.19 0 6.8.16 7.83.44c.9.25 1.48.83 1.73 1.73"/></svg>"#;
const ICON_EMAIL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M3 7a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="m3 7l9 6l9-6"/></g></svg>"#;

// ============================================================================
// Data structures
// ============================================================================

pub struct ProfileData {
    pub user_id: i32,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub joined_at: time::OffsetDateTime,
    pub post_count: i32,
    pub question_count: i32,
    pub answer_count: i32,
    pub refract_count: i32,
    pub links: Vec<UserLink>,
    pub is_own_profile: bool,
}

pub struct UserLink {
    pub id: i32,
    pub platform: String, // "github", "website", "twitter", "youtube", "email"
    pub url: String,
}

pub struct ActivityItem {
    pub activity_type: String, // "post", "question", "answer", "refract"
    pub id: i32,
    pub title: String,
    pub content_rendered_html: String,
    pub slug: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub tags: Vec<String>,
    pub comment_count: i32,
    pub echo_count: i32,
    pub has_echoed: bool,
    pub answer_count: Option<i32>,
    pub question_title: Option<String>,
    pub question_slug: Option<String>,
    pub refract_content: Option<String>, 
    pub refract_count: Option<i32>,  
}

pub enum ProfileFeedItem {
    Activity(ActivityItem),
    Refract(RefractFeedItem),
}

// ============================================================================
// Main profile page
// ============================================================================

pub fn render_profile_page(
    profile: ProfileData,
    activities: Vec<ProfileFeedItem>,
    current_user: Option<(i32, String, Option<String>)>,
    csrf_token: Option<String>,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (profile.username) " - StringTechHub" }
                link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@500;600&family=Source+Serif+4:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=IBM+Plex+Mono:wght@400;500&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/feed.css";
                link rel="stylesheet" href="/static/search.css";
                link rel="stylesheet" href="/static/profile.css";
                link rel="stylesheet" href="/static/refract.css";
                link rel="icon" type="image/x-icon" href="/static/favicon.ico";
                link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png";
                link rel="apple-touch-icon" sizes="180x180" href="/static/apple-touch-icon.png";
                link rel="manifest" href="/static/site.webmanifest";
                
                script src="/static/htmx.min.js" {}
                script src="/static/script.js" defer {}
                script src="/static/refract.js" defer {}
                @if let Some(ref token) = csrf_token {
                    meta name="csrf-token" content=(token);
                }
            }
            body {
                (render_header(current_user.as_ref(), csrf_token.as_deref()))

                div class="container" {
                    // Profile header card
                    (render_profile_header(&profile))

                    // Activity section
                    (render_activity_section(&activities))
                }
                @if current_user.is_some() {
                    (crate::templates::refract::render_refract_modal_empty(csrf_token.as_deref()))
                }
            }
        }
    }
}

// ============================================================================
// Header (same as feed/search)
// ============================================================================

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
                                    // Logout Button
                                    button 
                                        hx-post="/auth/logout" 
                                        hx-headers=(format!(r#"{{"X-CSRF-Token": "{}"}}"#, csrf_token.unwrap_or(" ")))
                                        class="settings-item" 
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
// Profile header card
// ============================================================================

fn render_profile_header(profile: &ProfileData) -> Markup {
    let initial = profile.display_name
        .as_ref()
        .or(Some(&profile.username))
        .and_then(|n| n.chars().next())
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    html! {
        div class="profile-header" {
            div class="profile-cover" {}

            div class="profile-info" {
                @if let Some(ref url) = profile.avatar_url {
                    div class="profile-avatar" {
                        img src=(url) alt=(initial) style="width:100%;height:100%;object-fit:cover;border-radius:50%;";
                    }
                } @else {
                    div class="profile-avatar" { (initial) }
                }

                div class="profile-header-row" {
                    div class="profile-name-section" {
                        h1 { (profile.display_name.as_ref().unwrap_or(&profile.username)) }
                        div class="profile-username" { "@" (profile.username) }
                        div class="profile-joined" {
                            "Joined " (format_joined_date(&profile.joined_at))
                        }
                    }

                    @if profile.is_own_profile {
                        a href="/profile/edit" class="btn-edit-profile" { "Edit Profile" }
                    }
                }

                @if let Some(ref bio) = profile.bio {
                    div class="profile-bio" {
                        (PreEscaped(bio))
                    }
                }

                @if !profile.links.is_empty() {
                    div class="profile-links" {
                        @for link in &profile.links {
                            (render_profile_link(link))
                        }
                    }
                }

                // Stats
                div class="profile-stats" {
                    div class="stat-item" {
                        span class="stat-value" { (profile.post_count) }
                        span class="stat-label" { "Posts" }
                    }
                    div class="stat-item" {
                        span class="stat-value" { (profile.question_count) }
                        span class="stat-label" { "Questions" }
                    }
                    div class="stat-item" {
                        span class="stat-value" { (profile.answer_count) }
                        span class="stat-label" { "Answers" }
                    }
                    div class="stat-item" {
                        span class="stat-value" { (profile.refract_count) }
                        span class="stat-label" { "Refracts" }
                    }
                }
            }
        }
    }
}

fn render_profile_link(link: &UserLink) -> Markup {
    let (icon, display, href) = match link.platform.as_str() {
        "github" => (ICON_GITHUB, link.url.replace("https://", ""), link.url.clone()),
        "website" => (ICON_WEBSITE, link.url.replace("https://", "").replace("http://", ""), link.url.clone()),
        "twitter" => (ICON_TWITTER, link.url.replace("https://x.com/", "@"), link.url.clone()),
        "youtube" => (ICON_YOUTUBE, "YouTube".to_string(), link.url.clone()),
        "email" => {
            let display = link.url.replace("mailto:", "");
            let href = if link.url.starts_with("mailto:") {
                link.url.clone()
            } else {
                format!("mailto:{}", link.url)
            };
            (ICON_EMAIL, display, href)
        },
        _ => (ICON_WEBSITE, link.url.clone(), link.url.clone()),
    };

    html! {
        a href=(link.url) class="profile-link" target="_blank" rel="noopener noreferrer" {
            span { (PreEscaped(icon)) }
            (display)
        }
    }
}

// ============================================================================
// Activity section
// ============================================================================

fn render_activity_section(activities: &[ProfileFeedItem]) -> Markup {
    html! {
        div class="activity-section" {
            h2 class="section-header" { "Activity" }

            div class="feed-tabs" {
                button class="feed-tab active" { "All" }
            }

            @if activities.is_empty() {
                div class="empty-state" {
                    div class="empty-state-icon" { "📭" }
                    div class="empty-state-text" { "No activity yet" }
                }
            } @else {
                div class="activity-feed" {
                    @for item in activities {
                        @match item {
                            ProfileFeedItem::Refract(r) => (render_profile_refract_card(r)),
                            ProfileFeedItem::Activity(a) => (render_activity_item(a)),

                        }
                    }
                }
            }
        }
    }
}

pub fn render_activity_item(item: &ActivityItem) -> Markup {
    let type_badge = match item.activity_type.as_str() {
        "post" => "POST",
        "question" => "QUESTION",
        "answer" => "ANSWER",
        "refract" => "REFRACT",
        _ => "POST",
    };

    let href = match item.activity_type.as_str() {
        "question" => format!("/questions/{}", item.slug.as_ref().unwrap_or(&"".to_string())),
        "answer" => format!("/answers/{}", item.slug.as_ref().unwrap_or(&"".to_string())),
        "refract" => format!("/refracts/{}", item.id),
        _ => format!("/posts/{}", item.slug.as_ref().unwrap_or(&"".to_string())),
    };

    html! {
        article class="activity-item"
            data-slug=(item.slug.as_deref().unwrap_or(""))
            data-type=(item.activity_type) {

            div class="activity-header" { 
                span class="activity-type" { (type_badge) } 
                span class="activity-time" {
                    (crate::templates::feed::format_time(&item.created_at))
                }
            }
            a href=(href) class="activity-title" { (item.title) }
                div class="content-preview" { 
                    (PreEscaped(truncate_html(&item.content_rendered_html, 300))) 
                }
                
                @if should_show_read_more(&item.content_rendered_html, 300) {
                    a href=(href) class="read-more" { "Read more →"}
                }
            

            @if !item.tags.is_empty() {
                div class="tags" {
                    @for tag in &item.tags {
                        a href=(format!("/tags/{}", tag)) class="tag" { (tag) }
                    }
                }
            }

            div class="action-bar" {
                a href=(format!("{}#comments", href)) class="action-btn" title="Comments" {
                    (PreEscaped(ICON_COMMENT))
                    span { (item.comment_count) " comments" }
                }
                @if item.activity_type == "question" {
                    a href=(format!("{}#answers", href)) class="action-btn" title="Answers" {
                        (PreEscaped(ICON_ANSWER))
                        span { (item.answer_count.unwrap_or(0)) " answers" }
                    }
                }
                @if item.activity_type == "post" {
                    button class="action-btn refract-btn" title="Refract"
                    data-post-id=(item.id) {
                        (PreEscaped(ICON_REFRACT))
                        span { (item.refract_count.unwrap_or(0)) " refracts" }
                    }
                }
                @if item.has_echoed {
                    span class="action-btn echoed" {
                        (PreEscaped(ICON_ECHO))
                        span class="count" { (item.echo_count) }
                    }
                } @else {
                    button class="action-btn action-echo"
                        data-echo-type=(item.activity_type)
                        data-echo-id=(item.id) {
                            (PreEscaped(ICON_ECHO))
                            
                        }
                }

                @if item.activity_type == "refract" {
                    button class="action-btn copy-link-btn"
                        data-copy-link=(format!("/refracts/{}", item.id)) {
                        (PreEscaped(ICON_LINK))
                    }
                } @else {
                    button class="action-btn copy-link-btn"
                        data-copy-link=(href) {
                        (PreEscaped(ICON_LINK))
                    }
                }
            }
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn format_joined_date(date: &time::OffsetDateTime) -> String {
    date.format(&format_description!("[month repr:long] [year]"))
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn render_profile_refract_card(refract: &crate::handlers::feed::RefractFeedItem) -> Markup {
    let refract_id = refract.id;
    let original_slug = refract.original_post.slug.clone();
    html! {
        article class="activity-item" data-type="refract" {
            div class="activity-header" {
                span class="activity-type" { "REFRACT" }
                span class="activity-time" { (crate::templates::feed::format_time(&refract.created_at)) }
            }
            div class="card-content" {
                div class="content-preview" {
                    (PreEscaped(truncate_html(&refract.content_rendered_html, 300)))
                }
                @if should_show_read_more(&refract.content_rendered_html, 300) {
                    div class="content-hidden" id=(format!("refract-full-{}", refract_id)) style="display:none" {
                        (PreEscaped(&refract.content_rendered_html))
                    }
                    button class="expand-btn" onclick=(format!("toggleRefractExpand({})", refract_id)) {
                        span class="expand-text" { "See more" }
                    }
                }
            }
            @if !refract.original_post.is_deleted {
                div class="embedded-post" data-href=(format!("/posts/{}", original_slug)) {
                    @if let Some(ref title) = refract.original_post.title {
                        h3 class="embedded-title" { (title) }
                    }
                    div class="embedded-preview" {
                        (PreEscaped(truncate_html(&refract.original_post.content_rendered_html, 200)))
                    }
                }
            }
            div class="action-bar" {
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
                            span class="count" { (refract.echo_count) }
                        }
                }
                button class="action-btn copy-link-btn"
                    data-copy-link=(format!("/refracts/{}", refract.id)) {
                    (PreEscaped(ICON_LINK))
                }
            }
        }
    }
}