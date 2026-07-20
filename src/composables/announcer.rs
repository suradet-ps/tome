//! A tiny global announcer for screen-reader live-region messages.
//!
//! Call `announce(msg)` from any async result handler to surface a polite
//! spoken announcement (e.g. "Note saved", "Card reviewed") without a visible
//! toast. A single `aria-live="polite"` region in the app shell renders the
//! latest message.

use leptos::prelude::*;

thread_local! {
    static MESSAGE: RwSignal<String> = RwSignal::new(String::new());
}

/// Push a message to the screen-reader live region.
pub fn announce(message: impl Into<String>) {
  MESSAGE.with(|signal| signal.set(message.into()));
}

/// Read the current announcement (used by the rendered live region).
pub fn current_message() -> String {
  MESSAGE.with(|signal| signal.get_untracked())
}

/// The live-region node. Render exactly once near the app root.
#[component]
pub fn Announcer() -> impl IntoView {
  let message = MESSAGE.with(|signal| signal.read_only());
  view! {
      <div class="visually-hidden" role="status" aria-live="polite" aria-atomic="true">
          {message}
      </div>
  }
}
