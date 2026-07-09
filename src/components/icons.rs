//! Inline Lucide SVG icons.
//!
//! Each icon is a thin component that renders the original Lucide SVG path.
//! This keeps the bundle small and avoids depending on a third-party icon
//! crate whose icon set may not match the original Vue project 1:1.

use leptos::prelude::*;

/// Render an SVG with the supplied `view_box`, paths and the configured
/// `size` / colour attributes.
#[component]
pub fn Icon(
    /// SVG viewBox (`"0 0 24 24"` by default).
    #[prop(default = "0 0 24 24")]
    view_box: &'static str,
    /// Number of SVG path elements.
    paths: Vec<&'static str>,
    /// Optional CSS class to apply to the root `<svg>`.
    #[prop(optional, into)]
    class: Option<String>,
    /// Stroke colour (defaults to `currentColor`).
    #[prop(default = "currentColor")]
    stroke: &'static str,
    /// Fill colour (defaults to `none`).
    #[prop(default = "none")]
    fill: &'static str,
    /// Width / height in pixels.
    #[prop(default = 16)]
    size: u32,
    /// Stroke width.
    #[prop(default = 2.0)]
    stroke_width: f32,
) -> impl IntoView {
    let class_attr = class.unwrap_or_default();
    let path_views = paths
        .into_iter()
        .map(|d| {
            view! { <path d=d fill="none" stroke="currentColor" stroke-width=stroke_width stroke-linecap="round" stroke-linejoin="round" /> }
        })
        .collect_view();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=size
            height=size
            viewBox=view_box
            fill=fill
            stroke=stroke
            stroke-width=stroke_width
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class_attr
            aria-hidden="true"
        >
            {path_views}
        </svg>
    }
}

macro_rules! icon {
    ($name:ident, $size:expr, $view_box:expr, $($path:expr),+ $(,)?) => {
        #[component]
        pub fn $name(
            #[prop(default = $size)] size: u32,
            #[prop(default = 2.0)] stroke_width: f32,
            #[prop(optional)] class: Option<String>,
        ) -> impl IntoView {
            let paths: Vec<&'static str> = vec![$($path),+];
            let class_value = class.unwrap_or_default();
            view! {
                <Icon
                    view_box=$view_box
                    paths=paths
                    size=size
                    stroke_width=stroke_width
                    class=class_value
                />
            }
        }
    };
}

icon!(
    book_open,
    24,
    "0 0 24 24",
    "M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z",
    "M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"
);

icon!(arrow_right, 24, "0 0 24 24", "M5 12h14", "m12 5 7 7-7 7");

icon!(arrow_left, 24, "0 0 24 24", "m12 19-7-7 7-7", "M19 12H5");

icon!(plus, 24, "0 0 24 24", "M5 12h14", "m12 5v14");

icon!(x, 24, "0 0 24 24", "M18 6 6 18", "m6 6 12 12");

icon!(play, 24, "0 0 24 24", "M6 3l14 9-14 9V3z");

icon!(pause, 24, "0 0 24 24", "M6 4h4v16H6z", "M14 4h4v16h-4z");

icon!(
    rotate_ccw,
    24,
    "0 0 24 24",
    "M3 12a9 9 0 1 0 3-6.7L3 8",
    "M3 3v5h5"
);

icon!(
    save,
    24,
    "0 0 24 24",
    "M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z",
    "M17 21v-8H7v8",
    "M7 3v5h8"
);

icon!(
    eye,
    24,
    "0 0 24 24",
    "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z",
    "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"
);

icon!(
    eye_off,
    24,
    "0 0 24 24",
    "M9.88 9.88a3 3 0 1 0 4.24 4.24",
    "M10.73 5.08A11 11 0 0 1 12 5c7 0 11 7 11 7a13.16 13.16 0 0 1-1.67 2.68",
    "M6.61 6.61A13.526 13.526 0 0 0 1 12s4 7 11 7a9.74 9.74 0 0 0 5.39-1.61",
    "M1 1l22 22"
);

icon!(
    layout_dashboard,
    24,
    "0 0 24 24",
    "M3 3h7v9H3z",
    "M14 3h7v5h-7z",
    "M14 12h7v9h-7z",
    "M3 16h7v5H3z"
);

icon!(
    brain,
    24,
    "0 0 24 24",
    "M9.5 2A2.5 2.5 0 0 1 12 4.5v15a2.5 2.5 0 0 1-4.96.44 2.5 2.5 0 0 1-2.96-3.08 3 3 0 0 1-.34-5.58 2.5 2.5 0 0 1 1.32-4.24 2.5 2.5 0 0 1 1.98-3A2.5 2.5 0 0 1 9.5 2z",
    "M14.5 2A2.5 2.5 0 0 0 12 4.5v15a2.5 2.5 0 0 0 4.96.44 2.5 2.5 0 0 0 2.96-3.08 3 3 0 0 0 .34-5.58 2.5 2.5 0 0 0-1.32-4.24 2.5 2.5 0 0 0-1.98-3A2.5 2.5 0 0 0 14.5 2z"
);

icon!(
    log_out,
    24,
    "0 0 24 24",
    "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4",
    "M16 17l5-5-5-5",
    "M21 12H9"
);

icon!(menu, 24, "0 0 24 24", "M4 6h16", "M4 12h16", "M4 18h16");

icon!(
    circle,
    24,
    "0 0 24 24",
    "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z"
);

icon!(
    check_circle,
    24,
    "0 0 24 24",
    "M22 11.08V12a10 10 0 1 1-5.93-9.14",
    "m9 11 3 3L22 4"
);

icon!(
    alert_circle,
    24,
    "0 0 24 24",
    "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z",
    "M12 8v4",
    "M12 16h.01"
);

icon!(
    clock,
    24,
    "0 0 24 24",
    "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z",
    "M12 6v6l4 2"
);

icon!(chevron_down, 24, "0 0 24 24", "m6 9 6 6 6-6");

icon!(chevron_right, 24, "0 0 24 24", "m9 18 6-6-6-6");

icon!(
    thumbs_up,
    24,
    "0 0 24 24",
    "M7 10v12",
    "M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H7V10l3.34-7A2 2 0 1 1 14 4.34L15 5.88z"
);

icon!(
    thumbs_down,
    24,
    "0 0 24 24",
    "M17 14V2",
    "M9 18.12 10 14H4.17a2 2 0 0 1-1.92-2.56l2.33-8A2 2 0 0 1 6.5 2H17v12l-3.34 7a2 2 0 0 1-3.66-1.88z"
);

icon!(minus, 24, "0 0 24 24", "M5 12h14");

icon!(check, 24, "0 0 24 24", "M20 6 9 17l-5-5");

icon!(
    check_check,
    24,
    "0 0 24 24",
    "M2 12 7 17l5-5",
    "m11 12 4 4 7-9"
);

icon!(
    clock3,
    24,
    "0 0 24 24",
    "M12 6v6l4 2",
    "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z"
);
