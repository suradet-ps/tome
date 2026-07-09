use crate::components::layout::app_topbar::AppTopbar;
use crate::stores::auth::{use_auth, AuthState};
use crate::stores::books::BooksState;
use crate::stores::notes::NotesState;
use crate::stores::progress::ProgressState;
use crate::views::{
    book_view::BookView, dashboard_view::DashboardView, login_view::LoginView, not_found::NotFound,
    register_view::RegisterView, review_view::ReviewView,
};
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    WildcardSegment,
    components::{Route, Router, Routes},
    path,
};

fn make_auth() -> AuthState {
    let state = AuthState {
        user: RwSignal::new(None),
        profile: RwSignal::new(None),
        initialized: RwSignal::new(false),
        loading: RwSignal::new(false),
        error: RwSignal::new(None),
    };
    provide_context(state);
    state
}

/// Root component.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let auth = make_auth();
    let books = BooksState::provide();
    let progress = ProgressState::provide();
    let notes = NotesState::provide();

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
            <RouterChild auth books progress notes />
        </Router>
    }
}

/// Re-provides all stores for routes inside the Router scope.
#[component]
fn RouterChild(
    auth: AuthState,
    books: BooksState,
    progress: ProgressState,
    notes: NotesState,
) -> impl IntoView {
    provide_context(auth);
    provide_context(books);
    provide_context(progress);
    provide_context(notes);

    let user = auth.user;
    let fallback = || view! { <NotFound /> };

    view! {
        <div class="app">
            <Show
                when=move || user.get().is_some()
                fallback=move || view! {
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
                }
            >
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
