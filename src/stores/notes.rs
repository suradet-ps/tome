//! Per-chapter notes (root-scoped singleton).

use crate::core::error::{AppError, AppResult};
use crate::core::supabase;
use crate::core::time::{now_iso, to_iso};
use crate::core::types::Note;
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use std::{collections::HashMap, sync::OnceLock};

static STATE: OnceLock<NotesState> = OnceLock::new();
pub fn install() {
    let _ = STATE.set(NotesState::new());
}

const MAX_NOTE_LENGTH: usize = 200_000;

#[derive(Debug, Clone, Copy)]
pub struct NotesState {
    pub map: RwSignal<HashMap<uuid::Uuid, Note>>,
    pub error: RwSignal<Option<String>>,
}

impl Default for NotesState {
    fn default() -> Self {
        Self::new()
    }
}

impl NotesState {
    pub fn new() -> Self {
        Self {
            map: RwSignal::new(HashMap::new()),
            error: RwSignal::new(None),
        }
    }
    pub fn use_ctx() -> Self {
        *STATE.get().expect("NotesState not initialized")
    }
    pub fn get(&self, id: uuid::Uuid) -> Option<Note> {
        self.map.get().get(&id).cloned()
    }

    pub async fn fetch(&self, cid: uuid::Uuid) -> AppResult<Option<Note>> {
        let a = use_auth();
        if a.user.get_untracked().is_none() {
            return Ok(None);
        }
        let Some(uid) = a.user.get_untracked() else {
            return Ok(None);
        };
        let c = supabase::supabase()?;
        let note: Option<Note> = c
            .postgrest()
            .from("reading_notes")
            .select("*")
            .eq("user_id", uid.to_string())
            .eq("chapter_id", cid.to_string())
            .get_one()
            .await?;
        if let Some(ref n) = note {
            let mut cur = self.map.get();
            cur.insert(cid, n.clone());
            self.map.set(cur);
        }
        Ok(note)
    }

    pub async fn save(&self, cid: uuid::Uuid, content: &str) -> AppResult<Note> {
        if content.len() > MAX_NOTE_LENGTH {
            return Err(AppError::other(format!(
                "Note exceeds max length of {MAX_NOTE_LENGTH}"
            )));
        }
        let a = use_auth();
        if a.user.get_untracked().is_none() {
            return Err(AppError::Unauthorized);
        }
        let Some(uid) = a.user.get_untracked() else {
            return Err(AppError::Unauthorized);
        };
        let c = supabase::supabase()?;
        let ex = self.get(cid);
        let body = serde_json::json!({"id":ex.as_ref().map(|n|n.id),"user_id":uid,"chapter_id":cid,"content":content,"created_at":ex.as_ref().map(|n|to_iso(n.created_at)),"updated_at":now_iso()});
        let note: Note = c
            .postgrest()
            .from("reading_notes")
            .upsert_one(&body, "user_id,chapter_id")
            .await?;
        let mut cur = self.map.get();
        cur.insert(note.chapter_id, note.clone());
        self.map.set(cur);
        Ok(note)
    }

    pub fn reset(&self) {
        self.map.set(HashMap::new());
    }
}
