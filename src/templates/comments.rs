use maud::{html, Markup, PreEscaped};
use crate::handlers::comments::CommentResponse;
use crate::templates::feed::format_time;

// ── Icons (fill these in) ──────────────────────────────────────────────────
const ICON_HEART: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">heart-outline</title><path fill="currentColor" d="m12.1 18.55l-.1.1l-.11-.1C7.14 14.24 4 11.39 4 8.5C4 6.5 5.5 5 7.5 5c1.54 0 3.04 1 3.57 2.36h1.86C13.46 6 14.96 5 16.5 5c2 0 3.5 1.5 3.5 3.5c0 2.89-3.14 5.74-7.9 10.05M16.5 3c-1.74 0-3.41.81-4.5 2.08C10.91 3.81 9.24 3 7.5 3C4.42 3 2 5.41 2 8.5c0 3.77 3.4 6.86 8.55 11.53L12 21.35l1.45-1.32C18.6 15.36 22 12.27 22 8.5C22 5.41 19.58 3 16.5 3"/></svg>"#;
const ICON_REPLY: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M17 5H6c-1.1 0-2 .9-2 2v5h2V7h11v3l5-4-5-4zm1 12H7v-3l-5 4 5 4v-3h11c1.1 0 2-.9 2-2v-5h-2z"></path></svg>"#;
const ICON_CHEVRON_DOWN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">down-fill</title><g fill="none" fill-rule="evenodd"><path d="M24 0v24H0V0zM12.593 23.258l-.011.002l-.071.035l-.02.004l-.014-.004l-.071-.035q-.016-.005-.024.005l-.004.01l-.017.428l.005.02l.01.013l.104.074l.015.004l.012-.004l.104-.074l.012-.016l.004-.017l-.017-.427q-.004-.016-.017-.018m.265-.113l-.013.002l-.185.093l-.01.01l-.003.011l.018.43l.005.012l.008.007l.201.093q.019.005.029-.008l.004-.014l-.034-.614q-.005-.019-.02-.022m-.715.002a.02.02 0 0 0-.027.006l-.006.014l-.034.614q.001.018.017.024l.015-.002l.201-.093l.01-.008l.004-.011l.017-.43l-.003-.012l-.01-.01z"/><path fill="currentColor" d="M13.06 16.06a1.5 1.5 0 0 1-2.12 0l-5.658-5.656a1.5 1.5 0 1 1 2.122-2.121L12 12.879l4.596-4.596a1.5 1.5 0 0 1 2.122 2.12l-5.657 5.658Z"/></g></svg>"#;
const ICON_BOLD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 80 80"><title xmlns="">b-bold</title><path fill="currentColor" d="M24 41h17v-6H24zm17 0h2v-6h-2zm0-28H24.5v6H41zm2 48H24.5v6H43zM21 16.5V38h6V16.5zM21 38v25.5h6V38zm20 3c7.732 0 14-6.268 14-14h-6a8 8 0 0 1-8 8zm0-22a8 8 0 0 1 8 8h6c0-7.732-6.268-14-14-14zm2 22c5.523 0 10 4.477 10 10h6c0-8.837-7.163-16-16-16zM24.5 61a2.5 2.5 0 0 1 2.5 2.5h-6a3.5 3.5 0 0 0 3.5 3.5zM53 51c0 5.523-4.477 10-10 10v6c8.837 0 16-7.163 16-16zM24.5 13a3.5 3.5 0 0 0-3.5 3.5h6a2.5 2.5 0 0 1-2.5 2.5z"/></svg>"#;
const ICON_ITALIC: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">italic</title><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5h6M7 19h6m1-14l-4 14"/></svg>"#;
const ICON_STRIKETHROUGH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><title xmlns="">editor-strike</title><path fill="currentColor" d="M10.023 10h1.274q.01.12.01.25a2.56 2.56 0 0 1-.883 1.949q-.426.375-1.03.588A4.1 4.1 0 0 1 8.028 13a4.62 4.62 0 0 1-3.382-1.426q-.29-.388 0-.724q.29-.334.735-.13q.515.544 1.213.876q.699.33 1.449.33q.956 0 1.485-.433q.53-.435.53-1.14a1.7 1.7 0 0 0-.034-.353M5.586 7a2.5 2.5 0 0 1-.294-.507a2.3 2.3 0 0 1-.177-.934q0-.544.228-1.015t.633-.816t.955-.537A3.7 3.7 0 0 1 8.145 3q.867 0 1.603.33q.735.332 1.25.861q.24.423 0 .692q-.24.27-.662.102a3.4 3.4 0 0 0-.978-.669a2.9 2.9 0 0 0-1.213-.242q-.81 0-1.302.375t-.492 1.036q0 .354.14.596q.138.243.374.426q.236.184.515.324q.179.09.362.169zM2.5 8h11a.5.5 0 1 1 0 1h-11a.5.5 0 0 1 0-1"/></svg>"#;
const ICON_INLINE_CODE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">code-bold</title><path fill="currentColor" d="M14.18 4.276a.75.75 0 0 1 .531.918l-3.973 14.83a.75.75 0 0 1-1.45-.389l3.974-14.83a.75.75 0 0 1 .919-.53m2.262 3.053a.75.75 0 0 1 1.059-.056l1.737 1.564c.737.662 1.347 1.212 1.767 1.71c.44.525.754 1.088.754 1.784c0 .695-.313 1.258-.754 1.782c-.42.499-1.03 1.049-1.767 1.711l-1.737 1.564a.75.75 0 0 1-1.004-1.115l1.697-1.527c.788-.709 1.319-1.19 1.663-1.598c.33-.393.402-.622.402-.818s-.072-.424-.402-.817c-.344-.409-.875-.89-1.663-1.598l-1.697-1.527a.75.75 0 0 1-.056-1.06m-8.94 1.06a.75.75 0 1 0-1.004-1.115L4.761 8.836c-.737.662-1.347 1.212-1.767 1.71c-.44.525-.754 1.088-.754 1.784c0 .695.313 1.258.754 1.782c.42.499 1.03 1.049 1.767 1.711l1.737 1.564a.75.75 0 0 0 1.004-1.115l-1.697-1.527c-.788-.709-1.319-1.19-1.663-1.598c-.33-.393-.402-.622-.402-.818s.072-.424.402-.817c.344-.409.875-.89 1.663-1.598z"/></svg>"#;
const ICON_CODE_BLOCK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">code-blocks-outline</title><path fill="currentColor" d="m9.6 14.908l.708-.714L8.114 12l2.174-2.175l-.707-.713L6.692 12zm4.8 0L17.308 12L14.4 9.092l-.708.714L15.887 12l-2.195 2.194zM5.616 20q-.691 0-1.153-.462T4 18.384V5.616q0-.691.463-1.153T5.616 4h12.769q.69 0 1.153.463T20 5.616v12.769q0 .69-.462 1.153T18.384 20zm0-1h12.769q.23 0 .423-.192t.192-.424V5.616q0-.231-.192-.424T18.384 5H5.616q-.231 0-.424.192T5 5.616v12.769q0 .23.192.423t.423.192M5 5v14z"/></svg>"#;
const ICON_LINK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">link</title><path fill="none" stroke="currentColor" stroke-dasharray="28" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 6l2 -2c1 -1 3 -1 4 0l1 1c1 1 1 3 0 4l-5 5c-1 1 -3 1 -4 0M11 18l-2 2c-1 1 -3 1 -4 0l-1 -1c-1 -1 -1 -3 0 -4l5 -5c1 -1 3 -1 4 0"><animate fill="freeze" attributeName="stroke-dashoffset" dur="0.6s" values="28;0"/></path></svg>"#;
const ICON_BULLET_LIST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">list-bullet</title><path fill="currentColor" fill-rule="evenodd" d="M10 4h10a1 1 0 0 1 0 2H10a1 1 0 1 1 0-2m0 7h10a1 1 0 0 1 0 2H10a1 1 0 0 1 0-2m0 7h10a1 1 0 0 1 0 2H10a1 1 0 0 1 0-2M5 7a2 2 0 1 1 0-4a2 2 0 0 1 0 4m0 7a2 2 0 1 1 0-4a2 2 0 0 1 0 4m0 7a2 2 0 1 1 0-4a2 2 0 0 1 0 4"/></svg>"#;
const ICON_NUMBERED_LIST: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><title xmlns="">ordered-list</title><path fill="currentColor" d="M3.59 3.03h12.2v1.26H3.59zm0 4.29h12.2v1.26H3.59zm0 4.35h12.2v1.26H3.59zM.99 4.79h.49V2.52H.6v.45h.39zm.87 3.88H.91l.14-.11l.3-.24c.35-.28.49-.5.49-.79A.74.74 0 0 0 1 6.8a.77.77 0 0 0-.81.84h.52A.34.34 0 0 1 1 7.25a.31.31 0 0 1 .31.31a.6.6 0 0 1-.22.44l-.87.75v.39h1.64zm-.36 3.56a.52.52 0 0 0 .28-.48a.67.67 0 0 0-.78-.62a.71.71 0 0 0-.77.75h.5a.3.3 0 0 1 .27-.32a.26.26 0 1 1 0 .51H.91v.38H1c.23 0 .37.11.37.29a.29.29 0 0 1-.33.29a.35.35 0 0 1-.36-.35H.21a.76.76 0 0 0 .83.8a.74.74 0 0 0 .83-.72a.53.53 0 0 0-.37-.53"/></svg>"#;
const ICON_QUOTE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">quote-left-filled</title><path fill="currentColor" d="M21.01 10h-2.85c.27-1.02 1.01-2.51 3.09-3.03l.76-.19V4h-1c-2.78 0-4.91.77-6.31 2.29c-1.89 2.05-1.7 4.68-1.69 4.71v7c0 1.1.9 2 2 2h6c1.1 0 2-.9 2-2v-6c0-1.1-.9-2-2-2m-12 0H6.16c.27-1.02 1.01-2.51 3.09-3.03l.76-.19V4h-1C6.23 4 4.1 4.77 2.7 6.29C.81 8.34 1 10.97 1.01 11v7c0 1.1.9 2 2 2h6c1.1 0 2-.9 2-2v-6c0-1.1-.9-2-2-2"/></svg>"#;


