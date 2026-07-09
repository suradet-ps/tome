//! Per-chapter notes (context-based).

use crate::core::error::{AppError, AppResult};
use crate::core::supabase;
use crate::core::time::now_iso;
use crate::core::types::Note;
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use std::collections::HashMap;

const MAX_NOTE_LENGTH: usize = 200_000;

#[derive(Debug, Clone, Copy)]
pub struct NotesState {
    pub map: RwSignal<HashMap<uuid::Uuid, Note>>,
    pub error: RwSignal<Option<String>>,
}

impl NotesState {
    pub fn provide() -> Self {
        let s = Self { map: RwSignal::new(HashMap::new()), error: RwSignal::new(None) };
        provide_context(s);
        s
    }
    #[must_use] pub fn use_ctx() -> Self { use_context::<Self>().expect("NotesState must be provided at the root") }
    #[must_use] pub fn get(&self, id: uuid::Uuid) -> Option<Note> { self.map.get().get(&id).cloned() }

    pub async fn fetch(&self, chapter_id: uuid::Uuid) -> AppResult<Option<Note>> {
        let auth = use_auth(); if auth.user.get_untracked().is_none() { return Ok(None); }
        let Some(user_id) = auth.user.get_untracked() else { return Ok(None); };
        let client = supabase::supabase()?;
        let note: Option<Note> = client.postgrest().from("reading_notes").select("*").eq("user_id", user_id.to_string()).eq("chapter_id", chapter_id.to_string()).get_one().await?;
        if let Some(ref note) = note { let mut c = self.map.get(); c.insert(chapter_id, note.clone()); self.map.set(c); }
        Ok(note)
    }

    pub async fn save(&self, chapter_id: uuid::Uuid, content: &str) -> AppResult<Note> {
        if content.len() > MAX_NOTE_LENGTH { return Err(AppError::other(format!("Note exceeds max length of {MAX_NOTE_LENGTH}"))); }
        let auth = use_auth(); if auth.user.get_untracked().is_none() { return Err(AppError::Unauthorized); }
        let Some(user_id) = auth.user.get_untracked() else { return Err(AppError::Unauthorized); };
        let client = supabase::supabase()?;
        let existing = self.get(chapter_id);
        let body = serde_json::json!({"id":existing.as_ref().map(|n|n.id),"user_id":user_id,"chapter_id":chapter_id,"content":content,"created_at":existing.as_ref().map(|n|crate::core::time::to_iso(n.created_at)),"updated_at":now_iso()});
        let note: Note = client.postgrest().from("reading_notes").upsert_one(&body, "user_id,chapter_id").await?;
        let mut c = self.map.get(); c.insert(note.chapter_id, note.clone()); self.map.set(c);
        Ok(note)
    }

    pub fn reset(&self) { self.map.set(HashMap::new()); }
}
