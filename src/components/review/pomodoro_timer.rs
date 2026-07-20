//! Pomodoro-style countdown timer with three modes.

use crate::components::common::base_button::{BaseButton, ButtonSize, ButtonVariant};
use crate::components::icons::{Pause, Play, RotateCcw};
use crate::composables::use_timer::use_timer;
use leptos::prelude::*;

const FOCUS_SECONDS: i64 = 25 * 60;
const SHORT_BREAK_SECONDS: i64 = 5 * 60;
const LONG_BREAK_SECONDS: i64 = 15 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mode {
  Focus,
  Short,
  Long,
}

impl Mode {
  const fn label(self) -> &'static str {
    match self {
      Self::Focus => "Focus",
      Self::Short => "Short",
      Self::Long => "Long",
    }
  }
  const fn duration(self) -> i64 {
    match self {
      Self::Focus => FOCUS_SECONDS,
      Self::Short => SHORT_BREAK_SECONDS,
      Self::Long => LONG_BREAK_SECONDS,
    }
  }
}

const MODES: [Mode; 3] = [Mode::Focus, Mode::Short, Mode::Long];

/// Pomodoro-style timer.
#[component]
pub fn PomodoroTimer() -> impl IntoView {
  let mode = RwSignal::new(Mode::Focus);
  let remaining = RwSignal::new(Mode::Focus.duration());
  let handle = use_timer();

  // Keep `remaining` in sync with the composable.
  Effect::new(move |_| {
    remaining.set(handle.seconds.get());
  });

  // Detect end of session.
  Effect::new(move |_| {
    if handle.seconds.get() == 0 && !handle.running.get() {
      // Pause triggered by user.
    }
  });

  let set_mode = move |next: Mode| {
    if mode.get() == next {
      return;
    }
    if handle.running.get() || handle.seconds.get() < mode.get().duration() {
      let confirmed = web_sys::window()
        .and_then(|w| {
          w.confirm_with_message("Switching modes will end the current session. Continue?")
            .ok()
        })
        .unwrap_or(false);
      if !confirmed {
        return;
      }
    }
    handle.reset.run(());
    mode.set(next);
    remaining.set(next.duration());
  };

  let display = Signal::derive(move || handle.format.run(remaining.get()));

  let total = Signal::derive(move || mode.get().duration());
  let progress_percent = move || {
    let total = total.get();
    if total == 0 {
      100.0
    } else {
      ((total - remaining.get()) as f64 / total as f64) * 100.0
    }
  };

  let toggle = move |_| {
    if handle.running.get() {
      handle.pause.run(());
    } else {
      handle.start.run(());
    }
  };

  // Arrow-key roving navigation across the timer mode tablist.
  let on_modes_keydown = move |ev: web_sys::KeyboardEvent| {
    let current = mode.get();
    let idx = MODES.iter().position(|m| *m == current).unwrap_or(0);
    let next = match ev.key().as_str() {
      "ArrowRight" | "ArrowDown" => Some(MODES[(idx + 1) % MODES.len()]),
      "ArrowLeft" | "ArrowUp" => Some(MODES[(idx + MODES.len() - 1) % MODES.len()]),
      "Home" => Some(MODES[0]),
      "End" => Some(MODES[MODES.len() - 1]),
      _ => None,
    };
    if let Some(target) = next {
      ev.prevent_default();
      set_mode(target);
    }
  };

  let reset = move |_| {
    handle.reset.run(());
    remaining.set(mode.get().duration());
  };

  view! {
      <div class="pomodoro">
          <div class="pomodoro__modes" role="tablist" aria-label="Timer mode" on:keydown=on_modes_keydown>
              <For
                  each=move || MODES.iter().copied()
                  key=|m| *m
                  children=move |m: Mode| {
                      let m_for_click = m;
                      view! {
                          <button
                              type="button"
                              role="tab"
                              class="pomodoro__mode"
                              class:is-active=move || mode.get() == m_for_click
                              aria-selected=move || (mode.get() == m_for_click).to_string()
                              tabindex=move || if mode.get() == m_for_click { 0_i32 } else { -1_i32 }
                              on:click=move |_| set_mode(m_for_click)
                          >
                              {m_for_click.label()}
                          </button>
                      }
                  }
              />
          </div>

          <div class="pomodoro__clock" role="timer" aria-label=move || format!("{} remaining", display.get())>
              <svg class="pomodoro__ring" viewBox="0 0 120 120" aria-hidden="true">
                  <circle
                      cx="60"
                      cy="60"
                      r="54"
                      fill="none"
                      stroke="var(--color-hairline)"
                      stroke-width="4"
                  />
                  <circle
                      cx="60"
                      cy="60"
                      r="54"
                      fill="none"
                      stroke="var(--color-primary)"
                      stroke-width="4"
                      stroke-linecap="round"
                      stroke-dasharray=move || format!("{}", 2.0 * std::f64::consts::PI * 54.0)
                      stroke-dashoffset=move || format!("{}", 2.0 * std::f64::consts::PI * 54.0 * (1.0 - progress_percent() / 100.0))
                      transform="rotate(-90 60 60)"
                      class="pomodoro__progress"
                  />
              </svg>
              <span class="pomodoro__time numeric">{display}</span>
          </div>

          <div class="pomodoro__controls">
              <button
                  class="pomodoro__icon"
                  type="button"
                  on:click=reset
                  title="Reset"
                  aria-label="Reset timer"
              >
                  <RotateCcw size=16 />
              </button>
              <BaseButton
                  size=ButtonSize::Medium
                  variant=ButtonVariant::Primary
                  on_click=toggle
              >
                  <Show
                      when=move || handle.running.get()
                      fallback=move || view! { <Play size=14 /> }
                  >
                      <Pause size=14 />
                  </Show>
                  {move || if handle.running.get() { "Pause" } else { "Start" }}
              </BaseButton>
          </div>
      </div>
  }
}
