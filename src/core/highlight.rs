//! Tiny keyword-based code highlighter.
//!
//! It is deliberately lightweight — just enough to match the languages
//! registered in the original Vue implementation (JavaScript, TypeScript,
//! Rust, Python, Bash, JSON, HTML, XML, CSS, SQL). It does not try to
//! compete with full-blown lexers like `syntect` (which are too heavy for
//! the wasm bundle). The output is the same class-based markup that
//! `highlight.js` produces so the existing theme CSS keeps working.

use std::fmt::Write as _;

/// Whitelisted HTML tags produced by the renderer.
pub const ALLOWED_TAGS: &[&str] = &[
  "a",
  "abbr",
  "b",
  "blockquote",
  "br",
  "code",
  "del",
  "details",
  "div",
  "em",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "hr",
  "i",
  "img",
  "input",
  "ins",
  "kbd",
  "li",
  "mark",
  "ol",
  "p",
  "pre",
  "q",
  "s",
  "small",
  "span",
  "strong",
  "sub",
  "summary",
  "sup",
  "table",
  "tbody",
  "td",
  "th",
  "thead",
  "tr",
  "ul",
];

/// Whitelisted global attributes.
pub const ALLOWED_GENERIC_ATTRS: &[&str] = &["class", "id", "title", "dir", "lang", "role"];

/// Whitelisted per-tag attributes.
pub const ALLOWED_TAG_ATTRS: &[(&str, &[&str])] = &[
  ("a", &["href", "target", "name"]),
  ("img", &["src", "alt", "width", "height"]),
  ("input", &["type", "checked", "disabled"]),
  ("th", &["align"]),
  ("td", &["align"]),
  ("code", &["class"]),
  ("span", &["class"]),
];

/// Whitelisted URL schemes.
pub const ALLOWED_URL_SCHEMES: &[&str] = &["http", "https", "mailto", "data"];

/// Apply highlighting to a code block.
#[must_use]
pub fn highlight_code(code: &str, language: Option<&str>) -> String {
  let lang = language.unwrap_or("plaintext");
  let body = highlight(code, lang);
  let mut out = String::with_capacity(body.len() + 32);
  let _ = write!(
    &mut out,
    "<pre><code class=\"hljs language-{lang}\">{body}</code></pre>",
    lang = escape(lang),
    body = body
  );
  out
}

fn highlight(code: &str, lang: &str) -> String {
  let normalised = lang.to_ascii_lowercase();
  let mut result = String::with_capacity(code.len() * 2);
  for (idx, line) in code.lines().enumerate() {
    if idx > 0 {
      result.push('\n');
    }
    match normalised.as_str() {
      "js" | "javascript" | "jsx" | "ts" | "typescript" | "tsx" => {
        highlight_c_like(line, JS_KEYWORDS, &mut result);
      }
      "rust" | "rs" => highlight_c_like(line, RUST_KEYWORDS, &mut result),
      "py" | "python" => highlight_python(line, &mut result),
      "sh" | "bash" | "zsh" => highlight_bash(line, &mut result),
      "json" => highlight_json(line, &mut result),
      "html" | "xml" | "svg" => highlight_markup(line, &mut result),
      "css" => highlight_css(line, &mut result),
      "sql" => highlight_sql(line, &mut result),
      _ => push_plain(line, &mut result),
    }
  }
  result
}

fn push_plain(line: &str, out: &mut String) {
  escape_into(line, out);
}

fn highlight_c_like(line: &str, keywords: &[&str], out: &mut String) {
  let mut current = String::new();
  let mut chars = line.chars().peekable();
  let mut in_string: Option<char> = None;
  let mut in_line_comment = false;
  let mut in_block_comment = false;

  while let Some(c) = chars.next() {
    if in_line_comment {
      current.push(c);
      continue;
    }
    if in_block_comment {
      current.push(c);
      if c == '*' && matches!(chars.peek(), Some('/')) {
        current.push(chars.next().expect("peeked"));
        in_block_comment = false;
      }
      continue;
    }
    if let Some(quote) = in_string {
      current.push(c);
      if c == '\\' {
        if let Some(next) = chars.next() {
          current.push(next);
        }
        continue;
      }
      if c == quote {
        in_string = None;
      }
      continue;
    }
    if c == '/' && matches!(chars.peek(), Some('/')) {
      in_line_comment = true;
      current.push(c);
      continue;
    }
    if c == '/' && matches!(chars.peek(), Some('*')) {
      in_block_comment = true;
      current.push(c);
      current.push(chars.next().expect("peeked"));
      continue;
    }
    if c == '"' || c == '\'' {
      in_string = Some(c);
      current.push(c);
      continue;
    }
    if c.is_ascii_alphanumeric() || c == '_' || c == '@' {
      current.push(c);
      // Consume rest of identifier
      while let Some(&next) = chars.peek() {
        if next.is_ascii_alphanumeric() || next == '_' {
          current.push(chars.next().expect("peeked"));
        } else {
          break;
        }
      }
      if keywords.contains(&current.as_str()) {
        let _ = write!(
          out,
          "<span class=\"hljs-keyword\">{}</span>",
          escape(&current)
        );
      } else if is_number(&current) {
        let _ = write!(
          out,
          "<span class=\"hljs-number\">{}</span>",
          escape(&current)
        );
      } else {
        out.push_str(&escape(&current));
      }
      current.clear();
      continue;
    }
    current.push(c);
    if !current.is_empty() {
      out.push_str(&escape(&current));
      current.clear();
    }
  }
  if !current.is_empty() {
    if in_line_comment || in_block_comment {
      let _ = write!(
        out,
        "<span class=\"hljs-comment\">{}</span>",
        escape(&current)
      );
    } else if in_string.is_some() {
      let _ = write!(
        out,
        "<span class=\"hljs-string\">{}</span>",
        escape(&current)
      );
    } else {
      out.push_str(&escape(&current));
    }
  }
}

