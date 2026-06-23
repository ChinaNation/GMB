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
    if pubkey
        .verify(ctx.bytes(message.as_bytes()), &signature)
        .is_ok()
    {
        return true;
    }
    let wrapped = format!("<Bytes>{}</Bytes>", message);
    pubkey
        .verify(ctx.bytes(wrapped.as_bytes()), &signature)
        .is_ok()
}

pub(super) fn build_login_qr_system_signature(
    state: &AppState,
    system: &str,
    challenge: &str,
    issued_at: i64,
    expires_at: i64,
) -> Result<(String, String), String> {
    // ADR-008 Phase 23e:登录二维码的"系统签名"由 CID main signer(全局唯一)产出。
    // signer 仍是 CID 系统签名密钥(CID_SIGNING_SEED_HEX 派生),与联邦注册局管理员/市注册局管理员公民钱包无关。
    let main_seed_hex = std::env::var("CID_SIGNING_SEED_HEX")
        .map_err(|_| "CID_SIGNING_SEED_HEX not set".to_string())?;
    let signer = crate::crypto::sr25519::try_load_signing_key_from_seed(main_seed_hex.as_str())?;
    let sys_pubkey = format!("0x{}", hex::encode(signer.public().0));
    let _ = state; // state 不再持有 signing seed/pubkey,签名走 env + crypto helper
    let message = crate::core::qr::build_signature_message(
        crate::core::qr::QrKind::LoginChallenge,
        challenge,
        Some(system),
        Some(expires_at),
        &sys_pubkey,
    );
    let _ = issued_at; // 统一签名原文不再包含 issued_at
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

pub(crate) fn build_admin_name(
    // 中文注释:参数保留以稳定调用签名;内置联邦注册局清单已迁链上,显示名不再按账号反查。
    _admin_account: &str,
    registry_org_code: &RegistryOrgCode,
    scope_province_name: Option<&str>,
) -> String {
    if *registry_org_code == RegistryOrgCode::FederalRegistry {
        if let Some(province) = scope_province_name {
            return format!("{province}联邦注册局管理员");
        }
    }
    // ADR-008 后只剩两角色。
    match registry_org_code {
        RegistryOrgCode::CityRegistry => "市注册局管理员".to_string(),
        RegistryOrgCode::FederalRegistry => "联邦注册局管理员".to_string(),
    }
}

pub(super) fn build_admin_name_from_user(
    admin: &AdminUser,
    scope_province_name: Option<&str>,
) -> String {
    // 二角色统一:优先使用 admin_name(真实姓名),空则 fallback 到角色默认名
    let name = admin.admin_name.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    build_admin_name(
        &admin.admin_account,
        &admin.registry_org_code,
        scope_province_name,
    )
}

/// 仅 CityRegistry 暴露 scope_city_name，其他角色或空字符串一律返回 None。
pub(super) fn resolve_scope_city_name(admin: &AdminUser) -> Option<String> {
    if admin.registry_org_code == RegistryOrgCode::CityRegistry
        && !admin.city_name.trim().is_empty()
    {
        Some(admin.city_name.clone())
    } else {
        None
    }
}