// ============================================================================
// Entry point — called from post/question page handler
// ============================================================================

pub fn render_comments_section(
    parent_type: &str,   // "post" or "question"
    parent_id: i32,
    parent_author_id: i32,  // NEW: author of the post/question
    comments: Vec<CommentResponse>,
    total_count: i32,
    has_more: bool,
    next_cursor: Option<String>,
    current_user: Option<(i32, String, Option<String>)>,
    csrf_token: Option<&str>,
) -> Markup {
    html! {
        section class="comments-section" id="comments" {
            div class="comments-header" {
                h2 class="comments-title" {
                    span class="comments-count" { (total_count) }
                    " Comments"
                }
            }

            // ── Add comment box (only if logged in AND not own post/question) ─────
            @if let Some((user_id, ref username, ref avatar)) = current_user {
                @if user_id != parent_author_id {
                    (render_comment_input_box(
                        parent_type,
                        parent_id,
                        None, // no parent comment
                        0,    // depth 0
                        username,
                        avatar.as_deref(),
                        csrf_token,
                    ))
                }
            } @else {
                div class="login-to-comment" {
                    a href="/auth/github" class="btn-login-comment" {
                        "Sign in to comment"
                    }
                }
            }

            // ── Comment list ─────────────────────────────────────────────
            div class="comment-list" id="comment-list" {
                @if comments.is_empty() {
                    div class="no-comments" {
                        p { "No comments yet. Be the first to comment!" }
                    }
                } @else {
                    @for comment in &comments {
                        (render_comment(
                            comment,
                            parent_type,
                            parent_id,
                            current_user.as_ref(),
                            csrf_token,
                        ))
                    }

                    // ── Load more ────────────────────────────────────────
                    @if has_more {
                        @if let Some(ref cursor) = next_cursor {
                            div class="load-more-comments" id="load-more-comments" {
                                button
                                    class="btn-load-more"
                                    data-url=(format!(
                                        "/{}s/{}/comments?cursor={}",
                                        parent_type, parent_id, cursor
                                    ))
                                {
                                    "Load more comments"
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
// Single comment (recursive — renders up to depth 3)
// ============================================================================

pub fn render_comment(
    comment: &CommentResponse,
    parent_type: &str,
    parent_id: i32,
    current_user: Option<&(i32, String, Option<String>)>,
    csrf_token: Option<&str>,
) -> Markup {
    let depth = comment.depth_level;
    let comment_id = comment.id;
    let is_deleted = comment.is_deleted;

    html! {
        div
            class=(format!("comment depth-{}", depth))
            id=(format!("comment-{}", comment_id))
        {
            // ── Header ───────────────────────────────────────────────────
            div class="comment-header" {
                div class="comment-avatar" {
                    span { (comment.username.chars().next().unwrap_or('?').to_uppercase().to_string()) }
                }
                div class="comment-meta" {
                    a href=(format!("/@{}", comment.username)) class="comment-author" {
                        (comment.username)
                    }
                    span class="comment-time" {
                        (format_time(&comment.created_at))
                    }
                    @if comment.edited_at.is_some() {
                        span class="comment-edited" { "· edited" }
                    }
                }
            }

            // ── Body ─────────────────────────────────────────────────────
            div class="comment-body" {
                @if is_deleted {
                    span class="comment-deleted" { "[deleted]" }
                } @else {
                    (PreEscaped(&comment.content_rendered_html))
                }
            }

            // ── Actions ──────────────────────────────────────────────────
            @if !is_deleted {
                div class="comment-actions" {
                    // Helpful button
                    button
                        class=(format!("comment-action-btn helpful-btn{}", if comment.has_marked_helpful { " active" } else { "" }))
                        data-comment-id=(comment_id)
                        data-csrf=(csrf_token.unwrap_or(""))
                    {
                        (PreEscaped(ICON_HEART))
                        span class="helpful-count" { (comment.helpful_count) }
                    }

                    // Reply button (not at max depth, not own comment, and user is logged in)
                    @if depth < 3 && current_user.is_some() {
                        @if let Some((user_id, _, _)) = current_user {
                            @if *user_id != comment.user_id {
                                button
                                    class="comment-action-btn reply-btn"
                                    data-comment-id=(comment_id)
                                    data-depth=(depth)
                                {
                                    (PreEscaped(ICON_REPLY))
                                    "Reply"
                                }
                            }
                        }
                    }

                    // Show replies button
                    @if comment.reply_count > 0 {
                        button
                            class="comment-action-btn show-replies-btn"
                            data-comment-id=(comment_id)
                            data-reply-count=(comment.reply_count)
                        {
                            (PreEscaped(ICON_CHEVRON_DOWN))
                            span {
                                (format!("{} repl{}", comment.reply_count, if comment.reply_count == 1 { "y" } else { "ies" }))
                            }
                        }
                    }
                }
            }

            // ── Reply input (hidden by default, not shown for own comments) ──────
            @if depth < 3 && current_user.is_some() {
                @if let Some((user_id, ref username, ref avatar)) = current_user {
                    @if *user_id != comment.user_id {
                        div
                            class="reply-input-wrapper"
                            id=(format!("reply-input-{}", comment_id))
                            style="display:none"
                        {
                            (render_comment_input_box(
                                parent_type,
                                parent_id,
                                Some(comment_id),
                                depth + 1,
                                username,
                                avatar.as_deref(),
                                csrf_token,
                            ))
                        }
                    }
                }
            }

            // ── Replies container ─────────────────────────────────────────
            @if comment.reply_count > 0 && depth < 3 {
                div
                    class="replies-container"
                    id=(format!("replies-{}", comment_id))
                    style="display:none"
                {}
            }
        }
    }
}

// ============================================================================
// Comment input box — reused for top-level and replies
// depth 0: 1500 chars, full markdown
// depth 1: 1000 chars, bold/italic/link only
// depth 2: 500 chars, bold/italic/link only
// depth 3: 250 chars, plain text only
// ============================================================================

pub fn render_comment_input_box(
    parent_type: &str,
    parent_id: i32,
    parent_comment_id: Option<i32>,
    depth: i32,
    username: &str,
    _avatar: Option<&str>,
    csrf_token: Option<&str>,
) -> Markup {
    let max_chars = match depth {
        0 => 1500,
        1 => 1000,
        2 => 500,
        _ => 250,
    };
    let placeholder = if parent_comment_id.is_some() {
        "Write a reply..."
    } else {
        "Add a comment..."
    };
    let input_id = match parent_comment_id {
        Some(id) => format!("textarea-reply-{}", id),
        None => "textarea-comment".to_string(),
    };
    let form_id = match parent_comment_id {
        Some(id) => format!("form-reply-{}", id),
        None => "form-comment".to_string(),
    };
    let initial = username.chars().next().unwrap_or('?').to_uppercase().to_string();

    html! {
        div class="comment-input-container" id=(format!("input-container-{}", form_id)) {
            div class="comment-input-header" {
                div class="comment-avatar" { span { (initial) } }
                div class="comment-input-body" {
                    textarea
                        class="comment-textarea"
                        id=(input_id)
                        placeholder=(placeholder)
                        maxlength=(max_chars)
                        data-form-id=(form_id)
                        data-depth=(depth)
                        rows="1"
                    {}
                }
            }

            // Toolbar — only shown when textarea focused
            div class="comment-toolbar" id=(format!("toolbar-{}", form_id)) style="display:none" {

                // Markdown buttons — depth-dependent
                @if depth < 3 {
                    div class="comment-markdown-buttons" {
                        @if depth == 0 {
                            // Full markdown for depth 0
                            button type="button" class="comment-md-btn" data-form=(form_id) data-action="bold" title="Bold" { (PreEscaped(ICON_BOLD)) }
                            button type="button" class="comment-md-btn" data-form=(form_id) data-action="italic" title="Italic" { (PreEscaped(ICON_ITALIC)) }
                            button type="button" class="comment-md-btn" data-form=(form_id) data-action="inline-code" title="Code" { (PreEscaped(ICON_INLINE_CODE)) }
                            button type="button" class="comment-md-btn" data-form=(form_id) data-action="link" title="Link" { (PreEscaped(ICON_LINK)) }
                            button type="button" class="comment-md-btn" data-form=(form_id) data-action="quote" title="Quote" { (PreEscaped(ICON_QUOTE)) }
                            button type="button" class="comment-md-btn" data-form=(form_id) data-action="bullet-list" title="List" { (PreEscaped(ICON_BULLET_LIST)) }
                        } @else {
                            // Bold/italic/link only for depth 1-2
                            button type="button" class="comment-md-btn" data-form=(form_id) data-action="bold" title="Bold" { (PreEscaped(ICON_BOLD)) }
                            button type="button" class="comment-md-btn" data-form=(form_id) data-action="italic" title="Italic" { (PreEscaped(ICON_ITALIC)) }
                            button type="button" class="comment-md-btn" data-form=(form_id) data-action="link" title="Link" { (PreEscaped(ICON_LINK)) }
                        }
                    }
                }

                div class="comment-toolbar-right" {
                    span class="comment-char-counter" data-form-id=(form_id) {
                        "0 / " (max_chars)
                    }
                    @if depth == 0 {
                        button
                            type="button"
                            class="comment-preview-btn"
                            data-form-id=(form_id)
                        {
                            "Preview"
                        }
                    }
                    @if parent_comment_id.is_some() {
                        button
                            type="button"
                            class="comment-cancel-btn"
                            data-comment-id=(parent_comment_id.unwrap_or(0))
                        {
                            "Cancel"
                        }
                    }
                    button
                        type="button"
                        class="comment-submit-btn"
                        data-form-id=(form_id)
                        data-parent-type=(parent_type)
                        data-parent-id=(parent_id)
                        data-parent-comment-id=(parent_comment_id.unwrap_or(0))
                        data-depth=(depth)
                        data-csrf=(csrf_token.unwrap_or(""))
                        disabled
                    {
                        @if parent_comment_id.is_some() { "Reply" } @else { "Comment" }
                    }
                }
            }

            // Preview panel for depth-0 only
            @if depth == 0 {
                div class="comment-preview-panel" id=(format!("preview-{}", form_id)) style="display:none" {
                    div class="comment-preview-label" { "Preview" }
                    div class="comment-preview-content" id=(format!("preview-content-{}", form_id)) {}
                }
            }
        }
    }
}

// ============================================================================
// Fragment returned by HTMX after submitting a comment
// ============================================================================

pub fn render_comment_fragment(
    comment: &CommentResponse,
    parent_type: &str,
    parent_id: i32,
    csrf_token: Option<&str>,
) -> Markup {
    
    render_comment(comment, parent_type, parent_id, None, csrf_token)
}

pub fn render_comment_list_fragment(
    comments: &[CommentResponse],
    parent_type: &str,
    parent_id: i32,
    current_user: Option<&(i32, String, Option<String>)>,
    csrf_token: Option<&str>,
    has_more: bool,
    next_cursor: Option<String>
) -> Markup {
    html! {
        @for comment in comments {
            (render_comment(comment, parent_type, parent_id, current_user, csrf_token))
        }
        @if has_more {
            @if let Some(ref cursor) = next_cursor {
                div class="load-more-comments" id="load-more-comments" {
                    button
                        class="btn-load-more"
                        data-url=(format!(
                            "/{}s/{}/comments?cursor={}&limit=10",
                            parent_type, parent_id, cursor
                        ))
                    {
                        "Load more comments"
                    }
                }
            }
        }
    }
}

pub fn render_replies_fragment(
    replies: &[CommentResponse], 
    parent_type: &str,
    parent_id: i32,
    current_user: Option<&(i32, String, Option<String>)>,
    csrf_token: Option<&str>,
    has_more: bool,
    next_cursor: Option<String>,
    parent_comment_id: i32,
) -> Markup {
    html! {
        @for reply in replies {
            (render_comment(reply, parent_type, parent_id, current_user, csrf_token))
        }
        @if has_more {
            @if let Some(ref cursor) = next_cursor {
                div class="load-more-comments" id=(format!("load-more-replies-{}", parent_comment_id)) {
                    button
                        class="btn-load-more-replies"
                        data-comment-id=(parent_comment_id)
                        data-url=(format!(
                            "/comments/{}/replies?cursor={}",
                            parent_comment_id, cursor
                        ))
                    {
                        "Load more replies"
                    }
                }
            }
        }
    }    
}
