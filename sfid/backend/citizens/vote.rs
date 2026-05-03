//! 公民投票账户登记/查询 handler。
//!
//! 这里承接 wuminapp 自有的投票账户注册/状态查询接口,与
//! `citizens::binding` 的身份绑定/解绑、`citizens::chain_vote` 的链端凭证签发分离。

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;

use super::binding::{pubkey_hex_to_ss58, ss58_to_pubkey_hex, verify_citizen_bind_signature};
use crate::*;

/// wuminapp 推送投票账户（公共接口，无 admin 认证）。
///
/// 用户在 wuminapp 选择钱包后,签名证明私钥所有权,再把 pubkey 写入 SFID 待绑定记录。
pub(crate) async fn app_vote_account_register(
    State(state): State<AppState>,
    Json(input): Json<VoteAccountRegisterInput>,
) -> impl IntoResponse {
    if input.address.trim().is_empty()
        || input.pubkey.trim().is_empty()
        || input.signature.trim().is_empty()
        || input.sign_message.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "address, pubkey, signature, sign_message are required",
        );
    }

    // 中文注释:先校验 SS58 地址和用户提交的 pubkey 是否同源,避免替别人登记账户。
    let derived_pubkey = match ss58_to_pubkey_hex(input.address.trim()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid SS58 address"),
    };
    let input_pubkey = input.pubkey.trim().to_lowercase();
    if derived_pubkey.to_lowercase() != input_pubkey {
        return api_error(StatusCode::BAD_REQUEST, 1001, "address and pubkey mismatch");
    }

    // 中文注释:签名原文必须绑定 address 和短时 timestamp,防止跨账户和长期重放。
    let parts: Vec<&str> = input.sign_message.trim().split('|').collect();
    if parts.len() != 3 || parts[0] != "CITIZEN_VOTE_REGISTER" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "invalid sign_message format");
    }
    if parts[1] != input.address.trim() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "sign_message address mismatch",
        );
    }
    let timestamp: i64 = match parts[2].parse() {
        Ok(v) => v,
        Err(_) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "invalid timestamp in sign_message",
            )
        }
    };
    let now = Utc::now().timestamp();
    if (now - timestamp).abs() > 300 {
        return api_error(
            StatusCode::UNAUTHORIZED,
            1007,
            "sign_message expired (>5 min)",
        );
    }

    let pubkey_bytes = match crate::login::parse_sr25519_pubkey_bytes(&input_pubkey) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid pubkey format"),
    };
    let sig_bytes = match hex::decode(input.signature.trim().trim_start_matches("0x")) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid signature hex"),
    };
    if !verify_citizen_bind_signature(&pubkey_bytes, input.sign_message.trim(), &sig_bytes) {
        return api_error(StatusCode::UNAUTHORIZED, 2004, "signature verify failed");
    }

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if store.citizen_id_by_pubkey.contains_key(&input_pubkey) {
        drop(store);
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: serde_json::json!({}),
        })
        .into_response();
    }

    // 中文注释:此时只有钱包 pubkey,等市管理员扫码档案后才补齐 sfid_code。
    let cid = store.next_citizen_id;
    store.next_citizen_id += 1;
    let account_address = pubkey_hex_to_ss58(&input_pubkey);
    let record = CitizenRecord {
        id: cid,
        account_pubkey: Some(input_pubkey.clone()),
        account_address,
        archive_no: None,
        sfid_code: None,
        sfid_signature: None,
        province_code: None,
        chain_confirmed: false,
        bound_at: None,
        bound_by: None,
        created_at: Utc::now(),
    };
    store.citizen_records.insert(cid, record);
    store.citizen_id_by_pubkey.insert(input_pubkey, cid);
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: serde_json::json!({}),
    })
    .into_response()
}

/// wuminapp 查询投票账户绑定状态（公共接口）。
pub(crate) async fn app_vote_account_status(
    State(state): State<AppState>,
    Query(params): Query<VoteAccountStatusQuery>,
) -> impl IntoResponse {
    if params.address.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "address is required");
    }
    let pubkey_hex = match ss58_to_pubkey_hex(params.address.trim()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid SS58 address"),
    };

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let result = match store
        .citizen_id_by_pubkey
        .get(&pubkey_hex)
        .and_then(|cid| store.citizen_records.get(cid))
    {
        Some(record) => {
            let status_str = match record.status() {
                CitizenBindStatus::Bound => "bound",
                CitizenBindStatus::Pending | CitizenBindStatus::Bindable => "pending",
                CitizenBindStatus::Unlinked => "unset",
            };
            VoteAccountStatusOutput {
                status: status_str.to_string(),
                address: record.account_address.clone(),
                sfid_code: record.sfid_code.clone(),
            }
        }
        None => VoteAccountStatusOutput {
            status: "unset".to_string(),
            address: None,
            sfid_code: None,
        },
    };
    drop(store);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: result,
    })
    .into_response()
}
