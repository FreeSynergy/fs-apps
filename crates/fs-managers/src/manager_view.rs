// manager_view.rs — ManagerView: the standardized package manager UI.
//
// This is ONE component that works for ANY package type.
// Changing it here changes it everywhere — for every package.
//
// Layout:
//   Left sidebar:  package list + sub-instances (for Bot/Container/Bridge)
//   Top tab bar:   Info | Config | Builder
//   Content area:  renders the active tab
//
// The Manager offers the HOW.
// The Package (via PackageViewModel) owns the WHAT.

use dioxus::prelude::*;

use crate::view_model::{ConfigFieldView, ConfigKindView, PackageViewModel};

// ── Active tab ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug, Default)]
enum ManagerTab {
    #[default]
    Info,
    Config,
    Builder,
}

impl ManagerTab {
    fn label(&self) -> &'static str {
        match self {
            Self::Info    => "Info",
            Self::Config  => "Config",
            Self::Builder => "Builder",
        }
    }
}

// ── Sidebar navigation state ──────────────────────────────────────────────────

/// Which level the sidebar is showing.
#[derive(Clone, PartialEq, Debug, Default)]
enum SidebarLevel {
    /// Shows the package list.
    #[default]
    Packages,
    /// Drilled into sub-instances of the given package.
    Instances { package_id: String },
}

// ── CSS ───────────────────────────────────────────────────────────────────────

const MANAGER_CSS: &str = r#"
/* ── Status badges ─────────────────────────────── */
.fs-status-badge {
    display: inline-flex; align-items: center; gap: 5px;
    padding: 2px 8px; border-radius: 999px; font-size: 11px; font-weight: 600;
}
.fs-status--running     { background: var(--fs-success-bg); color: var(--fs-success); }
.fs-status--stopped     { background: var(--fs-bg-elevated); color: var(--fs-text-muted); }
.fs-status--transitioning { background: var(--fs-warning-bg); color: var(--fs-warning); }
.fs-status--error       { background: var(--fs-error-bg); color: var(--fs-error); }
.fs-status--not-installed { background: var(--fs-bg-elevated); color: var(--fs-text-muted); }

/* ── Manager tab bar ───────────────────────────── */
.fs-manager-tabs {
    display: flex; gap: 0;
    border-bottom: 1px solid var(--fs-border);
    background: var(--fs-bg-surface);
    flex-shrink: 0;
}
.fs-manager-tab {
    padding: 10px 20px; font-size: 13px; font-weight: 500;
    cursor: pointer; border-bottom: 2px solid transparent;
    color: var(--fs-text-secondary); transition: color 120ms, border-color 120ms;
    background: none; border-top: none; border-left: none; border-right: none;
    user-select: none;
}
.fs-manager-tab:hover { color: var(--fs-text-primary); }
.fs-manager-tab--active {
    color: var(--fs-primary) !important;
    border-bottom-color: var(--fs-primary) !important;
}

/* ── Manager sidebar entry ─────────────────────── */
.fs-mgr-item {
    display: flex; align-items: center; gap: 10px;
    padding: 9px 12px; cursor: pointer;
    border-radius: 6px; margin: 1px 6px;
    color: var(--fs-text-secondary); font-size: 13px;
    transition: background 100ms, color 100ms;
    user-select: none;
}
.fs-mgr-item:hover       { background: var(--fs-bg-hover); color: var(--fs-text-primary); }
.fs-mgr-item--active     { background: var(--fs-sidebar-active-bg); color: var(--fs-sidebar-active); }
.fs-mgr-item--instance   { padding-left: 28px; font-size: 12px; }

