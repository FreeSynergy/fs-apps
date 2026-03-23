/// Service detail panel — shown when a service is selected in the Services list.
///
/// Displays: service status, start/stop/restart actions, environment variables
/// editor (.env file), module instance config, and a mini log tail.
use dioxus::prelude::*;
use fs_container::{SystemctlManager, UnitActiveState};
use fs_i18n;

use crate::instance_config::InstanceConfigEditor;
use crate::log_viewer::LogViewer;
use crate::service_list::ServiceAction;

// ── ServiceDetail ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum DetailTab {
    Config,
    Module,
    Logs,
}

/// Full detail panel for a single service.
#[component]
pub fn ServiceDetail(service_name: String, on_close: EventHandler<()>) -> Element {
    let mut tab = use_signal(|| DetailTab::Config);

    rsx! {
        div {
            class: "fs-service-detail",
            style: "display: flex; flex-direction: column; height: 100%; \
                    border-left: 1px solid var(--fs-border); \
                    background: var(--fs-color-bg-base);",

            // ── Header ─────────────────────────────────────────────────────────
            div {
                style: "display: flex; align-items: center; justify-content: space-between; \
                        padding: 12px 16px; border-bottom: 1px solid var(--fs-border); \
                        background: var(--fs-bg-surface); flex-shrink: 0;",
                div {
                    style: "display: flex; align-items: center; gap: 10px;",
                    button {
                        style: "background: none; border: none; cursor: pointer; \
                                color: var(--fs-text-muted); font-size: 18px; padding: 0 4px;",
                        title: "Close detail",
                        onclick: move |_| on_close.call(()),
                        "‹"
                    }
                    span {
                        style: "font-weight: 600; font-size: 14px;",
                        "{service_name}"
                    }
                }
                ServiceStatusBadge { service_name: service_name.clone() }
            }

            // ── Tab bar ────────────────────────────────────────────────────────
            div {
                style: "display: flex; border-bottom: 1px solid var(--fs-border); flex-shrink: 0;",
                DetailTabBtn {
                    label: fs_i18n::t("container.tab.config"),
                    active: *tab.read() == DetailTab::Config,
                    onclick: move |_| tab.set(DetailTab::Config),
                }
                DetailTabBtn {
                    label: fs_i18n::t("container.tab.module"),
                    active: *tab.read() == DetailTab::Module,
                    onclick: move |_| tab.set(DetailTab::Module),
                }
                DetailTabBtn {
                    label: fs_i18n::t("container.tab.logs"),
                    active: *tab.read() == DetailTab::Logs,
                    onclick: move |_| tab.set(DetailTab::Logs),
                }
            }

            // ── Tab content ────────────────────────────────────────────────────
            div {
                style: "flex: 1; overflow: auto;",
                match *tab.read() {
                    DetailTab::Config => rsx! {
                        ServiceConfigTab { service_name: service_name.clone() }
                    },
                    DetailTab::Module => rsx! {
                        div { style: "padding: 16px;",
                            InstanceConfigEditor { service_name: service_name.clone() }
                        }
                    },
                    DetailTab::Logs => rsx! {
                        LogViewer { service: service_name.clone() }
                    },
                }
            }
        }
    }
}

// ── ServiceStatusBadge ─────────────────────────────────────────────────────────

