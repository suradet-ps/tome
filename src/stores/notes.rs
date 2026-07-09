//! Per-chapter note state (module-level singleton).

use crate::core::error::{AppError, AppResult};
use crate::core::supabase;
use crate::core::time::now_iso;
use crate::core::types::Note;
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use std::{cell::LazyCell, collections::HashMap};

thread_local! {
    static STATE: LazyCell<NotesState> = LazyCell::new(NotesState::default);
}

const MAX_NOTE_LENGTH: usize = 200_000;

#[derive(Debug, Clone, Copy)]
pub struct NotesState {
    pub map: RwSignal<HashMap<uuid::Uuid, Note>>,
    pub error: RwSignal<Option<String>>,
}

impl Default for NotesState {
    fn default() -> Self {
        Self { map: RwSignal::new(HashMap::new()), error: RwSignal::new(None) }
    }
}

impl NotesState {
    #[must_use]
    pub fn use_ctx() -> Self {
        STATE.with(|cell| **cell)
    }

    #[must_use]
    pub fn get(&self, chapter_id: uuid::Uuid) -> Option<Note> {
        self.map.get().get(&chapter_id).cloned()
    }

    pub async fn fetch(&self, chapter_id: uuid::Uuid) -> AppResult<Option<Note>> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() { return Ok(None); }
        let user_id = match auth.user.get_untracked() { Some(id) => id, None => return Ok(None) };
        let client = supabase::supabase()?;
        let note: Option<Note> = client
            .postgrest().from("reading_notes").select("*")
            .eq("user_id", user_id.to_string())
            .eq("chapter_id", chapter_id.to_string())
            .get_one().await?;
        if let Some(ref note) = note {
            let mut current = self.map.get();
            current.insert(chapter_id, note.clone());
            self.map.set(current);
        }
        Ok(note)
    }

    pub async fn save(&self, chapter_id: uuid::Uuid, content: &str) -> AppResult<Note> {
        if content.len() > MAX_NOTE_LENGTH {
            return Err(AppError::other(format!("Note exceeds maximum length of {MAX_NOTE_LENGTH} characters.")));
        }
        let auth = use_auth();
        if auth.user.get_untracked().is_none() { return Err(AppError::Unauthorized); }
        let user_id = match auth.user.get_untracked() { Some(id) => id, None => return Err(AppError::Unauthorized) };
        let client = supabase::supabase()?;
        let existing = self.get(chapter_id);
        let body = serde_json::json!({
            "id": existing.as_ref().map(|n| n.id),
            "user_id": user_id, "chapter_id": chapter_id,
            "content": content,
            "created_at": existing.as_ref().map(|n| n.created_at),
            "updated_at": now_iso(),
        });
        let note: Note = client.postgrest().from("reading_notes").upsert_one(&body, "user_id,chapter_id").await?;
        let mut current = self.map.get();
        current.insert(note.chapter_id, note.clone());
        self.map.set(current);
        Ok(note)
    }

    pub fn reset(&self) { self.map.set(HashMap::new()); }
}
