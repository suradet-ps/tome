//! Books and chapters state (root-scoped singleton).

use crate::core::error::AppResult;
use crate::core::supabase;
use crate::core::types::{Book, Chapter};
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use std::sync::OnceLock;

static STATE: OnceLock<BooksState> = OnceLock::new();

pub fn install() {
    let _ = STATE.set(BooksState::new());
}

const MAX_TITLE: usize = 200;
const MAX_AUTHOR: usize = 200;

#[derive(Debug, Clone, Copy)]
pub struct BooksState {
    pub books: RwSignal<Vec<Book>>,
    pub chapters: RwSignal<Vec<Chapter>>,
    pub current_book_id: RwSignal<Option<uuid::Uuid>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
}

impl Default for BooksState {
    fn default() -> Self {
        Self::new()
    }
}

impl BooksState {
    pub fn new() -> Self {
        Self {
            books: RwSignal::new(Vec::new()),
            chapters: RwSignal::new(Vec::new()),
            current_book_id: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(None),
        }
    }
    pub fn use_ctx() -> Self {
        *STATE.get().expect("BooksState not initialized")
    }
    pub fn book(&self, id: uuid::Uuid) -> Option<Book> {
        self.books.get().into_iter().find(|b| b.id == id)
    }

    pub async fn fetch_books(&self) -> AppResult<Vec<Book>> {
        let a = use_auth();
        if a.user.get_untracked().is_none() {
            self.books.set(Vec::new());
            return Ok(Vec::new());
        }
        self.loading.set(true);
        self.error.set(None);
        let Some(uid) = a.user.get_untracked() else {
            self.loading.set(false);
            return Ok(Vec::new());
        };
        let r = async {
            let c = supabase::supabase()?;
            let b: Vec<Book> = c
                .postgrest()
                .from("reading_books")
                .select("*")
                .eq("user_id", uid.to_string())
                .order("created_at", false)
                .range(0, 999)
                .get()
                .await?;
            self.books.set(b.clone());
            AppResult::Ok(b)
        }
        .await;
        match r {
            Ok(b) => {
                self.loading.set(false);
                Ok(b)
            }
            Err(e) => {
                self.error.set(Some(e.to_string()));
                self.loading.set(false);
                Err(e)
            }
        }
    }

    pub async fn fetch_book(&self, id: uuid::Uuid) -> AppResult<Option<Book>> {
        let a = use_auth();
        if a.user.get_untracked().is_none() {
            return Ok(None);
        }
        let Some(uid) = a.user.get_untracked() else {
            return Ok(None);
        };
        let c = supabase::supabase()?;
        let b: Option<Book> = c
            .postgrest()
            .from("reading_books")
            .select("*")
            .eq("id", id.to_string())
            .eq("user_id", uid.to_string())
            .get_one()
            .await?;
        if let Some(ref b) = b {
            let mut cur = self.books.get();
            if let Some(ex) = cur.iter_mut().find(|x| x.id == b.id) {
                *ex = b.clone();
            } else {
                cur.insert(0, b.clone());
            }
            self.books.set(cur);
        }
        Ok(b)
    }

    pub async fn add_book(&self, title: &str, author: &str) -> AppResult<Option<Book>> {
        let a = use_auth();
        if a.user.get_untracked().is_none() {
            return Ok(None);
        }
        let t = trunc(title.trim(), MAX_TITLE);
        if t.is_empty() {
            return Ok(None);
        }
        let au = trunc(author.trim(), MAX_AUTHOR);
        let Some(uid) = a.user.get_untracked() else {
            return Ok(None);
        };
        let c = supabase::supabase()?;
        let body = serde_json::json!({"user_id":uid,"title":t,"author":if au.is_empty(){serde_json::Value::Null}else{serde_json::Value::String(au)}});
        let b: Book = c
            .postgrest()
            .from("reading_books")
            .insert_one(&body)
            .await?;
        let mut cur = self.books.get();
        cur.insert(0, b.clone());
        self.books.set(cur);
        self.current_book_id.set(Some(b.id));
        Ok(Some(b))
    }

    pub async fn fetch_chapters(&self, book_id: uuid::Uuid) -> AppResult<Vec<Chapter>> {
        let a = use_auth();
        if a.user.get_untracked().is_none() {
            self.chapters.set(Vec::new());
            return Ok(Vec::new());
        }
        self.loading.set(true);
        self.error.set(None);
        let r = async {
            let c = supabase::supabase()?;
            let f: Vec<Chapter> = c
                .postgrest()
                .from("reading_chapters")
                .select("*")
                .eq("book_id", book_id.to_string())
                .order("sequence_number", true)
                .range(0, 4999)
                .get()
                .await?;
            let t = build_chapter_tree(f);
            self.chapters.set(t.clone());
            self.current_book_id.set(Some(book_id));
            AppResult::Ok(t)
        }
        .await;
        match r {
            Ok(t) => {
                self.loading.set(false);
                Ok(t)
            }
            Err(e) => {
                self.error.set(Some(e.to_string()));
                self.loading.set(false);
                Err(e)
            }
        }
    }

    pub async fn add_chapter(
        &self,
        bid: uuid::Uuid,
        title: &str,
        seq: f64,
        pid: Option<uuid::Uuid>,
    ) -> AppResult<()> {
        let t = title.trim();
        if t.is_empty() {
            return Ok(());
        }
        let t = trunc(t, MAX_TITLE);
        let c = supabase::supabase()?;
        c.postgrest()
            .from("reading_chapters")
            .insert::<Chapter, _>(
                &serde_json::json!({"book_id":bid,"title":t,"sequence_number":seq,"parent_id":pid}),
            )
            .await?;
        self.fetch_chapters(bid).await?;
        Ok(())
    }

    pub fn flat_chapters(&self) -> Vec<Chapter> {
        flatten_tree(&self.chapters.get())
    }
    pub fn reset(&self) {
        self.books.set(Vec::new());
        self.chapters.set(Vec::new());
        self.current_book_id.set(None);
    }
}

pub fn build_chapter_tree(flat: Vec<Chapter>) -> Vec<Chapter> {
    let mut map: std::collections::HashMap<uuid::Uuid, Chapter> = std::collections::HashMap::new();
    let mut roots = Vec::new();
    for c in &flat {
        map.insert(
            c.id,
            Chapter {
                children: Vec::new(),
                ..c.clone()
            },
        );
    }
    for c in &flat {
        let Some(n) = map.get(&c.id).cloned() else {
            continue;
        };
        if let Some(pid) = c.parent_id {
            if let Some(p) = map.get_mut(&pid) {
                p.children.push(n);
            } else {
                roots.push(n);
            }
        } else {
            roots.push(n);
        }
    }
    sort_tree(&mut roots);
    roots
}
fn sort_tree(n: &mut [Chapter]) {
    n.sort_by(|a, b| {
        a.sequence_number
            .partial_cmp(&b.sequence_number)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for c in n.iter_mut() {
        sort_tree(&mut c.children);
    }
}
fn flatten_tree(t: &[Chapter]) -> Vec<Chapter> {
    let mut r = Vec::new();
    for c in t {
        r.push(c.clone());
        r.extend(flatten_tree(&c.children));
    }
    r
}
fn trunc(v: &str, max: usize) -> String {
    if v.chars().count() <= max {
        v.to_string()
    } else {
        v.chars().take(max).collect()
    }
}
