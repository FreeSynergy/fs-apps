// rest.rs — REST API + OpenAPI for fs-browser (axum + utoipa).
#![allow(clippy::needless_for_each)]
//
// Endpoints:
//   GET  /api/v1/bookmarks
//   POST /api/v1/bookmarks
//   DELETE /api/v1/bookmarks/{id}
//   GET  /api/v1/history
//   POST /api/v1/navigate

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::bookmark::Bookmark;
use crate::{bookmark::BookmarkStore, controller::BrowserController, model::HistoryEntry};

// ── Shared state ──────────────────────────────────────────────────────────────

pub struct AppState<S: BookmarkStore> {
    pub controller: Arc<BrowserController<S>>,
}

impl<S: BookmarkStore> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            controller: Arc::clone(&self.controller),
        }
    }
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddBookmarkBody {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NavigateBody {
    /// Direction: "back", "forward", or "reload".
    pub direction: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NavigateResult {
    pub ok: bool,
    pub current_url: Option<String>,
}

// ── OpenAPI document ──────────────────────────────────────────────────────────

#[derive(OpenApi)]
#[openapi(
    paths(
        list_bookmarks,
        add_bookmark,
        remove_bookmark,
        get_history,
        navigate,
    ),
    components(schemas(
        Bookmark,
        AddBookmarkBody,
        NavigateBody,
        NavigateResult,
        HistoryEntry,
    )),
    tags((name = "browser", description = "FreeSynergy Browser API"))
)]
pub struct ApiDoc;

// ── Router ────────────────────────────────────────────────────────────────────

/// Build the axum router with all REST endpoints and the Swagger UI.
pub fn router<S: BookmarkStore>(controller: Arc<BrowserController<S>>) -> Router {
    let state = AppState { controller };
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/v1/bookmarks", get(list_bookmarks::<S>))
        .route("/api/v1/bookmarks", post(add_bookmark::<S>))
        .route("/api/v1/bookmarks/:id", delete(remove_bookmark::<S>))
        .route("/api/v1/history", get(get_history::<S>))
        .route("/api/v1/navigate", post(navigate::<S>))
        .with_state(state)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// List all saved bookmarks.
#[utoipa::path(
    get,
    path = "/api/v1/bookmarks",
    responses(
        (status = 200, description = "All bookmarks", body = Vec<Bookmark>)
    ),
    tag = "browser"
)]
async fn list_bookmarks<S: BookmarkStore>(State(s): State<AppState<S>>) -> impl IntoResponse {
    Json(s.controller.list_bookmarks().await)
}

/// Add a new bookmark.
#[utoipa::path(
    post,
    path = "/api/v1/bookmarks",
    request_body = AddBookmarkBody,
    responses(
        (status = 201, description = "Bookmark created", body = Bookmark)
    ),
    tag = "browser"
)]
async fn add_bookmark<S: BookmarkStore>(
    State(s): State<AppState<S>>,
    Json(body): Json<AddBookmarkBody>,
) -> impl IntoResponse {
    let b = s.controller.add_bookmark(&body.title, &body.url).await;
    (StatusCode::CREATED, Json(b))
}

/// Remove a bookmark by ID.
#[utoipa::path(
    delete,
    path = "/api/v1/bookmarks/{id}",
    params(("id" = String, Path, description = "Bookmark ID")),
    responses(
        (status = 200, description = "Removed flag", body = bool),
        (status = 404, description = "Not found")
    ),
    tag = "browser"
)]
async fn remove_bookmark<S: BookmarkStore>(
    State(s): State<AppState<S>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let removed = s.controller.remove_bookmark(&id).await;
    if removed {
        (StatusCode::OK, Json(true)).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(false)).into_response()
    }
}

/// Return the navigation history.
#[utoipa::path(
    get,
    path = "/api/v1/history",
    responses(
        (status = 200, description = "Navigation history", body = Vec<HistoryEntry>)
    ),
    tag = "browser"
)]
async fn get_history<S: BookmarkStore>(State(s): State<AppState<S>>) -> impl IntoResponse {
    Json(s.controller.history())
}

/// Navigate (back / forward / reload).
#[utoipa::path(
    post,
    path = "/api/v1/navigate",
    request_body = NavigateBody,
    responses(
        (status = 200, description = "Navigation result", body = NavigateResult)
    ),
    tag = "browser"
)]
async fn navigate<S: BookmarkStore>(
    State(s): State<AppState<S>>,
    Json(body): Json<NavigateBody>,
) -> impl IntoResponse {
    let current_url = match body.direction.as_str() {
        "back" => s.controller.navigate_back(),
        "forward" => s.controller.navigate_forward(),
        _ => {
            s.controller.reload();
            s.controller.snapshot().current_url
        }
    };
    Json(NavigateResult {
        ok: true,
        current_url,
    })
}
