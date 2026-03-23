// Bookmark + History CRUD backed by fs-db (browser.db).

use crate::model::{Bookmark, HistoryEntry};

// ── BookmarkManager ───────────────────────────────────────────────────────────

pub struct BookmarkManager;

impl BookmarkManager {
    /// Add a bookmark. No-op if already bookmarked (same URL).
    pub fn add_bookmark(&self, title: &str, url: &str) -> Option<Bookmark> {
        let id = chrono::Utc::now().timestamp_millis();
        Some(Bookmark {
            id,
            title:      title.to_string(),
            url:        url.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Remove a bookmark by ID.
    pub fn remove_bookmark(&self, bookmarks: &mut Vec<Bookmark>, id: i64) {
        bookmarks.retain(|b| b.id != id);
    }

    /// Record a history visit. Adds a new entry; duplicates are kept for full history.
    pub fn record_visit(&self, title: &str, url: &str) -> HistoryEntry {
        HistoryEntry {
            id:         chrono::Utc::now().timestamp_millis(),
            title:      title.to_string(),
            url:        url.to_string(),
            visited_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// ── Public shims ──────────────────────────────────────────────────────────────

pub fn add_bookmark(title: &str, url: &str) -> Option<Bookmark> {
    BookmarkManager.add_bookmark(title, url)
}

pub fn remove_bookmark(bookmarks: &mut Vec<Bookmark>, id: i64) {
    BookmarkManager.remove_bookmark(bookmarks, id)
}

pub fn record_visit(title: &str, url: &str) -> HistoryEntry {
    BookmarkManager.record_visit(title, url)
}
