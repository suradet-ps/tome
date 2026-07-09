//! Route guards and small router helpers.
//!
//! The router doesn't have a built-in guard mechanism in Leptos 0.8, so the
//! shell component (`App::Shell`) handles the `user == None` case by
//! mounting the unauthenticated routes. This module currently only exposes
//! the loader view used while auth is initialising.

use leptos::prelude::*;

/// Loader view rendered while the auth state is initialising.
#[component]
pub fn AuthLoader() -> impl IntoView {
    view! {
        <div class="auth__loading">
            <div class="loader" role="status" aria-label="Loading">
                <div class="loader__ring"></div>
            </div>
        </div>
    }
}
