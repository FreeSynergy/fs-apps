/// Per-instance configuration — stored at ~/.local/share/fsn/services/<name>/instance.toml
///
/// Each deployed service instance writes this file so the Conductor can display
/// and let the user edit instance-specific settings without re-running the wizard.
///
/// Structure:
///   [instance]   – which module class + version + image override
///   [vars]       – user-set values for the module's [vars] template variables
///   [subservices] – which sub-services are enabled
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ── InstanceConfig ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceConfig {
    #[serde(default)]
    pub instance: InstanceMeta,

    /// User-provided values for the module's [vars] template variables.
    #[serde(default)]
    pub vars: HashMap<String, String>,

    /// Sub-service name → enabled flag.
    /// Absent = use module default (usually true).
    #[serde(default)]
    pub subservices: HashMap<String, bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceMeta {
    /// The module class path, e.g. "auth/kanidm".
    pub module_class: String,

    /// Module version at install time.
    pub module_version: String,

    /// Container image tag override (empty = use module default).
    #[serde(default)]
    pub image_tag: String,
}

// ── Persistence ────────────────────────────────────────────────────────────────

/// Return the path for a service's instance.toml.
pub fn instance_config_path(service_name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".local/share/fsn/services")
        .join(service_name)
        .join("instance.toml")
}

/// Load the instance config for a service.
///
/// Returns a default (empty) config when the file doesn't exist.
pub fn load_instance_config(service_name: &str) -> InstanceConfig {
    let path = instance_config_path(service_name);
    match std::fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => InstanceConfig::default(),
    }
}

/// Save the instance config for a service.
pub fn save_instance_config(service_name: &str, config: &InstanceConfig) -> Result<(), String> {
    let path = instance_config_path(service_name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

// ── InstanceConfigEditor (Dioxus component) ────────────────────────────────────

use dioxus::prelude::*;

/// Editor panel for an instance's configuration.
///
/// Shows module class, image tag, template variables, and sub-service toggles.
/// Reads from / writes to `~/.local/share/fsn/services/<name>/instance.toml`.
#[component]
pub fn InstanceConfigEditor(service_name: String) -> Element {
    let mut config: Signal<InstanceConfig> = use_signal(|| load_instance_config(&service_name));
    let mut save_msg: Signal<Option<String>> = use_signal(|| None);

    let svc = service_name.clone();
    let save = move |_| {
        match save_instance_config(&svc, &config.read()) {
            Ok(()) => save_msg.set(Some("Saved.".into())),
            Err(e) => save_msg.set(Some(format!("Save failed: {e}"))),
        }
    };

    rsx! {
        div {
            // ── Header row ─────────────────────────────────────────────────────
            div {
                style: "display: flex; align-items: center; justify-content: space-between; \
                        margin-bottom: 16px;",
                span { style: "font-size: 13px; font-weight: 600;", "Module Configuration" }
                button {
                    style: "padding: 4px 12px; background: var(--fs-color-primary); \
                            color: white; border: none; border-radius: var(--fs-radius-md); \
                            cursor: pointer; font-size: 12px;",
                    onclick: save,
                    "Save"
                }
            }

            if let Some(msg) = save_msg.read().as_deref() {
                div {
                    style: "margin-bottom: 10px; padding: 6px 10px; \
                            background: var(--fs-color-bg-elevated); \
                            border-radius: var(--fs-radius-md); \
                            font-size: 12px; color: var(--fs-color-text-muted);",
                    "{msg}"
                }
            }

            // ── Module metadata ────────────────────────────────────────────────
            SectionLabel { label: "Module" }
            ConfigRow {
                label: "Class",
                value: config.read().instance.module_class.clone(),
                placeholder: "e.g. auth/kanidm",
                on_change: move |v| config.write().instance.module_class = v,
            }
            ConfigRow {
                label: "Image Tag",
                value: config.read().instance.image_tag.clone(),
                placeholder: "latest",
                on_change: move |v| config.write().instance.image_tag = v,
            }

            // ── Template Variables ─────────────────────────────────────────────
            if !config.read().vars.is_empty() {
                SectionLabel { label: "Variables" }
                div {
                    for (key, val) in config.read().vars.clone() {
                        ConfigRow {
                            key: "{key}",
                            label: key.clone(),
                            value: val,
                            placeholder: "",
                            on_change: {
                                let k = key.clone();
                                move |v| { config.write().vars.insert(k.clone(), v); }
                            },
                        }
                    }
                }
            } else {
                div {
                    style: "font-size: 12px; color: var(--fs-color-text-muted); \
                            padding: 8px 0; margin-bottom: 8px;",
                    "No template variables defined. Add them by editing instance.toml directly, \
                     or re-run the install wizard."
                }
            }

            // ── Sub-services ───────────────────────────────────────────────────
            if !config.read().subservices.is_empty() {
                SectionLabel { label: "Sub-services" }
                div {
                    for (name, enabled) in config.read().subservices.clone() {
                        SubServiceToggle {
                            key: "{name}",
                            name: name.clone(),
                            enabled,
                            on_toggle: {
                                let n = name.clone();
                                move |v| { config.write().subservices.insert(n.clone(), v); }
                            },
                        }
                    }
                }
            }
        }
    }
}

// ── Helper components ──────────────────────────────────────────────────────────

#[component]
fn SectionLabel(label: String) -> Element {
    rsx! {
        div {
            style: "font-size: 11px; font-weight: 600; letter-spacing: 0.06em; \
                    text-transform: uppercase; color: var(--fs-color-text-muted); \
                    margin: 12px 0 6px;",
            "{label}"
        }
    }
}

#[component]
fn ConfigRow(
    label:       String,
    value:       String,
    placeholder: String,
    on_change:   EventHandler<String>,
) -> Element {
    rsx! {
        div {
            style: "display: grid; grid-template-columns: 140px 1fr; \
                    align-items: center; gap: 8px; margin-bottom: 6px;",
            span {
                style: "font-size: 12px; color: var(--fs-color-text-muted); \
                        font-family: monospace; word-break: break-all;",
                "{label}"
            }
            input {
                r#type: "text",
                style: "padding: 4px 8px; font-size: 12px; \
                        background: var(--fs-color-bg-elevated); \
                        border: 1px solid var(--fs-color-border-default); \
                        border-radius: var(--fs-radius-sm); \
                        color: var(--fs-color-text-primary); \
                        font-family: monospace;",
                value: "{value}",
                placeholder: "{placeholder}",
                oninput: move |e| on_change.call(e.value()),
            }
        }
    }
}

#[component]
fn SubServiceToggle(
    name:      String,
    enabled:   bool,
    on_toggle: EventHandler<bool>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 10px; \
                    padding: 6px 0; font-size: 13px;",
            input {
                r#type: "checkbox",
                checked: enabled,
                onchange: move |e| on_toggle.call(e.checked()),
            }
            span { style: "font-family: monospace; font-size: 12px;", "{name}" }
        }
    }
}
