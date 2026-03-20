// src/templates/answers.rs
use maud::{html, Markup, PreEscaped};
use crate::handlers::answers::AnswerResponse;
use crate::templates::feed::{truncate_html, should_show_read_more, format_time};

// ── Icons ──────────────────────────────────────────────────────────────────
const ICON_ECHO: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" fill="currentColor" viewBox="0 0 24 24"><path d="M14 12c0-1.1-.9-2-2-2s-2 .9-2 2a2 2 0 1 0 4 0m-6 0c0-1.07.42-2.07 1.17-2.82L7.76 7.76A5.97 5.97 0 0 0 6 12c0 1.6.62 3.11 1.76 4.25l1.41-1.42A3.96 3.96 0 0 1 8 12m8.24-4.24-1.41 1.41C15.59 9.93 16 10.93 16 12s-.42 2.07-1.17 2.83l1.41 1.41C17.37 15.11 18 13.6 18 12s-.62-3.11-1.76-4.24"></path><path d="M6.34 17.66C4.83 16.15 3.99 14.14 3.99 12s.83-4.14 2.34-5.65L4.92 4.93C3.03 6.82 1.99 9.33 1.99 12s1.04 5.18 2.93 7.07l1.41-1.41ZM19.07 4.93l-1.41 1.41C19.17 7.85 20 9.86 20 12s-.83 4.15-2.34 5.66l1.41 1.41C20.96 17.18 22 14.67 22 12s-1.04-5.18-2.93-7.07"></path></svg>"#;
const ICON_LINK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" fill="currentColor" viewBox="0 0 24 24"><path d="M9.88 18.36a3 3 0 0 1-4.24 0 3 3 0 0 1 0-4.24l2.83-2.83-1.41-1.41-2.83 2.83a5.003 5.003 0 0 0 0 7.07c.98.97 2.25 1.46 3.54 1.46s2.56-.49 3.54-1.46l2.83-2.83-1.41-1.41-2.83 2.83Zm2.83-14.14L9.88 7.05l1.41 1.41 2.83-2.83a3 3 0 0 1 4.24 0 3 3 0 0 1 0 4.24l-2.83 2.83 1.41 1.41 2.83-2.83a5.003 5.003 0 0 0 0-7.07 5.003 5.003 0 0 0-7.07 0Z"></path><path d="m16.95 8.46-.71-.7-.7-.71-4.25 4.24-4.24 4.25.71.7.7.71 4.25-4.24z"></path></svg>"#;
const ICON_DELETE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="M17 6V4c0-1.1-.9-2-2-2H9c-1.1 0-2 .9-2 2v2H2v2h2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8h2V6zM9 4h6v2H9zM6 20V8h12v12z"></path><path d="M9 10h2v8H9zm4 0h2v8h-2z"></path></svg>"#;
const ICON_CHEVRON_DOWN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 24 24"><path d="m6 9 6 6 6-6"/></svg>"#;

