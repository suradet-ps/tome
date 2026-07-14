//! Sign-in form.

use crate::components::common::base_button::{BaseButton, ButtonSize, ButtonVariant};
use crate::components::common::base_input::BaseInput;
use crate::components::icons::BookOpen;
use crate::core::supabase;
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

/// Sign-in page.
#[component]
pub fn LoginView() -> impl IntoView {
  let auth = use_auth();
  let navigate = use_navigate();
  let email = RwSignal::new(String::new());
  let password = RwSignal::new(String::new());
  let error = RwSignal::new(String::new());

  let disposed = RwSignal::new(false);
  on_cleanup(move || disposed.set(true));

  // In-app configuration form (shown when env vars are missing).
  let config_url = RwSignal::new(String::new());
  let config_anon = RwSignal::new(String::new());
  let config_saved = RwSignal::new(false);
  let config_error = RwSignal::new(String::new());

  let submit = move |_: web_sys::SubmitEvent| {
    error.set(String::new());
    let email_value = email.get();
    let password_value = password.get();
    let navigate = navigate.clone();
    leptos::task::spawn_local(async move {
      match auth.sign_in(&email_value, &password_value).await {
        Ok(()) => navigate("/", Default::default()),
        Err(err) => {
          if !disposed.get_untracked() {
            error.set(err.to_string());
          }
        }
      }
    });
  };

  let save_config = move || {
    let url = config_url.get().trim().to_string();
    let anon = config_anon.get().trim().to_string();
    if url.is_empty() || anon.is_empty() {
      return;
    }
    supabase::save_config(&url, &anon);
    config_saved.set(true);
    config_error.set(String::new());
  };

  let config_message: Option<String> = supabase::supabase_config_error();
  let has_config_error = config_message.is_some();
  let config_message_text = config_message.unwrap_or_default();

  view! {
      <div class="auth">
          <div class="auth__card">
              <div class="auth__brand">
                  <BookOpen size=22 />
                  <span>"Tome"</span>
              </div>

              <div class="auth__intro">
                  <h1 class="auth__title">"Welcome back"</h1>
                  <p class="auth__subtitle">"Sign in to keep tracking your reading."</p>
              </div>

              <Show when=move || has_config_error && !config_saved.get()
                  fallback=|| view! {}
              >
                  <div class="notice" style="display:flex;flex-direction:column;gap:8px;">
                      <p style="font-size:12px;line-height:1.5">
                          {config_message_text.clone()}
                      </p>
                      <BaseInput
                          value=Signal::derive(move || config_url.get())
                          on_input=Callback::new(move |v: String| config_url.set(v))
                          label="Supabase URL"
                          placeholder="https://your-project.supabase.co"
                      />
                      <BaseInput
                          value=Signal::derive(move || config_anon.get())
                          on_input=Callback::new(move |v: String| config_anon.set(v))
                          label="Anon Key"
                          placeholder="eyJhbGci..."
                      />
                      <div style="display:flex;gap:8px;align-items:center;">
                          <BaseButton on_click=Callback::new(move |_: web_sys::MouseEvent| save_config())>
                              "Save config"
                          </BaseButton>
                          <Show when=move || config_saved.get() fallback=|| view! {}>
                              <span style="color:var(--color-success);font-size:12px;">"Saved!"</span>
                          </Show>
                      </div>
                  </div>
              </Show>

              <Show when=move || config_saved.get() && has_config_error
                  fallback=|| view! {}
              >
                  <p class="notice" style="font-size:12px;">
                      "Config saved. If credentials are correct, sign in below."
                  </p>
              </Show>

              <div style:opacity=move || if has_config_error { "0.3" } else { "1" }>
                  <form class="auth__form" aria-label="Sign in" on:submit=move |ev| {
                      ev.prevent_default();
                      submit(ev);
                  }>
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
                          "Sign in"
                      </BaseButton>
                  </form>
              </div>

              <p class="auth__switch">
                  "Don't have an account? "
                  <A href="/register" attr:class="auth__link">"Create one"</A>
              </p>
          </div>
      </div>
  }
}

#[allow(dead_code)]
const fn _ensure_button_size_unused(_: ButtonSize, _: ButtonVariant) {}
