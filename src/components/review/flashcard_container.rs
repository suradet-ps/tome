//! Flippable flashcard with three quality buttons (Hard / OK / Easy).

use crate::components::common::base_button::{BaseButton, ButtonSize, ButtonVariant};
use crate::components::icons::{Minus, ThumbsDown, ThumbsUp};
use crate::core::types::Flashcard;
use leptos::prelude::*;

/// Flippable flashcard component.
#[component]
pub fn FlashcardContainer(
  /// The card to display. Passing an owned `Flashcard` (rather than a signal)
  /// makes non-emptiness a compile-time guarantee: the caller can only render
  /// this component when it actually holds a card. A new card recreates the
  /// component, which resets the flip state for free.
  card: Flashcard,
  /// Emitted when the user rates the card. The quality values are:
  /// `1` (Hard), `3` (OK), `5` (Easy) — matching the SM-2 algorithm.
  on_rated: Callback<(uuid::Uuid, i32)>,
) -> impl IntoView {
  let flipped = RwSignal::new(false);
  let card_id = card.id;
  let front = card.front;
  let back = card.back;

  let flip = move |_| {
    flipped.update(|value| *value = !*value);
  };

  let rate = move |quality: i32| {
    flipped.set(false);
    on_rated.run((card_id, quality));
  };

  // Keyboard: Space/Enter flips (handled on the native button); 1/3/5 grade
  // once revealed. Graded via a document-level capture so it works without
  // moving focus to the buttons.
  let on_actions_keydown = move |ev: web_sys::KeyboardEvent| {
    if !flipped.get() {
      return;
    }
    let quality = match ev.key().as_str() {
      "1" => Some(1),
      "3" => Some(3),
      "5" => Some(5),
      _ => None,
    };
    if let Some(q) = quality {
      ev.prevent_default();
      rate(q);
    }
  };

  let card_keydown = move |ev: web_sys::KeyboardEvent| {
    if ev.key() == " " {
      ev.prevent_default();
      flipped.update(|value| *value = !*value);
    }
  };

  view! {
      <div class="flashcard">
          <button
              type="button"
              class="flashcard__card"
              class:is-flipped=move || flipped.get()
              aria-label=move || if flipped.get() { "Show question" } else { "Show answer" }
              aria-pressed=move || flipped.get().to_string()
              on:click=flip
              on:keydown=card_keydown
          >
              <div class="flashcard__face flashcard__face--front">
                  <span class="flashcard__label">"Question"</span>
                  <p class="flashcard__content">{front}</p>
                  <span class="flashcard__hint">"Click to reveal"</span>
              </div>
              <div class="flashcard__face flashcard__face--back">
                  <span class="flashcard__label flashcard__label--accent">"Answer"</span>
                  <p class="flashcard__content">{back}</p>
              </div>
          </button>

          <Show when=move || flipped.get() fallback=|| view! {}>
              <div class="flashcard__actions" on:keydown=on_actions_keydown>
                  <BaseButton
                      size=ButtonSize::Small
                      variant=ButtonVariant::Danger
                      on_click=move |_| rate(1)
                  >
                      <ThumbsDown size=13 />
                      "Hard"
                  </BaseButton>
                  <BaseButton
                      size=ButtonSize::Small
                      variant=ButtonVariant::Secondary
                      on_click=move |_| rate(3)
                  >
                      <Minus size=13 />
                      "OK"
                  </BaseButton>
                  <BaseButton
                      size=ButtonSize::Small
                      variant=ButtonVariant::Primary
                      on_click=move |_| rate(5)
                  >
                      <ThumbsUp size=13 />
                      "Easy"
                  </BaseButton>
              </div>
          </Show>
      </div>
  }
}
