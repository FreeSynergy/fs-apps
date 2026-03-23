//! Container App Builder — Docker Compose YAML → ContainerResource.
//!
//! Analyzes a pasted Compose file, displays detected services, variables,
//! and roles, lets the user edit all fields, validates the result, and
//! offers a "Publish to Store" button.

use dioxus::prelude::*;
use fs_types::resources::{
    container::{ContainerResource, ContainerVariable, VarType},
    meta::ValidationStatus,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::ollama::{OllamaClient, OLLAMA_BASE_URL};

// ── Compose parsing (minimal) ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct ComposeFile {
    services: HashMap<String, ComposeServiceDef>,
}

#[derive(Deserialize)]
struct ComposeServiceDef {
    image: Option<String>,
    #[serde(default)]
    ports: Vec<serde_json::Value>,
    environment: Option<serde_json::Value>,
    healthcheck: Option<serde_json::Value>,
}

// ── Builder state ─────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug, Default)]
enum BuilderStep {
    #[default]
    Input,
    Review,
    Export,
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Container App Builder component.
#[component]
pub fn ContainerBuilder() -> Element {
    let mut step         = use_signal(BuilderStep::default);
    let mut yaml_input   = use_signal(String::new);
    let mut parse_error  = use_signal(|| Option::<String>::None);
    let mut resource     = use_signal(|| Option::<ContainerResource>::None);
    let mut export_path  = use_signal(String::new);
    let mut export_msg   = use_signal(|| Option::<String>::None);
    let mut ai_status    = use_signal(|| Option::<String>::None);
    let mut ai_prompt    = use_signal(String::new);

    rsx! {
        div {
            class: "fs-container-app-builder",
            style: "padding: 24px;",

            match *step.read() {
                // ── Step 1: Input ─────────────────────────────────────────────
                BuilderStep::Input => rsx! {
                    h3 { "Container App Builder" }
                    p { style: "color: var(--fs-color-text-muted); margin-bottom: 16px;",
                        "Paste a Docker Compose file. The builder analyzes it and creates \
                         a ContainerResource ready for the FSN Store."
                    }

                    div { style: "margin-bottom: 16px;",
                        label { style: "display: block; font-weight: 500; margin-bottom: 4px;",
                            "Docker Compose YAML"
                        }
                        textarea {
                            style: "width: 100%; height: 280px; padding: 8px 12px; \
                                    border: 1px solid var(--fs-color-border-default); \
                                    border-radius: var(--fs-radius-md); font-family: var(--fs-font-mono); \
                                    font-size: 13px; resize: vertical; box-sizing: border-box;",
                            placeholder: "version: '3'\nservices:\n  myapp:\n    image: myapp:latest\n    ports:\n      - '8080:8080'",
                            oninput: move |e| {
                                *yaml_input.write() = e.value();
                                *parse_error.write() = None;
                            },
                            "{yaml_input.read()}"
                        }
                    }

                    if let Some(err) = parse_error.read().as_deref() {
                        div {
                            style: "color: var(--fs-color-error); font-size: 13px; margin-bottom: 8px;",
                            "{err}"
                        }
                    }

                    button {
                        style: "padding: 8px 24px; background: var(--fs-color-primary); color: white; \
                                border: none; border-radius: var(--fs-radius-md); cursor: pointer;",
                        disabled: yaml_input.read().is_empty(),
                        onclick: move |_| {
                            match analyze_compose_yaml(&yaml_input.read()) {
                                Ok(r) => {
                                    *resource.write() = Some(r);
                                    *step.write() = BuilderStep::Review;
                                }
                                Err(e) => *parse_error.write() = Some(e),
                            }
                        },
                        "Analyze →"
                    }
                },

                // ── Step 2: Review ────────────────────────────────────────────
                BuilderStep::Review => rsx! {
                    if let Some(res) = resource.read().as_ref() {
                        // Validation badge
                        div { style: "margin-bottom: 16px; display: flex; align-items: center; gap: 12px;",
                            h3 { style: "margin: 0;", "{res.meta.name}" }
                            span {
                                style: "font-size: 18px;",
                                match res.meta.status {
                                    ValidationStatus::Ok         => "✅",
                                    ValidationStatus::Incomplete => "⚠️",
                                    ValidationStatus::Broken     => "❌",
                                }
                            }
                            span {
                                style: "font-size: 12px; color: var(--fs-color-text-muted);",
                                match res.meta.status {
                                    ValidationStatus::Ok         => "Ready to publish",
                                    ValidationStatus::Incomplete => "Incomplete — fill in required fields",
                                    ValidationStatus::Broken     => "Broken — critical fields missing",
                                }
                            }
                        }

                        // Services
                        div { style: "margin-bottom: 16px;",
                            div { style: "font-weight: 500; margin-bottom: 8px;", "Services" }
                            div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                for svc in &res.services {
                                    span {
                                        key: "{svc.name}",
                                        style: "padding: 4px 10px; border-radius: 999px; font-size: 12px; \
                                                background: var(--fs-color-bg-surface); \
                                                border: 1px solid var(--fs-color-border-default);",
                                        if svc.is_main { "★ " } else { "" }
                                        "{svc.name} ({svc.image})"
                                    }
                                }
                            }
                        }

                        // Roles provided
                        if !res.roles_provided.is_empty() {
                            div { style: "margin-bottom: 16px;",
                                div { style: "font-weight: 500; margin-bottom: 8px;", "Roles Provided" }
                                div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                    for role in &res.roles_provided {
                                        span {
                                            key: "{role.as_str()}",
                                            style: "padding: 4px 10px; border-radius: 999px; font-size: 12px; \
                                                    background: var(--fs-color-primary); color: white;",
                                            "{role.as_str()}"
                                        }
                                    }
                                }
                            }
                        }

                        // Variables
                        if !res.variables.is_empty() {
                            div { style: "margin-bottom: 16px;",
                                div { style: "font-weight: 500; margin-bottom: 8px;",
                                    "Variables ({res.variables.len()})"
                                }
                                div {
                                    style: "border: 1px solid var(--fs-color-border-default); \
                                            border-radius: var(--fs-radius-md); overflow: hidden;",
                                    // Header
                                    div {
                                        style: "display: grid; grid-template-columns: 2fr 1fr 1fr auto; \
                                                padding: 6px 12px; background: var(--fs-color-bg-surface); \
                                                font-size: 11px; font-weight: 600; text-transform: uppercase; \
                                                letter-spacing: 0.07em; color: var(--fs-color-text-muted);",
                                        span { "Name" }
                                        span { "Type" }
                                        span { "Role" }
                                        span { "Req" }
                                    }
                                    for (i, var) in res.variables.iter().enumerate() {
                                        VariableRow {
                                            key: "{var.name}",
                                            name: var.name.clone(),
                                            type_label: format!("{:?}", var.var_type),
                                            role_label: var.role.as_ref().map(|r| r.as_str().to_owned()).unwrap_or_default(),
                                            required: var.required,
                                            description: var.description.clone(),
                                            index: i,
                                        }
                                    }
                                }
                            }
                        }

                        // AI Assist
                        div {
                            style: "margin-bottom: 16px; padding: 12px 16px; \
                                    background: var(--fs-color-bg-surface); \
                                    border-radius: var(--fs-radius-md); \
                                    border: 1px solid var(--fs-color-border-default);",
                            div { style: "font-weight: 500; margin-bottom: 8px;", "AI Assist (Ollama)" }
                            div { style: "display: flex; gap: 8px; margin-bottom: 8px;",
                                input {
                                    r#type: "text",
                                    placeholder: "Describe the service to enrich metadata...",
                                    value: "{ai_prompt.read()}",
                                    style: "flex: 1; padding: 6px 10px; border: 1px solid var(--fs-color-border-default); \
                                            border-radius: var(--fs-radius-md); font-size: 13px;",
                                    oninput: move |e| *ai_prompt.write() = e.value(),
                                }
                                button {
                                    style: "padding: 6px 16px; background: var(--fs-color-primary); color: white; \
                                            border: none; border-radius: var(--fs-radius-md); cursor: pointer; \
                                            font-size: 13px; white-space: nowrap;",
                                    disabled: ai_prompt.read().is_empty(),
                                    onclick: move |_| {
                                        let prompt = ai_prompt.read().clone();
                                        *ai_status.write() = Some("Asking Ollama…".into());
                                        spawn(async move {
                                            let client = OllamaClient::new(OLLAMA_BASE_URL, "llama3.2");
                                            if !client.is_available().await {
                                                *ai_status.write() = Some("Ollama not reachable at localhost:11434".into());
                                                return;
                                            }
                                            match client.suggest_metadata(&prompt).await {
                                                Ok(s)  => *ai_status.write() = Some(format!("AI: {}", s.description)),
                                                Err(e) => *ai_status.write() = Some(format!("AI error: {e}")),
                                            }
                                        });
                                    },
                                    "Ask AI"
                                }
                            }
                            if let Some(msg) = ai_status.read().as_deref() {
                                div {
                                    style: "font-size: 12px; color: var(--fs-color-text-muted);",
                                    "{msg}"
                                }
                            }
                        }

                        div { style: "display: flex; gap: 8px;",
                            button {
                                style: "padding: 8px 16px; background: var(--fs-color-bg-surface); \
                                        border: 1px solid var(--fs-color-border-default); \
                                        border-radius: var(--fs-radius-md); cursor: pointer;",
                                onclick: move |_| *step.write() = BuilderStep::Input,
                                "← Back"
                            }
                            button {
                                style: "padding: 8px 24px; background: var(--fs-color-primary); color: white; \
                                        border: none; border-radius: var(--fs-radius-md); cursor: pointer;",
                                onclick: move |_| {
                                    let id = resource.read().as_ref().map(|r| r.meta.id.clone()).unwrap_or_default();
                                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                                    *export_path.write() = format!("{home}/.local/share/fsn/packages/{id}");
                                    *step.write() = BuilderStep::Export;
                                },
                                "Export →"
                            }
                        }
                    }
                },

                // ── Step 3: Export ────────────────────────────────────────────
                BuilderStep::Export => rsx! {
                    h3 { "Export Resource" }
                    p { style: "color: var(--fs-color-text-muted); margin-bottom: 16px;",
                        "Save the ContainerResource as resource.toml to your local packages directory."
                    }

                    div { style: "margin-bottom: 16px;",
                        label { style: "display: block; font-weight: 500; margin-bottom: 4px;",
                            "Save to directory"
                        }
                        input {
                            r#type: "text",
                            value: "{export_path.read()}",
                            style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fs-color-border-default); \
                                    border-radius: var(--fs-radius-md); font-family: var(--fs-font-mono); font-size: 13px; \
                                    box-sizing: border-box;",
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
                            onclick: move |_| *step.write() = BuilderStep::Review,
                            "← Back"
                        }
                        button {
                            style: "padding: 8px 24px; background: var(--fs-color-bg-surface); color: var(--fs-color-text-primary); \
                                    border: 1px solid var(--fs-color-border-default); \
                                    border-radius: var(--fs-radius-md); cursor: pointer;",
                            onclick: move |_| {
                                let dir = std::path::PathBuf::from(export_path.read().clone());
                                if let Some(res) = resource.read().as_ref() {
                                    match save_resource(res, &dir) {
                                        Ok(p)  => *export_msg.write() = Some(format!("Saved to {}", p.display())),
                                        Err(e) => *export_msg.write() = Some(format!("Error: {e}")),
                                    }
                                }
                            },
                            "Save Locally"
                        }
                        button {
                            style: "padding: 8px 24px; background: var(--fs-color-primary); color: white; \
                                    border: none; border-radius: var(--fs-radius-md); cursor: pointer;",
                            onclick: move |_| {
                                *export_msg.write() = Some(
                                    "Use `fs-builder publish <dir>` to sign and publish to the Store.".into()
                                );
                            },
                            "Publish to Store"
                        }
                    }
                },
            }
        }
    }
}

