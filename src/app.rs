//! Top-level `App` component: sets up routing and the page shell.

use crate::components::layout::app_topbar::AppTopbar;
use crate::stores::auth::use_auth;
use crate::views::{
    book_view::BookView, dashboard_view::DashboardView, login_view::LoginView, not_found::NotFound,
    register_view::RegisterView, review_view::ReviewView,
};
use leptos::prelude::*;
use leptos_meta::{Meta, Title, provide_meta_context};
use leptos_router::{
    WildcardSegment,
    components::{Route, Router, Routes},
    path,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let auth = use_auth();

    Effect::new(move |_| {
        if !auth.initialized.get_untracked() {
            leptos::task::spawn_local(async move {
                auth.init_auth().await;
            });
        }
    });

    view! {
        <Title text="Tome - Technical Reading Tracker" />
        <Meta name="description" content="Tome - track technical books, notes and flashcards." />
        <Router>
            <Shell />
        </Router>
    }
}

#[component]
fn Shell() -> impl IntoView {
    let auth = use_auth();
    let user = auth.user;
    let fallback = || view! { <NotFound /> };

    view! {
        <div class="app">
            <Show when=move || user.get().is_some() fallback=move || view! {
                <main class="app-main">
                    <div class="app-main__inner">
                        <Routes fallback=fallback>
                            <Route path=path!("/login") view=LoginView />
                            <Route path=path!("/register") view=RegisterView />
                            <Route path=path!("/") view=LoginView />
                            <Route path=WildcardSegment("") view=NotFound />
                        </Routes>
                    </div>
                </main>
            }>
                <div class="app-shell">
                    <AppTopbar />
                    <main class="app-main">
                        <div class="app-main__inner">
                            <Routes fallback=fallback>
                                <Route path=path!("/") view=DashboardView />
                                <Route path=path!("/books/:id") view=BookView />
                                <Route path=path!("/review") view=ReviewView />
                                <Route path=path!("/login") view=LoginView />
                                <Route path=path!("/register") view=RegisterView />
                                <Route path=WildcardSegment("") view=NotFound />
                            </Routes>
                        </div>
                    </main>
                </div>
            </Show>
        </div>
    }
}
