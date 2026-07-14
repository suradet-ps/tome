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
