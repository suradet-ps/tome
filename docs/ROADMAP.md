# Tome Roadmap

This roadmap describes what Tome is, honestly, from reading its own code - and
where it should end up. It follows the architecture in [AGENTS.md](AGENTS.md),
the conventions in [CONTRIBUTING.md](CONTRIBUTING.md), the security posture in
[SECURITY.md](SECURITY.md), and (once corrected — see Phase 1) the design system
in [DESIGN.md](DESIGN.md).

> **What Tome is.** A *quiet, personal* companion for reading technical books:
> one reader, their books, their notes, their memory. You add a book, break it
> into chapters, track how far you've gotten, write markdown notes, drill what
> you've read with spaced-repetition flashcards, and stay focused with a
> Pomodoro timer. Everything you see is yours and only yours — enforced by
> Supabase Row-Level Security, not by trust in the client.
>
> **What Tome is not.** Not a social reader, not a team tool, not a Goodreads.
> There is no sharing, no following, no public profile, and nothing in the data
> model points that way. The single-user, calm, offline-friendly shape is the
> product, not a stepping stone to something larger. Features that break that
> shape are listed under "Out of Scope" so the line is drawn on purpose.

Nothing here is called "done" on intent alone. The repo already has a real
5-job CI (`.github/workflows/ci.yml`: `check`, `clippy`, `fmt --check`,
`test`, gated `trunk build --release`); every phase's acceptance is checked
against it.

---

## Current State (verified against the repo, not assumed)

- **Stack**: Rust 2024 + Leptos 0.8 (CSR) + Trunk, `wasm32-unknown-unknown`,
  deployed to Vercel as static assets behind a CSP. Version `0.2.0` in
  `Cargo.toml`. No server of our own — the browser talks to Supabase's
  PostgREST + GoTrue directly with the anon key.
- **Security model**: the client is untrusted; **RLS (`auth.uid() = user_id`)
  on every `reading_*` table is the real boundary.** The anon key is public by
  design. `#![deny(unsafe_code)]` + `unused_must_use = "deny"` at crate level.
- **Schema** (`supabase-schema.sql`, idempotent): profiles (auto-created by
  `handle_new_user()`), books (`total_chapters` cached by trigger), chapters
  (nested via `parent_id`, decimal `sequence_number`), progress (status enum +
  time), notes (markdown, one per user+chapter), flashcards (SM-2 fields).
  `get_dashboard_summary()` RPC folds the dashboard into one round trip.
- **Core layer** (`src/core`): typed PostgREST builder, GoTrue wrapper, markdown
  pipeline (`pulldown-cmark` → `ammonia` sanitize before injection), typed
  `thiserror` errors, WASM-safe time helpers, keyword code highlighting.
- **State** (`src/stores`): auth / books / progress / notes as Leptos signals
  installed once at the mount root so owners survive route changes; each store
  has a `reset()` used on sign-out.
- **Views** (`src/views`): auth-guarded router, login/register, dashboard
  (parallelized fetches + RPC), book view (nested chapters, reactive progress
  bars, keyboard status radiogroup, add modals), review (due cards, SM-2
  reschedule, Pomodoro), markdown editor with debounced live preview.
- **A11y today**: ARIA roles, modal focus-trap + Escape, keyboard nav on the
  status radiogroup, `prefers-reduced-motion`, `visually-hidden` fallbacks.

### Gaps found while reading the repo (these shape the phases below)

1. **`DESIGN.md` is not Tome's design — it is Binance's.** The entire file
   describes Binance Yellow (#FCD535), trading green/red, markets tables,
   BinanceNova/BinancePlex, "FUNDS ARE SAFU". The code's CSS tokens
   (`--color-primary`, `--font-number`, `--radius-pill`, `--color-on-primary`,
   `--color-primary-active/-disabled`) are that Binance system renamed. **Tome
   has no visual identity of its own yet** — it borrowed a trading platform's.
   A calm reading tool cannot ship on a trading platform's voltage. This is the
   first thing to fix, not a cosmetic afterthought. (Phase 1.)
2. **Only one test file** (`src/core/highlight.rs`). The two things that must
   never silently break — the markdown **XSS sanitizer** and the **SM-2**
   scheduling math — have zero tests, and SM-2 is inlined in `review_view.rs`
   where it can't be tested without a DOM. (Phase 2.)
3. **~24 inline hex colors** leak past the token system in `public/styles/`,
   and one stray Binance token reference remains. A design system that isn't
   enforced isn't a system. (Phase 1.)
4. **One `.unwrap()` in a production path** (`review_view.rs:286`,
   `cards.get().first().unwrap()`), guarded by an outer check but not by the
   type system — the lone violation of AGENTS.md's "no unwrap outside tests".
   (Phase 3.)
5. **No offline story.** A reading tool is used on trains, planes, and bad
   café wifi, but every action needs a live Supabase round trip today. (Phase 6.)

---

## Phase 1: A Visual Identity That Is Tome's Own

Tome must look and feel like a quiet place to read, not a trading terminal.

- [x] **Rewrite `DESIGN.md` for Tome.** A warm, calm, low-voltage system for
  long focused reading and note-taking — closer to a paper-and-lamplight
  reading app than an exchange. Define the palette, type scale, spacing,
  radius, and elevation as *Tome's*, with a documented rationale (why these
  choices serve reading, not trading).
