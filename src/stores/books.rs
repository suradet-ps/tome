//! Books and chapters state (context-based).

use crate::core::error::AppResult;
use crate::core::supabase;
use crate::core::types::{Book, Chapter};
use crate::stores::auth::use_auth;
use leptos::prelude::*;

const MAX_TITLE_LENGTH: usize = 200;
const MAX_AUTHOR_LENGTH: usize = 200;

#[derive(Debug, Clone, Copy)]
pub struct BooksState {
    pub books: RwSignal<Vec<Book>>,
    pub chapters: RwSignal<Vec<Chapter>>,
    pub current_book_id: RwSignal<Option<uuid::Uuid>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
}

impl BooksState {
    pub fn provide() -> Self {
        let state = Self {
            books: RwSignal::new(Vec::new()),
            chapters: RwSignal::new(Vec::new()),
            current_book_id: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(None),
        };
        provide_context(state);
        state
    }

    #[must_use]
    pub fn use_ctx() -> Self {
        use_context::<Self>().expect("BooksState must be provided at the root")
    }

    #[must_use]
    pub fn book(&self, id: uuid::Uuid) -> Option<Book> {
        self.books.get().into_iter().find(|b| b.id == id)
    }

    pub async fn fetch_books(&self) -> AppResult<Vec<Book>> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() { self.books.set(Vec::new()); return Ok(Vec::new()); }
        self.loading.set(true);
        self.error.set(None);
        let Some(user_id) = auth.user.get_untracked() else { self.loading.set(false); return Ok(Vec::new()); };
        let result = async {
            let client = supabase::supabase()?;
            let books: Vec<Book> = client.postgrest().from("reading_books").select("*").eq("user_id", user_id.to_string()).order("created_at", false).range(0, 999).get().await?;
            self.books.set(books.clone());
            AppResult::Ok(books)
        }.await;
        match result { Ok(b) => { self.loading.set(false); Ok(b) } Err(e) => { self.error.set(Some(e.to_string())); self.loading.set(false); Err(e) } }
    }

    pub async fn fetch_book(&self, id: uuid::Uuid) -> AppResult<Option<Book>> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() { return Ok(None); }
        let Some(user_id) = auth.user.get_untracked() else { return Ok(None); };
        let client = supabase::supabase()?;
        let book: Option<Book> = client.postgrest().from("reading_books").select("*").eq("id", id.to_string()).eq("user_id", user_id.to_string()).get_one().await?;
        if let Some(ref book) = book { let mut c = self.books.get(); if let Some(ex) = c.iter_mut().find(|b| b.id == book.id) { *ex = book.clone(); } else { c.insert(0, book.clone()); } self.books.set(c); }
        Ok(book)
    }

    pub async fn add_book(&self, title: &str, author: &str) -> AppResult<Option<Book>> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() { return Ok(None); }
        let t = truncate(title.trim(), MAX_TITLE_LENGTH);
        if t.is_empty() { return Ok(None); }
        let a = truncate(author.trim(), MAX_AUTHOR_LENGTH);
        let Some(user_id) = auth.user.get_untracked() else { return Ok(None); };
        let client = supabase::supabase()?;
        let body = serde_json::json!({"user_id": user_id, "title": t, "author": if a.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(a) }});
        let book: Book = client.postgrest().from("reading_books").insert_one(&body).await?;
        let mut c = self.books.get(); c.insert(0, book.clone()); self.books.set(c); self.current_book_id.set(Some(book.id));
        Ok(Some(book))
    }

    pub async fn fetch_chapters(&self, book_id: uuid::Uuid) -> AppResult<Vec<Chapter>> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() { self.chapters.set(Vec::new()); return Ok(Vec::new()); }
        self.loading.set(true); self.error.set(None);
        let result = async {
            let client = supabase::supabase()?;
            let flat: Vec<Chapter> = client.postgrest().from("reading_chapters").select("*").eq("book_id", book_id.to_string()).order("sequence_number", true).range(0, 4999).get().await?;
            let tree = build_chapter_tree(flat); self.chapters.set(tree.clone()); self.current_book_id.set(Some(book_id));
            AppResult::Ok(tree)
        }.await;
        match result { Ok(t) => { self.loading.set(false); Ok(t) } Err(e) => { self.error.set(Some(e.to_string())); self.loading.set(false); Err(e) } }
    }

    pub async fn add_chapter(&self, book_id: uuid::Uuid, title: &str, seq: f64, parent_id: Option<uuid::Uuid>) -> AppResult<()> {
        let t = title.trim();
        if t.is_empty() { return Ok(()); }
        let t = truncate(t, MAX_TITLE_LENGTH);
        let client = supabase::supabase()?;
        let body = serde_json::json!({"book_id": book_id, "title": t, "sequence_number": seq, "parent_id": parent_id});
        client.postgrest().from("reading_chapters").insert::<Chapter, _>(&body).await?;
        self.fetch_chapters(book_id).await?;
        Ok(())
    }

    #[must_use]
    pub fn flat_chapters(&self) -> Vec<Chapter> { flatten_tree(&self.chapters.get()) }

    pub fn reset(&self) { self.books.set(Vec::new()); self.chapters.set(Vec::new()); self.current_book_id.set(None); }
}

pub fn build_chapter_tree(flat: Vec<Chapter>) -> Vec<Chapter> {
    let mut map: std::collections::HashMap<uuid::Uuid, Chapter> = std::collections::HashMap::new();
    let mut roots = Vec::new();
    for c in &flat { map.insert(c.id, Chapter { children: Vec::new(), ..c.clone() }); }
    for c in &flat {
        let Some(node) = map.get(&c.id).cloned() else { continue };
        if let Some(pid) = c.parent_id {
            if let Some(p) = map.get_mut(&pid) { p.children.push(node); } else { roots.push(node); }
        } else { roots.push(node); }
    }
    sort_tree(&mut roots);
    roots
}

fn sort_tree(nodes: &mut [Chapter]) {
    nodes.sort_by(|a, b| a.sequence_number.partial_cmp(&b.sequence_number).unwrap_or(std::cmp::Ordering::Equal));
    for n in nodes.iter_mut() { sort_tree(&mut n.children); }
}

fn flatten_tree(tree: &[Chapter]) -> Vec<Chapter> {
    let mut r = Vec::new();
    for c in tree { r.push(c.clone()); r.extend(flatten_tree(&c.children)); }
    r
}

fn truncate(v: &str, max: usize) -> String {
    if v.chars().count() <= max { v.to_string() } else { v.chars().take(max).collect() }
}
