/// MissingIcon — placeholder shown wherever a real icon could not be loaded.
///
/// Renders an SVG of a document with a red cross, indicating the icon is absent.
/// Use this instead of emoji or hardcoded fallback text so it is immediately visible
/// where icons are missing.
use dioxus::prelude::*;

/// Inline SVG placeholder for a missing package icon.
///
/// Size is controlled by the parent container.
#[component]
pub fn MissingIcon(
    #[props(default = 32)]
    size: u32,
) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            title { "No icon available" }
            // Outer border
            rect {
                x: "2",
                y: "2",
                width: "20",
                height: "20",
                rx: "3",
                stroke: "var(--fs-color-border-default, #4a4a5a)",
                stroke_width: "1.5",
            }
            // Red diagonal cross
            line {
                x1: "6",
                y1: "6",
                x2: "18",
                y2: "18",
                stroke: "var(--fs-color-error, #ef4444)",
                stroke_width: "2",
                stroke_linecap: "round",
            }
            line {
                x1: "18",
                y1: "6",
                x2: "6",
                y2: "18",
                stroke: "var(--fs-color-error, #ef4444)",
                stroke_width: "2",
                stroke_linecap: "round",
            }
        }
    }
}
