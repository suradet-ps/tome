//! Markdown editor with live preview and code highlighting.

use crate::components::common::base_button::{BaseButton, ButtonSize, ButtonVariant};
use crate::components::icons::{Eye, EyeOff, Save};
use crate::composables::use_markdown::{LinePrefix, apply_line_prefix, use_markdown};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::HtmlTextAreaElement;

/// Pause between the last keystroke and an automatic save.
const AUTOSAVE_DELAY_MS: i32 = 1_500;

/// Why a save was requested — controls how the result is reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveSource {
  /// The reader pressed Save or Ctrl/Cmd + S.
  Manual,
  /// The debounced autosave fired after a pause in typing.
  Automatic,
}

/// Markdown editor with write/preview tabs, formatting shortcuts, and a
/// clear saved / dirty indicator. Unsaved changes are flushed automatically
/// after a short pause of typing.
#[component]
pub fn MarkdownEditor(
  /// Current note content.
  value: Signal<String>,
  /// Updates the note content.
  on_input: Callback<String>,
  /// Whether the note differs from what was last saved (reactive).
  #[prop(into)]
  dirty: Signal<bool>,
  /// Whether a save is in flight (reactive).
  #[prop(into)]
  saving: Signal<bool>,
  /// Save handler. The [`SaveSource`] lets the caller decide how loudly to
  /// report the result — an autosave shouldn't pop a toast next to the
  /// reader's cursor.
  on_save: Callback<SaveSource>,
) -> impl IntoView {
  let handle = use_markdown();
  // Initialise the composable source with the current value.
  handle.set_source(value.get_untracked());

  // Keep the composable source in sync with the parent signal.
  Effect::new(move |_| {
    let current = value.get();
    let source = handle.source();
    if current != source {
      handle.set_source(current);
    }
  });

  // Debounced autosave: each keystroke resets the timer; when it fires we
  // save only if the content is still the version we scheduled (i.e. the
  // user kept the same text and no save is already in flight). Clearing on
  // cleanup keeps a stale timer from saving a chapter we left.
  let autosave_handle: RwSignal<Option<i32>> = RwSignal::new(None);
  on_cleanup(move || {
    if let Some(handle) = autosave_handle.get_untracked()
      && let Some(win) = web_sys::window()
    {
      win.clear_timeout_with_handle(handle);
    }
  });
  let schedule_autosave = move || {
    if let Some(handle) = autosave_handle.get_untracked()
      && let Some(win) = web_sys::window()
    {
      win.clear_timeout_with_handle(handle);
      autosave_handle.set(None);
    }
    let Some(win) = web_sys::window() else {
      return;
    };
    let captured = value.get_untracked();
    let callback = Closure::wrap(Box::new(move || {
      if dirty.get_untracked() && value.get_untracked() == captured && !saving.get_untracked() {
        on_save.run(SaveSource::Automatic);
      }
    }) as Box<dyn FnMut()>);
    if let Ok(handle) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
      callback.as_ref().unchecked_ref(),
      AUTOSAVE_DELAY_MS,
    ) {
      callback.forget();
      autosave_handle.set(Some(handle));
    }
  };

  let set_preview = move |target: bool| {
    handle.set_preview(target);
  };

  // Arrow-key roving navigation across the Write/Preview tablist.
  let on_tabs_keydown = move |ev: web_sys::KeyboardEvent| {
    let current = handle.is_preview.get();
    let next = match ev.key().as_str() {
      "ArrowRight" | "ArrowDown" => Some(!current),
      "ArrowLeft" | "ArrowUp" => Some(!current),
      "Home" => Some(false),
      "End" => Some(true),
      _ => None,
    };
    if let Some(target) = next {
      ev.prevent_default();
      set_preview(target);
    }
  };

  // Formatting shortcuts (Ctrl/Cmd + 1/•/>): toggle a markdown prefix on the
  // current line. Pure logic lives in `apply_line_prefix`; here we just read
  // the caret, transform, and push back.
  let on_keydown = move |ev: web_sys::KeyboardEvent| {
    let is_mod = ev.ctrl_key() || ev.meta_key();
    if !is_mod {
      return;
    }
    let key = ev.key();
    // Ctrl/Cmd + S saves immediately and stops the browser's save dialog.
    if key.eq_ignore_ascii_case("s") {
      ev.prevent_default();
      on_save.run(SaveSource::Manual);
      return;
    }
    let prefix = match key.as_str() {
      "1" => Some(LinePrefix::Heading),
      "•" | "8" => Some(LinePrefix::Bullet), // • or Ctrl+8
      ">" | "." => Some(LinePrefix::Quote),
      _ => None,
    };
    let Some(prefix) = prefix else {
      return;
    };
    let target = event_target::<HtmlTextAreaElement>(&ev);
    let text = target.value();
    let caret: usize = target.selection_start().ok().flatten().unwrap_or(0) as usize;
    let (next, new_caret) = apply_line_prefix(&text, caret, prefix);
    on_input.run(next.clone());
    handle.set_source(next);
    schedule_autosave();
    // Restore the caret after Leptos re-renders the value.
    let _ = target.set_value(&handle.source());
    let _ = target.set_selection_range(new_caret as u32, new_caret as u32);
    ev.prevent_default();
  };

  let status_label = move || {
    if saving.get() {
      "Saving…"
    } else if dirty.get() {
      "Unsaved changes"
    } else {
      "Saved"
    }
  };
  let status_class = move || {
    if saving.get() {
      "editor__status editor__status--busy"
    } else if dirty.get() {
      "editor__status editor__status--dirty"
    } else {
      "editor__status editor__status--saved"
    }
  };

  view! {
      <div class="editor">
          <div class="editor__toolbar">
              <div class="editor__switch" role="tablist" aria-label="Editor mode" on:keydown=on_tabs_keydown>
                  <button
                      type="button"
                      role="tab"
                      class="editor__toggle"
                      class:is-active=move || !handle.is_preview.get()
                      aria-selected=move || (!handle.is_preview.get()).to_string()
                      tabindex=move || if handle.is_preview.get() { -1_i32 } else { 0_i32 }
                      on:click=move |_| set_preview(false)
                  >
                      <EyeOff size=13 />
                      "Write"
                  </button>
                  <button
                      type="button"
                      role="tab"
                      class="editor__toggle"
                      class:is-active=move || handle.is_preview.get()
                      aria-selected=move || handle.is_preview.get().to_string()
                      tabindex=move || if handle.is_preview.get() { 0_i32 } else { -1_i32 }
                      on:click=move |_| set_preview(true)
                  >
                      <Eye size=13 />
                      "Preview"
                  </button>
              </div>
              <div class="editor__status-group">
                  <span class=status_class aria-live="polite">{status_label}</span>
                  <BaseButton
                      size=ButtonSize::Small
                      variant=ButtonVariant::Primary
                      loading=saving
                      on_click=Callback::new(move |_: web_sys::MouseEvent| {
                          on_save.run(SaveSource::Manual)
                      })
                  >
                      <Save size=13 />
                      "Save"
                  </BaseButton>
              </div>
          </div>

          <div class="editor__body">
              <Show
                  when=move || !handle.is_preview.get()
                  fallback=move || view! {
                      <div
                          class="editor__panel editor__preview markdown-body"
                          role="tabpanel"
                          aria-label="Preview"
                          inner_html=move || handle.rendered.get()
                      ></div>
                  }
              >
                  <div class="editor__panel" role="tabpanel" aria-label="Write">
                      <textarea
                          class="editor__textarea"
                          placeholder="Write your notes in Markdown…"
                          spellcheck="false"
                          aria-label="Markdown notes"
                          on:keydown=on_keydown
                          on:input=move |ev| {
                              let v = event_target_value(&ev);
                              on_input.run(v.clone());
                              let handle = handle;
                              handle.set_source(v);
                              schedule_autosave();
                          }
                          prop:value=move || value.get()
                      ></textarea>
                      <p class="editor__hint">
                          "Changes are saved automatically as you write · "
                          <kbd>"Ctrl/Cmd + S"</kbd> " saves now, "
                          <kbd>"Ctrl/Cmd + 1"</kbd> " heading, "
                          <kbd>"•"</kbd> " list, "
                          <kbd>">"</kbd> " quote"
                      </p>
                  </div>

              </Show>
          </div>
      </div>
  }
}
