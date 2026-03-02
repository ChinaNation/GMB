use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    models::{ApiResponse, WalletBalanceData},
    services::wallet_service,
};

#[derive(Deserialize)]
struct WalletBalanceQuery {
    account: String,
    pubkey_hex: Option<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/wallet/balance", get(wallet_balance))
}

async fn wallet_balance(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<WalletBalanceQuery>,
) -> impl IntoResponse {
    match wallet_service::get_wallet_balance(&query.account, query.pubkey_hex.as_deref()).await {
        Ok(data) => Json(ApiResponse {
            code: 0,
            message: "ok",
            data,
        }),
        Err(err) => Json(ApiResponse {
            code: err.code,
            message: err.message,
            data: WalletBalanceData {
                account: query.account,
                balance: 0.0,
                symbol: "CIT",
                updated_at: 0,
            },
        }),
    }
}
