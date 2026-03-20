use axum::{
    extract::{State,Form},
    response::{Html, IntoResponse},
    http::StatusCode,
};
use serde::Deserialize;
use crate::{
    state::AppState,
    errors::Result,
    markdown::parse_markdown
};

#[derive(Debug, Deserialize)]
pub struct CharCountRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct AddTagRequest {
    #[serde(rename = "tag-input")]
    pub tag_input: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveTagRequest {
    pub tag_index: usize,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    pub content: String,
}

pub async fn get_char_count(
    Form(req): Form<CharCountRequest>,
) -> Html<String> {
    let count = req.content.len();
    Html(count.to_string())
}

pub async fn add_tag(
    Form(mut req): Form<AddTagRequest>,
) -> Result<Html<String>> {
    let tag = req.tag_input.trim().to_lowercase();

    if tag.is_empty() {
        return Ok(render_tags_html(&req.tags));
    }

    if tag.len() > 35 {
        return Ok(render_tags_html(&req.tags));
    }

    if req.tags.len() >= 5 {
        return Ok(render_tags_html(&req.tags));
    }

    if req.tags.contains(&tag) {
        return Ok(render_tags_html(&req.tags));
    }

    req.tags.push(tag);

    Ok(render_tags_html(&req.tags))
}

pub async fn remove_tag(
    Form(mut req): Form<RemoveTagRequest>,
) -> Html<String> {
    if req.tag_index < req.tags.len() {
        req.tags.remove(req.tag_index);
    }

    render_tags_html(&req.tags)
}

pub async fn preview_markdown(
    State(_state): State<AppState>,
    Form(req): Form<PreviewRequest>,
) -> Html<String> {
    if req.content.trim().is_empty() {
        return Html(r#"<p class="preview-empty"> Start writing to see preview...</p>"#.to_string());
    }

    let rendered = parse_markdown(&req.content);
    Html(rendered)
}

pub async fn close_preview() -> Html<String> {
    Html(crate::templates::create::render_closed_preview_modal().into_string())
}

fn render_tags_html(tags: &[String]) -> Html<String> {
    use maud::{html, Markup};

    let markup: Markup = html! {
        @for (i, tag) in tags.iter().enumerate() {
            span class="tag-pill" {
                (tag)
                button
                   type="button"
                   class="tag-remove"
                   hx-post="/api/remove-tag"
                   hx-vals=(format!(r#"{{"tag_index": {}}}"#, i))
                   hx-target="#tags-container"
                   hx-swap="innerHTML"
                {
                    "x"
                }
            }
            input type="hidden" name=(format!("tags[{}]", i)) value=(tag);
        }
    };

    Html(markup.into_string())
}