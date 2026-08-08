//! Review-session helpers for the flashcard queue.
//!
//! The pure SM-2 scheduling math (`Schedule`, `schedule_next`,
//! `MIN_EASE_FACTOR`) now lives in the external [`srs-sm2`] crate, which tome
//! depends on. This module keeps only the pieces that are specific to the
//! application: the calm review-header copy, and the queue maintenance that
//! removes a graded card without panicking on an empty queue.

use crate::core::types::Flashcard;
use chrono::{DateTime, Duration, Utc};
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

/// Compute the next schedule for a graded card and return the full updated
/// row.
///
/// Pure — the clock is passed in explicitly so the grading path can be
/// tested without a DOM or a clock. The updated row is what gets written
/// both to Supabase and (offline) to the local cache and the sync queue.
#[must_use]
pub fn schedule_card(card: &Flashcard, quality: i32, now: DateTime<Utc>) -> Flashcard {
  let scheduled = srs_sm2::schedule_next(
    srs_sm2::Schedule {
      interval_days: card.interval_days,
      ease_factor: card.ease_factor,
    },
    quality,
  );
  Flashcard {
    id: card.id,
    user_id: card.user_id,
    chapter_id: card.chapter_id,
    front: card.front.clone(),
    back: card.back.clone(),
    next_review: now + Duration::days(i64::from(scheduled.interval_days)),
    interval_days: scheduled.interval_days,
    ease_factor: scheduled.ease_factor,
    created_at: card.created_at,
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

  fn card(id: Uuid, interval_days: i32, ease_factor: f64) -> Flashcard {
    Flashcard {
      id,
      user_id: Uuid::new_v4(),
      chapter_id: None,
      front: "front".into(),
      back: "back".into(),
      next_review: DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap(),
      interval_days,
      ease_factor,
      created_at: DateTime::<Utc>::from_timestamp(1_690_000_000, 0).unwrap(),
    }
  }

  fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
  }

  #[test]
  fn perfect_recall_seeds_first_interval_and_raises_ease() {
    let now = Utc::now();
    let c = card(Uuid::new_v4(), 0, 2.5);

    let next = schedule_card(&c, 5, now);

    assert_eq!(next.id, c.id, "identity fields are preserved");
    assert_eq!(next.user_id, c.user_id);
    assert_eq!(next.front, c.front);
    assert_eq!(next.back, c.back);
    assert_eq!(next.created_at, c.created_at);
    assert_eq!(
      next.interval_days, 1,
      "first success seeds a 1-day interval"
    );
    assert!(approx(next.ease_factor, 2.6), "q=5: 2.5 + 0.1");
    assert_eq!(next.next_review, now + Duration::days(1));
  }

  #[test]
  fn failed_recall_resets_interval_and_drops_ease() {
    let now = Utc::now();
    let c = card(Uuid::new_v4(), 30, 2.5);

    let next = schedule_card(&c, 1, now);

    assert_eq!(next.interval_days, 1, "failure resets to tomorrow");
    assert!(approx(next.ease_factor, 1.96), "q=1: 2.5 - 0.54");
    assert_eq!(next.next_review, now + Duration::days(1));
  }

  #[test]
  fn streak_multiplies_interval_by_ease() {
    let now = Utc::now();
    let first = schedule_card(&card(Uuid::new_v4(), 0, 2.5), 5, now);
    let second = schedule_card(&first, 4, now);
    let third = schedule_card(&second, 5, now);

    assert_eq!(first.interval_days, 1);
    assert_eq!(second.interval_days, 6, "second success seeds 6 days");
    assert_eq!(third.interval_days, 16, "6 * 2.7 rounded");
    assert!(approx(third.ease_factor, 2.7), "2.6 + 0.1 after q=5");
    assert_eq!(third.next_review, now + Duration::days(16));
  }

  #[test]
  fn ease_never_below_floor() {
    let now = Utc::now();
    let mut c = card(Uuid::new_v4(), 10, 1.3);
    for _ in 0..10 {
      c = schedule_card(&c, 0, now);
    }
    assert!(c.ease_factor >= 1.3);
  }
}
