//! Time helpers (WASM-compatible).

use chrono::{DateTime, Utc};

/// Returns the current UTC time as an ISO 8601 string with `Z` suffix.
#[must_use]
pub fn now_iso() -> String {
  Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Convert a `DateTime<Utc>` into an ISO 8601 string with `Z` suffix.
#[must_use]
pub fn to_iso(date: DateTime<Utc>) -> String {
  date.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Parse an ISO 8601 string into `DateTime<Utc>`. Returns `Utc::now()` on
/// failure to keep callers robust.
#[must_use]
pub fn parse_iso(value: &str) -> DateTime<Utc> {
  DateTime::parse_from_rfc3339(value).map_or_else(|_| Utc::now(), |d| d.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn iso_format_has_z_suffix_and_millis() {
    let dt = DateTime::parse_from_rfc3339("2024-03-15T08:30:45.123Z")
      .unwrap()
      .with_timezone(&Utc);
    assert_eq!(to_iso(dt), "2024-03-15T08:30:45.123Z");
  }

  #[test]
  fn iso_round_trips() {
    let dt = DateTime::parse_from_rfc3339("2024-12-31T23:59:59.000Z")
      .unwrap()
      .with_timezone(&Utc);
    let s = to_iso(dt);
    let parsed = parse_iso(&s);
    // Round-trip preserves the instant to millisecond precision.
    assert_eq!(parsed.timestamp_millis(), dt.timestamp_millis());
  }

  #[test]
  fn parse_accepts_offset_and_normalizes_to_utc() {
    let parsed = parse_iso("2024-01-01T12:00:00+02:00");
    // 12:00 at +02:00 is 10:00 UTC.
    assert_eq!(to_iso(parsed), "2024-01-01T10:00:00.000Z");
  }
}
