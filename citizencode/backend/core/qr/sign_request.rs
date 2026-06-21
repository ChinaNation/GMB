//! CITIZEN_QR_V1/sign_request 构造工具。
//!
//! 中文注释:这里只负责把已确定的签名原文包装成统一二维码 envelope;
//! 业务模块仍负责决定签名内容、字段展示和权限语义。

use axum::http::StatusCode;
use serde_json::json;

use crate::citizens::binding::pubkey_hex_to_ss58;
use crate::{api_error, core::qr::CITIZEN_QR_V1};

pub(crate) const ADMIN_SIGN_ACTION: &str = "cid_admin_action";

pub(crate) fn display_field(key: &str, label: &str, value: &str) -> serde_json::Value {
    json!({ "key": key, "label": label, "value": value })
}

pub(crate) fn display_account(value: &str) -> String {
    pubkey_hex_to_ss58(value).unwrap_or_else(|| value.to_string())
}

pub(crate) fn build_sign_request(
    request_id: &str,
    issued_at: i64,
    expires_at: i64,
    actor_account: &str,
    payload_text: &str,
    summary: &str,
    fields: Vec<serde_json::Value>,
) -> Result<String, axum::response::Response> {
    let Some(account_ss58) = pubkey_hex_to_ss58(actor_account) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "actor account cannot be encoded as SS58",
        ));
    };
    let sign_request = json!({
        "proto": CITIZEN_QR_V1,
        "kind": "sign_request",
        "id": request_id,
        "issued_at": issued_at,
        "expires_at": expires_at,
        "body": {
            "address": account_ss58,
            "pubkey": actor_account,
            "sig_alg": "sr25519",
            "payload_hex": format!("0x{}", hex::encode(payload_text.as_bytes())),
            "display": {
                "action": ADMIN_SIGN_ACTION,
                "summary": summary,
                "fields": fields,
            }
        }
    });
    serde_json::to_string(&sign_request).map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1503,
            "encode sign request failed",
        )
    })
}
