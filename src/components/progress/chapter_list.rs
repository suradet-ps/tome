//! Recursive chapter list with status icons and expand/collapse behaviour.

use crate::components::icons::{
  AlertCircle, CheckCircle, ChevronDown, ChevronRight, Circle, Clock,
};
use crate::core::types::{Chapter, ReadingStatus};
use crate::stores::progress::ProgressState;
use leptos::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::JsCast;

/// Recursive chapter list component.
#[component]
pub fn ChapterList(
  /// Chapter tree to render.
  chapters: Signal<Vec<Chapter>>,
  /// Currently selected chapter id.
  #[prop(optional, into)]
  selected: Signal<Option<uuid::Uuid>>,
  /// Indentation depth (0 for the root call).
  #[prop(default = 0)]
  depth: u32,
  /// Selection event handler.
  on_select: Callback<Chapter>,
) -> AnyView {
  let expanded: RwSignal<HashSet<uuid::Uuid>> = RwSignal::new(HashSet::new());
  let progress = ProgressState::use_ctx();

  // Auto-expand parents at the root level.
  Effect::new(move |_| {
    if depth > 0 {
      return;
    }
    let mut current = expanded.get_untracked();
    for chapter in chapters.get() {
      if !chapter.children.is_empty() && !current.contains(&chapter.id) {
        current.insert(chapter.id);
      }
    }
    expanded.set(current);
  });

  let is_expanded = move |id: uuid::Uuid| {
    if depth == 0 {
      expanded.get().contains(&id)
    } else {
      true
    }
  };

  let toggle = move |id: uuid::Uuid| {
    expanded.update(|set| {
      if set.contains(&id) {
        set.remove(&id);
      } else {
        set.insert(id);
      }
    });
  };

  // Tree-level keyboard navigation. Queries every visible treeitem in this
  // tree (including nested ones) and moves focus / expands+collapses so the
  // chapter list is fully operable without a mouse.
  let on_tree_keydown = move |ev: web_sys::KeyboardEvent| {
    if depth != 0 {
      return;
    }
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
      return;
    };
    let Some(tree) = doc
      .query_selector(".chapter-list[role=\"tree\"]")
      .ok()
      .flatten()
    else {
      return;
    };
    let items: Vec<web_sys::HtmlElement> = tree
      .query_selector_all("[role=\"treeitem\"]:not([aria-hidden=\"true\"])")
      .ok()
      .map(|n| {
        (0..n.length())
          .filter_map(|i| n.item(i)?.dyn_ref::<web_sys::HtmlElement>().cloned())
          .collect()
      })
      .unwrap_or_default();
    if items.is_empty() {
      return;
    }
    let active = doc
      .active_element()
      .and_then(|el| el.dyn_ref::<web_sys::HtmlElement>().cloned());
    let pos = active
      .as_ref()
      .and_then(|a| items.iter().position(|it| it == a))
      .unwrap_or(0);
    let mut next: Option<usize> = None;
    match ev.key().as_str() {
      "ArrowDown" => next = Some((pos + 1).min(items.len() - 1)),
      "ArrowUp" => next = Some(pos.saturating_sub(1)),
      "Home" => next = Some(0),
      "End" => next = Some(items.len() - 1),
      _ => {}
    }
    if let Some(idx) = next {
      ev.prevent_default();
      let _ = items[idx].focus();
    }
  };

  let inner = view! {
      <ul
          class=move || {
              if depth > 0 { "chapter-list chapter-list--nested" } else { "chapter-list" }
          }
          role=move || if depth == 0 { "tree" } else { "group" }
          on:keydown=on_tree_keydown
      >
          <For
              each=move || chapters.get()
              key=|chapter| chapter.id
              children=move |chapter: Chapter| {
                  let chapter_for_click = chapter.clone();
                  let chapter_for_key = chapter.clone();
                  let chapter_id = chapter.id;
                  let expanded_now = is_expanded(chapter.id);
                  let status_signal = Signal::derive(move || {
                      progress
                          .get(chapter_id)
                          .map(|p| p.status)
                          .unwrap_or_default()
                  });
                  let has_children = !chapter.children.is_empty();
                  let children_signal = Signal::derive(move || chapter.children.clone());
                  let row_on_keydown = move |ev: web_sys::KeyboardEvent| {
                      match ev.key().as_str() {
                          "Enter" | " " => {
                              ev.prevent_default();
                              on_select.run(chapter_for_key.clone());
                          }
                          "ArrowRight" if has_children && !expanded_now => {
                              ev.prevent_default();
                              toggle(chapter_id);
                          }
                          "ArrowLeft" if has_children && expanded_now => {
                              ev.prevent_default();
                              toggle(chapter_id);
                          }
                          _ => {}
                      }
                  };
                  view! {
                      <li class="chapter-item">
                          <div
                              class=move || {
                                  if selected.get() == Some(chapter_id) {
                                      "chapter-row chapter-row--active"
                                  } else {
                                      "chapter-row"
                                  }
                              }
                              style:padding-left=move || format!("{}px", depth * 14 + 8)
                              role="treeitem"
                              tabindex="0"
                              aria-selected=move || (selected.get() == Some(chapter_id)).to_string()
                              aria-expanded=move || if has_children { expanded_now.to_string() } else { String::new() }
                              on:click=move |_| on_select.run(chapter_for_click.clone())
                              on:keydown=row_on_keydown
                          >
                              {has_children.then(|| view! {
                                  <button
                                      class="chapter-expand"
                                      type="button"
                                      aria-expanded=move || expanded_now.to_string()
                                      aria-label=move || if expanded_now { "Collapse" } else { "Expand" }
                                      on:click=move |ev| {
                                          ev.stop_propagation();
                                          toggle(chapter_id);
                                      }
                                  >
                                      <Show
                                          when=move || expanded_now
                                          fallback=move || view! { <ChevronRight size=12 /> }
                                      >
                                          <ChevronDown size=12 />
                                      </Show>
                                  </button>
                              })}
                              {(!has_children).then(|| view! { <span class="chapter-expand chapter-expand--spacer"></span> })}

                              <ChapterStatusIcon status=status_signal />

                              <span class="chapter-seq numeric">{chapter.sequence_number}</span>
                              <span class="chapter-title">{chapter.title}</span>
                          </div>

                          <Show when=move || has_children && expanded_now fallback=|| view! {}>
                              <ChapterList
                                  chapters=children_signal
                                  selected=selected
                                  depth=depth + 1
                                  on_select=on_select
                              />
                          </Show>
                      </li>
                  }
              }
          />
      </ul>
  };
  inner.into_any()
}

#[component]
fn ChapterStatusIcon(status: Signal<ReadingStatus>) -> impl IntoView {
  let style_success = "color: var(--color-success)".to_string();
  let style_info = "color: var(--color-info)".to_string();
  let style_warning = "color: var(--color-warning)".to_string();
  let style_muted = "color: var(--color-muted)".to_string();
  let class_name = "chapter-icon".to_string();
  view! {
      {move || match status.get() {
          ReadingStatus::Completed => view! { <CheckCircle size=14 attr:style=style_success.clone() class=class_name.clone() /> }.into_any(),
          ReadingStatus::InProgress => view! { <Clock size=14 attr:style=style_info.clone() class=class_name.clone() /> }.into_any(),
          ReadingStatus::ReviewNeeded => view! { <AlertCircle size=14 attr:style=style_warning.clone() class=class_name.clone() /> }.into_any(),
          ReadingStatus::NotStarted => view! { <Circle size=14 attr:style=style_muted.clone() class=class_name.clone() /> }.into_any(),
      }}
  }
}
