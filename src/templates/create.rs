use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::middleware::csrf;

// SVG icon constants - you'll fill these
const ICON_SEARCH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M18 10c0-4.41-3.59-8-8-8s-8 3.59-8 8 3.59 8 8 8c1.85 0 3.54-.63 4.9-1.69l5.1 5.1L21.41 20l-5.1-5.1A8 8 0 0 0 18 10M4 10c0-3.31 2.69-6 6-6s6 2.69 6 6-2.69 6-6 6-6-2.69-6-6"></path></svg>"#;
const ICON_POST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="m17.71 7.29-3-3a.996.996 0 0 0-1.41 0l-11.01 11A1 1 0 0 0 2 16v3c0 .55.45 1 1 1h3c.27 0 .52-.11.71-.29l11-11a.996.996 0 0 0 0-1.41ZM5.59 18H4v-1.59l7.5-7.5 1.59 1.59zm8.91-8.91L12.91 7.5 14 6.41 15.59 8zM11 18h11v2H11z"></path></svg>"#;
const ICON_QUESTION: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M12 2C6.49 2 2 6.49 2 12s4.49 10 10 10 10-4.49 10-10S17.51 2 12 2m0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8"></path><path d="M11 16h2v2h-2zm2.27-9.75c-2.08-.75-4.47.35-5.21 2.41l1.88.68c.18-.5.56-.9 1.07-1.13s1.08-.26 1.58-.08a2.01 2.01 0 0 1 1.32 1.86c0 1.04-1.66 1.86-2.24 2.07-.4.14-.67.52-.67.94v1h2v-.34c1.04-.51 2.91-1.69 2.91-3.68a4.015 4.015 0 0 0-2.64-3.73"></path></svg>"#;
const ICON_PROFILE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M12 7c-2 0-3.5 1.5-3.5 3.5S10 14 12 14s3.5-1.5 3.5-3.5S14 7 12 7m0 5c-.88 0-1.5-.62-1.5-1.5S11.12 9 12 9s1.5.62 1.5 1.5S12.88 12 12 12"></path><path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2M8.18 19c.41-1.16 1.51-2 2.82-2h2c1.3 0 2.4.84 2.82 2H8.19Zm9.71 0a5 5 0 0 0-4.9-4h-2c-2.41 0-4.43 1.72-4.9 4h-1.1V5h14v14z"></path></svg>"#;
const ICON_SETTINGS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M12 8c-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4-1.79-4-4-4m0 6c-1.08 0-2-.92-2-2s.92-2 2-2 2 .92 2 2-.92 2-2 2"></path><path d="m20.42 13.4-.51-.29c.05-.37.08-.74.08-1.11s-.03-.74-.08-1.11l.51-.29c.96-.55 1.28-1.78.73-2.73l-1-1.73a2.006 2.006 0 0 0-2.73-.73l-.53.31c-.58-.46-1.22-.83-1.9-1.11v-.6c0-1.1-.9-2-2-2h-2c-1.1 0-2 .9-2 2v.6c-.67.28-1.31.66-1.9 1.11l-.53-.31c-.96-.55-2.18-.22-2.73.73l-1 1.73c-.55.96-.22 2.18.73 2.73l.51.29c-.05.37-.08.74-.08 1.11s.03.74.08 1.11l-.51.29c-.96.55-1.28 1.78-.73 2.73l1 1.73c.55.95 1.77 1.28 2.73.73l.53-.31c.58.46 1.22.83 1.9 1.11v.6c0 1.1.9 2 2 2h2c1.1 0 2-.9 2-2v-.6a8.7 8.7 0 0 0 1.9-1.11l.53.31c.95.55 2.18.22 2.73-.73l1-1.73c.55-.96.22-2.18-.73-2.73m-2.59-2.78c.11.45.17.92.17 1.38s-.06.92-.17 1.38a1 1 0 0 0 .47 1.11l1.12.65-1 1.73-1.14-.66c-.38-.22-.87-.16-1.19.14-.68.65-1.51 1.13-2.38 1.4-.42.13-.71.52-.71.96v1.3h-2v-1.3c0-.44-.29-.83-.71-.96-.88-.27-1.7-.75-2.38-1.4a1.01 1.01 0 0 0-1.19-.15l-1.14.66-1-1.73 1.12-.65c.39-.22.58-.68.47-1.11-.11-.45-.17-.92-.17-1.38s.06-.93.17-1.38A1 1 0 0 0 5.7 9.5l-1.12-.65 1-1.73 1.14.66c.38.22.87.16 1.19-.14.68-.65 1.51-1.13 2.38-1.4.42-.13.71-.52.71-.96v-1.3h2v1.3c0 .44.29.83.71.96.88.27 1.7.75 2.38 1.4.32.31.81.36 1.19.14l1.14-.66 1 1.73-1.12.65c-.39.22-.58.68-.47 1.11Z"></path></svg>"#;
const ICON_LOGOUT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M9 13h7v-2H9V7l-6 5 6 5z"></path><path d="M19 3h-7v2h7v14h-7v2h7c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2"></path></svg>"#;
const ICON_DELETE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M17 6V4c0-1.1-.9-2-2-2H9c-1.1 0-2 .9-2 2v2H2v2h2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8h2V6zM9 4h6v2H9zM6 20V8h12v12z"></path><path d="M9 10h2v8H9zm4 0h2v8h-2z"></path></svg>"#;
const ICON_HEADING: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">heading</title><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 12h10M7 5v14M17 5v14m-2 0h4M15 5h4M5 19h4M5 5h4"/></svg>"#;
const ICON_BOLD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 80 80"><title xmlns="">b-bold</title><path fill="currentColor" d="M24 41h17v-6H24zm17 0h2v-6h-2zm0-28H24.5v6H41zm2 48H24.5v6H43zM21 16.5V38h6V16.5zM21 38v25.5h6V38zm20 3c7.732 0 14-6.268 14-14h-6a8 8 0 0 1-8 8zm0-22a8 8 0 0 1 8 8h6c0-7.732-6.268-14-14-14zm2 22c5.523 0 10 4.477 10 10h6c0-8.837-7.163-16-16-16zM24.5 61a2.5 2.5 0 0 1 2.5 2.5h-6a3.5 3.5 0 0 0 3.5 3.5zM53 51c0 5.523-4.477 10-10 10v6c8.837 0 16-7.163 16-16zM24.5 13a3.5 3.5 0 0 0-3.5 3.5h6a2.5 2.5 0 0 1-2.5 2.5z"/></svg>"#;
const ICON_ITALIC: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">italic</title><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5h6M7 19h6m1-14l-4 14"/></svg>"#;
const ICON_STRIKETHROUGH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><title xmlns="">editor-strike</title><path fill="currentColor" d="M10.023 10h1.274q.01.12.01.25a2.56 2.56 0 0 1-.883 1.949q-.426.375-1.03.588A4.1 4.1 0 0 1 8.028 13a4.62 4.62 0 0 1-3.382-1.426q-.29-.388 0-.724q.29-.334.735-.13q.515.544 1.213.876q.699.33 1.449.33q.956 0 1.485-.433q.53-.435.53-1.14a1.7 1.7 0 0 0-.034-.353M5.586 7a2.5 2.5 0 0 1-.294-.507a2.3 2.3 0 0 1-.177-.934q0-.544.228-1.015t.633-.816t.955-.537A3.7 3.7 0 0 1 8.145 3q.867 0 1.603.33q.735.332 1.25.861q.24.423 0 .692q-.24.27-.662.102a3.4 3.4 0 0 0-.978-.669a2.9 2.9 0 0 0-1.213-.242q-.81 0-1.302.375t-.492 1.036q0 .354.14.596q.138.243.374.426q.236.184.515.324q.179.09.362.169zM2.5 8h11a.5.5 0 1 1 0 1h-11a.5.5 0 0 1 0-1"/></svg>"#;
const ICON_INLINE_CODE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">code-bold</title><path fill="currentColor" d="M14.18 4.276a.75.75 0 0 1 .531.918l-3.973 14.83a.75.75 0 0 1-1.45-.389l3.974-14.83a.75.75 0 0 1 .919-.53m2.262 3.053a.75.75 0 0 1 1.059-.056l1.737 1.564c.737.662 1.347 1.212 1.767 1.71c.44.525.754 1.088.754 1.784c0 .695-.313 1.258-.754 1.782c-.42.499-1.03 1.049-1.767 1.711l-1.737 1.564a.75.75 0 0 1-1.004-1.115l1.697-1.527c.788-.709 1.319-1.19 1.663-1.598c.33-.393.402-.622.402-.818s-.072-.424-.402-.817c-.344-.409-.875-.89-1.663-1.598l-1.697-1.527a.75.75 0 0 1-.056-1.06m-8.94 1.06a.75.75 0 1 0-1.004-1.115L4.761 8.836c-.737.662-1.347 1.212-1.767 1.71c-.44.525-.754 1.088-.754 1.784c0 .695.313 1.258.754 1.782c.42.499 1.03 1.049 1.767 1.711l1.737 1.564a.75.75 0 0 0 1.004-1.115l-1.697-1.527c-.788-.709-1.319-1.19-1.663-1.598c-.33-.393-.402-.622-.402-.818s.072-.424.402-.817c.344-.409.875-.89 1.663-1.598z"/></svg>"#;
const ICON_CODE_BLOCK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">code-blocks-outline</title><path fill="currentColor" d="m9.6 14.908l.708-.714L8.114 12l2.174-2.175l-.707-.713L6.692 12zm4.8 0L17.308 12L14.4 9.092l-.708.714L15.887 12l-2.195 2.194zM5.616 20q-.691 0-1.153-.462T4 18.384V5.616q0-.691.463-1.153T5.616 4h12.769q.69 0 1.153.463T20 5.616v12.769q0 .69-.462 1.153T18.384 20zm0-1h12.769q.23 0 .423-.192t.192-.424V5.616q0-.231-.192-.424T18.384 5H5.616q-.231 0-.424.192T5 5.616v12.769q0 .23.192.423t.423.192M5 5v14z"/></svg>"#;
const ICON_LINK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">link</title><path fill="none" stroke="currentColor" stroke-dasharray="28" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 6l2 -2c1 -1 3 -1 4 0l1 1c1 1 1 3 0 4l-5 5c-1 1 -3 1 -4 0M11 18l-2 2c-1 1 -3 1 -4 0l-1 -1c-1 -1 -1 -3 0 -4l5 -5c1 -1 3 -1 4 0"><animate fill="freeze" attributeName="stroke-dashoffset" dur="0.6s" values="28;0"/></path></svg>"#;
const ICON_IMAGE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">image</title><path fill="currentColor" d="M5 3h13a3 3 0 0 1 3 3v13a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3m0 1a2 2 0 0 0-2 2v11.59l4.29-4.3l2.5 2.5l5-5L20 16V6a2 2 0 0 0-2-2zm4.79 13.21l-2.5-2.5L3 19a2 2 0 0 0 2 2h13a2 2 0 0 0 2-2v-1.59l-5.21-5.2zM7.5 6A2.5 2.5 0 0 1 10 8.5A2.5 2.5 0 0 1 7.5 11A2.5 2.5 0 0 1 5 8.5A2.5 2.5 0 0 1 7.5 6m0 1A1.5 1.5 0 0 0 6 8.5A1.5 1.5 0 0 0 7.5 10A1.5 1.5 0 0 0 9 8.5A1.5 1.5 0 0 0 7.5 7"/></svg>"#;
const ICON_BULLET_LIST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">list-bullet</title><path fill="currentColor" fill-rule="evenodd" d="M10 4h10a1 1 0 0 1 0 2H10a1 1 0 1 1 0-2m0 7h10a1 1 0 0 1 0 2H10a1 1 0 0 1 0-2m0 7h10a1 1 0 0 1 0 2H10a1 1 0 0 1 0-2M5 7a2 2 0 1 1 0-4a2 2 0 0 1 0 4m0 7a2 2 0 1 1 0-4a2 2 0 0 1 0 4m0 7a2 2 0 1 1 0-4a2 2 0 0 1 0 4"/></svg>"#;
const ICON_NUMBERED_LIST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><title xmlns="">ordered-list</title><path fill="currentColor" d="M3.59 3.03h12.2v1.26H3.59zm0 4.29h12.2v1.26H3.59zm0 4.35h12.2v1.26H3.59zM.99 4.79h.49V2.52H.6v.45h.39zm.87 3.88H.91l.14-.11l.3-.24c.35-.28.49-.5.49-.79A.74.74 0 0 0 1 6.8a.77.77 0 0 0-.81.84h.52A.34.34 0 0 1 1 7.25a.31.31 0 0 1 .31.31a.6.6 0 0 1-.22.44l-.87.75v.39h1.64zm-.36 3.56a.52.52 0 0 0 .28-.48a.67.67 0 0 0-.78-.62a.71.71 0 0 0-.77.75h.5a.3.3 0 0 1 .27-.32a.26.26 0 1 1 0 .51H.91v.38H1c.23 0 .37.11.37.29a.29.29 0 0 1-.33.29a.35.35 0 0 1-.36-.35H.21a.76.76 0 0 0 .83.8a.74.74 0 0 0 .83-.72a.53.53 0 0 0-.37-.53"/></svg>"#;
const ICON_QUOTE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">quote-left-filled</title><path fill="currentColor" d="M21.01 10h-2.85c.27-1.02 1.01-2.51 3.09-3.03l.76-.19V4h-1c-2.78 0-4.91.77-6.31 2.29c-1.89 2.05-1.7 4.68-1.69 4.71v7c0 1.1.9 2 2 2h6c1.1 0 2-.9 2-2v-6c0-1.1-.9-2-2-2m-12 0H6.16c.27-1.02 1.01-2.51 3.09-3.03l.76-.19V4h-1C6.23 4 4.1 4.77 2.7 6.29C.81 8.34 1 10.97 1.01 11v7c0 1.1.9 2 2 2h6c1.1 0 2-.9 2-2v-6c0-1.1-.9-2-2-2"/></svg>"#;
const ICON_TABLE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">table</title><path fill="currentColor" d="M6 5h11a3 3 0 0 1 3 3v9a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V8a3 3 0 0 1 3-3M4 17a2 2 0 0 0 2 2h5v-3H4zm7-5H4v3h7zm6 7a2 2 0 0 0 2-2v-1h-7v3zm2-7h-7v3h7zM4 11h7V8H4zm8 0h7V8h-7z"/></svg>"#;
const ICON_HORIZONTAL_RULE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><title xmlns="">horizontal-rule</title><path fill="currentColor" d="M14.5 13.5a.5.5 0 0 1-.5.5H2a.5.5 0 0 1 0-1h12a.5.5 0 0 1 .5.5M2.5 11a.5.5 0 0 0 .5-.5v-3h3v3a.5.5 0 0 0 1 0v-7a.5.5 0 0 0-1 0v3H3v-3a.5.5 0 0 0-1 0v7a.5.5 0 0 0 .5.5m6.5-.5v-7a.5.5 0 0 1 .5-.5h2.25C12.99 3 14 4.009 14 5.25c0 .892-.526 1.657-1.28 2.021c.606.664.901 1.609 1.091 2.236c.058.189.132.437.186.558a.5.5 0 0 1-.247.935c-.531 0-.692-.531-.896-1.203C12.487 8.587 12.069 7.5 11 7.5h-1v3a.5.5 0 0 1-1 0m1-4h1.75c.689 0 1.25-.561 1.25-1.25S12.439 4 11.75 4H10z"/></svg>"#;
const ICON_FOOTNOTE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 32 32"><title xmlns="">text-footnote</title><path fill="currentColor" d="M2 7v2h7v16h2V9h7V7zm28 4.076l-.744-1.857L26 10.522V7h-2v3.523L20.744 9.22L20 11.077l3.417 1.367L20.9 15.8l1.6 1.2l2.5-3.333L27.5 17l1.6-1.2l-2.517-3.357z"/></svg>"#;

