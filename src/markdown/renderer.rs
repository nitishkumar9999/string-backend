use pulldown_cmark::{Parser, Options, Event, Tag, TagEnd, CowStr, CodeBlockKind, HeadingLevel};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;

pub struct MarkdownRenderer {
    html: String,
    in_code_block: bool,
    code_language: Option<String>,
    code_content: String,
    heading_count: usize,
    toc: Vec<TocEntry>,
    syntax_set: SyntaxSet,
    theme: Theme,
    in_image: bool,
    image_url: String,
    image_title: String,
    image_alt: String,
}

pub struct TocEntry {
    pub level: u8,
    pub text: String,
    pub id: String,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["InspiredGitHub"].clone();
        
        Self {
            html: String::new(),
            in_code_block: false,
            code_language: None,
            code_content: String::new(),
            heading_count: 0,
            toc: Vec::new(),
            syntax_set,
            theme,
            in_image: false,
            image_url: String::new(),
            image_title: String::new(),
            image_alt: String::new()
        }
    }

    pub fn render(mut self, markdown_input: &str) -> (String, Vec<TocEntry>) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        
        let parser = Parser::new_ext(markdown_input, options);
        
        for event in parser {
            self.process_event(event);
        }
        
        (self.html, self.toc)
    }

    fn process_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag_end) => self.end_tag(tag_end),
            Event::Text(text) => self.text(text),
            Event::Code(code) => self.inline_code(code),
            Event::Html(html) => self.html.push_str(&html),
            Event::SoftBreak => self.html.push('\n'),
            Event::HardBreak => self.html.push_str("<br>\n"),
            Event::Rule => self.html.push_str("<hr>\n"),
            Event::FootnoteReference(name) => {
                self.html.push_str(&format!(
                    "<sup class=\"footnote-reference\"><a href=\"#{}\">{}</a></sup>",
                    name,
                    name
                ));
            }
            Event::TaskListMarker(checked) => {
                let checked_attr = if checked { r#" checked="""# } else { "" };
                self.html.push_str(&format!(
                    r#"<input disabled="" type="checkbox"{checked_attr}> "#
                ));
            }
            _ => {}
        }
    }

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => self.html.push_str("<p>"),
            Tag::Heading { level, id, classes, attrs } => {
                self.heading_count += 1;
                let heading_id = id.as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("heading-{}", self.heading_count));
                
                let level_num = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                
                self.html.push_str(&format!(r#"<h{level_num} id="{heading_id}">"#));
            }
            Tag::CodeBlock(kind) => {
                self.in_code_block = true;
                self.code_content.clear();
                
                if let CodeBlockKind::Fenced(lang) = kind {
                    self.code_language = Some(lang.to_string());
                } else {
                    self.code_language = None;
                }
            }
            Tag::Strong => self.html.push_str("<strong>"),
            Tag::Emphasis => self.html.push_str("<em>"),
            Tag::Strikethrough => self.html.push_str("<del>"),
            Tag::Link { dest_url, title, .. } => {
                let title_attr = if title.is_empty() {
                    String::new()
                } else {
                    format!(r#" title="{title}""#)
                };
                self.html.push_str(&format!(r#"<a href="{dest_url}"{title_attr}>"#));
            }
            Tag::Image { dest_url, title, .. } => {
                self.in_image = true;
                self.image_url = dest_url.to_string();
                self.image_title = title.to_string();
                self.image_alt.clear();
            }
            Tag::List(None) => self.html.push_str("<ul>\n"),
            Tag::List(Some(_)) => self.html.push_str("<ol>\n"),
            Tag::Item => self.html.push_str("<li>"),
            Tag::Table(_) => self.html.push_str("<table>"),
            Tag::TableHead => self.html.push_str("<thead><tr>"),
            Tag::TableRow => self.html.push_str("<tr>"),
            Tag::TableCell => self.html.push_str("<td>"),
            Tag::BlockQuote(_) => self.html.push_str("<blockquote>\n"),
            Tag::FootnoteDefinition(name) => {
                self.html.push_str(&format!(
                    r#"<div class="footnote-definition" id="{0}"><sup class="footnote-definition-label">{0}</sup>"#,
                    name
                ));
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Paragraph => self.html.push_str("</p>\n"),
            TagEnd::Heading(level) => {
                let level_num = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                self.html.push_str(&format!("</h{level_num}>\n"));
            }
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                let lang = self.code_language.as_deref().unwrap_or("text");
                
                let highlighted = self.render_code_block(lang, &self.code_content);
                self.html.push_str(&highlighted);
                
                self.code_content.clear();
                self.code_language = None;
            }
            TagEnd::Strong => self.html.push_str("</strong>"),
            TagEnd::Emphasis => self.html.push_str("</em>"),
            TagEnd::Strikethrough => self.html.push_str("</del>"),
            TagEnd::Link => self.html.push_str("</a>"),
            TagEnd::Image => {
                self.in_image = false;
                let title_attr = if self.image_title.is_empty() {
                    String::new()
                } else {
                    format!(r#" title="{}""#, self.image_title)
                };
                self.html.push_str(&format!(
                    r#"<img src="{}" alt="{}"{}> "#,
                    self.image_url,
                    escape_html(&self.image_alt),
                    title_attr
                ));
            }
            TagEnd::List(false) => self.html.push_str("</ul>\n"),
            TagEnd::List(true) => self.html.push_str("</ol>\n"),
            TagEnd::Item => self.html.push_str("</li>\n"),
            TagEnd::Table => self.html.push_str("</table>\n"),
            TagEnd::TableHead => self.html.push_str("</tr></thead><tbody>\n"),
            TagEnd::TableRow => self.html.push_str("</tr>\n"),
            TagEnd::TableCell => self.html.push_str("</td>"),
            TagEnd::BlockQuote(_) => self.html.push_str("</blockquote>\n"),
            TagEnd::FootnoteDefinition => self.html.push_str("</div>\n"),
            _ => {}
        }
    }

    fn text(&mut self, text: CowStr) {
        if self.in_code_block {
            self.code_content.push_str(&text);
        } else if self.in_image {
            self.image_alt.push_str(&text);
        } else {
            self.html.push_str(&escape_html(&text));
        }
    }

    fn inline_code(&mut self, code: CowStr) {
        self.html.push_str(&format!("<code>{}</code>", escape_html(&code)));
    }

    fn render_code_block(&self, language: &str, code: &str) -> String {
        let syntax = self.syntax_set
            .find_syntax_by_token(language)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut html = String::new();
        
        html.push_str(&format!(
            r#"<div class="code-block-wrapper" data-language="{language}">
<div class="code-block-header">
    <span class="code-language">{language}</span>
    <button class="copy-button" data-action="copy-code">Copy</button>
</div>
<pre class="code-block"><code>"#
        ));

        for (line_num, line) in LinesWithEndings::from(code).enumerate() {
            let ranges = highlighter.highlight_line(line, &self.syntax_set).unwrap();
            let line_html = syntect::html::styled_line_to_highlighted_html(
                &ranges[..],
                syntect::html::IncludeBackground::No
            ).unwrap();
            
            html.push_str(&format!(
                r#"<span class="line-number">{}</span>{}"#,
                line_num + 1,
                line_html
            ));
        }

        html.push_str("</code></pre>\n</div>\n");
        html
    }
}

pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}