// rest.rs — REST + OpenAPI routes for fs-theme-app.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::controller::{ThemeController, ThemeInfo};

// ── OpenAPI doc ───────────────────────────────────────────────────────────────

#[allow(clippy::needless_for_each)] // triggered by utoipa macro internals
#[derive(OpenApi)]
#[openapi(
    paths(list_themes, get_active, activate_theme, preview_theme),
    components(schemas(ThemeInfo, ActivateBody, PreviewResponse))
)]
pub struct ApiDoc;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct ActivateBody {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PreviewResponse {
    pub css: String,
}

// ── Router ────────────────────────────────────────────────────────────────────

/// Build the axum router for the theme REST API.
pub fn router(ctrl: ThemeController) -> Router {
    Router::new()
        .route("/themes", get(list_themes))
        .route("/themes/active", get(get_active).put(activate_theme))
        .route("/themes/{name}/preview", get(preview_theme))
        .with_state(ctrl)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// List all available themes.
#[utoipa::path(get, path = "/themes", responses((status = 200, body = Vec<ThemeInfo>)))]
async fn list_themes(State(ctrl): State<ThemeController>) -> Json<Vec<ThemeInfo>> {
    Json(ctrl.list())
}

/// Get the currently active theme.
#[utoipa::path(get, path = "/themes/active", responses((status = 200, body = ThemeInfo)))]
async fn get_active(State(ctrl): State<ThemeController>) -> Json<ThemeInfo> {
    Json(ctrl.active())
}

/// Activate a theme by name.
#[utoipa::path(
    put,
    path = "/themes/active",
    request_body = ActivateBody,
    responses((status = 204), (status = 404, description = "Theme not found"))
)]
async fn activate_theme(
    State(ctrl): State<ThemeController>,
    Json(body): Json<ActivateBody>,
) -> StatusCode {
    match ctrl.activate(&body.name) {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

/// Get CSS preview for a named theme.
#[utoipa::path(
    get,
    path = "/themes/{name}/preview",
    params(("name" = String, Path, description = "Theme name")),
    responses((status = 200, body = PreviewResponse), (status = 404))
)]
async fn preview_theme(
    State(ctrl): State<ThemeController>,
    Path(name): Path<String>,
) -> Result<Json<PreviewResponse>, StatusCode> {
    ctrl.preview_css(&name)
        .map(|css| Json(PreviewResponse { css }))
        .ok_or(StatusCode::NOT_FOUND)
}