// Full markdown toolbar icons (same as create.rs)
const ICON_HEADING: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 12h10M7 5v14M17 5v14m-2 0h4M15 5h4M5 19h4M5 5h4"/></svg>"#;
const ICON_BOLD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 80 80"><path fill="currentColor" d="M24 41h17v-6H24zm17 0h2v-6h-2zm0-28H24.5v6H41zm2 48H24.5v6H43zM21 16.5V38h6V16.5zM21 38v25.5h6V38zm20 3c7.732 0 14-6.268 14-14h-6a8 8 0 0 1-8 8zm0-22a8 8 0 0 1 8 8h6c0-7.732-6.268-14-14-14zm2 22c5.523 0 10 4.477 10 10h6c0-8.837-7.163-16-16-16zM24.5 61a2.5 2.5 0 0 1 2.5 2.5h-6a3.5 3.5 0 0 0 3.5 3.5zM53 51c0 5.523-4.477 10-10 10v6c8.837 0 16-7.163 16-16zM24.5 13a3.5 3.5 0 0 0-3.5 3.5h6a2.5 2.5 0 0 1-2.5 2.5z"/></svg>"#;
const ICON_ITALIC: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5h6M7 19h6m1-14l-4 14"/></svg>"#;
const ICON_STRIKETHROUGH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><path fill="currentColor" d="M10.023 10h1.274q.01.12.01.25a2.56 2.56 0 0 1-.883 1.949q-.426.375-1.03.588A4.1 4.1 0 0 1 8.028 13a4.62 4.62 0 0 1-3.382-1.426q-.29-.388 0-.724q.29-.334.735-.13q.515.544 1.213.876q.699.33 1.449.33q.956 0 1.485-.433q.53-.435.53-1.14a1.7 1.7 0 0 0-.034-.353M5.586 7a2.5 2.5 0 0 1-.294-.507a2.3 2.3 0 0 1-.177-.934q0-.544.228-1.015t.633-.816t.955-.537A3.7 3.7 0 0 1 8.145 3q.867 0 1.603.33q.735.332 1.25.861q.24.423 0 .692q-.24.27-.662.102a3.4 3.4 0 0 0-.978-.669a2.9 2.9 0 0 0-1.213-.242q-.81 0-1.302.375t-.492 1.036q0 .354.14.596q.138.243.374.426q.236.184.515.324q.179.09.362.169zM2.5 8h11a.5.5 0 1 1 0 1h-11a.5.5 0 0 1 0-1"/></svg>"#;
const ICON_INLINE_CODE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M14.18 4.276a.75.75 0 0 1 .531.918l-3.973 14.83a.75.75 0 0 1-1.45-.389l3.974-14.83a.75.75 0 0 1 .919-.53m2.262 3.053a.75.75 0 0 1 1.059-.056l1.737 1.564c.737.662 1.347 1.212 1.767 1.71c.44.525.754 1.088.754 1.784c0 .695-.313 1.258-.754 1.782c-.42.499-1.03 1.049-1.767 1.711l-1.737 1.564a.75.75 0 0 1-1.004-1.115l1.697-1.527c.788-.709 1.319-1.19 1.663-1.598c.33-.393.402-.622.402-.818s-.072-.424-.402-.817c-.344-.409-.875-.89-1.663-1.598l-1.697-1.527a.75.75 0 0 1-.056-1.06m-8.94 1.06a.75.75 0 1 0-1.004-1.115L4.761 8.836c-.737.662-1.347 1.212-1.767 1.71c-.44.525-.754 1.088-.754 1.784c0 .695.313 1.258.754 1.782c.42.499 1.03 1.049 1.767 1.711l1.737 1.564a.75.75 0 0 0 1.004-1.115l-1.697-1.527c-.788-.709-1.319-1.19-1.663-1.598c-.33-.393-.402-.622-.402-.818s.072-.424.402-.817c.344-.409.875-.89 1.663-1.598z"/></svg>"#;
const ICON_CODE_BLOCK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="m9.6 14.908l.708-.714L8.114 12l2.174-2.175l-.707-.713L6.692 12zm4.8 0L17.308 12L14.4 9.092l-.708.714L15.887 12l-2.195 2.194zM5.616 20q-.691 0-1.153-.462T4 18.384V5.616q0-.691.463-1.153T5.616 4h12.769q.69 0 1.153.463T20 5.616v12.769q0 .69-.462 1.153T18.384 20zm0-1h12.769q.23 0 .423-.192t.192-.424V5.616q0-.231-.192-.424T18.384 5H5.616q-.231 0-.424.192T5 5.616v12.769q0 .23.192.423t.423.192M5 5v14z"/></svg>"#;
const ICON_LINK_MD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-dasharray="28" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 6l2 -2c1 -1 3 -1 4 0l1 1c1 1 1 3 0 4l-5 5c-1 1 -3 1 -4 0M11 18l-2 2c-1 1 -3 1 -4 0l-1 -1c-1 -1 -1 -3 0 -4l5 -5c1 -1 3 -1 4 0"><animate fill="freeze" attributeName="stroke-dashoffset" dur="0.6s" values="28;0"/></path></svg>"#;
const ICON_IMAGE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M5 3h13a3 3 0 0 1 3 3v13a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3m0 1a2 2 0 0 0-2 2v11.59l4.29-4.3l2.5 2.5l5-5L20 16V6a2 2 0 0 0-2-2zm4.79 13.21l-2.5-2.5L3 19a2 2 0 0 0 2 2h13a2 2 0 0 0 2-2v-1.59l-5.21-5.2zM7.5 6A2.5 2.5 0 0 1 10 8.5A2.5 2.5 0 0 1 7.5 11A2.5 2.5 0 0 1 5 8.5A2.5 2.5 0 0 1 7.5 6m0 1A1.5 1.5 0 0 0 6 8.5A1.5 1.5 0 0 0 7.5 10A1.5 1.5 0 0 0 9 8.5A1.5 1.5 0 0 0 7.5 7"/></svg>"#;
const ICON_BULLET_LIST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M10 4h10a1 1 0 0 1 0 2H10a1 1 0 1 1 0-2m0 7h10a1 1 0 0 1 0 2H10a1 1 0 0 1 0-2m0 7h10a1 1 0 0 1 0 2H10a1 1 0 0 1 0-2M5 7a2 2 0 1 1 0-4a2 2 0 0 1 0 4m0 7a2 2 0 1 1 0-4a2 2 0 0 1 0 4m0 7a2 2 0 1 1 0-4a2 2 0 0 1 0 4"/></svg>"#;
const ICON_NUMBERED_LIST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><path fill="currentColor" d="M3.59 3.03h12.2v1.26H3.59zm0 4.29h12.2v1.26H3.59zm0 4.35h12.2v1.26H3.59zM.99 4.79h.49V2.52H.6v.45h.39zm.87 3.88H.91l.14-.11l.3-.24c.35-.28.49-.5.49-.79A.74.74 0 0 0 1 6.8a.77.77 0 0 0-.81.84h.52A.34.34 0 0 1 1 7.25a.31.31 0 0 1 .31.31a.6.6 0 0 1-.22.44l-.87.75v.39h1.64zm-.36 3.56a.52.52 0 0 0 .28-.48a.67.67 0 0 0-.78-.62a.71.71 0 0 0-.77.75h.5a.3.3 0 0 1 .27-.32a.26.26 0 1 1 0 .51H.91v.38H1c.23 0 .37.11.37.29a.29.29 0 0 1-.33.29a.35.35 0 0 1-.36-.35H.21a.76.76 0 0 0 .83.8a.74.74 0 0 0 .83-.72a.53.53 0 0 0-.37-.53"/></svg>"#;
const ICON_QUOTE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M21.01 10h-2.85c.27-1.02 1.01-2.51 3.09-3.03l.76-.19V4h-1c-2.78 0-4.91.77-6.31 2.29c-1.89 2.05-1.7 4.68-1.69 4.71v7c0 1.1.9 2 2 2h6c1.1 0 2-.9 2-2v-6c0-1.1-.9-2-2-2m-12 0H6.16c.27-1.02 1.01-2.51 3.09-3.03l.76-.19V4h-1C6.23 4 4.1 4.77 2.7 6.29C.81 8.34 1 10.97 1.01 11v7c0 1.1.9 2 2 2h6c1.1 0 2-.9 2-2v-6c0-1.1-.9-2-2-2"/></svg>"#;
const ICON_TABLE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M6 5h11a3 3 0 0 1 3 3v9a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V8a3 3 0 0 1 3-3M4 17a2 2 0 0 0 2 2h5v-3H4zm7-5H4v3h7zm6 7a2 2 0 0 0 2-2v-1h-7v3zm2-7h-7v3h7zM4 11h7V8H4zm8 0h7V8h-7z"/></svg>"#;
const ICON_HORIZONTAL_RULE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><path fill="currentColor" d="M14.5 13.5a.5.5 0 0 1-.5.5H2a.5.5 0 0 1 0-1h12a.5.5 0 0 1 .5.5M2.5 11a.5.5 0 0 0 .5-.5v-3h3v3a.5.5 0 0 0 1 0v-7a.5.5 0 0 0-1 0v3H3v-3a.5.5 0 0 0-1 0v7a.5.5 0 0 0 .5.5m6.5-.5v-7a.5.5 0 0 1 .5-.5h2.25C12.99 3 14 4.009 14 5.25c0 .892-.526 1.657-1.28 2.021c.606.664.901 1.609 1.091 2.236c.058.189.132.437.186.558a.5.5 0 0 1-.247.935c-.531 0-.692-.531-.896-1.203C12.487 8.587 12.069 7.5 11 7.5h-1v3a.5.5 0 0 1-1 0m1-4h1.75c.689 0 1.25-.561 1.25-1.25S12.439 4 11.75 4H10z"/></svg>"#;
const ICON_FOOTNOTE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 32 32"><path fill="currentColor" d="M2 7v2h7v16h2V9h7V7zm28 4.076l-.744-1.857L26 10.522V7h-2v3.523L20.744 9.22L20 11.077l3.417 1.367L20.9 15.8l1.6 1.2l2.5-3.333L27.5 17l1.6-1.2l-2.517-3.357z"/></svg>"#;
const ICON_COMMENT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M20 2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h6.72l4.76 2.86c.16.09.34.14.51.14s.34-.04.49-.13c.31-.18.51-.51.51-.87v-2h3c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2Zm0 14h-4c-.55 0-1 .45-1 1v1.23l-3.49-2.09A1.03 1.03 0 0 0 11 16H4V4h16z"></path></svg>"#;

