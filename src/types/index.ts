export type ReadingStatus = 'not_started' | 'in_progress' | 'completed' | 'review_needed';

export interface Profile {
  id: string;
  updated_at: string | null;
  username: string | null;
  avatar_url: string | null;
}

export interface Book {
  id: string;
  user_id: string;
  title: string;
  author: string | null;
  cover_url?: string | null;
  description?: string | null;
  total_chapters: number;
  created_at: string;
}

export interface Chapter {
  id: string;
  book_id: string;
  title: string;
  sequence_number: number;
  parent_id: string | null;
  children?: Chapter[];
}

export interface Progress {
  id: string;
  user_id: string;
  chapter_id: string;
  status: ReadingStatus;
  time_spent_seconds: number;
  updated_at: string;
}

export interface Note {
  id: string;
  user_id: string;
  chapter_id: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface Flashcard {
  id: string;
  user_id: string;
  chapter_id: string | null;
  front: string;
  back: string;
  next_review: string;
  interval_days: number;
  ease_factor: number;
  created_at: string;
}
