// rest.rs — REST + OpenAPI routes for fs-container-app.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use fs_container::ContainerEngine;
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::controller::ContainerAppController;
use crate::model::ContainerEntry;

// ── OpenAPI doc ───────────────────────────────────────────────────────────────

#[allow(clippy::needless_for_each, clippy::ignored_unit_patterns)]
#[derive(OpenApi)]
#[openapi(
    paths(list_services, start_service, stop_service, get_logs),
    components(schemas(ContainerEntry))
)]
pub struct ApiDoc;

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct LogsQuery {
    #[serde(default = "default_lines")]
    pub lines: usize,
}

fn default_lines() -> usize {
    50
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router<E: ContainerEngine + 'static>(ctrl: ContainerAppController<E>) -> Router {
    Router::new()
        .route("/containers", get(list_services::<E>))
        .route("/containers/{name}/start", post(start_service::<E>))
        .route("/containers/{name}/stop", post(stop_service::<E>))
        .route("/containers/{name}/logs", get(get_logs::<E>))
        .with_state(ctrl)
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// List all managed container services.
#[utoipa::path(get, path = "/containers", responses((status = 200, body = Vec<ContainerEntry>)))]
async fn list_services<E: ContainerEngine + 'static>(
    State(ctrl): State<ContainerAppController<E>>,
) -> Json<Vec<ContainerEntry>> {
    ctrl.refresh().await;
    Json(ctrl.snapshot().containers)
}

/// Start a container service.
#[utoipa::path(
    post,
    path = "/containers/{name}/start",
    params(("name" = String, Path, description = "Service name")),
    responses((status = 200), (status = 400))
)]
async fn start_service<E: ContainerEngine + 'static>(
    State(ctrl): State<ContainerAppController<E>>,
    Path(name): Path<String>,
) -> StatusCode {
    match ctrl.start(&name).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

/// Stop a container service.
#[utoipa::path(
    post,
    path = "/containers/{name}/stop",
    params(("name" = String, Path, description = "Service name")),
    responses((status = 200), (status = 400))
)]
async fn stop_service<E: ContainerEngine + 'static>(
    State(ctrl): State<ContainerAppController<E>>,
    Path(name): Path<String>,
) -> StatusCode {
    match ctrl.stop(&name).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

/// Get log lines for a container.
#[utoipa::path(
    get,
    path = "/containers/{name}/logs",
    params(
        ("name" = String, Path, description = "Service name"),
        LogsQuery,
    ),
    responses((status = 200, body = Vec<String>))
)]
async fn get_logs<E: ContainerEngine + 'static>(
    State(ctrl): State<ContainerAppController<E>>,
    Path(name): Path<String>,
    Query(q): Query<LogsQuery>,
) -> Json<Vec<String>> {
    Json(ctrl.logs(&name, q.lines).await)
}
