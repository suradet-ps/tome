//! Review view: flashcards and Pomodoro timer.

use crate::components::common::base_button::{BaseButton, ButtonSize, ButtonVariant};
use crate::components::common::base_input::BaseInput;
use crate::components::common::base_loader::BaseLoader;
use crate::components::common::base_modal::BaseModal;
use crate::components::common::base_textarea::BaseTextarea;
use crate::components::icons::{Brain, CheckCheck, Plus, clock3 as Clock3};
use crate::components::review::flashcard_container::FlashcardContainer;
use crate::components::review::pomodoro_timer::PomodoroTimer;
use crate::core::supabase;
use crate::core::time::now_iso;
use crate::core::types::Flashcard;
use crate::stores::auth::use_auth;
use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
  Cards,
  Timer,
}

/// Review page.
#[component]
pub fn ReviewView() -> impl IntoView {
  let auth = use_auth();
  let cards: RwSignal<Vec<Flashcard>> = RwSignal::new(Vec::new());
  let loading = RwSignal::new(false);
  let reviewed: RwSignal<usize> = RwSignal::new(0);
  let show_add_modal = RwSignal::new(false);
  let new_front = RwSignal::new(String::new());
  let new_back = RwSignal::new(String::new());
  let adding = RwSignal::new(false);
  let error = RwSignal::new(String::new());
  let active_tab = RwSignal::new(Tab::Cards);

  let disposed = RwSignal::new(false);
  on_cleanup(move || disposed.set(true));

  let config_message: Option<String> = supabase::supabase_config_error();
  let config_message_for_show = config_message.clone();
  let config_message_text = Signal::derive(move || config_message.clone().unwrap_or_default());

  let load_cards = move || {
    let user = auth.user.get();
    if user.is_none() || supabase::supabase_config_error().is_some() {
      cards.set(Vec::new());
      return;
    }
    let user = user.expect("checked above");
    loading.set(true);
    error.set(String::new());
    leptos::task::spawn_local(async move {
      let result: Result<(), String> = async {
        let client = supabase::supabase().map_err(|e| e.to_string())?;
        let fetched: Vec<Flashcard> = client
          .postgrest()
          .from("reading_flashcards")
          .select("*")
          .eq("user_id", user.to_string())
          .lte("next_review", now_iso())
          .order("next_review", true)
          .range(0, 999)
          .get()
          .await
          .map_err(|e| e.to_string())?;
        cards.set(fetched);
        Ok::<(), String>(())
      }
      .await;
      if !disposed.get_untracked() {
        loading.set(false);
      }
      if let Err(err) = result {
        if !disposed.get_untracked() {
          error.set(err);
        }
      }
    });
  };

  Effect::new(move |_| {
    let _ = auth.user.get();
    load_cards();
  });

  let handle_rated = move |(card_id, quality): (uuid::Uuid, i32)| {
    let Some(target) = cards.get().iter().find(|c| c.id == card_id).cloned() else {
      return;
    };
    let scheduled = srs_sm2::schedule_next(
      srs_sm2::Schedule {
        interval_days: target.interval_days,
        ease_factor: target.ease_factor,
      },
      quality,
    );
    let new_ease = scheduled.ease_factor;
    let interval = scheduled.interval_days;
    let next = {
      let dt = chrono::Utc::now() + chrono::Duration::days(i64::from(interval));
      crate::core::time::to_iso(dt)
    };
    let user = match auth.user.get() {
      Some(id) => id,
      None => return,
    };
    let body = serde_json::json!({
        "ease_factor": new_ease,
        "interval_days": interval,
        "next_review": next,
    });
    leptos::task::spawn_local(async move {
      let result: Result<(), String> = async {
        let client = supabase::supabase().map_err(|e| e.to_string())?;
        client
          .postgrest()
          .from("reading_flashcards")
          .eq("id", card_id.to_string())
          .eq("user_id", user.to_string())
          .update::<Flashcard, _>(&body)
          .await
          .map_err(|e| e.to_string())?;
        Ok::<(), String>(())
      }
      .await;
      if !disposed.get_untracked() {
        if let Err(err) = result {
          error.set(err);
        }
        reviewed.update(|n| *n += 1);
        cards.update(|list| {
          crate::core::srs::remove_card(list, card_id);
        });
        crate::composables::announce("Card reviewed");
      }
    });
  };

  let add_card = move |_: web_sys::SubmitEvent| {
    let front = new_front.get();
    let back = new_back.get();
    if front.trim().is_empty() || back.trim().is_empty() {
      return;
    }
    let user = match auth.user.get() {
      Some(id) => id,
      None => return,
    };
    adding.set(true);
    error.set(String::new());
    let body = serde_json::json!({
        "user_id": user,
        "chapter_id": serde_json::Value::Null,
        "front": front.trim(),
        "back": back.trim(),
    });
    leptos::task::spawn_local(async move {
      let result: Result<Flashcard, String> = async {
        let client = supabase::supabase().map_err(|e| e.to_string())?;
        let card: Flashcard = client
          .postgrest()
          .from("reading_flashcards")
          .insert_one(&body)
          .await
          .map_err(|e| e.to_string())?;
        Ok(card)
      }
      .await;
      if disposed.get_untracked() {
        return;
      }
      adding.set(false);
      match result {
        Ok(card) => {
          if card.next_review <= chrono::Utc::now() {
            cards.update(|list| list.push(card));
          }
          new_front.set(String::new());
          new_back.set(String::new());
          show_add_modal.set(false);
          crate::composables::announce("Flashcard added");
        }
        Err(err) => error.set(err),
      }
    });
  };

  let set_tab = move |target: Tab| {
    active_tab.set(target);
  };

  // Arrow-key roving navigation across the review tablist.
  let tabs: [Tab; 2] = [Tab::Cards, Tab::Timer];
  let on_tabs_keydown = move |ev: web_sys::KeyboardEvent| {
    let current = active_tab.get();
    let idx = tabs.iter().position(|t| *t == current).unwrap_or(0);
    let next = match ev.key().as_str() {
      "ArrowRight" | "ArrowDown" => Some(tabs[(idx + 1) % tabs.len()]),
      "ArrowLeft" | "ArrowUp" => Some(tabs[(idx + tabs.len() - 1) % tabs.len()]),
      "Home" => Some(tabs[0]),
      "End" => Some(tabs[tabs.len() - 1]),
      _ => None,
    };
    if let Some(target) = next {
      ev.prevent_default();
      set_tab(target);
    }
  };

  view! {
      <div class="page review">
          <header class="page-header">
              <div>
                  <h1 class="page-header__title">"Review"</h1>
                  <p class="page-header__sub">"Recall flashcards and run focus sessions."</p>
              </div>
              <div class="page-header__actions">
                  <BaseButton
                      size=ButtonSize::Small
                      variant=ButtonVariant::Secondary
                      on_click=Callback::new(move |_| show_add_modal.set(true))
                  >
                      <Plus size=14 />
                      "Add card"
                  </BaseButton>
              </div>
          </header>

          <Show when=move || config_message_for_show.is_some() fallback=move || view! { <span class="visually-hidden">""</span> }>
              <p class="notice">{{config_message_text}}</p>
          </Show>
          <Show when=move || !error.get().is_empty() fallback=move || view! { <span class="visually-hidden">""</span> }>
              <p class="notice" role="alert">{error}</p>
          </Show>

          <div class="review__tabs" role="tablist" aria-label="Review sections" on:keydown=on_tabs_keydown>
              <button
                  type="button"
                  role="tab"
                  class="review__tab"
                  class:is-active=move || active_tab.get() == Tab::Cards
                  aria-selected=move || (active_tab.get() == Tab::Cards).to_string()
                  aria-controls="review-panel-cards"
                  tabindex=move || if active_tab.get() == Tab::Cards { 0_i32 } else { -1_i32 }
                  on:click=move |_| set_tab(Tab::Cards)
              >
                  <Brain size=14 />
                  "Flashcards"
                  <Show when=move || !cards.get().is_empty() fallback=|| view! { <span class="visually-hidden">""</span> }>
                      <span class="review__badge" aria-hidden="true">{move || cards.get().len()}</span>
                  </Show>
              </button>
              <button
                  type="button"
                  role="tab"
                  class="review__tab"
                  class:is-active=move || active_tab.get() == Tab::Timer
                  aria-selected=move || (active_tab.get() == Tab::Timer).to_string()
                  aria-controls="review-panel-timer"
                  tabindex=move || if active_tab.get() == Tab::Timer { 0_i32 } else { -1_i32 }
                  on:click=move |_| set_tab(Tab::Timer)
              >
                  <Clock3 size=14 />
                  "Timer"
              </button>
          </div>

          <Show
              when=move || active_tab.get() == Tab::Cards
              fallback=move || view! {
                  <section
                      id="review-panel-timer"
                      role="tabpanel"
                      aria-label="Review timer"
                      class="review__content surface"
                  >
                      <PomodoroTimer />
                  </section>
              }
          >
              <section
                  id="review-panel-cards"
                  role="tabpanel"
                  aria-label="Review cards"
                  class="review__content surface"
              >
                  <Show
                      when=move || loading.get()
                      fallback=move || {
                          view! {
                              <Show
                                  when=move || cards.get().is_empty()
                                  fallback=move || view! {
                                  <div class="review__cards">
                                      <p class="review__count">
                                          {move || crate::core::srs::review_header_copy(cards.get().len(), reviewed.get())}
                                      </p>
                                      <p class="review__meta">
                                          <span class="numeric">{move || cards.get().len()}</span>
                                          " "
                                          {move || if cards.get().len() == 1 { "card" } else { "cards" }}
                                          " left · "
                                          <span class="numeric">{move || reviewed.get()}</span>
                                          " reviewed"
                                      </p>
                                      {move || cards.get().first().cloned().map(|current| view! {
                                          <FlashcardContainer
                                              card=current
                                              on_rated=Callback::new(handle_rated)
                                          />
                                      })}
                                  </div>
                                  }
                              >
                                  <div class="review__done">
                                      <div class="review__done-icon">
                                          <CheckCheck size=22 />
                                      </div>
                                      <h2 class="review__done-title">"All caught up"</h2>
                                      <p class="review__done-sub">"No cards are due. Add new prompts or come back later."</p>
                                      <BaseButton
                                          size=ButtonSize::Small
                                          variant=ButtonVariant::Secondary
                                          on_click=Callback::new(move |_| show_add_modal.set(true))
                                      >
                                          <Plus size=14 />
                                          "Add card"
                                      </BaseButton>
                                  </div>
                              </Show>
                          }
                      }
                  >
                      <div class="review__loading">
                          <BaseLoader size=28 />
                      </div>
                  </Show>
              </section>
          </Show>

          <BaseModal
              open=Signal::derive(move || show_add_modal.get())
              on_close=Callback::new(move |_| show_add_modal.set(false))
              title="Add flashcard"
          >
              <form class="review__form" aria-label="Add flashcard" on:submit=move |ev| {
                  ev.prevent_default();
                  add_card(ev);
              }>
                  <BaseInput
                      value=Signal::derive(move || new_front.get())
                      on_input=Callback::new(move |v: String| new_front.set(v))
                      label="Front (question)"
                      placeholder="What is ownership?"
                  />
                  <BaseTextarea
                      value=Signal::derive(move || new_back.get())
                      on_input=Callback::new(move |v: String| new_back.set(v))
                      label="Back (answer)"
                      placeholder="Ownership is a set of rules that..."
                      rows=5
                  />
                  <div class="form-actions">
                      <BaseButton
                          variant=ButtonVariant::Secondary
                          on_click=Callback::new(move |_| show_add_modal.set(false))
                      >
                          "Cancel"
                      </BaseButton>
                      <BaseButton button_type="submit" loading=adding.get_untracked()>
                          "Add"
                      </BaseButton>
                  </div>
              </form>
          </BaseModal>
      </div>
  }
}
