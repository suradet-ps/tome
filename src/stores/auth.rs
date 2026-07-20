//! Authentication state — stored in a root-scoped singleton.

use crate::core::error::AppResult;
use crate::core::supabase;
use crate::core::types::Profile;
use leptos::prelude::*;
use std::sync::OnceLock;

static AUTH: OnceLock<AuthState> = OnceLock::new();

/// Install the singleton (called once from `start()`).
pub fn install() {
  let _ = AUTH.set(AuthState::new());
}

#[derive(Debug, Clone, Copy)]
pub struct AuthState {
  pub user: RwSignal<Option<uuid::Uuid>>,
  pub profile: RwSignal<Option<Profile>>,
  pub initialized: RwSignal<bool>,
  pub loading: RwSignal<bool>,
  pub error: RwSignal<Option<String>>,
}

impl Default for AuthState {
  fn default() -> Self {
    Self::new()
  }
}

impl AuthState {
  pub fn new() -> Self {
    Self {
      user: RwSignal::new(None),
      profile: RwSignal::new(None),
      initialized: RwSignal::new(false),
      loading: RwSignal::new(false),
      error: RwSignal::new(None),
    }
  }
}

pub fn use_auth() -> AuthState {
  *AUTH.get().expect("AuthState not initialized")
}

impl AuthState {
  pub async fn init_auth(&self) {
    if self.initialized.get_untracked() {
      return;
    }
    if supabase::supabase_config_error().is_none()
      && let Ok(Some((uid, p))) = restore_session().await
    {
      self.user.set(Some(uid));
      self.profile.set(Some(p));
    }
    self.initialized.set(true);
  }

  pub async fn sign_in(&self, email: &str, password: &str) -> AppResult<()> {
    self.loading.set(true);
    self.error.set(None);
    let r = async {
      let c = supabase::supabase()?;
      let s = c.auth().sign_in_with_password(email, password).await?;
      s.persist();
      let mut c = c;
      c.set_token(Some(s.access_token));
      let p = fetch_profile(&c, s.user.id).await?;
      AppResult::Ok((s.user.id, p))
    }
    .await;
    self.loading.set(false);
    match r {
      Ok((_, Some(p))) => {
        self.user.set(Some(p.id));
        self.profile.set(Some(p));
        Ok(())
      }
      Ok((_, None)) => Ok(()),
      Err(e) => {
        self.error.set(Some(e.to_string()));
        Err(e)
      }
    }
  }

  pub async fn sign_up(&self, email: &str, password: &str, username: &str) -> AppResult<()> {
    self.loading.set(true);
    self.error.set(None);
    let meta = serde_json::json!({"username": username});
    let r = async {
      let c = supabase::supabase()?;
      let s = c.auth().sign_up(email, password, meta).await?;
      s.persist();
      let mut c = c;
      c.set_token(Some(s.access_token));
      let p = fetch_profile(&c, s.user.id).await?;
      AppResult::Ok((s.user.id, p))
    }
    .await;
    self.loading.set(false);
    match r {
      Ok((_, Some(p))) => {
        self.user.set(Some(p.id));
        self.profile.set(Some(p));
        Ok(())
      }
      Ok((_, None)) => Ok(()),
      Err(e) => {
        self.error.set(Some(e.to_string()));
        Err(e)
      }
    }
  }

  pub async fn sign_out(&self) {
    if supabase::supabase_config_error().is_none()
      && let Ok(c) = supabase::supabase()
    {
      let _ = c.auth().sign_out().await;
    }
    supabase::SupabaseClient::persist_token(None);
    supabase::SupabaseClient::persist_refresh_token(None);
    self.user.set(None);
    self.profile.set(None);
  }

  /// Attempt to recover from an expired access token detected mid-session.
  /// Exchanges the stored refresh token for a new session; on success the new
  /// tokens are persisted and `true` is returned so the caller can retry.
  /// On failure the session is cleared (signed out) and `false` is returned so
  /// the caller can bounce to login.
  pub async fn try_recover_session(&self) -> bool {
    let Ok(mut c) = supabase::supabase() else {
      return false;
    };
    if try_refresh(&mut c).await.is_some() {
      true
    } else {
      self.user.set(None);
      self.profile.set(None);
      false
    }
  }
}

async fn restore_session() -> AppResult<Option<(uuid::Uuid, Profile)>> {
  let mut c = supabase::supabase()?;
  if c.token().is_none() {
    return Ok(None);
  }

  // Try the stored access token first. If it has expired (401/403), attempt a
  // refresh-token exchange before giving up, so a reload with a stale access
  // token self-heals instead of bouncing the reader to login.
  let user_id = match c.auth().get_user().await {
    Ok(user) => Some(user.id),
    Err(e) if e.is_unauthorized() => match try_refresh(&mut c).await {
      Some(user_id) => Some(user_id),
      None => return Ok(None),
    },
    Err(_) => None,
  };

  match user_id {
    Some(user_id) => Ok(fetch_profile(&c, user_id).await?.map(|p| (user_id, p))),
    None => Ok(None),
  }
}

/// Attempt to exchange the persisted refresh token for a fresh session,
/// updating the client's token and persisting the new pair. Returns the user
/// id on success. On failure the stored tokens are cleared so the caller falls
/// back to login cleanly.
async fn try_refresh(c: &mut supabase::SupabaseClient) -> Option<uuid::Uuid> {
  let refresh_token = supabase::SupabaseClient::load_persisted_refresh_token()?;
  match c.auth().refresh_session(&refresh_token).await {
    Ok(session) => {
      session.persist();
      c.set_token(Some(session.access_token.clone()));
      Some(session.user.id)
    }
    Err(_) => {
      supabase::SupabaseClient::persist_token(None);
      supabase::SupabaseClient::persist_refresh_token(None);
      None
    }
  }
}

async fn fetch_profile(
  c: &supabase::SupabaseClient,
  uid: uuid::Uuid,
) -> AppResult<Option<Profile>> {
  c.postgrest()
    .from("reading_profiles")
    .select("*")
    .eq("id", uid.to_string())
    .get_one()
    .await
}
