//! Supabase client (REST + Auth) used by the application.

use crate::core::auth::SupabaseAuth;
use crate::core::error::{AppError, AppResult};
use crate::core::postgrest::PostgrestClient;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

const URL_STORAGE_KEY: &str = "tome_supabase_url";
const ANON_STORAGE_KEY: &str = "tome_supabase_anon";
const TOKEN_STORAGE_KEY: &str = "tome_supabase_token";

/// Bundles a Supabase URL + anon key and provides typed access to `PostgREST`
/// and the `GoTrue` auth endpoints.
#[derive(Debug, Clone)]
pub struct SupabaseClient {
  url: String,
  anon_key: String,
  token: Option<String>,
}

impl SupabaseClient {
  /// Create a new client. Returns `None` if either value is missing.
  #[must_use]
  pub fn new(url: Option<String>, anon_key: Option<String>) -> Option<Self> {
    let (url, anon_key) = url.zip(anon_key)?;
    if url.is_empty() || anon_key.is_empty() {
      return None;
    }
    Some(Self {
      url,
      anon_key,
      token: None,
    })
  }

  /// Returns a `PostgREST` client configured with the current auth token.
  #[must_use]
  pub fn postgrest(&self) -> PostgrestClient {
    let mut client = PostgrestClient::new(&self.url).with_api_key(&self.anon_key);
    if let Some(token) = &self.token {
      client = client.with_token(token);
    }
    client
  }

  /// Returns a [`SupabaseAuth`] handle for issuing auth requests.
  #[must_use]
  pub fn auth(&self) -> SupabaseAuth<'_> {
    SupabaseAuth::new(&self.url, &self.anon_key, self.token.as_deref())
  }

  /// Read the persisted auth token (if any) from `localStorage`.
  pub fn load_persisted_token() -> Option<String> {
    LocalStorage::get::<String>(TOKEN_STORAGE_KEY).ok()
  }

  /// Persist or clear the auth token in `localStorage`.
  pub fn persist_token(token: Option<&str>) {
    match token {
      Some(value) => {
        if LocalStorage::set(TOKEN_STORAGE_KEY, value).is_err() {
          log::warn!("Failed to persist Supabase auth token");
        }
      }
      None => {
        LocalStorage::delete(TOKEN_STORAGE_KEY);
      }
    }
  }

  /// Replace the current access token.
  pub fn set_token(&mut self, token: Option<String>) {
    self.token = token;
  }

  /// Returns the current access token, if any.
  #[must_use]
  pub fn token(&self) -> Option<&str> {
    self.token.as_deref()
  }

  /// Returns the project URL.
  #[must_use]
  pub fn url(&self) -> &str {
    &self.url
  }

  /// Returns the anon key.
  #[must_use]
  pub fn anon_key(&self) -> &str {
    &self.anon_key
  }
}

// ----------------------------------------------------------------
// Configuration detection
// ----------------------------------------------------------------

thread_local! {
    static CONFIG_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Returns a human-readable error when Supabase is not configured, or
/// `None` when ready.
#[must_use]
pub fn supabase_config_error() -> Option<String> {
  CONFIG_ERROR.with(|cell| {
    let mut borrowed = cell.borrow_mut();
    if borrowed.is_none() {
      let (url, anon) = read_config();
      if url.is_empty() || anon.is_empty() {
        *borrowed = Some(
          "Missing Supabase configuration. Add URL and anon key below or \
                     set SUPABASE_URL / SUPABASE_ANON_KEY in your environment."
            .to_string(),
        );
      } else {
        *borrowed = Some(String::new());
      }
    }
    let value = borrowed.as_ref()?;
    if value.is_empty() {
      None
    } else {
      Some(value.clone())
    }
  })
}

/// Reset the cached error so the next access re-evaluates.
pub fn clear_config_error_cache() {
  CONFIG_ERROR.with(|cell| {
    *cell.borrow_mut() = None;
  });
}

/// Panics (via [`AppError`]) if the Supabase environment is not configured.
pub fn assert_supabase_configured() -> AppResult<()> {
  if let Some(message) = supabase_config_error() {
    return Err(AppError::config(message));
  }
  Ok(())
}

/// Returns a fresh [`SupabaseClient`].
pub fn supabase() -> AppResult<SupabaseClient> {
  let (url, anon) = read_config();
  if url.is_empty() || anon.is_empty() {
    return Err(AppError::config(
      "Supabase environment variables are not configured.",
    ));
  }
  let token = SupabaseClient::load_persisted_token();
  let mut client = SupabaseClient::new(Some(url), Some(anon))
    .ok_or_else(|| AppError::config("Invalid Supabase configuration."))?;
  client.set_token(token);
  Ok(client)
}

fn read_config() -> (String, String) {
  // 1. Try env vars at compile time (works on Vercel / CI).
  let build_url = option_env!("SUPABASE_URL").unwrap_or_default().to_string();
  let build_anon = option_env!("SUPABASE_ANON_KEY")
    .unwrap_or_default()
    .to_string();
  if !build_url.is_empty() && !build_anon.is_empty() {
    return (build_url, build_anon);
  }

  // 2. Fall back to localStorage (set via the in-app config form).
  let url = LocalStorage::get::<String>(URL_STORAGE_KEY).unwrap_or_default();
  let anon = LocalStorage::get::<String>(ANON_STORAGE_KEY).unwrap_or_default();
  (url, anon)
}

/// Save credentials to localStorage (called from the config form).
pub fn save_config(url: &str, anon_key: &str) {
  let _ = LocalStorage::set(URL_STORAGE_KEY, url);
  let _ = LocalStorage::set(ANON_STORAGE_KEY, anon_key);
  clear_config_error_cache();
}

// ----------------------------------------------------------------
// Auth session
// ----------------------------------------------------------------

/// Represents a session response from the `GoTrue` API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
  /// Access token used in the `Authorization` header.
  pub access_token: String,
  /// Token type (always `bearer`).
  pub token_type: String,
  /// Token lifetime in seconds.
  pub expires_in: i64,
  /// Refresh token used to obtain a new access token.
  pub refresh_token: String,
  /// The authenticated user.
  pub user: AuthUser,
}

impl AuthSession {
  /// Persist the access token in `localStorage`.
  pub fn persist(&self) {
    SupabaseClient::persist_token(Some(&self.access_token));
  }
}

/// Represents an authenticated user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
  /// Supabase user id.
  pub id: uuid::Uuid,
  /// Email address.
  pub email: Option<String>,
  /// Optional phone number.
  pub phone: Option<String>,
  /// User metadata (Supabase stores `username` here on signup).
  #[serde(default)]
  pub user_metadata: serde_json::Value,
  /// Created-at timestamp (string).
  #[serde(default)]
  pub created_at: Option<String>,
}
