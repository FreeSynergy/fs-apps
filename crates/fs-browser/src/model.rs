// Browser data model: tabs, bookmarks, history entries.

use serde::{Deserialize, Serialize};

/// A single browser tab.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowserTab {
    pub id:    u32,
    pub title: String,
    pub url:   String,
    /// `true` while the page is loading.
    pub loading: bool,
}

impl BrowserTab {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            title:   "New Tab".to_string(),
            url:     String::new(),
            loading: false,
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        self.title = url.clone();
        self.url   = url;
        self
    }

    /// Navigate to `url`, updating title from the URL string.
    pub fn navigate(&mut self, url: String) {
        self.title = url.chars().take(32).collect();
        self.url   = url;
    }

    /// Reset to an empty new-tab state.
    pub fn reset(&mut self) {
        self.url   = String::new();
        self.title = "New Tab".to_string();
    }
}

/// A bookmarked URL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id:         i64,
    pub title:      String,
    pub url:        String,
    pub created_at: String,
}

/// A history entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id:         i64,
    pub title:      String,
    pub url:        String,
    pub visited_at: String,
}

/// A download tracked by the browser.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadEntry {
    pub id:          i64,
    pub filename:    String,
    pub url:         String,
    /// S3 destination path (e.g. `/shared/downloads/file.zip`).
    pub s3_path:     String,
    pub status:      DownloadStatus,
    pub started_at:  String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Saving,
    Done,
    Error(String),
}

impl DownloadStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Pending => "Pending",
            Self::Saving  => "Saving…",
            Self::Done    => "Done",
            Self::Error(_) => "Error",
        }
    }
}