/* ── Config field ──────────────────────────────── */
.fs-cfg-field {
    display: grid; grid-template-columns: 180px 1fr auto;
    align-items: start; gap: 10px 16px;
    padding: 12px 0; border-bottom: 1px solid var(--fs-border);
}
.fs-cfg-field:last-child { border-bottom: none; }
.fs-cfg-label {
    font-size: 13px; font-weight: 500; color: var(--fs-text-primary);
    padding-top: 8px;
}
.fs-cfg-label--required::after { content: " *"; color: var(--fs-error); }
.fs-cfg-input {
    padding: 7px 10px; font-size: 13px;
    background: var(--fs-bg-input); color: var(--fs-text-primary);
    border: 1px solid var(--fs-border); border-radius: var(--fs-radius-md);
    width: 100%; box-sizing: border-box; outline: none;
    transition: border-color 120ms;
}
.fs-cfg-input:focus { border-color: var(--fs-border-focus); }
.fs-cfg-help {
    padding-top: 4px; font-size: 12px; color: var(--fs-text-muted);
    line-height: 1.5;
}
.fs-cfg-help--missing { color: var(--fs-warning); font-style: italic; }
.fs-cfg-restart-hint {
    font-size: 11px; color: var(--fs-warning); margin-top: 3px;
}

/* ── Health check item ─────────────────────────── */
.fs-health-item {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 0; font-size: 13px;
}
.fs-health-dot {
    width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0;
}
.fs-health-dot--ok   { background: var(--fs-success); }
.fs-health-dot--fail { background: var(--fs-error); }

/* ── Manager action buttons ────────────────────── */
.fs-mgr-actions {
    display: flex; gap: 8px; flex-wrap: wrap;
}
.fs-mgr-btn {
    padding: 6px 14px; font-size: 12px; font-weight: 600;
    border-radius: var(--fs-radius-md); border: 1px solid transparent;
    cursor: pointer; transition: background 120ms, opacity 120ms;
}
.fs-mgr-btn--start   { background: var(--fs-success-bg); color: var(--fs-success); border-color: var(--fs-success); }
.fs-mgr-btn--stop    { background: var(--fs-error-bg);   color: var(--fs-error);   border-color: var(--fs-error); }
.fs-mgr-btn--persist { background: var(--fs-bg-elevated); color: var(--fs-text-secondary); border-color: var(--fs-border); }
.fs-mgr-btn:disabled { opacity: 0.4; cursor: not-allowed; }
"#;

// ── ManagerView ───────────────────────────────────────────────────────────────

/// Props for the standardized Manager UI.
///
/// The caller builds a [`PackageViewModel`] (by calling
/// `PackageViewModel::from_manageable(&pkg)`) and passes it here together
/// with a list of all packages (for the sidebar) and action callbacks.
#[derive(Props, Clone, PartialEq)]
pub struct ManagerViewProps {
    /// All packages shown in the sidebar.
    pub packages:        Vec<PackageViewModel>,

    /// The package currently selected / being managed.
    pub selected:        PackageViewModel,

    /// Fired when the user selects a different package from the sidebar.
    pub on_select:       EventHandler<String>,

    /// Fired when the user clicks Start.
    pub on_start:        EventHandler<String>,

    /// Fired when the user clicks Stop.
    pub on_stop:         EventHandler<String>,

    /// Fired when the user clicks "Enable persistent" (systemd).
    pub on_persist:      EventHandler<String>,

    /// Fired when the user saves a config change: `(package_id, field_key, new_value)`.
    pub on_config_save:  EventHandler<(String, String, String)>,
}

