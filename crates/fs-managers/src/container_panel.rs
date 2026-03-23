/// Container App Manager panel — lists installed apps with status and controls.
use dioxus::prelude::*;
use fs_i18n;
use fs_manager_container::{AppStatus, ContainerManager};

#[component]
pub fn ContainerManagerPanel() -> Element {
    let mgr       = ContainerManager::with_noop();
    let installed = use_signal(|| mgr.installed());

    rsx! {
        div {
            style: "padding: 24px; max-width: 560px;",

            h3 { style: "margin-top: 0; color: var(--fs-text-primary);",
                {fs_i18n::t("managers.containers.title")}
            }
            p { style: "font-size: 13px; color: var(--fs-color-text-muted); margin-top: -8px;",
                {fs_i18n::t("managers.containers.description")}
            }

            if installed.read().is_empty() {
                div {
                    style: "padding: 32px; text-align: center; \
                            color: var(--fs-color-text-muted); font-size: 13px; \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md);",
                    span { style: "display: block; font-size: 32px; margin-bottom: 12px;", "📦" }
                    {fs_i18n::t("managers.containers.empty")}
                }
            } else {
                div {
                    style: "border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); overflow: hidden;",
                    for app in installed.read().clone() {
                        {
                            let status_label = match &app.status {
                                AppStatus::Error(msg) => msg.clone(),
                                other => other.label().to_string(),
                            };
                            let status_color = app.status.css_color();
                            let app_id_start = app.id.clone();
                            let app_id_stop  = app.id.clone();
                            let app_id_remove = app.id.clone();
                            rsx! {
                                div {
                                    style: "display: flex; align-items: center; gap: 14px; \
                                            padding: 12px 16px; \
                                            border-bottom: 1px solid var(--fs-color-border-default); \
                                            color: var(--fs-color-text-primary);",
                                    span { style: "font-size: 22px;", "📦" }
                                    div { style: "flex: 1;",
                                        div { style: "font-size: 14px; font-weight: 500;", "{app.name}" }
                                        div { style: "font-size: 11px; color: var(--fs-color-text-muted);",
                                            "v{app.version}"
                                        }
                                    }
                                    span {
                                        style: "font-size: 11px; padding: 2px 8px; \
                                                border-radius: 999px; \
                                                background: var(--fs-color-bg-overlay); \
                                                color: {status_color};",
                                        "{status_label}"
                                    }
                                    // Start/Stop button
                                    if matches!(app.status, AppStatus::Stopped) {
                                        button {
                                            style: "padding: 4px 10px; font-size: 12px; \
                                                    background: var(--fs-color-primary, #06b6d4); \
                                                    color: white; border: none; \
                                                    border-radius: var(--fs-radius-sm, 4px); \
                                                    cursor: pointer;",
                                            onclick: move |_| {
                                                let mgr = ContainerManager::with_noop();
                                                let _ = mgr.start(&app_id_start);
                                            },
                                            {fs_i18n::t("actions.start")}
                                        }
                                    }
                                    if matches!(app.status, AppStatus::Running) {
                                        button {
                                            style: "padding: 4px 10px; font-size: 12px; \
                                                    background: transparent; \
                                                    border: 1px solid var(--fs-color-border-default); \
                                                    border-radius: var(--fs-radius-sm, 4px); \
                                                    cursor: pointer; color: var(--fs-color-text-muted);",
                                            onclick: move |_| {
                                                let mgr = ContainerManager::with_noop();
                                                let _ = mgr.stop(&app_id_stop);
                                            },
                                            {fs_i18n::t("actions.stop")}
                                        }
                                    }
                                    // Remove button
                                    button {
                                        style: "padding: 4px 8px; font-size: 12px; \
                                                background: transparent; \
                                                border: 1px solid var(--fs-color-border-default); \
                                                border-radius: var(--fs-radius-sm, 4px); \
                                                cursor: pointer; color: var(--fs-color-text-muted); \
                                                opacity: 0.6;",
                                        title: "Remove",
                                        onclick: move |_| {
                                            let mgr = ContainerManager::with_noop();
                                            let _ = mgr.remove(&app_id_remove);
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
