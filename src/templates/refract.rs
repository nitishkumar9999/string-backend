// src/templates/refract.rs
use maud::{html, Markup, PreEscaped};
use crate::handlers::posts::PostResponse;

const ICON_CLOSE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M18.3 5.71a.996.996 0 0 0-1.41 0L12 10.59 7.11 5.7A.996.996 0 1 0 5.7 7.11L10.59 12 5.7 16.89a.996.996 0 1 0 1.41 1.41L12 13.41l4.89 4.89a.996.996 0 1 0 1.41-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z"></path></svg>"#;   // fill in
const ICON_REFRACT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24"><path d="M17 5H6c-1.1 0-2 .9-2 2v5h2V7h11v3l5-4-5-4zm1 12H7v-3l-5 4 5 4v-3h11c1.1 0 2-.9 2-2v-5h-2z"></path></svg>"#;
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

// ============================================================================
// Refract modal — render once on post/question page
// ============================================================================

pub fn render_refract_modal(post: &PostResponse, csrf_token: Option<&str>) -> Markup {
    html! {
        div class="refract-modal-overlay" id="refract-modal-overlay" {
            div class="refract-modal" {

                // Header
                div class="refract-modal-header" {
                    h2 class="refract-modal-title" { "Add your commentary" }
                    button
                        class="refract-modal-close"
                        id="refract-modal-close"
                        aria-label="Close"
                    {
                        (PreEscaped(ICON_CLOSE))
                    }
                }

                // Body
                div class="refract-modal-body" {

                    // Original post preview
                    div class="original-post-preview" {
                        div class="original-post-header" {
                            div class="original-post-avatar" {
                                (post.username.chars().next().unwrap_or('?').to_uppercase().to_string())
                            }
                            div class="original-post-info" {
                                div class="original-post-author" { (post.username) }
                            }
                        }
                        @if let Some(ref title) = post.title {
                            div class="original-post-title" { (title) }
                        }
                        @if !post.tags.is_empty() {
                            div class="original-post-tags" {
                                @for tag in &post.tags {
                                    span class="original-post-tag" { (tag.name) }
                                }
                            }
                        }
                    }

                    // Commentary input
                    div class="refract-input-section" {
                        label class="refract-input-label" for="refract-textarea" {
                            (PreEscaped(ICON_REFRACT))
                            "Your Commentary"
                        }

                        textarea
                            class="refract-textarea"
                            id="refract-textarea"
                            placeholder="Share your thoughts, insights, or perspective on this post..."
                            maxlength="1500"
                        {}

                        // Toolbar
                        div class="editor-toolbar" {
                            div class="markdown-buttons" {
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

                                button type="button" class="markdown-btn" data-action="link" title="Link (Ctrl+K)" {
                                    (PreEscaped(ICON_LINK))
                                }
                                 
                                button type="button" class="markdown-btn" data-action="bullet-list" title="Bullet List" {
                                    (PreEscaped(ICON_BULLET_LIST))
                                }
                                button type="button" class="markdown-btn" data-action="quote" title="Quote" {
                                    (PreEscaped(ICON_QUOTE))
                                }
                               
                            }
                            div class="refract-toolbar-right" {
                                span class="refract-char-counter" id="refract-char-counter" { "0 / 1500" }
                                button type="button" class="btn-preview" id="refract-preview-btn" { "Preview" }
                            }
                        }

                        // Preview panel
                        div class="refract-preview-panel" id="refract-preview-panel" style="display:none" {
                            div class="refract-preview-label" { "PREVIEW" }
                            div class="refract-preview-content" id="refract-preview-content" {
                                p style="color:var(--text-secondary);font-style:italic" {
                                    "Start typing to see preview..."
                                }
                            }
                        }
                    }
                }

                // Footer — hidden fields + buttons
                div class="refract-modal-footer" {
                    input type="hidden" id="refract-post-id" value=(post.id);
                    input type="hidden" id="refract-csrf" value=(csrf_token.unwrap_or(""));
                    button type="button" class="btn-cancel" id="refract-cancel-btn" { "Cancel" }
                    button
                        type="button"
                        class="btn-refract"
                        id="refract-submit-btn"
                        disabled
                    {
                        (PreEscaped(ICON_REFRACT))
                        "Refract"
                    }
                }
            }
        }
    }
}

pub fn render_refract_modal_empty(csrf_token: Option<&str>) -> Markup {
    html! {
        div class="refract-modal-overlay" id="refract-modal-overlay" {
            div class="refract-modal" {
                // Header
                div class="refract-modal-header" {
                    h2 class="refract-modal-title" { "Add your commentary" }
                    button
                        class="refract-modal-close"
                        id="refract-modal-close"
                        aria-label="Close"
                    {
                        (PreEscaped(ICON_CLOSE))
                    }
                }

                // Body
                div class="refract-modal-body" {
                    // Commentary input
                    div class="refract-input-section" {
                        label class="refract-input-label" for="refract-textarea" {
                            (PreEscaped(ICON_REFRACT))
                            "Your Commentary"
                        }

                        textarea
                            class="refract-textarea"
                            id="refract-textarea"
                            placeholder="Share your thoughts, insights, or perspective on this post..."
                            maxlength="1500"
                        {}

                        // Toolbar
                        div class="refract-toolbar" {
                            div class="refract-md-buttons" {
                                button type="button" class="refract-md-btn" data-refract-action="bold" title="Bold" { (PreEscaped(ICON_BOLD)) }
                                button type="button" class="refract-md-btn" data-refract-action="italic" title="Italic" { (PreEscaped(ICON_ITALIC)) }
                                button type="button" class="refract-md-btn" data-refract-action="strikethrough" title="Strikethrough" { (PreEscaped(ICON_STRIKETHROUGH)) }
                                button type="button" class="refract-md-btn" data-refract-action="inline-code" title="Code" { (PreEscaped(ICON_INLINE_CODE)) }
                                button type="button" class="refract-md-btn" data-refract-action="link" title="Link" { (PreEscaped(ICON_LINK)) }
                                button type="button" class="refract-md-btn" data-refract-action="quote" title="Quote" { (PreEscaped(ICON_QUOTE)) }
                                button type="button" class="refract-md-btn" data-refract-action="bullet-list" title="List" { (PreEscaped(ICON_BULLET_LIST)) }
                            }
                            div class="refract-toolbar-right" {
                                span class="refract-char-counter" id="refract-char-counter" { "0 / 1500" }
                                button type="button" class="btn-preview" id="refract-preview-btn" { "Preview" }
                            }
                        }

                        // Preview panel
                        div class="refract-preview-panel" id="refract-preview-panel" style="display:none" {
                            div class="refract-preview-label" { "PREVIEW" }
                            div class="refract-preview-content" id="refract-preview-content" {
                                p style="color:var(--text-secondary);font-style:italic" {
                                    "Start typing to see preview..."
                                }
                            }
                        }
                    }
                }

                // Footer — hidden fields + buttons
                div class="refract-modal-footer" {
                    input type="hidden" id="refract-post-id" value="";  // ← Empty, JS will fill
                    input type="hidden" id="refract-csrf" value=(csrf_token.unwrap_or(""));
                    button type="button" class="btn-cancel" id="refract-cancel-btn" { "Cancel" }
                    button
                        type="button"
                        class="btn-refract"
                        id="refract-submit-btn"
                        disabled
                    {
                        "Refract"
                    }
                }
            }
        }
    }
}