/// Small live status badge (polls systemctl every 5s).
#[component]
fn ServiceStatusBadge(service_name: String) -> Element {
    let mut state: Signal<UnitActiveState> = use_signal(|| UnitActiveState::Unknown);

    {
        let name = service_name.clone();
        use_future(move || {
            let name = name.clone();
            async move {
                let mgr = SystemctlManager::user();
                loop {
                    if let Ok(s) = mgr.service_status(&name).await {
                        state.set(s.active_state);
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });
    }

    let (label, color, bg) = match *state.read() {
        UnitActiveState::Active       => (fs_i18n::t("status.running"),  "var(--fs-success)",      "rgba(34,197,94,0.1)"),
        UnitActiveState::Inactive     => (fs_i18n::t("status.stopped"),  "var(--fs-text-muted)",   "var(--fs-bg-elevated)"),
        UnitActiveState::Activating   => (fs_i18n::t("status.starting"), "var(--fs-info)",         "rgba(99,179,237,0.1)"),
        UnitActiveState::Deactivating => (fs_i18n::t("status.stopping"), "var(--fs-warning)",      "rgba(251,191,36,0.1)"),
        UnitActiveState::Failed       => (fs_i18n::t("status.failed"),   "var(--fs-error)",        "rgba(239,68,68,0.1)"),
        UnitActiveState::Unknown      => (fs_i18n::t("status.unknown"),  "var(--fs-text-muted)",   "var(--fs-bg-elevated)"),
    };

    rsx! {
        span {
            style: "padding: 3px 10px; border-radius: 999px; font-size: 12px; \
                    color: {color}; background: {bg}; border: 1px solid {color};",
            "{label}"
        }
    }
}

// ── ServiceConfigTab ───────────────────────────────────────────────────────────

/// Config tab: action buttons + .env editor.
#[component]
fn ServiceConfigTab(service_name: String) -> Element {
    let action_msg: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div { style: "padding: 16px;",

            // Action buttons
            div {
                style: "display: flex; gap: 8px; margin-bottom: 20px;",
                ActionBtn {
                    label: fs_i18n::t("actions.start"),
                    color: "var(--fs-success)",
                    name: service_name.clone(),
                    action: ServiceAction::Start,
                    msg: action_msg,
                }
                ActionBtn {
                    label: fs_i18n::t("actions.stop"),
                    color: "var(--fs-error)",
                    name: service_name.clone(),
                    action: ServiceAction::Stop,
                    msg: action_msg,
                }
                ActionBtn {
                    label: fs_i18n::t("actions.restart"),
                    color: "var(--fs-warning)",
                    name: service_name.clone(),
                    action: ServiceAction::Restart,
                    msg: action_msg,
                }
            }

            // Action feedback
            if let Some(msg) = action_msg.read().as_deref() {
                div {
                    style: "margin-bottom: 12px; padding: 8px 12px; \
                            background: var(--fs-bg-elevated); \
                            border-radius: var(--fs-radius-md); \
                            font-size: 12px; color: var(--fs-text-muted);",
                    "{msg}"
                }
            }

            // Env vars editor
            EnvEditor { service_name: service_name.clone() }
        }
    }
}

// ── ActionBtn ─────────────────────────────────────────────────────────────────

#[component]
fn ActionBtn(
    label:  String,
    color:  String,
    name:   String,
    action: ServiceAction,
    mut msg: Signal<Option<String>>,
) -> Element {
    rsx! {
        button {
            style: "padding: 6px 14px; background: {color}; color: white; \
                    border: none; border-radius: var(--fs-radius-md); \
                    cursor: pointer; font-size: 13px;",
            onclick: {
                let name   = name.clone();
                let action = action.clone();
                let label  = label.clone();
                move |_| {
                    let name   = name.clone();
                    let action = action.clone();
                    let label  = label.clone();
                    spawn(async move {
                        let mgr = SystemctlManager::user();
                        match action.execute(&mgr, &name).await {
                            Ok(()) => msg.set(Some(format!("{label} OK"))),
                            Err(e) => msg.set(Some(format!("{label} failed: {e}"))),
                        }
                    });
                }
            },
            "{label}"
        }
    }
}

// ── EnvEditor ─────────────────────────────────────────────────────────────────

/// Reads ~/.local/share/fsn/services/<name>/.env and lets the user edit + save it.
#[component]
fn EnvEditor(service_name: String) -> Element {
    let env_path = {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        std::path::PathBuf::from(home)
            .join(".local/share/fsn/services")
            .join(&service_name)
            .join(".env")
    };

    let env_path_str = env_path.to_string_lossy().into_owned();

    let mut content: Signal<String>       = use_signal(String::new);
    let mut loaded:  Signal<bool>         = use_signal(|| false);
    let mut save_msg: Signal<Option<String>> = use_signal(|| None);

    // Load once
    {
        let path = env_path.clone();
        use_effect(move || {
            if !*loaded.read() {
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                content.set(text);
                loaded.set(true);
            }
        });
    }

    rsx! {
        div {
            div {
                style: "display: flex; align-items: center; justify-content: space-between; \
                        margin-bottom: 8px;",
                span {
                    style: "font-size: 13px; font-weight: 600;",
                    {fs_i18n::t("container.env.heading")}
                }
                button {
                    style: "padding: 4px 12px; background: var(--fs-color-primary); \
                            color: white; border: none; border-radius: var(--fs-radius-md); \
                            cursor: pointer; font-size: 12px;",
                    onclick: {
                        let path = env_path.clone();
                        move |_| {
                            let path    = path.clone();
                            let to_save = content.read().clone();
                            match std::fs::write(&path, &to_save) {
                                Ok(()) => {
                                    save_msg.set(Some(fs_i18n::t("container.env.saved_hint").to_string()));
                                }
                                Err(e) => {
                                    save_msg.set(Some(format!("Save failed: {e}")));
                                }
                            }
                        }
                    },
                    {fs_i18n::t("actions.save")}
                }
            }

            p { style: "font-size: 11px; color: var(--fs-text-muted); margin-bottom: 8px;",
                "{env_path_str}"
            }

            if let Some(msg) = save_msg.read().as_deref() {
                div {
                    style: "margin-bottom: 8px; padding: 6px 10px; \
                            background: var(--fs-bg-elevated); \
                            border-radius: var(--fs-radius-md); \
                            font-size: 12px; color: var(--fs-text-muted);",
                    "{msg}"
                }
            }

            textarea {
                style: "width: 100%; min-height: 220px; padding: 10px 12px; \
                        font-family: monospace; font-size: 12px; \
                        background: var(--fs-color-bg-elevated); \
                        border: 1px solid var(--fs-color-border-default); \
                        border-radius: var(--fs-radius-md); \
                        color: var(--fs-color-text-primary); \
                        resize: vertical; box-sizing: border-box;",
                placeholder: "KEY=value\nANOTHER_KEY=value",
                value: "{content.read()}",
                oninput: move |e| content.set(e.value()),
            }

            p { style: "font-size: 11px; color: var(--fs-text-muted); margin-top: 6px;",
                {fs_i18n::t("container.env.restart_hint")}
            }
        }
    }
}

// ── DetailTabBtn ───────────────────────────────────────────────────────────────

#[component]
fn DetailTabBtn(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let bg     = if active { "var(--fs-bg-elevated)" } else { "transparent" };
    let border = if active {
        "border-bottom: 2px solid var(--fs-color-primary);"
    } else {
        "border-bottom: 2px solid transparent;"
    };
    let color  = if active { "var(--fs-text-primary)" } else { "var(--fs-text-muted)" };

    rsx! {
        button {
            style: "padding: 8px 16px; background: {bg}; border: none; \
                    {border} cursor: pointer; font-size: 13px; color: {color};",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}
