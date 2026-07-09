//! Centred loading spinner.

use leptos::prelude::*;

/// Indeterminate spinner.
#[component]
pub fn BaseLoader(
    /// Diameter in pixels.
    #[prop(default = 24)]
    size: u32,
) -> impl IntoView {
    view! {
        <div
            class="loader"
            style:--loader-size=format!("{size}px")
            role="status"
            aria-live="polite"
            aria-label="Loading"
        >
            <div class="loader__ring"></div>
        </div>
    }
}
