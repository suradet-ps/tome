//! Application top navigation bar (responsive).

use crate::components::icons::{BookOpen, Brain, LayoutDashboard, LogOut, Menu, X};
use crate::stores::auth::use_auth;
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
