//! Book view: chapter list, status pills, timer and markdown editor.

use crate::components::common::base_button::{BaseButton, ButtonSize, ButtonVariant};
use crate::components::common::base_input::BaseInput;
use crate::components::common::base_loader::BaseLoader;
use crate::components::common::base_modal::BaseModal;
use crate::components::editor::markdown_editor::MarkdownEditor;
use crate::components::icons::{ArrowLeft, Pause, Play, Plus, RotateCcw, Save};
use crate::components::progress::chapter_list::ChapterList;
use crate::components::progress::progress_bar::ProgressBar;
use crate::composables::use_timer::use_timer;
use crate::core::types::{Chapter, ReadingStatus};
use crate::core::utils;
use crate::stores::books::BooksState;
use crate::stores::notes::NotesState;
use crate::stores::progress::ProgressState;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

const STATUS_OPTIONS: [(ReadingStatus, &str); 4] = [
  (ReadingStatus::NotStarted, "Not started"),
  (ReadingStatus::InProgress, "Reading"),
  (ReadingStatus::Completed, "Done"),
  (ReadingStatus::ReviewNeeded, "Review"),
];

fn next_status(current: ReadingStatus) -> ReadingStatus {
  let idx = STATUS_OPTIONS
    .iter()
    .position(|(s, _)| *s == current)
    .unwrap_or(0);
  STATUS_OPTIONS[(idx + 1) % STATUS_OPTIONS.len()].0
}

fn prev_status(current: ReadingStatus) -> ReadingStatus {
  let idx = STATUS_OPTIONS
    .iter()
    .position(|(s, _)| *s == current)
    .unwrap_or(0);
  STATUS_OPTIONS[(idx + STATUS_OPTIONS.len() - 1) % STATUS_OPTIONS.len()].0
}

