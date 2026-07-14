//! Markdown -> sanitised HTML rendering using `pulldown-cmark` and
//! `ammonia`.

use crate::core::error::AppResult;
use crate::core::highlight::highlight_code;
use ammonia::Builder;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};
use std::fmt::Write as _;

/// Configuration for the markdown renderer.
#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
  /// GitHub-flavoured markdown features (tables, task lists, etc.).
  pub gfm: bool,
  /// Convert `\n` to `<br>` inside paragraphs.
  pub hard_breaks: bool,
}

impl Default for RenderConfig {
  fn default() -> Self {
    Self {
      gfm: true,
      hard_breaks: true,
    }
  }
}

/// Render `input` as sanitised HTML.
#[must_use]
pub fn render_markdown(input: &str) -> String {
  let config = RenderConfig::default();
  let mut options = Options::empty();
  if config.gfm {
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
  }
  let raw = render_to_html(input, options);
  sanitise(&raw)
}

/// Convert the markdown source to HTML while running fenced code blocks
/// through our highlighter.
fn render_to_html(input: &str, options: Options) -> String {
  let parser = Parser::new_ext(input, options);
  let mut events: Vec<Event<'_>> = Vec::with_capacity(input.len() / 4);
  let mut in_code = false;
  let mut code_buf = String::new();
  let mut code_lang: Option<String> = None;

  for event in parser {
    match event {
      Event::Start(Tag::CodeBlock(kind)) => {
        in_code = true;
        code_buf.clear();
        code_lang = match kind {
          CodeBlockKind::Indented => None,
          CodeBlockKind::Fenced(lang) => {
            if lang.is_empty() {
              None
            } else {
              Some(lang.to_string())
            }
          }
        };
      }
      Event::End(TagEnd::CodeBlock) => {
        in_code = false;
        let html = highlight_code(&code_buf, code_lang.as_deref());
        events.push(Event::Html(html.into()));
      }
      Event::Text(text) if in_code => {
        code_buf.push_str(&text);
      }
      other => events.push(other),
    }
  }

  let mut output = String::with_capacity(input.len() * 2);
  html::push_html(&mut output, events.into_iter());
  output
}

/// Sanitise HTML with a strict tag/attribute allowlist.
fn sanitise(input: &str) -> String {
  use std::collections::HashSet;
  let mut builder = Builder::default();
  let tags: HashSet<&str> = super::highlight::ALLOWED_TAGS.iter().copied().collect();
  let generic: HashSet<&str> = super::highlight::ALLOWED_GENERIC_ATTRS
    .iter()
    .copied()
    .collect();
  let mut attributes = std::collections::HashMap::<&str, HashSet<&str>>::new();
  for (tag, attrs) in super::highlight::ALLOWED_TAG_ATTRS {
    let set: HashSet<&str> = attrs.iter().copied().collect();
    attributes.insert(tag, set);
  }
  let schemes: HashSet<&str> = super::highlight::ALLOWED_URL_SCHEMES
    .iter()
    .copied()
    .collect();
  builder
    .tags(tags)
    .generic_attributes(generic)
    .tag_attributes(attributes)
    .url_schemes(schemes)
    .link_rel(Some("noopener noreferrer nofollow"))
    .add_generic_attributes(&["class", "id", "aria-label", "role"]);
  builder.clean(input).to_string()
}

/// Truncate a markdown source for use in `<title>` or other small surfaces.
#[must_use]
pub fn plain_summary(input: &str, max_chars: usize) -> String {
  let mut summary = String::new();
  let mut in_code = false;
  for line in input.lines() {
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    if line.starts_with("```") {
      in_code = !in_code;
      continue;
    }
    if in_code {
      continue;
    }
    if !summary.is_empty() {
      summary.push(' ');
    }
    summary.push_str(line);
    if summary.chars().count() >= max_chars {
      break;
    }
  }
  if summary.chars().count() > max_chars {
    let mut s: String = summary.chars().take(max_chars).collect();
    s.push('…');
    s
  } else {
    summary
  }
}

/// HTML-escape a string for safe interpolation.
#[must_use]
pub fn escape_html(input: &str) -> String {
  let mut escaped = String::with_capacity(input.len());
  for ch in input.chars() {
    match ch {
      '&' => escaped.push_str("&amp;"),
      '<' => escaped.push_str("&lt;"),
      '>' => escaped.push_str("&gt;"),
      '"' => escaped.push_str("&quot;"),
      '\'' => escaped.push_str("&#39;"),
      _ => escaped.push(ch),
    }
  }
  escaped
}

/// Render markdown to HTML and return a [`Result`] for callers that want
/// to surface errors (e.g. tests).
pub fn try_render(input: &str) -> AppResult<String> {
  Ok(render_markdown(input))
}

/// Helper used by templates: writes escaped text to a buffer.
pub fn write_escaped(out: &mut String, input: &str) -> std::fmt::Result {
  for ch in input.chars() {
    match ch {
      '&' => out.write_str("&amp;")?,
      '<' => out.write_str("&lt;")?,
      '>' => out.write_str("&gt;")?,
      '"' => out.write_str("&quot;")?,
      '\'' => out.write_str("&#39;")?,
      _ => out.write_char(ch)?,
    }
  }
  Ok(())
}
