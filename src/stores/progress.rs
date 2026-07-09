//! Per-chapter progress (context-based).

use crate::core::error::AppResult;
use crate::core::supabase;
use crate::core::time::now_iso;
use crate::core::types::{Progress, ReadingStatus};
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct ProgressState {
    pub map: RwSignal<HashMap<uuid::Uuid, Progress>>,
    pub error: RwSignal<Option<String>>,
}

impl ProgressState {
    pub fn provide() -> Self {
        let s = Self { map: RwSignal::new(HashMap::new()), error: RwSignal::new(None) };
        provide_context(s);
        s
    }
    #[must_use] pub fn use_ctx() -> Self { use_context::<Self>().expect("ProgressState must be provided at the root") }
    #[must_use] pub fn get(&self, id: uuid::Uuid) -> Option<Progress> { self.map.get().get(&id).cloned() }

    pub async fn fetch_for_book(&self, book_id: uuid::Uuid) -> AppResult<()> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() { return Ok(()); }
        let Some(user_id) = auth.user.get_untracked() else { return Ok(()); };
        let client = supabase::supabase()?;
        let rows: Vec<ProgressWithBook> = client.postgrest().from("reading_progress")
            .select("id, user_id, chapter_id, status, time_spent_seconds, updated_at, reading_chapters!inner(book_id)")
            .eq("user_id", user_id.to_string()).eq("reading_chapters.book_id", book_id.to_string())
            .range(0, 4999).get().await?;
        let mut next = self.map.get();
        for row in rows { next.insert(row.chapter_id, row.into_progress()); }
        self.map.set(next);
        Ok(())
    }

    pub async fn update_status(&self, chapter_id: uuid::Uuid, status: ReadingStatus) -> AppResult<Option<Progress>> {
        let auth = use_auth(); if auth.user.get_untracked().is_none() { return Ok(None); }
        let Some(user_id) = auth.user.get_untracked() else { return Ok(None); };
        let client = supabase::supabase()?;
        let existing = self.get(chapter_id);
        let body = serde_json::json!({"user_id":user_id,"chapter_id":chapter_id,"status":status.as_str(),"time_spent_seconds":existing.as_ref().map(|p|p.time_spent_seconds).unwrap_or(0),"updated_at":now_iso()});
        let progress: Progress = client.postgrest().from("reading_progress").upsert_one(&body, "user_id,chapter_id").await?;
        let mut c = self.map.get(); c.insert(progress.chapter_id, progress.clone()); self.map.set(c);
        Ok(Some(progress))
    }

    pub async fn log_time(&self, chapter_id: uuid::Uuid, seconds: i32) -> AppResult<Option<Progress>> {
        if seconds <= 0 { return Ok(None); }
        let auth = use_auth(); if auth.user.get_untracked().is_none() { return Ok(None); }
        let Some(user_id) = auth.user.get_untracked() else { return Ok(None); };
        let client = supabase::supabase()?;
        let existing = self.get(chapter_id);
        let nt = existing.as_ref().map(|p| p.time_spent_seconds).unwrap_or(0) + seconds;
        let body = serde_json::json!({"user_id":user_id,"chapter_id":chapter_id,"status":existing.as_ref().map(|p|p.status).unwrap_or_default().as_str(),"time_spent_seconds":nt,"updated_at":now_iso()});
        let progress: Progress = client.postgrest().from("reading_progress").upsert_one(&body, "user_id,chapter_id").await?;
        let mut c = self.map.get(); c.insert(progress.chapter_id, progress.clone()); self.map.set(c);
        Ok(Some(progress))
    }

    pub fn reset(&self) { self.map.set(HashMap::new()); }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ProgressWithBook {
    id: uuid::Uuid, user_id: uuid::Uuid, chapter_id: uuid::Uuid,
    status: ReadingStatus, time_spent_seconds: i32,
    updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)] #[allow(dead_code)] reading_chapters: Option<BookRef>,
}
#[derive(Debug, Clone, serde::Deserialize)]
struct BookRef { #[allow(dead_code)] book_id: uuid::Uuid }

impl ProgressWithBook {
    fn into_progress(self) -> Progress {
        Progress { id: self.id, user_id: self.user_id, chapter_id: self.chapter_id, status: self.status, time_spent_seconds: self.time_spent_seconds, updated_at: self.updated_at }
    }
}
