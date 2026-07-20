# Agent Instructions: Technical Reading Tracker Application

This document outlines the architecture, database schema, state management, and
implementation guidelines for building **Tome** — a dark-first technical reading tracker.

> **Note:** This project is built with **Rust + Leptos 0.8 (CSR)**, **Trunk**, and
> **Supabase** — not a JS framework. Earlier drafts of this file described a Vue/Pinia
> stack; that no longer applies. Follow the conventions below, which match the actual
> codebase in `src/`.

All agents (Frontend, Backend, and QA) must adhere to these specifications, using this
file together with the UI/UX specifications in `DESIGN.md` and the developer guide in
`CONTRIBUTING.md`.

---

## 1. System Overview & Tech Stack

Tome is a structured tracker for technical books (e.g. *The Rust Book*). It focuses on
hierarchical progress tracking, rich markdown note-taking with code highlighting, active
recall (flashcards), and focus sessions. It is a **client-side rendered (CSR) WASM app**.

### Core Stack
- **Language:** Rust (edition 2024, stable toolchain; minimum `rust-version` in `Cargo.toml`)
- **Frontend Framework:** Leptos 0.8 (Composition API style via signals/contexts, CSR mode)
- **Build Tool:** Trunk (WASM bundler/dev server)
- **Target:** `wasm32-unknown-unknown`
- **Styling:** Pure CSS with design tokens (no Tailwind, no CSS framework)
- **Icons:** Inline Lucide SVG components in `src/components/icons.rs`
- **Backend-as-a-Service:** Supabase (Postgres, Auth/GoTrue, Storage)

### Key Crates
- `leptos`, `leptos_meta`, `leptos_router` (0.8, CSR features)
- `gloo-net` (HTTP/`fetch`), `gloo-storage`, `gloo-utils`
- `serde` / `serde_json`
- `chrono`, `uuid` (wasm/js), `web-time`
- `pulldown-cmark` (markdown → HTML) + `ammonia` (HTML sanitization)
- `console_log`, `console_error_panic_hook`, `log`
- `thiserror`, `url`, `js-sys`, `wasm-bindgen` / `wasm-bindgen-futures`, `web-sys`

---

## 2. Supabase Database Schema (PostgreSQL)

All tables use the `reading_` prefix. RLS is enabled on every table so users can only
read/write their own rows (`auth.uid() = user_id`). The full, **idempotent** schema lives
in `supabase-schema.sql` (run it in the Supabase SQL Editor; safe to re-run).

### `reading_profiles`
Linked to Supabase Auth; auto-created by a `handle_new_user()` trigger on signup.
```sql
create table reading_profiles (
  id uuid references auth.users on delete cascade primary key,
  updated_at timestamp with time zone,
  username text unique,
  avatar_url text,
  constraint username_length check (char_length(username) >= 3)
);
```

### `reading_books`
Standard or custom technical books added by users.
```sql
create table reading_books (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  title text not null,
  author text,
  total_chapters integer not null default 0,  -- cached count via trigger
  created_at timestamp with time zone default timezone('utc'::text, now()) not null
);
```

### `reading_chapters`
Structural chapters / sub-chapters of a book (supports nesting via `parent_id`).
```sql
create table reading_chapters (
  id uuid default gen_random_uuid() primary key,
  book_id uuid references reading_books(id) on delete cascade not null,
  title text not null,
  sequence_number decimal not null,  -- allows 1.1, 1.2, ...
  parent_id uuid references reading_chapters(id) on delete cascade
);
```

### `reading_progress`
Per-user reading status for each chapter.
```sql
create type reading_status as enum ('not_started', 'in_progress', 'completed', 'review_needed');

create table reading_progress (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  chapter_id uuid references reading_chapters(id) on delete cascade not null,
  status reading_status default 'not_started'::reading_status not null,
  time_spent_seconds integer default 0 not null,
  updated_at timestamp with time zone default timezone('utc'::text, now()) not null,
  unique (user_id, chapter_id)
);
```

### `reading_notes`
Markdown notes per chapter (rendered through `ammonia` before injection).
```sql
create table reading_notes (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  chapter_id uuid references reading_chapters(id) on delete cascade not null,
  content text not null,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null,
  updated_at timestamp with time zone default timezone('utc'::text, now()) not null,
  unique (user_id, chapter_id)
);
```

### `reading_flashcards`
Active recall cards (SM-2-inspired scheduling: `ease_factor` + `interval_days`).
```sql
create table reading_flashcards (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  chapter_id uuid references reading_chapters(id) on delete cascade not null,
  front text not null,
  back text not null,
  next_review timestamp with time zone default timezone('utc'::text, now()) not null,
  interval_days integer default 0 not null,
  ease_factor double precision default 2.5 not null,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null
);
```

### Triggers & RPCs (in `supabase-schema.sql`)
- `handle_new_user()` — auto-creates a `reading_profiles` row on signup.
- `update_updated_at_column()` — maintains `updated_at` on relevant tables.
- `sync_reading_book_total_chapters()` — keeps `reading_books.total_chapters` cached.
- `get_dashboard_summary()` — RPC powering the dashboard (progress + cards due).

---

## 3. State Management (Leptos Contexts/Signals)

There is no Pinia. State is held in Leptos `RwSignal`s/`Memo`s installed as a context via
`provide_context` and read with `expect_context`. Stores are "installed" once at the
mount root in `src/lib.rs` (inside `mount_to_body`) so their signal owners are never
disposed.

