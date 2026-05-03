//! 登录签名验签、公钥解析与登录辅助工具。
//!
//! 这里放纯工具函数和 challenge 清理逻辑;会话鉴权在 `guards.rs`,HTTP handler 在
//! `handler.rs` / `qr_login.rs`。

use chrono::{DateTime, Duration, Utc};
use hex::FromHex;
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
use sp_core::Pair;

use crate::crypto::pubkey::same_admin_pubkey;
use crate::sheng_admins::province_admins::sheng_admin_display_name;
use crate::*;

pub(crate) fn verify_admin_signature(
    admin_pubkey: &str,
    message: &str,
    signature_text: &str,
) -> bool {
    let Some(pubkey_bytes) = parse_sr25519_pubkey_bytes(admin_pubkey) else {
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
    // ADR-008 Phase 23e:登录二维码的"系统签名"由 SFID main signer(全局唯一)产出。
    // signer 仍是 SFID main(SFID_SIGNING_SEED_HEX 派生),与省管理员 3-tier 无关。
    let main_seed_hex = std::env::var("SFID_SIGNING_SEED_HEX")
        .map_err(|_| "SFID_SIGNING_SEED_HEX not set".to_string())?;
    let signer = crate::crypto::sr25519::try_load_signing_key_from_seed(main_seed_hex.as_str())?;
    let sys_pubkey = format!("0x{}", hex::encode(signer.public().0));
    let _ = state; // state 不再持有 signing seed/pubkey,签名走 env + crypto helper
    let message = crate::qr::build_signature_message(
        crate::qr::QrKind::LoginChallenge,
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
pub(crate) fn parse_sr25519_pubkey(admin_pubkey: &str) -> Option<String> {
    let raw = admin_pubkey
        .trim()
        .strip_prefix("0x")
        .or_else(|| admin_pubkey.trim().strip_prefix("0X"))
        .unwrap_or(admin_pubkey.trim());
    if raw.len() == 64 && raw.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(format!("0x{}", raw.to_ascii_lowercase()));
    }
    None
}

pub(crate) fn parse_sr25519_pubkey_bytes(admin_pubkey: &str) -> Option<[u8; 32]> {
    if let Some(hex_pubkey) = parse_sr25519_pubkey(admin_pubkey) {
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

pub(super) fn resolve_admin_pubkey_key(store: &Store, candidate: &str) -> Option<String> {
    store
        .admin_users_by_pubkey
        .keys()
        .find(|pubkey| same_admin_pubkey(pubkey.as_str(), candidate))
        .cloned()
}

pub(super) fn parse_admin_identity_qr(identity_qr: &str) -> String {
    let trimmed = identity_qr.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('{') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(v) = value
                .get("admin_pubkey")
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

pub(super) fn cleanup_expired_challenges(store: &mut Store, now: DateTime<Utc>) {
    store.login_challenges.retain(|_, c| {
        c.expire_at > now - Duration::minutes(10) && (!c.consumed || c.expire_at > now)
    });
    store.qr_login_results.retain(|_, r| {
        r.created_at > now - Duration::hours(1) && r.expire_at > now - Duration::minutes(10)
    });
}

/// 中文注释：清理过期/空闲超时的 admin session。
///
/// 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 步骤 8：
/// 返回本次被驱逐的 ShengAdmin session 所属 province 列表，供外层调用
/// `state.sheng_admin_signing_cache.unload_province` 释放内存 Pair。
/// ADR-008 Phase 23e 后:cache 已迁到 `sheng_admins::signing_cache::ShengSigningCache`,
/// `unload_province` 会清理本省所有 3-tier slot 的 keypair。
#[allow(dead_code)]
fn cleanup_admin_sessions(
    store: &mut Store,
    now: DateTime<Utc>,
    idle_timeout_minutes: i64,
) -> Vec<String> {
    let mut evicted_sheng_provinces: Vec<String> = Vec::new();
    let mut remaining_sheng_pubkeys: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // 先收集"被驱逐"的 sheng admin pubkey。
    let mut evicted_sheng_pubkeys: Vec<String> = Vec::new();
    store.admin_sessions.retain(|_, session| {
        let keep = now <= session.expire_at
            && now <= session.last_active_at + Duration::minutes(idle_timeout_minutes);
        if !keep && session.role == AdminRole::ShengAdmin {
            evicted_sheng_pubkeys.push(session.admin_pubkey.clone());
        }
        if keep && session.role == AdminRole::ShengAdmin {
            remaining_sheng_pubkeys.insert(session.admin_pubkey.clone());
        }
        keep
    });

    let max_sessions = bounded_cache_limit("SFID_ADMIN_SESSION_MAX", 50_000);
    if store.admin_sessions.len() > max_sessions {
        let mut entries = store
            .admin_sessions
            .iter()
            .map(|(token, session)| {
                (
                    token.clone(),
                    session.last_active_at,
                    session.role.clone(),
                    session.admin_pubkey.clone(),
                )
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|(_, last_active, _, _)| *last_active);
        let overflow = store.admin_sessions.len() - max_sessions;
        for (token, _, role, pubkey) in entries.into_iter().take(overflow) {
            store.admin_sessions.remove(&token);
            if role == AdminRole::ShengAdmin {
                evicted_sheng_pubkeys.push(pubkey);
            }
        }
        // 重新计算 remaining_sheng_pubkeys
        remaining_sheng_pubkeys.clear();
        for (_, s) in store.admin_sessions.iter() {
            if s.role == AdminRole::ShengAdmin {
                remaining_sheng_pubkeys.insert(s.admin_pubkey.clone());
            }
        }
    }

    // 只有当该 sheng admin 所有 session 都被清掉时，才驱逐本省 cache。
    for pubkey in evicted_sheng_pubkeys {
        if remaining_sheng_pubkeys.contains(&pubkey) {
            continue;
        }
        if let Some(province) = store.sheng_admin_province_by_pubkey.get(&pubkey) {
            evicted_sheng_provinces.push(province.clone());
        }
    }
    evicted_sheng_provinces
}

pub(crate) fn build_admin_display_name(
    admin_pubkey: &str,
    role: &AdminRole,
    admin_province: Option<&str>,
) -> String {
    if *role == AdminRole::ShengAdmin {
        if let Some(province) = admin_province {
            return format!("{province}机构管理员");
        }
    }
    if let Some(name) = sheng_admin_display_name(admin_pubkey) {
        return name;
    }
    // ADR-008 后只剩两角色。
    match role {
        AdminRole::ShiAdmin => "系统管理员".to_string(),
        AdminRole::ShengAdmin => "机构管理员".to_string(),
    }
}

pub(super) fn build_admin_display_name_from_user(
    admin: &AdminUser,
    admin_province: Option<&str>,
) -> String {
    // 二角色统一:优先使用 admin_name(真实姓名),空则 fallback 到角色默认名
    let name = admin.admin_name.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    build_admin_display_name(&admin.admin_pubkey, &admin.role, admin_province)
}

/// 仅 ShiAdmin 暴露 admin_city，其他角色或空字符串一律返回 None。
pub(super) fn resolve_admin_city(admin: &AdminUser) -> Option<String> {
    if admin.role == AdminRole::ShiAdmin && !admin.city.trim().is_empty() {
        Some(admin.city.clone())
    } else {
        None
    }
}
