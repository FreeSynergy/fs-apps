// grpc.rs — gRPC service implementation for fs-browser.
//
// Wraps Arc<BrowserController<S>> and exposes it via the BrowserService proto.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::instrument;

use crate::{
    bookmark::{Bookmark, BookmarkStore},
    controller::BrowserController,
};

// Include the generated tonic code.
pub mod proto {
    #![allow(clippy::all, clippy::pedantic, warnings)]
    tonic::include_proto!("browser");
}

pub use proto::browser_service_server::{BrowserService, BrowserServiceServer};
pub use proto::{
    AddBookmarkRequest, AddBookmarkResponse, BookmarkProto, GetHistoryRequest, GetHistoryResponse,
    HealthRequest, HealthResponse, HistoryEntryProto, ListBookmarksRequest, ListBookmarksResponse,
    NavigateRequest, NavigateResponse, NavigationDirection, OpenUrlRequest, OpenUrlResponse,
    RemoveBookmarkRequest, RemoveBookmarkResponse,
};

// ── Conversions ───────────────────────────────────────────────────────────────

fn bookmark_to_proto(b: &Bookmark) -> BookmarkProto {
    BookmarkProto {
        id: b.id.clone(),
        title: b.title.clone(),
        url: b.url.clone(),
        created_at: b.created_at.to_rfc3339(),
    }
}

// ── GrpcBrowser ───────────────────────────────────────────────────────────────

/// gRPC service wrapper around a shared [`BrowserController`].
pub struct GrpcBrowser<S: BookmarkStore> {
    controller: Arc<BrowserController<S>>,
}

impl<S: BookmarkStore> GrpcBrowser<S> {
    /// Wrap `controller` in a gRPC service.
    #[must_use]
    pub fn new(controller: Arc<BrowserController<S>>) -> Self {
        Self { controller }
    }
}

#[tonic::async_trait]
impl<S: BookmarkStore + Send + Sync + 'static> BrowserService for GrpcBrowser<S> {
    #[instrument(name = "grpc.browser.open_url", skip(self))]
    async fn open_url(
        &self,
        req: Request<OpenUrlRequest>,
    ) -> Result<Response<OpenUrlResponse>, Status> {
        let url = req.into_inner().url;
        self.controller.open_url(&url);
        Ok(Response::new(OpenUrlResponse {
            ok: true,
            message: format!("Navigated to {url}"),
        }))
    }

    #[instrument(name = "grpc.browser.navigate", skip(self))]
    async fn navigate(
        &self,
        req: Request<NavigateRequest>,
    ) -> Result<Response<NavigateResponse>, Status> {
        let direction = NavigationDirection::try_from(req.into_inner().direction)
            .unwrap_or(NavigationDirection::Reload);

        let current_url = match direction {
            NavigationDirection::Back => self.controller.navigate_back(),
            NavigationDirection::Forward => self.controller.navigate_forward(),
            NavigationDirection::Reload => {
                self.controller.reload();
                self.controller.snapshot().current_url
            }
        };
        Ok(Response::new(NavigateResponse {
            ok: true,
            current_url,
        }))
    }

    #[instrument(name = "grpc.browser.get_history", skip(self))]
    async fn get_history(
        &self,
        _req: Request<GetHistoryRequest>,
    ) -> Result<Response<GetHistoryResponse>, Status> {
        let entries = self
            .controller
            .history()
            .into_iter()
            .map(|e| HistoryEntryProto {
                url: e.url,
                visited_at: e.visited_at,
            })
            .collect();
        Ok(Response::new(GetHistoryResponse { entries }))
    }

    #[instrument(name = "grpc.browser.list_bookmarks", skip(self))]
    async fn list_bookmarks(
        &self,
        _req: Request<ListBookmarksRequest>,
    ) -> Result<Response<ListBookmarksResponse>, Status> {
        let bookmarks = self
            .controller
            .list_bookmarks()
            .await
            .iter()
            .map(bookmark_to_proto)
            .collect();
        Ok(Response::new(ListBookmarksResponse { bookmarks }))
    }

    #[instrument(name = "grpc.browser.add_bookmark", skip(self))]
    async fn add_bookmark(
        &self,
        req: Request<AddBookmarkRequest>,
    ) -> Result<Response<AddBookmarkResponse>, Status> {
        let r = req.into_inner();
        let bookmark = self.controller.add_bookmark(&r.title, &r.url).await;
        Ok(Response::new(AddBookmarkResponse {
            bookmark: Some(bookmark_to_proto(&bookmark)),
        }))
    }

    #[instrument(name = "grpc.browser.remove_bookmark", skip(self))]
    async fn remove_bookmark(
        &self,
        req: Request<RemoveBookmarkRequest>,
    ) -> Result<Response<RemoveBookmarkResponse>, Status> {
        let id = req.into_inner().id;
        let removed = self.controller.remove_bookmark(&id).await;
        Ok(Response::new(RemoveBookmarkResponse { removed }))
    }

    #[instrument(name = "grpc.browser.health", skip(self))]
    async fn health(
        &self,
        _req: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            ok: true,
            version: env!("CARGO_PKG_VERSION").into(),
        }))
    }
}
