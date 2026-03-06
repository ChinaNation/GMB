use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};

use crate::{
    app_state::AppState,
    models::{ApiResponse, ChainBindRequest, ChainBindRequestData},
    services::chain_binding_service,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/chain/bind/request", post(request_bind))
}

async fn request_bind(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChainBindRequest>,
) -> impl IntoResponse {
    match chain_binding_service::request_chain_bind(&state.db, &req.account_pubkey).await {
        Ok(data) => Json(ApiResponse {
            code: 0,
            message: "ok",
            data,
        }),
        Err(err) => Json(ApiResponse {
            code: err.code,
            message: err.message,
            data: ChainBindRequestData {
                accepted: false,
                requested_at: 0,
            },
        }),
    }
}
