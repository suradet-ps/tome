//! Review-session helpers for the flashcard queue.
//!
//! The pure SM-2 scheduling math (`Schedule`, `schedule_next`,
//! `MIN_EASE_FACTOR`) now lives in the external [`srs-sm2`] crate, which tome
//! depends on. This module keeps only the pieces that are specific to the
//! application: the calm review-header copy, and the queue maintenance that
//! removes a graded card without panicking on an empty queue.

use crate::core::types::Flashcard;
use uuid::Uuid;

/// Gentle, non-gamified copy for the review session header.
///
/// Tome is a quiet tool: recall is framed as "what's left to look at",
/// never as a streak or a score. Returns the calm phrasing for the
/// current state — all caught up, or how many remain and how many you've
/// reviewed this session. Pure so the wording can be smoke-tested.
#[must_use]
pub fn review_header_copy(due: usize, reviewed: usize) -> &'static str {
  if due == 0 {
    "All caught up"
  } else if reviewed == 0 {
    "A few to look at"
  } else {
    "Still a little to go"
  }
}

/// Remove the card with `id` from a review queue in place.
///
/// Extracted from the review view so the queue invariant — grading the last
/// due card empties the queue rather than panicking on an out-of-range access —
/// can be tested without a DOM. Returns `true` when a card was removed.
pub fn remove_card<T: HasId>(queue: &mut Vec<T>, id: Uuid) -> bool {
  let before = queue.len();
  queue.retain(|card| card.id() != id);
  queue.len() != before
}

/// A queue item identified by a UUID. Implemented for the row type in
/// production and for a lightweight stand-in in tests.
pub trait HasId {
  /// The item's unique identifier.
  fn id(&self) -> Uuid;
}

impl HasId for Flashcard {
  fn id(&self) -> Uuid {
    self.id
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone)]
  struct QueueCard {
    id: Uuid,
  }

  impl HasId for QueueCard {
    fn id(&self) -> Uuid {
      self.id
    }
  }

  #[test]
  fn grading_last_card_empties_queue_without_panic() {
    let only = Uuid::new_v4();
    let mut queue = vec![QueueCard { id: only }];

    let removed = remove_card(&mut queue, only);

    assert!(removed, "the graded card must be removed");
    assert!(
      queue.is_empty(),
      "grading the last due card empties the queue"
    );
    assert!(
      queue.first().is_none(),
      "an empty queue yields no next card instead of panicking"
    );
  }

  #[test]
  fn grading_advances_to_next_card() {
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    let mut queue = vec![QueueCard { id: first }, QueueCard { id: second }];

    let removed = remove_card(&mut queue, first);

    assert!(removed);
    assert_eq!(queue.len(), 1);
    assert_eq!(
      queue.first().map(HasId::id),
      Some(second),
      "the next due card surfaces after grading"
    );
  }

  #[test]
  fn removing_unknown_card_is_a_no_op() {
    let present = Uuid::new_v4();
    let mut queue = vec![QueueCard { id: present }];

    let removed = remove_card(&mut queue, Uuid::new_v4());

    assert!(!removed, "removing a card not in the queue changes nothing");
    assert_eq!(queue.len(), 1);
  }

  #[test]
  fn header_copy_all_caught_up_when_no_due() {
    assert_eq!(review_header_copy(0, 0), "All caught up");
  }

  #[test]
  fn header_copy_a_few_before_first_review() {
    assert_eq!(review_header_copy(3, 0), "A few to look at");
  }

  #[test]
  fn header_copy_still_a_little_after_reviewing() {
    assert_eq!(review_header_copy(2, 4), "Still a little to go");
  }
}