/// The standardized package manager window.
///
/// ONE component — works for every package type (App, Container, Bot, Widget, …).
/// Changing this component changes the Manager for all packages simultaneously.
#[component]
pub fn ManagerView(props: ManagerViewProps) -> Element {
    let mut active_tab:   Signal<ManagerTab>   = use_signal(ManagerTab::default);
    let mut sidebar_level: Signal<SidebarLevel> = use_signal(SidebarLevel::default);

    let pkg = &props.selected;

    rsx! {
        style { "{MANAGER_CSS}" }

        div {
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-bg-base); color: var(--fs-text-primary);",

            // ── Left Sidebar ─────────────────────────────────────────────────
            div {
                style: "width: 220px; flex-shrink: 0; overflow-y: auto; \
                        background: var(--fs-bg-sidebar); \
                        border-right: 1px solid var(--fs-border); \
                        display: flex; flex-direction: column;",

                match sidebar_level.read().clone() {
                    SidebarLevel::Packages => rsx! {
                        // Sidebar header
                        div {
                            style: "padding: 12px 16px 6px; font-size: 11px; font-weight: 600; \
                                    letter-spacing: 0.06em; text-transform: uppercase; \
                                    color: var(--fs-text-muted); flex-shrink: 0;",
                            "Packages"
                        }

                        for p in props.packages.iter().cloned() {
                            SidebarPackageRow {
                                key: "{p.id}",
                                is_active: p.id == pkg.id,
                                pkg: p,
                                on_select: props.on_select.clone(),
                                on_drill: move |id| sidebar_level.set(SidebarLevel::Instances { package_id: id }),
                            }
                        }
                    },

                    SidebarLevel::Instances { package_id } => {
                        let instances = props.packages.iter()
                            .find(|p| p.id == package_id)
                            .map(|p| p.instances.clone())
                            .unwrap_or_default();

                        rsx! {
                            InstancesSidebar {
                                instances,
                                on_back: move |_| sidebar_level.set(SidebarLevel::Packages),
                            }
                        }
                    }
                }
            }

            // ── Main content ──────────────────────────────────────────────────
            div {
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",

                // ── Tab bar ───────────────────────────────────────────────────
                div {
                    class: "fs-manager-tabs",

                    for tab in [ManagerTab::Info, ManagerTab::Config, ManagerTab::Builder] {
                        {
                            let tab_clone = tab.clone();
                            let is_active = *active_tab.read() == tab;
                            rsx! {
                                button {
                                    class: if is_active { "fs-manager-tab fs-manager-tab--active" } else { "fs-manager-tab" },
                                    onclick: move |_| active_tab.set(tab_clone.clone()),
                                    "{tab.label()}"
                                }
                            }
                        }
                    }
                }

                // ── Tab content ───────────────────────────────────────────────
                div {
                    style: "flex: 1; overflow-y: auto; padding: 20px 24px;",

                    match *active_tab.read() {
                        ManagerTab::Info    => rsx! { InfoTab { pkg: pkg.clone(), on_start: props.on_start.clone(), on_stop: props.on_stop.clone(), on_persist: props.on_persist.clone() } },
                        ManagerTab::Config  => rsx! { ConfigTab { pkg_id: pkg.id.clone(), fields: pkg.config_fields.clone(), on_save: props.on_config_save.clone() } },
                        ManagerTab::Builder => rsx! { BuilderTab { pkg_id: pkg.id.clone(), fields: pkg.build_fields.clone(), on_save: props.on_config_save.clone() } },
                    }
                }
            }
        }
    }
}

// ── SidebarPackageRow ─────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct SidebarPackageRowProps {
    pkg:       PackageViewModel,
    is_active: bool,
    on_select: EventHandler<String>,
    on_drill:  EventHandler<String>,
}

#[component]
fn SidebarPackageRow(props: SidebarPackageRowProps) -> Element {
    let pkg = &props.pkg;
    let id_for_select = pkg.id.clone();
    let id_for_drill  = pkg.id.clone();

    rsx! {
        div {
            class: if props.is_active { "fs-mgr-item fs-mgr-item--active" } else { "fs-mgr-item" },
            onclick: move |_| props.on_select.call(id_for_select.clone()),

            span {
                style: "font-size: 16px; flex-shrink: 0; min-width: 20px; text-align: center;",
                if pkg.icon.is_empty() { "📦" } else { "{pkg.icon}" }
            }

            div {
                style: "flex: 1; min-width: 0;",
                div {
                    style: "display: flex; align-items: center; gap: 6px; \
                            white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                    span { "{pkg.name}" }
                    if pkg.is_running {
                        span {
                            style: "width: 6px; height: 6px; border-radius: 50%; \
                                    background: var(--fs-success); flex-shrink: 0;",
                            ""
                        }
                    }
                }
                div {
                    style: "font-size: 11px; color: var(--fs-text-muted); margin-top: 1px;",
                    "{pkg.version}"
                }
            }

            if pkg.has_instances {
                span {
                    style: "font-size: 13px; color: var(--fs-text-muted); flex-shrink: 0; \
                            padding: 2px 4px;",
                    onclick: move |e| {
                        e.stop_propagation();
                        props.on_drill.call(id_for_drill.clone());
                    },
                    "›"
                }
            }
        }
    }
}

// ── InfoTab ───────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct InfoTabProps {
    pkg:        PackageViewModel,
    on_start:   EventHandler<String>,
    on_stop:    EventHandler<String>,
    on_persist: EventHandler<String>,
}

