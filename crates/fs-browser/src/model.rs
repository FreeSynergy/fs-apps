// model.rs — BrowserModel: the current state of the browser.
//
// BrowserModel is the "M" in MVC.  It holds all observable state — current URL,
// page title, loading flag, and the navigation history.  Bookmarks are managed
// via BrowserController / BookmarkStore and passed in when the view needs them.

use fs_web_engine::NavigationHistory;
use serde::{Deserialize, Serialize};

use crate::bookmark::Bookmark;

// ── BrowserModel ──────────────────────────────────────────────────────────────

/// Observable state of the browser.
#[derive(Debug, Clone, Default)]
pub struct BrowserModel {
    /// The URL that is currently loaded (or being loaded).
    pub current_url: Option<String>,
    /// The `<title>` of the current page.
    pub current_title: Option<String>,
    /// Whether a page is currently being loaded.
    pub loading: bool,
    /// Linear navigation history (back / forward stack).
    pub history: NavigationHistory,
    /// Snapshot of bookmarks (refreshed by the controller when needed).
    pub bookmarks: Vec<Bookmark>,
}

impl BrowserModel {
    /// Create a blank browser state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that a URL has started loading.
    pub fn set_loading(&mut self, url: impl Into<String>) {
        let url = url.into();
        self.current_url = Some(url.clone());
        self.loading = true;
        if let Ok(parsed) = fs_web_engine::WebUrl::parse(&url) {
            self.history.push(parsed);
        }
    }

    /// Record that the current page finished loading.
    pub fn set_loaded(&mut self, title: Option<String>) {
        self.loading = false;
        self.current_title = title;
    }

    /// Record a load failure (stop spinner, keep URL).
    pub fn set_load_error(&mut self) {
        self.loading = false;
    }

    /// Update the current URL (e.g. after a redirect).
    pub fn set_current_url(&mut self, url: impl Into<String>) {
        self.current_url = Some(url.into());
    }
}

// ── HistoryEntry ──────────────────────────────────────────────────────────────

/// A single item in the browser's navigation history as reported to the UI /
/// gRPC layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct HistoryEntry {
    /// The URL that was visited.
    pub url: String,
    /// RFC-3339 timestamp of the visit.
    pub visited_at: String,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_model_is_empty() {
        let m = BrowserModel::new();
        assert!(m.current_url.is_none());
        assert!(m.current_title.is_none());
        assert!(!m.loading);
        assert!(m.history.is_empty());
    }

    #[test]
    fn set_loading_updates_state() {
        let mut m = BrowserModel::new();
        m.set_loading("https://example.com");
        assert_eq!(m.current_url.as_deref(), Some("https://example.com"));
        assert!(m.loading);
    }

    #[test]
    fn set_loaded_clears_spinner() {
        let mut m = BrowserModel::new();
        m.set_loading("https://example.com");
        m.set_loaded(Some("Example".into()));
        assert!(!m.loading);
        assert_eq!(m.current_title.as_deref(), Some("Example"));
    }

    #[test]
    fn set_load_error_clears_spinner() {
        let mut m = BrowserModel::new();
        m.set_loading("https://bad.example");
        m.set_load_error();
        assert!(!m.loading);
        // URL is preserved so the user sees what failed.
        assert_eq!(m.current_url.as_deref(), Some("https://bad.example"));
    }

    #[test]
    fn history_grows_on_navigation() {
        let mut m = BrowserModel::new();
        m.set_loading("https://a.com");
        m.set_loaded(None);
        m.set_loading("https://b.com");
        m.set_loaded(None);
        assert_eq!(m.history.len(), 2);
        assert!(m.history.can_go_back());
    }
}
