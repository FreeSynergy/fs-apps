/// Ollama AI client — natural language → module metadata.
///
/// Talks to a local Ollama instance at `http://localhost:11434`.
/// Used by the Studio Module Builder to enrich generated module definitions
/// with AI-suggested names, descriptions, service types, and tags.
use serde::{Deserialize, Serialize};

/// Default Ollama base URL (local instance).
pub const OLLAMA_BASE_URL: &str = "http://localhost:11434";

/// Default model to use for module metadata generation.
pub const DEFAULT_MODEL: &str = "llama3.2";

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    format: &'a str,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

/// AI-suggested module metadata.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ModuleMetadataSuggestion {
    /// Suggested module name (snake_case).
    pub name: String,
    /// Short description (1-2 sentences).
    pub description: String,
    /// FSN service type (e.g. "git", "mail", "wiki").
    pub service_type: String,
    /// Suggested tags.
    pub tags: Vec<String>,
    /// Health check path (e.g. "/health" or "/api/health").
    pub health_path: String,
    /// Whether the service has a web UI.
    pub has_web_ui: bool,
}

// ── OllamaClient ─────────────────────────────────────────────────────────────

/// Thin async client for the Ollama HTTP API.
#[derive(Clone)]
pub struct OllamaClient {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new(OLLAMA_BASE_URL, DEFAULT_MODEL)
    }
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Check whether Ollama is running and reachable.
    pub async fn is_available(&self) -> bool {
        self.client.get(format!("{}/api/tags", self.base_url))
            .send().await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Ask Ollama to suggest module metadata from a natural language description.
    ///
    /// Returns structured JSON parsed into `ModuleMetadataSuggestion`.
    pub async fn suggest_metadata(
        &self,
        description: &str,
    ) -> Result<ModuleMetadataSuggestion, String> {
        let prompt = build_prompt(description);

        let req = OllamaGenerateRequest {
            model: &self.model,
            prompt: &prompt,
            stream: false,
            format: "json",
        };

        let resp = self.client
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {e}"))?;

        let body: OllamaGenerateResponse = resp.json()
            .await
            .map_err(|e| format!("Ollama response parse failed: {e}"))?;

        serde_json::from_str::<ModuleMetadataSuggestion>(&body.response)
            .map_err(|e| format!("Metadata JSON parse failed: {e}\nRaw: {}", body.response))
    }
}

fn build_prompt(description: &str) -> String {
    format!(
        r#"You are a DevOps assistant that generates FreeSynergy module metadata.

Given the following service description, output a JSON object with these fields:
- "name": snake_case module name (short, e.g. "my_service")
- "description": one or two sentences describing what the service does
- "service_type": one of: proxy, iam, mail, git, wiki, chat, collab, tasks, tickets, maps, monitoring, database, cache, custom
- "tags": array of relevant tags (max 5)
- "health_path": HTTP health check path (e.g. "/health", "/api/health", "/ping")
- "has_web_ui": true if the service has a browser UI, false otherwise

Service description:
{description}

Respond ONLY with valid JSON. No explanation, no markdown, just the JSON object."#
    )
}

// ── Available models ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    name: String,
}

/// List models available in the local Ollama instance.
pub async fn list_ollama_models(base_url: &str) -> Vec<String> {
    let client = reqwest::Client::new();
    let Ok(resp) = client.get(format!("{base_url}/api/tags")).send().await else {
        return Vec::new();
    };
    let Ok(body) = resp.json::<TagsResponse>().await else {
        return Vec::new();
    };
    body.models.into_iter().map(|m| m.name).collect()
}
