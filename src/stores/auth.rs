//! Authentication state.
//!
//! Provides a single [`AuthState`] value that the application installs at
//! the root via `provide_auth`. Components access it through `use_auth`.

use crate::core::error::{AppError, AppResult};
use crate::core::supabase;
use crate::core::types::Profile;
use leptos::prelude::*;
use serde_json::Value;

/// Snapshot of the current authentication state, exposed to views.
#[derive(Debug, Clone, Copy)]
pub struct AuthState {
    /// Reactive handle to the current user.
    pub user: RwSignal<Option<uuid::Uuid>>,
    /// Reactive handle to the user profile (`reading_profiles`).
    pub profile: RwSignal<Option<Profile>>,
    /// Whether auth is initialising.
    pub initialized: RwSignal<bool>,
    /// Whether a sign-in/sign-up is in flight.
    pub loading: RwSignal<bool>,
    /// Last error message.
    pub error: RwSignal<Option<String>>,
}

impl AuthState {
    /// Returns the current user id, if any.
    #[must_use]
    pub fn user_id(&self) -> Option<uuid::Uuid> {
        self.user.get()
    }

    /// Returns the current profile, if any.
    #[must_use]
    pub fn profile_value(&self) -> Option<Profile> {
        self.profile.get()
    }
}

/// Install the [`AuthState`] into the current reactive scope.
#[must_use]
pub fn provide_auth() -> AuthState {
    let state = AuthState {
        user: RwSignal::new(None),
        profile: RwSignal::new(None),
        initialized: RwSignal::new(false),
        loading: RwSignal::new(false),
        error: RwSignal::new(None),
    };
    provide_context(state);
    state
}

/// Read the active [`AuthState`] from context.
#[must_use]
pub fn use_auth() -> AuthState {
    use_context::<AuthState>().expect("AuthState must be provided at the root")
}

impl AuthState {
    /// Initialise the auth state from the persisted session (if any).
    pub async fn init_auth(&self) {
        if self.initialized.get() {
            return;
        }
        if supabase_configured() {
            match restore_session().await {
                Ok(Some((user_id, profile))) => {
                    self.user.set(Some(user_id));
                    self.profile.set(Some(profile));
                }
                Ok(None) => {
                    self.user.set(None);
                    self.profile.set(None);
                }
                Err(err) => {
                    log::warn!("Failed to restore session: {err}");
                    self.error.set(Some(err.to_string()));
                }
            }
        }
        self.initialized.set(true);
    }

    /// Sign in with email/password.
    pub async fn sign_in(&self, email: &str, password: &str) -> AppResult<()> {
        self.loading.set(true);
        self.error.set(None);
        let result: AppResult<Option<Profile>> = async {
            let client = supabase::supabase()?;
            let session = client.auth().sign_in_with_password(email, password).await?;
            let user_id = session.user.id;
            session.persist();
            let client = with_token(&client, Some(session.access_token.as_str()));
            let profile = fetch_profile(&client, user_id).await?;
            Ok::<Option<Profile>, AppError>(profile)
        }
        .await;
        match result {
            Ok(Some(profile)) => {
                self.user.set(Some(profile.id));
                self.profile.set(Some(profile));
                self.loading.set(false);
                Ok(())
            }
            Ok(None) => {
                self.loading.set(false);
                Ok(())
            }
            Err(err) => {
                self.error.set(Some(err.to_string()));
                self.loading.set(false);
                Err(err)
            }
        }
    }

    /// Sign up a new user.
    pub async fn sign_up(&self, email: &str, password: &str, username: &str) -> AppResult<()> {
        self.loading.set(true);
        self.error.set(None);
        let result: AppResult<Option<Profile>> = async {
            let client = supabase::supabase()?;
            let metadata = serde_json::json!({ "username": username });
            let session = client.auth().sign_up(email, password, metadata).await?;
            let user_id = session.user.id;
            session.persist();
            let client = with_token(&client, Some(session.access_token.as_str()));
            let profile = fetch_profile(&client, user_id).await?;
            Ok::<Option<Profile>, AppError>(profile)
        }
        .await;
        match result {
            Ok(Some(profile)) => {
                self.user.set(Some(profile.id));
                self.profile.set(Some(profile));
                self.loading.set(false);
                Ok(())
            }
            Ok(None) => {
                self.loading.set(false);
                Ok(())
            }
            Err(err) => {
                self.error.set(Some(err.to_string()));
                self.loading.set(false);
                Err(err)
            }
        }
    }

    /// Sign out and clear all state.
    pub async fn sign_out(&self) {
        if supabase_configured() {
            if let Ok(client) = supabase::supabase() {
                let _ = client.auth().sign_out().await;
            }
        }
        supabase::SupabaseClient::persist_token(None);
        self.user.set(None);
        self.profile.set(None);
    }

    /// Refresh the profile (after a sign-in or mutation).
    pub async fn refresh_profile(&self) -> AppResult<()> {
        let user_id = match self.user.get() {
            Some(id) => id,
            None => return Ok(()),
        };
        let client = supabase::supabase()?;
        let profile = fetch_profile(&client, user_id).await?;
        self.profile.set(profile);
        Ok(())
    }
}

fn supabase_configured() -> bool {
    supabase::supabase_config_error().is_none()
}

async fn restore_session() -> AppResult<Option<(uuid::Uuid, Profile)>> {
    let client = supabase::supabase()?;
    if client.token().is_none() {
        return Ok(None);
    }
    let user = client
        .auth()
        .get_user()
        .await
        .ok()
        .map(|u| (u.id, u.user_metadata));
    let user_id = match user {
        Some((id, _)) => id,
        None => return Ok(None),
    };
    let profile = fetch_profile(&client, user_id).await?;
    match profile {
        Some(profile) => Ok(Some((user_id, profile))),
        None => Ok(None),
    }
}

async fn fetch_profile(
    client: &supabase::SupabaseClient,
    user_id: uuid::Uuid,
) -> AppResult<Option<Profile>> {
    let profile: Option<Profile> = client
        .postgrest()
        .from("reading_profiles")
        .select("*")
        .eq("id", user_id.to_string())
        .get_one()
        .await?;
    Ok(profile)
}

fn with_token<'a>(
    client: &'a supabase::SupabaseClient,
    token: Option<&str>,
) -> supabase::SupabaseClient {
    let mut next = client.clone();
    next.set_token(token.map(str::to_string));
    next
}

// Allow `Value` to be used in the metadata payload without an explicit import
// at every call site.
#[allow(dead_code)]
fn _ensure_value_unused(_: Value) {}
