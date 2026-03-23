/// Colors view — inspect and edit CSS color variables.
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct ColorVar {
    name:  &'static str,
    value: &'static str,
}

const CSS_VARS: &[ColorVar] = &[
    ColorVar { name: "--fs-color-primary",      value: "#00bcd4" },
    ColorVar { name: "--fs-color-bg-base",       value: "#0f172a" },
    ColorVar { name: "--fs-color-bg-surface",    value: "#1e293b" },
    ColorVar { name: "--fs-color-text-primary",  value: "#f1f5f9" },
    ColorVar { name: "--fs-color-text-muted",    value: "#64748b" },
    ColorVar { name: "--fs-color-border-default",value: "#334155" },
    ColorVar { name: "--fs-color-accent",        value: "#7c3aed" },
    ColorVar { name: "--fs-color-error",         value: "#ef4444" },
];

#[component]
pub fn ColorsView() -> Element {
    // Editable values (initialized from CSS_VARS defaults)
    let mut values: Signal<Vec<String>> = use_signal(|| {
        CSS_VARS.iter().map(|v| v.value.to_string()).collect()
    });

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px; max-width: 640px;",

            h3 { style: "margin: 0 0 4px; font-size: 16px; color: var(--fs-color-text-primary);",
                "Colors"
            }

            // Variable rows
            div { style: "display: flex; flex-direction: column; gap: 4px;",
                for (idx, var) in CSS_VARS.iter().enumerate() {
                    ColorRow {
                        key: "{var.name}",
                        name: var.name,
                        value: values.read()[idx].clone(),
                        on_change: {
                            move |new_val: String| {
                                values.write()[idx] = new_val;
                            }
                        },
                    }
                }
            }

            // Apply button
            button {
                style: "align-self: flex-start; \
                        background: var(--fs-color-primary, #00bcd4); color: #fff; \
                        border: none; border-radius: var(--fs-radius-md, 6px); \
                        padding: 10px 20px; font-size: 14px; font-weight: 600; cursor: pointer;",
                "Apply Changes"
            }

            // Note
            p {
                style: "margin: 0; font-size: 12px; color: var(--fs-color-text-muted); font-style: italic;",
                "Changes affect the current session only until saved as a theme."
            }
        }
    }
}

#[component]
fn ColorRow(
    name:      &'static str,
    value:     String,
    on_change: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 12px; \
                    padding: 8px 0; border-bottom: 1px solid var(--fs-color-border-default, #334155);",

            // Variable name
            span {
                style: "flex: 1; font-size: 12px; font-family: monospace; \
                        color: var(--fs-color-text-muted);",
                "{name}"
            }

            // Color swatch
            div {
                style: "width: 20px; height: 20px; border-radius: 4px; \
                        background: {value}; \
                        border: 1px solid rgba(255,255,255,0.15); flex-shrink: 0;",
            }

            // Hex value input
            input {
                r#type: "text",
                value: "{value}",
                style: "width: 100px; background: var(--fs-color-bg-surface, #1e293b); \
                        border: 1px solid var(--fs-color-border-default, #334155); \
                        border-radius: 4px; padding: 4px 8px; \
                        font-size: 12px; font-family: monospace; \
                        color: var(--fs-color-text-primary); flex-shrink: 0;",
                oninput: move |e| on_change.call(e.value()),
            }
        }
    }
}
