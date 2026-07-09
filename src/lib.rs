//! Tome - Technical Reading Tracker.
//!
//! A dark-first technical reading tracker built with Leptos 0.8 (CSR) and
//! Supabase.

#![allow(missing_docs)]

pub mod app;
pub mod components;
pub mod composables;
pub mod core;
pub mod stores;
pub mod views;

pub use app::App;

use console_error_panic_hook::set_once as set_panic_hook;
use console_log::init_with_level;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use log::Level;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
    init_with_level(Level::Debug).ok();

    // Initialise stores inside mount_to_body's closure so their
    // RwSignals live in the mount root owner (never disposed).
    mount_to_body(|| {
        crate::stores::auth::install();
        crate::stores::books::install();
        crate::stores::progress::install();
        crate::stores::notes::install();

        view! { <App /> }
    });
}
