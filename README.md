# Tome

<p>
  <img src="https://img.shields.io/badge/version-0.1.0-blue?style=flat-square" alt="version">
  <img src="https://img.shields.io/badge/Vue_3-4FC08D?style=flat-square&logo=vue.js&logoColor=white" alt="Vue 3">
  <img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/Supabase-3ECF8E?style=flat-square&logo=supabase&logoColor=white" alt="Supabase">
  <img src="https://img.shields.io/badge/Vite-646CFF?style=flat-square&logo=vite&logoColor=white" alt="Vite">
  <img src="https://img.shields.io/badge/Pinia-FFD859?style=flat-square&logo=pinia&logoColor=333" alt="Pinia">
  <img src="https://img.shields.io/badge/Vercel-000000?style=flat-square&logo=vercel&logoColor=white" alt="Vercel">
</p>

Tome is a dark-first technical reading tracker for developers who read technical books. Track hierarchical chapter progress, write markdown notes with code highlighting, drill concepts with spaced-repetition flashcards, and stay focused with a Pomodoro timer — all synced to Supabase.

## Features

| Feature | Description |
|---------|-------------|
| **Auth** | Supabase auth with user profile management |
| **Books & Chapters** | Nested chapter structure with custom sequence numbering (1.1, 1.2, …) |
| **Progress Tracking** | Per-chapter status: not started, in progress, completed, review needed |
| **Markdown Notes** | Rich editor with live preview, syntax-highlighted code (highlight.js), DOMPurify-sanitized |
| **Flashcards** | Spaced-repetition review (SM-2-inspired ease factor + interval scheduling) |
| **Pomodoro Timer** | Focus / short break / long break modes with auto-log on chapter switch |
| **Dashboard** | Single-RPC summary of book progress and cards due |

## Tech Stack

- **Framework:** Vue 3.5 (Composition API, `<script setup>`, `defineModel`, `defineOptions`)
- **Build Tool:** Vite 8
- **State Management:** Pinia 3
- **Routing:** Vue Router 4
- **Backend:** Supabase (Postgres + Auth, RLS-enforced)
- **Styling:** Pure CSS with design tokens (no UI framework)
- **Icons:** Lucide Vue Next
- **Markdown:** Marked 18 + highlight.js (core, 10 langs registered)
- **Sanitization:** DOMPurify
- **Linting/Format:** Biome 2
- **Deployment:** Vercel (with CSP headers + SPA fallback)

## Project Structure

```
src/
├── assets/styles/        # CSS variables, reset, base layout
├── components/
│   ├── common/           # BaseButton, BaseInput, BaseTextarea, BaseModal, BaseLoader
│   ├── progress/         # ChapterList, ProgressBar
│   ├── editor/           # MarkdownEditor
│   └── review/           # FlashcardContainer, PomodoroTimer
├── composables/          # useTimer, useMarkdown
├── lib/                  # supabase client
├── router/               # Vue Router configuration
├── stores/               # Pinia stores: auth, books, progress, notes
├── types/                # Database schema types
└── views/                # Dashboard, Book, Review, Login, Register
```

```
public/                   # Static assets copied to dist/ (favicon.svg)
vercel.json               # Vercel rewrites + security headers
supabase-schema.sql       # Idempotent full schema (DROP + CREATE)
```

## Getting Started

### Prerequisites

- [Bun](https://bun.sh) (or Node 20+ with npm/pnpm)
- A Supabase project ([supabase.com](https://supabase.com))

### Setup

```bash
# Install dependencies
bun install

# Copy environment template
cp .env.example .env
```

Edit `.env` with your Supabase credentials:

```env
VITE_SUPABASE_URL=https://your-project.supabase.co
VITE_SUPABASE_ANON_KEY=your-anon-key
```

### Database

Run the schema in Supabase SQL Editor (in order):

1. **`supabase-schema.sql`** — full idempotent setup (tables, RLS, triggers, RPC)
2. **`supabase-migration-002-dashboard-rpc.sql`** — additive: `get_dashboard_summary()` (already included in the main schema, kept separate for re-deploys)

The schema uses RLS to ensure users can only read/write their own data.

### Development

```bash
bun run dev          # Vite dev server (HMR)
```

## Scripts

| Command | Description |
|---------|-------------|
| `bun run dev` | Start Vite dev server |
| `bun run build` | Type-check (`vue-tsc`) + production build |
| `bun run preview` | Preview production build locally |
| `bun run check` | Biome lint + format check |
| `bun run check:fix` | Biome auto-fix |
| `bun run fmt` | Biome format only |
| `bun run lint` | Biome lint only |
| `bun run lint:fix` | Biome lint auto-fix |
| `bun run ci` | Biome CI check |

## Security

Tome ships with a defense-in-depth security posture:

- **CSP** via `Content-Security-Policy` header in `vercel.json` (script/style/connect origins whitelisted)
- **XSS protection** — markdown rendered through DOMPurify before injection
- **X-Frame-Options: DENY** + **frame-ancestors 'none'** — clickjacking protection
- **X-Content-Type-Options: nosniff** — MIME-sniffing protection
- **Strict Referrer-Policy** + **Permissions-Policy** (no camera/mic/geo)
- **RLS** on every Supabase table — `auth.uid() = user_id` policies
- **Length caps** — note content (200k chars), book/chapter titles (200 chars)
- **Type-safe DB** — `createClient<Database>` with generated schema types (no `as` casts)

## Accessibility

- Semantic HTML + ARIA roles (radiogroup, tablist, status, timer)
- Focus trap + Escape close in modals
- Keyboard navigation (arrow keys on status radiogroup + tablist)
- `:focus-visible` outlines on interactive elements
- `aria-label` on icon-only buttons
- `prefers-reduced-motion` support in CSS

## Deployment

Deployed on [Vercel](https://vercel.com). `vercel.json` provides:

- **SPA fallback** — `/:path*` rewrites to `/index.html` for client-side routing
- **404.html** — auto-generated by a Vite plugin from `index.html` for deep-link refresh
- **Security headers** — CSP, X-Frame-Options, Referrer-Policy, Permissions-Policy
- **Asset caching** — `/assets/*` cached for 1 year (immutable, Vite hashes filenames)
- **No cache** — `/index.html` always fresh

```bash
# Deploy
vercel deploy --prod
```

Or connect the repo to Vercel for automatic deploys on push to `main`.

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `VITE_SUPABASE_URL` | Yes | Supabase project URL |
| `VITE_SUPABASE_ANON_KEY` | Yes | Supabase anon (public) key |

Both are exposed to the client (`VITE_` prefix) and required for the app to boot. If missing, the app shows a configuration notice instead of crashing.

## License

Private — not yet licensed for public use.
