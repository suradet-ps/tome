//! Sign-up form.

use crate::components::common::base_button::BaseButton;
use crate::components::common::base_input::BaseInput;
use crate::components::icons::BookOpen;
use crate::core::supabase;
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

/// Sign-up page.
#[component]
pub fn RegisterView() -> impl IntoView {
  let auth = use_auth();
  let navigate = use_navigate();
  let email = RwSignal::new(String::new());
  let password = RwSignal::new(String::new());
  let username = RwSignal::new(String::new());
  let error = RwSignal::new(String::new());

  let disposed = RwSignal::new(false);
  on_cleanup(move || disposed.set(true));

  let submit = move |_: web_sys::SubmitEvent| {
    error.set(String::new());
    if username.get().trim().chars().count() < 3 {
      error.set("Username must be at least 3 characters".to_string());
      return;
    }
    let email_value = email.get();
    let password_value = password.get();
    let username_value = username.get();
    let navigate = navigate.clone();
    leptos::task::spawn_local(async move {
      match auth
        .sign_up(&email_value, &password_value, &username_value)
        .await
      {
        Ok(()) => navigate("/", Default::default()),
        Err(err) => {
          if !disposed.get_untracked() {
            error.set(err.to_string());
          }
        }
      }
    });
  };

  let config_message: Option<String> = supabase::supabase_config_error();
  let config_message_for_show = config_message.clone();
  let config_message_text = Signal::derive(move || config_message.clone().unwrap_or_default());

  view! {
      <div class="auth">
          <div class="auth__card">
              <div class="auth__brand">
                  <BookOpen size=22 />
                  <span>"Tome"</span>
              </div>

              <div class="auth__intro">
                  <h1 class="auth__title">"Create account"</h1>
                  <p class="auth__subtitle">"Start tracking the books you read."</p>
              </div>

              <Show when=move || config_message_for_show.is_some() fallback=|| view! { <span class="visually-hidden">""</span> }>
                  <p class="notice">{{config_message_text}}</p>
              </Show>

              <form class="auth__form" aria-label="Create account" on:submit=move |ev| {
                  ev.prevent_default();
                  submit(ev);
              }>
                  <BaseInput
                      value=Signal::derive(move || username.get())
                      on_input=Callback::new(move |v: String| username.set(v))
                      label="Username"
                      placeholder="reader42"
                  />
                  <BaseInput
                      value=Signal::derive(move || email.get())
                      on_input=Callback::new(move |v: String| email.set(v))
                      label="Email"
                      input_type="email"
                      placeholder="you@example.com"
                  />
                  <BaseInput
                      value=Signal::derive(move || password.get())
                      on_input=Callback::new(move |v: String| password.set(v))
                      label="Password"
                      input_type="password"
                      placeholder="••••••••"
                  />
                  <Show when=move || !error.get().is_empty() fallback=|| view! { <span class="visually-hidden">""</span> }>
                      <p class="auth__error">{error}</p>
                  </Show>
                  <BaseButton
                      button_type="submit"
                      block=true
                      loading=auth.loading.get_untracked()
                  >
                      "Create account"
                  </BaseButton>
              </form>

              <p class="auth__switch">
                  "Already have an account? "
                  <A href="/login" attr:class="auth__link">"Sign in"</A>
              </p>
          </div>
      </div>
  }
}
