-- =====================================================
-- Tome - Technical Reading Tracker
-- Supabase Database Schema
-- Run this in Supabase SQL Editor
-- =====================================================

-- Enable UUID extension if needed
-- create extension if not exists "pgcrypto";

-- =====================================================
-- TYPES
-- =====================================================

create type reading_status as enum (
  'not_started',
  'in_progress',
  'completed',
  'review_needed'
);

-- =====================================================
-- TABLES
-- =====================================================

create table reading_profiles (
  id uuid references auth.users on delete cascade primary key,
  updated_at timestamp with time zone,
  username text unique,
  avatar_url text,
  constraint username_length check (char_length(username) >= 3)
);

create table reading_books (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  title text not null,
  author text,
  cover_url text,
  description text,
  total_chapters integer not null default 0,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null
);

create table reading_chapters (
  id uuid default gen_random_uuid() primary key,
  book_id uuid references reading_books(id) on delete cascade not null,
  title text not null,
  sequence_number decimal not null,
  parent_id uuid references reading_chapters(id) on delete cascade
);

create table reading_progress (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  chapter_id uuid references reading_chapters(id) on delete cascade not null,
  status reading_status default 'not_started'::reading_status not null,
  time_spent_seconds integer default 0 not null,
  updated_at timestamp with time zone default timezone('utc'::text, now()) not null,
  unique (user_id, chapter_id)
);

create table reading_notes (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  chapter_id uuid references reading_chapters(id) on delete cascade not null,
  content text not null,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null,
  updated_at timestamp with time zone default timezone('utc'::text, now()) not null,
  unique (user_id, chapter_id)
);

create table reading_flashcards (
  id uuid default gen_random_uuid() primary key,
  user_id uuid references reading_profiles(id) on delete cascade not null,
  chapter_id uuid references reading_chapters(id) on delete cascade,
  front text not null,
  back text not null,
  next_review timestamp with time zone default timezone('utc'::text, now()) not null,
  interval_days integer default 0 not null,
  ease_factor double precision default 2.5 not null,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null
);

-- =====================================================
-- INDEXES
-- =====================================================

create index reading_books_user_id_idx on reading_books(user_id);
create index reading_chapters_book_id_idx on reading_chapters(book_id);
create index reading_chapters_parent_id_idx on reading_chapters(parent_id);
create index reading_progress_user_id_idx on reading_progress(user_id);
create index reading_progress_chapter_id_idx on reading_progress(chapter_id);
create index reading_notes_user_id_idx on reading_notes(user_id);
create index reading_notes_chapter_id_idx on reading_notes(chapter_id);
create index reading_flashcards_user_id_idx on reading_flashcards(user_id);
create index reading_flashcards_chapter_id_idx on reading_flashcards(chapter_id);
create index reading_flashcards_next_review_idx on reading_flashcards(next_review);

-- =====================================================
-- RLS
-- =====================================================

alter table reading_profiles enable row level security;
alter table reading_books enable row level security;
alter table reading_chapters enable row level security;
alter table reading_progress enable row level security;
alter table reading_notes enable row level security;
alter table reading_flashcards enable row level security;

create policy "Users can view own profile"
  on reading_profiles for select
  using (auth.uid() = id);

create policy "Users can insert own profile"
  on reading_profiles for insert
  with check (auth.uid() = id);

create policy "Users can update own profile"
  on reading_profiles for update
  using (auth.uid() = id);

create policy "Users can view own books"
  on reading_books for select
  using (auth.uid() = user_id);

create policy "Users can insert own books"
  on reading_books for insert
  with check (auth.uid() = user_id);

create policy "Users can update own books"
  on reading_books for update
  using (auth.uid() = user_id);

create policy "Users can delete own books"
  on reading_books for delete
  using (auth.uid() = user_id);

create policy "Users can view chapters of own books"
  on reading_chapters for select
  using (
    exists (
      select 1
      from reading_books
      where reading_books.id = reading_chapters.book_id
        and reading_books.user_id = auth.uid()
    )
  );

