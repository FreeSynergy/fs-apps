// view_model.rs — PackageViewModel: data extracted from Manageable for the Manager UI.
//
// The Manager UI works with this view model, not with trait objects.
// The caller extracts the data from a Manageable and passes the view model to
// the ManagerView component.
//
// This keeps the UI layer pure (Clone + PartialEq props) while the domain
// layer stays clean (Manageable trait without UI concerns).

use fs_pkg::manageable::{ConfigField, ConfigFieldKind, ConfigValue, InstanceRef};
use fs_pkg::manifest::PackageType;

// ── View structs ──────────────────────────────────────────────────────────────

/// One health check result as display strings.
#[derive(Clone, PartialEq, Debug)]
pub struct HealthCheckView {
    pub name:    String,
    pub passed:  bool,
    pub message: Option<String>,
}

/// One config field as a display-ready struct (Clone + PartialEq).
#[derive(Clone, PartialEq, Debug)]
pub struct ConfigFieldView {
    pub key:           String,
    pub label:         String,
    pub help:          String,
    pub kind:          ConfigKindView,
    pub value:         String,
    pub required:      bool,
    pub needs_restart: bool,
    pub missing_help:  bool,
}

impl ConfigFieldView {
    pub fn from_config_field(f: &ConfigField) -> Self {
        let kind = match &f.kind {
            ConfigFieldKind::Text      => ConfigKindView::Text,
            ConfigFieldKind::Password  => ConfigKindView::Password,
            ConfigFieldKind::Number { min, max } => ConfigKindView::Number { min: *min, max: *max },
            ConfigFieldKind::Bool      => ConfigKindView::Bool(
                f.value.as_bool().unwrap_or(false)
            ),
            ConfigFieldKind::Select { options } => ConfigKindView::Select {
                options: options.iter().map(|o| (o.value.clone(), o.label.clone())).collect(),
            },
            ConfigFieldKind::Port     => ConfigKindView::Port,
            ConfigFieldKind::Path     => ConfigKindView::Path,
            ConfigFieldKind::Textarea => ConfigKindView::Textarea,
        };
        let value = match &f.value {
            ConfigValue::Text(s)   => s.clone(),
            ConfigValue::Bool(b)   => b.to_string(),
            ConfigValue::Number(n) => n.to_string(),
            ConfigValue::Port(p)   => p.to_string(),
            ConfigValue::Empty     => String::new(),
        };
        Self {
            key:           f.key.clone(),
            label:         f.label.clone(),
            help:          f.help.clone(),
            kind,
            value,
            required:      f.required,
            needs_restart: f.needs_restart,
            missing_help:  f.help.is_empty(),
        }
    }
}

/// Simplified field kind for the UI — avoids dragging ConfigFieldKind (non-PartialEq enums).
#[derive(Clone, PartialEq, Debug)]
pub enum ConfigKindView {
    Text,
    Password,
    Number { min: Option<f64>, max: Option<f64> },
    Bool(bool),
    Select { options: Vec<(String, String)> }, // (value, label)
    Port,
    Path,
    Textarea,
}

/// One sub-instance as display strings.
#[derive(Clone, PartialEq, Debug)]
pub struct InstanceView {
    pub id:          String,
    pub name:        String,
    pub status:      String,
    pub status_css:  String,
    pub is_running:  bool,
}

impl InstanceView {
    pub fn from_instance(i: &InstanceRef) -> Self {
        Self {
            id:         i.id.clone(),
            name:       i.name.clone(),
            status:     i.status.label().to_string(),
            status_css: i.status.css_class().to_string(),
            is_running: i.status.is_running(),
        }
    }
}

/// All data the Manager UI needs — extracted from a Manageable implementor.
#[derive(Clone, PartialEq, Debug)]
pub struct PackageViewModel {
    // ── Identity ──────────────────────────────────────────────────────────────
    pub id:          String,
    pub name:        String,
    pub version:     String,
    pub description: String,
    pub author:      String,
    pub icon:        String,
    pub category:    String,
    pub type_label:  String,
    pub type_css:    String,

    // ── State ─────────────────────────────────────────────────────────────────
    pub is_installed:   bool,
    pub status_label:   String,
    pub status_css:     String,
    pub is_running:     bool,
    pub is_healthy:     bool,
    pub health_checks:  Vec<HealthCheckView>,

    // ── Tabs ──────────────────────────────────────────────────────────────────
    pub config_fields:  Vec<ConfigFieldView>,
    pub build_fields:   Vec<ConfigFieldView>,

    // ── Multi-instance (Bot, Container, Bridge) ───────────────────────────────
    pub instances:      Vec<InstanceView>,
    pub has_instances:  bool,

    // ── Actions ───────────────────────────────────────────────────────────────
    pub can_start:   bool,
    pub can_stop:    bool,
    pub can_persist: bool,
}

impl PackageViewModel {
    /// Build a view model from a [`fs_pkg::manageable::Manageable`] implementor.
    pub fn from_manageable(pkg: &dyn fs_pkg::manageable::Manageable) -> Self {
        let meta        = pkg.meta();
        let run_status  = pkg.run_status();
        let health      = pkg.check_health();
        let instances   = pkg.instances();
        let config      = pkg.config_fields();
        let build       = pkg.build_fields();

        let (type_label, type_css) = Self::package_type_display(pkg.package_type());
        let instances_view: Vec<InstanceView> = instances.iter().map(InstanceView::from_instance).collect();
        let has_instances = !instances_view.is_empty();

        Self {
            id:          meta.id.to_string(),
            name:        meta.name.clone(),
            version:     meta.version.clone(),
            description: meta.description.clone(),
            author:      meta.author.clone(),
            icon:        meta.icon.clone(),
            category:    meta.category.clone(),
            type_label,
            type_css,

            is_installed:  pkg.is_installed(),
            status_label:  run_status.label().to_string(),
            status_css:    run_status.css_class().to_string(),
            is_running:    run_status.is_running(),
            is_healthy:    health.is_healthy(),
            health_checks: health.checks.iter().map(|c| HealthCheckView {
                name:    c.name.clone(),
                passed:  c.passed,
                message: c.message.clone(),
            }).collect(),

            config_fields: config.iter().map(ConfigFieldView::from_config_field).collect(),
            build_fields:  build.iter().map(ConfigFieldView::from_config_field).collect(),

            has_instances,
            instances: instances_view,

            can_start:   pkg.can_start(),
            can_stop:    pkg.can_stop(),
            can_persist: pkg.can_persist(),
        }
    }

    fn package_type_display(t: PackageType) -> (String, String) {
        let (label, css) = match t {
            PackageType::App       => ("App",       "fs-type--app"),
            PackageType::Container => ("Container", "fs-type--container"),
            PackageType::Bundle    => ("Bundle",    "fs-type--bundle"),
            PackageType::Language  => ("Language",  "fs-type--language"),
            PackageType::Theme     => ("Theme",     "fs-type--theme"),
            PackageType::Widget    => ("Widget",    "fs-type--widget"),
            PackageType::Bot       => ("Bot",       "fs-type--bot"),
            PackageType::Bridge    => ("Bridge",    "fs-type--bridge"),
            PackageType::Task      => ("Task",      "fs-type--task"),
        };
        (label.into(), css.into())
    }
}
