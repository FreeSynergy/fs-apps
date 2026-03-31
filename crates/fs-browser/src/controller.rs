// controller.rs — BrowserController: the "C" in MVC.
//
// Knows only:
//   - WebEngine-Trait    (fs-web-engine::WebEngine)
//   - BookmarkStore-Trait (crate::bookmark::BookmarkStore)
//
// Never imports a concrete engine or concrete store — those are injected.

use std::sync::{Arc, Mutex};

use chrono::Utc;
use tracing::{info, warn};

use crate::{
    bookmark::{Bookmark, BookmarkStore},
    model::{BrowserModel, HistoryEntry},
};

// ── BrowserController ─────────────────────────────────────────────────────────

/// Business-logic layer of the browser.
///
/// Wraps the mutable [`BrowserModel`] behind a `Mutex` so gRPC handlers and
/// UI callbacks can share a single controller across async tasks.
pub struct BrowserController<S: BookmarkStore> {
    model: Mutex<BrowserModel>,
    bookmarks: Arc<S>,
}

impl<S: BookmarkStore> BrowserController<S> {
    /// Create a new controller backed by `bookmarks`.
    #[must_use]
    pub fn new(bookmarks: Arc<S>) -> Self {
        Self {
            model: Mutex::new(BrowserModel::new()),
            bookmarks,
        }
    }

    // ── Navigation ─────────────────────────────────────────────────────────────

    /// Navigate to `url`.
    ///
    /// Updates the model (loading state, history) and logs the navigation.
    /// In a full implementation this would drive the [`WebEngine`] view.
    ///
    /// # Panics
    ///
    /// Panics if the internal model mutex is poisoned (another thread panicked
    /// while holding the lock — should never happen in normal operation).
    pub fn open_url(&self, url: &str) {
        info!("browser: open {url}");
        self.model
            .lock()
            .expect("model mutex poisoned")
            .set_loading(url);
        // In production: engine.load_url(view_id, url);
        // For now we immediately mark as loaded (stub).
        self.model
            .lock()
            .expect("model mutex poisoned")
            .set_loaded(None);
    }

    /// Navigate backward.  Returns the new current URL or `None` if already at
    /// the beginning of history.
    ///
    /// # Panics
    ///
    /// Panics if the model mutex is poisoned.
    pub fn navigate_back(&self) -> Option<String> {
        let mut model = self.model.lock().expect("model mutex poisoned");
        let url = model.history.back().map(ToString::to_string);
        if let Some(ref u) = url {
            info!("browser: back → {u}");
            model.current_url = Some(u.clone());
        } else {
            warn!("browser: back — already at start");
        }
        url
    }

    /// Navigate forward.  Returns the new current URL or `None` if already at
    /// the end of history.
    ///
    /// # Panics
    ///
    /// Panics if the model mutex is poisoned.
    pub fn navigate_forward(&self) -> Option<String> {
        let mut model = self.model.lock().expect("model mutex poisoned");
        let url = model.history.forward().map(ToString::to_string);
        if let Some(ref u) = url {
            info!("browser: forward → {u}");
            model.current_url = Some(u.clone());
        } else {
            warn!("browser: forward — already at end");
        }
        url
    }

    /// Reload the current page.
    ///
    /// # Panics
    ///
    /// Panics if the model mutex is poisoned.
    pub fn reload(&self) {
        let url = self
            .model
            .lock()
            .expect("model mutex poisoned")
            .current_url
            .clone();
        if let Some(u) = url {
            info!("browser: reload {u}");
            self.open_url(&u);
        }
    }

    // ── State accessors ────────────────────────────────────────────────────────

    /// Return a snapshot of the current model state.
    ///
    /// # Panics
    ///
    /// Panics if the model mutex is poisoned.
    #[must_use]
    pub fn snapshot(&self) -> BrowserModel {
        self.model.lock().expect("model mutex poisoned").clone()
    }

    /// Return the navigation history as a list of [`HistoryEntry`] items.
    ///
    /// # Panics
    ///
    /// Panics if the model mutex is poisoned.
    #[must_use]
    pub fn history(&self) -> Vec<HistoryEntry> {
        let model = self.model.lock().expect("model mutex poisoned");
        let now = Utc::now().to_rfc3339();
        // NavigationHistory doesn't store timestamps — use current time as
        // approximation until a richer model is added.
        (0..model.history.len())
            .map(|_| HistoryEntry {
                url: model
                    .current_url
                    .clone()
                    .unwrap_or_else(|| "about:blank".into()),
                visited_at: now.clone(),
            })
            .collect()
    }

    // ── Bookmarks ──────────────────────────────────────────────────────────────

    /// Add a bookmark.
    pub async fn add_bookmark(&self, title: &str, url: &str) -> Bookmark {
        let b = self.bookmarks.add(title, url).await;
        info!("browser: bookmark added: {} → {}", b.title, b.url);
        b
    }

    /// Remove a bookmark by ID.
    pub async fn remove_bookmark(&self, id: &str) -> bool {
        let removed = self.bookmarks.remove(id).await;
        if removed {
            info!("browser: bookmark removed: {id}");
        }
        removed
    }

    /// List all bookmarks.
    pub async fn list_bookmarks(&self) -> Vec<Bookmark> {
        self.bookmarks.list().await
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::bookmark::InMemoryBookmarkStore;

    use super::*;

    fn ctrl() -> BrowserController<InMemoryBookmarkStore> {
        BrowserController::new(Arc::new(InMemoryBookmarkStore::new()))
    }

    #[test]
    fn open_url_updates_model() {
        let c = ctrl();
        c.open_url("https://freesynergy.net");
        let snap = c.snapshot();
        assert_eq!(snap.current_url.as_deref(), Some("https://freesynergy.net"));
    }

    #[test]
    fn back_returns_none_when_empty() {
        let c = ctrl();
        assert!(c.navigate_back().is_none());
    }

    #[test]
    fn back_after_two_navigations() {
        let c = ctrl();
        c.open_url("https://a.com");
        c.open_url("https://b.com");
        let prev = c.navigate_back();
        assert!(prev.is_some());
    }

    #[test]
    fn reload_reopens_current_url() {
        let c = ctrl();
        c.open_url("https://example.com");
        c.reload();
        assert_eq!(
            c.snapshot().current_url.as_deref(),
            Some("https://example.com")
        );
    }

    #[tokio::test]
    async fn add_and_list_bookmarks() {
        let c = ctrl();
        let b = c.add_bookmark("FS", "https://freesynergy.net").await;
        assert_eq!(b.url, "https://freesynergy.net");
        let list = c.list_bookmarks().await;
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn remove_bookmark() {
        let c = ctrl();
        let b = c.add_bookmark("FS", "https://freesynergy.net").await;
        assert!(c.remove_bookmark(&b.id).await);
        assert!(c.list_bookmarks().await.is_empty());
    }
}
