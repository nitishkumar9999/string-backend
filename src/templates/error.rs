use maud::{html, Markup, DOCTYPE, PreEscaped};

const CLOCK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><title xmlns="">clock</title><g fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="7"/><path stroke-linecap="round" d="M5.965 3.136a4 4 0 0 0-2.829 2.829m14.899-2.829a4 4 0 0 1 2.829 2.829M12 8v3.75c0 .138.112.25.25.25H15"/></g></svg>"#;
const LOCK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="2"><rect width="14" height="10" x="5" y="11" rx="2"/><path stroke-linecap="round" d="M8 11V7a4 4 0 0 1 8 0v4"/></g></svg>"#;
const SEARCH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path stroke-linecap="round" d="m20 20-3-3"/></g></svg>"#;
const WARNING: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><path stroke-linecap="round" d="M12 9v4m0 4h.01"/></g></svg>"#;

pub fn render_rate_limit_error(
    limit: &str,
    retry_after: i64,
    message: &str,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Rate Limit Exceeded - StringTechHub" }
                link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/error.css";
                script src="/static/error.js" defer {}
            }

            body {
                div class="error-card" {
                    div class="icon" { (PreEscaped(CLOCK)) } 
                    h1 { "Whoa, slow down!" }
                    p { (message) }
                    
                    div 
                        class="countdown" 
                        id="countdown"
                        data-seconds=(retry_after) 
                    {
                        (retry_after) "s"
                    }
                    
                    div class="limit-info" {
                        strong { "Rate limit: " }
                        (limit)
                    }
                    
                    a href="/feed" class="back-btn" { "← Back to Feed" }
                }
            }
        }
    }
}

pub fn render_404_error() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "404 - Not Found" }
                link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/error.css";
            }
            body {
                div class="error-card" {
                    div class="icon" { (PreEscaped(SEARCH)) }
                    h1 { "404 - Not Found" }
                    p { "The page you're looking for doesn't exist or has been moved." }
                    
                    a href="/feed" class="back-btn" { "← Back to Feed" }
                }
            }
        }
    }
}

// 403 Forbidden
pub fn render_403_error(message: &str) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "403 - Forbidden" }
                link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/error.css";
            }
            body {
                div class="error-card" {
                    div class="icon" { (PreEscaped(LOCK)) }
                    h1 { "Access Denied" }
                    p { (message) }
                    
                    a href="/feed" class="back-btn" { "← Back to Feed" }
                }
            }
        }
    }
}

// 401 Unauthorized
pub fn render_401_error() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Login Required" }
                link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/error.css";
            }
            body {
                div class="error-card" {
                    div class="icon" { (PreEscaped(LOCK)) }
                    h1 { "Login Required" }
                    p { "You need to be logged in to access this page." }
                    
                    a href="/auth/github" class="back-btn" { "Log in with GitHub" }
                }
            }
        }
    }
}

// 500 Internal Server Error
pub fn render_500_error() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "500 - Server Error" }
                link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/error.css";
            }
            body {
                div class="error-card" {
                    div class="icon" { (PreEscaped(WARNING)) }
                    h1 { "Something went wrong" }
                    p { "We're experiencing technical difficulties. Please try again later." }
                    
                    a href="/feed" class="back-btn" { "← Back to Feed" }
                }
            }
        }
    }
}