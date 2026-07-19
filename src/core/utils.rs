//! Misc small helpers.

use std::time::Duration;

/// Truncate a string to at most `max_chars` characters, appending `…` when
/// the truncation actually happens.
#[must_use]
pub fn truncate(value: &str, max_chars: usize) -> String {
  if value.chars().count() <= max_chars {
    return value.to_string();
  }
  let truncated: String = value.chars().take(max_chars).collect();
  format!("{truncated}…")
}

/// Format a number of seconds as `Hh Mm`.
#[must_use]
pub fn format_duration_human(seconds: i32) -> String {
  if seconds <= 0 {
    return "0m".to_string();
  }
  let total = seconds as i64;
  let hours = total / 3600;
  let minutes = (total % 3600) / 60;
  if hours == 0 {
    format!("{minutes}m")
  } else if minutes == 0 {
    format!("{hours}h")
  } else {
    format!("{hours}h {minutes}m")
  }
}

/// Format a number of seconds as `MM:SS`.
#[must_use]
pub fn format_clock(total_seconds: i64) -> String {
  let minutes = (total_seconds / 60).to_string();
  let remaining_seconds = (total_seconds.rem_euclid(60)).to_string();
  format!(
    "{minutes:0>2}:{remaining_seconds:0>2}",
    minutes = format!("{:0>2}", minutes),
    remaining_seconds = format!("{:0>2}", remaining_seconds)
  )
}

/// Convert a [`Duration`] to total seconds (rounded down).
#[must_use]
pub const fn duration_to_seconds(duration: Duration) -> i64 {
  duration.as_secs() as i64
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn truncate_leaves_short_strings_untouched() {
    assert_eq!(truncate("hello", 10), "hello");
    assert_eq!(truncate("hello", 5), "hello");
  }

  #[test]
  fn truncate_appends_ellipsis_when_cutting() {
    assert_eq!(truncate("hello world", 5), "hello…");
  }

  #[test]
  fn truncate_counts_chars_not_bytes() {
    // Multi-byte input must be cut on a char boundary, not a byte offset.
    assert_eq!(truncate("日本語テスト", 3), "日本語…");
  }

  #[test]
  fn duration_human_zero_and_negative() {
    assert_eq!(format_duration_human(0), "0m");
    assert_eq!(format_duration_human(-10), "0m");
  }

  #[test]
  fn duration_human_boundaries() {
    assert_eq!(format_duration_human(59), "0m");
    assert_eq!(format_duration_human(60), "1m");
    assert_eq!(format_duration_human(3599), "59m");
    assert_eq!(format_duration_human(3600), "1h");
    assert_eq!(format_duration_human(3660), "1h 1m");
    assert_eq!(format_duration_human(7325), "2h 2m");
  }

  #[test]
  fn clock_pads_to_two_digits() {
    assert_eq!(format_clock(0), "00:00");
    assert_eq!(format_clock(5), "00:05");
    assert_eq!(format_clock(65), "01:05");
    assert_eq!(format_clock(600), "10:00");
    assert_eq!(format_clock(1500), "25:00");
  }

  #[test]
  fn duration_to_seconds_floors() {
    assert_eq!(duration_to_seconds(Duration::from_millis(1999)), 1);
    assert_eq!(duration_to_seconds(Duration::from_secs(90)), 90);
  }
}
