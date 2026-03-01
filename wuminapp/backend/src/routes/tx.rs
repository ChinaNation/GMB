use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::{
    app_state::AppState,
    models::{ApiResponse, TxStatusData, TxSubmitData, TxSubmitRequest},
    services::tx_service,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/tx/submit", post(submit_tx))
        .route("/api/v1/tx/status/:tx_hash", get(tx_status))
}

async fn submit_tx(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TxSubmitRequest>,
) -> impl IntoResponse {
    match tx_service::submit_tx(req) {
        Ok(data) => Json(ApiResponse {
            code: 0,
            message: "ok",
            data,
        }),
        Err(err) => Json(ApiResponse {
            code: err.code,
            message: err.message,
            data: TxSubmitData {
                tx_hash: None,
                status: "failed",
                failure_reason: Some(err.message),
            },
        }),
    }
}

async fn tx_status(
    State(_state): State<Arc<AppState>>,
    Path(tx_hash): Path<String>,
) -> impl IntoResponse {
    match tx_service::get_tx_status(&tx_hash) {
        Ok(data) => Json(ApiResponse {
            code: 0,
            message: "ok",
            data,
        }),
        Err(err) => Json(ApiResponse {
            code: err.code,
            message: err.message,
            data: TxStatusData {
                tx_hash,
                status: "failed",
                failure_reason: Some(err.message),
                updated_at: 0,
            },
        }),
    }
}
