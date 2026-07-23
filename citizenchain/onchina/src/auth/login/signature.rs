//! 登录签名验签、公钥解析与登录辅助工具。
//!
//! 这里放纯工具函数和 challenge 清理逻辑;会话鉴权在 `guards.rs`,HTTP handler 在
//! `handler.rs` / `qr_login.rs`。

use hex::FromHex;
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};

use crate::*;

pub(crate) fn verify_admin_signature(
    signer_public_key: &str,
    message: &str,
    signature_text: &str,
) -> bool {
    if verify_admin_signature_bytes(signer_public_key, message.as_bytes(), signature_text) {
        return true;
    }
    let wrapped = format!("<Bytes>{}</Bytes>", message);
    verify_admin_signature_bytes(signer_public_key, wrapped.as_bytes(), signature_text)
}

/// 校验管理员使用的签名钱包对原始字节的 sr25519 签名。
///
/// 链上中国治理 JSON 统一走 `verify_admin_signature`；本函数只作为内部底层验签工具，
/// 不再承载机构创建内层凭证入口。
pub(crate) fn verify_admin_signature_bytes(
    signer_public_key: &str,
    message: &[u8],
    signature_text: &str,
) -> bool {
    let Some(public_key_bytes) = parse_sr25519_public_key_bytes(signer_public_key) else {
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
    let public_key = match Sr25519PublicKey::from_bytes(&public_key_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let signature = match Sr25519Signature::from_bytes(&sig_arr) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let ctx = signing_context(b"substrate");
    public_key.verify(ctx.bytes(message), &signature).is_ok()
}

/// 解析 sr25519 签名公钥，返回小写 `0x` 加 64 位十六进制。
pub(crate) fn parse_sr25519_public_key(public_key: &str) -> Option<String> {
    let public_key = public_key.trim();
    if public_key.len() != 66 || !public_key.starts_with("0x") {
        return None;
    }
    let raw = &public_key[2..];
    if raw
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Some(public_key.to_string());
    }
    None
}

pub(crate) fn parse_sr25519_public_key_bytes(public_key: &str) -> Option<[u8; 32]> {
    let public_key = parse_sr25519_public_key(public_key)?;
    let bytes = Vec::from_hex(strip_0x_prefix(&public_key)).ok()?;
    bytes.as_slice().try_into().ok()
}

/// 把规范账户 ID 解码为当前 runtime 使用的 32 字节账户值。
pub(crate) fn parse_account_id_bytes(account_id: &str) -> Option<[u8; 32]> {
    let account_id = crate::crypto::pubkey::normalize_account_id(account_id)?;
    let bytes = Vec::from_hex(&account_id[2..]).ok()?;
    bytes.as_slice().try_into().ok()
}

/// 去掉 0x/0X 前缀，仅用于 hex::decode 前的临时处理，不用于存储。
fn strip_0x_prefix(value: &str) -> &str {
    value
        .trim()
        .strip_prefix("0x")
        .or_else(|| value.trim().strip_prefix("0X"))
        .unwrap_or(value.trim())
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
