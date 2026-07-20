//! Application top navigation bar (responsive).

use crate::components::icons::{BookOpen, Brain, LayoutDashboard, LogOut, Menu, Sun, X};
use crate::stores::auth::use_auth;
use crate::stores::settings::{SettingsState, Theme};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

const MOBILE_BREAKPOINT: i32 = 768;

/// Sticky topbar with brand, primary nav, user chip and sign-out button.
#[component]
pub fn AppTopbar() -> impl IntoView {
  let auth = use_auth();
  let location = use_location();
  let mobile_open = RwSignal::new(false);
  let display_open = RwSignal::new(false);
  let settings = SettingsState::use_ctx();

  Effect::new(move |_| {
    // Close the mobile menu whenever the route changes.
    let _ = location.pathname.get();
    mobile_open.set(false);
  });

  let sign_out = Callback::new(move |_: web_sys::MouseEvent| {
    let auth = auth;
    leptos::task::spawn_local(async move {
      auth.sign_out().await;
    });
  });

  let navigate = leptos_router::hooks::use_navigate();
  let sign_out_and_redirect = Callback::new(move |_: web_sys::MouseEvent| {
    let navigate = navigate.clone();
    let auth = auth;
    leptos::task::spawn_local(async move {
      auth.sign_out().await;
      navigate("/login", Default::default());
    });
  });

  let _user_value = auth.user;
  let profile = auth.profile;
  let initial = move || {
    profile
      .get()
      .as_ref()
      .and_then(|p| p.username.as_ref())
      .and_then(|name| name.chars().next())
      .map_or_else(|| "?".to_string(), |c| c.to_ascii_uppercase().to_string())
  };
  let user_name = move || {
    profile
      .get()
      .as_ref()
      .and_then(|p| p.username.clone())
      .or_else(|| {
        // Email fallback when profile is not yet loaded.
        web_sys::window()
          .and_then(|w| w.document())
          .and_then(|d| d.active_element().map(|_| ()))
          .map(|_| String::new())
      })
      .unwrap_or_default()
  };

  let is_active = move |path: &str| {
    let current = location.pathname.get();
    if path == "/" {
      current == "/" || current.starts_with("/books")
    } else {
      current.starts_with(path)
    }
  };

  // Reading-comfort controls.
  let current_theme = Signal::derive(move || settings.settings.get().theme);
  let current_width = Signal::derive(move || settings.settings.get().width_ch);
  let current_scale = Signal::derive(move || settings.settings.get().font_scale);
  let pick_theme = move |theme: Theme| settings.set_theme(theme);
  let change_width = move |ev: web_sys::Event| {
    let value = event_target_value(&ev);
    if let Ok(n) = value.parse::<u32>() {
      settings.set_width(n);
    }
  };
  let change_scale = move |ev: web_sys::Event| {
    let value = event_target_value(&ev);
    if let Ok(n) = value.parse::<f32>() {
      settings.set_font_scale(n);
    }
  };

  view! {
      <header class="topbar">
          <div class="topbar__inner">
              <A href="/" attr:class="topbar__brand" attr:aria-label="Tome">
                  <BookOpen size=20 />
                  <span>"Tome"</span>
              </A>

              <nav class="topbar__nav" aria-label="Primary">
                  <A
                      href="/"
                      attr:class=move || {
                          if is_active("/") {
                              "topbar__link topbar__link--active"
                          } else {
                              "topbar__link"
                          }
                      }
                  >
                      <LayoutDashboard size=15 />
                      <span>"Library"</span>
                  </A>
                  <A
                      href="/review"
                      attr:class=move || {
                          if is_active("/review") {
                              "topbar__link topbar__link--active"
                          } else {
                              "topbar__link"
                          }
                      }
                  >
                      <Brain size=15 />
                      <span>"Review"</span>
                  </A>
              </nav>

              <div class="topbar__actions">
                  <div class="topbar__display">
                      <button
                          class="topbar__icon-btn"
                          type="button"
                          aria-expanded=move || display_open.get().to_string()
                          aria-label="Reading display settings"
                          title="Display"
                          on:click=move |_| display_open.update(|open| *open = !*open)
                      >
                          <Sun size=16 />
                      </button>
                      <Show when=move || display_open.get() fallback=|| view! {}>
                          <div class="display-pop" role="group" aria-label="Reading display">
                              <div class="display-pop__row">
                                  <span class="display-pop__label">"Theme"</span>
                                  <div class="display-pop__themes" role="radiogroup" aria-label="Theme">
                                      {Theme::ALL.map(|theme| {
                                          let theme_for_click = theme;
                                          view! {
                                              <button
                                                  type="button"
                                                  role="radio"
                                                  class="display-pop__theme"
                                                  class:is-active=move || current_theme.get() == theme_for_click
                                                  aria-checked=move || (current_theme.get() == theme_for_click).to_string()
                                                  on:click=move |_| pick_theme(theme_for_click)
                                              >
                                                  {theme_for_click.label()}
                                              </button>
                                          }
                                      })}
                                  </div>
                              </div>
                              <div class="display-pop__row">
                                  <label class="display-pop__label" for="display-width">
                                      "Width"
                                  </label>
                                  <input
                                      id="display-width"
                                      class="display-pop__range"
                                      type="range"
                                      min="48"
                                      max="120"
                                      step="2"
                                      prop:value=move || current_width.get().to_string()
                                      on:input=change_width
                                  />
                                  <span class="display-pop__value numeric">{move || current_width.get()}</span>
                              </div>
                              <div class="display-pop__row">
                                  <label class="display-pop__label" for="display-scale">
                                      "Text"
                                  </label>
                                  <input
                                      id="display-scale"
                                      class="display-pop__range"
                                      type="range"
                                      min="0.875"
                                      max="1.375"
                                      step="0.125"
                                      prop:value=move || format!("{:.3}", current_scale.get())
                                      on:input=change_scale
                                  />
                                  <span class="display-pop__value numeric">
                                      {move || format!("{}%", (current_scale.get() * 100.0).round() as i32)}
                                  </span>
                              </div>
                          </div>
                      </Show>
                  </div>
                  <div
                      class="topbar__user"
                      title=move || {
                          profile
                              .get()
                              .as_ref()
                              .and_then(|p| p.username.clone())
                              .unwrap_or_default()
                      }
                  >
                      <div class="topbar__avatar">{initial}</div>
                      <span class="topbar__user-name">{user_name}</span>
                  </div>
                  <button
                      class="topbar__icon-btn"
                      type="button"
                      on:click=move |ev: web_sys::MouseEvent| sign_out_and_redirect.run(ev)
                      aria-label="Sign out"
                      title="Sign out"
                  >
                      <LogOut size=16 />
                  </button>
                  <button
                      class="topbar__icon-btn topbar__menu-toggle"
                      type="button"
                      aria-expanded=move || mobile_open.get().to_string()
                      aria-label="Menu"
                      on:click=move |_| {
                          mobile_open.update(|open| *open = !*open);
                      }
                  >
                      <Show
                          when=move || mobile_open.get()
                          fallback=move || view! { <Menu size=18 /> }
                      >
                          <X size=18 />
                      </Show>
                  </button>
              </div>
          </div>

          <Show when=move || mobile_open.get() fallback=|| view! {}>
              <div class="topbar__sheet">
                  <A
                      href="/"
                      attr:class=move || {
                          if is_active("/") {
                              "topbar__sheet-link topbar__sheet-link--active"
                          } else {
                              "topbar__sheet-link"
                          }
                      }
                  >
                      <LayoutDashboard size=16 />
                      "Library"
                  </A>
                  <A
                      href="/review"
                      attr:class=move || {
                          if is_active("/review") {
                              "topbar__sheet-link topbar__sheet-link--active"
                          } else {
                              "topbar__sheet-link"
                          }
                      }
                  >
                      <Brain size=16 />
                      "Review"
                  </A>
                  <button
                      type="button"
                      class="topbar__sheet-link topbar__sheet-link--danger"
                      on:click=move |ev: web_sys::MouseEvent| sign_out.run(ev)
                  >
                      <LogOut size=16 />
                      "Sign out"
                  </button>
              </div>
          </Show>
      </header>
  }
}

// Suppress unused import warning when feature flags trim `Brain`.
#[allow(dead_code)]
const fn _ensure_brain(_: i32) -> i32 {
  MOBILE_BREAKPOINT
}