### `stores/auth.rs`
- **State:** current session/user, profile, loading.
- **Actions:** `sign_in()`, `sign_up()`, `sign_out()`, `fetch_profile()`.

### `stores/books.rs`
- **State:** `books`, `current_book`, `chapters`, `loading`.
- **Actions:** `fetch_books()`, `add_book()`, `fetch_chapters(book_id)`.

### `stores/progress.rs`
- **State:** progress keyed by `chapter_id`.
- **Actions:** `update_status(chapter_id, status)`, `log_time_spent(chapter_id, seconds)`.

### `stores/notes.rs`
- **State:** notes keyed by `chapter_id`.
- **Actions:** `fetch_note(chapter_id)`, `save_note(chapter_id, content)`.

---

## 4. Frontend Component & Directory Architecture

Clean, modular Rust/Leptos code following the project's module layout. `view!` macros
expand to a lot of code, so some clippy lints on UI files are explicitly allowed in
`Cargo.toml` (see `[lints]`).

```text
src/
├── lib.rs                # Crate root + WASM entry point (mount_to_body, install stores)
├── app.rs                # Root App component + top-level layout
├── components/
│   ├── common/           # BaseButton, BaseInput, BaseTextarea, BaseModal, BaseLoader
│   ├── editor/           # MarkdownEditor
│   ├── icons.rs          # Inline Lucide SVG icon components
│   ├── layout/           # AppTopbar (responsive nav)
│   ├── progress/         # ChapterList, ProgressBar
│   └── review/           # FlashcardContainer, PomodoroTimer
├── composables/          # Reusable logic (use_timer, use_markdown)
├── core/                 # Supabase client, PostgREST, auth, markdown, highlight, error
│   ├── types/            # database.rs (row types), mod.rs
│   ├── supabase.rs       # Client init from env (SUPABASE_URL, SUPABASE_ANON_KEY)
│   ├── postgrest.rs      # Typed REST queries
│   ├── auth.rs           # GoTrue wrapper
│   ├── markdown.rs       # pulldown-cmark + ammonia pipeline
│   ├── highlight.rs      # Keyword-based code highlighting
│   ├── error.rs          # thiserror error types
│   ├── time.rs / utils.rs
│   └── mod.rs
├── stores/               # Reactive contexts: auth, books, progress, notes
└── views/                # Dashboard, Book, Review, Login, Register, NotFound, Router
```

---

## 5. Pure CSS & Styling Guidelines

No CSS-in-JS, no UI framework. Stylesheets live in `public/styles/`:

- `variables.css` — design tokens (colors, typography, spacing) from `DESIGN.md`, under
  `:root` custom properties (e.g. `--color-primary`, `--font-sans`, `--spacing-md`).
- `reset.css` — CSS reset/normalization.
- `main.css` — global layout & utility classes.
- `highlight.css` — code highlighting theme.

Rules:
1. **Design tokens** come from `DESIGN.md` and live in `variables.css`.
2. **Scoping:** prefer component-scoped classes / BEM conventions; avoid style leakage.
3. **Layout:** use Flexbox/Grid; avoid hardcoded pixel widths for layout.
4. **Responsive:** respect the breakpoints defined in `DESIGN.md`.
5. **Accessibility:** semantic HTML + ARIA, `:focus-visible` outlines,
   `prefers-reduced-motion` support, `aria-label` on icon-only buttons.

---

## 6. Development Workflow & Agent Instructions

### General
- Read `DESIGN.md` (UI/UX) and `CONTRIBUTING.md` (setup/PRs) before generating code.
- Run `cargo fmt` (2-space indent, `edition = "2024"`) before committing — enforced in CI.
- Run `cargo clippy --target wasm32-unknown-unknown -- -D clippy::correctness -D clippy::suspicious`.
- The crate denies `unsafe_code` and `unused_must_use` (`#![deny(...)]` in `lib.rs`).
- Prefer typed errors from `core::error` over `unwrap()` in production paths.

### Frontend Agent (Leptos)
- Use the `view!` macro for markup; keep signal updates inside event handlers.
- Install reactive state via context (`provide_context`/`expect_context`), never globals.
- Use Lucide icons from `src/components/icons.rs`.
- Keep `view!` expansions lint-clean; reliance on allowed clippy exceptions is fine.

### Backend / Supabase Agent
- Edit `supabase-schema.sql`; keep it idempotent and safe to re-run.
- Enable RLS on every new table with `auth.uid() = user_id` policies.
- Index foreign keys (`user_id`, `chapter_id`, `book_id`).
- Update `src/core/types/database.rs` when row shapes change.

### Markdown / Sanitization
- Notes are rendered via `pulldown-cmark` then sanitized with `ammonia` before injection
  (XSS protection). Never inject raw user HTML.

### Integration Steps
1. `rustup target add wasm32-unknown-unknown` and install `trunk`.
2. `cp .env.example .env` and set `SUPABASE_URL` / `SUPABASE_ANON_KEY`.
3. Run `supabase-schema.sql` in the Supabase SQL Editor.
4. `trunk serve --port 3000 --open` for development; `trunk build --release` for prod.
5. CI (`.github/workflows/ci.yml`) runs `cargo check`, `clippy`, `fmt --check`,
   `cargo test --lib`, and a `trunk build --release`.

### Commit & PR conventions
- Follow [Conventional Commits](https://www.conventionalcommits.org)
  (`feat:`, `fix:`, `build:`, `style:`, `refactor:`, `test:`, `docs:`, `ci:`).
- One logical change per PR; keep CI green before requesting review.
