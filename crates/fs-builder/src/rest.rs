// rest.rs — REST + OpenAPI routes for fs-builder.

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use utoipa::OpenApi;

use crate::controller::BuilderController;
use crate::model::BuildPipeline;

// ── OpenAPI doc ───────────────────────────────────────────────────────────────

#[allow(clippy::needless_for_each)]
#[derive(OpenApi)]
#[openapi(paths(list_pipelines, health), components(schemas(BuildPipeline)))]
pub struct ApiDoc;

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router(ctrl: BuilderController) -> Router {
    Router::new()
        .route("/builder/pipelines", get(list_pipelines))
        .route("/builder/health", get(health))
        .with_state(ctrl)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// List all active build pipelines.
#[utoipa::path(get, path = "/builder/pipelines", responses((status = 200, body = Vec<BuildPipeline>)))]
async fn list_pipelines(State(ctrl): State<BuilderController>) -> Json<Vec<BuildPipeline>> {
    Json(ctrl.list())
}

/// Health check.
#[utoipa::path(get, path = "/builder/health", responses((status = 200)))]
async fn health(_: State<BuilderController>) -> StatusCode {
    StatusCode::OK
}
