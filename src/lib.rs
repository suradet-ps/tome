//! Tome - Technical Reading Tracker.
//!
//! A dark-first technical reading tracker built with Leptos 0.8 (CSR) and
//! Supabase. The library exports the [`App`] component which the WASM entry
//! point mounted from [`start`] mounts into `<body>`.
//!
//! # Architecture
//!
//! * [`components`] - reusable UI primitives and views.
//! * [`composables`] - reactive helpers (timers, markdown).
//! * [`core`] - thin wrappers over the Supabase REST + GoTrue APIs that run
//!   entirely in the browser via `gloo-net`.
//! * [`stores`] - reactive containers (auth, books, progress, notes) shared
//!   via Leptos contexts.
//! * [`views`] - top-level pages mounted by the router.

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
use log::Level;
use wasm_bindgen::prelude::wasm_bindgen;

/// Boot the application.
///
/// Called automatically by `wasm-bindgen` when the WASM module is loaded.
#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
    init_with_level(Level::Debug).ok();

    mount_to_body(App);
}