// ── VariableRow ───────────────────────────────────────────────────────────────

#[component]
fn VariableRow(
    name: String,
    type_label: String,
    role_label: String,
    required: bool,
    description: String,
    index: usize,
) -> Element {
    let bg = if index % 2 == 0 {
        "background: transparent;"
    } else {
        "background: var(--fs-color-bg-elevated);"
    };

    rsx! {
        div {
            style: "display: grid; grid-template-columns: 2fr 1fr 1fr auto; \
                    padding: 6px 12px; border-top: 1px solid var(--fs-color-border-default); \
                    font-size: 12px; align-items: center; {bg}",
            span {
                style: "font-family: var(--fs-font-mono); font-size: 11px; \
                        color: var(--fs-color-text-primary); overflow: hidden; text-overflow: ellipsis;",
                title: "{description}",
                "{name}"
            }
            span {
                style: "color: var(--fs-color-text-muted);",
                "{type_label}"
            }
            span {
                style: if !role_label.is_empty() {
                    "color: var(--fs-color-primary); font-weight: 500;"
                } else {
                    "color: var(--fs-color-text-muted);"
                },
                "{role_label}"
            }
            span {
                style: if required {
                    "color: var(--fs-color-error, #ef4444);"
                } else {
                    "color: var(--fs-color-text-muted);"
                },
                if required { "req" } else { "opt" }
            }
        }
    }
}

