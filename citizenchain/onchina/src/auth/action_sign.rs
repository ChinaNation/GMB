//! 管理员敏感动作的冷钱包扫码签名工具(PasskeyColdSign 档)。
//!
//! 中文注释:敏感动作 step-up 统一为
//! "会话(链上已证管理员)+ 冷钱包扫码签名"。本模块只承载扫码签名 payload 构造、
//! 哈希与验签工具,不含任何设备本地因子。

use axum::http::StatusCode;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::crypto::pubkey::same_admin_account;
use crate::*;

/// 敏感动作挑战有效期(秒)。
pub(crate) const ADMIN_ACTION_TTL_SECONDS: i64 = 300;

/// 冷钱包扫码签名的结构化 payload(序列化为 JSON 文本后冷签)。
#[derive(Serialize)]
pub(crate) struct AdminSignedPayload<'a> {
    pub(crate) domain: &'static str,
    pub(crate) qr_proto: &'static str,
    pub(crate) action_id: &'a str,
    pub(crate) action_type: &'a str,
    pub(crate) actor_pubkey: &'a str,
    pub(crate) actor_province_name: &'a str,
    pub(crate) target: &'a str,
    pub(crate) request_hash: &'a str,
    pub(crate) before_hash: &'a str,
    pub(crate) after_hash: &'a str,
    pub(crate) expires_at: i64,
}

pub(crate) fn signed_payload_text(payload: AdminSignedPayload<'_>) -> String {
    serde_json::to_string(&payload).unwrap_or_default()
}

pub(crate) fn payload_hash_for_text(text: &str) -> String {
    format!("0x{}", hex::encode(Sha256::digest(text.as_bytes())))
}

pub(crate) fn hash_json(value: &serde_json::Value) -> String {
    let encoded = serde_json::to_vec(value).unwrap_or_default();
    format!("0x{}", hex::encode(Sha256::digest(&encoded)))
}

/// 校验冷钱包对动作 payload 的扫码签名。
///
/// 中文注释:① signer 必须等于动作发起人;② 提交摘要与服务端预期摘要一致;
/// ③ sr25519 验签通过。调用方(actions::commit)还会额外校验 signer ∈ 本机构链上 Active 集合。
pub(crate) fn verify_citizen_wallet_signature(
    expected_actor_account: &str,
    signer_pubkey: &str,
    signature: &str,
    submitted_payload_hash: &str,
    expected_payload_hash: &str,
    payload_text: &str,
) -> Result<(), axum::response::Response> {
    if !same_admin_account(expected_actor_account, signer_pubkey) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "signer pubkey mismatch",
        ));
    }
    if submitted_payload_hash.trim().to_lowercase() != expected_payload_hash {
        return Err(api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "payload hash mismatch",
        ));
    }
    if !crate::auth::login::verify_admin_signature(signer_pubkey, payload_text, signature) {
        return Err(api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "signature verify failed",
        ));
    }
    Ok(())
}
