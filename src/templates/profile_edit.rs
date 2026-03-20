use maud::{html, Markup, PreEscaped, DOCTYPE};

const ICON_SEARCH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M18 10c0-4.41-3.59-8-8-8s-8 3.59-8 8 3.59 8 8 8c1.85 0 3.54-.63 4.9-1.69l5.1 5.1L21.41 20l-5.1-5.1A8 8 0 0 0 18 10M4 10c0-3.31 2.69-6 6-6s6 2.69 6 6-2.69 6-6 6-6-2.69-6-6"></path></svg>"#;
const ICON_POST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="m17.71 7.29-3-3a.996.996 0 0 0-1.41 0l-11.01 11A1 1 0 0 0 2 16v3c0 .55.45 1 1 1h3c.27 0 .52-.11.71-.29l11-11a.996.996 0 0 0 0-1.41ZM5.59 18H4v-1.59l7.5-7.5 1.59 1.59zm8.91-8.91L12.91 7.5 14 6.41 15.59 8zM11 18h11v2H11z"></path></svg>"#;
const ICON_QUESTION: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M12 2C6.49 2 2 6.49 2 12s4.49 10 10 10 10-4.49 10-10S17.51 2 12 2m0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8"></path><path d="M11 16h2v2h-2zm2.27-9.75c-2.08-.75-4.47.35-5.21 2.41l1.88.68c.18-.5.56-.9 1.07-1.13s1.08-.26 1.58-.08a2.01 2.01 0 0 1 1.32 1.86c0 1.04-1.66 1.86-2.24 2.07-.4.14-.67.52-.67.94v1h2v-.34c1.04-.51 2.91-1.69 2.91-3.68a4.015 4.015 0 0 0-2.64-3.73"></path></svg>"#;
const ICON_PROFILE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M12 7c-2 0-3.5 1.5-3.5 3.5S10 14 12 14s3.5-1.5 3.5-3.5S14 7 12 7m0 5c-.88 0-1.5-.62-1.5-1.5S11.12 9 12 9s1.5.62 1.5 1.5S12.88 12 12 12"></path><path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2M8.18 19c.41-1.16 1.51-2 2.82-2h2c1.3 0 2.4.84 2.82 2H8.19Zm9.71 0a5 5 0 0 0-4.9-4h-2c-2.41 0-4.43 1.72-4.9 4h-1.1V5h14v14z"></path></svg>"#;
const ICON_SETTINGS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M12 8c-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4-1.79-4-4-4m0 6c-1.08 0-2-.92-2-2s.92-2 2-2 2 .92 2 2-.92 2-2 2"></path><path d="m20.42 13.4-.51-.29c.05-.37.08-.74.08-1.11s-.03-.74-.08-1.11l.51-.29c.96-.55 1.28-1.78.73-2.73l-1-1.73a2.006 2.006 0 0 0-2.73-.73l-.53.31c-.58-.46-1.22-.83-1.9-1.11v-.6c0-1.1-.9-2-2-2h-2c-1.1 0-2 .9-2 2v.6c-.67.28-1.31.66-1.9 1.11l-.53-.31c-.96-.55-2.18-.22-2.73.73l-1 1.73c-.55.96-.22 2.18.73 2.73l.51.29c-.05.37-.08.74-.08 1.11s.03.74.08 1.11l-.51.29c-.96.55-1.28 1.78-.73 2.73l1 1.73c.55.95 1.77 1.28 2.73.73l.53-.31c.58.46 1.22.83 1.9 1.11v.6c0 1.1.9 2 2 2h2c1.1 0 2-.9 2-2v-.6a8.7 8.7 0 0 0 1.9-1.11l.53.31c.95.55 2.18.22 2.73-.73l1-1.73c.55-.96.22-2.18-.73-2.73m-2.59-2.78c.11.45.17.92.17 1.38s-.06.92-.17 1.38a1 1 0 0 0 .47 1.11l1.12.65-1 1.73-1.14-.66c-.38-.22-.87-.16-1.19.14-.68.65-1.51 1.13-2.38 1.4-.42.13-.71.52-.71.96v1.3h-2v-1.3c0-.44-.29-.83-.71-.96-.88-.27-1.7-.75-2.38-1.4a1.01 1.01 0 0 0-1.19-.15l-1.14.66-1-1.73 1.12-.65c.39-.22.58-.68.47-1.11-.11-.45-.17-.92-.17-1.38s.06-.93.17-1.38A1 1 0 0 0 5.7 9.5l-1.12-.65 1-1.73 1.14.66c.38.22.87.16 1.19-.14.68-.65 1.51-1.13 2.38-1.4.42-.13.71-.52.71-.96v-1.3h2v1.3c0 .44.29.83.71.96.88.27 1.7.75 2.38 1.4.32.31.81.36 1.19.14l1.14-.66 1 1.73-1.12.65c-.39.22-.58.68-.47 1.11Z"></path></svg>"#;
const ICON_LOGOUT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M9 13h7v-2H9V7l-6 5 6 5z"></path><path d="M19 3h-7v2h7v14h-7v2h7c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2"></path></svg>"#;
const ICON_DELETE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M17 6V4c0-1.1-.9-2-2-2H9c-1.1 0-2 .9-2 2v2H2v2h2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8h2V6zM9 4h6v2H9zM6 20V8h12v12z"></path><path d="M9 10h2v8H9zm4 0h2v8h-2z"></path></svg>"#;
const ICON_WARNING: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-miterlimit="10" stroke-width="1.5" d="M12 16h.008M12 8v5m10-1c0-5.523-4.477-10-10-10S2 6.477 2 12s4.477 10 10 10s10-4.477 10-10"/></svg>"#;
const ICON_GITHUB: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M12 2A10 10 0 0 0 2 12c0 4.42 2.87 8.17 6.84 9.5c.5.08.66-.23.66-.5v-1.69c-2.77.6-3.36-1.34-3.36-1.34c-.46-1.16-1.11-1.47-1.11-1.47c-.91-.62.07-.6.07-.6c1 .07 1.53 1.03 1.53 1.03c.87 1.52 2.34 1.07 2.91.83c.09-.65.35-1.09.63-1.34c-2.22-.25-4.55-1.11-4.55-4.92c0-1.11.38-2 1.03-2.71c-.1-.25-.45-1.29.1-2.64c0 0 .84-.27 2.75 1.02c.79-.22 1.65-.33 2.5-.33s1.71.11 2.5.33c1.91-1.29 2.75-1.02 2.75-1.02c.55 1.35.2 2.39.1 2.64c.65.71 1.03 1.6 1.03 2.71c0 3.82-2.34 4.66-4.57 4.91c.36.31.69.92.69 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12A10 10 0 0 0 12 2"/></svg>"#;
const ICON_WEBSITE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 48 48"><g fill="none" stroke="currentColor" stroke-width="3"><path stroke-linejoin="round" d="M3.539 39.743c.208 2.555 2.163 4.51 4.718 4.718C11.485 44.723 16.636 45 24 45s12.515-.277 15.743-.539c2.555-.208 4.51-2.163 4.718-4.718C44.723 36.515 45 31.364 45 24s-.277-12.515-.539-15.743c-.208-2.555-2.163-4.51-4.718-4.718C36.515 3.277 31.364 3 24 3s-12.515.277-15.743.539c-2.555.208-4.51 2.163-4.718 4.718C3.277 11.485 3 16.636 3 24s.277 12.515.539 15.743Z"/><path stroke-linecap="round" d="M3.5 13.5h41"/><path stroke-linecap="round" stroke-linejoin="round" d="M10 8.5h2m6 0h2"/></g></svg>"#;
const ICON_TWITTER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 14 14"><g fill="none"><g clip-path="url(#SVGG1Ot4cAD)"><path fill="currentColor" d="M11.025.656h2.147L8.482 6.03L14 13.344H9.68L6.294 8.909l-3.87 4.435H.275l5.016-5.75L0 .657h4.43L7.486 4.71zm-.755 11.4h1.19L3.78 1.877H2.504z"/></g><defs><clipPath id="SVGG1Ot4cAD"><path fill="currentColor" d="M0 0h14v14H0z"/></clipPath></defs></g></svg>"#;
const ICON_YOUTUBE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="m10 15l5.19-3L10 9zm11.56-7.83c.13.47.22 1.1.28 1.9c.07.8.1 1.49.1 2.09L22 12c0 2.19-.16 3.8-.44 4.83c-.25.9-.83 1.48-1.73 1.73c-.47.13-1.33.22-2.65.28c-1.3.07-2.49.1-3.59.1L12 19c-4.19 0-6.8-.16-7.83-.44c-.9-.25-1.48-.83-1.73-1.73c-.13-.47-.22-1.1-.28-1.9c-.07-.8-.1-1.49-.1-2.09L2 12c0-2.19.16-3.8.44-4.83c.25-.9.83-1.48 1.73-1.73c.47-.13 1.33-.22 2.65-.28c1.3-.07 2.49-.1 3.59-.1L12 5c4.19 0 6.8.16 7.83.44c.9.25 1.48.83 1.73 1.73"/></svg>"#;
const ICON_EMAIL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M3 7a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="m3 7l9 6l9-6"/></g></svg>"#;
const ICON_BOLD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 80 80"><path fill="currentColor" d="M24 41h17v-6H24zm17 0h2v-6h-2zm0-28H24.5v6H41zm2 48H24.5v6H43zM21 16.5V38h6V16.5zM21 38v25.5h6V38zm20 3c7.732 0 14-6.268 14-14h-6a8 8 0 0 1-8 8zm0-22a8 8 0 0 1 8 8h6c0-7.732-6.268-14-14-14zm2 22c5.523 0 10 4.477 10 10h6c0-8.837-7.163-16-16-16zM24.5 61a2.5 2.5 0 0 1 2.5 2.5h-6a3.5 3.5 0 0 0 3.5 3.5zM53 51c0 5.523-4.477 10-10 10v6c8.837 0 16-7.163 16-16zM24.5 13a3.5 3.5 0 0 0-3.5 3.5h6a2.5 2.5 0 0 1-2.5 2.5z"/></svg>"#;
const ICON_ITALIC: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5h6M7 19h6m1-14l-4 14"/></svg>"#;
const ICON_CODE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m8 8l-4 4l4 4m8 0l4-4l-4-4m-2-3l-4 14"/></svg>"#;
const ICON_MD_LINK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-dasharray="28" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 6l2 -2c1 -1 3 -1 4 0l1 1c1 1 1 3 0 4l-5 5c-1 1 -3 1 -4 0M11 18l-2 2c-1 1 -3 1 -4 0l-1 -1c-1 -1 -1 -3 0 -4l5 -5c1 -1 3 -1 4 0"><animate fill="freeze" attributeName="stroke-dashoffset" dur="0.6s" values="28;0"/></path></svg>"#;
const ICON_UL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M5.94 6.42H24v1.75H5.94zm0 5.29H24v1.75H5.94zm0 5.28H24v1.75H5.94z"/><circle cx="1.85" cy="7.29" r="1.52" fill="currentColor"/><circle cx="1.85" cy="12.58" r="1.52" fill="currentColor"/><circle cx="1.85" cy="17.87" r="1.52" fill="currentColor"/></svg>"#;
const ICON_OL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><path fill="currentColor" d="M3.59 3.03h12.2v1.26H3.59zm0 4.29h12.2v1.26H3.59zm0 4.35h12.2v1.26H3.59zM.99 4.79h.49V2.52H.6v.45h.39zm.87 3.88H.91l.14-.11l.3-.24c.35-.28.49-.5.49-.79A.74.74 0 0 0 1 6.8a.77.77 0 0 0-.81.84h.52A.34.34 0 0 1 1 7.25a.31.31 0 0 1 .31.31a.6.6 0 0 1-.22.44l-.87.75v.39h1.64zm-.36 3.56a.52.52 0 0 0 .28-.48a.67.67 0 0 0-.78-.62a.71.71 0 0 0-.77.75h.5a.3.3 0 0 1 .27-.32a.26.26 0 1 1 0 .51H.91v.38H1c.23 0 .37.11.37.29a.29.29 0 0 1-.33.29a.35.35 0 0 1-.36-.35H.21a.76.76 0 0 0 .83.8a.74.74 0 0 0 .83-.72a.53.53 0 0 0-.37-.53"/></svg>"#;
const ICON_QUOTE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="0.88em" height="1em" viewBox="0 0 448 512"><path fill="currentColor" d="M0 216C0 149.7 53.7 96 120 96h8c17.7 0 32 14.3 32 32s-14.3 32-32 32h-8c-30.9 0-56 25.1-56 56v8h64c35.3 0 64 28.7 64 64v64c0 35.3-28.7 64-64 64H64c-35.3 0-64-28.7-64-64zm256 0c0-66.3 53.7-120 120-120h8c17.7 0 32 14.3 32 32s-14.3 32-32 32h-8c-30.9 0-56 25.1-56 56v8h64c35.3 0 64 28.7 64 64v64c0 35.3-28.7 64-64 64h-64c-35.3 0-64-28.7-64-64z"/></svg>"#;

// ============================================================================
// Data structures
// ============================================================================

pub struct EditProfileData {
    pub user_id: i32,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub links: Vec<EditUserLink>,
}

pub struct EditUserLink {
    pub id: Option<i32>,
    pub platform: String,
    pub url: Option<String>,
}

// ============================================================================
// Main edit profile page
// ============================================================================

pub fn render_edit_profile_page(
    data: EditProfileData,
    csrf_token: String,
    current_user: (i32, String, Option<String>),
    username_error: Option<String>,
) -> Markup {
    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Edit Profile - StringTechHub" }
                link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@500;600&family=Source+Serif+4:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=IBM+Plex+Mono:wght@400;500&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/feed.css";
                link rel="stylesheet" href="/static/search.css";
                link rel="stylesheet" href="/static/profile.css";
                link rel="icon" type="image/x-icon" href="/static/favicon.ico";
                link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png";
                link rel="apple-touch-icon" sizes="180x180" href="/static/apple-touch-icon.png";
                link rel="manifest" href="/static/site.webmanifest";
                
                script src="/static/htmx.min.js" {}
                script src="/static/script.js" defer {}
                script src="/static/marked.min.js" {}
            }
            body {
                (render_header(Some(&current_user), &csrf_token))

                div class="edit-profile-page" {
                    div class="container" {
                        div class="page-header" {
                            h1 class="page-title" { "Edit Profile" }
                            p class="page-subtitle" { "Update your personal information and links" }
                        }

                        // Shared feedback div — all forms target this
                        div id="profile-feedback" {}

                        (render_basic_info_section(&data, &csrf_token))
                        (render_links_section(&data, &csrf_token))
                        (render_username_section(&data, &csrf_token, username_error))
                    }
                }
            }
        }
    }
}

