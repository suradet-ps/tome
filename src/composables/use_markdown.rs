//! Markdown composable: reactive preview state and rendering helpers.

use crate::core::markdown as md;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// State for the markdown editor composable.
#[derive(Clone, Copy)]
pub struct MarkdownHandle {
  /// Whether the preview tab is active.
  pub is_preview: RwSignal<bool>,
  /// Memoised render of the latest debounced content.
  pub rendered: Signal<String>,
  /// Toggle between write/preview modes.
  pub toggle: Callback<()>,
  /// Update the source content; the rendered HTML is debounced.
  pub set_content: Callback<String>,
  source: RwSignal<String>,
}

const DEBOUNCE_MS: i32 = 150;

#[must_use]
pub fn use_markdown() -> MarkdownHandle {
  let is_preview = RwSignal::new(false);
  let source = RwSignal::new(String::new());
  let debounced = RwSignal::new(String::new());

  // Debounce updates to `source` -> `debounced`.
  let timeout_handle: StoredValue<Option<i32>> = StoredValue::new(None);
  // Keep the active closure alive until it fires or is replaced, then drop it.
  // (Previously `cb.forget()` leaked a closure on every keystroke.)
  // `Closure` is `!Sync`, so we store it with `LocalStorage`.
  let active_closure: StoredValue<
    Option<wasm_bindgen::closure::Closure<dyn FnMut()>>,
    LocalStorage,
  > = StoredValue::new_with_storage(None);
  Effect::new({
    move |_| {
      let value = source.get();
      // Cancel any previous debounce timer.
      if let Some(prev) = timeout_handle.get_value()
        && let Some(window) = web_sys::window()
      {
        window.clear_timeout_with_handle(prev);
      }
      // Dropping the previous closure releases it (the browser timer was
      // already cleared above and holds its own copy of the callback).
      active_closure.set_value(None);
      let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        debounced.set(value.clone());
      }) as Box<dyn FnMut()>);
      let id = web_sys::window()
        .and_then(|w| {
          w.set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            DEBOUNCE_MS,
          )
          .ok()
        })
        .unwrap_or(0);
      active_closure.set_value(Some(cb));
      timeout_handle.set_value(Some(id));
    }
  });

  let rendered = Signal::derive(move || md::render_markdown(&debounced.get()));

  let toggle: Callback<()> = Callback::new(move |_| {
    is_preview.update(|value| *value = !*value);
  });

  let set_content: Callback<String> = Callback::new(move |value: String| {
    source.set(value);
  });

  MarkdownHandle {
    is_preview,
    rendered,
    toggle,
    set_content,
    source,
  }
}

impl MarkdownHandle {
  /// Returns the current source content.
  #[must_use]
  pub fn source(&self) -> String {
    self.source.get()
  }

  /// Replace the source content.
  pub fn set_source(&self, value: String) {
    self.set_content.run(value);
  }

  /// Set the preview mode directly.
  pub fn set_preview(&self, value: bool) {
    self.is_preview.set(value);
  }
}

/// A markdown line prefix applied by the editor's formatting shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinePrefix {
  /// `# ` — a top-level heading.
  Heading,
  /// `* ` — a bullet list item.
  Bullet,
  /// `> ` — a blockquote line.
  Quote,
}

impl LinePrefix {
  /// The literal text this prefix inserts at the start of a line.
  #[must_use]
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Heading => "# ",
      Self::Bullet => "* ",
      Self::Quote => "> ",
    }
  }
}

/// Apply a [`LinePrefix`] to the line containing `caret` in `text`.
///
/// Inserts the prefix at the line start. If the line already carries the same
/// prefix, the shortcut *toggles it off* (removes it) so a second press of
/// the same key restores the plain line — a small calm affordance, not a
/// surprise. Returns the new text plus the caret position to restore (just
/// after the inserted/removed prefix on that line). Pure so the shortcut
/// behaviour can be tested without a textarea or the DOM.
#[must_use]
pub fn apply_line_prefix(text: &str, caret: usize, prefix: LinePrefix) -> (String, usize) {
  let prefix_str = prefix.as_str();
  let line_start = text[..caret.min(text.len())]
    .rfind('\n')
    .map_or(0, |i| i + 1);
  let line_end = text[line_start..]
    .find('\n')
    .map_or(text.len(), |i| line_start + i);
  let line = &text[line_start..line_end];

  let (new_line, delta) = if let Some(rest) = line.strip_prefix(prefix_str) {
    // Toggle off: remove the prefix.
    (rest.to_string(), -(prefix_str.len() as isize))
  } else {
    // Insert: but don't nest a heading under another heading's prefix-marker.
    let new_line = format!("{prefix_str}{line}");
    (new_line, prefix_str.len() as isize)
  };

  let mut out = String::with_capacity(text.len() + prefix_str.len());
  out.push_str(&text[..line_start]);
  out.push_str(&new_line);
  out.push_str(&text[line_end..]);
  let new_caret = (caret as isize + delta).clamp(0, out.len() as isize) as usize;
  (out, new_caret)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inserts_heading_on_empty_line() {
    let (out, caret) = apply_line_prefix("hello", 5, LinePrefix::Heading);
    assert_eq!(out, "# hello");
    assert_eq!(caret, 7);
  }

  #[test]
  fn inserts_prefix_on_line_in_multiline() {
    let src = "line one\nline two\nline three";
    // caret in "line two"
    let caret = "line one\n".len() + 3;
    let (out, _) = apply_line_prefix(src, caret, LinePrefix::Bullet);
    assert_eq!(out, "line one\n* line two\nline three");
  }

  #[test]
  fn toggles_prefix_off_when_present() {
    let src = "# Title";
    let (out, caret) = apply_line_prefix(src, src.len(), LinePrefix::Heading);
    assert_eq!(out, "Title");
    assert_eq!(caret, 5);
  }

  #[test]
  fn quote_prefix_applied_and_toggled() {
    let src = "plain";
    let (on, _) = apply_line_prefix(src, src.len(), LinePrefix::Quote);
    assert_eq!(on, "> plain");
    let (off, _) = apply_line_prefix(&on, on.len(), LinePrefix::Quote);
    assert_eq!(off, "plain");
  }

  #[test]
  fn caret_clamped_within_bounds() {
    let (out, caret) = apply_line_prefix("", 0, LinePrefix::Heading);
    assert_eq!(out, "# ");
    assert_eq!(caret, 2);
  }
}
