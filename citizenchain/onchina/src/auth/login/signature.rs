//! 登录签名验签、公钥解析与登录辅助工具。
//!
//! 这里放纯工具函数和 challenge 清理逻辑;会话鉴权在 `guards.rs`,HTTP handler 在
//! `handler.rs` / `qr_login.rs`。

use hex::FromHex;
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
use sp_core::Pair;

use crate::*;

pub(crate) fn verify_admin_signature(
    admin_account: &str,
    message: &str,
    signature_text: &str,
) -> bool {
    if verify_admin_signature_bytes(admin_account, message.as_bytes(), signature_text) {
        return true;
    }
    let wrapped = format!("<Bytes>{}</Bytes>", message);
    verify_admin_signature_bytes(admin_account, wrapped.as_bytes(), signature_text)
}

/// 校验管理员钱包对原始字节的 sr25519 签名。
///
/// 链上中国治理 JSON 统一走 `verify_admin_signature`；本函数只作为内部底层验签工具，
/// 不再承载机构创建内层凭证入口。
pub(crate) fn verify_admin_signature_bytes(
    admin_account: &str,
    message: &[u8],
    signature_text: &str,
) -> bool {
    let Some(pubkey_bytes) = parse_sr25519_pubkey_bytes(admin_account) else {
        return false;
    };
    let normalized_sig = strip_0x_prefix(signature_text);
    let sig_bytes = match Vec::from_hex(normalized_sig) {
        Ok(v) if v.len() == 64 => v,
        _ => return false,
    };
    let sig_arr: [u8; 64] = match sig_bytes.as_slice().try_into() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let pubkey = match Sr25519PublicKey::from_bytes(&pubkey_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let signature = match Sr25519Signature::from_bytes(&sig_arr) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let ctx = signing_context(b"substrate");
    pubkey.verify(ctx.bytes(message), &signature).is_ok()
}

pub(super) fn build_login_qr_system_signature(
    state: &AppState,
    system: &str,
    challenge: &str,
    issued_at: i64,
    expires_at: i64,
) -> Result<(String, String), String> {
    // 登录二维码的"系统签名"由 OnChina 平台系统签名密钥产出。
    // 它只签平台挑战,不代表任何机构管理员,也不代替管理员冷钱包签名。
    let main_seed_hex = std::env::var("ONCHINA_SIGNING_SEED_HEX")
        .map_err(|_| "ONCHINA_SIGNING_SEED_HEX not set".to_string())?;
    let signer = crate::crypto::sr25519::try_load_signing_key_from_seed(main_seed_hex.as_str())?;
    let sys_pubkey = format!("0x{}", hex::encode(signer.public().0));
    let _ = state; // 签名走 env + crypto helper,不取自 state
    let message = crate::core::qr::build_signature_message(
        crate::core::qr::QrKind::SignRequest,
        challenge,
        Some(system),
        Some(expires_at),
        &sys_pubkey,
    );
    let _ = issued_at; // 统一签名原文不包含 issued_at
    let signature = signer.sign(message.as_bytes());
    Ok((sys_pubkey, format!("0x{}", hex::encode(signature.0))))
}

/// 解析 Sr25519 公钥，返回统一格式 `0x` + 64 位小写 hex。
pub(crate) fn parse_sr25519_pubkey(admin_account: &str) -> Option<String> {
    let raw = admin_account
        .trim()
        .strip_prefix("0x")
        .or_else(|| admin_account.trim().strip_prefix("0X"))
        .unwrap_or(admin_account.trim());
    if raw.len() == 64 && raw.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(format!("0x{}", raw.to_ascii_lowercase()));
    }
    None
}

pub(crate) fn parse_sr25519_pubkey_bytes(admin_account: &str) -> Option<[u8; 32]> {
    if let Some(hex_pubkey) = parse_sr25519_pubkey(admin_account) {
        // hex::decode 不接受 0x 前缀，去掉后解码
        let bytes = Vec::from_hex(strip_0x_prefix(&hex_pubkey)).ok()?;
        let arr: [u8; 32] = bytes.as_slice().try_into().ok()?;
        return Some(arr);
    }
    None
}

/// 去掉 0x/0X 前缀，仅用于 hex::decode 前的临时处理，不用于存储。
fn strip_0x_prefix(value: &str) -> &str {
    value
        .trim()
        .strip_prefix("0x")
        .or_else(|| value.trim().strip_prefix("0X"))
        .unwrap_or(value.trim())
}

pub(super) fn parse_admin_identity_qr(identity_qr: &str) -> String {
    let trimmed = identity_qr.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('{') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(v) = value
                .get("admin_account")
                .or_else(|| value.get("pubkey"))
                .and_then(|v| v.as_str())
            {
                return v.trim().to_string();
            }
        }
    }
    trimmed.to_string()
}

pub(super) fn extract_domain_from_origin(origin: &str) -> Option<String> {
    let trimmed = origin.trim();
    if trimmed.is_empty() {
        return None;
    }
    let no_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let host_port = no_scheme.split('/').next().unwrap_or("");
    if host_port.is_empty() {
        return None;
    }
    let domain = host_port.split(':').next().unwrap_or("");
    if domain.is_empty() {
        return None;
    }
    Some(domain.to_string())
}

/// 返回管理员链上姓、名；本地异常空值分别收敛为“管理”“员”。
pub(crate) fn admin_person_names(admin: &AdminUser) -> (String, String) {
    let family_name = admin.family_name.trim();
    let given_name = admin.given_name.trim();
    (
        if family_name.is_empty() {
            "管理".to_string()
        } else {
            family_name.to_string()
        },
        if given_name.is_empty() {
            "员".to_string()
        } else {
            given_name.to_string()
        },
    )
}