// ============================================================================
// Header
// ============================================================================

fn render_header(user: Option<&(i32, String, Option<String>)>, csrf_token: &str) -> Markup {
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
                        input type="hidden" name="q" id="search-query-hidden" value="";
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
                            button class="btn-settings" id="settings-btn"
                                aria-haspopup="true" aria-expanded="false" {
                                (PreEscaped(ICON_SETTINGS))
                            }
                            div class="settings-menu" role="menu" {
                                button
                                    hx-post="/auth/logout"
                                    hx-headers=(format!(r#"{{"X-CSRF-Token": "{}"}}"#, csrf_token))
                                    class="settings-item"
                                {
                                    (PreEscaped(ICON_LOGOUT))
                                    "Logout"
                                }
                                button
                                    hx-delete="/auth/delete-account"
                                    hx-headers=(format!(r#"{{"X-CSRF-Token": "{}"}}"#, csrf_token))
                                    class="settings-item danger"
                                {
                                    (PreEscaped(ICON_DELETE))
                                    "Delete account"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Basic info section
// ============================================================================

fn render_basic_info_section(data: &EditProfileData, csrf_token: &str) -> Markup {
    let initial = data.display_name
        .as_ref()
        .or(Some(&data.username))
        .and_then(|n| n.chars().next())
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    html! {
        // hx-patch sends as form-encoded, handler must use Form extractor
        form id="basic-info-form" data-csrf=(csrf_token) class="form-section"
            
        {
            h2 class="form-section-title" { "Basic Information" }

            div class="form-group" {
                label class="form-label" { "Profile Picture" }
                div class="avatar-upload" {
                    div class="avatar-preview" { 
                        @if let Some(ref url) = data.avatar_url {
                            img src=(url) alt="Avatar" style="width:100%;height:100%;object-fit:cover;border-radius:50%;";
                        } @else {
                            div class="avatar-preview" { (initial) }
                        }
                    }                    
                    p class="form-help" { "Profile picture is sourced from your GitHub account." }                    
                }
            }

            div class="form-group" {
                label class="form-label" { "Display Name" }
                // Field name must match UpdateProfileRequest.name
                input type="text" class="form-input" name="name"
                    value=(data.display_name.as_ref().unwrap_or(&String::new()))
                    placeholder="Your display name";
                p class="form-help" { "Your public display name (max 100 characters)" }
            }

            div class="form-group" {
                label class="form-label" { "Bio" }
                (render_bio_input(data.bio.as_deref().unwrap_or("")))
                p class="form-help" { "Markdown supported. Max 1000 characters." }
            }

            div class="form-actions" {
                a href=(format!("/@{}", data.username)) class="btn btn-secondary" { "Cancel" }
                button type="submit" class="btn btn-primary" { "Save Changes" }
            }
        }
    }
}

fn render_bio_input(bio: &str) -> Markup {
    html! {
        div class="bio-input-container" id="bio-input" {
            // Field name must match UpdateProfileRequest.bio
            textarea
                class="bio-textarea"
                id="bio-textarea"
                name="bio"
                placeholder="Tell us about yourself..."
                maxlength="1000"
            { (bio) }

            div class="bio-toolbar" {
                div class="toolbar-row" {
                    div class="markdown-buttons" {
                        button type="button" class="markdown-btn" data-action="bold" title="Bold"
                            { (PreEscaped(ICON_BOLD)) }
                        button type="button" class="markdown-btn" data-action="italic" title="Italic"
                            { (PreEscaped(ICON_ITALIC)) }
                        button type="button" class="markdown-btn" data-action="code" title="Code"
                            { (PreEscaped(ICON_CODE)) }
                        button type="button" class="markdown-btn" data-action="link" title="Link"
                            { (PreEscaped(ICON_MD_LINK)) }
                        button type="button" class="markdown-btn" data-action="bullet-list" title="Bullet List"
                            { (PreEscaped(ICON_UL)) }
                        button type="button" class="markdown-btn" data-action="numbered-list" title="Numbered List"
                            { (PreEscaped(ICON_OL)) }
                        button type="button" class="markdown-btn" data-action="quote" title="Quote"
                            { (PreEscaped(ICON_QUOTE)) }
                    }
                    div class="bio-actions" {
                        span class="char-counter" id="bio-char-counter" {
                            (bio.len()) " / 1000"
                        }
                        button type="button" class="btn-preview" id="bio-preview-toggle"
                            { "Live Preview" }
                    }
                }
            }
        }

        div class="preview-overlay" id="preview-overlay" {}
        div class="preview-panel" id="bio-preview-panel" {
            div class="preview-header" { "Preview" }
            div class="preview-content" id="bio-preview-content" {
                p { "Nothing to preview yet..." }
            }
        }
    }
}

// ============================================================================
// Links section — each platform is its own HTMX form
// ============================================================================

fn render_links_section(data: &EditProfileData, csrf_token: &str) -> Markup {
    let mut link_map = std::collections::HashMap::new();
    for link in &data.links {
        link_map.insert(link.platform.clone(), link.url.clone());
    }

    let platforms = vec![
        ("github",  "GitHub",  ICON_GITHUB),
        ("website", "Website", ICON_WEBSITE),
        ("twitter", "Twitter", ICON_TWITTER),
        ("youtube", "YouTube", ICON_YOUTUBE),
        ("email",   "Email",   ICON_EMAIL),
    ];

    html! {
        form id="links-form" data-csrf=(csrf_token) class="form-section" {
            h2 class="form-section-title" { "Social Links" }
            div class="links-list" {
                @for (platform, display, icon) in &platforms {
                    div class="link-item" {
                        span class="link-platform" {
                            (PreEscaped(icon))
                            " " (display)
                        }
                        input
                            type="text"
                            class="link-url-input"
                            name=(format!("link_{}", platform))
                            value=(link_map.get(*platform).and_then(|u| u.as_deref()).unwrap_or(""))
                            placeholder=(format!("https://{}.com/username", platform));
                    }
                }
            }
            div class="form-actions" {
                a href=(format!("/@{}", data.username)) class="btn btn-secondary" { "Cancel" }
                button type="submit" class="btn btn-primary" { "Save Changes" }
            }
        }  // ← form closes here, AFTER form-actions
    }
}


// ============================================================================
// Username change section
// ============================================================================

fn render_username_section(
    data: &EditProfileData,
    csrf_token: &str,
    username_error: Option<String>,
) -> Markup {
    html! {
        form id="username-form" data-csrf=(csrf_token) class="form-section"
        
        {
            h2 class="form-section-title" { "Change Username" }

            div class="warning-box" {
                div class="warning-title" {
                    (PreEscaped(ICON_WARNING))
                    " Important"
                }
                div class="warning-text" {
                    "You can only change your username once every 30 days. Your old username will become available for others to use."
                }
            }

            div class="form-group" {
                label class="form-label" { "Username" }
                // Field name must match UpdateUsernameRequest.username
                input type="text" class="form-input" name="username"
                    value=(data.username) placeholder="New username";
                p class="form-help" { "3-30 characters. Letters, numbers, _ and - only." }

                @if let Some(error) = username_error {
                    p class="form-error" { (error) }
                }
            }

            div class="form-actions" {
                a href=(format!("/@{}", data.username)) class="btn btn-secondary" { "Cancel" }
                button type="submit" class="btn btn-primary" { "Update Username" }
            }
        }
    }
}