#[component]
fn InfoTab(props: InfoTabProps) -> Element {
    let pkg = &props.pkg;

    rsx! {
        div {
            // ── Package header ─────────────────────────────────────────────
            div {
                style: "display: flex; align-items: center; gap: 16px; margin-bottom: 24px; \
                        padding-bottom: 20px; border-bottom: 1px solid var(--fs-border);",

                // Big icon
                div {
                    style: "width: 56px; height: 56px; border-radius: 12px; \
                            background: var(--fs-bg-elevated); display: flex; \
                            align-items: center; justify-content: center; \
                            font-size: 28px; flex-shrink: 0;",
                    if pkg.icon.is_empty() { "📦" } else { "{pkg.icon}" }
                }

                div {
                    style: "flex: 1; min-width: 0;",

                    div {
                        style: "display: flex; align-items: baseline; gap: 10px; flex-wrap: wrap; margin-bottom: 4px;",
                        h2 {
                            style: "margin: 0; font-size: 18px; font-weight: 700; \
                                    color: var(--fs-text-bright);",
                            "{pkg.name}"
                        }
                        span {
                            style: "font-size: 12px; color: var(--fs-text-muted); padding: 1px 6px; \
                                    background: var(--fs-bg-elevated); border-radius: 999px;",
                            "v{pkg.version}"
                        }
                        span {
                            style: "font-size: 11px; padding: 1px 7px; border-radius: 999px; \
                                    background: var(--fs-bg-elevated); color: var(--fs-text-muted);",
                            "{pkg.type_label}"
                        }
                    }

                    if !pkg.description.is_empty() {
                        p {
                            style: "margin: 0; font-size: 13px; color: var(--fs-text-secondary); \
                                    line-height: 1.5;",
                            "{pkg.description}"
                        }
                    }
                }
            }

            // ── Meta grid ─────────────────────────────────────────────────────
            div {
                style: "display: grid; grid-template-columns: 120px 1fr; gap: 8px 16px; \
                        font-size: 13px; margin-bottom: 24px;",

                if !pkg.author.is_empty() {
                    span { style: "color: var(--fs-text-muted); font-weight: 500;", "Author" }
                    span { style: "color: var(--fs-text-primary);", "{pkg.author}" }
                }

                if !pkg.category.is_empty() {
                    span { style: "color: var(--fs-text-muted); font-weight: 500;", "Category" }
                    span { style: "color: var(--fs-text-primary);", "{pkg.category}" }
                }

                span { style: "color: var(--fs-text-muted); font-weight: 500;", "Status" }
                span {
                    class: "fs-status-badge {pkg.status_css}",
                    "{pkg.status_label}"
                }
            }

            // ── Action buttons ────────────────────────────────────────────────
            if pkg.is_installed {
                div {
                    class: "fs-mgr-actions",
                    style: "margin-bottom: 24px;",

                    if pkg.can_start {
                        button {
                            class: "fs-mgr-btn fs-mgr-btn--start",
                            onclick: {
                                let id = pkg.id.clone();
                                move |_| props.on_start.call(id.clone())
                            },
                            "▶ Start"
                        }
                    }

                    if pkg.can_stop {
                        button {
                            class: "fs-mgr-btn fs-mgr-btn--stop",
                            onclick: {
                                let id = pkg.id.clone();
                                move |_| props.on_stop.call(id.clone())
                            },
                            "■ Stop"
                        }
                    }

                    if pkg.can_persist {
                        button {
                            class: "fs-mgr-btn fs-mgr-btn--persist",
                            onclick: {
                                let id = pkg.id.clone();
                                move |_| props.on_persist.call(id.clone())
                            },
                            "⚙ Enable persistent"
                        }
                    }
                }
            }

            // ── Health checks ─────────────────────────────────────────────────
            div {
                style: "margin-bottom: 24px;",

                h3 {
                    style: "font-size: 12px; font-weight: 600; letter-spacing: 0.06em; \
                            text-transform: uppercase; color: var(--fs-text-muted); \
                            margin: 0 0 10px;",
                    "Health"
                }

                if pkg.health_checks.is_empty() {
                    div {
                        style: "font-size: 13px; color: var(--fs-text-muted);",
                        "No health checks defined."
                    }
                } else {
                    for check in &pkg.health_checks {
                        div {
                            key: "{check.name}",
                            class: "fs-health-item",

                            div {
                                class: if check.passed { "fs-health-dot fs-health-dot--ok" } else { "fs-health-dot fs-health-dot--fail" },
                                ""
                            }
                            span {
                                style: "color: var(--fs-text-primary);",
                                "{check.name}"
                            }
                            if let Some(msg) = &check.message {
                                span {
                                    style: "color: var(--fs-text-muted); font-size: 12px;",
                                    "— {msg}"
                                }
                            }
                        }
                    }
                }
            }

            // ── Not installed notice ──────────────────────────────────────────
            if !pkg.is_installed {
                div {
                    style: "padding: 16px; background: var(--fs-bg-elevated); \
                            border: 1px solid var(--fs-border); border-radius: var(--fs-radius-md); \
                            font-size: 13px; color: var(--fs-text-muted);",
                    "This package is not installed. Install it via the Store."
                }
            }
        }
    }
}

