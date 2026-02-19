use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use crate::{app_state::AppState, models::ApiResponse, services::health_service};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(root))
        .route("/api/v1/health", get(health))
}

async fn root() -> impl IntoResponse {
    Json(ApiResponse {
        code: 0,
        message: "ok",
        data: "wuminapp backend is running",
    })
}

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let data = health_service::get_health(&state);
    Json(ApiResponse {
        code: 0,
        message: "ok",
        data,
    })
}
