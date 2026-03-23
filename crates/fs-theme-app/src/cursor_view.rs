/// Cursor view — select mouse cursor style.
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct CursorOption {
    id:          &'static str,
    name:        &'static str,
    description: &'static str,
}

const CURSOR_OPTIONS: &[CursorOption] = &[
    CursorOption { id: "system",     name: "System Default",      description: "Use the operating system cursor." },
    CursorOption { id: "freesynergy",name: "FreeSynergy",         description: "Custom FreeSynergy cursor set." },
    CursorOption { id: "minimal",    name: "Minimal",             description: "Clean, minimal cursor design." },
    CursorOption { id: "retro",      name: "Retro",               description: "Classic retro-style cursor." },
];

#[component]
pub fn CursorView() -> Element {
    let mut selected = use_signal(|| "system");

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px; max-width: 480px;",

            h3 { style: "margin: 0 0 4px; font-size: 16px; color: var(--fs-color-text-primary);",
                "Mouse Cursor"
            }

            div { style: "display: flex; flex-direction: column; gap: 10px;",
                for opt in CURSOR_OPTIONS.iter() {
                    CursorCard {
                        key: "{opt.id}",
                        option: opt,
                        is_selected: *selected.read() == opt.id,
                        on_select: move |id: &'static str| selected.set(id),
                    }
                }
            }
        }
    }
}

#[component]
fn CursorCard(
    option:      &'static CursorOption,
    is_selected: bool,
    on_select:   EventHandler<&'static str>,
) -> Element {
    let border = if is_selected {
        "border: 2px solid var(--fs-color-primary, #00bcd4);"
    } else {
        "border: 2px solid var(--fs-color-border-default, #334155);"
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 16px; \
                    padding: 14px 16px; border-radius: var(--fs-radius-md, 8px); \
                    background: var(--fs-color-bg-surface, #1e293b); \
                    cursor: pointer; {border}",
            onclick: move |_| on_select.call(option.id),

            span { style: "font-size: 24px; flex-shrink: 0;", "🖱" }

            div { style: "flex: 1;",
                div {
                    style: "font-size: 14px; font-weight: 600; \
                            color: var(--fs-color-text-primary); margin-bottom: 2px;",
                    "{option.name}"
                }
                div {
                    style: "font-size: 12px; color: var(--fs-color-text-muted);",
                    "{option.description}"
                }
            }

            if is_selected {
                span {
                    style: "font-size: 16px; color: var(--fs-color-primary, #00bcd4);",
                    "✓"
                }
            }
        }
    }
}
