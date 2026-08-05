//! Calm offline-indicator banner shown at the top of the viewport.
//!
//! Distinguishes "offline" from "request failed" — a quiet bar, not a toast
//! storm.  Hidden when online and nothing is pending.

use crate::stores::connectivity::use_connectivity;
use leptos::prelude::*;

#[component]
pub fn OfflineBanner() -> impl IntoView {
  let conn = use_connectivity();
  let is_online = conn.online;
  let pending = conn.pending_count;

  let should_show = move || !is_online.get() || pending.get() > 0;

  view! {
      <Show when=should_show>
          <div class="offline-banner" role="status" aria-live="polite">
              <Show
                when=move || !is_online.get()
                fallback=move || view! {
                  <span class="offline-banner__text">
                      "Syncing " {move || pending.get().to_string()} " pending…"
                  </span>
                }
              >
                  <span class="offline-banner__text">
                      "You're offline — changes are saved locally"
                  </span>
              </Show>
          </div>
      </Show>
  }
}
