// controller.rs — AiController: Facade over the LLM engine.
//
// Knows only the fs-manager-ai AiEngine trait — never the concrete implementation.

use std::sync::{Arc, Mutex};

use fs_manager_ai::{AiEngine, EngineStatus, LlmConfig, LlmEngine, LlmModel};

use crate::model::{AiModel, KnownModel};

// ── AiController ─────────────────────────────────────────────────────────────

/// Shared controller — cheaply cloneable (Arc-backed).
#[derive(Clone)]
pub struct AiController {
    state: Arc<Mutex<AiModel>>,
}

impl AiController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AiModel::new())),
        }
    }

    /// Snapshot of the current model state.
    #[must_use]
    pub fn snapshot(&self) -> AiModel {
        self.state.lock().unwrap().clone()
    }

    /// List all known models.
    #[must_use]
    pub fn list_models(&self) -> Vec<KnownModel> {
        KnownModel::all()
    }

    /// Start the LLM engine with the given model id.
    ///
    /// Returns `Ok(port)` on success or an error string.
    pub fn start(&self, model_id: &str) -> Result<u16, String> {
        let llm_model = Self::model_from_id(model_id)?;
        let config = LlmConfig {
            model: llm_model,
            ..LlmConfig::default()
        };
        let engine = LlmEngine::new(
            config,
            LlmEngine::default_binary(),
            LlmEngine::default_data_dir(),
        );
        engine.start().map_err(|e| e.to_string())?;

        let EngineStatus::Running { port } = engine.status() else {
            return Err("engine did not start".into());
        };

        let mut state = self.state.lock().unwrap();
        state.set_running(port);
        state.active_model = Some(model_id.to_string());
        Ok(port)
    }

    /// Stop the LLM engine.
    pub fn stop(&self) -> Result<(), String> {
        let snapshot = self.snapshot();
        let model_id = snapshot.active_model.as_deref().ok_or("no active model")?;

        let llm_model = Self::model_from_id(model_id)?;
        let config = LlmConfig {
            model: llm_model,
            ..LlmConfig::default()
        };
        let engine = LlmEngine::new(
            config,
            LlmEngine::default_binary(),
            LlmEngine::default_data_dir(),
        );
        engine.stop().map_err(|e| e.to_string())?;

        let mut state = self.state.lock().unwrap();
        state.set_stopped();
        state.active_model = None;
        Ok(())
    }

    fn model_from_id(id: &str) -> Result<LlmModel, String> {
        match id {
            "qwen3-4b" => Ok(LlmModel::Qwen3_4B),
            "qwen3-8b" => Ok(LlmModel::Qwen3_8B),
            "qwen2.5-coder-7b" => Ok(LlmModel::Qwen25Coder7B),
            other => Err(format!("unknown model: {other}")),
        }
    }
}

impl Default for AiController {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_controller_is_stopped() {
        let ctrl = AiController::new();
        let snap = ctrl.snapshot();
        assert!(!snap.running);
    }

    #[test]
    fn list_models_returns_entries() {
        let ctrl = AiController::new();
        assert!(!ctrl.list_models().is_empty());
    }

    #[test]
    fn unknown_model_returns_error() {
        let ctrl = AiController::new();
        let result = ctrl.start("nonexistent-model");
        assert!(result.is_err());
    }
}
