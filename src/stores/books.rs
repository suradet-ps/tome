//! Books and chapters state (root-scoped singleton).

use crate::core::error::AppResult;
use crate::core::supabase;
use crate::core::types::{Book, Chapter};
use crate::core::validate;
use crate::stores::auth::use_auth;
use leptos::prelude::*;
use std::sync::OnceLock;

static STATE: OnceLock<BooksState> = OnceLock::new();

pub fn install() {
  let _ = STATE.set(BooksState::new());
}

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
    if title.trim().is_empty() {
      return Ok(None);
    }
    let t = validate::check_title(title)?;
    let au = validate::check_author(author)?;
    let Some(uid) = a.user.get_untracked() else {
      return Ok(None);
    };
    let c = supabase::supabase()?;
    let body = serde_json::json!({"user_id":uid,"title":t,"author":if au.is_empty(){serde_json::Value::Null}else{serde_json::Value::String(au.to_string())}});
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
    if title.trim().is_empty() {
      return Ok(());
    }
    let t = validate::check_title(title)?;
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
  use std::collections::{HashMap, HashSet};

  // Which ids exist in this set (to detect orphaned parent_id references).
  let known: HashSet<uuid::Uuid> = flat.iter().map(|c| c.id).collect();

  // Map every id to its direct children, so a parent can be reconstructed with
  // its full subtree in one pass regardless of input order. Building the tree
  // bottom-up from this avoids the stale-clone bug where a parent already
  // copied into `roots` never saw children attached to it afterwards.
  let mut children_of: HashMap<uuid::Uuid, Vec<uuid::Uuid>> = HashMap::new();
  let by_id: HashMap<uuid::Uuid, &Chapter> = flat.iter().map(|c| (c.id, c)).collect();

  let mut root_ids: Vec<uuid::Uuid> = Vec::new();
  for c in &flat {
    match c.parent_id {
      // A chapter is a root if it has no parent, or its parent is not in this
      // set (orphaned reference must not vanish).
      Some(pid) if known.contains(&pid) => {
        children_of.entry(pid).or_default().push(c.id);
      }
      _ => root_ids.push(c.id),
    }
  }

  fn assemble(
    id: uuid::Uuid,
    by_id: &HashMap<uuid::Uuid, &Chapter>,
    children_of: &HashMap<uuid::Uuid, Vec<uuid::Uuid>>,
  ) -> Chapter {
    let base = by_id[&id];
    let children = children_of
      .get(&id)
      .map(|ids| {
        ids
          .iter()
          .map(|cid| assemble(*cid, by_id, children_of))
          .collect()
      })
      .unwrap_or_default();
    Chapter {
      children,
      ..(*base).clone()
    }
  }

  let mut roots: Vec<Chapter> = root_ids
    .iter()
    .map(|id| assemble(*id, &by_id, &children_of))
    .collect();
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
#[cfg(test)]
mod tests {
  use super::*;

  fn chapter(id: u128, seq: f64, parent: Option<u128>) -> Chapter {
    Chapter {
      id: uuid::Uuid::from_u128(id),
      book_id: uuid::Uuid::from_u128(999),
      title: format!("Chapter {seq}"),
      sequence_number: seq,
      parent_id: parent.map(uuid::Uuid::from_u128),
      children: Vec::new(),
    }
  }

  #[test]
  fn roots_sorted_by_sequence_number() {
    let flat = vec![
      chapter(3, 3.0, None),
      chapter(1, 1.0, None),
      chapter(2, 2.0, None),
    ];
    let tree = build_chapter_tree(flat);
    let seqs: Vec<f64> = tree.iter().map(|c| c.sequence_number).collect();
    assert_eq!(seqs, vec![1.0, 2.0, 3.0]);
  }

  #[test]
  fn decimal_sequence_numbers_sort_correctly() {
    // 1.1 must sort before 1.2 before 2.0 — the reason sequence_number is a
    // decimal rather than an integer.
    let flat = vec![
      chapter(1, 2.0, None),
      chapter(2, 1.2, None),
      chapter(3, 1.1, None),
    ];
    let tree = build_chapter_tree(flat);
    let seqs: Vec<f64> = tree.iter().map(|c| c.sequence_number).collect();
    assert_eq!(seqs, vec![1.1, 1.2, 2.0]);
  }

  #[test]
  fn children_nest_under_parent() {
    let flat = vec![
      chapter(1, 1.0, None),
      chapter(2, 1.2, Some(1)),
      chapter(3, 1.1, Some(1)),
    ];
    let tree = build_chapter_tree(flat);
    assert_eq!(tree.len(), 1, "only the parent should be a root");
    let parent = &tree[0];
    assert_eq!(parent.children.len(), 2, "both children should nest");
    let child_seqs: Vec<f64> = parent.children.iter().map(|c| c.sequence_number).collect();
    assert_eq!(child_seqs, vec![1.1, 1.2], "children sorted by sequence");
  }

  #[test]
  fn orphaned_parent_id_falls_back_to_root() {
    // A chapter whose parent_id points at a chapter not in the set must not
    // be dropped — it becomes a root rather than vanishing.
    let flat = vec![chapter(2, 1.0, Some(404))];
    let tree = build_chapter_tree(flat);
    assert_eq!(tree.len(), 1, "orphan must survive as a root");
    assert_eq!(tree[0].id, uuid::Uuid::from_u128(2));
  }

  #[test]
  fn empty_input_yields_empty_tree() {
    assert!(build_chapter_tree(Vec::new()).is_empty());
  }

  #[test]
  fn flatten_is_inverse_of_build_for_sorted_input() {
    let flat = vec![
      chapter(1, 1.0, None),
      chapter(2, 1.1, Some(1)),
      chapter(3, 2.0, None),
    ];
    let tree = build_chapter_tree(flat);
    let flattened = flatten_tree(&tree);
    // Depth-first: parent, its child, then next root.
    let ids: Vec<u128> = flattened.iter().map(|c| c.id.as_u128()).collect();
    assert_eq!(ids, vec![1, 2, 3]);
  }
}
