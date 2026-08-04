//! Online/offline connectivity state (root-scoped singleton).
//!
//! Listens to the browser's `online`/`offline` window events and exposes a
//! reactive `online` signal.  Also tracks a `pending_count` of writes waiting
//! to sync (incremented by stores when a write is queued, decremented after
//! flush).

use leptos::prelude::*;
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

static STATE: OnceLock<ConnectivityState> = OnceLock::new();

pub fn install() {
  let _ = STATE.set(ConnectivityState::new());
}

#[derive(Debug, Clone, Copy)]
pub struct ConnectivityState {
  /// Whether the browser reports itself as online.
  pub online: RwSignal<bool>,
  /// Number of writes queued locally, awaiting a sync to Supabase.
  pub pending_count: RwSignal<usize>,
}

impl ConnectivityState {
  fn new() -> Self {
    let online = RwSignal::new(navigator_on_line());
    let state = Self {
      online,
      pending_count: RwSignal::new(0),
    };

    // Register window event listeners for online/offline transitions.
    let online_sig = state.online;
    let on_online = Closure::wrap(Box::new(move || {
      online_sig.set(true);
    }) as Box<dyn FnMut()>);
    let on_offline = Closure::wrap(Box::new(move || {
      online_sig.set(false);
    }) as Box<dyn FnMut()>);

    if let Some(win) = web_sys::window() {
      let _ = win.add_event_listener_with_callback("online", on_online.as_ref().unchecked_ref());
      let _ = win.add_event_listener_with_callback("offline", on_offline.as_ref().unchecked_ref());
      on_online.forget();
      on_offline.forget();
    }

    state
  }

  pub fn use_ctx() -> Self {
    *STATE.get().expect("ConnectivityState not initialized")
  }

  /// Returns `true` when the browser is online.
  #[must_use]
  pub fn is_online(&self) -> bool {
    self.online.get_untracked()
  }

  /// Increment the pending count (called when a write is queued).
  pub fn inc_pending(&self) {
    self.pending_count.update(|n| *n += 1);
  }

  /// Decrement the pending count (called after a successful sync).
  pub fn dec_pending(&self) {
    self.pending_count.update(|n| {
      *n = n.saturating_sub(1);
    });
  }

  /// Reset the pending count to zero (e.g. after a full flush).
  pub fn clear_pending(&self) {
    self.pending_count.set(0);
  }
}

fn navigator_on_line() -> bool {
  web_sys::window()
    .map_or(true, |w| w.navigator().on_line())
}

/// Convenience re-export for components.
pub fn use_connectivity() -> ConnectivityState {
  ConnectivityState::use_ctx()
}
