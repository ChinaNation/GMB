//! wuminapp 电子护照状态查询 handler。
//!
//! 绑定由 SFID 后台扫描 CPMS 档案码并校验 wuminapp 签名完成；App 侧只查询
//! SFID 已落库的结果。

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use super::binding::ss58_to_pubkey_hex;
use crate::*;

/// wuminapp 查询电子护照绑定状态（公共接口）。
pub(crate) async fn app_myid_status(
    State(state): State<AppState>,
    Query(params): Query<MyIdStatusQuery>,
) -> impl IntoResponse {
    if params.wallet_address.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "wallet_address is required");
    }
    let wallet_pubkey = match ss58_to_pubkey_hex(params.wallet_address.trim()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid wallet_address"),
    };

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let result = match store
        .citizen_id_by_wallet_pubkey
        .get(&wallet_pubkey)
        .and_then(|cid| store.citizen_records.get(cid))
    {
        Some(record) => MyIdStatusOutput {
            bind_status: match record.bind_status() {
                CitizenBindStatus::Bound => "bound",
                CitizenBindStatus::Pending => "pending",
            }
            .to_string(),
            wallet_address: record.wallet_address.clone(),
            sfid_code: record.sfid_code.clone(),
            identity_status: Some(record.computed_identity_status()),
            valid_from: record.archive_valid_from.clone(),
            valid_until: record.archive_valid_until.clone(),
            status_updated_at: record.status_updated_at,
        },
        None => MyIdStatusOutput {
            bind_status: "unset".to_string(),
            wallet_address: None,
            sfid_code: None,
            identity_status: None,
            valid_from: None,
            valid_until: None,
            status_updated_at: None,
        },
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: result,
    })
    .into_response()
}
