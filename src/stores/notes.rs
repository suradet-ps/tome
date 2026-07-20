//! Per-chapter notes (root-scoped singleton).

use crate::core::error::{AppError, AppResult};
use crate::core::supabase;
use crate::core::time::{now_iso, to_iso};
use crate::core::types::Note;
use crate::core::validate;
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use std::{collections::HashMap, sync::OnceLock};

static STATE: OnceLock<NotesState> = OnceLock::new();
pub fn install() {
  let _ = STATE.set(NotesState::new());
}

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
    validate::check_note_content(content)?;
    let a = use_auth();
    if a.user.get_untracked().is_none() {
      return Err(AppError::Unauthorized);
    }
    let Some(uid) = a.user.get_untracked() else {
      return Err(AppError::Unauthorized);
    };
    let ex = self.get(cid);

    // The cached note (self.map) is only updated after the server confirms the
    // write, so a failed save can never leave the cache claiming "saved" while
    // the DB holds the old text. On error we surface it instead of swallowing
    // it, so the editor can show a failed/retry state rather than a false save.
    let result = async {
      let c = supabase::supabase()?;

      // Optimistic-concurrency check: if we hold a cached note, re-read the
      // server's current row first. If it has a newer updated_at than the one
      // we loaded, another tab/device saved in the meantime — refuse rather
      // than silently clobbering their edit (last-writer-wins).
      if let Some(cached) = ex.as_ref() {
        let current: Option<Note> = c
          .postgrest()
          .from("reading_notes")
          .select("*")
          .eq("user_id", uid.to_string())
          .eq("chapter_id", cid.to_string())
          .get_one()
          .await?;
        if let Some(server) = current.as_ref()
          && is_stale(cached.updated_at, server.updated_at)
        {
          return Err(AppError::Conflict);
        }
      }

      let body = serde_json::json!({"id":ex.as_ref().map(|n|n.id),"user_id":uid,"chapter_id":cid,"content":content,"created_at":ex.as_ref().map(|n|to_iso(n.created_at)),"updated_at":now_iso()});
      let note: Note = c
        .postgrest()
        .from("reading_notes")
        .upsert_one(&body, "user_id,chapter_id")
        .await?;
      AppResult::Ok(note)
    }
    .await;

    match result {
      Ok(note) => {
        let mut cur = self.map.get_untracked();
        cur.insert(note.chapter_id, note.clone());
        self.map.set(cur);
        self.error.set(None);
        Ok(note)
      }
      Err(e) => {
        self.error.set(Some(e.to_string()));
        Err(e)
      }
    }
  }

  pub fn reset(&self) {
    self.map.set(HashMap::new());
  }
}

/// Whether the note we hold is stale: the server's `updated_at` is strictly
/// newer than the timestamp we loaded, meaning someone else saved in between.
/// Pure so the concurrency rule can be tested without a network round trip.
fn is_stale(
  loaded_at: chrono::DateTime<chrono::Utc>,
  server_at: chrono::DateTime<chrono::Utc>,
) -> bool {
  server_at > loaded_at
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::{Duration, Utc};

  #[test]
  fn same_timestamp_is_not_stale() {
    let t = Utc::now();
    assert!(!is_stale(t, t), "an unchanged row is safe to overwrite");
  }

  #[test]
  fn newer_server_timestamp_is_stale() {
    let loaded = Utc::now();
    let server = loaded + Duration::seconds(1);
    assert!(
      is_stale(loaded, server),
      "a newer server row means another writer got there first"
    );
  }

  #[test]
  fn older_server_timestamp_is_not_stale() {
    // Our load is at least as new as the server row (e.g. we just wrote it),
    // so saving again is fine.
    let server = Utc::now();
    let loaded = server + Duration::seconds(1);
    assert!(!is_stale(loaded, server));
  }
}
