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
  /// The most recently opened book + chapter, captured as a lightweight
  /// snapshot so the dashboard can offer to resume it without re-fetching
  /// that book's chapters. Surfaced as a calm "continue reading" point.
  pub last_opened: RwSignal<Option<LastOpened>>,
  pub loading: RwSignal<bool>,
  pub error: RwSignal<Option<String>>,
}

/// A snapshot of the last chapter a reader opened, enough to label and
/// navigate back to it from the dashboard.
#[derive(Debug, Clone)]
pub struct LastOpened {
  /// The opened book's id.
  pub book_id: uuid::Uuid,
  /// The opened book's title (snapshotted).
  pub book_title: String,
  /// The opened chapter's id.
  pub chapter_id: uuid::Uuid,
  /// The opened chapter's title (snapshotted).
  pub chapter_title: String,
  /// The opened chapter's sequence number (snapshotted).
  pub chapter_seq: f64,
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
      last_opened: RwSignal::new(None),
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

  /// Record that `chapter` of `book_id` was just opened, so the dashboard
  /// can offer to resume it. The book/chapter titles are snapshotted so
  /// the dashboard can label the entry point without re-fetching chapters.
  pub fn mark_opened(&self, book_id: uuid::Uuid, book_title: &str, chapter: &Chapter) {
    self.last_opened.set(Some(LastOpened {
      book_id,
      book_title: book_title.to_string(),
      chapter_id: chapter.id,
      chapter_title: chapter.title.clone(),
      chapter_seq: chapter.sequence_number,
    }));
  }

