//! Centralised error type for the application.

use serde::{Deserialize, Serialize};

/// Convenience alias used throughout the codebase.
pub type AppResult<T> = Result<T, AppError>;

/// Application error type. Always renderable to a user-facing message.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum AppError {
  /// The required Supabase environment variables were missing.
  #[error("{message}")]
  Config {
    /// Description of the missing configuration.
    message: String,
  },
  /// The remote API returned an HTTP error.
  #[error("{message}")]
  Http {
    /// HTTP status code.
    status: u16,
    /// Error message returned by the server.
    message: String,
  },
  /// JSON (de)serialisation failed.
  #[error("JSON error: {0}")]
  Json(String),
  /// A network or transport failure occurred.
  #[error("Network error: {0}")]
  Network(String),
  /// The user is not allowed to perform the requested action.
  #[error("Unauthorized")]
  Unauthorized,
  /// Supabase returned `{}` for an operation that should have returned a row.
  #[error("No data returned from server.")]
  NoData,
  /// The row was modified elsewhere since it was loaded (optimistic-concurrency
  /// conflict). The save was refused rather than blindly overwriting the newer
  /// version.
  #[error(
    "This note was changed in another tab or device. Reload to see the latest version before saving."
  )]
  Conflict,
  /// Generic error with a user-facing message.
  #[error("{0}")]
  Other(String),
}

impl AppError {
  /// Construct a [`AppError::Config`] from a static message.
  #[must_use]
  pub fn config(message: impl Into<String>) -> Self {
    Self::Config {
      message: message.into(),
    }
  }

  /// Construct a [`AppError::Http`] from a status code and message.
  #[must_use]
  pub fn http(status: u16, message: impl Into<String>) -> Self {
    Self::Http {
      status,
      message: message.into(),
    }
  }

  /// Construct a [`AppError::Other`] from a string.
  #[must_use]
  pub fn other(message: impl Into<String>) -> Self {
    Self::Other(message.into())
  }

  /// Returns `true` when the error represents a 401/403 from the API.
  #[must_use]
  pub const fn is_unauthorized(&self) -> bool {
    match self {
      Self::Unauthorized => true,
      Self::Http { status, .. } => matches!(*status, 401 | 403),
      _ => false,
    }
  }

  /// Returns `true` when the error is an optimistic-concurrency conflict.
  #[must_use]
  pub const fn is_conflict(&self) -> bool {
    matches!(self, Self::Conflict)
  }
}

impl From<serde_json::Error> for AppError {
  fn from(err: serde_json::Error) -> Self {
    Self::Json(err.to_string())
  }
}

impl From<gloo_net::Error> for AppError {
  fn from(err: gloo_net::Error) -> Self {
    Self::Network(err.to_string())
  }
}

impl From<js_sys::Error> for AppError {
  fn from(err: js_sys::Error) -> Self {
    let message = err.message().as_string().unwrap_or_default();
    Self::Network(message)
  }
}

/// Helper used by `?` chains in async APIs that yield raw `JsValue`.
pub fn js_error_to_app(err: wasm_bindgen::JsValue) -> AppError {
  let message = js_sys::Error::from(err).message();
  let message = message
    .as_string()
    .unwrap_or_else(|| "Unknown error".to_string());
  AppError::Network(message)
}
