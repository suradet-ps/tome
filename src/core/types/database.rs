//! Database row types.
//!
//! These types mirror the Supabase schema. They use `snake_case` to match the
//! exact column names returned by `PostgREST` so no rename layer is needed at
//! the boundary.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Re-exported JSON value used by the Supabase schema.
pub type Json = serde_json::Value;

/// Status of a chapter for the current user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ReadingStatus {
    /// Default state when no progress entry exists.
    #[default]
    NotStarted,
    /// The user is currently reading this chapter.
    InProgress,
    /// The user has finished the chapter.
    Completed,
    /// The user wants to revisit the chapter later.
    ReviewNeeded,
}

impl ReadingStatus {
    /// String representation expected by the `reading_status` Postgres enum.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::ReviewNeeded => "review_needed",
        }
    }
}

/// Row of `reading_profiles`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Primary key — also the Supabase auth user id.
    pub id: Uuid,
    /// When the profile was last updated.
    pub updated_at: Option<DateTime<Utc>>,
    /// User-chosen display name.
    pub username: Option<String>,
    /// Optional avatar URL.
    pub avatar_url: Option<String>,
}

/// Row of `reading_books`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    /// Primary key.
    pub id: Uuid,
    /// Owning user (FK -> `reading_profiles.id`).
    pub user_id: Uuid,
    /// Book title.
    pub title: String,
    /// Author (optional).
    pub author: Option<String>,
    /// Optional cover image URL.
    pub cover_url: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Cached count of chapters for this book.
    pub total_chapters: i32,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Row of `reading_chapters` augmented with a `children` array used to
/// represent the chapter tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    /// Primary key.
    pub id: Uuid,
    /// Owning book.
    pub book_id: Uuid,
    /// Display title.
    pub title: String,
    /// Sequence number (decimal to support nested levels like `1.1`).
    pub sequence_number: f64,
    /// Optional parent chapter (for nesting).
    pub parent_id: Option<Uuid>,
    /// Lazily populated child chapters.
    #[serde(default)]
    pub children: Vec<Self>,
}

/// Row of `reading_progress`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Primary key.
    pub id: Uuid,
    /// Owning user.
    pub user_id: Uuid,
    /// Chapter being tracked.
    pub chapter_id: Uuid,
    /// Current status.
    pub status: ReadingStatus,
    /// Cumulative time spent (in seconds).
    pub time_spent_seconds: i32,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Row of `reading_notes`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Primary key.
    pub id: Uuid,
    /// Owning user.
    pub user_id: Uuid,
    /// Chapter this note belongs to.
    pub chapter_id: Uuid,
    /// Markdown content.
    pub content: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Row of `reading_flashcards`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    /// Primary key.
    pub id: Uuid,
    /// Owning user.
    pub user_id: Uuid,
    /// Chapter this card is associated with (optional).
    pub chapter_id: Option<Uuid>,
    /// Front of the card (the question).
    pub front: String,
    /// Back of the card (the answer).
    pub back: String,
    /// When the card is next due for review.
    pub next_review: DateTime<Utc>,
    /// Current spaced-repetition interval in days.
    pub interval_days: i32,
    /// SM-2 ease factor.
    pub ease_factor: f64,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Returned by the `get_dashboard_summary` RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummaryRow {
    /// Book this row summarises.
    pub book_id: Uuid,
    /// Total number of chapters for the book.
    pub total: i64,
    /// Number of chapters marked as completed.
    pub completed: i64,
    /// Number of flashcards due (always 0 — the RPC does not compute this;
    /// the client falls back to a separate count query).
    pub cards_due: i64,
}
