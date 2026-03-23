/// Chrome view — configure window decorations, sidebar style, and animations.
use dioxus::prelude::*;

#[component]
pub fn ChromeView() -> Element {
    let mut window_style  = use_signal(|| "kde");
    let mut sidebar_style = use_signal(|| "solid");
    let mut anim_enabled  = use_signal(|| true);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 24px; max-width: 480px;",

            h3 { style: "margin: 0 0 4px; font-size: 16px; color: var(--fs-color-text-primary);",
                "Window Chrome & Layout"
            }

            // Window Style
            div {
                label_heading { label: "Window Style" }
                div { style: "display: flex; flex-direction: column; gap: 8px; margin-top: 8px;",
                    RadioOption {
                        id: "macos",
                        label: "macOS (circles)",
                        group: "window_style",
                        checked: *window_style.read() == "macos",
                        on_change: move |_| window_style.set("macos"),
                    }
                    RadioOption {
                        id: "kde",
                        label: "KDE (rectangles)",
                        group: "window_style",
                        checked: *window_style.read() == "kde",
                        on_change: move |_| window_style.set("kde"),
                    }
                    RadioOption {
                        id: "windows",
                        label: "Windows (squares)",
                        group: "window_style",
                        checked: *window_style.read() == "windows",
                        on_change: move |_| window_style.set("windows"),
                    }
                    RadioOption {
                        id: "minimal",
                        label: "Minimal (none)",
                        group: "window_style",
                        checked: *window_style.read() == "minimal",
                        on_change: move |_| window_style.set("minimal"),
                    }
                }
            }

            // Sidebar Style
            div {
                label_heading { label: "Sidebar" }
                div { style: "display: flex; flex-direction: column; gap: 8px; margin-top: 8px;",
                    RadioOption {
                        id: "solid",
                        label: "Solid",
                        group: "sidebar_style",
                        checked: *sidebar_style.read() == "solid",
                        on_change: move |_| sidebar_style.set("solid"),
                    }
                    RadioOption {
                        id: "glass",
                        label: "Glass",
                        group: "sidebar_style",
                        checked: *sidebar_style.read() == "glass",
                        on_change: move |_| sidebar_style.set("glass"),
                    }
                    RadioOption {
                        id: "transparent",
                        label: "Transparent",
                        group: "sidebar_style",
                        checked: *sidebar_style.read() == "transparent",
                        on_change: move |_| sidebar_style.set("transparent"),
                    }
                }
            }

            // Animations toggle
            div {
                label_heading { label: "Animations" }
                div { style: "display: flex; gap: 12px; margin-top: 8px;",
                    ToggleBtn {
                        label: "Enabled",
                        active: *anim_enabled.read(),
                        on_click: move |_| anim_enabled.set(true),
                    }
                    ToggleBtn {
                        label: "Disabled",
                        active: !*anim_enabled.read(),
                        on_click: move |_| anim_enabled.set(false),
                    }
                }
            }
        }
    }
}

// ── Sub-components ─────────────────────────────────────────────────────────────

#[component]
fn label_heading(label: &'static str) -> Element {
    rsx! {
        span {
            style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                    letter-spacing: 0.06em; color: var(--fs-color-text-muted);",
            "{label}"
        }
    }
}

#[component]
fn RadioOption(
    id:        &'static str,
    label:     &'static str,
    group:     &'static str,
    checked:   bool,
    on_change: EventHandler<()>,
) -> Element {
    rsx! {
        label {
            style: "display: flex; align-items: center; gap: 10px; cursor: pointer; \
                    font-size: 13px; color: var(--fs-color-text-primary);",
            input {
                r#type: "radio",
                name: "{group}",
                value: "{id}",
                checked,
                onchange: move |_| on_change.call(()),
            }
            "{label}"
        }
    }
}

#[component]
fn ToggleBtn(
    label:    &'static str,
    active:   bool,
    on_click: EventHandler<()>,
) -> Element {
    let (bg, color, border) = if active {
        (
            "var(--fs-color-primary, #00bcd4)",
            "#fff",
            "border: 1px solid var(--fs-color-primary, #00bcd4);",
        )
    } else {
        (
            "transparent",
            "var(--fs-color-text-muted)",
            "border: 1px solid var(--fs-color-border-default, #334155);",
        )
    };

    rsx! {
        button {
            style: "padding: 6px 18px; background: {bg}; color: {color}; \
                    {border} border-radius: var(--fs-radius-md, 6px); \
                    font-size: 13px; cursor: pointer;",
            onclick: move |_| on_click.call(()),
            "{label}"
        }
    }
}
