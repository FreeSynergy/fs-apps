//! Bridge Builder — create a `BridgeResource` for a service's role API.
//!
//! Lets users select a target role, enter the service URL, and define
//! method mappings.  Validates the result and produces a resource.toml.

use dioxus::prelude::*;
use fs_types::resources::{
    bridge::{BridgeMethod, BridgeResource, FieldMapping, HttpMethod},
    meta::{ResourceMeta, ResourceType, Role, ValidationStatus},
    validator::Validate,
};
use std::path::PathBuf;

const ROLES: &[&str] = &[
    "iam", "wiki", "git", "chat", "database", "cache", "smtp", "llm", "map", "tasks", "monitoring",
];

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn BridgeBuilder() -> Element {
    let mut bridge_id      = use_signal(String::new);
    let mut bridge_name    = use_signal(String::new);
    let mut target_role    = use_signal(|| "iam".to_string());
    let mut target_service = use_signal(String::new);
    let mut export_path    = use_signal(String::new);
    let mut export_msg     = use_signal(|| Option::<String>::None);
    let mut validation     = use_signal(|| Option::<ValidationStatus>::None);

    let required_methods = use_memo(move || {
        Role::new(target_role.read().as_str())
            .required_bridge_methods()
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
    });

    rsx! {
        div {
            class: "fs-bridge-builder",
            style: "padding: 24px;",

            h3 { "Bridge Builder" }
            p { style: "color: var(--fs-color-text-muted); margin-bottom: 16px;",
                "Define a bridge that maps a standardized role API to a concrete service. \
                 The builder validates that all required methods are present."
            }

            // Bridge ID + Name
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;",
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Bridge ID" }
                    input {
                        r#type: "text",
                        placeholder: "e.g. myservice-iam-bridge",
                        value: "{bridge_id.read()}",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); font-family: var(--fs-font-mono); \
                                font-size: 13px; box-sizing: border-box;",
                        oninput: move |e| *bridge_id.write() = e.value(),
                    }
                }
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Display Name" }
                    input {
                        r#type: "text",
                        placeholder: "e.g. MyService IAM Bridge",
                        value: "{bridge_name.read()}",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); font-size: 13px; box-sizing: border-box;",
                        oninput: move |e| *bridge_name.write() = e.value(),
                    }
                }
            }

            // Target role + service
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;",
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Target Role" }
                    select {
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); font-size: 13px;",
                        onchange: move |e| *target_role.write() = e.value(),
                        for role in ROLES {
                            option { value: "{role}", "{role}" }
                        }
                    }
                }
                div {
                    label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Target Service" }
                    input {
                        r#type: "text",
                        placeholder: "e.g. myservice",
                        value: "{target_service.read()}",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); font-size: 13px; box-sizing: border-box;",
                        oninput: move |e| *target_service.write() = e.value(),
                    }
                }
            }

            // Required methods checklist
            div { style: "margin-bottom: 16px; padding: 12px 16px; \
                          background: var(--fs-color-bg-surface); \
                          border-radius: var(--fs-radius-md); \
                          border: 1px solid var(--fs-color-border-default);",
                div { style: "font-weight: 500; margin-bottom: 8px;",
                    "Required Methods for '{target_role.read()}'"
                }
                div { style: "display: flex; flex-wrap: wrap; gap: 6px;",
                    for method in required_methods.read().iter() {
                        span {
                            key: "{method}",
                            style: "padding: 3px 10px; border-radius: 999px; font-size: 11px; \
                                    font-family: var(--fs-font-mono); \
                                    background: var(--fs-color-bg-elevated); \
                                    border: 1px solid var(--fs-color-border-default);",
                            "{method}"
                        }
                    }
                }
                p { style: "font-size: 12px; color: var(--fs-color-text-muted); margin-top: 8px; margin-bottom: 0;",
                    "All these methods must be mapped in your bridge for it to validate as ✅."
                }
            }

            // Validation + export
            if let Some(status) = validation.read().as_ref() {
                div { style: "margin-bottom: 12px; display: flex; align-items: center; gap: 8px;",
                    span { style: "font-size: 20px;",
                        match status {
                            ValidationStatus::Ok         => "✅",
                            ValidationStatus::Incomplete => "⚠️",
                            ValidationStatus::Broken     => "❌",
                        }
                    }
                    span { style: "font-size: 13px; color: var(--fs-color-text-muted);",
                        match status {
                            ValidationStatus::Ok         => "Bridge is valid and ready to publish.",
                            ValidationStatus::Incomplete => "Bridge is incomplete — not all required methods are mapped.",
                            ValidationStatus::Broken     => "Bridge is broken — ID, role, or service is missing.",
                        }
                    }
                }
            }

            // Export path
            div { style: "margin-bottom: 16px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Save to directory" }
                input {
                    r#type: "text",
                    placeholder: "~/.local/share/fsn/packages/my-bridge",
                    value: "{export_path.read()}",
                    style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); font-family: var(--fs-font-mono); \
                            font-size: 13px; box-sizing: border-box;",
                    oninput: move |e| *export_path.write() = e.value(),
                }
            }

            if let Some(msg) = export_msg.read().as_deref() {
                div {
                    style: "font-size: 13px; color: var(--fs-color-text-muted); margin-bottom: 12px;",
                    "{msg}"
                }
            }

            div { style: "display: flex; gap: 8px;",
                button {
                    style: "padding: 8px 16px; background: var(--fs-color-bg-surface); \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); cursor: pointer;",
                    disabled: bridge_id.read().is_empty() || target_service.read().is_empty(),
                    onclick: move |_| {
                        let mut b = build_bridge_resource(
                            &bridge_id.read(),
                            &bridge_name.read(),
                            &target_role.read(),
                            &target_service.read(),
                        );
                        b.validate();
                        *validation.write() = Some(b.meta.status.clone());
                        // Auto-set export path if empty
                        if export_path.read().is_empty() {
                            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                            *export_path.write() = format!("{home}/.local/share/fsn/packages/{}", b.meta.id);
                        }
                    },
                    "Validate"
                }
                button {
                    style: "padding: 8px 24px; background: var(--fs-color-primary); color: white; \
                            border: none; border-radius: var(--fs-radius-md); cursor: pointer;",
                    disabled: bridge_id.read().is_empty() || target_service.read().is_empty() || export_path.read().is_empty(),
                    onclick: move |_| {
                        let mut b = build_bridge_resource(
                            &bridge_id.read(),
                            &bridge_name.read(),
                            &target_role.read(),
                            &target_service.read(),
                        );
                        b.validate();
                        *validation.write() = Some(b.meta.status.clone());
                        let dir = std::path::PathBuf::from(export_path.read().clone());
                        match save_bridge(&b, &dir) {
                            Ok(p)  => *export_msg.write() = Some(format!("Saved to {}", p.display())),
                            Err(e) => *export_msg.write() = Some(format!("Error: {e}")),
                        }
                    },
                    "Save Locally"
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a `BridgeResource` with skeleton method mappings for all required methods.
fn build_bridge_resource(
    id: &str,
    name: &str,
    role: &str,
    service: &str,
) -> BridgeResource {
    let required = Role::new(role).required_bridge_methods();
    let methods: Vec<BridgeMethod> = required
        .iter()
        .map(|m| BridgeMethod {
            standard_name:    m.to_string(),
            http_method:      HttpMethod::Post,
            endpoint:         format!("/api/{}", m.replace('.', "/")),
            request_mapping:  FieldMapping::identity(),
            response_mapping: FieldMapping::identity(),
        })
        .collect();

    BridgeResource {
        meta: ResourceMeta {
            id: id.to_owned(),
            name: if name.is_empty() { format!("{service} {role} bridge") } else { name.to_owned() },
            description: format!("Bridge for {service} implementing the {role} role API."),
            version: "0.1.0".into(),
            author: String::new(),
            license: "MIT".into(),
            icon: PathBuf::from("bridge.svg"),
            tags: vec![role.to_owned(), service.to_owned(), "bridge".to_owned()],
            resource_type: ResourceType::Bridge,
            dependencies: vec![],
            signature: None,
            platform: None,
            status: ValidationStatus::Incomplete,
            source: None,
        },
        target_role: Role::new(role),
        target_service: service.to_owned(),
        methods,
    }
}

fn save_bridge(
    resource: &BridgeResource,
    dir: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join("resource.toml");
    let toml = toml::to_string_pretty(resource).map_err(|e| e.to_string())?;
    std::fs::write(&path, toml).map_err(|e| e.to_string())?;
    Ok(path)
}