// ============================================================================
// Entry point — called from question page handler
// ============================================================================

pub fn render_answers_section(
    question_id: i32,
    question_author_id: i32,
    question_slug: &str,
    answers: Vec<AnswerResponse>,
    total_count: i32,
    has_more: bool,
    next_cursor: Option<i32>,
    current_user: Option<(i32, String, Option<String>)>,
    csrf_token: Option<&str>,
) -> Markup {
    html! {
        section class="answers-section" id="answers" {

            // ── Header ───────────────────────────────────────────────────
            div class="answers-header" {
                h2 class="answers-title" {
                    span class="answers-count" { (total_count) }
                    " Answer" (if total_count == 1 { "" } else { "s" })
                }
            }

            // ── Answer input (only if logged in AND not own question) ─────
            @if let Some((user_id, ref username, ref avatar)) = current_user {
                @if user_id != question_author_id {
                    (render_answer_input_box(
                        question_id,
                        question_slug,
                        username,
                        avatar.as_deref(),
                        csrf_token,
                    ))
                }
            } @else {
                div class="login-to-answer" {
                    a href="/auth/github" class="btn-login-answer" {
                        "Sign in to answer"
                    }
                }
            }

            // ── Answer list ───────────────────────────────────────────────
            div class="answer-list" id="answer-list" {
                @if answers.is_empty() {
                    div class="no-answers" {
                        p { "No answers yet. Be the first to answer!" }
                    }
                } @else {
                    @for answer in &answers {
                        (render_answer(
                            answer,
                            question_slug,
                            current_user.as_ref(),
                            csrf_token,
                        ))
                    }

                    // ── Load more ─────────────────────────────────────────
                    @if has_more {
                        @if let Some(cursor) = next_cursor {
                            div class="load-more-answers" id="load-more-answers" {
                                button
                                    class="btn-load-more-answers"
                                    data-question-id=(question_id)
                                    data-question-slug=(question_slug)
                                    data-cursor=(cursor)
                                {
                                    "Load more answers"
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
// Single answer card
// ============================================================================

pub fn render_answer(
    answer: &AnswerResponse,
    question_slug: &str,
    current_user: Option<&(i32, String, Option<String>)>,
    csrf_token: Option<&str>,
) -> Markup {
    let answer_id = answer.id;
    let is_own = current_user
        .map(|(uid, _, _)| *uid == answer.user_id)
        .unwrap_or(false);

    let initial = answer.username
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    let answer_link = format!("/questions/{}#{}", question_slug, answer.slug);
    let truncated = truncate_html(&answer.content_rendered_html, 300);
    let show_more = should_show_read_more(&answer.content_rendered_html, 300);

    html! {
        article
            class="answer-card feed-card"
            id=(format!("answer-{}",answer.id))
            data-answer-id=(answer_id)
            data-slug=(answer.slug)
            data-type="answer"
        {
            // ── Header ────────────────────────────────────────────────────
            
                div class="answer-card-header" {
                    div class="answer-avatar" { (initial) }
                    div class="answer-author-info" {
                        a href=(format!("/@{}", answer.username)) 
                        span class="answer-author-name" { (answer.username) }
                    
                        
                        div class="answer-meta" {
                            span class="answer-time" {
                                (format_time(&answer.created_at))
                            }
                            @if let Some(ref edited_at) = answer.edited_at {
                                span class="answer-edited" {
                                    "· edited " (format_time(edited_at))
                                }
                            }
                        }
                    }
                }
            

            // ── Body ──────────────────────────────────────────────────────
            div class="content-preview" {
                (PreEscaped(&truncated))
            } 
            @if show_more {
                a href=(format!("/answers/{}", answer.slug)) class="read-more" {
                    "Read more →"
                }
            }

            // ── Media (images/video) ──────────────────────────────────────
            @if !answer.media.is_empty() {
                div class="answer-media" {
                    @for media_item in &answer.media {
                        @if media_item.media_type == "image" {
                            img
                                src=(media_item.file_path)
                                alt="Answer attachment"
                                class="answer-media-img"
                                loading="lazy";
                        } @else {
                            video
                                src=(media_item.file_path)
                                class="answer-media-video"
                                controls
                                preload="metadata"
                            {}
                        }
                    }
                }
            }

            // ── Action bar ────────────────────────────────────────────────
            div class="answer-action-bar" {

                a href=(format!("/answers/{}#comments", answer.slug)) class="action-btn" title="Comments" {
                    (PreEscaped(ICON_COMMENT))
                    span class="count" { (answer.comment_count) }
                }
                // Echo
                button
                    class="answer-action-btn answer-echo-btn"
                    data-answer-id=(answer_id)
                    data-csrf=(csrf_token.unwrap_or(""))
                    title="Echo"
                {
                    (PreEscaped(ICON_ECHO))
                    span class="answer-echo-count" { (answer.echo_count) }
                }   

                // Copy link
                button
                    class="answer-action-btn answer-copy-link-btn"
                    data-copy-link=(answer_link)
                    title="Copy link to answer"
                {
                    (PreEscaped(ICON_LINK))
                }

                
            }
        }
    }
}


// ============================================================================
// Answer input box — collapsed prompt → expanded full editor
// ============================================================================

pub fn render_answer_input_box(
    question_id: i32,
    question_slug: &str,
    username: &str,
    _avatar: Option<&str>,
    csrf_token: Option<&str>,
) -> Markup {
    let initial = username
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    html! {
        div class="answer-input-container" id="answer-input-container" {

            // ── Collapsed prompt ──────────────────────────────────────────
            div class="answer-input-prompt" id="answer-input-prompt" {
                div class="answer-avatar" { (initial) }
                button
                    type="button"
                    class="answer-prompt-btn"
                    id="answer-prompt-btn"
                {
                    "Answer this question..."
                }
            }

            // ── Expanded editor (hidden by default) ───────────────────────
            div class="answer-editor" id="answer-editor" style="display:none" {

                // Editor top: avatar + textarea
                div class="answer-editor-top" {
                    div class="answer-avatar" { (initial) }
                    div class="answer-editor-body" {
                        textarea
                            class="answer-textarea"
                            id="answer-textarea"
                            placeholder="Write your answer here... Markdown is supported."
                            maxlength="30000"
                        {}
                    }
                }

                // ── Full markdown toolbar (mirrors create.rs) ─────────────
                div class="answer-editor-toolbar" {
                    div class="answer-md-buttons" {

                        // Heading dropdown
                        div class="answer-heading-dropdown" id="answer-heading-dropdown" {
                            button
                                type="button"
                                class="answer-md-btn"
                                id="answer-heading-btn"
                                title="Heading"
                            {
                                (PreEscaped(ICON_HEADING))
                            }
                            div class="answer-heading-menu" id="answer-heading-menu" {
                                button
                                    type="button"
                                    class="answer-heading-option answer-h2"
                                    data-answer-heading="## "
                                { "Heading 2" }
                                button
                                    type="button"
                                    class="answer-heading-option answer-h3"
                                    data-answer-heading="### "
                                { "Heading 3" }
                                button
                                    type="button"
                                    class="answer-heading-option answer-h4"
                                    data-answer-heading="#### "
                                { "Heading 4" }
                            }
                        }

                        // Standard toolbar buttons — data-answer-action drives JS
                        button type="button" class="answer-md-btn" data-answer-action="bold" title="Bold (Ctrl+B)" {
                            (PreEscaped(ICON_BOLD))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="italic" title="Italic (Ctrl+I)" {
                            (PreEscaped(ICON_ITALIC))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="strikethrough" title="Strikethrough" {
                            (PreEscaped(ICON_STRIKETHROUGH))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="inline-code" title="Inline Code" {
                            (PreEscaped(ICON_INLINE_CODE))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="code-block" title="Code Block" {
                            (PreEscaped(ICON_CODE_BLOCK))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="link" title="Link (Ctrl+K)" {
                            (PreEscaped(ICON_LINK_MD))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="image" title="Image" {
                            (PreEscaped(ICON_IMAGE))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="bullet-list" title="Bullet List" {
                            (PreEscaped(ICON_BULLET_LIST))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="numbered-list" title="Numbered List" {
                            (PreEscaped(ICON_NUMBERED_LIST))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="quote" title="Quote" {
                            (PreEscaped(ICON_QUOTE))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="table" title="Table" {
                            (PreEscaped(ICON_TABLE))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="hr" title="Horizontal Rule" {
                            (PreEscaped(ICON_HORIZONTAL_RULE))
                        }
                        button type="button" class="answer-md-btn" data-answer-action="footnote" title="Footnote" {
                            (PreEscaped(ICON_FOOTNOTE))
                        }
                    }

                    // Toolbar right: counter + preview toggle
                    div class="answer-toolbar-right" {
                        span class="answer-char-counter" id="answer-char-counter" {
                            "0 / 30,000"
                        }
                        button
                            type="button"
                            class="answer-preview-btn"
                            id="answer-preview-btn"
                        {
                            "Preview"
                        }
                        button 
                            type="button"
                            class="answer-submit-btn"
                            id="answer-submit-btn"
                            data-question-id=(question_id)
                            data-question-slug=(question_slug)
                            data-csrf=(csrf_token.unwrap_or(""))
                            disabled
                        {
                            "Answer"
                        }
                    }
                }
                
            }
        }

        // ── Live preview overlay (same pattern as create.rs) ─────────────
        div class="answer-preview-overlay" id="answer-preview-overlay" {}
        div class="answer-editor-split-panel" id="answer-editor-split-panel" style="display:none" {
            div class="split-panel-label" { "Your Answer" }
            textarea
                id="answer-textarea-split"
                placeholder="Write your answer here... Markdown is supported."
                maxlength="30000"
            {}

            div class="answer-editor-toolbar" style="border-top: 1px solid var(--border); margin-top: auto;" {
                div class="answer-md-buttons" {
                     div class="answer-heading-dropdown" id="answer-heading-dropdown-split" {
                            button
                                type="button"
                                class="answer-md-btn"
                                id="answer-heading-btn-split"
                                title="Heading"
                            {
                                (PreEscaped(ICON_HEADING))
                            }
                            div class="answer-heading-menu" id="answer-heading-menu-split" {
                                button
                                    type="button"
                                    class="answer-heading-option answer-h2"
                                    data-answer-heading="## "
                                { "Heading 2" }
                                button
                                    type="button"
                                    class="answer-heading-option answer-h3"
                                    data-answer-heading="### "
                                { "Heading 3" }
                                button
                                    type="button"
                                    class="answer-heading-option answer-h4"
                                    data-answer-heading="#### "
                                { "Heading 4" }
                            }
                        }

                    // same buttons as main toolbar...
                    button type="button" class="answer-md-btn" data-answer-action="bold" title="Bold" {
                        (PreEscaped(ICON_BOLD)) 
                    }
                    button type="button" class="answer-md-btn" data-answer-action="italic" title="Italic" { 
                        (PreEscaped(ICON_ITALIC)) 
                    }
                    button type="button" class="answer-md-btn" data-answer-action="strikethrough" title="Strikethrough" {
                        (PreEscaped(ICON_STRIKETHROUGH))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="inline-code" title="Inline Code" {
                        (PreEscaped(ICON_INLINE_CODE))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="code-block" title="Code Block" {
                        (PreEscaped(ICON_CODE_BLOCK))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="link" title="Link (Ctrl+K)" {
                        (PreEscaped(ICON_LINK_MD))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="image" title="Image" {
                        (PreEscaped(ICON_IMAGE))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="bullet-list" title="Bullet List" {
                        (PreEscaped(ICON_BULLET_LIST))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="numbered-list" title="Numbered List" {
                        (PreEscaped(ICON_NUMBERED_LIST))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="quote" title="Quote" {
                        (PreEscaped(ICON_QUOTE))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="table" title="Table" {
                        (PreEscaped(ICON_TABLE))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="hr" title="Horizontal Rule" {
                        (PreEscaped(ICON_HORIZONTAL_RULE))
                    }
                    button type="button" class="answer-md-btn" data-answer-action="footnote" title="Footnote" {
                        (PreEscaped(ICON_FOOTNOTE))
                    }
                }
                div class="answer-toolbar-right" {
                    span class="answer-char-counter" id="answer-char-counter-split" {}
                    button type="button" class="answer-preview-btn active" id="answer-preview-btn-split" {
                        "Cancel Preview"
                    }
                }
            }

        }
        div class="answer-preview-panel" id="answer-preview-panel" {
            div class="answer-preview-header" { "Live Preview" }
            div class="answer-preview-content" id="answer-preview-content" {
                p { "Start typing to see a preview..." }
            }
        }
    }
}

// ============================================================================
// HTMX / fetch fragments
// ============================================================================

/// Single answer fragment — returned after a successful POST /api/questions/:id/answers
pub fn render_answer_fragment(
    answer: &AnswerResponse,
    question_slug: &str,
    current_user: Option<&(i32, String, Option<String>)>,
    csrf_token: Option<&str>,
) -> Markup {
    render_answer(answer, question_slug, current_user, csrf_token)
}

/// Paginated answer list fragment — returned by GET /api/questions/:id/answers?cursor=…
pub fn render_answer_list_fragment(
    answers: &[AnswerResponse],
    question_slug: &str,
    current_user: Option<&(i32, String, Option<String>)>,
    csrf_token: Option<&str>,
    has_more: bool,
    next_cursor: Option<i32>,
    question_id: i32,
) -> Markup {
    html! {
        @for answer in answers {
            (render_answer(answer, question_slug, current_user, csrf_token))
        }
        @if has_more {
            @if let Some(cursor) = next_cursor {
                div class="load-more-answers" id="load-more-answers" {
                    button
                        class="btn-load-more-answers"
                        data-question-id=(question_id)
                        data-question-slug=(question_slug)
                        data-cursor=(cursor)
                    {
                        "Load more answers"
                    }
                }
            }
        }
    }
}

pub fn render_answer_page(
    answer: &AnswerResponse,
    question: &crate::handlers::questions::QuestionResponse,
    comments: Vec<crate::handlers::comments::CommentResponse>,
    total_comment_count: i32,
    has_more_comments: bool,
    next_comment_cursor: Option<String>,
    current_user: Option<(i32, String, Option<String>)>,
    csrf_token: Option<&str>,
    back_url: String,
) -> Markup {
    let initial = answer.username
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    let answer_url = format!("/answers/{}", answer.slug);
    let question_url = format!("/questions/{}", question.slug);

    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (answer.username) "'s answer — StringTechHub" }
                link rel="stylesheet"
                    href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@500;600&family=Source+Serif+4:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&family=IBM+Plex+Mono:wght@400;500&display=swap";
                link rel="stylesheet" href="/static/feed.css";
                link rel="stylesheet" href="/static/post.css";
                link rel="stylesheet" href="/static/comments.css";
                link rel="stylesheet" href="/static/answer_page.css";
                link rel="icon" type="image/x-icon" href="/static/favicon.ico";
                link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png";
                link rel="apple-touch-icon" sizes="180x180" href="/static/apple-touch-icon.png";
                link rel="manifest" href="/static/site.webmanifest";

                script src="/static/marked.min.js" defer {}
                script src="/static/script.js" defer {}
                script src="/static/comment.js" defer {}
                script src="/static/answers.js" defer {}

                @if let Some(ref token) = csrf_token {
                    meta name="csrf-token" content=(token);
                }
            }
            body {
                // ── Global nav (reuse feed header pattern) ────────────────
               (crate::templates::feed::render_header(current_user.as_ref(), csrf_token))

                div class="answer-page-container" {

                    // ── Back to question ──────────────────────────────────
                    a href=(back_url) class="answer-page-back" {
                        (PreEscaped(r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M21 11H6.83l3.58-3.59L9 6l-6 6 6 6 1.41-1.41L6.83 13H21z"/></svg>"#))
                        "Back"
                    }

                    // ── Collapsible question summary ──────────────────────
                    div class="aq-summary" {
                        span class="aq-badge" { "QUESTION" }
                        h2 class="aq-title" {
                            a href=(question_url) { (question.title) }
                        }
                        div class="aq-meta" {
                            "Asked by "
                            a href=(format!("/@{}", question.username)) class="aq-author" {
                                (question.username)
                            }
                            span class="aq-sep" { "." }
                            span { (format_time(&question.created_at)) }
                            
                        } 
                        @if !question.tags.is_empty() {
                            div class="aq-tags" {
                                @for tag in &question.tags {
                                    a href=(format!("/tags/{}", tag.slug)) class="aq-tag" {
                                        (tag.name)
                                    }
                                }
                            }
                        }
                        a href=(question_url) class="aq-view-full" {
                            "View full question →"
                        }

                    }

                    // ── Answer card ───────────────────────────────────────
                    

                        // Header: avatar + author + time
                        div class="answer-page-header" {
                            div class="answer-page-avatar" { (initial) }
                            div class="answer-page-author-info" {
                                a href=(format!("/@{}", answer.username)) class="answer-page-author" {
                                    (answer.username)
                                }
                                div class="answer-page-meta" {
                                    "Answered " (format_time(&answer.created_at))
                                    @if let Some(ref edited_at) = answer.edited_at {
                                        " • Edited " (format_time(edited_at))
                                    }
                                }
                            }
                        }

                        // Divider
                        hr class="answer-page-divider";

                        // Full markdown body — code blocks rendered server-side
                        // with language label + copy button (same as post/question pages)
                        div class="answer-page-body" {
                            (PreEscaped(&answer.content_rendered_html))
                        }

                        // Media
                        @if !answer.media.is_empty() {
                            div class="answer-page-media" {
                                @for item in &answer.media {
                                    @if item.media_type == "image" {
                                        img
                                            src=(item.file_path)
                                            alt="Answer attachment"
                                            class="answer-page-media-img"
                                            loading="lazy";
                                    } @else {
                                        video
                                            src=(item.file_path)
                                            class="answer-page-media-video"
                                            controls
                                            preload="metadata"
                                        {}
                                    }
                                }
                            }
                        }

                        // ── Action bar ────────────────────────────────────
                        div class="answer-page-actions" {
                            // Echo — same endpoint as posts/questions: POST /echo
                            button
                                class="answer-page-action-btn answer-page-echo-btn action-echo"
                                data-echo-type="answer"
                                data-echo-id=(answer.id)
                                data-csrf=(csrf_token.unwrap_or(""))
                                title="Echo"
                            {
                                (PreEscaped(ICON_ECHO))
                                span class="answer-page-echo-count" { (answer.echo_count) }
                            }

                            // Copy link
                            button
                                class="answer-page-action-btn answer-page-copy-btn"
                                data-copy-link=(format!("/answers/{}", answer.slug))
                                title="Copy link"
                            {
                                (PreEscaped(ICON_LINK))
                                "Copy link"
                            }
                        }

                        (crate::templates::comments::render_comments_section(
                            "answer",
                            answer.id,
                            answer.user_id,
                            comments,
                            total_comment_count,
                            has_more_comments,
                            next_comment_cursor,
                            current_user,
                            csrf_token,
                        ))
                    }
                }            
            }
        }
    }

