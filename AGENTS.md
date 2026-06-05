# Agent Instructions: Technical Reading Tracker Application

This document outlines the architecture, database schema, state management, and implementation guidelines for building the Technical Reading Tracker web application. 

All agents (Frontend, Backend, and QA) must adhere to these specifications, using this file in conjunction with the UI/UX specifications detailed in `DESIGN.md`.

---

## 1. System Overview & Tech Stack

The application is a structured tracker designed specifically for technical books (e.g., *The Rust Book*). It focuses on hierarchical progress tracking, rich markdown note-taking with code highlighting, active recall (flashcards), and focus sessions.

### Core Stack
- **Frontend Framework:** Vue 3.5 (Composition API, TypeScript)
- **Build Tool:** Vite 8+
- **State Management:** Pinia
- **Styling:** Pure CSS (No Tailwind, no UI framework; custom utility-first or scoped CSS architecture)
- **Icons:** `@lucide/vue`
- **Backend-as-a-Service:** Supabase (Database, Auth, and Storage)

---

## 2. Supabase Database Schema (PostgreSQL)

Agents must implement the following database structure using the `reading_` prefix for all tables. Ensure Row-Level Security (RLS) is enabled for all tables, allowing users to read and write only their own data.

### `reading_profiles`
Tracks user profile details. Linked to Supabase Auth.
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
  total_chapters integer not null default 0,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null
);
```

### `reading_chapters`
The structural chapters/sub-chapters of a book.
```sql
create table reading_chapters (
  id uuid default gen_random_uuid() primary key,
  book_id uuid references reading_books(id) on delete cascade not null,
  title text not null,
  sequence_number decimal not null, -- Allows nested chapters like 1.1, 1.2
  parent_id uuid references reading_chapters(id) on delete cascade -- For deep nesting if needed
);
```

### `reading_progress`
User-specific reading status for each chapter.
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
Markdown notes taken by the user per chapter.
```sql
create table reading_notes (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  chapter_id uuid references reading_chapters(id) on delete cascade not null,
  content text not null, -- Markdown format
  created_at timestamp with time zone default timezone('utc'::text, now()) not null,
  updated_at timestamp with time zone default timezone('utc'::text, now()) not null,
  unique (user_id, chapter_id)
);
```

### `reading_flashcards`
User-created active recall cards linked to chapters.
```sql
create table reading_flashcards (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  chapter_id uuid references reading_chapters(id) on delete cascade not null,
  front text not null, -- Question / Concept
  back text not null, -- Answer / Explanation
  next_review timestamp with time zone default timezone('utc'::text, now()) not null,
  interval_days integer default 0 not null,
  ease_factor double precision default 2.5 not null,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null
);
```

---

## 3. Pinia State Management Structure

Organize stores into logical, single-responsibility modules, mapped directly to the `reading_*` tables:

### `auth.ts`
- **State:** `user`, `profile`, `session`, `loading`
- **Actions:** `signIn()`, `signUp()`, `signOut()`, `fetchProfile()` (fetches from `reading_profiles`)

### `books.ts`
- **State:** `books`, `currentBook`, `chapters`, `loading`
- **Actions:** `fetchBooks()` (from `reading_books`), `addBook()`, `fetchChapters(bookId)` (from `reading_chapters`)

### `progress.ts`
- **State:** `progressMap` (Key-value of `chapterId` -> `reading_progress`)
- **Actions:** `updateStatus(chapterId, status)`, `logTimeSpent(chapterId, seconds)` (updates `reading_progress`)

### `notes.ts`
- **State:** `notesMap` (Key-value of `chapterId` -> `reading_notes`)
- **Actions:** `fetchNote(chapterId)`, `saveNote(chapterId, content)` (updates `reading_notes`)

---

## 4. Frontend Component & Directory Architecture

Implement clean, modular code following Vue 3 best practices and TypeScript strict typing.

```text
src/
├── assets/
│   └── styles/
│       ├── variables.css      # Colors, typography, spacing from DESIGN.md
│       ├── reset.css          # CSS Reset / Normalization
│       ├── main.css           # Global layout & utility classes
│       └── components/        # Component-specific styles if not scoped
├── components/
│   ├── common/                # Buttons, Inputs, Loaders, Modals
│   ├── layout/                # Sidebar, Navigation, Header
│   ├── progress/              # ChapterList, ProgressBar
│   ├── editor/                # MarkdownEditor, CodeViewer
│   └── review/                # FlashcardContainer, Timer
├── composables/               # Reusable logic (e.g., useTimer, useMarkdown)
├── router/                    # Vue Router configurations
├── stores/                    # Pinia Stores
├── views/                     # Page Views (Dashboard, BookView, ReviewPage)
└── App.vue
```

---

## 5. Pure CSS & Styling Guidelines

Since no prebuilt component frameworks or CSS-in-JS tools are used, strict styling guidelines must be enforced:

1. **Design Token Alignment:** Read `DESIGN.md` for designated custom properties. Define these in `assets/styles/variables.css` under the `:root` pseudo-class (e.g., `--color-primary`, `--font-sans`, `--spacing-md`).
2. **Scoping:** Prefer Vue's `<style scoped>` in components to avoid style leakage, or follow a strict BEM (Block Element Modifier) convention if utilizing global style files.
3. **Flexibility:** Use modern CSS layout models (Flexbox and CSS Grid) for alignment and responsive layouts. Avoid hardcoded pixel dimensions for layout widths.
4. **Responsive Breakpoints:** Define and respect the mobile-first or desktop-first breakpoints defined in `DESIGN.md`.

---

## 6. Development Workflow & Agent Instructions

### Frontend Agent
- Parse the UI/UX specifications in `DESIGN.md` before generating component templates.
- Implement strict TypeScript typing for all props, events, and API payloads.
- Integrate Lucide icons using `<@lucide/vue>` appropriately.
- Avoid inline styles. Ensure semantic HTML structure for accessibility (a11y).

### Backend/Supabase Agent
- Draft database migrations based on Section 2 using the proper `reading_` prefixes.
- Configure proper Postgres policies (RLS) to secure user data.
- Optimize indexing on foreign keys (`user_id`, `chapter_id`, `book_id`) for performance.

### Integration Steps
1. Initialize Vite 8+ project with TypeScript and ESLint.
2. Setup Supabase Client and Pinia stores.
3. Apply CSS variables and base layouts from `DESIGN.md`.
4. Build the Auth flow (Login, Register).
5. Develop the Book List and Chapter Tracking views.
6. Implement the Markdown Note-taking area with a code highlight preview.
7. Build the Pomodoro timer and active recall system.
