/// Themes view — browse and activate available themes.
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct ThemeEntry {
    name:  &'static str,
    color: &'static str,
}

const DEMO_THEMES: &[ThemeEntry] = &[
    ThemeEntry { name: "Midnight Blue", color: "#0f172a" },
    ThemeEntry { name: "Cloud White",   color: "#f8fafc" },
    ThemeEntry { name: "Nordic Dark",   color: "#2e3440" },
];

#[component]
pub fn ThemesView() -> Element {
    let mut active_idx = use_signal(|| 0usize);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",

            h3 { style: "margin: 0 0 4px; font-size: 16px; color: var(--fs-color-text-primary);",
                "Themes"
            }

            div { style: "display: flex; flex-wrap: wrap; gap: 16px;",
                for (idx, theme) in DEMO_THEMES.iter().enumerate() {
                    ThemeCard {
                        key: "{theme.name}",
                        name: theme.name,
                        color: theme.color,
                        is_active: *active_idx.read() == idx,
                        on_activate: move |_| active_idx.set(idx),
                    }
                }
            }
        }
    }
}

#[component]
fn ThemeCard(
    name:        &'static str,
    color:       &'static str,
    is_active:   bool,
    on_activate: EventHandler<()>,
) -> Element {
    let border = if is_active {
        "border: 2px solid var(--fs-color-primary, #00bcd4);"
    } else {
        "border: 2px solid var(--fs-color-border-default, #334155);"
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; gap: 12px; \
                    padding: 20px; border-radius: var(--fs-radius-md, 8px); \
                    background: var(--fs-color-bg-surface, #1e293b); \
                    width: 160px; cursor: pointer; \
                    {border}",
            onclick: move |_| on_activate.call(()),

            // Color circle
            div {
                style: "width: 48px; height: 48px; border-radius: 50%; \
                        background: {color}; \
                        border: 2px solid rgba(255,255,255,0.1);",
            }

            // Theme name
            span {
                style: "font-size: 13px; font-weight: 600; \
                        color: var(--fs-color-text-primary);",
                "{name}"
            }

            // Badge or button
            if is_active {
                span {
                    style: "font-size: 11px; padding: 3px 10px; \
                            border-radius: 999px; \
                            background: var(--fs-color-primary, #00bcd4); \
                            color: #fff; font-weight: 600;",
                    "Active"
                }
            } else {
                button {
                    style: "font-size: 12px; padding: 4px 14px; \
                            border-radius: var(--fs-radius-md, 6px); \
                            background: transparent; \
                            border: 1px solid var(--fs-color-border-default, #334155); \
                            color: var(--fs-color-text-muted); cursor: pointer;",
                    onclick: move |e| { e.stop_propagation(); on_activate.call(()); },
                    "Activate"
                }
            }
        }
    }
}
