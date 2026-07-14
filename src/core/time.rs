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
