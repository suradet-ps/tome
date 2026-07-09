//! Low-level browser APIs and Supabase wrappers used by the application.

pub mod auth;
pub mod error;
pub mod highlight;
pub mod markdown;
pub mod postgrest;
pub mod supabase;
pub mod time;
pub mod types;
pub mod utils;

pub use error::{AppError, AppResult};
pub use supabase::{SupabaseClient, assert_supabase_configured, supabase, supabase_config_error};