// ============================================================================
// Main create post page
// ============================================================================

pub fn render_create_post_page(
    csrf_token: String,
    current_user: (i32, String, Option<String>),
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Create Post - StringTechHub" }
                link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@500;600&family=Source+Serif+4:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=IBM+Plex+Mono:wght@400;500&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/feed.css";
                link rel="stylesheet" href="/static/search.css";
                link rel="stylesheet" href="/static/create.css";
                link rel="icon" type="image/x-icon" href="/static/favicon.ico";
                link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png";
                link rel="apple-touch-icon" sizes="180x180" href="/static/apple-touch-icon.png";
                link rel="manifest" href="/static/site.webmanifest";

                script src="/static/create.js" defer {}
                script src="/static/script.js" defer {}
                script src="/static/marked.min.js" defer {}
            }
            body {
                (render_header(Some(&current_user), &csrf_token))

                div class="container" {
                    div class="page-header" {
                        h1 class="page-title" { "Create a Post" }
                        p class="page-subtitle" { "Share your knowledge and insights with the community" }
                    }

                    div class="info-box" {
                        div class="info-title" { "Tips for writing a great post" }
                        div class="info-text" {
                            "Make sure your title clearly describes what your post is about. Use proper formatting and add relevant tags to help others find your content."
                        }
                    }

                    form class="create-form" id="create-form" method="post" action="/posts/create" {
                        input type="hidden" name="csrf_token" value=(csrf_token);

                        // Title (optional for posts)
                        div class="form-section" {
                            label class="form-label" {
                                "Title "
                                span class="optional" { "(optional)" }
                            }
                            input
                                type="text"
                                class="title-input"
                                id="post-title"
                                name="title"
                                placeholder="Enter a descriptive title for your post..."
                                maxlength="200";
                            p class="form-help" { "A clear title helps others understand your post at a glance" }
                        }

                        // Body
                        (render_body_section("30000"))

                        // Tags
                        (render_tags_section())

                        // Form actions
                        div class="form-actions" {
                            a href="/" class="btn btn-secondary" { "Cancel" }
                            button type="submit" class="btn btn-primary" { "Publish Post" }
                        }
                    }
                }

                // Preview panel and overlay
                (render_preview_panel())
                (render_preview_overlay())
            }
        }
    }
}

