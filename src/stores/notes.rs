//! Per-chapter notes (root-scoped singleton).

use crate::core::db;
use crate::core::error::{AppError, AppResult};
use crate::core::sync::{self, OpKind, PendingOp};
use crate::core::time::{now_iso, to_iso};
use crate::core::types::Note;
use crate::core::validate;
use crate::stores::auth::{authed, use_auth};
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

  /// Fetch a note — try Supabase first, fall back to IndexedDB on network error.
  pub async fn fetch(&self, cid: uuid::Uuid) -> AppResult<Option<Note>> {
    let a = use_auth();
    if a.user.get_untracked().is_none() {
      return Ok(None);
    }
    let Some(uid) = a.user.get_untracked() else {
      return Ok(None);
    };

    // Try Supabase first.
    let result = authed(|c| async move {
      let note: Option<Note> = c
        .postgrest()
        .from("reading_notes")
        .select("*")
        .eq("user_id", uid.to_string())
        .eq("chapter_id", cid.to_string())
        .get_one()
        .await?;
      Ok::<_, AppError>(note)
    })
    .await;

    match result {
      Ok(note) => {
        // Cache in IndexedDB for offline access.
        if let Some(ref n) = note {
          let _ = db::put(db::stores::NOTES, n).await;
        }
        if let Some(ref n) = note {
          let mut cur = self.map.get();
          cur.insert(cid, n.clone());
          self.map.set(cur);
        }
        Ok(note)
      }
      Err(e) if e.is_network() => {
        // Offline — try IndexedDB cache.
        let cache_key = format!("{uid}_{cid}");
        let cached: Option<Note> = db::get(db::stores::NOTES, &cache_key).await?;
        if let Some(ref n) = cached {
          let mut cur = self.map.get();
          cur.insert(cid, n.clone());
          self.map.set(cur);
        }
        Ok(cached)
      }
      Err(e) => Err(e),
    }
  }

  /// Save a note — write to IndexedDB immediately, sync to Supabase.
  /// On network failure, queue the write for later sync.
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
    let now = now_iso();

    // Build the body for both local cache and remote upsert. On the first
    // save the database generates `id` and `created_at` (they are NOT NULL);
    // on updates we send the existing values so the columns keep their
    // original identity.
    let body = note_body(uid, cid, content, &now, ex.as_ref());

    // Build the note to cache locally.
    let local_note = Note {
      id: ex.as_ref().map_or_else(uuid::Uuid::new_v4, |n| n.id),
      user_id: uid,
      chapter_id: cid,
      content: content.to_string(),
      created_at: ex.as_ref().map_or_else(chrono::Utc::now, |n| n.created_at),
      updated_at: chrono::Utc::now(),
    };

    // Write to IndexedDB immediately so the data is available offline.
    let _ = db::put(db::stores::NOTES, &local_note).await;

    // Try Supabase.
    let result = authed(|c| {
      let ex = &ex;
      let body = &body;
      async move {
        // Optimistic-concurrency check.
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

        let note: Note = c
          .postgrest()
          .from("reading_notes")
          .upsert_one(body, "user_id,chapter_id")
          .await?;
        AppResult::Ok(note)
      }
    })
    .await;

    match result {
      Ok(note) => {
        // Reconcile with the server's authoritative row.
        let mut cur = self.map.get_untracked();
        cur.insert(note.chapter_id, note.clone());
        self.map.set(cur);
        self.error.set(None);
        Ok(note)
      }
      Err(e) if e.is_network() => {
        // Offline — queue for sync, use the local note as the result.
        let op = PendingOp {
          id: uuid::Uuid::new_v4().to_string(),
          table: "reading_notes".to_string(),
          kind: OpKind::Upsert {
            on_conflict: "user_id,chapter_id".to_string(),
          },
          payload: body,
          created_at: now,
        };
        let _ = sync::enqueue(op).await;

        let mut cur = self.map.get_untracked();
        cur.insert(local_note.chapter_id, local_note.clone());
        self.map.set(cur);
        self.error.set(None);
        Ok(local_note)
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

/// Build the upsert body for a note save.
///
/// On the first save the database generates `id` and `created_at` — sending
/// an explicit `null` for them would violate the NOT NULL constraints and
/// fail the whole save, so they are omitted entirely. On updates the existing
/// values are sent so the columns keep their original identity. Pure so the
/// invariant can be tested.
fn note_body(
  uid: uuid::Uuid,
  cid: uuid::Uuid,
  content: &str,
  updated_at: &str,
  existing: Option<&Note>,
) -> serde_json::Value {
  let mut body = serde_json::json!({
    "user_id": uid,
    "chapter_id": cid,
    "content": content,
    "updated_at": updated_at,
  });
  if let Some(n) = existing {
    body["id"] = serde_json::json!(n.id);
    body["created_at"] = serde_json::json!(to_iso(n.created_at));
  }
  body
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

  #[test]
  fn first_save_omits_id_and_created_at() {
    // Regression: sending `id: null` on the first save violated the NOT NULL
    // constraint on `reading_notes.id` and failed every first save.
    let body = note_body(
      uuid::Uuid::new_v4(),
      uuid::Uuid::new_v4(),
      "hello",
      "2026-01-01T00:00:00Z",
      None,
    );
    assert!(body.get("id").is_none(), "db must generate the id");
    assert!(
      body.get("created_at").is_none(),
      "db must generate created_at"
    );
    assert_eq!(body["content"], "hello");
  }

  #[test]
  fn update_keeps_id_and_created_at() {
    let uid = uuid::Uuid::new_v4();
    let cid = uuid::Uuid::new_v4();
    let existing = Note {
      id: uuid::Uuid::new_v4(),
      user_id: uid,
      chapter_id: cid,
      content: "old".to_string(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
    };
    let body = note_body(uid, cid, "new", "2026-01-01T00:00:01Z", Some(&existing));
    assert_eq!(body["id"], serde_json::json!(existing.id));
    assert_eq!(
      body["created_at"],
      serde_json::json!(to_iso(existing.created_at))
    );
  }
}
