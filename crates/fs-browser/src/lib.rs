//! `fs-browser` — web browser for `FreeSynergy`.
//!
//! A renderer-agnostic browser built on the MVC pattern:
//!
//! - [`BrowserModel`]      — observable state (URL, title, loading, history)
//! - [`BrowserController`] — navigation + bookmark logic (knows only traits)
//! - [`BrowserView`]       — `FsView` impl bridging model to `fs-render`
//!
//! The concrete web engine (`fs-web-engine-servo`, future Blitz, …) is
//! injected via the [`WebEngine`] trait.  Bookmarks are persisted through the
//! [`BookmarkStore`] trait.
//!
//! # Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use fs_browser::{BrowserController, InMemoryBookmarkStore};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let ctrl = BrowserController::new(Arc::new(InMemoryBookmarkStore::new()));
//! ctrl.open_url("https://freesynergy.net");
//! # }
//! ```

#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

pub mod bookmark;
pub mod cli;
pub mod controller;
pub mod grpc;
pub mod keys;
pub mod model;
pub mod rest;
pub mod view;

pub use bookmark::{Bookmark, BookmarkStore, InMemoryBookmarkStore};
pub use controller::BrowserController;
pub use model::{BrowserModel, HistoryEntry};
pub use view::BrowserView;
