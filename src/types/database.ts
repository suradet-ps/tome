export type Json = string | number | boolean | null | { [key: string]: Json | undefined } | Json[];

export type ReadingStatus = 'not_started' | 'in_progress' | 'completed' | 'review_needed';

export interface Database {
  public: {
    Tables: {
      reading_profiles: {
        Row: {
          id: string;
          updated_at: string | null;
          username: string | null;
          avatar_url: string | null;
        };
        Insert: {
          id: string;
          updated_at?: string | null;
          username?: string | null;
          avatar_url?: string | null;
        };
        Update: {
          id?: string;
          updated_at?: string | null;
          username?: string | null;
          avatar_url?: string | null;
        };
        Relationships: [];
      };
      reading_books: {
        Row: {
          id: string;
          user_id: string;
          title: string;
          author: string | null;
          cover_url: string | null;
          description: string | null;
          total_chapters: number;
          created_at: string;
        };
        Insert: {
          id?: string;
          user_id: string;
          title: string;
          author?: string | null;
          cover_url?: string | null;
          description?: string | null;
          total_chapters?: number;
          created_at?: string;
        };
        Update: {
          id?: string;
          user_id?: string;
          title?: string;
          author?: string | null;
          cover_url?: string | null;
          description?: string | null;
          total_chapters?: number;
          created_at?: string;
        };
        Relationships: [];
      };
      reading_chapters: {
        Row: {
          id: string;
          book_id: string;
          title: string;
          sequence_number: number;
          parent_id: string | null;
        };
        Insert: {
          id?: string;
          book_id: string;
          title: string;
          sequence_number: number;
          parent_id?: string | null;
        };
        Update: {
          id?: string;
          book_id?: string;
          title?: string;
          sequence_number?: number;
          parent_id?: string | null;
        };
        Relationships: [
          {
            foreignKeyName: 'reading_chapters_book_id_fkey';
            columns: ['book_id'];
            referencedRelation: 'reading_books';
            referencedColumns: ['id'];
          },
          {
            foreignKeyName: 'reading_chapters_parent_id_fkey';
            columns: ['parent_id'];
            referencedRelation: 'reading_chapters';
            referencedColumns: ['id'];
          },
        ];
      };
      reading_progress: {
        Row: {
          id: string;
          user_id: string;
          chapter_id: string;
          status: ReadingStatus;
          time_spent_seconds: number;
          updated_at: string;
        };
        Insert: {
          id?: string;
          user_id: string;
          chapter_id: string;
          status?: ReadingStatus;
          time_spent_seconds?: number;
          updated_at?: string;
        };
        Update: {
          id?: string;
          user_id?: string;
          chapter_id?: string;
          status?: ReadingStatus;
          time_spent_seconds?: number;
          updated_at?: string;
        };
        Relationships: [
          {
            foreignKeyName: 'reading_progress_chapter_id_fkey';
            columns: ['chapter_id'];
            referencedRelation: 'reading_chapters';
            referencedColumns: ['id'];
          },
          {
            foreignKeyName: 'reading_progress_user_id_fkey';
            columns: ['user_id'];
            referencedRelation: 'reading_profiles';
            referencedColumns: ['id'];
          },
        ];
      };
      reading_notes: {
        Row: {
          id: string;
          user_id: string;
          chapter_id: string;
          content: string;
          created_at: string;
          updated_at: string;
        };
        Insert: {
          id?: string;
          user_id: string;
          chapter_id: string;
          content: string;
          created_at?: string;
          updated_at?: string;
        };
        Update: {
          id?: string;
          user_id?: string;
          chapter_id?: string;
          content?: string;
          created_at?: string;
          updated_at?: string;
        };
        Relationships: [];
      };
      reading_flashcards: {
        Row: {
          id: string;
          user_id: string;
          chapter_id: string | null;
          front: string;
          back: string;
          next_review: string;
          interval_days: number;
          ease_factor: number;
          created_at: string;
        };
        Insert: {
          id?: string;
          user_id: string;
          chapter_id?: string | null;
          front: string;
          back: string;
          next_review?: string;
          interval_days?: number;
          ease_factor?: number;
          created_at?: string;
        };
        Update: {
          id?: string;
          user_id?: string;
          chapter_id?: string | null;
          front?: string;
          back?: string;
          next_review?: string;
          interval_days?: number;
          ease_factor?: number;
          created_at?: string;
        };
        Relationships: [];
      };
    };
    Views: Record<string, never>;
    Functions: {
      get_dashboard_summary: {
        Args: Record<string, never>;
        Returns: {
          book_id: string;
          total: number;
          completed: number;
          cards_due: number;
        }[];
      };
    };
    Enums: {
      reading_status: ReadingStatus;
    };
    CompositeTypes: Record<string, never>;
  };
}

export type Profile = Database['public']['Tables']['reading_profiles']['Row'];
export type Book = Database['public']['Tables']['reading_books']['Row'];
export type Chapter = Database['public']['Tables']['reading_chapters']['Row'] & {
  children: Chapter[];
};
export type Progress = Database['public']['Tables']['reading_progress']['Row'];
export type Note = Database['public']['Tables']['reading_notes']['Row'];
export type Flashcard = Database['public']['Tables']['reading_flashcards']['Row'];