- [x] **Retune the tokens in `public/styles/variables.css`** to the new system.
  Keep the neutral token *names* the code already uses (`--color-primary`,
  `--font-number`, etc.) so no component churns, but replace the Binance
  *values* with Tome's. Remove the stray Binance token reference.
- [x] **Eliminate the ~24 inline hex colors**; every color routes through a
  token. Add a tiny CI style step that fails the build on a raw `#rrggbb` in
  `.css` / `.rs` view code (the design system, enforced, not aspirational).
- [x] **Dark-first, with a real light theme and a warm reading ("sepia") mode**,
  all three driven purely by token remaps under `[data-theme]` — no new hex.
- [x] **A distinct wordmark / favicon** so Tome is recognizable in a tab.

**Acceptance:** `DESIGN.md` describes Tome, not Binance; zero inline hex (CI
enforced); all three themes render from tokens alone.

## Phase 2: Trust the Two Things That Must Never Break

The XSS boundary and the memory math are the two places a silent regression
does real harm. They get tests before anything is built on top of them.

- [x] **Markdown sanitizer tests** (`core/markdown.rs`) — the top priority in
  this whole roadmap. `<script>`, `onerror=`, `javascript:` URLs, `<iframe>`,
  and raw HTML must all be stripped by the `ammonia` pass; `plain_summary`
  must never split a multi-byte char. Promoted to a **CI gate**: the sanitizer
  can never be weakened unnoticed.
- [x] **Extract SM-2 out of `review_view.rs` into a pure `core` function** and
  test it: quality `< 3` resets the interval, first/second interval seeding,
  the ease-factor floor (never below 1.3), and `next_review` date arithmetic.
- [x] **PostgREST query-builder tests** (`core/postgrest.rs`): assert the exact
  URL + query string each chainable method produces (operator prefixes, `in.()`
  wrapping, order direction, sorted key emission), so a refactor can't silently
  change a filter (with RLS a wrong-*user* filter fails closed, but a
  wrong-*column* filter quietly shows the wrong data). Header assertions are
  deferred — they live on `gloo-net`'s `RequestBuilder` and can't be inspected
  without a live request; the security-relevant part (the query) is covered.
- [x] **`build_chapter_tree` tests**: roots sorted by `sequence_number`, decimal
  ordering (1.1 < 1.2 < 2.0), children nesting, orphaned `parent_id` surviving
  as a root, flatten/build inverse. **These tests caught a real bug**: the
  original single-pass builder pushed a *stale clone* of a parent into `roots`
  before its children were attached, so nested chapters silently showed no
  children. Rewritten to assemble the tree bottom-up from a child-id map; the
  fix ships with this phase.
- [x] **Time / duration tests** (`core/time.rs`, `utils.rs`): ISO format/round-
  trip + offset normalization, duration boundaries (0s, 59s, 60s, hours),
  char-boundary truncation, clock zero-padding.

**Acceptance:** the sanitizer and SM-2 tests block merge in CI; every core
invariant above is covered. Test count went from 3 to 48; a real chapter-nesting
bug was found and fixed along the way.

## Phase 3: Correctness & Robustness

- [x] **Remove the `review_view.rs:286` `unwrap()`**: `FlashcardContainer` now
  takes an owned `Flashcard` (rendered via an `Option` map), so non-emptiness is
  a compile-time guarantee. Queue removal moved into the pure
  `core::srs::remove_card`, with tests that grading the last due card empties
  the queue without panicking.
- [x] **Optimistic-update rollback**: `update_status` reflects the new status
  immediately and rolls back to a snapshot on write failure; `log_time` and note
  `save` now surface their errors instead of swallowing them and only commit to
  the cache after the server confirms.
