//! Per-chapter note state.

use crate::core::error::{AppError, AppResult};
use crate::core::supabase;
use crate::core::time::now_iso;
use crate::core::types::Note;
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use std::collections::HashMap;

/// Maximum allowed note length.
const MAX_NOTE_LENGTH: usize = 200_000;

/// Reactive container for chapter notes.
#[derive(Debug, Clone, Copy)]
pub struct NotesState {
    /// Map of chapter id -> note.
    pub map: RwSignal<HashMap<uuid::Uuid, Note>>,
    /// Last error message.
    pub error: RwSignal<Option<String>>,
}

impl NotesState {
    /// Install the state into the current reactive scope.
    #[must_use]
    pub fn provide() -> Self {
        let state = Self {
            map: RwSignal::new(HashMap::new()),
            error: RwSignal::new(None),
        };
        provide_context(state);
        state
    }

    /// Read the state from context.
    #[must_use]
    pub fn use_ctx() -> Self {
        use_context::<Self>().expect("NotesState must be provided at the root")
    }

    /// Returns the note for a chapter, if loaded.
    #[must_use]
    pub fn get(&self, chapter_id: uuid::Uuid) -> Option<Note> {
        self.map.get().get(&chapter_id).cloned()
    }

    /// Fetch the note for a chapter.
    pub async fn fetch(&self, chapter_id: uuid::Uuid) -> AppResult<Option<Note>> {
        let auth = use_auth();
        if auth.user.get().is_none() {
            return Ok(None);
        }
        let user_id = match auth.user.get() {
            Some(id) => id,
            None => return Ok(None),
        };
        let client = supabase::supabase()?;
        let note: Option<Note> = client
            .postgrest()
            .from("reading_notes")
            .select("*")
            .eq("user_id", user_id.to_string())
            .eq("chapter_id", chapter_id.to_string())
            .get_one()
            .await?;
        if let Some(ref note) = note {
            let mut current = self.map.get();
            current.insert(chapter_id, note.clone());
            self.map.set(current);
        }
        Ok(note)
    }

    /// Upsert a note for a chapter.
    pub async fn save(&self, chapter_id: uuid::Uuid, content: &str) -> AppResult<Note> {
        if content.len() > MAX_NOTE_LENGTH {
            return Err(AppError::other(format!(
                "Note exceeds maximum length of {MAX_NOTE_LENGTH} characters."
            )));
        }
        let auth = use_auth();
        if auth.user.get().is_none() {
            return Err(AppError::Unauthorized);
        }
        let user_id = match auth.user.get() {
            Some(id) => id,
            None => return Err(AppError::Unauthorized),
        };
        let client = supabase::supabase()?;
        let existing = self.get(chapter_id);
        let body = serde_json::json!({
            "id": existing.as_ref().map(|n| n.id),
            "user_id": user_id,
            "chapter_id": chapter_id,
            "content": content,
            "created_at": existing.as_ref().map(|n| n.created_at),
            "updated_at": now_iso(),
        });
        let note: Note = client
            .postgrest()
            .from("reading_notes")
            .upsert_one(&body, "user_id,chapter_id")
            .await?;
        let mut current = self.map.get();
        current.insert(note.chapter_id, note.clone());
        self.map.set(current);
        Ok(note)
    }

    /// Reset the state.
    pub fn reset(&self) {
        self.map.set(HashMap::new());
    }
}
