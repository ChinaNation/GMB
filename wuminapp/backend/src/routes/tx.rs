use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::{
    app_state::AppState,
    models::{
        ApiResponse, TxPrepareData, TxPrepareRequest, TxStatusData, TxSubmitData, TxSubmitRequest,
    },
    services::tx_service,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/tx/prepare", post(prepare_tx))
        .route("/api/v1/tx/submit", post(submit_tx))
        .route("/api/v1/tx/status/:tx_hash", get(tx_status))
}

async fn prepare_tx(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TxPrepareRequest>,
) -> impl IntoResponse {
    match tx_service::prepare_tx(req).await {
        Ok(data) => Json(ApiResponse {
            code: 0,
            message: "ok",
            data,
        }),
        Err(err) => Json(ApiResponse {
            code: err.code,
            message: err.message,
            data: TxPrepareData {
                prepared_id: String::new(),
                signer_payload_hex: String::new(),
                expires_at: 0,
            },
        }),
    }
}

async fn submit_tx(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TxSubmitRequest>,
) -> impl IntoResponse {
    match tx_service::submit_tx(req).await {
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
                status: "failed".to_string(),
                failure_reason: Some(err.message.to_string()),
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
                status: "failed".to_string(),
                failure_reason: Some(err.message.to_string()),
                updated_at: 0,
            },
        }),
    }
}