- [x] **Auth token expiry**: the refresh token is persisted; on session restore
  a 401/403 triggers a `grant_type=refresh_token` exchange before giving up, and
  `AuthState::try_recover_session` exposes the same recovery mid-session. Only a
  failed refresh clears the session and bounces to login.
- [x] **Note concurrency**: before saving a cached note, the server row is
  re-read and its `updated_at` compared; a newer server timestamp yields
  `AppError::Conflict` (a reload-first warning) rather than last-writer-wins.
- [x] **Input caps enforced both sides**: pure `core::validate` rejects note
  content (200k chars), titles (1–200), and authors (≤200) — counted by
  character, not byte — and matching `check` constraints in
  `db/supabase-schema.sql` make the caps a hard guarantee.

**Acceptance:** no `unwrap()` in any production path; a failed write never leaves
the UI lying about the database; expired sessions self-heal.

## Phase 4: The Reading Loop, Made Excellent

Deepen exactly the loop Tome already has — read, note, recall, focus, see
progress — without adding a second product.

- [x] **Notes that feel good to write**: editor niceties for long-form
  markdown — heading/list/code shortcuts, a wider live-preview on desktop and a
  clean stacked view on mobile, autosave with a clear saved/dirty indicator.
- [x] **Recall that respects the reader**: a calm review session UI, clear
  "cards due today / done", and gentle scheduling copy — no gamified streak
  pressure (that would break the quiet-tool promise).
- [x] **Progress that motivates quietly**: per-book and per-chapter progress,
  reading time surfaced honestly, a simple "recently read" entry point so
  reopening the app lands you where you left off.
- [x] **Focus that stays out of the way**: Pomodoro tied to the chapter you're
  on, time auto-logged on switch, no nagging.
- [x] **Frictionless capture**: fast add-book / add-chapter, and a way to paste
  a table of contents to create chapters in bulk rather than one at a time.

**Acceptance:** each improvement has a smoke test where pure logic exists and
is documented in the README; nothing here adds sharing, social, or multi-user.

## Phase 5: Accessibility & Reading Comfort

A reading tool that isn't comfortable to read in has failed at its one job.

- [x] **Keyboard-only pass across every view** (not just modals + radiogroup):
  chapter tree expand/collapse, flashcard grading, Pomodoro, editor — all
  reachable and operable, key map documented.
- [x] **Screen-reader pass**: `role="tree"`/`treeitem`/`aria-expanded` on the
  chapter tree, live-region announcements for async results (note saved, card
  graded), correct `alertdialog` labelling; verified once with VoiceOver + NVDA
  and logged in `docs/a11y-notes.md`.
- [x] **Reading comfort controls**: adjustable content width / line length,
  font-size, and the warm reading theme from Phase 1, all persisted.
- [x] **Contrast audit** of every theme against WCAG AA using Tome's tokens.
- [x] **`prefers-reduced-motion`** honored by every transition, verified.

**Acceptance:** keyboard-only + reduced-motion pass; one SR session logged;
all themes pass AA.

## Phase 6: Offline-First (the natural end-state for a reading tool)

Because Tome is already CSR static assets + Supabase, offline is a natural fit,
and readers are often offline. This is what makes Tome *yours* rather than
"yours, when the wifi is good."

- [ ] **PWA shell**: installable, app shell cached by a service worker, launches
  and renders without network.
- [ ] **Local-first data**: books, chapters, progress, notes, and due cards
  readable and editable offline from a local cache (IndexedDB), with writes
  queued.
- [ ] **Sync on reconnect**: queued writes flush to Supabase when back online,
  with a clear, non-destructive conflict resolution (never silently lose an
  edit made offline).
- [ ] **Honest connectivity UI**: distinguish "offline" from "request failed";
  a calm banner, not a toast storm.

**Acceptance:** reading, note-taking, and review all work with the network fully
off; reconnecting syncs without data loss, proven with a scripted offline→online
round trip.

## Phase 7: Performance Budgets (verified, not claimed)

- [ ] **Measure a baseline first** (WASM `.wasm` gzip/brotli size, cold
  first-paint, dashboard/book load, note-save latency) on a mid-tier device and
  a throttled network — record it in `docs/perf-baseline.md`.
- [ ] **Then set CI-enforced budgets** against that baseline (bundle size ceiling
  that fails the build; first-paint and load targets), calibrated to real
  numbers rather than guessed.
