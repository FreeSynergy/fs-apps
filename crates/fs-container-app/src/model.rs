// model.rs — ContainerAppModel: observable state of the container manager.

use serde::{Deserialize, Serialize};

// ── ContainerEntry ────────────────────────────────────────────────────────────

/// Runtime snapshot of a single managed container service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ContainerEntry {
    /// Systemd service name (e.g. `kanidm`).
    pub name: String,
    /// Human-readable status label (e.g. `"active"`, `"failed"`).
    pub state_label: String,
}

// ── ContainerAppModel ─────────────────────────────────────────────────────────

/// Observable state of the container app.
#[derive(Debug, Clone, Default)]
pub struct ContainerAppModel {
    /// Currently known container services.
    pub containers: Vec<ContainerEntry>,
    /// Name of the container being operated on (start/stop/logs).
    pub active: Option<String>,
    /// Whether a reload is in progress.
    pub loading: bool,
}

impl ContainerAppModel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_is_empty() {
        let m = ContainerAppModel::new();
        assert!(m.containers.is_empty());
        assert!(m.active.is_none());
        assert!(!m.loading);
    }

    #[test]
    fn container_entry_fields() {
        let entry = ContainerEntry {
            name: "kanidm".into(),
            state_label: "active".into(),
        };
        assert_eq!(entry.name, "kanidm");
        assert_eq!(entry.state_label, "active");
    }
}
