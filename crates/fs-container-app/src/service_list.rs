/// Service list — shows all FSN systemd services grouped by category with
/// collapsible accordion sections and start/stop/restart actions.
///
/// Uses SystemctlManager (no Podman socket) to list and control services.
use std::collections::BTreeMap;

use dioxus::prelude::*;
use fs_components::AppContext;
use fs_container::{SystemctlManager, UnitActiveState};
use fs_error::FsError;
use fs_i18n;

/// A single service entry displayed in the list.
#[derive(Clone, Debug, PartialEq)]
pub struct ServiceEntry {
    pub name:        String,
    pub active:      UnitActiveState,
    pub sub_state:   String,
    pub description: String,
}

impl ServiceEntry {
    pub fn status_label(&self) -> String {
        match self.active {
            UnitActiveState::Active       => fs_i18n::t("status.running").to_string(),
            UnitActiveState::Inactive     => fs_i18n::t("status.stopped").to_string(),
            UnitActiveState::Activating   => fs_i18n::t("status.starting").to_string(),
            UnitActiveState::Deactivating => fs_i18n::t("status.stopping").to_string(),
            UnitActiveState::Failed       => fs_i18n::t("status.failed").to_string(),
            UnitActiveState::Unknown      => fs_i18n::t("status.unknown").to_string(),
        }
    }

    pub fn status_color(&self) -> &str {
        match self.active {
            UnitActiveState::Active       => "var(--fs-success)",
            UnitActiveState::Inactive     => "var(--fs-text-muted)",
            UnitActiveState::Activating   => "var(--fs-info)",
            UnitActiveState::Deactivating => "var(--fs-warning)",
            UnitActiveState::Failed       => "var(--fs-error)",
            UnitActiveState::Unknown      => "var(--fs-text-muted)",
        }
    }
}

/// Container lifecycle action.
///
/// Carries its own execution logic via `execute` — no external `match` needed.
/// Follows the *Strategy* pattern: the action knows how to apply itself.
#[derive(Clone, Debug, PartialEq)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
}

impl ServiceAction {
    /// Execute this action against `name` via `mgr`.
    pub async fn execute(&self, mgr: &SystemctlManager, name: &str) -> Result<(), FsError> {
        match self {
            Self::Start   => mgr.start(name).await,
            Self::Stop    => mgr.stop(name).await,
            Self::Restart => mgr.restart(name).await,
        }
    }
}

