use std::sync::Arc;

use axum::{
    extract::State,
    response::IntoResponse,
    routing::post,
    Json, Router,
};

use crate::{
    app_state::AppState,
    models::{ApiResponse, SfidPubkeyPushData, SfidPubkeyPushRequest},
    services::sfid_service,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/auth/sfid/pubkey", post(push_pubkey))
}

async fn push_pubkey(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SfidPubkeyPushRequest>,
) -> impl IntoResponse {
    match sfid_service::push_pubkey_to_sfid(&req.pubkey_hex).await {
        Ok(data) => Json(ApiResponse {
            code: 0,
            message: "ok",
            data,
        }),
        Err(err) => Json(ApiResponse {
            code: err.code,
            message: err.message,
            data: SfidPubkeyPushData {
                accepted: false,
                pushed_at: 0,
            },
        }),
    }
}
