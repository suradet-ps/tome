//! Horizontal progress bar with optional status color.

use leptos::prelude::*;

/// A simple progress bar showing `completed / total`.
#[component]
pub fn ProgressBar(
  /// Number of completed units.
  #[prop(into)]
  completed: Signal<u32>,
  /// Total number of units.
  #[prop(into)]
  total: Signal<u32>,
  /// Whether to render the percentage label.
  #[prop(default = false)]
  show_label: bool,
) -> impl IntoView {
  let percent = move || -> u32 {
    let total = total.get();
    if total == 0 {
      0
    } else {
      ((completed.get() as f64 / total as f64) * 100.0).round() as u32
    }
  };

  view! {
      <div class="progress">
          <div class="progress__bar">
              <div
                  class="progress__fill"
                  style:width=move || format!("{}%", percent())
              ></div>
          </div>
          <Show when=move || show_label fallback=|| view! {}>
              <span class="progress__label numeric">{move || format!("{}%", percent())}</span>
          </Show>
      </div>
  }
}
