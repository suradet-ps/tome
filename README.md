# Tome

![Rust](https://img.shields.io/badge/Rust-2024-000000?style=flat-square&logo=rust&logoColor=white)
![Leptos](https://img.shields.io/badge/Leptos-0.8-543e7c?style=flat-square&logo=leptos&logoColor=white)
![Supabase](https://img.shields.io/badge/Supabase-3ECF8E?style=flat-square&logo=supabase&logoColor=white)
![Vercel](https://img.shields.io/badge/Vercel-black?style=flat-square&logo=vercel&logoColor=white)

Tome is a dark-first technical reading tracker for developers who read technical books. Track hierarchical chapter progress, write markdown notes with code highlighting, drill concepts with spaced-repetition flashcards, and stay focused with a Pomodoro timer — all synced to Supabase.

## Features

| Feature | Description |
|---------|-------------|
| **Auth** | Supabase GoTrue (email/password) with profile management |
| **Books & Chapters** | Nested chapter structure with custom sequence numbering (1.1, 1.2, …) |
| **Progress Tracking** | Per-chapter status: not started, in progress, completed, review needed |
| **Markdown Notes** | Rich editor with live preview, keyword-based code highlighting, ammonia-sanitized |
| **Flashcards** | SM-2-inspired spaced repetition (ease factor + interval scheduling) |
| **Pomodoro Timer** | Focus / short break / long break modes with auto-log on chapter switch |
| **Dashboard** | RPC-powered summary of book progress and cards due |

## Tech Stack

- **Framework:** Leptos 0.8 (CSR, stable Rust)
- **Build:** Trunk
- **Backend:** Supabase (Postgres + Auth, RLS-enforced)
- **HTTP:** gloo-net (browser `fetch` API)
- **Styling:** Pure CSS with design tokens
- **Icons:** Inline Lucide SVG components
- **Markdown:** pulldown-cmark + ammonia (sanitization)
- **Deployment:** Vercel (with CSP headers + SPA fallback)

## Project Structure

```
src/
├── app.rs               # Root App component + Router
├── lib.rs               # Crate root + WASM entry point
├── components/
│   ├── common/          # BaseButton, BaseInput, BaseTextarea, BaseModal, BaseLoader
│   ├── editor/          # MarkdownEditor
│   ├── icons.rs         # Inline Lucide SVG icon components
│   ├── layout/          # AppTopbar (responsive nav)
│   ├── progress/        # ChapterList, ProgressBar
│   └── review/          # FlashcardContainer, PomodoroTimer
├── composables/         # use_timer, use_markdown
├── core/                # Supabase client, PostgREST, auth, markdown, highlight, error
├── stores/              # Reactive contexts: auth, books, progress, notes
└── views/               # Dashboard, Book, Review, Login, Register, Not Found
```

```
public/                  # Static assets (favicon.svg, styles/)
vercel.json              # Vercel rewrites + security headers (CSP, HSTS, etc.)
supabase-schema.sql      # Idempotent full schema (DROP + CREATE + RLS + triggers + RPC)
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs) (stable, edition 2024)
- `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- [Trunk](https://trunkrs.dev) (`cargo install trunk`)
- A [Supabase](https://supabase.com) project

### Setup

```bash
# Clone the repo
git clone https://github.com/vate-ps/tome
cd tome

# Copy environment template
cp .env.example .env
```

Edit `.env` with your Supabase credentials:

```env
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
```

### Database

Run `supabase-schema.sql` in the Supabase SQL Editor. The script is **idempotent** — safe to re-run. It creates:

- All `reading_*` tables with RLS policies
- Indexes on foreign keys
- `handle_new_user()` trigger (auto-creates profiles on signup)
- `update_updated_at_column()` trigger (auto-timestamping)
- `sync_reading_book_total_chapters()` trigger (cached chapter counts)
- `get_dashboard_summary()` RPC

### Development

```bash
trunk serve --port 3000 --open
```

### Build (production)

```bash
trunk build --release
```

Output is in `dist/` — deploy the folder to any static host.

## Scripts

| Command | Description |
|---------|-------------|
| `cargo check` | Type-check (no code gen) |
| `cargo clippy` | Lint |
| `cargo fmt --check` | Format check |
| `cargo test` | Run unit tests |
| `trunk serve` | Dev server with HMR |
| `trunk build --release` | Production build |

## Security

- **CSP** via `Content-Security-Policy` header in `vercel.json`
- **XSS protection** — markdown rendered through ammonia before injection
- **X-Frame-Options: DENY** — clickjacking protection
- **X-Content-Type-Options: nosniff** — MIME-sniffing protection
- **RLS** on every Supabase table — `auth.uid() = user_id` policies
- **Length caps** — note content (200k chars), book/chapter titles (200 chars)
- **No unsafe code** — `#![deny(unsafe_code)]` at crate level

## Accessibility

- Semantic HTML + ARIA roles
- Focus trap + Escape close in modals
- Keyboard navigation (arrow keys on status radiogroup)
- `:focus-visible` outlines
- `aria-label` on icon-only buttons
- `prefers-reduced-motion` support

## Deployment (Vercel)

1. Connect the repository to Vercel
2. Set build command: `trunk build --release`
3. Set output directory: `dist`
4. Add environment variables in Vercel dashboard:
   - `SUPABASE_URL`
   - `SUPABASE_ANON_KEY`

## License

MIT