// ============================================================================
// Main create question page
// ============================================================================

pub fn render_create_question_page(
    csrf_token: String,
    current_user: (i32, String, Option<String>),
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Ask a Question - StringTechHub" }
                link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@500;600&family=Source+Serif+4:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=IBM+Plex+Mono:wght@400;500&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/feed.css";
                link rel="stylesheet" href="/static/search.css";
                link rel="stylesheet" href="/static/create.css";
                link rel="icon" type="image/x-icon" href="/static/favicon.ico";
                link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png";
                link rel="apple-touch-icon" sizes="180x180" href="/static/apple-touch-icon.png";
                link rel="manifest" href="/static/site.webmanifest";
                
                script src="/static/create.js" defer {}
                script src="/static/marked.min.js" {}
                script src="/static/script.js" defer{}
            }
            body {
                (render_header(Some(&current_user), &csrf_token))

                div class="container" {
                    div class="page-header" {
                        h1 class="page-title" { "Ask a Question" }
                        p class="page-subtitle" { "Be specific and imagine you're asking a question to another person" }
                    }

                    div class="info-box" {
                        div class="info-title" { "Tips for asking a great question" }
                        div class="info-text" {
                            "Include all the information someone would need to answer your question. What have you tried? What's the expected vs actual behavior?"
                        }
                    }

                    form class="create-form" id="create-form" method="post" action="/questions/create" {
                        input type="hidden" name="csrf_token" value=(csrf_token);

                        // Title (required for questions)
                        div class="form-section" {
                            label class="form-label" {
                                "Title "
                                span class="required" { "*" }
                            }
                            input
                                type="text"
                                class="title-input"
                                id="post-title"
                                name="title"
                                placeholder="e.g., Why does my Axum middleware drop the request body before my handler reads it?"
                                maxlength="200"
                                required;
                            p class="form-help" { "Be specific and describe the problem you're facing" }
                        }

                        // Body
                        (render_body_section("15000"))

                        // Tags
                        (render_tags_section())

                        // Form actions
                        div class="form-actions" {
                            a href="/" class="btn btn-secondary" { "Cancel" }
                            button type="submit" class="btn btn-primary" { "Post Question" }
                        }
                    }
                }

                // Preview panel and overlay
                (render_preview_panel())
                (render_preview_overlay())
            }
        }
    }
}

