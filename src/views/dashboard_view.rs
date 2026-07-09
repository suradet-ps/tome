//! Dashboard view: list of books with progress, stats and quick actions.

use crate::components::common::base_button::BaseButton;
use crate::components::common::base_input::BaseInput;
use crate::components::common::base_loader::BaseLoader;
use crate::components::common::base_modal::BaseModal;
use crate::components::icons::{ArrowRight, BookOpen, Plus};
use crate::components::progress::progress_bar::ProgressBar;
use crate::core::supabase;
use crate::core::types::DashboardSummaryRow;
use crate::stores::auth::use_auth;
use crate::stores::books::BooksState;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[derive(Clone)]
struct BookSnapshot {
    completed: u32,
    total: u32,
    percent: u32,
}

/// Dashboard page.
#[component]
pub fn DashboardView() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let books_store = BooksState::use_ctx();

    let show_add_modal = RwSignal::new(false);
    let new_title = RwSignal::new(String::new());
    let new_author = RwSignal::new(String::new());
    let adding = RwSignal::new(false);
    let add_error = RwSignal::new(String::new());
    let dashboard_error = RwSignal::new(String::new());
    let stats = RwSignal::new((0_u32, 0_u32));
    let book_progress = RwSignal::new(Vec::<(uuid::Uuid, BookSnapshot)>::new());

    let config_message: Option<String> = supabase::supabase_config_error();
    let config_message_for_show = config_message.clone();
    let config_message_text = Signal::derive(move || config_message.clone().unwrap_or_default());
    let greeting = move || {
        auth.profile
            .get()
            .as_ref()
            .and_then(|p| p.username.clone())
            .unwrap_or_else(|| "there".to_string())
    };

    let load = move || {
        leptos::task::spawn_local(async move {
            dashboard_error.set(String::new());
            let result: Result<(), String> = async {
                books_store.fetch_books().await.map_err(|e| e.to_string())?;
                let user = match auth.user.get() {
                    Some(id) => id,
                    None => {
                        stats.set((0, 0));
                        book_progress.set(Vec::new());
                        return Ok(());
                    }
                };
                if supabase::supabase_config_error().is_some() {
                    stats.set((0, 0));
                    book_progress.set(Vec::new());
                    return Ok(());
                }
                let client = supabase::supabase().map_err(|e| e.to_string())?;
                let summary: Vec<DashboardSummaryRow> = client
                    .postgrest()
                    .rpc("get_dashboard_summary", &serde_json::json!({}))
                    .await
                    .map_err(|e| e.to_string())?;
                let next: Vec<(uuid::Uuid, BookSnapshot)> = books_store
                    .books
                    .get()
                    .iter()
                    .map(|book| {
                        let row = summary.iter().find(|r| r.book_id == book.id);
                        let total = row.map_or(book.total_chapters as u32, |r| r.total as u32);
                        let completed = row.map_or(0, |r| r.completed as u32);
                        let percent = if total == 0 {
                            0
                        } else {
                            ((completed as f64 / total as f64) * 100.0).round() as u32
                        };
                        (
                            book.id,
                            BookSnapshot {
                                completed,
                                total,
                                percent,
                            },
                        )
                    })
                    .collect();
                let total_completed: u32 = next.iter().map(|(_, snap)| snap.completed).sum();
                let cards: Vec<serde_json::Value> = client
                    .postgrest()
                    .from("reading_flashcards")
                    .select("id")
                    .eq("user_id", user.to_string())
                    .lte("next_review", crate::core::time::now_iso())
                    .range(0, 999)
                    .get()
                    .await
                    .map_err(|e| e.to_string())?;
                let cards_due = cards.len() as u32;
                stats.set((total_completed, cards_due));
                book_progress.set(next);
                Ok::<(), String>(())
            }
            .await;
            if let Err(err) = result {
                dashboard_error.set(err);
            }
        });
    };

    Effect::new(move |_| {
        let _ = auth.user.get();
        load();
    });

    let close_modal = move |_| {
        show_add_modal.set(false);
    };

    let add_book = move |_: web_sys::SubmitEvent| {
        add_error.set(String::new());
        if new_title.get().trim().is_empty() {
            add_error.set("Title is required.".to_string());
            return;
        }
        let title = new_title.get();
        let author = new_author.get();
        adding.set(true);
        leptos::task::spawn_local(async move {
            let result = books_store.add_book(&title, &author).await;
            adding.set(false);
            match result {
                Ok(Some(_)) => {
                    new_title.set(String::new());
                    new_author.set(String::new());
                    show_add_modal.set(false);
                    load();
                }
                Ok(None) => {
                    add_error.set("Title is required.".to_string());
                }
                Err(err) => add_error.set(err.to_string()),
            }
        });
    };

    let open_book = Callback::new(move |id: uuid::Uuid| {
        let navigate = navigate.clone();
        navigate(&format!("/books/{id}"), Default::default());
    });

    view! {
        <div class="page dashboard">
            <header class="page-header">
                <div>
                    <h1 class="page-header__title">"Hi, " {greeting}</h1>
                    <p class="page-header__sub">"Track what you read, one chapter at a time."</p>
                </div>
                <div class="page-header__actions">
                    <BaseButton on_click=Callback::new(move |_| show_add_modal.set(true))>
                        <Plus size=16 />
                        "Add book"
                    </BaseButton>
                </div>
            </header>

            <Show when=move || config_message_for_show.is_some() fallback=move || view! { <span class="visually-hidden">""</span> }>
                <p class="notice">{{config_message_text}}</p>
            </Show>
            <Show when=move || !dashboard_error.get().is_empty() fallback=move || view! { <span class="visually-hidden">""</span> }>
                <p class="notice">{dashboard_error}</p>
            </Show>

            <section class="stats">
                <div class="stats__item">
                    <span class="stats__label">"Books"</span>
                    <span class="stats__value numeric">{move || books_store.books.get().len()}</span>
                </div>
                <div class="stats__divider" aria-hidden="true"></div>
                <div class="stats__item">
                    <span class="stats__label">"Chapters done"</span>
                    <span class="stats__value numeric">{move || stats.get().0}</span>
                </div>
                <div class="stats__divider" aria-hidden="true"></div>
                <div class="stats__item">
                    <span class="stats__label">"Cards due"</span>
                    <span class="stats__value numeric">{move || stats.get().1}</span>
                </div>
            </section>

            <Show
                when=move || books_store.loading.get()
                fallback=move || view! {
                    <Show
                        when=move || books_store.books.get().is_empty()
                        fallback=move || view! {
                            <section class="book-grid">
                                <For
                                    each=move || books_store.books.get()
                                    key=|book| book.id
                                    children=move |book| {
                                        let id = book.id;
                                        let snapshot = move || {
                                            book_progress
                                                .get()
                                                .iter()
                                                .find(|(b, _)| *b == id)
                                                .map_or(BookSnapshot {
                                                    completed: 0,
                                                    total: book.total_chapters as u32,
                                                    percent: 0,
                                                }, |(_, s)| s.clone())
                                        };
                                        view! {
                                            <button
                                                type="button"
                                                class="book-card"
                                                on:click=move |_| open_book.run(id)
                                            >
                                                <div class="book-card__head">
                                                    <h3 class="book-card__title">{book.title.clone()}</h3>
                                                    <ArrowRight size=16 attr:class="book-card__arrow" />
                                                </div>
                                                <p class="book-card__author">
                                                    {book.author.unwrap_or_else(|| "Unknown author".to_string())}
                                                </p>
                                                <div class="book-card__progress">
                                                    <ProgressBar
                                                        completed=snapshot().completed
                                                        total=snapshot().total
                                                    />
                                                </div>
                                                <div class="book-card__meta">
                                                    <span class="numeric">{snapshot().completed} " / " {snapshot().total}</span>
                                                    <span>" chapters"</span>
                                                </div>
                                            </button>
                                        }
                                    }
                                />
                            </section>
                        }
                    >
                        <section class="empty">
                            <BookOpen size=32 attr:class="empty__icon" />
                            <h3 class="empty__title">"No books yet"</h3>
                            <p class="empty__copy">"Add your first book to start tracking chapters and notes."</p>
                            <BaseButton on_click=Callback::new(move |_| show_add_modal.set(true))>
                                <Plus size=16 />
                                "Add book"
                            </BaseButton>
                        </section>
                    </Show>
                }
            >
                <section class="dashboard__loading">
                    <BaseLoader size=28 />
                </section>
            </Show>

            <BaseModal open=Signal::derive(move || show_add_modal.get()) on_close=Callback::new(close_modal) title="Add book">
                <form class="dashboard__form" aria-label="Add book" on:submit=move |ev| {
                    ev.prevent_default();
                    add_book(ev);
                }>
                    <BaseInput
                        value=Signal::derive(move || new_title.get())
                        on_input=Callback::new(move |v: String| new_title.set(v))
                        label="Title *"
                        placeholder="e.g. Atomic Habits"
                    />
                    <BaseInput
                        value=Signal::derive(move || new_author.get())
                        on_input=Callback::new(move |v: String| new_author.set(v))
                        label="Author"
                        placeholder="e.g. James Clear"
                    />
                    <Show when=move || !add_error.get().is_empty() fallback=|| view! { <span class="visually-hidden">""</span> }>
                        <p class="dashboard__form-error">{add_error}</p>
                    </Show>
                    <div class="form-actions">
                        <BaseButton
                            variant=crate::components::common::base_button::ButtonVariant::Secondary
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
