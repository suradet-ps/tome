//! Accessible modal dialog with focus trap and Escape support.

use crate::components::icons::X;
use leptos::children::ChildrenFn;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Modal dialog.
///
/// Renders into `<body>` via a portal and traps focus while open. Closes on
/// Escape or click on the overlay.
#[component]
pub fn BaseModal(
    /// Whether the modal is visible.
    open: Signal<bool>,
    /// Called when the user wants to close the modal.
    on_close: Callback<()>,
    /// Title shown in the header.
    #[prop(optional, into)]
    title: Option<String>,
    /// Modal body.
    children: ChildrenFn,
) -> impl IntoView {
    let container_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    // Track the previously focused element to restore it on close.
    let previous_focus: StoredValue<Option<web_sys::HtmlElement>> = StoredValue::new(None);

    // Focus trap.
    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        // Move focus to the first focusable element inside the modal.
        if let Some(node) = container_ref.get() {
            if let Some(first) = focusable_within(&node).into_iter().next() {
                let _ = first.focus();
            } else {
                let _ = node.focus();
            }
        }
    });

    Effect::new(move |_| {
        let is_open = open.get();
        if is_open {
            if let Some(active) = web_sys::window().and_then(|w| w.document())
                && let Some(active) = active.active_element()
                && let Some(el) = active.dyn_ref::<web_sys::HtmlElement>()
            {
                previous_focus.set_value(Some(el.clone()));
            }
        } else if let Some(prev) = previous_focus.get_value() {
            let _ = prev.focus();
            previous_focus.set_value(None);
        }
    });

    // Keyboard handlers.
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if !open.get() {
            return;
        }
        match ev.key().as_str() {
            "Escape" => {
                ev.stop_propagation();
                on_close.run(());
            }
            "Tab" => {
                if let Some(node) = container_ref.get() {
                    let items = focusable_within(&node);
                    if items.is_empty() {
                        ev.prevent_default();
                        return;
                    }
                    let active = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.active_element())
                        .and_then(|el| el.dyn_ref::<web_sys::HtmlElement>().cloned());
                    let first = items[0].clone();
                    let last = items[items.len() - 1].clone();
                    let contains_active = active
                        .as_ref()
                        .and_then(|a| a.dyn_ref::<web_sys::Node>())
                        .is_some_and(|n| node.contains(Some(n)));
                    if ev.shift_key() {
                        if active.as_deref() == Some(&first) || !contains_active {
                            ev.prevent_default();
                            let _ = last.focus();
                        }
                    } else if active.as_deref() == Some(&last) {
                        ev.prevent_default();
                        let _ = first.focus();
                    }
                }
            }
            _ => {}
        }
    };

    view! {
        <Show when=move || open.get() fallback=|| view! {}>
            <div
                class="modal-overlay"
                on:click=move |ev| {
                    if ev.target() == ev.current_target() {
                        on_close.run(());
                    }
                }
                on:keydown=on_keydown
                tabindex="-1"
            >
                <div
                    node_ref=container_ref
                    class="modal-container"
                    role="dialog"
                    aria-modal="true"
                    tabindex="-1"
                    aria-label=title.clone().unwrap_or_else(|| "Dialog".to_string())
                >
                    <div class="modal-header">
                        <h3 class="modal-title">{title.clone().unwrap_or_default()}</h3>
                        <button
                            class="modal-close"
                            type="button"
                            on:click=move |_| on_close.run(())
                            aria-label="Close"
                        >
                            <X size=18 />
                        </button>
                    </div>
                    <div class="modal-body">{children()}</div>
                </div>
            </div>
        </Show>
    }
}

#[allow(dead_code)]
const fn _ensure_unused() {}

fn focusable_within(root: &web_sys::HtmlElement) -> Vec<web_sys::HtmlElement> {
    let selector = "a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), \
         textarea:not([disabled]), [tabindex]:not([tabindex=\"-1\"])";
    let element: &web_sys::Element = root.as_ref();
    let mut out = Vec::new();
    if let Ok(nodes) = element.query_selector_all(selector) {
        for idx in 0..nodes.length() {
            if let Some(node) = nodes.item(idx)
                && let Some(el) = node.dyn_ref::<web_sys::HtmlElement>()
            {
                let hidden = node
                    .parent_element()
                    .is_some_and(|p| p.has_attribute("aria-hidden"));
                if !hidden && el.offset_parent().is_some() {
                    out.push(el.clone());
                }
            }
        }
    }
    out
}
