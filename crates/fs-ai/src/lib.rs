pub mod app;

pub use app::AiManagerApp;

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-ai (`ai.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &str { "fs-ai" }
    fn snippets(&self) -> &[(&str, &str)] { I18N_SNIPPETS }
}

// ── AiStatus ─────────────────────────────────────────────────────────────────

use fs_manager_ai::{AiEngine, LlmConfig, LlmEngine, LlmModel};

pub struct AiStatus;

impl AiStatus {
    fn engine() -> LlmEngine {
        LlmEngine::new(
            LlmConfig { model: LlmModel::Qwen3_4B, ..LlmConfig::default() },
            LlmEngine::default_binary(),
            LlmEngine::default_data_dir(),
        )
    }

    /// Returns `true` if the LLM engine binary is installed.
    pub fn is_installed() -> bool {
        Self::engine().is_installed()
    }

    /// Returns the OpenAI-compatible API base URL if the engine is running,
    /// e.g. `"http://127.0.0.1:1234/v1"`.
    pub fn api_url() -> Option<String> {
        match Self::engine().status() {
            fs_manager_ai::EngineStatus::Running { port } =>
                Some(format!("http://127.0.0.1:{port}/v1")),
            _ => None,
        }
    }
}

// ── Public shims ──────────────────────────────────────────────────────────────

pub fn is_ai_installed() -> bool       { AiStatus::is_installed() }
pub fn ai_api_url()      -> Option<String> { AiStatus::api_url() }
