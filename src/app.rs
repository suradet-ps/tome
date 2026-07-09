//! Top-level `App` component: sets up routing, contexts and the page shell.

use crate::components::layout::app_topbar::AppTopbar;
use crate::stores::auth::AuthState;
use crate::stores::books::BooksState;
use crate::stores::notes::NotesState;
use crate::stores::progress::ProgressState;
use crate::views::{
    book_view::BookView, dashboard_view::DashboardView, login_view::LoginView,
    not_found::NotFound, register_view::RegisterView, review_view::ReviewView,
};
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    WildcardSegment,
    components::{Route, Router, Routes},
    path,
};

/// Root component.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let auth = AuthState::new();
    let books = BooksState::new();
    let progress = ProgressState::new();
    let notes = NotesState::new();

    Effect::new(move |_| {
        if !auth.initialized.get_untracked() {
            leptos::task::spawn_local(async move { auth.init_auth().await; });
        }
    });

    view! {
        <Stylesheet id="main" href="/styles/main.css" />
        <Title text="Tome - Technical Reading Tracker" />
        <Meta name="description" content="Tome - track technical books, notes and flashcards." />
        <Router>
            <Shell auth books progress notes />
        </Router>
    }
}

/// Shell renders the app layout. Each route view closure re-provides
/// the stores via `provide_context` before rendering, because the
/// leptos 0.8 Router creates isolated scopes that don't inherit parent
/// contexts.
#[component]
fn Shell(
    auth: AuthState,
    books: BooksState,
    progress: ProgressState,
    notes: NotesState,
) -> impl IntoView {
    let user = auth.user;
    let fallback = || view! { <NotFound /> };

    // Each route needs its own `provide_context` call because Router
    // scopes are isolated. We create a helper: provide stores, then
    // return the view.
    let login_provided = move || {
        provide_context(auth);
        provide_context(books);
        provide_context(progress);
        provide_context(notes);
        LoginView
    };
    let reg_provided = move || {
        provide_context(auth);
        provide_context(books);
        provide_context(progress);
        provide_context(notes);
        RegisterView
    };
    let dash_provided = move || {
        provide_context(auth);
        provide_context(books);
        provide_context(progress);
        provide_context(notes);
        DashboardView
    };
    let book_provided = move || {
        provide_context(auth);
        provide_context(books);
        provide_context(progress);
        provide_context(notes);
        BookView
    };
    let review_provided = move || {
        provide_context(auth);
        provide_context(books);
        provide_context(progress);
        provide_context(notes);
        ReviewView
    };

    view! {
        <div class="app">
            <Show
                when=move || user.get().is_some()
                fallback=move || view! {
                    <main class="app-main">
                        <div class="app-main__inner">
                            <Routes fallback=fallback>
                                <Route path=path!("/login") view=login_provided />
                                <Route path=path!("/register") view=reg_provided />
                                <Route path=path!("/") view=login_provided />
                                <Route path=WildcardSegment("") view=NotFound />
                            </Routes>
                        </div>
                    </main>
                }
            >
                <div class="app-shell">
                    <AppTopbar />
                    <main class="app-main">
                        <div class="app-main__inner">
                            <Routes fallback=fallback>
                                <Route path=path!("/") view=dash_provided />
                                <Route path=path!("/books/:id") view=book_provided />
                                <Route path=path!("/review") view=review_provided />
                                <Route path=path!("/login") view=login_provided />
                                <Route path=path!("/register") view=reg_provided />
                                <Route path=WildcardSegment("") view=NotFound />
                            </Routes>
                        </div>
                    </main>
                </div>
            </Show>
        </div>
    }
}
