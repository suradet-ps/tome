//! Per-chapter progress (root-scoped singleton).

use crate::core::db;
use crate::core::error::AppResult;
use crate::core::sync::{self, OpKind, PendingOp};
use crate::core::time::now_iso;
use crate::core::types::{Progress, ReadingStatus};
use crate::stores::auth::{authed, use_auth};
use leptos::prelude::*;
use std::{collections::HashMap, sync::OnceLock};

static STATE: OnceLock<ProgressState> = OnceLock::new();
pub fn install() {
  let _ = STATE.set(ProgressState::new());
}

#[derive(Debug, Clone, Copy)]
pub struct ProgressState {
  pub map: RwSignal<HashMap<uuid::Uuid, Progress>>,
  pub error: RwSignal<Option<String>>,
}

impl Default for ProgressState {
  fn default() -> Self {
    Self::new()
  }
}

impl ProgressState {
  pub fn new() -> Self {
    Self {
      map: RwSignal::new(HashMap::new()),
      error: RwSignal::new(None),
    }
  }
  pub fn use_ctx() -> Self {
    *STATE.get().expect("ProgressState not initialized")
  }
  pub fn get(&self, id: uuid::Uuid) -> Option<Progress> {
    self.map.get().get(&id).cloned()
  }

  /// Fetch progress for a book — try Supabase, fall back to IndexedDB.
  pub async fn fetch_for_book(&self, bid: uuid::Uuid) -> AppResult<()> {
    let a = use_auth();
    if a.user.get_untracked().is_none() {
      return Ok(());
    }
    let Some(uid) = a.user.get_untracked() else {
      return Ok(());
    };

    // Try Supabase first.
    let result = authed(|c| async move {
      let rows: Vec<ProgressWithBook> = c.postgrest().from("reading_progress").select("id,user_id,chapter_id,status,time_spent_seconds,updated_at,reading_chapters!inner(book_id)").eq("user_id", uid.to_string()).eq("reading_chapters.book_id", bid.to_string()).range(0,4999).get().await?;
      AppResult::Ok(rows)
    })
    .await;

    match result {
      Ok(rows) => {
        let mut next = self.map.get();
        let mut cached = Vec::new();
        for row in &rows {
          let p = row.clone().into_progress();
          cached.push(p.clone());
          next.insert(p.chapter_id, p);
        }
        self.map.set(next);
        // Cache in IndexedDB for offline access.
        let _ = db::put_many(db::stores::PROGRESS, &cached).await;
        Ok(())
      }
      Err(e) if e.is_network() => {
        // Offline — load from IndexedDB cache.
        let all: Vec<Progress> = db::get_all(db::stores::PROGRESS).await?;
        let mut next = self.map.get();
        for p in &all {
          next.insert(p.chapter_id, p.clone());
        }
        self.map.set(next);
        Ok(())
      }
      Err(e) => {
        self.error.set(Some(e.to_string()));
        Err(e)
      }
    }
  }

  /// Update a chapter's status — optimistic update, queue on offline.
  pub async fn update_status(
    &self,
    cid: uuid::Uuid,
    status: ReadingStatus,
  ) -> AppResult<Option<Progress>> {
    let a = use_auth();
    if a.user.get_untracked().is_none() {
      return Ok(None);
    }
    let Some(uid) = a.user.get_untracked() else {
      return Ok(None);
    };
    let ex = self.get(cid);

    // Optimistic update.
    let snapshot = self.map.get_untracked();
    let optimistic = optimistic_status(ex.as_ref(), uid, cid, status);
    self.set_entry(cid, optimistic.clone());

    let now = now_iso();
    let body = serde_json::json!({
      "user_id": uid,
      "chapter_id": cid,
      "status": status.as_str(),
      "time_spent_seconds": ex.as_ref().map_or(0, |p| p.time_spent_seconds),
      "updated_at": &now,
    });

    let result = authed(|c| {
      let body = &body;
      async move {
        let p: Progress = c
          .postgrest()
          .from("reading_progress")
          .upsert_one(body, "user_id,chapter_id")
          .await?;
        AppResult::Ok(p)
      }
    })
    .await;

    match result {
      Ok(p) => {
        // Reconcile with server's authoritative row.
        self.set_entry(cid, p.clone());
        let _ = db::put(db::stores::PROGRESS, &p).await;
        self.error.set(None);
        Ok(Some(p))
      }
      Err(e) if e.is_network() => {
        // Offline — keep the optimistic entry, queue for sync.
        let op = PendingOp {
          id: uuid::Uuid::new_v4().to_string(),
          table: "reading_progress".to_string(),
          kind: OpKind::Upsert {
            on_conflict: "user_id,chapter_id".to_string(),
          },
          payload: body,
          created_at: now,
        };
        let _ = sync::enqueue(op).await;

        // Cache the optimistic entry in IndexedDB.
        let _ = db::put(db::stores::PROGRESS, &optimistic).await;
        self.error.set(None);
        Ok(Some(optimistic))
      }
      Err(e) => {
        self.map.set(snapshot);
        self.error.set(Some(e.to_string()));
        Err(e)
      }
    }
  }

