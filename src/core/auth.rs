//! Supabase Auth (`GoTrue`) wrappers.

use crate::core::error::{AppError, AppResult};
use crate::core::supabase::AuthSession;
use gloo_net::http::{Method, Request, RequestBuilder};
use serde::{Deserialize, Serialize};

/// Auth handle bound to a Supabase URL/anon key/token.
pub struct SupabaseAuth<'a> {
  url: &'a str,
  api_key: &'a str,
  token: Option<&'a str>,
}

impl<'a> SupabaseAuth<'a> {
  /// Create a new auth handle.
  #[must_use]
  pub const fn new(url: &'a str, api_key: &'a str, token: Option<&'a str>) -> Self {
    Self {
      url,
      api_key,
      token,
    }
  }

  /// Sign in with email and password.
  pub async fn sign_in_with_password(&self, email: &str, password: &str) -> AppResult<AuthSession> {
    let url = format!("{}/auth/v1/token?grant_type=password", self.url);
    let body = serde_json::json!({ "email": email, "password": password });
    self.post_session(&url, &body).await
  }

  /// Sign up with email and password.
  pub async fn sign_up(
    &self,
    email: &str,
    password: &str,
    data: serde_json::Value,
  ) -> AppResult<AuthSession> {
    let url = format!("{}/auth/v1/signup", self.url);
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "data": data,
    });
    self.post_session(&url, &body).await
  }

  /// Sign out the current user.
  pub async fn sign_out(&self) -> AppResult<()> {
    let url = format!("{}/auth/v1/logout", self.url);
    let mut builder = RequestBuilder::new(&url)
      .method(Method::POST)
      .header("apikey", self.api_key);
    if let Some(token) = self.token {
      let value = format!("Bearer {token}");
      builder = builder.header("Authorization", &value);
    }
    let request: Request = builder.build()?;
    let response = request.send().await?;
    if !response.ok() {
      return Err(AppError::http(
        response.status(),
        "Failed to sign out.".to_string(),
      ));
    }
    Ok(())
  }

  /// Look up the current user by access token.
  pub async fn get_user(&self) -> AppResult<AuthUserResponse> {
    let url = format!("{}/auth/v1/user", self.url);
    let token = self.token.ok_or(AppError::Unauthorized)?;
    let value = format!("Bearer {token}");
    let request: Request = RequestBuilder::new(&url)
      .header("apikey", self.api_key)
      .header("Authorization", &value)
      .build()?;
    let response = request.send().await?;
    if !response.ok() {
      return Err(AppError::http(
        response.status(),
        "Failed to fetch user.".to_string(),
      ));
    }
    let body: AuthUserResponse = response.json().await?;
    Ok(body)
  }

  async fn post_session(&self, url: &str, body: &serde_json::Value) -> AppResult<AuthSession> {
    let request: Request = RequestBuilder::new(url)
      .method(Method::POST)
      .header("Content-Type", "application/json")
      .header("apikey", self.api_key)
      .body(body.to_string())?;
    let response = request.send().await?;
    if !response.ok() {
      let status = response.status();
      let body = response.text().await.unwrap_or_default();
      if matches!(status, 400 | 401) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) {
          if let Some(message) = value.get("msg").and_then(|m| m.as_str()) {
            return Err(AppError::http(status, message.to_string()));
          }
          if let Some(message) = value.get("error_description").and_then(|m| m.as_str()) {
            return Err(AppError::http(status, message.to_string()));
          }
          if let Some(message) = value.get("message").and_then(|m| m.as_str()) {
            return Err(AppError::http(status, message.to_string()));
          }
        }
        return Err(AppError::http(status, "Authentication failed.".to_string()));
      }
      return Err(AppError::http(
        status,
        if body.is_empty() {
          status.to_string()
        } else {
          body
        },
      ));
    }
    let session: AuthSession = response.json().await?;
    Ok(session)
  }
}

/// Response from `GET /auth/v1/user`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUserResponse {
  /// Supabase user id.
  pub id: uuid::Uuid,
  /// Email address.
  pub email: Option<String>,
  /// Optional phone number.
  pub phone: Option<String>,
  /// User metadata.
  #[serde(default)]
  pub user_metadata: serde_json::Value,
  /// Created-at timestamp (string).
  #[serde(default)]
  pub created_at: Option<String>,
}
