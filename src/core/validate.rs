//! Pure input validation shared by the stores and enforced again in the
//! database. These caps exist in two places on purpose: the client rejects
//! oversized input early (a clear error instead of a silent truncation or a
//! server round-trip), and matching `check` constraints in the schema
//! (`db/supabase-schema.sql`) make the cap a hard guarantee even if a request
//! bypasses the client. Keeping the logic here — pure and unit-tested — means
//! the two sides can't drift on how they count length.
//!
//! Length is measured in Unicode scalar values (`chars().count()`), matching
//! Postgres `char_length()`, so a multi-byte string is counted the same way on
//! both sides rather than by UTF-8 byte length.

use crate::core::error::{AppError, AppResult};

/// Maximum note content length, in characters.
pub const MAX_NOTE_LENGTH: usize = 200_000;

/// Maximum book/chapter title length, in characters.
pub const MAX_TITLE: usize = 200;

/// Maximum author name length, in characters.
pub const MAX_AUTHOR: usize = 200;

/// Reject a note whose content exceeds [`MAX_NOTE_LENGTH`] characters.
///
/// # Errors
/// Returns [`AppError::Other`] when the content is too long.
pub fn check_note_content(content: &str) -> AppResult<()> {
  check_len(content, MAX_NOTE_LENGTH, "Note")
}

/// Reject a title that is empty (after trimming) or exceeds [`MAX_TITLE`]
/// characters. Returns the trimmed title on success.
///
/// # Errors
/// Returns [`AppError::Other`] when the trimmed title is empty or too long.
pub fn check_title(title: &str) -> AppResult<&str> {
  check_required(title, MAX_TITLE, "Title")
}

/// Reject an author name longer than [`MAX_AUTHOR`] characters. An empty
/// author is allowed (books may have no author). Returns the trimmed value.
///
/// # Errors
/// Returns [`AppError::Other`] when the trimmed author is too long.
pub fn check_author(author: &str) -> AppResult<&str> {
  let trimmed = author.trim();
  check_len(trimmed, MAX_AUTHOR, "Author")?;
  Ok(trimmed)
}

fn check_required<'a>(value: &'a str, max: usize, label: &str) -> AppResult<&'a str> {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return Err(AppError::other(format!("{label} cannot be empty.")));
  }
  check_len(trimmed, max, label)?;
  Ok(trimmed)
}

fn check_len(value: &str, max: usize, label: &str) -> AppResult<()> {
  let len = value.chars().count();
  if len > max {
    return Err(AppError::other(format!(
      "{label} exceeds the maximum length of {max} characters ({len} given)."
    )));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn note_at_the_limit_is_accepted() {
    let content: String = "a".repeat(MAX_NOTE_LENGTH);
    assert!(check_note_content(&content).is_ok());
  }

  #[test]
  fn note_over_the_limit_is_rejected() {
    let content: String = "a".repeat(MAX_NOTE_LENGTH + 1);
    assert!(check_note_content(&content).is_err());
  }

  #[test]
  fn note_length_counts_characters_not_bytes() {
    // Each 'é' is 2 UTF-8 bytes; MAX_NOTE_LENGTH of them is 2x the byte cap
    // but exactly the character cap, so it must be accepted. The old
    // byte-based check would have wrongly rejected this.
    let content: String = "é".repeat(MAX_NOTE_LENGTH);
    assert_eq!(content.len(), MAX_NOTE_LENGTH * 2);
    assert!(
      check_note_content(&content).is_ok(),
      "length must be measured in characters, not bytes"
    );
  }

  #[test]
  fn note_over_limit_in_characters_is_rejected_even_if_multibyte() {
    let content: String = "é".repeat(MAX_NOTE_LENGTH + 1);
    assert!(check_note_content(&content).is_err());
  }

  #[test]
  fn empty_title_is_rejected() {
    assert!(check_title("   ").is_err());
    assert!(check_title("").is_err());
  }

  #[test]
  fn title_is_trimmed_on_success() {
    assert_eq!(check_title("  Rust Book  ").unwrap(), "Rust Book");
  }

  #[test]
  fn title_at_the_limit_is_accepted() {
    let title: String = "t".repeat(MAX_TITLE);
    assert!(check_title(&title).is_ok());
  }

  #[test]
  fn title_over_the_limit_is_rejected_not_truncated() {
    let title: String = "t".repeat(MAX_TITLE + 1);
    assert!(
      check_title(&title).is_err(),
      "an over-long title is rejected rather than silently truncated"
    );
  }

  #[test]
  fn empty_author_is_allowed() {
    assert_eq!(check_author("   ").unwrap(), "");
  }

  #[test]
  fn author_over_the_limit_is_rejected() {
    let author: String = "a".repeat(MAX_AUTHOR + 1);
    assert!(check_author(&author).is_err());
  }
}
