//! Reading-comfort settings: theme, content width, and base font size.
//!
//! Persisted to `localStorage` so a reader's preferences survive reloads. The
//! store is installed once at the mount root (like the other stores) and the
//! values are pushed onto the document root as `data-theme` and CSS custom
//! properties (`--reading-width`, `--reading-font-scale`) that `main.css`
//! consumes.

use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const STORAGE_KEY: &str = "tome.settings";

/// Available reading themes. `Auto` follows the OS via `prefers-color-scheme`
/// (mapped to dark here) — kept simple: three explicit, calm themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
  Dark,
  Light,
  Sepia,
}

impl Theme {
  pub const ALL: [Theme; 3] = [Theme::Dark, Theme::Light, Theme::Sepia];

  pub fn as_data_attr(self) -> &'static str {
    match self {
      Theme::Dark => "dark",
      Theme::Light => "light",
      Theme::Sepia => "sepia",
    }
  }

  pub fn label(self) -> &'static str {
    match self {
      Theme::Dark => "Dark",
      Theme::Light => "Light",
      Theme::Sepia => "Sepia",
    }
  }
}

/// Reader-configurable comfort settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
  pub theme: Theme,
  /// Content column width in `ch` (characters). 60–100 is comfortable.
  pub width_ch: u32,
  /// Multiplier applied to the base font size (0.875–1.375).
  pub font_scale: f32,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      theme: Theme::Dark,
      width_ch: 72,
      font_scale: 1.0,
    }
  }
}

impl Settings {
  fn to_json(&self) -> String {
    serde_json::json!({
      "theme": self.theme.as_data_attr(),
      "width_ch": self.width_ch,
      "font_scale": self.font_scale,
    })
    .to_string()
  }

  fn from_json(raw: &str) -> Option<Settings> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    let theme = match value.get("theme")?.as_str()? {
      "light" => Theme::Light,
      "sepia" => Theme::Sepia,
      _ => Theme::Dark,
    };
    let width_ch = value
      .get("width_ch")
      .and_then(|v| v.as_u64())
      .map(|n| n.clamp(48, 120) as u32)
      .unwrap_or(72);
    let font_scale = value
      .get("font_scale")
      .and_then(|v| v.as_f64())
      .map(|n| (n as f32).clamp(0.875, 1.375))
      .unwrap_or(1.0);
    Some(Settings {
      theme,
      width_ch,
      font_scale,
    })
  }
}

/// Reactive settings store.
#[derive(Clone, Copy)]
pub struct SettingsState {
  pub settings: RwSignal<Settings>,
}

impl SettingsState {
  /// Install the store, loading any saved preferences from `localStorage`.
  pub fn install() {
    let initial = LocalStorage::get::<String>(STORAGE_KEY)
      .ok()
      .and_then(|raw| Settings::from_json(&raw))
      .unwrap_or_default();
    let state = SettingsState {
      settings: RwSignal::new(initial),
    };
    provide_context(state);
    state.apply();
  }

  /// Read the store from context.
  pub fn use_ctx() -> Self {
    expect_context()
  }

  /// Persist and apply the current settings to the document.
  pub fn save(&self) {
    let settings = self.settings.get_untracked();
    let _ = LocalStorage::set(STORAGE_KEY, settings.to_json());
    self.apply();
  }

  /// Set the theme and persist.
  pub fn set_theme(&self, theme: Theme) {
    self.settings.update(|s| s.theme = theme);
    self.save();
  }

  /// Set content width (in `ch`) and persist.
  pub fn set_width(&self, width_ch: u32) {
    self
      .settings
      .update(|s| s.width_ch = width_ch.clamp(48, 120));
    self.save();
  }

  /// Set font-scale multiplier and persist.
  pub fn set_font_scale(&self, scale: f32) {
    self
      .settings
      .update(|s| s.font_scale = scale.clamp(0.875, 1.375));
    self.save();
  }

  /// Push the current settings onto the document root.
  fn apply(&self) {
    let settings = self.settings.get_untracked();
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
      if let Some(root) = doc
        .document_element()
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
      {
        let _ = root.set_attribute("data-theme", settings.theme.as_data_attr());
        let style = root.style();
        let _ = style.set_property("--reading-width", &format!("{}ch", settings.width_ch));
        let _ = style.set_property(
          "--reading-font-scale",
          &format!("{:.3}", settings.font_scale),
        );
      }
    }
  }
}
