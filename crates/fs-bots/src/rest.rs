// rest.rs — REST + OpenAPI routes for fs-bots.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::controller::BotController;
use crate::model::MessagingBot;

// ── OpenAPI doc ───────────────────────────────────────────────────────────────

#[allow(clippy::needless_for_each)]
#[derive(OpenApi)]
#[openapi(
    paths(list_bots, get_bot, enable_bot, disable_bot),
    components(schemas(BotSummary))
)]
pub struct ApiDoc;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct BotSummary {
    pub id: String,
    pub name: String,
    pub kind_label: String,
    pub enabled: bool,
}

impl From<&MessagingBot> for BotSummary {
    fn from(b: &MessagingBot) -> Self {
        Self {
            id: b.id.clone(),
            name: b.name.clone(),
            kind_label: b.kind.label().to_owned(),
            enabled: b.enabled,
        }
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router(ctrl: BotController) -> Router {
    Router::new()
        .route("/bots", get(list_bots))
        .route("/bots/{id}", get(get_bot))
        .route("/bots/{id}/enable", post(enable_bot))
        .route("/bots/{id}/disable", post(disable_bot))
        .with_state(ctrl)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// List all bots.
#[utoipa::path(get, path = "/bots", responses((status = 200, body = Vec<BotSummary>)))]
async fn list_bots(State(ctrl): State<BotController>) -> Json<Vec<BotSummary>> {
    Json(ctrl.list().iter().map(BotSummary::from).collect())
}

/// Get a single bot by id.
#[utoipa::path(
    get,
    path = "/bots/{id}",
    params(("id" = String, Path, description = "Bot id")),
    responses((status = 200, body = BotSummary), (status = 404))
)]
async fn get_bot(
    State(ctrl): State<BotController>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Option<BotSummary>>) {
    match ctrl.get(&id) {
        Some(b) => (StatusCode::OK, Json(Some(BotSummary::from(&b)))),
        None => (StatusCode::NOT_FOUND, Json(None)),
    }
}

/// Enable a bot.
#[utoipa::path(
    post,
    path = "/bots/{id}/enable",
    params(("id" = String, Path, description = "Bot id")),
    responses((status = 200), (status = 404))
)]
async fn enable_bot(State(ctrl): State<BotController>, Path(id): Path<String>) -> StatusCode {
    if ctrl.enable(&id) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Disable a bot.
#[utoipa::path(
    post,
    path = "/bots/{id}/disable",
    params(("id" = String, Path, description = "Bot id")),
    responses((status = 200), (status = 404))
)]
async fn disable_bot(State(ctrl): State<BotController>, Path(id): Path<String>) -> StatusCode {
    if ctrl.disable(&id) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}