create policy "Users can insert chapters into own books"
  on reading_chapters for insert
  with check (
    exists (
      select 1
      from reading_books
      where reading_books.id = reading_chapters.book_id
        and reading_books.user_id = auth.uid()
    )
  );

create policy "Users can update chapters of own books"
  on reading_chapters for update
  using (
    exists (
      select 1
      from reading_books
      where reading_books.id = reading_chapters.book_id
        and reading_books.user_id = auth.uid()
    )
  );

create policy "Users can delete chapters of own books"
  on reading_chapters for delete
  using (
    exists (
      select 1
      from reading_books
      where reading_books.id = reading_chapters.book_id
        and reading_books.user_id = auth.uid()
    )
  );

create policy "Users can view own progress"
  on reading_progress for select
  using (auth.uid() = user_id);

create policy "Users can insert own progress"
  on reading_progress for insert
  with check (auth.uid() = user_id);

create policy "Users can update own progress"
  on reading_progress for update
  using (auth.uid() = user_id);

create policy "Users can delete own progress"
  on reading_progress for delete
  using (auth.uid() = user_id);

create policy "Users can view own notes"
  on reading_notes for select
  using (auth.uid() = user_id);

create policy "Users can insert own notes"
  on reading_notes for insert
  with check (auth.uid() = user_id);

create policy "Users can update own notes"
  on reading_notes for update
  using (auth.uid() = user_id);

create policy "Users can delete own notes"
  on reading_notes for delete
  using (auth.uid() = user_id);

create policy "Users can view own flashcards"
  on reading_flashcards for select
  using (auth.uid() = user_id);

create policy "Users can insert own flashcards"
  on reading_flashcards for insert
  with check (auth.uid() = user_id);

create policy "Users can update own flashcards"
  on reading_flashcards for update
  using (auth.uid() = user_id);

create policy "Users can delete own flashcards"
  on reading_flashcards for delete
  using (auth.uid() = user_id);

-- =====================================================
-- FUNCTIONS & TRIGGERS
-- =====================================================

create or replace function public.handle_new_user()
returns trigger as $$
begin
  insert into public.reading_profiles (id, username, updated_at)
  values (
    new.id,
    nullif(new.raw_user_meta_data ->> 'username', ''),
    timezone('utc'::text, now())
  );
  return new;
end;
$$ language plpgsql security definer;

create trigger on_auth_user_created
  after insert on auth.users
  for each row execute procedure public.handle_new_user();

create or replace function update_updated_at_column()
returns trigger as $$
begin
  new.updated_at = timezone('utc'::text, now());
  return new;
end;
$$ language plpgsql;

create or replace function sync_reading_book_total_chapters()
returns trigger as $$
declare
  target_book_id uuid;
begin
  target_book_id := coalesce(new.book_id, old.book_id);

  update reading_books
  set total_chapters = (
    select count(*)
    from reading_chapters
    where reading_chapters.book_id = target_book_id
  )
  where id = target_book_id;

  return coalesce(new, old);
end;
$$ language plpgsql;

create trigger update_reading_notes_updated_at
  before update on reading_notes
  for each row execute procedure update_updated_at_column();

create trigger update_reading_progress_updated_at
  before update on reading_progress
  for each row execute procedure update_updated_at_column();

create trigger update_reading_profiles_updated_at
  before update on reading_profiles
  for each row execute procedure update_updated_at_column();

create trigger sync_reading_books_total_chapters_on_insert
  after insert on reading_chapters
  for each row execute procedure sync_reading_book_total_chapters();

create trigger sync_reading_books_total_chapters_on_delete
  after delete on reading_chapters
  for each row execute procedure sync_reading_book_total_chapters();

create trigger sync_reading_books_total_chapters_on_update
  after update of book_id on reading_chapters
  for each row execute procedure sync_reading_book_total_chapters();
