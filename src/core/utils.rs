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
pub fn duration_to_seconds(duration: Duration) -> i64 {
    duration.as_secs() as i64
}