fn highlight_python(line: &str, out: &mut String) {
  highlight_c_like(line, PY_KEYWORDS, out);
}

fn highlight_bash(line: &str, out: &mut String) {
  let mut current = String::new();
  let mut chars = line.chars().peekable();
  let mut in_string: Option<char> = None;
  let mut is_command_start = true;

  while let Some(c) = chars.next() {
    if let Some(quote) = in_string {
      current.push(c);
      if c == '\\' {
        if let Some(next) = chars.next() {
          current.push(next);
        }
        continue;
      }
      if c == quote {
        in_string = None;
      }
      continue;
    }
    if c == '"' || c == '\'' {
      in_string = Some(c);
      current.push(c);
      continue;
    }
    if c == '#' {
      current.push(c);
      for ch in chars.by_ref() {
        current.push(ch);
      }
      let _ = write!(
        out,
        "<span class=\"hljs-comment\">{}</span>",
        escape(&current)
      );
      current.clear();
      continue;
    }
    if c.is_whitespace() {
      if !current.is_empty() {
        if is_command_start && BUILTIN_COMMANDS.contains(&current.as_str()) {
          let _ = write!(
            out,
            "<span class=\"hljs-built_in\">{}</span>",
            escape(&current)
          );
        } else {
          out.push_str(&escape(&current));
        }
        current.clear();
      }
      out.push(c);
      is_command_start = c.is_whitespace();
      continue;
    }
    current.push(c);
    is_command_start = false;
  }
  if !current.is_empty() {
    if is_command_start && BUILTIN_COMMANDS.contains(&current.as_str()) {
      let _ = write!(
        out,
        "<span class=\"hljs-built_in\">{}</span>",
        escape(&current)
      );
    } else {
      out.push_str(&escape(&current));
    }
  }
}

fn highlight_json(line: &str, out: &mut String) {
  let mut current = String::new();
  let mut chars = line.chars().peekable();
  let mut in_string = false;
  let mut after_colon = false;

  while let Some(c) = chars.next() {
    if in_string {
      current.push(c);
      if c == '\\' {
        if let Some(next) = chars.next() {
          current.push(next);
        }
        continue;
      }
      if c == '"' {
        let _ = write!(out, "<span class=\"hljs-attr\">{}</span>", escape(&current));
        current.clear();
        in_string = false;
      }
      continue;
    }
    if c == '"' {
      in_string = true;
      current.push(c);
      continue;
    }
    if c == ':' {
      if !current.is_empty() {
        let _ = write!(out, "<span class=\"hljs-attr\">{}</span>", escape(&current));
        current.clear();
      }
      out.push(':');
      after_colon = true;
      continue;
    }
    if c.is_ascii_digit() || c == '-' && after_colon {
      current.push(c);
      while let Some(&next) = chars.peek() {
        if next.is_ascii_digit()
          || next == '.'
          || next == 'e'
          || next == 'E'
          || next == '+'
          || next == '-'
        {
          current.push(chars.next().expect("peeked"));
        } else {
          break;
        }
      }
      let _ = write!(
        out,
        "<span class=\"hljs-number\">{}</span>",
        escape(&current)
      );
      current.clear();
      after_colon = false;
      continue;
    }
    if !current.is_empty() {
      out.push_str(&escape(&current));
      current.clear();
    }
    out.push(c);
    after_colon = false;
  }
  if !current.is_empty() {
    out.push_str(&escape(&current));
  }
}

fn highlight_markup(line: &str, out: &mut String) {
  let mut current = String::new();
  let chars = line.chars().peekable();
  let mut in_tag = false;
  let mut in_attr_string: Option<char> = None;

  for c in chars {
    if let Some(quote) = in_attr_string {
      current.push(c);
      if c == quote {
        in_attr_string = None;
      }
      continue;
    }
    if c == '<' {
      if !current.is_empty() {
        out.push_str(&escape(&current));
        current.clear();
      }
      in_tag = true;
      current.push(c);
      continue;
    }
    if c == '>' && in_tag {
      current.push(c);
      let _ = write!(out, "<span class=\"hljs-tag\">{}</span>", escape(&current));
      current.clear();
      in_tag = false;
      continue;
    }
    if in_tag && (c == '"' || c == '\'') {
      in_attr_string = Some(c);
      current.push(c);
      continue;
    }
    current.push(c);
  }
  if !current.is_empty() {
    out.push_str(&escape(&current));
  }
}

