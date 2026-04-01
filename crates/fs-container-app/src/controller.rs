// controller.rs — ContainerAppController: MVC controller.
//
// Knows only the ContainerEngine trait — never QuadletManager directly.

use std::sync::{Arc, Mutex};

use fs_container::ContainerEngine;
use fs_error::FsError;

use crate::model::{ContainerAppModel, ContainerEntry};

// ── ContainerAppController ────────────────────────────────────────────────────

/// Shared controller — cheaply cloneable (Arc-backed).
pub struct ContainerAppController<E: ContainerEngine> {
    engine: Arc<E>,
    state: Arc<Mutex<ContainerAppModel>>,
}

impl<E: ContainerEngine + 'static> ContainerAppController<E> {
    #[must_use]
    pub fn new(engine: E) -> Self {
        Self {
            engine: Arc::new(engine),
            state: Arc::new(Mutex::new(ContainerAppModel::new())),
        }
    }

    /// Snapshot of the current model.
    #[must_use]
    pub fn snapshot(&self) -> ContainerAppModel {
        self.state.lock().unwrap().clone()
    }

    /// Reload the container list from the engine.
    pub async fn refresh(&self) {
        let services = self.engine.list().await.unwrap_or_default();
        let containers = services
            .into_iter()
            .map(|s| ContainerEntry {
                name: s.name.clone(),
                state_label: format!("{:?}", s.active_state),
            })
            .collect();
        let mut guard = self.state.lock().unwrap();
        guard.containers = containers;
        guard.loading = false;
    }

    /// Start a container service by name.
    pub async fn start(&self, name: &str) -> Result<(), FsError> {
        self.engine.start(name).await
    }

    /// Stop a container service by name.
    pub async fn stop(&self, name: &str) -> Result<(), FsError> {
        self.engine.stop(name).await
    }

    /// Retrieve log lines for a container.
    pub async fn logs(&self, name: &str, lines: usize) -> Vec<String> {
        self.engine.logs(name, lines).await.unwrap_or_default()
    }
}

impl<E: ContainerEngine + 'static> Clone for ContainerAppController<E> {
    fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
            state: Arc::clone(&self.state),
        }
    }
}

// ── StubEngine for tests ──────────────────────────────────────────────────────

#[cfg(test)]
pub(crate) struct StubEngine;

#[cfg(test)]
#[async_trait::async_trait]
impl ContainerEngine for StubEngine {
    async fn list(&self) -> Result<Vec<fs_container::systemctl::ServiceStatus>, FsError> {
        Ok(vec![fs_container::systemctl::ServiceStatus {
            name: "test-svc".into(),
            active_state: fs_container::systemctl::UnitActiveState::Active,
            sub_state: String::new(),
            description: String::new(),
        }])
    }
    async fn start(&self, _name: &str) -> Result<(), FsError> {
        Ok(())
    }
    async fn stop(&self, _name: &str) -> Result<(), FsError> {
        Ok(())
    }
    async fn restart(&self, _name: &str) -> Result<(), FsError> {
        Ok(())
    }
    async fn remove(&self, _name: &str) -> Result<(), FsError> {
        Ok(())
    }
    async fn status(&self, _name: &str) -> Result<fs_container::systemctl::ServiceStatus, FsError> {
        Ok(fs_container::systemctl::ServiceStatus {
            name: "test-svc".into(),
            active_state: fs_container::systemctl::UnitActiveState::Active,
            sub_state: String::new(),
            description: String::new(),
        })
    }
    async fn logs(&self, _name: &str, _lines: usize) -> Result<Vec<String>, FsError> {
        Ok(vec!["log line 1".into()])
    }
    async fn deploy(&self, _config: &fs_container::ServiceConfig) -> Result<(), FsError> {
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ctrl() -> ContainerAppController<StubEngine> {
        ContainerAppController::new(StubEngine)
    }

    #[tokio::test]
    async fn refresh_populates_list() {
        let ctrl = ctrl();
        ctrl.refresh().await;
        assert_eq!(ctrl.snapshot().containers.len(), 1);
    }

    #[tokio::test]
    async fn start_returns_ok() {
        assert!(ctrl().start("test-svc").await.is_ok());
    }

    #[tokio::test]
    async fn stop_returns_ok() {
        assert!(ctrl().stop("test-svc").await.is_ok());
    }

    #[tokio::test]
    async fn logs_returns_lines() {
        let lines = ctrl().logs("test-svc", 10).await;
        assert!(!lines.is_empty());
    }

    #[tokio::test]
    async fn snapshot_after_refresh_has_containers() {
        let ctrl = ctrl();
        ctrl.refresh().await;
        let snap = ctrl.snapshot();
        assert!(!snap.containers.is_empty());
        assert_eq!(snap.containers[0].name, "test-svc");
    }
}
