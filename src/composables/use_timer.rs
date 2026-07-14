//! Stopwatch-style timer composable.
//!
//! Uses `web-sys` `setInterval` directly. The interval id is stored in a
//! reactive `RwSignal` (which is `Copy`) so it can be shared across
//! callbacks without needing `Send` / `Sync` references.

use crate::core::utils;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::Window;

const TICK_MS: i32 = 1000;

/// A reactive timer: start, pause, reset and format helpers.
pub struct TimerHandle {
  /// Elapsed seconds (read-only signal).
  pub seconds: Signal<i64>,
  /// Whether the timer is running (read-only signal).
  pub running: ReadSignal<bool>,
  /// Start the timer (no-op when already running).
  pub start: Callback<()>,
  /// Pause the timer.
  pub pause: Callback<()>,
  /// Reset to zero and pause.
  pub reset: Callback<()>,
  /// Format seconds as `MM:SS`.
  pub format: Callback<i64, String>,
}

/// Returns a [`TimerHandle`]. The interval is automatically cleared on
/// cleanup.
#[must_use]
pub fn use_timer() -> TimerHandle {
  let seconds = RwSignal::new(0_i64);
  let running = RwSignal::new(false);
  let interval_id: RwSignal<Option<i32>> = RwSignal::new(None);

  let clear = move || {
    if let Some(id) = interval_id.get()
      && let Some(window) = window()
    {
      window.clear_interval_with_handle(id);
    }
    interval_id.set(None);
  };

  let start: Callback<()> = Callback::new(move |_| {
    if running.get() {
      return;
    }
    running.set(true);
    let seconds_signal = seconds;
    let cb = Closure::wrap(Box::new(move || {
      seconds_signal.update(|value| *value += 1);
    }) as Box<dyn FnMut()>);
    if let Some(window) = window() {
      if let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
        cb.as_ref().unchecked_ref(),
        TICK_MS,
      ) {
        cb.forget();
        interval_id.set(Some(id));
      } else {
        running.set(false);
      }
    } else {
      running.set(false);
    }
  });

  #[allow(clippy::redundant_locals)]
  let pause: Callback<()> = Callback::new({
    let clear = clear;
    move |_| {
      running.set(false);
      clear();
    }
  });

  #[allow(clippy::redundant_locals)]
  let reset: Callback<()> = Callback::new({
    let clear = clear;
    move |_| {
      running.set(false);
      seconds.set(0);
      clear();
    }
  });

  let format: Callback<i64, String> = Callback::new(|value: i64| utils::format_clock(value));

  on_cleanup(move || {
    running.set(false);
    clear();
  });

  TimerHandle {
    seconds: seconds.read_only().into(),
    running: running.read_only(),
    start,
    pause,
    reset,
    format,
  }
}

fn window() -> Option<Window> {
  web_sys::window()
}
