//! 404 Not Found page.

use crate::components::icons::ArrowLeft;
use leptos::prelude::*;
use leptos_router::components::A;

/// 404 page shown when no route matches.
#[component]
pub fn NotFound() -> impl IntoView {
  view! {
      <div class="page not-found">
          <div class="not-found__icon" aria-hidden="true">
              <span class="not-found__glyph numeric">"404"</span>
          </div>
          <h1 class="not-found__title">"Page not found"</h1>
          <p class="not-found__copy">
              "The page you're looking for doesn't exist or has moved."
          </p>
          <A href="/" attr:class="not-found__home">
              <ArrowLeft size=14 />
              "Back to library"
          </A>
      </div>
  }
}
