//! Authentication state stored as a module-level reactive container.
//!
//! Uses `std::cell::LazyCell` (single-threaded WASM) instead of leptos
//! `provide_context` / `use_context` because the router creates isolated
//! scopes that don't inherit parent contexts reliably in 0.8.

use crate::core::error::{AppError, AppResult};
use crate::core::supabase;
use crate::core::types::Profile;
use leptos::prelude::*;
use std::cell::LazyCell;

/// Singleton auth state, initialised once on first access.
thread_local! {
    static AUTH: LazyCell<AuthState> = LazyCell::new(AuthState::default);
}

/// Snapshot of the current authentication state.
#[derive(Debug, Clone, Copy)]
pub struct AuthState {
    /// Reactive handle to the current user.
    pub user: RwSignal<Option<uuid::Uuid>>,
    /// Reactive handle to the user profile.
    pub profile: RwSignal<Option<Profile>>,
    /// Whether auth is initialising.
    pub initialized: RwSignal<bool>,
    /// Whether a sign-in/sign-up is in flight.
    pub loading: RwSignal<bool>,
    /// Last error message.
    pub error: RwSignal<Option<String>>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            user: RwSignal::new(None),
            profile: RwSignal::new(None),
            initialized: RwSignal::new(false),
            loading: RwSignal::new(false),
            error: RwSignal::new(None),
        }
    }
}

impl AuthState {
    /// Returns the current user id, if any.
    #[must_use]
    pub fn user_id(&self) -> Option<uuid::Uuid> {
        self.user.get_untracked()
    }

    /// Returns the current profile, if any.
    #[must_use]
    pub fn profile_value(&self) -> Option<Profile> {
        self.profile.get_untracked()
    }
}

/// Install a singleton auth state (idempotent — only first call has effect).
pub fn provide_auth() -> AuthState {
    AUTH.with(|cell| **cell)
}

/// Read the active auth state.
#[must_use]
pub fn use_auth() -> AuthState {
    AUTH.with(|cell| **cell)
}

impl AuthState {
    /// Initialise the auth state from the persisted session (if any).
    pub async fn init_auth(&self) {
        if self.initialized.get_untracked() {
            return;
        }
        if supabase::supabase_config_error().is_none() {
            match restore_session().await {
                Ok(Some((user_id, profile))) => {
                    self.user.set(Some(user_id));
                    self.profile.set(Some(profile));
                },
                Ok(None) => {
                    self.user.set(None);
                    self.profile.set(None);
                },
                Err(err) => {
                    log::warn!("Failed to restore session: {err}");
                    self.error.set(Some(err.to_string()));
                },
            }
        }
        self.initialized.set(true);
    }

    /// Sign in with email/password.
    pub async fn sign_in(&self, email: &str, password: &str) -> AppResult<()> {
        self.loading.set(true);
        self.error.set(None);
        let result = async {
            let client = supabase::supabase()?;
            let session = client
                .auth()
                .sign_in_with_password(email, password)
                .await?;
            let user_id = session.user.id;
            session.persist();
            let mut client = client;
            client.set_token(Some(session.access_token));
            let profile = fetch_profile(&client, user_id).await?;
            AppResult::Ok((profile, user_id))
        }
        .await;

        self.loading.set(false);
        match result {
            Ok((Some(profile), _)) => {
                self.user.set(Some(profile.id));
                self.profile.set(Some(profile));
                Ok(())
            },
            Ok((None, _)) => Ok(()),
            Err(err) => {
                self.error.set(Some(err.to_string()));
                Err(err)
            },
        }
    }

    /// Sign up a new user.
    pub async fn sign_up(
        &self,
        email: &str,
        password: &str,
        username: &str,
    ) -> AppResult<()> {
        self.loading.set(true);
        self.error.set(None);
        let metadata = serde_json::json!({ "username": username });
        let result = async {
            let client = supabase::supabase()?;
            let session = client.auth().sign_up(email, password, metadata).await?;
            let user_id = session.user.id;
            session.persist();
            let mut client = client;
            client.set_token(Some(session.access_token));
            let profile = fetch_profile(&client, user_id).await?;
            AppResult::Ok((user_id, profile))
        }
        .await;

        self.loading.set(false);
        match result {
            Ok((_, Some(profile))) => {
                self.user.set(Some(profile.id));
                self.profile.set(Some(profile));
                Ok(())
            },
            Ok((_, None)) => Ok(()),
            Err(err) => {
                self.error.set(Some(err.to_string()));
                Err(err)
            },
        }
    }

    /// Sign out and clear all state.
    pub async fn sign_out(&self) {
        if supabase::supabase_config_error().is_none() {
            if let Ok(client) = supabase::supabase() {
                let _ = client.auth().sign_out().await;
            }
        }
        supabase::SupabaseClient::persist_token(None);
        self.user.set(None);
        self.profile.set(None);
    }
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