// ============================================================================
// Header (same as feed/search/profile)
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
// Body editor section with markdown toolbar
// ============================================================================

fn render_body_section(max_chars: &str) -> Markup {
    html! {
        div class="form-section" {
            label class="form-label" {
                "Body "
                span class="required" { "*" }
            }

            div class="body-editor-container" id="body-editor" {
                textarea
                    class="body-textarea"
                    id="body-textarea"
                    name="content"
                    placeholder="Write your post content here... "
                    maxlength=(max_chars)
                    required
                    {}

                div class="editor-toolbar" {
                    div class="markdown-buttons" {
                        // Heading dropdown
                        div class="heading-dropdown" {
                            button type="button" class="markdown-btn" title="Heading" {
                                (PreEscaped(ICON_HEADING))
                            }
                            div class="heading-menu" id="heading-menu" {
                                button type="button" class="heading-option h2" data-heading="## " { "Heading 2" }
                                button type="button" class="heading-option h3" data-heading="### " { "Heading 3" }
                                button type="button" class="heading-option h4" data-heading="#### " { "Heading 4" }
                            }
                        }

                        // Bold
                        button type="button" class="markdown-btn" data-action="bold" title="Bold (Ctrl+B)" {
                            (PreEscaped(ICON_BOLD))
                        }

                        // Italic
                        button type="button" class="markdown-btn" data-action="italic" title="Italic (Ctrl+I)" {
                            (PreEscaped(ICON_ITALIC))
                        }

                        // Strikethrough
                        button type="button" class="markdown-btn" data-action="strikethrough" title="Strikethrough" {
                            (PreEscaped(ICON_STRIKETHROUGH))
                        }

                        // Inline code
                        button type="button" class="markdown-btn" data-action="inline-code" title="Inline Code" title="Inline Code" {
                            (PreEscaped(ICON_INLINE_CODE))
                        }

                        // Code block
                        button type="button" class="markdown-btn" data-action="code-block" title="Code Block" {
                            (PreEscaped(ICON_CODE_BLOCK))
                        }

                        // Link
                        button type="button" class="markdown-btn" data-action="link" title="Link (Ctrl+K)" {
                            (PreEscaped(ICON_LINK))
                        }

                        // Image
                        button type="button" class="markdown-btn" data-action="image" title="Image" {
                            (PreEscaped(ICON_IMAGE))
                        }

                        // Bullet list
                        button type="button" class="markdown-btn" data-action="bullet-list" title="Bullet List" {
                            (PreEscaped(ICON_BULLET_LIST))
                        }

                        // Numbered list
                        button type="button" class="markdown-btn" data-action="numbered-list" title="Numbered List" {
                            (PreEscaped(ICON_NUMBERED_LIST))
                        }

                        // Quote
                        button type="button" class="markdown-btn" data-action="quote" title="Quote" {
                            (PreEscaped(ICON_QUOTE))
                        }

                        // Table
                        button type="button" class="markdown-btn" data-action="table" title="Table" {
                            (PreEscaped(ICON_TABLE))
                        }

                        // Horizontal rule
                        button type="button" class="markdown-btn" data-action="hr" title="Horizontal Rule" {
                            (PreEscaped(ICON_HORIZONTAL_RULE))
                        }

                        // Footnote
                        button type="button" class="markdown-btn" data-action="footnote" title="Footnote" {
                            (PreEscaped(ICON_FOOTNOTE))
                        }
                    }

                    div class="editor-actions" {
                        span class="char-counter" id="body-char-counter" { "0 / " (max_chars) }
                        button type="button" class="btn-preview" id="preview-toggle" {
                            "Live Preview"
                        }
                    }
                }
            }

            p class="form-help" { "Minimum 10 characters required." }
        }
    }
}

// ============================================================================
// Tags section
// ============================================================================

fn render_tags_section() -> Markup {
    html! {
        div class="form-section" {
            label class="form-label" {
                "Tags "
                span class="required" { "*" }
            }

            div class="tags-input-container" id="tags-container" {
                input
                    type="text"
                    class="tags-input"
                    id="tags-input"
                    placeholder="Type a tag and press Space or Enter...";
            }

            input type="hidden" name="tags" id="tags-hidden" value="";

            p class="form-help" { "Add up to 5 tags to describe what your post is about. Press Space or Enter after each tag." }
        }
    }
}

// ============================================================================
// Preview panel
// ============================================================================

fn render_preview_panel() -> Markup {
    html! {
        div class="preview-panel" id="preview-panel" {
            div class="preview-header" { "Live Preview" }
            div class="preview-content" id="preview-content" {
                p { "Start typing to see a preview of your post..." }
            }
        }
    }
}

fn render_preview_overlay() -> Markup {
    html! {
        div class="preview-overlay" id="preview-overlay" {}
    }
}