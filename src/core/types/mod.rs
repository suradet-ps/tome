//! Database row types mirrored from the `reading_*` tables in Supabase.

pub mod database;

pub use database::{
  Book, Chapter, DashboardSummaryRow, Flashcard, Json, Note, Profile, Progress, ReadingStatus,
};
