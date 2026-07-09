//! Tome - Technical Reading Tracker.

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
    mount_to_body(|| view! { <App /> });
}
