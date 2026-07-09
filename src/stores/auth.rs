//! Authentication state provided via leptos contexts.

use crate::core::error::{AppError, AppResult};
use crate::core::supabase;
use crate::core::types::Profile;
use leptos::prelude::*;

/// Snapshot of the current authentication state.
#[derive(Debug, Clone, Copy)]
pub struct AuthState {
    pub user: RwSignal<Option<uuid::Uuid>>,
    pub profile: RwSignal<Option<Profile>>,
    pub initialized: RwSignal<bool>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
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

    #[must_use]
    pub fn user_id(&self) -> Option<uuid::Uuid> {
        self.user.get_untracked()
    }

    #[must_use]
    pub fn profile_value(&self) -> Option<Profile> {
        self.profile.get_untracked()
    }
}

pub fn provide_auth() -> AuthState {
    let state = AuthState::new();
    provide_context(state);
    state
}

#[must_use]
pub fn use_auth() -> AuthState {
    use_context::<AuthState>().expect("AuthState must be provided at the root")
}

impl AuthState {
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
                Ok(None) => {},
                Err(err) => {
                    log::warn!("Failed to restore session: {err}");
                    self.error.set(Some(err.to_string()));
                },
            }
        }
        self.initialized.set(true);
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> AppResult<()> {
        self.loading.set(true);
        self.error.set(None);
        let result = async {
            let client = supabase::supabase()?;
            let session = client.auth().sign_in_with_password(email, password).await?;
            session.persist();
            let mut client = client;
            client.set_token(Some(session.access_token));
            let profile = fetch_profile(&client, session.user.id).await?;
            AppResult::Ok((session.user.id, profile))
        }.await;
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

    pub async fn sign_up(&self, email: &str, password: &str, username: &str) -> AppResult<()> {
        self.loading.set(true);
        self.error.set(None);
        let metadata = serde_json::json!({"username": username});
        let result = async {
            let client = supabase::supabase()?;
            let session = client.auth().sign_up(email, password, metadata).await?;
            session.persist();
            let mut client = client;
            client.set_token(Some(session.access_token));
            let profile = fetch_profile(&client, session.user.id).await?;
            AppResult::Ok((session.user.id, profile))
        }.await;
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
    if client.token().is_none() { return Ok(None); }
    let user = client.auth().get_user().await.ok().map(|u| u.id);
    match user {
        Some(user_id) => {
            match fetch_profile(&client, user_id).await? {
                Some(profile) => Ok(Some((user_id, profile))),
                None => Ok(None),
            }
        },
        None => Ok(None),
    }
}

async fn fetch_profile(client: &supabase::SupabaseClient, user_id: uuid::Uuid) -> AppResult<Option<Profile>> {
    client.postgrest().from("reading_profiles").select("*").eq("id", user_id.to_string()).get_one().await
}
