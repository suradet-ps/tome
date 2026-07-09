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

    let submit = move |_: web_sys::SubmitEvent| {
        error.set(String::new());
        let email_value = email.get();
        let password_value = password.get();
        let navigate = navigate.clone();
        leptos::task::spawn_local(async move {
            match auth.sign_in(&email_value, &password_value).await {
                Ok(()) => navigate("/", Default::default()),
                Err(err) => error.set(err.to_string()),
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
                    <h1 class="auth__title">"Welcome back"</h1>
                    <p class="auth__subtitle">"Sign in to keep tracking your reading."</p>
                </div>

                <Show when=move || config_message_for_show.is_some() fallback=|| view! { <span class="visually-hidden">""</span> }>
                    <p class="notice">{{config_message_text}}</p>
                </Show>

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

                <p class="auth__switch">
                    "Don't have an account? "
                    <A href="/register" attr:class="auth__link">"Create one"</A>
                </p>
            </div>
        </div>
    }
}

#[allow(dead_code)]
fn _ensure_button_size_unused(_: ButtonSize, _: ButtonVariant) {}
