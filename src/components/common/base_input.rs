//! Labelled `<input>` with error state.

use leptos::prelude::*;

/// `<input>` wrapper that provides a label, error and a unique id.
#[component]
pub fn BaseInput(
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
    /// Input type (`text`, `email`, `password`, ...).
    #[prop(default = "text")]
    input_type: &'static str,
    /// Optional error message.
    #[prop(optional, into)]
    error: Option<String>,
    /// Whether the input is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Optional `inputmode` attribute.
    #[prop(optional)]
    inputmode: Option<&'static str>,
) -> impl IntoView {
    let id = format!("base-input-{}", uuid::Uuid::new_v4());
    let error_id = format!("{id}-error");
    let error_id_signal = Signal::derive(move || error_id.clone());
    let error_signal = Signal::derive(move || error.clone());
    let has_error = Signal::derive(move || error_signal.get().is_some_and(|e| !e.is_empty()));
    let error_message = Signal::derive(move || error_signal.get().unwrap_or_default());

    view! {
        <div class="input-group">
            {label
                .as_ref()
                .map(|label| view! { <label class="input-label" for=id.clone()>{label.clone()}</label> })}

            <input
                id=id.clone()
                type=input_type
                value=move || value.get()
                on:input=move |ev| {
                    on_input.run(event_target_value(&ev));
                }
                placeholder=placeholder.unwrap_or_default()
                disabled=disabled
                inputmode=inputmode.unwrap_or("")
                class=move || if has_error.get() { "input-field input-field--error" } else { "input-field" }
                aria-invalid=move || if has_error.get() { "true" } else { "false" }
                aria-describedby=move || if has_error.get() { Some(error_id_signal.get()) } else { None }
            />

            <Show when=move || has_error.get() fallback=|| view! { <span class="visually-hidden">""</span> }>
                <p id=error_id_signal.get() class="input-error">{error_message}</p>
            </Show>
        </div>
    }
}