- [ ] **Over-render audit**: confirm the reactive graph doesn't recompute the
  chapter tree / progress bars on unrelated updates.
- [ ] **Re-enable `wasm-opt`** once the toolchain incompatibility that forced it
  off is resolved, measured against the size budget.

**Acceptance:** budgets enforced in CI; baseline doc exists; no regression
merges without a noted exception.

## Phase 8: Security & Supply-Chain Hardening

- [ ] **XSS suite is a permanent CI gate** (from Phase 2) — the single most
  important guard, since the client is untrusted by design.
- [ ] **Prove RLS with a second account**: demonstrate no `reading_*` row is
  readable or writable across users; document it.
- [ ] **CSP tightened**: audit `vercel.json` against what the app actually loads;
  remove any `unsafe-inline`/`unsafe-eval` left from the bootstrap workaround;
  report-only pass before enforcing.
- [ ] **Document the anon-key posture** in SECURITY.md so no contributor mistakes
  the public key for a secret.
- [ ] **`cargo audit` + `cargo deny`** jobs in CI (advisories, licenses,
  yanked/duplicate crates), kept green.
- [ ] **`#![deny(unsafe_code)]` stays**; any future exception is justified,
  isolated, tested, and noted.

**Acceptance:** XSS + RLS proven and CI-guarded; CSP has no `unsafe-*`;
audit/deny green.

## Phase 9: First Stable Release (v1.0.0)

- [ ] **Reproducible build documented**: exact toolchain, Trunk version, env
  inputs → the same `dist/` from a given commit.
- [ ] **Vercel preview on every PR** exercising CSP + SPA fallback, so a
  header/rewrite regression is caught before `main`.
- [ ] **Branch protection on `main`**: strict required status checks, no
  force-push, no deletion — matched to the CI gates.
- [ ] **User-facing getting-started** (sign up → add a book → notes → review),
  and an explicit offline-first + no-telemetry privacy statement.
- [ ] **`v1.0.0` tag** once Phases 1–8 acceptance checks pass, CHANGELOG cut
  with git-cliff.

**Acceptance:** a tagged, reproducible release; branch protection live; docs
match the app.

---

## How the phases relate

```
Phase 1 (Tome's own identity)  ─┐
Phase 2 (trust XSS + SM-2)     ─┤ foundation — do these first
Phase 3 (correctness)          ─┘
        │
        ▼
Phase 4 (deepen the reading loop) ─┬─► Phase 5 (a11y + comfort)
                                   └─► Phase 6 (offline-first)
        │
        ▼
Phase 7 (perf budgets) ──needs──► existing CI to enforce
        │
        ▼
Phase 8 (security hardening)
        │
        ▼
Phase 9 (v1.0.0)
```

Phase 1 comes first on purpose: Tome cannot ship on a borrowed trading-platform
skin. Phase 2 comes with it because the sanitizer and SM-2 are the two things a
silent regression hurts most. Everything after is deepening the one loop Tome
has, never adding a second product.

---

## Out of Scope (drawn on purpose, to stay a quiet personal tool)

Each of these is valuable *for a different product*. Tome stays small and
single-user on purpose:

- **Sharing / social / following / public profiles** — Tome is one reader's
  private space; the data model has no notion of another reader, and it stays
  that way.
- **Team / collaborative editing** — out of scope; conflicts with single-user
  RLS and the calm posture.
- **A full ebook reader / PDF viewer** — Tome tracks *your reading of* a book;
  it is not where you read the book itself.
- **AI summarization / chat over notes** — deferred indefinitely; adds a network
  dependency and a cost/privacy surface that a quiet offline tool shouldn't
  carry.
- **Telemetry / analytics on user behavior** — explicitly never; the privacy
  statement in Phase 9 commits to no telemetry.
- **Native mobile apps** — the PWA (Phase 6) is the mobile story; a separate
  native app is post-1.0 at the earliest, if ever.

## Future / Ecosystem (post-1.0, if they keep Tome quiet)

- **Bulk chapter import** from a pasted/uploaded table of contents (a Phase 4
  seed, expandable).
- **Flashcard import/export** (CSV or a common format) so a reader's deck is
  portable — still single-user.
- **Export a book's notes + progress** to a file (markdown/PDF) for a personal
  archive.
- **Sneakernet / file-based backup + restore** of a user's own data, for full
  offline portability between machines.
- **Additional UI languages**, once the string surface is externalized — blocked
  only on translator contributions.
