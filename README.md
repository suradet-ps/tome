# Tome

<p>
  <img src="https://img.shields.io/badge/version-0.1.0-blue?style=flat-square" alt="version">
  <img src="https://img.shields.io/badge/Vue_3-4FC08D?style=flat-square&logo=vue.js&logoColor=white" alt="Vue 3">
  <img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/Supabase-3ECF8E?style=flat-square&logo=supabase&logoColor=white" alt="Supabase">
  <img src="https://img.shields.io/badge/Vite-646CFF?style=flat-square&logo=vite&logoColor=white" alt="Vite">
  <img src="https://img.shields.io/badge/Pinia-FFD859?style=flat-square&logo=pinia&logoColor=333" alt="Pinia">
</p>

Tome is a dark-first technical reading tracker designed for developers who read technical books. It helps you track chapter progress, write markdown notes with code highlighting, review concepts with flashcards, and stay focused with a Pomodoro timer — all synced to Supabase.

## Features

| Feature | Description |
|---------|-------------|
| **Auth** | Supabase authentication with user profile management |
| **Books & Chapters** | Nested chapter structure with custom sequence numbering |
| **Progress Tracking** | Per-chapter status: not started, in progress, completed, review needed |
| **Markdown Notes** | Rich text editor with live preview and syntax-highlighted code blocks via Highlight.js |
| **Flashcards** | Spaced-repetition review system with configurable ease factor and intervals |
| **Pomodoro Timer** | Built-in focus session timer |

## Tech Stack

- **Framework:** Vue 3.5 (Composition API, TypeScript)
- **Build Tool:** Vite 8
- **State Management:** Pinia
- **Routing:** Vue Router 4
- **Backend:** Supabase (Auth, Database, Storage)
- **Styling:** Pure CSS with custom design tokens
- **Icons:** Lucide Vue Next
- **Markdown:** Marked + Highlight.js

## Project Structure

```
src/
├── assets/styles/     # CSS variables, reset, base layout
├── components/        # UI components (common, layout, progress, editor, review)
├── composables/       # Reusable logic (useTimer, useMarkdown, etc.)
├── router/            # Vue Router configuration
├── stores/            # Pinia stores (auth, books, progress, notes)
└── views/             # Page views (Dashboard, BookView, ReviewPage)
```

## Getting Started

```bash
# Install dependencies
bun install

# Copy environment variables
cp .env.example .env

# Start development server
bun run dev
```

## Build

```bash
bun run build
```

## Database

The Supabase schema is defined in [`supabase-schema.sql`](./supabase-schema.sql) with RLS policies for user data isolation.
