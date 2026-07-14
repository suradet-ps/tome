//! `<textarea>` wrapper with label and error state.

use leptos::prelude::*;

/// `<textarea>` wrapper used by the note editor and forms.
#[component]
pub fn BaseTextarea(
  /// Current value.
  #[prop(into)]
  value: Signal<String>,
  /// Update handler.
  on_input: Callback<String>,
  /// Label text.
  #[prop(optional, into)]
  label: Option<String>,
  /// Placeholder text.
  #[prop(optional, into)]
  placeholder: Option<String>,
  /// Optional error message.
  #[prop(optional, into)]
  error: Option<String>,
  /// Whether the textarea is disabled.
  #[prop(default = false)]
  disabled: bool,
  /// Number of visible rows.
  #[prop(default = 5)]
  rows: u32,
) -> impl IntoView {
  let id = format!("base-textarea-{}", uuid::Uuid::new_v4());
  let error_id = format!("{id}-error");
  let error_id_signal = Signal::derive(move || error_id.clone());
  let error_signal = Signal::derive(move || error.clone());
  let has_error = Signal::derive(move || error_signal.get().is_some_and(|e| !e.is_empty()));
  let error_message = Signal::derive(move || error_signal.get().unwrap_or_default());

  view! {
      <div class="textarea-group">
          {label
              .as_ref()
              .map(|label| view! { <label class="textarea-label" for=id.clone()>{label.clone()}</label> })}

          <textarea
              id=id.clone()
              rows=rows
              prop:value=move || value.get()
              on:input=move |ev| {
                  on_input.run(event_target_value(&ev));
              }
              placeholder=placeholder.unwrap_or_default()
              disabled=disabled
              class=move || if has_error.get() { "textarea-field textarea-field--error" } else { "textarea-field" }
              aria-invalid=move || if has_error.get() { "true" } else { "false" }
              aria-describedby=move || if has_error.get() { Some(error_id_signal.get()) } else { None }
          ></textarea>

          <Show when=move || has_error.get() fallback=|| view! { <span class="visually-hidden">""</span> }>
              <p id=error_id_signal.get() class="textarea-error">{error_message}</p>
          </Show>
      </div>
  }
}
