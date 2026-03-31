// bookmark.rs — Bookmark domain type + BookmarkStore trait (Repository Pattern).

use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Bookmark ──────────────────────────────────────────────────────────────────

/// A saved browser bookmark.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct Bookmark {
    /// Unique identifier.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Bookmarked URL.
    pub url: String,
    /// When the bookmark was created.
    pub created_at: DateTime<Utc>,
}

impl Bookmark {
    /// Create a new bookmark with a generated ID and current timestamp.
    #[must_use]
    pub fn new(title: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            url: url.into(),
            created_at: Utc::now(),
        }
    }
}

// ── BookmarkStore ─────────────────────────────────────────────────────────────

/// Persistence abstraction for bookmarks (Repository Pattern).
///
/// The concrete backend is injected by the calling code — `BrowserController`
/// knows only this trait, never the concrete store.
#[async_trait]
pub trait BookmarkStore: Send + Sync + 'static {
    /// Persist a new bookmark and return it (with generated ID).
    async fn add(&self, title: &str, url: &str) -> Bookmark;

    /// Remove a bookmark by ID.  Returns `true` if it was present.
    async fn remove(&self, id: &str) -> bool;

    /// Return all bookmarks.
    async fn list(&self) -> Vec<Bookmark>;

    /// Find a bookmark by its URL, if any.
    async fn find_by_url(&self, url: &str) -> Option<Bookmark>;
}

// ── InMemoryBookmarkStore ─────────────────────────────────────────────────────

/// In-memory [`BookmarkStore`] backed by a `Mutex<Vec<Bookmark>>`.
///
/// Used in tests, CLI mode, and as the default until a DB backend is wired.
pub struct InMemoryBookmarkStore {
    bookmarks: Mutex<Vec<Bookmark>>,
}

impl InMemoryBookmarkStore {
    /// Create an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bookmarks: Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryBookmarkStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BookmarkStore for InMemoryBookmarkStore {
    async fn add(&self, title: &str, url: &str) -> Bookmark {
        let b = Bookmark::new(title, url);
        self.bookmarks
            .lock()
            .expect("bookmark mutex poisoned")
            .push(b.clone());
        b
    }

    async fn remove(&self, id: &str) -> bool {
        let mut guard = self.bookmarks.lock().expect("bookmark mutex poisoned");
        let before = guard.len();
        guard.retain(|b| b.id != id);
        guard.len() < before
    }

    async fn list(&self) -> Vec<Bookmark> {
        self.bookmarks
            .lock()
            .expect("bookmark mutex poisoned")
            .clone()
    }

    async fn find_by_url(&self, url: &str) -> Option<Bookmark> {
        self.bookmarks
            .lock()
            .expect("bookmark mutex poisoned")
            .iter()
            .find(|b| b.url == url)
            .cloned()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn add_and_list() {
        let s = InMemoryBookmarkStore::new();
        let b = s.add("FreeSynergy", "https://freesynergy.net").await;
        assert_eq!(b.title, "FreeSynergy");
        assert_eq!(b.url, "https://freesynergy.net");

        let list = s.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, b.id);
    }

    #[tokio::test]
    async fn remove_existing() {
        let s = InMemoryBookmarkStore::new();
        let b = s.add("Test", "https://example.com").await;
        assert!(s.remove(&b.id).await);
        assert!(s.list().await.is_empty());
    }

    #[tokio::test]
    async fn remove_nonexistent_returns_false() {
        let s = InMemoryBookmarkStore::new();
        assert!(!s.remove("no-such-id").await);
    }

    #[tokio::test]
    async fn find_by_url_returns_match() {
        let s = InMemoryBookmarkStore::new();
        s.add("A", "https://a.com").await;
        s.add("B", "https://b.com").await;
        let found = s.find_by_url("https://b.com").await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "B");
    }

    #[tokio::test]
    async fn find_by_url_returns_none_for_missing() {
        let s = InMemoryBookmarkStore::new();
        assert!(s.find_by_url("https://nope.com").await.is_none());
    }

    #[tokio::test]
    async fn multiple_bookmarks_order_preserved() {
        let s = InMemoryBookmarkStore::new();
        s.add("First", "https://first.com").await;
        s.add("Second", "https://second.com").await;
        s.add("Third", "https://third.com").await;
        let list = s.list().await;
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].title, "First");
        assert_eq!(list[2].title, "Third");
    }
}
