//! Markdown editor with live preview and code highlighting.

use crate::components::common::base_button::{BaseButton, ButtonSize, ButtonVariant};
use crate::components::icons::{Eye, EyeOff, Save};
use crate::composables::use_markdown::{LinePrefix, apply_line_prefix, use_markdown};
use leptos::prelude::*;
use web_sys::HtmlTextAreaElement;

/// Markdown editor with write/preview tabs, formatting shortcuts, and a
/// clear saved / dirty indicator.
#[component]
pub fn MarkdownEditor(
  /// Current note content.
  value: Signal<String>,
  /// Updates the note content.
  on_input: Callback<String>,
  /// Whether the note differs from what was last saved.
  #[prop(default = false)]
  dirty: bool,
  /// Whether a save is in flight.
  #[prop(default = false)]
  saving: bool,
  /// Save handler.
  on_save: Callback<()>,
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

  let set_preview = move |target: bool| {
    handle.set_preview(target);
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
    // Restore the caret after Leptos re-renders the value.
    let _ = target.set_value(&handle.source());
    let _ = target.set_selection_range(new_caret as u32, new_caret as u32);
    ev.prevent_default();
  };

  let status_label = move || {
    if saving {
      "Saving…"
    } else if dirty {
      "Unsaved changes"
    } else {
      "Saved"
    }
  };
  let status_class = move || {
    if saving {
      "editor__status editor__status--busy"
    } else if dirty {
      "editor__status editor__status--dirty"
    } else {
      "editor__status editor__status--saved"
    }
  };

  view! {
      <div class="editor">
          <div class="editor__toolbar">
              <div class="editor__switch" role="tablist" aria-label="Editor mode">
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
                      on_click=Callback::new(move |_: web_sys::MouseEvent| on_save.run(()))
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
                          }
                          prop:value=move || value.get()
                      ></textarea>
                      <p class="editor__hint">
                          "Tip: "
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
