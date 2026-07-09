//! Time helpers.

use chrono::{DateTime, Utc};

/// Returns the current UTC time as an ISO 8601 string suitable for PostgREST.
#[must_use]
pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

/// Convert a `DateTime<Utc>` into an ISO 8601 string with `Z` suffix.
#[must_use]
pub fn to_iso(date: DateTime<Utc>) -> String {
    date.to_rfc3339()
}

/// Parse an ISO 8601 string into `DateTime<Utc>`. Returns `Utc::now()` on
/// failure to keep callers robust.
#[must_use]
pub fn parse_iso(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
