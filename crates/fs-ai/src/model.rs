// model.rs — AiModel: observable state of the AI assistant.

use serde::{Deserialize, Serialize};

// ── AiModel ───────────────────────────────────────────────────────────────────

/// Current state of the AI assistant app.
#[derive(Debug, Clone, Default)]
pub struct AiModel {
    /// Whether the LLM engine is currently running.
    pub running: bool,
    /// Port the LLM engine is listening on (when running).
    pub port: Option<u16>,
    /// Active model ID.
    pub active_model: Option<String>,
    /// Whether an operation (start/stop) is in progress.
    pub busy: bool,
}

impl AiModel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// OpenAI-compatible API base URL, if the engine is running.
    #[must_use]
    pub fn api_url(&self) -> Option<String> {
        self.port.map(|p| format!("http://127.0.0.1:{p}/v1"))
    }

    pub fn set_running(&mut self, port: u16) {
        self.running = true;
        self.port = Some(port);
        self.busy = false;
    }

    pub fn set_stopped(&mut self) {
        self.running = false;
        self.port = None;
        self.busy = false;
    }
}

// ── KnownModel ────────────────────────────────────────────────────────────────

/// A model entry in the catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct KnownModel {
    pub id: String,
    pub name: String,
}

impl KnownModel {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                id: "qwen3-4b".into(),
                name: "Qwen 3 4B".into(),
            },
            Self {
                id: "qwen3-8b".into(),
                name: "Qwen 3 8B".into(),
            },
            Self {
                id: "qwen2.5-coder-7b".into(),
                name: "Qwen 2.5 Coder 7B".into(),
            },
        ]
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_model_is_stopped() {
        let m = AiModel::new();
        assert!(!m.running);
        assert!(m.port.is_none());
        assert!(m.api_url().is_none());
    }

    #[test]
    fn set_running_updates_state() {
        let mut m = AiModel::new();
        m.set_running(1234);
        assert!(m.running);
        assert_eq!(m.api_url(), Some("http://127.0.0.1:1234/v1".into()));
    }

    #[test]
    fn set_stopped_clears_state() {
        let mut m = AiModel::new();
        m.set_running(1234);
        m.set_stopped();
        assert!(!m.running);
        assert!(m.api_url().is_none());
    }

    #[test]
    fn known_models_not_empty() {
        assert!(!KnownModel::all().is_empty());
    }

    #[test]
    fn known_models_have_unique_ids() {
        let models = KnownModel::all();
        let mut ids: Vec<_> = models.iter().map(|m| &m.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), models.len());
    }
}
