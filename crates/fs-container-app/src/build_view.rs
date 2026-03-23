/// Build & Publish view — paste docker-compose YAML, analyze, publish to Store.
use dioxus::prelude::*;

#[component]
pub fn BuildView() -> Element {
    let mut yaml_input = use_signal(String::new);
    let mut analyzed   = use_signal(|| false);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 20px; max-width: 800px;",

            // Header
            div {
                h3 { style: "margin: 0 0 4px; font-size: 16px; color: var(--fs-color-text-primary);",
                    "Build & Publish"
                }
                p { style: "margin: 0; font-size: 13px; color: var(--fs-color-text-muted);",
                    "Paste a docker-compose.yml to analyze and publish as a FreeSynergy package."
                }
            }

            // YAML input
            div {
                label {
                    style: "display: block; font-size: 12px; font-weight: 600; \
                            color: var(--fs-color-text-muted); margin-bottom: 6px; \
                            text-transform: uppercase; letter-spacing: 0.06em;",
                    "docker-compose.yml"
                }
                textarea {
                    style: "width: 100%; min-height: 200px; \
                            background: var(--fs-color-bg-overlay, #0f172a); \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md, 6px); \
                            padding: 12px; font-size: 13px; font-family: monospace; \
                            color: var(--fs-color-text-primary); resize: vertical; \
                            box-sizing: border-box;",
                    placeholder: "version: '3'\nservices:\n  app:\n    image: myapp:latest\n    ports:\n      - 8080:8080",
                    oninput: move |e| {
                        yaml_input.set(e.value());
                        analyzed.set(false);
                    },
                }
            }

            // Analyze button
            div { style: "display: flex; gap: 10px; align-items: center;",
                button {
                    style: "background: var(--fs-color-primary, #06b6d4); color: #fff; \
                            border: none; border-radius: var(--fs-radius-md, 6px); \
                            padding: 10px 20px; font-size: 14px; font-weight: 600; cursor: pointer;",
                    disabled: yaml_input.read().trim().is_empty(),
                    onclick: move |_| {
                        if !yaml_input.read().trim().is_empty() {
                            analyzed.set(true);
                        }
                    },
                    "🔍 Analyze"
                }
            }

            // Results area
            div {
                style: "background: var(--fs-color-bg-surface, #1e293b); \
                        border: 1px solid var(--fs-color-border-default); \
                        border-radius: var(--fs-radius-md, 6px); \
                        padding: 20px; min-height: 120px;",

                if *analyzed.read() {
                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                        p { style: "margin: 0; font-size: 13px; color: var(--fs-color-text-muted);",
                            "Analysis results will appear here"
                        }
                        p { style: "margin: 0; font-size: 12px; color: var(--fs-color-text-muted); opacity: 0.7;",
                            "Services detected, port mappings, volume mounts, and resource requirements will be shown."
                        }
                    }
                } else {
                    p {
                        style: "margin: 0; font-size: 13px; color: var(--fs-color-text-muted); \
                                font-style: italic;",
                        "Paste your docker-compose.yml above and click Analyze to see results."
                    }
                }
            }

            // Publish button (disabled until analyzed)
            button {
                style: {
                    let opacity = if *analyzed.read() { "1" } else { "0.4" };
                    format!(
                        "align-self: flex-start; \
                         background: var(--fs-color-accent, #7c3aed); color: #fff; \
                         border: none; border-radius: var(--fs-radius-md, 6px); \
                         padding: 10px 24px; font-size: 14px; font-weight: 600; \
                         cursor: {}; opacity: {opacity};",
                        if *analyzed.read() { "pointer" } else { "not-allowed" }
                    )
                },
                disabled: !*analyzed.read(),
                "🚀 Publish to Store"
            }
        }
    }
}
