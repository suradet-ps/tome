//! Per-chapter progress (root-scoped singleton).

use crate::core::error::AppResult;
use crate::core::supabase;
use crate::core::time::now_iso;
use crate::core::types::{Progress, ReadingStatus};
use crate::stores::auth::use_auth;
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

  pub async fn fetch_for_book(&self, bid: uuid::Uuid) -> AppResult<()> {
    let a = use_auth();
    if a.user.get_untracked().is_none() {
      return Ok(());
    }
    let Some(uid) = a.user.get_untracked() else {
      return Ok(());
    };
    let c = supabase::supabase()?;
    let rows: Vec<ProgressWithBook> = c.postgrest().from("reading_progress").select("id,user_id,chapter_id,status,time_spent_seconds,updated_at,reading_chapters!inner(book_id)").eq("user_id", uid.to_string()).eq("reading_chapters.book_id", bid.to_string()).range(0,4999).get().await?;
    let mut next = self.map.get();
    for row in rows {
      next.insert(row.chapter_id, row.into_progress());
    }
    self.map.set(next);
    Ok(())
  }

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

    // Optimistic update: reflect the new status immediately so the UI feels
    // instant, remembering the previous entry so it can be rolled back if the
    // write fails. Without this the checkbox would only flip after the round
    // trip; with it a failed write no longer leaves the UI lying about the DB.
    let snapshot = self.map.get_untracked();
    let optimistic = optimistic_status(ex.as_ref(), uid, cid, status);
    self.set_entry(cid, optimistic);

    let result = async {
      let c = supabase::supabase()?;
      let body = serde_json::json!({"user_id":uid,"chapter_id":cid,"status":status.as_str(),"time_spent_seconds":ex.as_ref().map_or(0, |p|p.time_spent_seconds),"updated_at":now_iso()});
      let p: Progress = c
        .postgrest()
        .from("reading_progress")
        .upsert_one(&body, "user_id,chapter_id")
        .await?;
      AppResult::Ok(p)
    }
    .await;

    match result {
      Ok(p) => {
        // Reconcile with the server's authoritative row (real id/updated_at).
        self.set_entry(cid, p.clone());
        self.error.set(None);
        Ok(Some(p))
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

    // Time logging runs in the background on chapter switch, so there is no
    // instant-feedback need to update optimistically. But the write must still
    // fail loudly: on error the signal is left untouched (no divergence) and
    // the error surfaces instead of being swallowed.
    let result = async {
      let c = supabase::supabase()?;
      let body = serde_json::json!({"user_id":uid,"chapter_id":cid,"status":ex.as_ref().map(|p|p.status).unwrap_or_default().as_str(),"time_spent_seconds":nt,"updated_at":now_iso()});
      let p: Progress = c
        .postgrest()
        .from("reading_progress")
        .upsert_one(&body, "user_id,chapter_id")
        .await?;
      AppResult::Ok(p)
    }
    .await;

    match result {
      Ok(p) => {
        self.set_entry(cid, p.clone());
        Ok(Some(p))
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
///
/// Given the flat chapter list and the progress map, returns the id of the
/// chapter with the most recent `updated_at` whose status is not `Completed`
/// (you don't "continue" a chapter you've finished). Chapters with no
/// progress row fall back to the first non-completed chapter in sequence
/// order, then to `None`. Pure so the "continue reading" pick — which the
/// dashboard surfaces as a calm re-entry point — can be tested without
/// signals or a network call.
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
      // Most recently updated wins; on a tie, the earlier chapter in
      // sequence order wins (a calm, predictable re-entry point).
      ta.cmp(&tb)
        .then(b.sequence_number.total_cmp(&a.sequence_number))
    })
    .map(|c| c.id)
}

/// Build the progress entry to show optimistically for a status change, before
/// the server confirms it. Reuses the existing row's id, user and accumulated
/// time when present; otherwise seeds a fresh entry. Pure so the optimistic
/// shape can be tested without a signal or a network call.
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
    // c1 oldest in-progress, c2 recently touched, c3 completed (excluded).
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
    // No progress rows: calm default is the first chapter in sequence.
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