// ── Compose analysis ──────────────────────────────────────────────────────────

fn analyze_compose_yaml(yaml: &str) -> Result<ContainerResource, String> {
    let compose: ComposeFile = serde_yaml::from_str(yaml)
        .map_err(|e| format!("YAML parse error: {e}"))?;
    if compose.services.is_empty() {
        return Err("No services found in YAML.".into());
    }
    // Delegate to the same logic as the CLI (but we inline a simplified version here
    // since we don't have a shared library import path in the Desktop crate).
    build_resource(yaml, compose)
}

fn build_resource(yaml: &str, compose: ComposeFile) -> Result<ContainerResource, String> {
    use fs_types::resources::{
        container::{ContainerService, NetworkDef, RoleDep},
        meta::{ResourceMeta, ResourceType, ValidationStatus},
    };

    // Detect primary service (non-infra with a port, or first).
    let primary_name = compose.services.iter()
        .find(|(_, d)| !is_infra(d.image.as_deref().unwrap_or("")) && extract_port(d).is_some())
        .or_else(|| compose.services.iter().next())
        .map(|(n, _)| n.clone())
        .unwrap_or_default();

    let services: Vec<ContainerService> = compose.services.iter().map(|(name, def)| {
        let (_, tag) = split_img(def.image.as_deref().unwrap_or(""));
        ContainerService {
            name: name.clone(),
            image: def.image.clone().unwrap_or_default(),
            is_main: *name == primary_name,
            internal: is_infra(def.image.as_deref().unwrap_or("")),
            port: extract_port(def),
            healthcheck: def.healthcheck.as_ref().map(|_| "defined".into()),
            version_tag: tag,
        }
    }).collect();

    let primary_image = compose.services.get(&primary_name)
        .and_then(|d| d.image.clone())
        .unwrap_or_default();
    let (img, img_tag) = split_img(&primary_image);

    let roles_provided = detect_roles(&img);
    let all_svc_names: Vec<String> = compose.services.keys().cloned().collect();
    let mut variables: Vec<ContainerVariable> = Vec::new();
    for (_, def) in &compose.services {
        for var in env_vars(def) {
            variables.push(analyze_var(&var, &all_svc_names));
        }
    }
    variables.dedup_by(|a, b| a.name == b.name);
    let _ = &variables; // suppress type inference warning
    variables.sort_by(|a, b| a.name.cmp(&b.name));

    let roles_required: Vec<RoleDep> = variables.iter()
        .filter_map(|v| v.role.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .map(|r| RoleDep { role: r, optional: false })
        .collect();

    let networks = vec![NetworkDef { name: format!("{primary_name}-backend"), external: false }];
    let volumes = vec![];

    let mut resource = ContainerResource {
        meta: ResourceMeta {
            id: primary_name.replace([' ', '-'], "_").to_lowercase(),
            name: capitalize(&primary_name),
            description: format!("{} — containerized application.", capitalize(&primary_name)),
            version: img_tag,
            author: String::new(),
            license: "MIT".into(),
            icon: std::path::PathBuf::from("icon.svg"),
            tags: roles_provided.iter().map(|r| r.as_str().to_owned()).collect(),
            resource_type: ResourceType::Container,
            dependencies: vec![],
            signature: None,
            platform: None,
            status: ValidationStatus::Incomplete,
            source: None,
        },
        compose_yaml: yaml.to_owned(),
        services,
        roles_provided,
        roles_required,
        apis: vec![],
        variables,
        networks,
        volumes,
    };

    use fs_types::resources::validator::Validate;
    resource.validate();

    Ok(resource)
}

fn is_infra(image: &str) -> bool {
    let img = image.to_lowercase();
    img.contains("postgres") || img.contains("mysql") || img.contains("mariadb")
        || img.contains("redis") || img.contains("dragonfly") || img.contains("valkey")
        || img.contains("nginx") || img.contains("caddy") || img.contains("traefik")
        || img.contains("minio")
}

fn split_img(image: &str) -> (String, String) {
    match image.rsplit_once(':') {
        Some((i, t)) => (i.to_owned(), t.to_owned()),
        None         => (image.to_owned(), "latest".to_owned()),
    }
}

fn extract_port(def: &ComposeServiceDef) -> Option<u16> {
    def.ports.first().and_then(|v| {
        v.as_str().unwrap_or_default().split(':').last().and_then(|p| p.parse().ok())
    })
}

fn env_vars(def: &ComposeServiceDef) -> Vec<String> {
    match &def.environment {
        Some(serde_json::Value::Object(m)) => m.keys().cloned().collect(),
        Some(serde_json::Value::Array(a))  => a.iter()
            .filter_map(|v| v.as_str().and_then(|s| s.split_once('=').map(|(k, _)| k.to_owned())))
            .collect(),
        _ => vec![],
    }
}

fn detect_roles(image: &str) -> Vec<fs_types::resources::meta::Role> {
    use fs_types::resources::meta::Role;
    let img = image.to_lowercase();
    let mut roles = vec![];
    if img.contains("kanidm") || img.contains("keycloak") || img.contains("authentik") { roles.push(Role::new("iam")); }
    if img.contains("forgejo") || img.contains("gitea") || img.contains("gitlab") { roles.push(Role::new("git")); }
    if img.contains("outline") || img.contains("bookstack") { roles.push(Role::new("wiki")); }
    if img.contains("stalwart") || img.contains("postfix") { roles.push(Role::new("smtp")); }
    if img.contains("element") || img.contains("synapse") || img.contains("tuwunel") { roles.push(Role::new("chat")); }
    if img.contains("vikunja") || img.contains("plane") { roles.push(Role::new("tasks")); }
    if img.contains("openobserve") || img.contains("grafana") { roles.push(Role::new("monitoring")); }
    if img.contains("postgres") || img.contains("mysql") { roles.push(Role::new("database")); }
    if img.contains("redis") || img.contains("dragonfly") { roles.push(Role::new("cache")); }
    if img.contains("ollama") { roles.push(Role::new("llm")); }
    roles
}

fn analyze_var(name: &str, all_svcs: &[String]) -> ContainerVariable {
    use fs_types::resources::container::AutoSource;
    use fs_types::resources::meta::Role;
    let upper = name.to_uppercase();

    let var_type = if upper.contains("SECRET") || upper.contains("PASSWORD") || upper.contains("TOKEN") {
        VarType::Secret
    } else if upper.contains("_URL") || upper.contains("_URI") {
        VarType::Url
    } else if upper.contains("_HOST") || upper.contains("_HOSTNAME") {
        VarType::Hostname
    } else if upper.contains("_PORT") {
        VarType::Port
    } else if upper.contains("_EMAIL") || upper.contains("_MAIL") {
        VarType::Email
    } else if upper.starts_with("ENABLE_") || upper.ends_with("_ENABLED") {
        VarType::Bool
    } else {
        VarType::String
    };

    let role = if upper.contains("OIDC") || upper.contains("KANIDM") || upper.contains("AUTH_URL") {
        Some(Role::new("iam"))
    } else if upper.contains("SMTP") || upper.contains("MAIL_HOST") {
        Some(Role::new("smtp"))
    } else if upper.contains("REDIS") || upper.contains("CACHE_URL") {
        Some(Role::new("cache"))
    } else if upper.contains("POSTGRES") || upper.contains("DATABASE_URL") || upper.contains("DB_HOST") {
        Some(Role::new("database"))
    } else {
        None
    };

    let auto_from = all_svcs.iter().find(|svc| {
        let svc_u = svc.to_uppercase();
        upper.contains(&svc_u)
    }).map(|svc| AutoSource::InternalService {
        service_name: svc.clone(),
        url_template: format!("http://{}:{{{{ port }}}}", svc),
    });

    let confidence = if role.is_some() { 0.8 } else { 0.4 };

    ContainerVariable {
        name: name.to_owned(),
        var_type,
        role,
        required: !upper.contains("OPTIONAL"),
        default: None,
        auto_from,
        description: name.replace('_', " ").to_lowercase(),
        confidence,
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None    => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn save_resource(resource: &ContainerResource, dir: &std::path::Path) -> Result<std::path::PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join("resource.toml");
    let toml = toml::to_string_pretty(resource).map_err(|e| e.to_string())?;
    std::fs::write(&path, toml).map_err(|e| e.to_string())?;
    Ok(path)
}
