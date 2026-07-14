//! 404 Not Found page.

use leptos::prelude::*;

/// 404 page shown when no route matches.
#[component]
pub fn NotFound() -> impl IntoView {
  view! {
      <div class="page not-found">
          <h1 class="page-header__title">"Not found"</h1>
          <p class="page-header__sub">"We couldn't find what you were looking for."</p>
      </div>
  }
}
