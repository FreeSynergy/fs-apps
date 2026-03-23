// fs-browser/search_engine.rs — SearchEngine, SearchEngineRegistry, BrowserConfig.
//
// Design: Registry Pattern for the engine list + plain config struct persisted
// to ~/.config/fsn/browser.toml (same pattern as DesktopConfig).
//
// Privacy-respecting engines only.
// Google, Bing, and Yandex are deliberately excluded.

use serde::{Deserialize, Serialize};

// ── SearchEngine ──────────────────────────────────────────────────────────────

/// A search engine with a URL template.
///
/// The template must contain `{query}` as a placeholder for the
/// URL-encoded search term, e.g.:
/// `"https://search.brave.com/search?q={query}"`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchEngine {
    /// Stable identifier (e.g. `"brave"`).
    pub id: String,
    /// Display name shown in settings (e.g. `"Brave Search"`).
    pub name: String,
    /// URL template — `{query}` is replaced with the URL-encoded search term.
    pub url_template: String,
}

impl SearchEngine {
    pub fn new(
        id:           impl Into<String>,
        name:         impl Into<String>,
        url_template: impl Into<String>,
    ) -> Self {
        Self {
            id:           id.into(),
            name:         name.into(),
            url_template: url_template.into(),
        }
    }

    /// Build a full search URL for the given query string.
    pub fn build_url(&self, query: &str) -> String {
        self.url_template.replace("{query}", &url_encode(query))
    }
}

// ── SearchEngineRegistry ──────────────────────────────────────────────────────

/// Registry of available search engines.
///
/// All built-in engines respect user privacy.
/// No Google, Bing, or Yandex.
pub struct SearchEngineRegistry;

impl SearchEngineRegistry {
    /// All built-in search engines, in display order.
    pub fn all() -> Vec<SearchEngine> {
        vec![
            SearchEngine::new(
                "brave",
                "Brave Search",
                "https://search.brave.com/search?q={query}",
            ),
            SearchEngine::new(
                "ecosia",
                "Ecosia",
                "https://www.ecosia.org/search?q={query}",
            ),
            SearchEngine::new(
                "startpage",
                "Startpage",
                "https://www.startpage.com/search?q={query}",
            ),
            SearchEngine::new(
                "kagi",
                "Kagi",
                "https://kagi.com/search?q={query}",
            ),
            SearchEngine::new(
                "duckduckgo",
                "DuckDuckGo",
                "https://duckduckgo.com/?q={query}",
            ),
            SearchEngine::new(
                "searxng",
                "SearXNG (local)",
                "http://localhost:8888/search?q={query}",
            ),
        ]
    }

    /// Find an engine by ID, falling back to Brave Search.
    pub fn find(id: &str) -> SearchEngine {
        Self::all()
            .into_iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| SearchEngine::new(
                "brave",
                "Brave Search",
                "https://search.brave.com/search?q={query}",
            ))
    }
}

// ── BrowserConfig ─────────────────────────────────────────────────────────────

/// Browser configuration — persisted to `~/.config/fsn/browser.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// ID of the active search engine (e.g. `"brave"`).
    #[serde(default = "default_engine_id")]
    pub search_engine: String,
}

fn default_engine_id() -> String { "brave".into() }

impl Default for BrowserConfig {
    fn default() -> Self {
        Self { search_engine: default_engine_id() }
    }
}

impl BrowserConfig {
    pub fn load() -> Self {
        let path = config_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = std::fs::write(&path, text);
        }
    }

    /// Returns the currently configured [`SearchEngine`].
    pub fn active_engine(&self) -> SearchEngine {
        SearchEngineRegistry::find(&self.search_engine)
    }

    /// Build a search URL using the configured engine.
    pub fn build_search_url(&self, query: &str) -> String {
        self.active_engine().build_url(query)
    }
}

fn config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("fsn")
        .join("browser.toml")
}

// ── URL encoding ──────────────────────────────────────────────────────────────

fn url_encode(s: &str) -> String {
    s.chars().map(|c| match c {
        ' '                                          => '+'.to_string(),
        c if c.is_alphanumeric() || "-_.~".contains(c) => c.to_string(),
        c                                            => format!("%{:02X}", c as u32),
    }).collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brave_is_default() {
        let cfg = BrowserConfig::default();
        assert_eq!(cfg.active_engine().id, "brave");
    }

    #[test]
    fn build_url_encodes_query() {
        let engine = SearchEngineRegistry::find("brave");
        let url = engine.build_url("kanidm identity provider");
        assert_eq!(url, "https://search.brave.com/search?q=kanidm+identity+provider");
    }

    #[test]
    fn unknown_engine_falls_back_to_brave() {
        let engine = SearchEngineRegistry::find("nonexistent");
        assert_eq!(engine.id, "brave");
    }

    #[test]
    fn all_engines_have_query_placeholder() {
        for e in SearchEngineRegistry::all() {
            assert!(
                e.url_template.contains("{query}"),
                "Engine '{}' is missing {{query}} placeholder",
                e.id,
            );
        }
    }
}
