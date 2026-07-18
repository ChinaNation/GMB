//! QR_V1/k=1 签名请求构造工具。
//!
//! 这里只负责把已确定的签名原文包装成统一二维码 envelope;
//! 业务模块仍负责决定签名内容和权限语义。

use crate::{
    api_error,
    core::qr::{bytes_to_b64, pubkey_hex_to_b64, QR_V1},
};
use axum::http::StatusCode;

pub(crate) fn build_sign_request(
    request_id: &str,
    issued_at: i64,
    expires_at: i64,
    actor_pubkey: &str,
    payload_text: &str,
    action: u16,
) -> Result<String, axum::response::Response> {
    build_sign_request_bytes(
        request_id,
        issued_at,
        expires_at,
        actor_pubkey,
        payload_text.as_bytes(),
        action,
    )
}

/// 把已确定的待签 payload **裸字节**包装成 QR_V1/k=1 envelope。
///
/// 普通链交易传入值必须是完整 `review_payload`，钱包依赖它完整解码和中文展示；
/// 32 字节 `signing_bytes` 只允许 Runtime 升级 hash-only 专用入口使用。
pub(crate) fn build_sign_request_bytes(
    request_id: &str,
    _issued_at: i64,
    expires_at: i64,
    actor_pubkey: &str,
    payload_bytes: &[u8],
    action: u16,
) -> Result<String, axum::response::Response> {
    let Some(pubkey_b64) = pubkey_hex_to_b64(actor_pubkey) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "actor pubkey must be 32-byte hex",
        ));
    };
    let sign_request = serde_json::json!({
        "p": QR_V1,
        "k": 1,
        "i": request_id,
        "e": expires_at,
        "b": {
            "a": action,
            "g": 1,
            "u": pubkey_b64,
            "d": bytes_to_b64(payload_bytes),
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
