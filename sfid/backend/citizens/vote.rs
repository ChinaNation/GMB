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

    let result = match state.db.find_bound_citizen_by_wallet(&wallet_pubkey) {
        Ok(Some(record)) => MyIdStatusOutput {
            bind_status: match record.bind_status() {
                CitizenBindStatus::Bound => "bound",
                CitizenBindStatus::Pending => "pending",
            }
            .to_string(),
            wallet_address: record.wallet_address.clone(),
            sfid_number: record.sfid_number.clone(),
            citizen_status: record.citizen_status.clone(),
            voting_eligible: Some(record.voting_eligible),
            vote_status: Some(record.computed_vote_status()),
            identity_status: Some(record.computed_identity_status()),
            valid_from: record.archive_valid_from.clone(),
            valid_until: record.archive_valid_until.clone(),
            status_updated_at: record.status_updated_at,
            residence_province_code: record.residence_province_code.clone(),
            residence_city_code: record.residence_city_code.clone(),
            residence_town_code: record.residence_town_code.clone(),
            birth_province_code: record.birth_province_code.clone(),
            birth_city_code: record.birth_city_code.clone(),
            birth_town_code: record.birth_town_code.clone(),
            election_scope_level: record.election_scope_level.clone(),
        },
        Ok(None) => MyIdStatusOutput {
            bind_status: "unset".to_string(),
            wallet_address: None,
            sfid_number: None,
            citizen_status: None,
            voting_eligible: None,
            vote_status: None,
            identity_status: None,
            valid_from: None,
            valid_until: None,
            status_updated_at: None,
            residence_province_code: None,
            residence_city_code: None,
            residence_town_code: None,
            birth_province_code: None,
            birth_city_code: None,
            birth_town_code: None,
            election_scope_level: None,
        },
        Err(err) => {
            tracing::error!(error = %err, "query myid status failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "myid query failed");
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: result,
    })
    .into_response()
}
