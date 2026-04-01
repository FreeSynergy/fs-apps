// rest.rs — REST + OpenAPI routes for fs-lenses.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::controller::LensController;
use crate::model::Lens;

// ── OpenAPI doc ───────────────────────────────────────────────────────────────

#[allow(clippy::needless_for_each)] // triggered by utoipa macro internals
#[derive(OpenApi)]
#[openapi(
    paths(list_lenses, create_lens, delete_lens, query_lens),
    components(schemas(Lens, CreateLensBody, QueryLensBody, QueryLensResult))
)]
pub struct ApiDoc;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLensBody {
    pub name: String,
    pub query: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct QueryLensBody {
    pub query: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QueryLensResult {
    pub results: Vec<String>,
}

// ── Router ────────────────────────────────────────────────────────────────────

/// Build the axum router for the lenses REST API.
pub fn router(ctrl: LensController) -> Router {
    Router::new()
        .route("/lenses", get(list_lenses).post(create_lens))
        .route("/lenses/{id}", delete(delete_lens))
        .route("/lenses/{id}/query", post(query_lens))
        .with_state(ctrl)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// List all saved lenses.
#[utoipa::path(get, path = "/lenses", responses((status = 200, body = Vec<Lens>)))]
async fn list_lenses(State(ctrl): State<LensController>) -> Json<Vec<Lens>> {
    Json(ctrl.list())
}

/// Create a new lens.
#[utoipa::path(
    post,
    path = "/lenses",
    request_body = CreateLensBody,
    responses((status = 201, body = Lens))
)]
async fn create_lens(
    State(ctrl): State<LensController>,
    Json(body): Json<CreateLensBody>,
) -> (StatusCode, Json<Lens>) {
    let lens = ctrl.create(body.name, body.query);
    (StatusCode::CREATED, Json(lens))
}

/// Delete a lens by id.
#[utoipa::path(
    delete,
    path = "/lenses/{id}",
    params(("id" = i64, Path, description = "Lens id")),
    responses((status = 204), (status = 404))
)]
async fn delete_lens(State(ctrl): State<LensController>, Path(id): Path<i64>) -> StatusCode {
    if ctrl.delete(id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Run a query through a lens.
#[utoipa::path(
    post,
    path = "/lenses/{id}/query",
    params(("id" = i64, Path, description = "Lens id")),
    request_body = QueryLensBody,
    responses((status = 200, body = QueryLensResult), (status = 404))
)]
async fn query_lens(
    State(ctrl): State<LensController>,
    Path(id): Path<i64>,
    Json(body): Json<QueryLensBody>,
) -> Json<QueryLensResult> {
    let results = ctrl.query(id, &body.query);
    Json(QueryLensResult { results })
}