fn highlight_css(line: &str, out: &mut String) {
  let mut current = String::new();
  let mut chars = line.chars().peekable();
  let mut in_string: Option<char> = None;

  while let Some(c) = chars.next() {
    if let Some(quote) = in_string {
      current.push(c);
      if c == quote {
        in_string = None;
      }
      continue;
    }
    if c == '"' || c == '\'' {
      in_string = Some(c);
      current.push(c);
      continue;
    }
    if c.is_ascii_alphabetic() {
      current.push(c);
      while let Some(&next) = chars.peek() {
        if next.is_ascii_alphanumeric() || next == '-' || next == '_' {
          current.push(chars.next().expect("peeked"));
        } else {
          break;
        }
      }
      if looks_like_css_property(&current) {
        let _ = write!(
          out,
          "<span class=\"hljs-attribute\">{}</span>",
          escape(&current)
        );
      } else {
        let _ = write!(
          out,
          "<span class=\"hljs-keyword\">{}</span>",
          escape(&current)
        );
      }
      current.clear();
      continue;
    }
    if !current.is_empty() {
      out.push_str(&escape(&current));
      current.clear();
    }
    out.push(c);
  }
  if !current.is_empty() {
    out.push_str(&escape(&current));
  }
}

fn highlight_sql(line: &str, out: &mut String) {
  highlight_c_like(line, SQL_KEYWORDS, out);
}

fn looks_like_css_property(token: &str) -> bool {
  matches!(
    token,
    "color"
      | "background"
      | "background-color"
      | "font"
      | "font-size"
      | "font-weight"
      | "margin"
      | "padding"
      | "display"
      | "border"
      | "border-radius"
      | "width"
      | "height"
      | "gap"
      | "flex"
      | "grid"
      | "align-items"
      | "justify-content"
  )
}

fn is_number(token: &str) -> bool {
  if token.is_empty() {
    return false;
  }
  token
    .chars()
    .all(|c| c.is_ascii_digit() || c == '.' || c == '_')
    && token.chars().any(|c| c.is_ascii_digit())
}

fn escape(input: &str) -> String {
  let mut out = String::with_capacity(input.len());
  escape_into(input, &mut out);
  out
}

fn escape_into(input: &str, out: &mut String) {
  for ch in input.chars() {
    match ch {
      '&' => out.push_str("&amp;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      '"' => out.push_str("&quot;"),
      '\'' => out.push_str("&#39;"),
      _ => out.push(ch),
    }
  }
}

const JS_KEYWORDS: &[&str] = &[
  "as",
  "async",
  "await",
  "break",
  "case",
  "catch",
  "class",
  "const",
  "continue",
  "debugger",
  "default",
  "delete",
  "do",
  "else",
  "enum",
  "export",
  "extends",
  "false",
  "finally",
  "for",
  "from",
  "function",
  "if",
  "import",
  "in",
  "instanceof",
  "let",
  "new",
  "null",
  "of",
  "return",
  "static",
  "super",
  "switch",
  "this",
  "throw",
  "true",
  "try",
  "typeof",
  "undefined",
  "var",
  "void",
  "while",
  "with",
  "yield",
  "interface",
  "type",
  "namespace",
  "implements",
  "public",
  "private",
  "protected",
  "readonly",
];

const RUST_KEYWORDS: &[&str] = &[
  "as", "async", "await", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
  "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
  "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
  "where", "while", "dyn",
];

const PY_KEYWORDS: &[&str] = &[
  "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
  "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in",
  "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with",
  "yield",
];

const SQL_KEYWORDS: &[&str] = &[
  "SELECT",
  "FROM",
  "WHERE",
  "INSERT",
  "INTO",
  "VALUES",
  "UPDATE",
  "SET",
  "DELETE",
  "JOIN",
  "INNER",
  "LEFT",
  "RIGHT",
  "ON",
  "AS",
  "AND",
  "OR",
  "NOT",
  "NULL",
  "IS",
  "CREATE",
  "TABLE",
  "DROP",
  "ALTER",
  "ADD",
  "PRIMARY",
  "KEY",
  "FOREIGN",
  "REFERENCES",
  "INDEX",
  "GRANT",
  "REVOKE",
];

const BUILTIN_COMMANDS: &[&str] = &[
  "cd", "ls", "pwd", "echo", "cat", "mkdir", "rm", "cp", "mv", "touch", "grep", "sed", "awk",
  "curl", "wget", "git", "npm", "bun", "pnpm", "yarn", "cargo", "rustc", "trunk", "vercel", "node",
  "deno",
];

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn highlight_rust_keywords() {
    let html = highlight_code("fn main() {}", Some("rust"));
    assert!(html.contains("hljs-keyword"));
  }

  #[test]
  fn highlight_unknown_language_passthrough() {
    let html = highlight_code("hello", Some("klingon"));
    assert!(html.contains("<pre>"));
  }

  #[test]
  fn escape_preserves_characters() {
    assert_eq!(escape("<>&"), "&lt;&gt;&amp;");
  }
}