/// Book detail page.
#[component]
pub fn BookView() -> impl IntoView {
  let books_store = BooksState::use_ctx();
  let progress_store = ProgressState::use_ctx();
  let notes_store = NotesState::use_ctx();
  let params = use_params_map();

  // Memoised flat chapter list — `flat_chapters()` clones the whole tree, so
  // derive it once per `chapters` change instead of on every render.
  let flat = Memo::new(move |_| books_store.flat_chapters());
  let total = move || flat.get().len() as u32;
  let completed = move || {
    let progress = progress_store;
    flat
      .get()
      .iter()
      .filter(|chapter| {
        progress
          .get(chapter.id)
          .is_some_and(|p| p.status == ReadingStatus::Completed)
      })
      .count() as u32
  };

  let book_id = move || {
    params
      .get()
      .get("id")
      .and_then(|raw| uuid::Uuid::parse_str(raw.as_str()).ok())
  };

  let selected: RwSignal<Option<Chapter>> = RwSignal::new(None);
  let note_content = RwSignal::new(String::new());
  let loaded_note_content = RwSignal::new(String::new());
  let note_dirty = RwSignal::new(false);
  let saving_note = RwSignal::new(false);
  let show_add_chapter_modal = RwSignal::new(false);
  let new_chapter_title = RwSignal::new(String::new());
  let new_chapter_seq = RwSignal::new(String::new());
  let new_chapter_parent_id = RwSignal::new(String::new());
  let adding_chapter = RwSignal::new(false);
  let add_chapter_error = RwSignal::new(String::new());
  let view_error = RwSignal::new(String::new());

  let show_toc_modal = RwSignal::new(false);
  let toc_text = RwSignal::new(String::new());
  let toc_count = RwSignal::new(0_usize);
  let importing_toc = RwSignal::new(false);
  let toc_error = RwSignal::new(String::new());

  let disposed = RwSignal::new(false);
  on_cleanup(move || disposed.set(true));

  let timer = use_timer();
  let timer_seconds = timer.seconds;
  let timer_running = timer.running;

  let current_status = move || {
    selected
      .get()
      .as_ref()
      .and_then(|chapter| progress_store.get(chapter.id))
      .map(|p| p.status)
  };
  let selected_progress = move || {
    selected
      .get()
      .as_ref()
      .and_then(|chapter| progress_store.get(chapter.id))
  };

  let chapter_time_label =
    move || utils::format_duration_human(selected_progress().map_or(0, |p| p.time_spent_seconds));

  // Mark note dirty when content diverges from loaded value.
  Effect::new(move |_| {
    if note_content.get() == loaded_note_content.get() {
      note_dirty.set(false);
    } else {
      note_dirty.set(true);
    }
  });

  let flush_timer_for_chapter = move |chapter_id: uuid::Uuid| {
    if timer_seconds.get() == 0 {
      return;
    }
    let seconds = timer_seconds.get();
    let disposed = disposed;
    leptos::task::spawn_local(async move {
      let _ = progress_store.log_time(chapter_id, seconds as i32).await;
      if !disposed.get_untracked() {
        timer.reset.run(());
      }
    });
  };

  let load_book = move |book_id_value: uuid::Uuid| {
    view_error.set(String::new());
    let selected_value = selected.get();
    if let Some(current) = selected_value.as_ref() {
      if note_dirty.get() {
        let confirmed = web_sys::window()
          .and_then(|w| {
            w.confirm_with_message("You have unsaved notes. Discard them?")
              .ok()
          })
          .unwrap_or(false);
        if !confirmed {
          return;
        }
      }
      flush_timer_for_chapter(current.id);
    }
    leptos::task::spawn_local(async move {
      let result: Result<(), String> = async {
        // Book, chapters and progress are independent — fetch them concurrently.
        let (book_res, chapters_res, progress_res) = futures::join!(
          books_store.fetch_book(book_id_value),
          books_store.fetch_chapters(book_id_value),
          progress_store.fetch_for_book(book_id_value),
        );
        let book = book_res.map_err(|e| e.to_string())?;
        chapters_res.map_err(|e| e.to_string())?;
        progress_res.map_err(|e| e.to_string())?;
        if disposed.get_untracked() {
          return Ok(());
        }
        if book.is_some() {
          books_store.current_book_id.set(Some(book_id_value));
        }
        let available = books_store.flat_chapters();
        let next = available
          .iter()
          .find(|chapter| Some(chapter.id) == selected.get_untracked().map(|c| c.id))
          .cloned()
          .or_else(|| available.first().cloned());
        if let Some(chapter) = next {
          let chapter_id = chapter.id;
          if let Some(book) = books_store.book(book_id_value) {
            books_store.mark_opened(book_id_value, &book.title, &chapter);
          }
          selected.set(Some(chapter.clone()));
          let note = notes_store
            .fetch(chapter_id)
            .await
            .map_err(|e| e.to_string())?;
          if disposed.get_untracked() {
            return Ok(());
          }
          loaded_note_content.set(note.as_ref().map(|n| n.content.clone()).unwrap_or_default());
          note_content.set(note.as_ref().map(|n| n.content.clone()).unwrap_or_default());
          note_dirty.set(false);
          timer.reset.run(());
        } else {
          selected.set(None);
          loaded_note_content.set(String::new());
          note_content.set(String::new());
          note_dirty.set(false);
        }
        Ok::<(), String>(())
      }
      .await;
      if !disposed.get_untracked() {
        if let Err(err) = result {
          view_error.set(err);
        }
      }
    });
  };

  Effect::new(move |_| {
    if let Some(id) = book_id() {
      untrack(move || load_book(id));
    }
  });

  let select_chapter = move |chapter: Chapter| {
    let chapter_id = chapter.id;
    if let Some(book_id) = book_id() {
      if let Some(book) = books_store.book(book_id) {
        books_store.mark_opened(book_id, &book.title, &chapter);
      }
    }
    if let Some(current) = selected.get_untracked() {
      if note_dirty.get_untracked() && current.id != chapter.id {
        let confirmed = web_sys::window()
          .and_then(|w| {
            w.confirm_with_message("You have unsaved notes. Discard them?")
              .ok()
          })
          .unwrap_or(false);
        if !confirmed {
          return;
        }
      }
      if current.id != chapter.id {
        flush_timer_for_chapter(current.id);
      }
    }
    selected.set(Some(chapter));
    note_dirty.set(false);
    leptos::task::spawn_local(async move {
      match notes_store.fetch(chapter_id).await {
        Ok(Some(note)) => {
          if disposed.get_untracked() {
            return;
          }
          let content = note.content;
          loaded_note_content.set(content.clone());
          note_content.set(content);
        }
        Ok(None) => {
          if disposed.get_untracked() {
            return;
          }
          loaded_note_content.set(String::new());
          note_content.set(String::new());
        }
        Err(err) => {
          if !disposed.get_untracked() {
            view_error.set(err.to_string());
          }
        }
      }
      if !disposed.get_untracked() {
        timer.reset.run(());
      }
    });
  };

  let update_status = move |status: ReadingStatus| {
    let chapter_id = match selected.get() {
      Some(c) => c.id,
      None => return,
    };
    leptos::task::spawn_local(async move {
      let _ = progress_store.update_status(chapter_id, status).await;
    });
  };

  let save_note = Callback::new(move |_: ()| {
    let chapter_id = match selected.get() {
      Some(c) => c.id,
      None => return,
    };
    let content = note_content.get();
    saving_note.set(true);
    leptos::task::spawn_local(async move {
      let result = notes_store.save(chapter_id, &content).await;
      if disposed.get_untracked() {
        return;
      }
      saving_note.set(false);
      match result {
        Ok(note) => {
          let content = note.content;
          loaded_note_content.set(content.clone());
          note_content.set(content);
          crate::composables::announce("Note saved");
        }
        Err(err) => view_error.set(err.to_string()),
      }
    });
  });

  let add_chapter = move |_: web_sys::SubmitEvent| {
    if adding_chapter.get() {
      return;
    }
    let title = new_chapter_title.get();
    let seq = new_chapter_seq.get();
    let parent = new_chapter_parent_id.get();
    let title_trim = title.trim().to_string();
    if title_trim.is_empty() {
      add_chapter_error.set("Title is required.".to_string());
      return;
    }
    let parsed_seq: f64 = match seq.parse() {
      Ok(value) => value,
      Err(_) => {
        add_chapter_error.set("Sequence number is required (e.g. 1, 1.1, 2).".to_string());
        return;
      }
    };
    let book_id_value = match book_id() {
      Some(id) => id,
      None => {
        add_chapter_error.set("No book selected.".to_string());
        return;
      }
    };
    let parent_id = if parent.is_empty() {
      None
    } else {
      uuid::Uuid::parse_str(&parent).ok()
    };
    adding_chapter.set(true);
    add_chapter_error.set(String::new());
    leptos::task::spawn_local(async move {
      let result = books_store
        .add_chapter(book_id_value, &title_trim, parsed_seq, parent_id)
        .await;
      if disposed.get_untracked() {
        return;
      }
      adding_chapter.set(false);
      match result {
        Ok(()) => {
          new_chapter_title.set(String::new());
          new_chapter_seq.set(String::new());
          new_chapter_parent_id.set(String::new());
          show_add_chapter_modal.set(false);
          crate::composables::announce("Chapter added");
        }
        Err(err) => add_chapter_error.set(err.to_string()),
      }
    });
  };

  // Live preview of how many chapters a pasted table of contents will create.
  let update_toc_count = move |_: web_sys::Event| {
    let parsed = crate::stores::books::parse_toc(&toc_text.get_untracked());
    toc_count.set(parsed.len());
  };

  let import_toc = move |_: web_sys::SubmitEvent| {
    if importing_toc.get() {
      return;
    }
    let book_id_value = match book_id() {
      Some(id) => id,
      None => {
        toc_error.set("No book selected.".to_string());
        return;
      }
    };
    let parsed = crate::stores::books::parse_toc(&toc_text.get_untracked());
    let inserts = crate::stores::books::toc_to_inserts(&parsed);
    if inserts.is_empty() {
      toc_error.set("Paste a list of chapter titles, one per line.".to_string());
      return;
    }
    importing_toc.set(true);
    toc_error.set(String::new());
    leptos::task::spawn_local(async move {
      let result = books_store.add_chapters_bulk(book_id_value, &inserts).await;
      if disposed.get_untracked() {
        return;
      }
      importing_toc.set(false);
      match result {
        Ok(n) if n > 0 => {
          toc_text.set(String::new());
          toc_count.set(0);
          show_toc_modal.set(false);
        }
        Ok(_) => toc_error.set("No valid chapters found in that text.".to_string()),
        Err(err) => toc_error.set(err.to_string()),
      }
    });
  };

  let log_session = move |_: web_sys::MouseEvent| {
    if let Some(chapter) = selected.get() {
      flush_timer_for_chapter(chapter.id);
    }
  };

  let selected_id = move || selected.get().map(|c| c.id);
  let chapters_signal = Signal::derive(move || books_store.chapters.get());

  // Persist timer on unmount.
  on_cleanup(move || {
    if let Some(chapter) = selected.get_untracked()
      && timer_seconds.get_untracked() > 0
    {
      let store = progress_store;
      let seconds = timer_seconds.get_untracked() as i32;
      let chapter_id = chapter.id;
      leptos::task::spawn_local(async move {
        let _ = store.log_time(chapter_id, seconds).await;
      });
    }
    disposed.set(true);
  });

  view! {
      <div class="page book">
          <header class="book__header">
              <A href="/" attr:class="book__back">
                  <ArrowLeft size=14 />
                  "Library"
              </A>

              <div class="book__title-row">
                  <div class="book__title-block">
                      <Show
                          when=move || books_store.current_book_id.get().and_then(|id| books_store.book(id)).is_some()
                          fallback=move || view! { <h1 class="book__title">"Book"</h1> }
                      >
                          <h1 class="book__title">
                              {move || {
                                  books_store
                                      .current_book_id
                                      .get()
                                      .and_then(|id| books_store.book(id))
                                      .map(|b| b.title)
                                      .unwrap_or_default()
                              }}
                          </h1>
                          <p class="book__author">
                              {move || {
                                  books_store
                                      .current_book_id
                                      .get()
                                      .and_then(|id| books_store.book(id))
                                      .and_then(|b| b.author)
                                      .unwrap_or_default()
                              }}
                          </p>
                      </Show>
                  </div>
                  <div class="book__title-actions">
                      <BaseButton
                          size=ButtonSize::Small
                          variant=ButtonVariant::Secondary
                          on_click=Callback::new(move |_| show_add_chapter_modal.set(true))
                      >
                          <Plus size=14 />
                          "Add chapter"
                      </BaseButton>
                      <BaseButton
                          size=ButtonSize::Small
                          variant=ButtonVariant::Ghost
                          on_click=Callback::new(move |_| show_toc_modal.set(true))
                      >
                          <Plus size=14 />
                          "Paste contents"
                      </BaseButton>
                  </div>
              </div>

              <div class="book__progress">
                  <ProgressBar completed=Signal::derive(completed) total=Signal::derive(total) />
                  <span class="book__progress-label numeric">{completed} " / " {total}</span>
              </div>
          </header>

          <Show when=move || !view_error.get().is_empty() fallback=|| view! { <span class="visually-hidden">""</span> }>
              <p class="notice">{view_error}</p>
          </Show>

          <div class="book__layout">
              <aside class="book__sidebar surface">
                  <Show
                      when=move || books_store.loading.get()
                      fallback=move || {
                          view! {
                              <Show
                                  when=move || chapters_signal.get().is_empty()
                                  fallback=move || view! {
                                      <ChapterList
                                          chapters=chapters_signal
                                          selected=Signal::derive(selected_id)
                                          on_select=Callback::new(select_chapter)
                                      />
                                  }
                              >
                                  <div class="book__empty">
                                      <p>"No chapters yet."</p>
                                      <BaseButton
                                          size=ButtonSize::Small
                                          variant=ButtonVariant::Secondary
                                          on_click=Callback::new(move |_| show_add_chapter_modal.set(true))
                                      >
                                          <Plus size=14 />
                                          "Add chapter"
                                      </BaseButton>
                                  </div>
                              </Show>
                          }
                      }
                  >
                      <div class="book__loading">
                          <BaseLoader />
                      </div>
                  </Show>
              </aside>

              <div class="book__workspace">
                  <Show
                      when=move || selected.get().is_some()
                      fallback=move || view! {
                          <div class="book__no-chapter surface">
                              <h3>"Select a chapter"</h3>
                              <p>"Pick a chapter from the sidebar to start writing notes."</p>
                          </div>
                      }
                  >
                      <div class="chapter-bar surface">
                          <div class="chapter-bar__head">
                              <h2 class="chapter-bar__title">
                                  <span class="chapter-bar__seq numeric">
                                      {move || selected.get().map(|c| c.sequence_number).unwrap_or_default()}
                                  </span>
                                  {move || selected.get().map(|c| c.title).unwrap_or_default()}
                              </h2>
                              <span class="chapter-bar__time numeric">{chapter_time_label} " logged"</span>
                          </div>

                          <div class="chapter-bar__row">
                              <div class="chapter-bar__pills" role="radiogroup" aria-label="Status">
                                  <For
                                      each=move || STATUS_OPTIONS.iter().copied()
                                      key=|(status, _)| *status
                                      children=move |(status, label): (ReadingStatus, &'static str)| {
                                          let value = status;
                                          let active = move || {
                                              current_status().unwrap_or(ReadingStatus::NotStarted) == value
                                          };
                                          view! {
                                              <button
                                                  type="button"
                                                  role="radio"
                                                  class="status-pill"
                                                  class:is-active=active
                                                  attr:data-status=match value {
                                                      ReadingStatus::NotStarted => "not_started",
                                                      ReadingStatus::InProgress => "in_progress",
                                                      ReadingStatus::Completed => "completed",
                                                      ReadingStatus::ReviewNeeded => "review_needed",
                                                  }
                                                  aria-checked=move || active().to_string()
                                                  tabindex=move || if active() { 0_i32 } else { -1_i32 }
                                                  on:click=move |_| update_status(value)
                                                  on:keydown=move |ev| match ev.key().as_str() {
                                                      "ArrowLeft" => {
                                                          ev.prevent_default();
                                                          update_status(prev_status(value));
                                                      },
                                                      "ArrowRight" => {
                                                          ev.prevent_default();
                                                          update_status(next_status(value));
                                                      },
                                                      _ => {},
                                                  }
                                              >
                                                  {label}
                                              </button>
                                          }
                                      }
                                  />
                              </div>

                              <div class="chapter-bar__timer" role="group" aria-label="Session timer">
                                  <span
                                      class="chapter-bar__clock numeric"
                                      role="timer"
                                      aria-label=move || format!("Elapsed: {}", timer.format.run(timer_seconds.get()))
                                  >
                                      {move || timer.format.run(timer_seconds.get())}
                                  </span>
                                  <button
                                      class="timer-btn"
                                      type="button"
                                      on:click=move |_| timer.reset.run(())
                                      title="Reset"
                                      aria-label="Reset timer"
                                  >
                                      <RotateCcw size=14 />
                                  </button>
                                  <button
                                      class="timer-btn timer-btn--primary"
                                      type="button"
                                      title=move || if timer_running.get() { "Pause" } else { "Start" }
                                      aria-label=move || if timer_running.get() { "Pause timer" } else { "Start timer" }
                                      on:click=move |_| {
                                          if timer_running.get() {
                                              timer.pause.run(());
                                          } else {
                                              timer.start.run(());
                                          }
                                      }
                                  >
                                      <Show
                                          when=move || timer_running.get()
                                          fallback=move || view! { <Play size=14 /> }
                                      >
                                          <Pause size=14 />
                                      </Show>
                                  </button>
                                  <button
                                      class="timer-btn"
                                      type="button"
                                      disabled=move || timer_seconds.get() == 0
                                      title="Log session"
                                      aria-label="Log session"
                                      on:click=log_session
                                  >
                                      <Save size=14 />
                                  </button>
                              </div>
                              <p class="chapter-bar__note">
                                  "Time is logged to this chapter automatically when you switch away."
                              </p>
                          </div>
                      </div>

                      <MarkdownEditor
                          value=Signal::derive(move || note_content.get())
                          on_input=Callback::new(move |v: String| note_content.set(v))
                          dirty=note_dirty.get_untracked()
                          saving=saving_note.get_untracked()
                          on_save=save_note
                      />
                  </Show>
              </div>
          </div>

          <BaseModal
              open=Signal::derive(move || show_add_chapter_modal.get())
              on_close=Callback::new(move |_| show_add_chapter_modal.set(false))
              title="Add chapter"
          >
              <form class="book__form" on:submit=move |ev| {
                  ev.prevent_default();
                  add_chapter(ev);
              }>
                  <BaseInput
                      value=Signal::derive(move || new_chapter_title.get())
                      on_input=Callback::new(move |v: String| new_chapter_title.set(v))
                      label="Title *"
                      placeholder="Getting started"
                  />
                  <BaseInput
                      value=Signal::derive(move || new_chapter_seq.get())
                      on_input=Callback::new(move |v: String| new_chapter_seq.set(v))
                      label="Sequence number *"
                      input_type="text"
                      inputmode="decimal"
                      placeholder="e.g. 1 or 1.1"
                  />

                  <div class="book__select-group">
                      <label class="book__select-label" for="parent-chapter">"Parent chapter"</label>
                      <select
                          id="parent-chapter"
                          class="book__select"
                          on:change=move |ev| {
                              new_chapter_parent_id.set(event_target_value(&ev));
                          }
                          prop:value=move || new_chapter_parent_id.get()
                      >
                          <option value="">"None (top level)"</option>
                          <For
                              each=move || flat.get()
                              key=|chapter| chapter.id
                              children=move |chapter: Chapter| {
                                  let value = chapter.id.to_string();
                                  let title = chapter.title.clone();
                                  let sequence = chapter.sequence_number;
                                  view! {
                                      <option value=value>
                                          {format!("{sequence} · {title}")}
                                      </option>
                                  }
                              }
                          />
                      </select>
                  </div>

                  <Show when=move || !add_chapter_error.get().is_empty() fallback=|| view! { <span class="visually-hidden">""</span> }>
                      <p class="book__form-error">{add_chapter_error}</p>
                  </Show>

                  <div class="form-actions">
                      <BaseButton
                          variant=ButtonVariant::Secondary
                          on_click=Callback::new(move |_| show_add_chapter_modal.set(false))
                      >
                          "Cancel"
                      </BaseButton>
                      <BaseButton button_type="submit" loading=adding_chapter.get_untracked()>
                          "Add chapter"
                      </BaseButton>
                  </div>
              </form>
          </BaseModal>

          <BaseModal
              open=Signal::derive(move || show_toc_modal.get())
              on_close=Callback::new(move |_| show_toc_modal.set(false))
              title="Add chapters from a table of contents"
          >
              <form class="book__form" aria-label="Import table of contents" on:submit=move |ev| {
                  ev.prevent_default();
                  import_toc(ev);
              }>
                  <p class="book__form-hint">
                      "Paste a list of chapter titles, one per line. Headings (#, ##),
                      indentation, and trailing page numbers are handled for you."
                  </p>
                  <textarea
                      class="book__toc"
                      rows=10
                      placeholder={"1. Getting Started\n2. Ownership\n  2.1 Borrowing\n3. Traits"}
                      prop:value=move || toc_text.get()
                      on:input=move |ev| {
                          toc_text.set(event_target_value(&ev));
                          update_toc_count(ev);
                      }
                  ></textarea>
                  <p class="book__form-count">
                      {move || toc_count.get()}
                      " chapters will be created"
                  </p>
                  <Show when=move || !toc_error.get().is_empty() fallback=|| view! { <span class="visually-hidden">""</span> }>
                      <p class="book__form-error">{toc_error}</p>
                  </Show>
                  <div class="form-actions">
                      <BaseButton
                          variant=ButtonVariant::Secondary
                          on_click=Callback::new(move |_| show_toc_modal.set(false))
                      >
                          "Cancel"
                      </BaseButton>
                      <BaseButton button_type="submit" loading=importing_toc.get_untracked()>
                          "Add chapters"
                      </BaseButton>
                  </div>
              </form>
          </BaseModal>
      </div>
  }
}