/// Fetch all FSN-managed service unit names via systemctl.
///
/// Queries `systemctl --user list-units` for all `fs-*.service` units, which
/// covers both Podman Quadlet-generated units and manually registered services.
pub async fn list_fs_units() -> Vec<String> {
    let Ok(out) = tokio::process::Command::new("systemctl")
        .args(["--user", "list-units", "--type=service", "--no-legend", "--plain", "--all"])
        .output()
        .await
    else {
        return vec![];
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let unit = line.split_whitespace().next()?;
            if unit.starts_with("fs-") && unit.ends_with(".service") {
                Some(unit.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Service list component — renders all FSN services with start/stop/restart buttons.
#[component]
pub fn ServiceList(mut selected: Signal<Option<String>>) -> Element {
    // Request the shell to open the Store when the user clicks "Install Service".
    // Uses try_use_context so standalone mode (no AppContext) doesn't panic.
    let app_ctx = try_use_context::<AppContext>();
    let mut services: Signal<Vec<ServiceEntry>> = use_signal(Vec::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Poll every 5 seconds
    use_future(move || async move {
        let mgr = SystemctlManager::user();
        loop {
            let units = list_fs_units().await;
            if units.is_empty() {
                error.set(Some(fs_i18n::t("container.services.not_found").to_string()));
            } else {
                let mut entries = Vec::new();
                for unit in &units {
                    match mgr.service_status(unit).await {
                        Ok(s) => entries.push(ServiceEntry {
                            name:        s.name.clone(),
                            active:      s.active_state.clone(),
                            sub_state:   s.sub_state.clone(),
                            description: s.description.clone(),
                        }),
                        Err(_) => entries.push(ServiceEntry {
                            name:        unit.clone(),
                            active:      UnitActiveState::Unknown,
                            sub_state:   String::new(),
                            description: String::new(),
                        }),
                    }
                }
                services.set(entries);
                error.set(None);
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    // Action handler: start/stop/restart then refresh
    let on_action = move |(name, action): (String, ServiceAction)| {
        spawn(async move {
            let mgr = SystemctlManager::user();
            let _ = action.execute(&mgr, &name).await;
        });
    };

    // Group services by category (parsed from "fs-<category>-<name>.service")
    let groups: BTreeMap<String, Vec<ServiceEntry>> = {
        let mut map: BTreeMap<String, Vec<ServiceEntry>> = BTreeMap::new();
        for svc in services.read().iter() {
            let category = service_category(&svc.name);
            map.entry(category).or_default().push(svc.clone());
        }
        map
    };

    rsx! {
        div {
            class: "fs-service-list",

            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                h2 { style: "margin: 0; font-size: 18px;", {fs_i18n::t("container.section.installed")} }
                button {
                    style: "background: var(--fs-primary); color: white; border: none; \
                            padding: 8px 16px; border-radius: var(--fs-radius-md); cursor: pointer;",
                    onclick: move |_| {
                        if let Some(mut ctx) = app_ctx {
                            ctx.app_open_req.set(Some("store".to_string()));
                        }
                    },
                    {fs_i18n::t("container.services.install_btn")}
                }
            }

            // Connection error
            if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fs-warning); background: var(--fs-warning-bg); \
                            border: 1px solid var(--fs-warning); border-radius: 6px; \
                            padding: 12px; margin-bottom: 16px; font-size: 13px;",
                    "{err}"
                }
            }

            // Empty state
            if services.read().is_empty() && error.read().is_none() {
                div {
                    style: "text-align: center; color: var(--fs-text-muted); padding: 48px;",
                    p { {fs_i18n::t("container.services.empty")} }
                    p { {fs_i18n::t("container.services.empty_hint")} }
                }
            }

            // Accordion groups
            for (category, entries) in groups {
                ServiceAccordionGroup {
                    key: "{category}",
                    category: category.clone(),
                    entries: entries.clone(),
                    selected,
                    on_action,
                }
            }
        }
    }
}

// ── ServiceAccordionGroup ─────────────────────────────────────────────────────

/// Collapsible accordion section showing services of the same category.
#[component]
fn ServiceAccordionGroup(
    category: String,
    entries: Vec<ServiceEntry>,
    selected: Signal<Option<String>>,
    on_action: EventHandler<(String, ServiceAction)>,
) -> Element {
    let mut expanded = use_signal(|| true);
    let count = entries.len();
    let running = entries.iter().filter(|e| e.active == UnitActiveState::Active).count();
    let icon = if *expanded.read() { "▾" } else { "▸" };

    rsx! {
        div {
            style: "margin-bottom: 8px; border: 1px solid var(--fs-border); \
                    border-radius: var(--fs-radius-md); overflow: hidden;",

            // Accordion header
            div {
                style: "display: flex; align-items: center; gap: 10px; \
                        padding: 10px 14px; cursor: pointer; \
                        background: var(--fs-bg-surface); \
                        border-bottom: 1px solid var(--fs-border);",
                onclick: move |_| {
                    let v = *expanded.read();
                    expanded.set(!v);
                },
                span { style: "font-size: 14px; color: var(--fs-text-muted);", "{icon}" }
                span { style: "font-weight: 600; font-size: 14px;", "{category}" }
                span {
                    style: "margin-left: auto; font-size: 12px; color: var(--fs-text-muted);",
                    {fs_i18n::t_with("container.services.running_count", &[("running", &running.to_string()), ("count", &count.to_string())])}
                }
            }

            // Collapsible table
            if *expanded.read() {
                table {
                    style: "width: 100%; border-collapse: collapse;",
                    thead {
                        tr {
                            style: "border-bottom: 1px solid var(--fs-border); \
                                    font-size: 11px; color: var(--fs-text-muted); \
                                    background: var(--fs-bg-elevated);",
                            th { style: "text-align: left; padding: 6px 8px;", "NAME" }
                            th { style: "text-align: left; padding: 6px 8px;", "STATUS" }
                            th { style: "text-align: left; padding: 6px 8px;", "STATE" }
                            th { style: "text-align: right; padding: 6px 8px;", "ACTIONS" }
                        }
                    }
                    tbody {
                        for svc in entries {
                            ServiceRow {
                                key: "{svc.name}",
                                service: svc,
                                selected,
                                on_action,
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Derive a human-readable category from a systemd unit name like
/// `fs-proxy-zentinel.service` → `proxy`.
fn service_category(name: &str) -> String {
    let bare = name.trim_end_matches(".service").trim_start_matches("fs-");
    bare.split('-').next().unwrap_or("other").to_string()
}

/// A single row in the service table.
#[component]
fn ServiceRow(
    service: ServiceEntry,
    mut selected: Signal<Option<String>>,
    on_action: EventHandler<(String, ServiceAction)>,
) -> Element {
    let name = service.name.clone();
    rsx! {
        tr {
            style: "border-bottom: 1px solid var(--fs-border); cursor: pointer;",
            onclick: move |_| *selected.write() = Some(name.clone()),

            td { style: "padding: 12px 8px; font-weight: 500; font-size: 13px;", "{service.name}" }
            td { style: "padding: 12px 8px;",
                span {
                    style: "color: {service.status_color()}; font-size: 13px;",
                    "{service.status_label()}"
                }
            }
            td { style: "padding: 12px 8px; font-size: 12px; color: var(--fs-text-muted);",
                "{service.sub_state}"
            }
            td { style: "padding: 12px 8px; text-align: right;",
                ServiceActions {
                    name: service.name.clone(),
                    active: service.active.clone(),
                    selected,
                    on_action,
                }
            }
        }
    }
}

/// Start/Stop/Restart + Logs buttons for a service.
#[component]
fn ServiceActions(
    name: String,
    active: UnitActiveState,
    mut selected: Signal<Option<String>>,
    on_action: EventHandler<(String, ServiceAction)>,
) -> Element {
    let is_running = matches!(active, UnitActiveState::Active);
    rsx! {
        div {
            style: "display: flex; gap: 4px; justify-content: flex-end;",

            if !is_running {
                button {
                    style: "padding: 4px 8px; background: var(--fs-success); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: fs_i18n::t("actions.start").to_string(),
                    onclick: {
                        let n = name.clone();
                        move |_| on_action.call((n.clone(), ServiceAction::Start))
                    },
                    "▶"
                }
            }

            if is_running {
                button {
                    style: "padding: 4px 8px; background: var(--fs-error); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: fs_i18n::t("actions.stop").to_string(),
                    onclick: {
                        let n = name.clone();
                        move |_| on_action.call((n.clone(), ServiceAction::Stop))
                    },
                    "■"
                }
                button {
                    style: "padding: 4px 8px; background: var(--fs-warning); color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    title: fs_i18n::t("actions.restart").to_string(),
                    onclick: {
                        let n = name.clone();
                        move |_| on_action.call((n.clone(), ServiceAction::Restart))
                    },
                    "↺"
                }
            }

            // Select service for log view
            button {
                style: "padding: 4px 8px; background: var(--fs-bg-surface); border: 1px solid var(--fs-border); border-radius: 4px; cursor: pointer; font-size: 12px; color: var(--fs-text-primary);",
                title: fs_i18n::t("container.section.logs").to_string(),
                onclick: {
                    let n = name.clone();
                    move |_| *selected.write() = Some(n.clone())
                },
                "≡"
            }
        }
    }
}