// ── ConfigTab ─────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct ConfigTabProps {
    pkg_id:  String,
    fields:  Vec<ConfigFieldView>,
    on_save: EventHandler<(String, String, String)>,
}

#[component]
fn ConfigTab(props: ConfigTabProps) -> Element {
    rsx! {
        div {
            if props.fields.is_empty() {
                div {
                    style: "padding: 20px 0; font-size: 13px; color: var(--fs-text-muted);",
                    "This package has no configurable settings."
                }
            } else {
                h3 {
                    style: "font-size: 12px; font-weight: 600; letter-spacing: 0.06em; \
                            text-transform: uppercase; color: var(--fs-text-muted); \
                            margin: 0 0 16px;",
                    "Configuration"
                }

                for field in &props.fields {
                    { config_field_row(field, props.pkg_id.clone(), props.on_save.clone()) }
                }
            }
        }
    }
}

/// Renders one config field row with label, input widget, and help text.
fn config_field_row(
    field:   &ConfigFieldView,
    pkg_id:  String,
    on_save: EventHandler<(String, String, String)>,
) -> Element {
    let key_clone    = field.key.clone();
    let current_val  = field.value.clone();
    let mut edit_val: Signal<String> = use_signal(|| current_val.clone());

    rsx! {
        div {
            key: "{field.key}",
            class: "fs-cfg-field",

            // ── Label ─────────────────────────────────────────────────────────
            div {
                class: if field.required { "fs-cfg-label fs-cfg-label--required" } else { "fs-cfg-label" },
                "{field.label}"
            }

            // ── Input ─────────────────────────────────────────────────────────
            div {
                match &field.kind {
                    ConfigKindView::Bool(checked) => {
                        let checked = *checked;
                        rsx! {
                            div {
                                style: "display: flex; align-items: center; gap: 10px; padding-top: 6px;",
                                input {
                                    r#type: "checkbox",
                                    checked: "{checked}",
                                    style: "width: 16px; height: 16px; cursor: pointer; accent-color: var(--fs-primary);",
                                    onchange: move |e| {
                                        let val = if e.checked() { "true" } else { "false" };
                                        on_save.call((pkg_id.clone(), key_clone.clone(), val.to_string()));
                                    },
                                }
                                if field.needs_restart {
                                    span { class: "fs-cfg-restart-hint", "Requires restart" }
                                }
                            }
                        }
                    },

                    ConfigKindView::Select { options } => rsx! {
                        select {
                            class: "fs-cfg-input",
                            onchange: move |e| {
                                on_save.call((pkg_id.clone(), key_clone.clone(), e.value()));
                            },
                            for (val, label) in options {
                                option {
                                    value: "{val}",
                                    selected: *val == field.value,
                                    "{label}"
                                }
                            }
                        }
                    },

                    ConfigKindView::Password => rsx! {
                        input {
                            r#type: "password",
                            class: "fs-cfg-input",
                            value: "{edit_val}",
                            placeholder: "••••••••",
                            oninput:  move |e| edit_val.set(e.value()),
                            onblur:   {
                                let pk = pkg_id.clone(); let kk = key_clone.clone();
                                move |_| { on_save.call((pk.clone(), kk.clone(), edit_val.read().clone())); }
                            },
                        }
                    },

                    ConfigKindView::Textarea => rsx! {
                        textarea {
                            class: "fs-cfg-input",
                            style: "min-height: 80px; resize: vertical;",
                            value: "{edit_val}",
                            oninput:  move |e| edit_val.set(e.value()),
                            onblur:   {
                                let pk = pkg_id.clone(); let kk = key_clone.clone();
                                move |_| { on_save.call((pk.clone(), kk.clone(), edit_val.read().clone())); }
                            },
                        }
                    },

                    // Text, Number, Port, Path → text input
                    _ => rsx! {
                        input {
                            r#type: "text",
                            class: "fs-cfg-input",
                            value: "{edit_val}",
                            oninput:  move |e| edit_val.set(e.value()),
                            onblur:   {
                                let pk = pkg_id.clone(); let kk = key_clone.clone();
                                move |_| { on_save.call((pk.clone(), kk.clone(), edit_val.read().clone())); }
                            },
                        }
                    },
                }
            }

            // ── Help text ─────────────────────────────────────────────────────
            div {
                if field.missing_help {
                    span {
                        class: "fs-cfg-help fs-cfg-help--missing",
                        "⚠ No help text defined for this setting."
                    }
                } else {
                    span { class: "fs-cfg-help", "{field.help}" }
                }
                if field.needs_restart {
                    div { class: "fs-cfg-restart-hint", "⟳ Restart required" }
                }
            }
        }
    }
}

