# Contributing to Tome

Thanks for your interest in contributing to **Tome** — a dark-first technical reading
tracker built with Leptos 0.8 (CSR), Supabase, and Trunk.

This guide covers how to set up your environment, the conventions we follow, and how to
get your changes merged. For architecture and UI/UX details, read
[AGENTS.md](./AGENTS.md) and [DESIGN.md](./DESIGN.md) first.

## Prerequisites

- [Rust](https://rustup.rs) — the repo pins a minimum version via `rust-version` in
  `Cargo.toml`. Use a recent stable toolchain (`rustup update stable`).
- The `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [Trunk](https://trunkrs.dev) — the WASM bundler/dev server:
  ```bash
  cargo install trunk
  ```
- A [Supabase](https://supabase.com) project (only needed to run the app locally).

## Project setup

```bash
# Clone and enter the repo
git clone https://github.com/vate-ps/tome
cd tome

# Configure environment
cp .env.example .env
#   Edit .env and set SUPABASE_URL and SUPABASE_ANON_KEY

# Apply the database schema in the Supabase SQL Editor
#   Run supabase-schema.sql (idempotent — safe to re-run)
```

## Development workflow

```bash
# Type-check only (no codegen)
cargo check --target wasm32-unknown-unknown

# Lint
cargo clippy --target wasm32-unknown-unknown -- -D clippy::correctness -D clippy::suspicious

# Check formatting
cargo fmt --all --check

# Run library unit tests
cargo test --lib

# Start the dev server with HMR (http://127.0.0.1:3000)
trunk serve --port 3000 --open

# Production build → dist/
trunk build --release
```

> CI runs `cargo check`, `cargo clippy`, `cargo fmt --check`, `cargo test`, and a
> `trunk build --release` on every push/PR (see `.github/workflows/ci.yml`). All jobs
> must pass before a PR can be merged.

## Code style & conventions

- **Formatting:** Run `cargo fmt` before committing. The project uses `rustfmt.toml`
  with 2-space indentation and `edition = "2024"`. The repo is strict about formatting —
  `cargo fmt --check` is enforced in CI.
- **Lints:** `unsafe_code` and `unused_must_use` are denied at the crate level; clippy
  `all` / `pedantic` / `nursery` are set to `warn`. Treat clippy warnings as errors for
  correctness/suspicious lints (`-D clippy::correctness -D clippy::suspicious`).
- **Edition:** Rust 2024.
- **No `unsafe`:** the crate denies `unsafe_code`; keep it that way.
- **TypeScript-style strictness:** write explicit types for public function signatures,
  props, and API payloads. Avoid `any`/`unwrap()` in production paths — use the
  `crate::core::error` types.
- **Pure CSS:** styling lives in `public/styles/` with design tokens from `DESIGN.md`.
  No CSS-in-JS, no UI frameworks. Prefer `<style scoped>` for component styles.
- **Icons:** use the inline Lucide SVG components in `src/components/icons.rs`. Do not
  add new icon libraries.
- **Architecture:** follow the module layout in [AGENTS.md](./AGENTS.md):
  - `core/` — Supabase client, PostgREST, auth, markdown, highlight, error
  - `stores/` — reactive Pinia-style contexts (auth, books, progress, notes)
  - `components/` — `common/`, `editor/`, `layout/`, `progress/`, `review/`
  - `views/` — page-level views wired through `src/views/router.rs`

## Database changes

All schema lives in `supabase-schema.sql` and uses the `reading_` prefix. When you change
the schema:

1. Keep the script **idempotent** (safe to re-run — use `DROP ... IF EXISTS`,
   `CREATE OR REPLACE`, etc.).
2. Enable RLS on every new table with `auth.uid() = user_id` policies.
3. Add indexes on foreign keys (`user_id`, `chapter_id`, `book_id`).
4. Update `src/core/types/database.rs` if column/row shapes change.

## Commit messages

We follow [Conventional Commits](https://www.conventionalcommits.org). Use lowercase
scopes where helpful:

```
build:    bump rust-version to 1.97
feat:     add flashcard bulk import
fix:      correct chapter-select reactive loop
style:    reformat codebase with 2-space indent
refactor: extract supabase client into core/
test:     add unit tests for sm-2 scheduling
docs:     document RLS policies in CONTRIBUTING.md
ci:       update trunk build --release
```

## Pull requests

1. Fork and create a feature branch off `main` (e.g. `feat/flashcard-import`).
2. Keep PRs focused — one logical change per PR.
3. Ensure local `cargo fmt --check`, `cargo clippy`, and `cargo test` all pass.
4. Describe what the change does and why; link any related issues.
5. Add screenshots/GIFs for UI changes where possible.
6. CI must be green before review.

## Reporting bugs & security issues

- For general bugs, open a GitHub issue with reproduction steps, expected vs. actual
  behavior, and your environment (OS, Rust version, browser).
- For security concerns (XSS, auth bypass, RLS gaps), please report privately rather
  than opening a public issue. See the Security section in [README.md](./README.md).

## License

By contributing, you agree that your contributions will be licensed under the
[MIT License](./LICENSE).
