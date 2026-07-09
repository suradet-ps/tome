//! Authentication state — stored in a root-scoped singleton.

use crate::core::error::{AppError, AppResult};
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
        if self.initialized.get_untracked() { return; }
        if supabase::supabase_config_error().is_none() {
            match restore_session().await {
                Ok(Some((uid, p))) => { self.user.set(Some(uid)); self.profile.set(Some(p)); }
                _ => {}
            }
        }
        self.initialized.set(true);
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> AppResult<()> {
        self.loading.set(true); self.error.set(None);
        let r = async {
            let c = supabase::supabase()?;
            let s = c.auth().sign_in_with_password(email, password).await?;
            s.persist();
            let mut c = c; c.set_token(Some(s.access_token));
            let p = fetch_profile(&c, s.user.id).await?;
            AppResult::Ok((s.user.id, p))
        }.await;
        self.loading.set(false);
        match r {
            Ok((_, Some(p))) => { self.user.set(Some(p.id)); self.profile.set(Some(p)); Ok(()) }
            Ok((_, None)) => Ok(()),
            Err(e) => { self.error.set(Some(e.to_string())); Err(e) }
        }
    }

    pub async fn sign_up(&self, email: &str, password: &str, username: &str) -> AppResult<()> {
        self.loading.set(true); self.error.set(None);
        let meta = serde_json::json!({"username": username});
        let r = async {
            let c = supabase::supabase()?;
            let s = c.auth().sign_up(email, password, meta).await?;
            s.persist();
            let mut c = c; c.set_token(Some(s.access_token));
            let p = fetch_profile(&c, s.user.id).await?;
            AppResult::Ok((s.user.id, p))
        }.await;
        self.loading.set(false);
        match r {
            Ok((_, Some(p))) => { self.user.set(Some(p.id)); self.profile.set(Some(p)); Ok(()) }
            Ok((_, None)) => Ok(()),
            Err(e) => { self.error.set(Some(e.to_string())); Err(e) }
        }
    }

    pub async fn sign_out(&self) {
        if supabase::supabase_config_error().is_none() {
            if let Ok(c) = supabase::supabase() { let _ = c.auth().sign_out().await; }
        }
        supabase::SupabaseClient::persist_token(None);
        self.user.set(None); self.profile.set(None);
    }
}

async fn restore_session() -> AppResult<Option<(uuid::Uuid, Profile)>> {
    let c = supabase::supabase()?;
    if c.token().is_none() { return Ok(None); }
    if let Some(user_id) = c.auth().get_user().await.ok().map(|u| u.id) {
        Ok(fetch_profile(&c, user_id).await?.map(|p| (user_id, p)))
    } else {
        Ok(None)
    }
}

async fn fetch_profile(c: &supabase::SupabaseClient, uid: uuid::Uuid) -> AppResult<Option<Profile>> {
    c.postgrest().from("reading_profiles").select("*").eq("id", uid.to_string()).get_one().await
}