// ── InstancesSidebar ──────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct InstancesSidebarProps {
    instances: Vec<crate::view_model::InstanceView>,
    on_back:   EventHandler<()>,
}

#[component]
fn InstancesSidebar(props: InstancesSidebarProps) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column;",

            // Back button
            div {
                style: "display: flex; align-items: center; gap: 8px; \
                        padding: 10px 12px; cursor: pointer; \
                        color: var(--fs-primary); font-size: 13px; font-weight: 500; \
                        border-bottom: 1px solid var(--fs-border);",
                onclick: move |_| props.on_back.call(()),
                span { "‹" }
                span { "Back" }
            }

            div {
                style: "padding: 10px 16px 4px; font-size: 11px; font-weight: 600; \
                        letter-spacing: 0.06em; text-transform: uppercase; \
                        color: var(--fs-text-muted);",
                "Instances"
            }

            if props.instances.is_empty() {
                div {
                    style: "padding: 12px 16px; font-size: 12px; color: var(--fs-text-muted);",
                    "No instances yet."
                }
            }

            for inst in props.instances.iter().cloned() {
                InstanceRow { inst }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct InstanceRowProps {
    inst: crate::view_model::InstanceView,
}

#[component]
fn InstanceRow(props: InstanceRowProps) -> Element {
    let dot_color = if props.inst.is_running {
        "var(--fs-success)"
    } else {
        "var(--fs-text-muted)"
    };
    rsx! {
        div {
            key: "{props.inst.id}",
            class: "fs-mgr-item fs-mgr-item--instance",
            span {
                style: "width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; background: {dot_color};",
                ""
            }
            div {
                style: "flex: 1; min-width: 0;",
                div {
                    style: "white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                    "{props.inst.name}"
                }
                div {
                    style: "font-size: 10px; color: var(--fs-text-muted); margin-top: 1px;",
                    "{props.inst.status}"
                }
            }
        }
    }
}

// ── BuilderTab ────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct BuilderTabProps {
    pkg_id:  String,
    fields:  Vec<ConfigFieldView>,
    on_save: EventHandler<(String, String, String)>,
}

#[component]
fn BuilderTab(props: BuilderTabProps) -> Element {
    rsx! {
        div {
            if props.fields.is_empty() {
                div {
                    style: "padding: 20px 0; font-size: 13px; color: var(--fs-text-muted);",
                    "This package has no builder configuration."
                }
            } else {
                h3 {
                    style: "font-size: 12px; font-weight: 600; letter-spacing: 0.06em; \
                            text-transform: uppercase; color: var(--fs-text-muted); \
                            margin: 0 0 16px;",
                    "Builder"
                }

                for field in &props.fields {
                    { config_field_row(field, props.pkg_id.clone(), props.on_save.clone()) }
                }
            }
        }
    }
}
