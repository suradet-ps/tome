# Tome

```
████████╗ ██████╗ ███╗   ███╗███████╗
╚══██╔══╝██╔═══██╗████╗ ████║██╔════╝
   ██║   ██║   ██║██╔████╔██║█████╗
   ██║   ██║   ██║██║╚██╔╝██║██╔══╝
   ██║   ╚██████╔╝██║ ╚═╝ ██║███████╗
   ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝
```

---

## ◆ PULSE

A technical book is read in stolen hours, and its knowledge dies in
the forgetting. Tome is the dark-first reading tracker for developers
who read to learn: chapter progress in a hierarchy that mirrors the
book, markdown notes with code highlighting, SM-2 flashcards that
bring the concepts back when the curve says they are fading, and a
Pomodoro timer that guards the stolen hour. Everything synced to
Supabase, everything readable offline, everything written in Rust
compiled to WASM.

| Progress ▣ | Notes ▣ | SRS ▣ | Offline ▣ |
|---|---|---|---|

*P1-P8 are sealed - identity, trust, correctness, the reading loop,
accessibility, offline-first, budgets, security. The v1.0.0 gate alone
stands open.*

> Built with Rust 2024 + Leptos 0.8, synced by Supabase, cached in
> IndexedDB, sanitized by ammonia - a reading room, not a dashboard.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One target, one tool, one command.

```
⟫ rustup target add wasm32-unknown-unknown
⟫ cargo install trunk
⟫ cp .env.example .env
⟫ trunk serve --port 3000 --open
```

<details>
<summary>Environment + database</summary>

`.env` takes the Supabase credentials:

```
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
```

Run `db/supabase-schema.sql` in the SQL Editor - it is idempotent and
creates the `reading_*` tables with RLS, the user/updated-at/chain
count triggers, and the `get_dashboard_summary()` RPC.

</details>

The release artifact: `⟫ trunk build --release` - output in `dist/`,
deployable to any static host (Vercel rewrites + CSP headers included).

---

## ◆ ANATOMY

One loop, several quiet disciplines, a memory that works offline.

- **Structures** - books hold chapters in a nested tree with custom
  sequence numbering (1.1, 1.2, ...); a pasted table of contents
  becomes the whole tree at once.
- **Notes** - markdown editing with live preview and keyword-based
  code highlighting, rendered through pulldown-cmark and sanitized by
  ammonia before a single tag reaches the DOM.
- **Reviews** - SM-2-inspired spaced repetition (ease factor and
  interval scheduling) decides when each card returns; the dashboard
  RPC reports progress and cards due in one call.
- **Focuses** - the Pomodoro timer runs focus, short-break, and
  long-break modes and auto-logs when the chapter changes - the
  stolen hour is counted, then spent.
- **Syncs** - IndexedDB keeps the books, notes, and cards offline;
  writes queue and sync when the network returns, with a calm banner
  that distinguishes offline from syncing.
- **Guards** - RLS on every table, length caps on every field, no
  `unsafe` code at the crate level, and an XSS boundary that is
  unit-tested.

---

## ◆ RITUALS

**The core ceremony** - the reading hour:

1. Open Tome. Continue Reading resumes the most recently active,
   unfinished chapter in one tap.
2. Read in the theme the hour wants - dark, light, or sepia - at the
   width and size the eyes prefer.
3. Mark the chapter's status; the progress bar and the dashboard
   move together.
4. When the chapter ends, the timer logs and the flashcards that are
   due wait at the top of the review tab - calm, counted, quiet.

**The ceremony of the sanitized page** - notes are rendered after
ammonia, not before: the markdown is trusted exactly as far as the
sanitizer allows, and the sanitizer is the first thing tested.

**The ceremony of the offline hour** - the train, the ward, the dead
wifi: reading, writing, and review all work offline, and the writes
sync themselves when the signal returns - with a banner that says
which is happening.

---

## ◆ ECHOES

**Where this artifact is heading**

```
P1-P2 ▸ Tome's identity, XSS + SM-2 trust ───────────────────────────── ▸ sealed
P3-P4 ▸ correctness, the reading loop ──────────────────────────────── ▸ sealed
P5-P8 ▸ a11y, offline-first, budgets, security ──────────────────────── ▸ sealed
P9    ▸ v1.0.0: reproducible build, branch protection, tag ──────────── ▸ open
```

**Raising the artifact** - the architecture lives in `docs/AGENTS.md`;
the design in `docs/DESIGN.md`; the baseline in
`docs/perf-baseline.md`; the version story in `docs/CHANGELOG.md`.
Gates: `cargo fmt --all --check`, clippy with correctness and
suspicious denied, `cargo test --lib`. Open an issue first to discuss
a change.

**Status** - CI gates every push with lint, tests, the Trunk build,
and the budget checks. [Watch the gates](.github/workflows).

---

```
  ─────────────────────────────────────────
   A book read and forgotten is a book
   that was never really read.
  ─────────────────────────────────────────
```

Released under the [MIT License](LICENSE).