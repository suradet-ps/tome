//! Books and chapters state (module-level singleton).

use crate::core::error::AppResult;
use crate::core::supabase;
use crate::core::types::{Book, Chapter};
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use std::cell::LazyCell;

/// Maximum length for book/chapter titles.
const MAX_TITLE_LENGTH: usize = 200;
/// Maximum length for author name.
const MAX_AUTHOR_LENGTH: usize = 200;

thread_local! {
    static STATE: LazyCell<BooksState> = LazyCell::new(BooksState::default);
}

/// Reactive container for the books and chapters.
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
        Self {
            books: RwSignal::new(Vec::new()),
            chapters: RwSignal::new(Vec::new()),
            current_book_id: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(None),
        }
    }
}

impl BooksState {
    /// Read the state singleton.
    #[must_use]
    pub fn use_ctx() -> Self {
        STATE.with(|cell| **cell)
    }

    #[must_use]
    pub fn book(&self, id: uuid::Uuid) -> Option<Book> {
        self.books.get().into_iter().find(|b| b.id == id)
    }

    pub async fn fetch_books(&self) -> AppResult<Vec<Book>> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() {
            self.books.set(Vec::new());
            return Ok(Vec::new());
        }
        self.loading.set(true);
        self.error.set(None);
        let user_id = match auth.user.get_untracked() {
            Some(id) => id,
            None => {
                self.loading.set(false);
                return Ok(Vec::new());
            },
        };
        let result = async {
            let client = supabase::supabase()?;
            let books: Vec<Book> = client
                .postgrest()
                .from("reading_books")
                .select("*")
                .eq("user_id", user_id.to_string())
                .order("created_at", false)
                .range(0, 999)
                .get()
                .await?;
            self.books.set(books.clone());
            AppResult::Ok(books)
        }
        .await;
        match result {
            Ok(books) => {
                self.loading.set(false);
                Ok(books)
            },
            Err(err) => {
                self.error.set(Some(err.to_string()));
                self.loading.set(false);
                Err(err)
            },
        }
    }

    pub async fn fetch_book(&self, id: uuid::Uuid) -> AppResult<Option<Book>> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() {
            return Ok(None);
        }
        let user_id = match auth.user.get_untracked() {
            Some(id) => id,
            None => return Ok(None),
        };
        let client = supabase::supabase()?;
        let book: Option<Book> = client
            .postgrest()
            .from("reading_books")
            .select("*")
            .eq("id", id.to_string())
            .eq("user_id", user_id.to_string())
            .get_one()
            .await?;
        if let Some(ref book) = book {
            let mut current = self.books.get();
            if let Some(existing) = current.iter_mut().find(|b| b.id == book.id) {
                *existing = book.clone();
            } else {
                current.insert(0, book.clone());
            }
            self.books.set(current);
        }
        Ok(book)
    }

    pub async fn add_book(&self, title: &str, author: &str) -> AppResult<Option<Book>> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() {
            return Ok(None);
        }
        let trimmed_title = truncate(title.trim(), MAX_TITLE_LENGTH);
        if trimmed_title.is_empty() {
            return Ok(None);
        }
        let trimmed_author = truncate(author.trim(), MAX_AUTHOR_LENGTH);
        let user_id = match auth.user.get_untracked() {
            Some(id) => id,
            None => return Ok(None),
        };
        let client = supabase::supabase()?;
        let body = serde_json::json!({
            "user_id": user_id,
            "title": trimmed_title,
            "author": if trimmed_author.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(trimmed_author) },
        });
        let book: Book = client
            .postgrest()
            .from("reading_books")
            .insert_one(&body)
            .await?;
        let mut current = self.books.get();
        current.insert(0, book.clone());
        self.books.set(current);
        self.current_book_id.set(Some(book.id));
        Ok(Some(book))
    }

    pub async fn fetch_chapters(&self, book_id: uuid::Uuid) -> AppResult<Vec<Chapter>> {
        let auth = use_auth();
        if auth.user.get_untracked().is_none() {
            self.chapters.set(Vec::new());
            return Ok(Vec::new());
        }
        self.loading.set(true);
        self.error.set(None);
        let result = async {
            let client = supabase::supabase()?;
            let flat: Vec<Chapter> = client
                .postgrest()
                .from("reading_chapters")
                .select("*")
                .eq("book_id", book_id.to_string())
                .order("sequence_number", true)
                .range(0, 4999)
                .get()
                .await?;
            let tree = build_chapter_tree(flat);
            self.chapters.set(tree.clone());
            self.current_book_id.set(Some(book_id));
            AppResult::Ok(tree)
        }
        .await;
        match result {
            Ok(tree) => {
                self.loading.set(false);
                Ok(tree)
            },
            Err(err) => {
                self.error.set(Some(err.to_string()));
                self.loading.set(false);
                Err(err)
            },
        }
    }

    pub async fn add_chapter(
        &self,
        book_id: uuid::Uuid,
        title: &str,
        sequence_number: f64,
        parent_id: Option<uuid::Uuid>,
    ) -> AppResult<()> {
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Ok(());
        }
        let trimmed_title = truncate(trimmed_title, MAX_TITLE_LENGTH);
        let client = supabase::supabase()?;
        let body = serde_json::json!({
            "book_id": book_id,
            "title": trimmed_title,
            "sequence_number": sequence_number,
            "parent_id": parent_id,
        });
        client
            .postgrest()
            .from("reading_chapters")
            .insert::<Chapter, _>(&body)
            .await?;
        self.fetch_chapters(book_id).await?;
        Ok(())
    }

    #[must_use]
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
    let mut roots: Vec<Chapter> = Vec::new();
    for chapter in &flat {
        map.insert(chapter.id, Chapter { children: Vec::new(), ..chapter.clone() });
    }
    for chapter in &flat {
        let node = match map.get(&chapter.id) { Some(n) => n.clone(), None => continue };
        if let Some(parent_id) = chapter.parent_id {
            if let Some(parent) = map.get_mut(&parent_id) {
                parent.children.push(node);
            } else {
                roots.push(node);
            }
        } else {
            roots.push(node);
        }
    }
    sort_tree(&mut roots);
    roots
}

fn sort_tree(nodes: &mut [Chapter]) {
    nodes.sort_by(|a, b| a.sequence_number.partial_cmp(&b.sequence_number).unwrap_or(std::cmp::Ordering::Equal));
    for node in nodes.iter_mut() {
        sort_tree(&mut node.children);
    }
}

fn flatten_tree(tree: &[Chapter]) -> Vec<Chapter> {
    let mut result = Vec::new();
    for chapter in tree {
        result.push(chapter.clone());
        result.extend(flatten_tree(&chapter.children));
    }
    result
}

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars { value.to_string() } else { value.chars().take(max_chars).collect() }
}
