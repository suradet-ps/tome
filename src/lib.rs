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
use reactive_graph::owner::Owner;
use log::Level;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
    init_with_level(Level::Debug).ok();

    // Create a persistent root owner that never gets disposed, and
    // initialise all stores inside it.
    let root = Owner::new_root(None);
    root.with(|| {
        crate::stores::auth::install();
        crate::stores::books::install();
        crate::stores::progress::install();
        crate::stores::notes::install();
    });

    mount_to_body(|| view! { <App /> });
}