  fn set_entry(&self, cid: uuid::Uuid, progress: Progress) {
    let mut cur = self.map.get_untracked();
    cur.insert(cid, progress);
    self.map.set(cur);
  }

  /// Log time against a chapter — queue on offline.
  pub async fn log_time(&self, cid: uuid::Uuid, seconds: i32) -> AppResult<Option<Progress>> {
    if seconds <= 0 {
      return Ok(None);
    }
    let a = use_auth();
    if a.user.get_untracked().is_none() {
      return Ok(None);
    }
    let Some(uid) = a.user.get_untracked() else {
      return Ok(None);
    };
    let ex = self.get(cid);
    let nt = ex.as_ref().map_or(0, |p| p.time_spent_seconds) + seconds;

    let now = now_iso();
    let body = serde_json::json!({
      "user_id": uid,
      "chapter_id": cid,
      "status": ex.as_ref().map(|p| p.status).unwrap_or_default().as_str(),
      "time_spent_seconds": nt,
      "updated_at": &now,
    });

    let result = authed(|c| {
      let body = &body;
      async move {
        let p: Progress = c
          .postgrest()
          .from("reading_progress")
          .upsert_one(body, "user_id,chapter_id")
          .await?;
        AppResult::Ok(p)
      }
    })
    .await;

    match result {
      Ok(p) => {
        self.set_entry(cid, p.clone());
        let _ = db::put(db::stores::PROGRESS, &p).await;
        Ok(Some(p))
      }
      Err(e) if e.is_network() => {
        // Offline — update local entry, queue for sync.
        let updated = Progress {
          time_spent_seconds: nt,
          ..ex.unwrap_or_else(|| Progress {
            id: uuid::Uuid::nil(),
            user_id: uid,
            chapter_id: cid,
            status: ReadingStatus::NotStarted,
            time_spent_seconds: 0,
            updated_at: chrono::Utc::now(),
          })
        };
        self.set_entry(cid, updated.clone());
        let _ = db::put(db::stores::PROGRESS, &updated).await;

        let op = PendingOp {
          id: uuid::Uuid::new_v4().to_string(),
          table: "reading_progress".to_string(),
          kind: OpKind::Upsert {
            on_conflict: "user_id,chapter_id".to_string(),
          },
          payload: body,
          created_at: now,
        };
        let _ = sync::enqueue(op).await;

        Ok(Some(updated))
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

/// The chapter a reader should pick up where they left off.
#[must_use]
pub fn continue_reading(
  chapters: &[crate::core::types::Chapter],
  progress: &std::collections::HashMap<uuid::Uuid, Progress>,
) -> Option<uuid::Uuid> {
  use chrono::DateTime;
  let in_progress: Vec<&crate::core::types::Chapter> = chapters
    .iter()
    .filter(|c| {
      progress
        .get(&c.id)
        .map_or(true, |p| p.status != ReadingStatus::Completed)
    })
    .collect();
  if in_progress.is_empty() {
    return None;
  }
  in_progress
    .iter()
    .max_by(|a, b| {
      let ta = progress
        .get(&a.id)
        .map_or(DateTime::<chrono::Utc>::MIN_UTC, |p| p.updated_at);
      let tb = progress
        .get(&b.id)
        .map_or(DateTime::<chrono::Utc>::MIN_UTC, |p| p.updated_at);
      ta.cmp(&tb)
        .then(b.sequence_number.total_cmp(&a.sequence_number))
    })
    .map(|c| c.id)
}

/// Build the progress entry to show optimistically for a status change.
fn optimistic_status(
  existing: Option<&Progress>,
  uid: uuid::Uuid,
  cid: uuid::Uuid,
  status: ReadingStatus,
) -> Progress {
  existing.map_or_else(
    || Progress {
      id: uuid::Uuid::nil(),
      user_id: uid,
      chapter_id: cid,
      status,
      time_spent_seconds: 0,
      updated_at: chrono::Utc::now(),
    },
    |p| Progress {
      status,
      updated_at: chrono::Utc::now(),
      ..p.clone()
    },
  )
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ProgressWithBook {
  id: uuid::Uuid,
  user_id: uuid::Uuid,
  chapter_id: uuid::Uuid,
  status: ReadingStatus,
  time_spent_seconds: i32,
  updated_at: chrono::DateTime<chrono::Utc>,
  #[serde(default)]
  #[allow(dead_code)]
  reading_chapters: Option<BookRef>,
}
#[derive(Debug, Clone, serde::Deserialize)]
struct BookRef {
  #[allow(dead_code)]
  book_id: uuid::Uuid,
}
impl ProgressWithBook {
  const fn into_progress(self) -> Progress {
    Progress {
      id: self.id,
      user_id: self.user_id,
      chapter_id: self.chapter_id,
      status: self.status,
      time_spent_seconds: self.time_spent_seconds,
      updated_at: self.updated_at,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn optimistic_status_preserves_existing_id_and_time() {
    let uid = uuid::Uuid::from_u128(1);
    let cid = uuid::Uuid::from_u128(2);
    let existing = Some(Progress {
      id: uuid::Uuid::from_u128(3),
      user_id: uid,
      chapter_id: cid,
      status: ReadingStatus::InProgress,
      time_spent_seconds: 120,
      updated_at: chrono::Utc::now(),
    });

    let next = optimistic_status(existing.as_ref(), uid, cid, ReadingStatus::Completed);

    assert_eq!(next.id, uuid::Uuid::from_u128(3), "keeps the real row id");
    assert_eq!(next.status, ReadingStatus::Completed, "reflects new status");
    assert_eq!(
      next.time_spent_seconds, 120,
      "does not discard accumulated time"
    );
  }

  #[test]
  fn optimistic_status_seeds_a_fresh_entry() {
    let uid = uuid::Uuid::from_u128(1);
    let cid = uuid::Uuid::from_u128(2);

    let next = optimistic_status(None, uid, cid, ReadingStatus::InProgress);

    assert_eq!(next.user_id, uid);
    assert_eq!(next.chapter_id, cid);
    assert_eq!(next.status, ReadingStatus::InProgress);
    assert_eq!(
      next.time_spent_seconds, 0,
      "a new entry starts at zero time"
    );
  }

  fn ch(id: u128, seq: f64) -> crate::core::types::Chapter {
    crate::core::types::Chapter {
      id: uuid::Uuid::from_u128(id),
      book_id: uuid::Uuid::from_u128(99),
      title: format!("Ch {seq}"),
      sequence_number: seq,
      parent_id: None,
      children: Vec::new(),
    }
  }

  fn prog(id: u128, status: ReadingStatus, secs: i64) -> Progress {
    Progress {
      id: uuid::Uuid::from_u128(id),
      user_id: uuid::Uuid::from_u128(1),
      chapter_id: uuid::Uuid::from_u128(id),
      status,
      time_spent_seconds: 0,
      updated_at: chrono::Utc::now() - chrono::Duration::seconds(secs),
    }
  }

  #[test]
  fn continue_reading_picks_most_recent_non_completed() {
    let chapters = vec![ch(1, 1.0), ch(2, 2.0), ch(3, 3.0)];
    let mut map = std::collections::HashMap::new();
    map.insert(
      uuid::Uuid::from_u128(1),
      prog(1, ReadingStatus::InProgress, 100),
    );
    map.insert(
      uuid::Uuid::from_u128(2),
      prog(2, ReadingStatus::NotStarted, 10),
    );
    map.insert(
      uuid::Uuid::from_u128(3),
      prog(3, ReadingStatus::Completed, 0),
    );

    let next = continue_reading(&chapters, &map);
    assert_eq!(
      next,
      Some(uuid::Uuid::from_u128(2)),
      "most recently updated, not completed"
    );
  }

  #[test]
  fn continue_reading_skips_all_completed() {
    let chapters = vec![ch(1, 1.0), ch(2, 2.0)];
    let mut map = std::collections::HashMap::new();
    map.insert(
      uuid::Uuid::from_u128(1),
      prog(1, ReadingStatus::Completed, 0),
    );
    map.insert(
      uuid::Uuid::from_u128(2),
      prog(2, ReadingStatus::Completed, 0),
    );

    assert_eq!(
      continue_reading(&chapters, &map),
      None,
      "nothing left to read"
    );
  }

  #[test]
  fn continue_reading_falls_back_to_first_without_progress() {
    let chapters = vec![ch(1, 1.0), ch(2, 2.0)];
    let map = std::collections::HashMap::new();
    assert_eq!(
      continue_reading(&chapters, &map),
      Some(uuid::Uuid::from_u128(1))
    );
  }

  #[test]
  fn continue_reading_empty_when_no_chapters() {
    let map = std::collections::HashMap::new();
    assert_eq!(continue_reading(&[], &map), None);
  }
}
