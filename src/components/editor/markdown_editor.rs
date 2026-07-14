//! Markdown editor with live preview and code highlighting.

use crate::components::common::base_button::{BaseButton, ButtonSize, ButtonVariant};
use crate::components::icons::{Eye, EyeOff, Save};
use crate::composables::use_markdown::use_markdown;
use leptos::prelude::*;

/// Markdown editor with write/preview tabs and a Save button.
#[component]
pub fn MarkdownEditor(
  /// Current note content.
  value: Signal<String>,
  /// Updates the note content.
  on_input: Callback<String>,
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
                          placeholder="Write your notes in Markdown..."
                          spellcheck="false"
                          aria-label="Markdown notes"
                          on:input=move |ev| {
                              let v = event_target_value(&ev);
                              on_input.run(v.clone());
                              let handle = handle;
                              handle.set_source(v);
                          }
                          prop:value=move || value.get()
                      ></textarea>
                  </div>

              </Show>
          </div>
      </div>
  }
}
