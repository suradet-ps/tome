//! Spaced-repetition scheduling (SM-2).
//!
//! This is the pure, side-effect-free core of flashcard review: given a card's
//! current schedule and the quality of a recall, compute the next schedule.
//! It was extracted out of `views/review_view.rs` so the arithmetic that
//! decides *when a reader sees a card again* can be tested without a DOM or a
//! network round trip — it is one of the two pieces (with markdown
//! sanitization) a silent regression hurts most.
//!
//! The algorithm is the classic SM-2 (`SuperMemo` 2):
//!   `EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))`,  clamped to >= 1.3
//! with the interval reset to 1 on a failed recall (q < 3), seeded to 1 then 6
//! on the first two successful reviews, and multiplied by `EF'` thereafter.

/// The lowest an ease factor may fall. Below this, intervals collapse toward
/// daily review; SM-2 fixes the floor at 1.3.
pub const MIN_EASE_FACTOR: f64 = 1.3;

/// A card's schedule state, independent of storage or rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Schedule {
  /// Days until the next review.
  pub interval_days: i32,
  /// `SuperMemo` ease factor (>= [`MIN_EASE_FACTOR`]).
  pub ease_factor: f64,
}

/// Compute the next [`Schedule`] from the current one and a recall `quality`
/// in `0..=5` (values outside the range are clamped).
///
/// `quality < 3` is treated as a failed recall: the interval resets to 1 day.
/// The ease factor is always updated and clamped to [`MIN_EASE_FACTOR`].
#[must_use]
pub fn schedule_next(current: Schedule, quality: i32) -> Schedule {
  let q = f64::from(quality.clamp(0, 5));
  let diff = 5.0 - q;
  let delta = diff.mul_add(-diff.mul_add(0.02, 0.08), 0.1);
  let ease_factor = (current.ease_factor + delta).max(MIN_EASE_FACTOR);

  let interval_days = if quality < 3 || current.interval_days == 0 {
    1
  } else if current.interval_days == 1 {
    6
  } else {
    (f64::from(current.interval_days) * ease_factor).round() as i32
  };

  Schedule {
    interval_days,
    ease_factor,
  }
}

/// Remove the card with `id` from a review queue in place.
///
/// Extracted from the review view so the queue invariant — grading the last
/// due card empties the queue rather than panicking on an out-of-range access —
/// can be tested without a DOM. Returns `true` when a card was removed.
pub fn remove_card<T: HasId>(queue: &mut Vec<T>, id: uuid::Uuid) -> bool {
  let before = queue.len();
  queue.retain(|card| card.id() != id);
  queue.len() != before
}

/// A queue item identified by a UUID. Implemented for the row type in
/// production and for a lightweight stand-in in tests.
pub trait HasId {
  /// The item's unique identifier.
  fn id(&self) -> uuid::Uuid;
}

impl HasId for crate::core::types::Flashcard {
  fn id(&self) -> uuid::Uuid {
    self.id
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
  }

  #[test]
  fn failed_recall_resets_interval() {
    let next = schedule_next(
      Schedule {
        interval_days: 30,
        ease_factor: 2.5,
      },
      2,
    );
    assert_eq!(
      next.interval_days, 1,
      "failed recall must reset interval to 1"
    );
  }

  #[test]
  fn first_success_seeds_one_day() {
    let next = schedule_next(
      Schedule {
        interval_days: 0,
        ease_factor: 2.5,
      },
      5,
    );
    assert_eq!(next.interval_days, 1);
  }

  #[test]
  fn second_success_seeds_six_days() {
    let next = schedule_next(
      Schedule {
        interval_days: 1,
        ease_factor: 2.5,
      },
      4,
    );
    assert_eq!(next.interval_days, 6);
  }

  #[test]
  fn subsequent_success_multiplies_by_ease() {
    // interval 6, perfect recall keeps EF at 2.6 -> 6 * 2.6 = 15.6 -> 16
    let next = schedule_next(
      Schedule {
        interval_days: 6,
        ease_factor: 2.5,
      },
      5,
    );
    // EF' = 2.5 + (0.1 - 0*..) = 2.6, interval = round(6 * 2.6) = 16
    assert!(
      approx(next.ease_factor, 2.6),
      "ease was {}",
      next.ease_factor
    );
    assert_eq!(next.interval_days, 16);
  }

  #[test]
  fn ease_factor_never_below_floor() {
    // Repeated poor recalls must not drive EF below 1.3.
    let mut sched = Schedule {
      interval_days: 10,
      ease_factor: 1.3,
    };
    for _ in 0..20 {
      sched = schedule_next(sched, 0);
      assert!(
        sched.ease_factor >= MIN_EASE_FACTOR,
        "ease dropped below floor: {}",
        sched.ease_factor
      );
    }
  }

  #[test]
  fn perfect_recall_raises_ease() {
    let next = schedule_next(
      Schedule {
        interval_days: 6,
        ease_factor: 2.5,
      },
      5,
    );
    assert!(next.ease_factor > 2.5, "perfect recall should raise ease");
  }

  #[test]
  fn quality_three_holds_ease_roughly_steady() {
    // q=3: delta = 0.1 - 2*(0.08 + 2*0.02) = 0.1 - 2*0.12 = -0.14
    let next = schedule_next(
      Schedule {
        interval_days: 6,
        ease_factor: 2.5,
      },
      3,
    );
    assert!(
      approx(next.ease_factor, 2.36),
      "ease was {}",
      next.ease_factor
    );
  }

  #[test]
  fn out_of_range_quality_is_clamped() {
    let hi = schedule_next(
      Schedule {
        interval_days: 6,
        ease_factor: 2.5,
      },
      99,
    );
    let five = schedule_next(
      Schedule {
        interval_days: 6,
        ease_factor: 2.5,
      },
      5,
    );
    assert_eq!(hi, five, "quality above 5 should clamp to 5");
  }

  #[derive(Debug, Clone)]
  struct QueueCard {
    id: uuid::Uuid,
  }

  impl HasId for QueueCard {
    fn id(&self) -> uuid::Uuid {
      self.id
    }
  }

  #[test]
  fn grading_last_card_empties_queue_without_panic() {
    let only = uuid::Uuid::new_v4();
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
    let first = uuid::Uuid::new_v4();
    let second = uuid::Uuid::new_v4();
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
    let present = uuid::Uuid::new_v4();
    let mut queue = vec![QueueCard { id: present }];

    let removed = remove_card(&mut queue, uuid::Uuid::new_v4());

    assert!(!removed, "removing a card not in the queue changes nothing");
    assert_eq!(queue.len(), 1);
  }
}