  /// Insert many chapters from a pasted table of contents in a single batch.
  ///
  /// The `inserts` come from [`toc_to_inserts`] (already parsed and validated);
  /// each tuple is `(title, sequence_number, parent_index)` where `parent_index`
  /// is an index into `inserts` (resolved to the chapter id we just inserted).
  /// Returns the number of chapters actually inserted (skips any whose title
  /// fails validation, and never inserts zero rows). The chapter tree is
  /// refreshed afterwards so the UI reflects the new structure immediately.
  pub async fn add_chapters_bulk(
    &self,
    bid: uuid::Uuid,
    inserts: &[(String, f64, Option<usize>)],
  ) -> AppResult<u32> {
    if inserts.is_empty() {
      return Ok(0);
    }
    let a = use_auth();
    if a.user.get_untracked().is_none() {
      return Ok(0);
    }
    let mut resolved_parents: Vec<Option<uuid::Uuid>> = Vec::with_capacity(inserts.len());
    let mut new_ids: Vec<uuid::Uuid> = Vec::with_capacity(inserts.len());
    let mut inserted: u32 = 0;
    let c = supabase::supabase()?;
    for (title, seq, parent_idx) in inserts {
      let t = match validate::check_title(title) {
        Ok(t) => t,
        Err(_) => continue, // skip titles that violate length rules
      };
      let pid = parent_idx.and_then(|i| new_ids.get(i).copied());
      let body = serde_json::json!({
        "book_id": bid,
        "title": t,
        "sequence_number": seq,
        "parent_id": pid,
      });
      match c
        .postgrest()
        .from("reading_chapters")
        .insert_one::<Chapter, _>(&body)
        .await
      {
        Ok(ch) => {
          new_ids.push(ch.id);
          resolved_parents.push(pid);
          inserted += 1;
        }
        Err(e) => return Err(e),
      }
    }
    if inserted > 0 {
      self.fetch_chapters(bid).await?;
    }
    Ok(inserted)
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
/// A chapter parsed from a pasted table of contents.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedChapter {
  /// Nest depth (0 for top level), derived from leading markers.
  pub depth: u32,
  /// Display title (whitespace-trimmed).
  pub title: String,
}

/// Parse a pasted table of contents into a flat list of [`ParsedChapter`]s.
///
/// Each non-empty line becomes one chapter. Depth is inferred, in priority order:
/// 1. Leading markdown `#`/`##`… markers (`#` = 0, `##` = 1, …).
/// 2. Otherwise, leading indentation (tabs or 2+ spaces) — every 2 spaces or
///    single tab steps one level down.
///
/// Titles are trimmed; a leading bullet (`-`, `*`, `+`), a trailing sequence
/// number (` 1.` / ` (3)`), or a page number (` … 42` / ` … p.42`) is
/// stripped so the title carries only the chapter name. Blank lines and lines that
/// are purely punctuation/whitespace are skipped. Pure so the parser — the heart
/// of frictionless bulk chapter capture — can be tested without a network call.
#[must_use]
pub fn parse_toc(input: &str) -> Vec<ParsedChapter> {
  let mut out = Vec::new();
  for raw in input.lines() {
    let line = raw.trim();
    if line.is_empty() {
      continue;
    }

    // 1) Markdown heading markers set the depth outright.
    if let Some(rest) = line.strip_prefix('#') {
      let mut hashes: i32 = 1;
      let mut chars = rest.chars();
      while chars.next() == Some('#') {
        hashes += 1;
      }
      let title = chars.as_str().trim();
      if title.is_empty() {
        continue;
      }
      let depth = u32::try_from(hashes.saturating_sub(1)).unwrap_or(0).min(5);
      out.push(ParsedChapter {
        depth,
        title: clean_title(title).to_string(),
      });
      continue;
    }

    // 2) Indentation-based depth.
    let indent = raw.len() - raw.trim_start().len();
    let depth = if raw.contains('\t') && indent > 0 {
      // A line with a tab: count leading tabs.
      raw.chars().take_while(|c| *c == '\t').count() as u32
    } else {
      (indent / 2) as u32
    };

    let mut title = line;
    // Drop a leading bullet if present.
    let stripped = title
      .strip_prefix("- ")
      .or_else(|| title.strip_prefix("* "))
      .or_else(|| title.strip_prefix("+ "));
    if let Some(s) = stripped {
      title = s;
    }
    let cleaned = clean_title(title);
    if cleaned.is_empty() {
      continue;
    }
    out.push(ParsedChapter {
      depth: depth.min(5),
      title: cleaned.to_string(),
    });
  }
  out
}

/// Strip trailing sequence numbers and page numbers from a chapter title
/// candidate, returning the trimmed title slice.
///
/// Handles: a trailing ` 12.`, ` (3)`, a page number ` 42` / ` p.42` /
/// ` pg 12` / a dotted leader ` ........ 12`, and internal runs of
/// whitespace. Only whitespace-delimited trailing tokens that look like a
/// marker (digits, an optional `p`/`pg` prefix, leader dots, or a
/// `(n)`) are removed, so a title like `C++ 11` is left alone.
fn clean_title(title: &str) -> &str {
  let mut t = title.trim();
  // Peel trailing whitespace-delimited tokens while they read as a marker.
  loop {
    let Some(split) = t.rfind(' ') else {
      break;
    };
    let token = &t[split + 1..];
    if token.is_empty() || is_marker_token(token) {
      t = t[..split].trim_end();
    } else {
      break;
    }
  }
  t
}

/// Whether `token` (a whitespace-delimited trailing piece of a title) is a
/// page/sequence marker: a `(n)` parenthetical, a run of leader dots, or an
/// optional `p`/`pg` prefix followed by dots and/or digits.
fn is_marker_token(token: &str) -> bool {
  if token.starts_with('(') && token.ends_with(')') {
    let inner = &token[1..token.len() - 1];
    return !inner.is_empty() && inner.chars().all(|c| c.is_ascii_digit());
  }
  if token.chars().all(|c| c == '.') {
    return true; // dotted leader like "........"
  }
  let after_prefix = token
    .strip_prefix("pg")
    .or_else(|| token.strip_prefix('p'))
    .unwrap_or(token);
  // A bare `p`/`pg` page prefix (no number on the same token) is a marker.
  if after_prefix.is_empty() {
    return true;
  }
  let body = after_prefix.trim_start_matches(['.', ' ']);
  !body.is_empty() && body.chars().all(|c| c.is_ascii_digit())
}

/// Turn a flat, depth-tagged TOC into `(title, sequence_number, parent_index)`
/// tuples ready to insert. Sequence numbers are assigned per-level as `1`, `2`, …
/// with dotted nesting (`1.1`, `1.2`, `2.1`) so they match the schema's
/// decimal `sequence_number`. Returns `None` for any chapter whose title fails
/// validation (kept out of the insertion batch). Pure so the mapping can be
/// tested without touching Supabase.
#[must_use]
pub fn toc_to_inserts(parsed: &[ParsedChapter]) -> Vec<(String, f64, Option<usize>)> {
  let mut counters: [u32; 6] = [0; 6];
  let mut parents: [Option<usize>; 6] = [None; 6];
  let mut out = Vec::new();
  for (idx, ch) in parsed.iter().enumerate() {
    let level = ch.depth.min(5) as usize;
    counters[level] += 1;
    // Reset deeper counters and set this level's parent.
    for d in (level + 1)..=5 {
      counters[d] = 0;
    }
    parents[level] = if level == 0 { None } else { parents[level - 1] };
    let mut seq = String::new();
    for d in 0..=level {
      if d > 0 {
        seq.push('.');
      }
      seq.push_str(&counters[d].to_string());
    }
    let Ok(seq_num) = seq.parse::<f64>() else {
      continue;
    };
    let title = ch.title.trim().to_string();
    out.push((title, seq_num, parents[level]));
    parents[level] = Some(idx);
  }
  out
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

  // --- TOC bulk import parser ----------------------------------------

  #[test]
  fn parses_plain_lines_one_per_chapter() {
    let toc = "Getting Started\nOwnership\nTraits and Generics";
    let parsed = parse_toc(toc);
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0].depth, 0);
    assert_eq!(parsed[0].title, "Getting Started");
    assert_eq!(parsed[2].title, "Traits and Generics");
  }

  #[test]
  fn parses_markdown_heading_depths() {
    let toc = "# Introduction\n## Background\n### Details\n## More";
    let parsed = parse_toc(toc);
    assert_eq!(
      parsed.iter().map(|c| c.depth).collect::<Vec<_>>(),
      vec![0, 1, 2, 1]
    );
    assert_eq!(parsed[1].title, "Background");
    assert_eq!(parsed[3].title, "More");
  }

  #[test]
  fn parses_indentation_depth() {
    let toc = "Top\n  Child A\n    Grandchild\n  Child B";
    let parsed = parse_toc(toc);
    assert_eq!(
      parsed.iter().map(|c| c.depth).collect::<Vec<_>>(),
      vec![0, 1, 2, 1]
    );
  }

  #[test]
  fn parses_tab_indentation() {
    let toc = "Top\n\tChild\n\t\tGrandchild";
    let parsed = parse_toc(toc);
    assert_eq!(
      parsed.iter().map(|c| c.depth).collect::<Vec<_>>(),
      vec![0, 1, 2]
    );
  }

  #[test]
  fn strips_bullets_and_trailing_numbers() {
    let toc = "- First Chapter 1\n* Second 2\n+ Third (3)";
    let parsed = parse_toc(toc);
    assert_eq!(parsed[0].title, "First Chapter");
    assert_eq!(parsed[1].title, "Second");
    assert_eq!(parsed[2].title, "Third");
  }

  #[test]
  fn strips_page_numbers() {
    let toc = "Introduction ........ 12\nConclusion  p.42\nIndex  pg 9";
    let parsed = parse_toc(toc);
    assert_eq!(parsed[0].title, "Introduction");
    assert_eq!(parsed[1].title, "Conclusion");
    assert_eq!(parsed[2].title, "Index");
  }

  #[test]
  fn skips_blank_lines() {
    let toc = "One\n\n\nTwo\n   \nThree";
    let parsed = parse_toc(toc);
    assert_eq!(parsed.len(), 3);
  }

  #[test]
  fn toc_to_inserts_assigns_dotted_sequences_and_parents() {
    let parsed = vec![
      ParsedChapter {
        depth: 0,
        title: "Intro".into(),
      },
      ParsedChapter {
        depth: 1,
        title: "Setup".into(),
      },
      ParsedChapter {
        depth: 1,
        title: "Run".into(),
      },
      ParsedChapter {
        depth: 0,
        title: "Deep".into(),
      },
      ParsedChapter {
        depth: 1,
        title: "Sub".into(),
      },
    ];
    let inserts = toc_to_inserts(&parsed);
    assert_eq!(inserts.len(), 5);
    // (title, sequence, parent_index)
    assert_eq!(inserts[0], ("Intro".to_string(), 1.0, None));
    assert_eq!(inserts[1], ("Setup".to_string(), 1.1, Some(0)));
    assert_eq!(inserts[2], ("Run".to_string(), 1.2, Some(0)));
    assert_eq!(inserts[3], ("Deep".to_string(), 2.0, None));
    assert_eq!(inserts[4], ("Sub".to_string(), 2.1, Some(3)));
  }

  #[test]
  fn toc_to_inserts_resets_counters_per_level() {
    let parsed = vec![
      ParsedChapter {
        depth: 0,
        title: "A".into(),
      },
      ParsedChapter {
        depth: 0,
        title: "B".into(),
      },
      ParsedChapter {
        depth: 1,
        title: "B1".into(),
      },
      ParsedChapter {
        depth: 1,
        title: "B2".into(),
      },
    ];
    let inserts = toc_to_inserts(&parsed);
    assert_eq!(inserts[0].1, 1.0);
    assert_eq!(inserts[1].1, 2.0);
    assert_eq!(inserts[2].1, 2.1);
    assert_eq!(inserts[3].1, 2.2);
  }
}
