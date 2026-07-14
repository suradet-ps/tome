//! Primary button component with variant/size/loading state.

use leptos::prelude::*;

/// Visual variant of a [`BaseButton`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
  /// Filled yellow primary button.
  #[default]
  Primary,
  /// Outlined secondary button.
  Secondary,
  /// Red-tinted destructive button.
  Danger,
  /// Borderless ghost button.
  Ghost,
}

impl ButtonVariant {
  /// Returns the CSS class suffix for the variant.
  #[must_use]
  pub const fn class(self) -> &'static str {
    match self {
      Self::Primary => "primary",
      Self::Secondary => "secondary",
      Self::Danger => "danger",
      Self::Ghost => "ghost",
    }
  }
}

/// Size of a [`BaseButton`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonSize {
  /// 32px tall, smaller padding.
  Small,
  /// 40px tall, default.
  #[default]
  Medium,
  /// 48px tall, larger.
  Large,
}

impl ButtonSize {
  /// Returns the CSS class suffix for the size.
  #[must_use]
  pub const fn class(self) -> &'static str {
    match self {
      Self::Small => "sm",
      Self::Medium => "md",
      Self::Large => "lg",
    }
  }
}

/// A primary button.
#[component]
pub fn BaseButton(
  /// Visual variant.
  #[prop(default = ButtonVariant::Primary)]
  variant: ButtonVariant,
  /// Button size.
  #[prop(default = ButtonSize::Medium)]
  size: ButtonSize,
  /// Whether the button is in a loading state.
  #[prop(default = false)]
  loading: bool,
  /// Whether the button is disabled.
  #[prop(default = false)]
  disabled: bool,
  /// HTML `type` attribute.
  #[prop(default = "button")]
  button_type: &'static str,
  /// Whether the button should stretch to fill its container.
  #[prop(default = false)]
  block: bool,
  /// Click handler.
  #[prop(optional, into)]
  on_click: Option<Callback<web_sys::MouseEvent>>,
  /// Button content.
  children: Children,
) -> impl IntoView {
  let class = format!(
    "btn btn--{} btn--{}{}{}",
    variant.class(),
    size.class(),
    if loading { " btn--loading" } else { "" },
    if block { " btn--block" } else { "" },
  );
  let is_disabled = disabled || loading;
  view! {
      <button
          type=button_type
          class=class
          disabled=is_disabled
          on:click=move |ev| {
              if !is_disabled
                  && let Some(handler) = on_click.as_ref() {
                      handler.run(ev);
                  }
          }
      >
          <span
              class="btn__spinner"
              aria-hidden="true"
              style:display=if loading { "inline-block" } else { "none" }
          ></span>
          {children()}
      </button>
  }
}
