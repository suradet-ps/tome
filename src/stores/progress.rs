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
        let c = supabase::supabase()?;
        let ex = self.get(cid);
        let body = serde_json::json!({"user_id":uid,"chapter_id":cid,"status":status.as_str(),"time_spent_seconds":ex.as_ref().map_or(0, |p|p.time_spent_seconds),"updated_at":now_iso()});
        let p: Progress = c
            .postgrest()
            .from("reading_progress")
            .upsert_one(&body, "user_id,chapter_id")
            .await?;
        let mut cur = self.map.get();
        cur.insert(p.chapter_id, p.clone());
        self.map.set(cur);
        Ok(Some(p))
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
        let c = supabase::supabase()?;
        let ex = self.get(cid);
        let nt = ex.as_ref().map_or(0, |p| p.time_spent_seconds) + seconds;
        let body = serde_json::json!({"user_id":uid,"chapter_id":cid,"status":ex.as_ref().map(|p|p.status).unwrap_or_default().as_str(),"time_spent_seconds":nt,"updated_at":now_iso()});
        let p: Progress = c
            .postgrest()
            .from("reading_progress")
            .upsert_one(&body, "user_id,chapter_id")
            .await?;
        let mut cur = self.map.get();
        cur.insert(p.chapter_id, p.clone());
        self.map.set(cur);
        Ok(Some(p))
    }

    pub fn reset(&self) {
        self.map.set(HashMap::new());
    }
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
