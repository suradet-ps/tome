//! Global toast notifications with quiet auto-dismiss.
//!
//! Success feedback ("Note saved", "Book added") surfaces as a small,
//! bottom-center toast in addition to the screen-reader announcement. Toasts
//! auto-dismiss after a short pause and can be dismissed manually; a maximum
//! of three stay on screen so a burst of saves never covers the workspace.

use crate::components::icons::X as XIcon;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

const TOAST_LIFETIME_MS: i32 = 3_500;
const MAX_TOASTS: usize = 3;

/// Visual tone of a toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
  /// A quiet success confirmation.
  Success,
  /// A destructive / failed action.
  Error,
}

#[derive(Clone)]
struct Toast {
  id: String,
  kind: ToastKind,
  message: String,
}

thread_local! {
  static TOASTS: RwSignal<Vec<Toast>> = RwSignal::new(Vec::new());
}

/// Push a success toast and speak it to screen readers.
pub fn toast(message: impl Into<String>) {
  push(ToastKind::Success, message);
}

/// Push an error toast and speak it to screen readers.
pub fn toast_error(message: impl Into<String>) {
  push(ToastKind::Error, message);
}

fn push(kind: ToastKind, message: impl Into<String>) {
  let id = uuid::Uuid::new_v4().to_string();
  let message = message.into();
  TOASTS.with(|toasts| {
    toasts.update(|list| {
      list.retain(|toast| toast.id != id);
      list.push(Toast {
        id: id.clone(),
        kind,
        message: message.clone(),
      });
      while list.len() > MAX_TOASTS {
        list.remove(0);
      }
    });
  });
  crate::composables::announce(message);

  // Schedule auto-dismiss.
  let Some(window) = web_sys::window() else {
    return;
  };
  let callback = Closure::wrap(Box::new(move || {
    dismiss(id.clone());
  }) as Box<dyn FnMut()>);
  let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
    callback.as_ref().unchecked_ref(),
    TOAST_LIFETIME_MS,
  );
  callback.forget();
}

/// Remove a toast by id (manual dismiss or timeout).
fn dismiss(id: String) {
  TOASTS.with(|toasts| {
    toasts.update(|list| list.retain(|toast| toast.id != id));
  });
}

/// The toast stack. Render once near the app root, outside the page shell.
#[component]
pub fn Toasts() -> impl IntoView {
  let toasts = TOASTS.with(|toasts| toasts.read_only());
  view! {
      <div class="toast-stack" aria-live="polite">
          <For
              each=move || toasts.get()
              key=|toast| toast.id.clone()
              children=move |toast| {
                  let id = toast.id.clone();
                  let class = match toast.kind {
                      ToastKind::Success => "toast toast--success",
                      ToastKind::Error => "toast toast--error",
                  };
                  view! {
                      <div class=class>
                          <span class="toast__dot" aria-hidden="true"></span>
                          <p class="toast__message">{toast.message}</p>
                          <button
                              type="button"
                              class="toast__close"
                              aria-label="Dismiss notification"
                              on:click=move |_| dismiss(id.clone())
                          >
                              <XIcon size=12 />
                          </button>
                      </div>
                  }
              }
          />
      </div>
  }
}
