// AI Manager UI
//
// Layout:
//   Left sidebar  — installed AI engines (icon + name + status dot)
//   Right panel   — selected engine: status badge, model dropdown, start/stop button
//
// On successful start: writes ~/.continue/config.json automatically.

use dioxus::prelude::*;
use fs_manager_ai::{AiEngine, EngineStatus, LlmConfig, LlmEngine, LlmModel};

// ── ModelConfig ───────────────────────────────────────────────────────────────

/// Holds per-model configuration and acts as a factory for [`LlmEngine`].
struct ModelConfig {
    model: LlmModel,
}

impl ModelConfig {
    fn for_model(model: LlmModel) -> Self {
        Self { model }
    }

    fn engine(&self) -> LlmEngine {
        LlmEngine::new(
            LlmConfig { model: self.model.clone(), ..LlmConfig::default() },
            LlmEngine::default_binary(),
            LlmEngine::default_data_dir(),
        )
    }
}

// ── AiManagerApp ─────────────────────────────────────────────────────────────

#[component]
pub fn AiManagerApp() -> Element {
    let mut selected_model = use_signal(|| LlmModel::Qwen3_4B);
    let mut status         = use_signal(|| ModelConfig::for_model(LlmModel::Qwen3_4B).engine().status());
    let mut feedback       = use_signal(String::new);

    let mut do_refresh = move || {
        status.set(ModelConfig::for_model(selected_model.read().clone()).engine().status());
    };

    rsx! {
        div {
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; background: var(--fs-color-bg-base);",

            // ── Sidebar ───────────────────────────────────────────────────────
            div {
                style: "width: 220px; flex-shrink: 0; background: var(--fs-color-bg-surface); \
                        border-right: 1px solid var(--fs-color-border); \
                        display: flex; flex-direction: column; padding: 16px 0;",

                div {
                    style: "padding: 0 16px 12px; font-size: 11px; font-weight: 600; \
                            letter-spacing: 0.08em; text-transform: uppercase; color: var(--fs-color-text-muted);",
                    "Engines"
                }

                div {
                    style: "display: flex; align-items: center; gap: 10px; \
                            padding: 10px 16px; background: var(--fs-color-bg-overlay); \
                            border-left: 3px solid var(--fs-color-primary);",
                    div {
                        style: format!(
                            "width: 8px; height: 8px; border-radius: 50%; \
                             background: {};",
                            if status.read().is_running() {
                                "var(--fs-color-success)"
                            } else {
                                "var(--fs-color-text-muted)"
                            }
                        ),
                    }
                    span { style: "color: var(--fs-color-text-primary); font-size: 14px;", "Mistral.rs" }
                }
            }

            // ── Detail panel ──────────────────────────────────────────────────
            div {
                style: "flex: 1; overflow-y: auto; padding: 32px;",

                h2 {
                    style: "margin: 0 0 4px; font-size: 20px; font-weight: 600; color: var(--fs-color-primary);",
                    "Mistral.rs"
                }
                p {
                    style: "margin: 0 0 24px; font-size: 13px; color: #94a3b8;",
                    "High-performance LLM inference engine — OpenAI-compatible API"
                }

                // Status card
                div {
                    style: "background: #1e293b; border: 1px solid #334155; \
                            border-radius: 8px; padding: 20px; margin-bottom: 20px;",

                    div {
                        style: "display: flex; align-items: center; gap: 12px; margin-bottom: 8px;",
                        span { style: "font-size: 13px; font-weight: 500; color: #94a3b8;", "Status" }
                        StatusBadge { status: status.read().clone() }
                    }

                    if let EngineStatus::Running { port } = *status.read() {
                        p {
                            style: "margin: 0; font-size: 12px; color: #94a3b8;",
                            "Listening on http://127.0.0.1:{port}/v1  ·  Continue.dev configured ✓"
                        }
                    }

                    if !make_engine(LlmModel::Qwen3_4B).is_installed() {
                        p {
                            style: "margin: 8px 0 0; font-size: 12px; color: #eab308;",
                            "Binary not found — install via: fsn store install mistral"
                        }
                    }
                }

                // Model selection
                div {
                    style: "background: #1e293b; border: 1px solid #334155; \
                            border-radius: 8px; padding: 20px; margin-bottom: 20px;",

                    label {
                        style: "display: block; font-size: 13px; font-weight: 500; \
                                color: #94a3b8; margin-bottom: 10px;",
                        "Model"
                    }

                    select {
                        style: "width: 100%; padding: 9px 12px; border-radius: 6px; \
                                background: #0c1222; border: 1px solid #334155; \
                                color: #e2e8f0; font-size: 14px; cursor: pointer; outline: none;",
                        disabled: status.read().is_running(),
                        onchange: move |e: Event<FormData>| {
                            selected_model.set(LlmModel::from_hf_id(&e.value()));
                        },

                        for model in LlmModel::all_predefined() {
                            option {
                                value: model.hf_id(),
                                selected: *selected_model.read() == model,
                                "{model.display_name()}"
                            }
                        }
                    }

                    {
                        let ram = selected_model.read().ram_gb();
                        if ram > 0.0 {
                            rsx! {
                                p {
                                    style: "margin: 10px 0 0; font-size: 12px; color: #94a3b8;",
                                    "RAM after ISQ Q4K: ~{ram} GB  ·  Port: 1234"
                                }
                            }
                        } else {
                            rsx! { span {} }
                        }
                    }
                }

                // Actions
                div {
                    style: "display: flex; gap: 12px; margin-bottom: 20px;",

                    if !status.read().is_running() {
                        button {
                            style: "padding: 10px 24px; border-radius: 6px; border: none; \
                                    background: #00d9ff; color: #0c1222; font-weight: 600; \
                                    font-size: 14px; cursor: pointer;",
                            onclick: move |_| {
                                feedback.set(String::new());
                                let engine = make_engine(selected_model.read().clone());
                                match engine.start() {
                                    Ok(()) => {
                                        let _ = engine.write_continue_config();
                                        feedback.set(
                                            "Starting… model download may take a few minutes on first run.".into()
                                        );
                                    }
                                    Err(e) => feedback.set(format!("Error: {e}")),
                                }
                                do_refresh();
                            },
                            "Start"
                        }
                    } else {
                        button {
                            style: "padding: 10px 24px; border-radius: 6px; border: none; \
                                    background: #ef4444; color: #fff; font-weight: 600; \
                                    font-size: 14px; cursor: pointer;",
                            onclick: move |_| {
                                let engine = make_engine(selected_model.read().clone());
                                match engine.stop() {
                                    Ok(()) => feedback.set("Stopped.".into()),
                                    Err(e) => feedback.set(format!("Error: {e}")),
                                }
                                do_refresh();
                            },
                            "Stop"
                        }
                    }

                    button {
                        style: "padding: 10px 20px; border-radius: 6px; \
                                background: transparent; border: 1px solid #334155; \
                                color: #94a3b8; font-size: 14px; cursor: pointer;",
                        onclick: move |_| do_refresh(),
                        "Refresh"
                    }
                }

                // Feedback
                if !feedback.read().is_empty() {
                    div {
                        style: "padding: 12px 16px; border-radius: 6px; \
                                background: #1e293b; border: 1px solid #334155; \
                                font-size: 13px; color: #94a3b8;",
                        "{feedback}"
                    }
                }

                // Editor integration hint
                div {
                    style: "background: #1e293b; border: 1px solid #334155; \
                            border-radius: 8px; padding: 16px; margin-top: 8px;",
                    p {
                        style: "margin: 0 0 6px; font-size: 13px; font-weight: 500; color: #e2e8f0;",
                        "Editor Integration"
                    }
                    p {
                        style: "margin: 0; font-size: 12px; color: #94a3b8; line-height: 1.6;",
                        "Continue.dev config is written to ~/.continue/config.json automatically on Start. \
                         Install the Continue extension in VSCode / VSCodium to use the local model."
                    }
                }
            }
        }
    }
}

// ── StatusBadge ───────────────────────────────────────────────────────────────

#[component]
fn StatusBadge(status: EngineStatus) -> Element {
    let (label, color, bg) = match &status {
        EngineStatus::Running { .. } => ("Running", "#22c55e", "#14532d22"),
        EngineStatus::Stopped        => ("Stopped", "#64748b", "#1e293b"),
        EngineStatus::Error(_)       => ("Error",   "#ef4444", "#7f1d1d22"),
    };
    rsx! {
        span {
            style: "padding: 3px 10px; border-radius: 12px; font-size: 12px; \
                    font-weight: 500; color: {color}; background: {bg}; \
                    border: 1px solid {color}44;",
            "{label}"
        }
    }
}
