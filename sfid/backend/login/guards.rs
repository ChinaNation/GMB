//! 登录会话鉴权守卫与省管理员签名密钥 bootstrap。
//!
//! 业务模块只通过 `require_admin_any`、`require_admin_write`、`require_sheng_admin`
//! 获取认证上下文,不直接读取 session 缓存。

use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Duration, Utc};
use std::sync::atomic::{AtomicI64, Ordering};
use tracing::warn;

use crate::scope::admin_province::province_scope_for_role;
use crate::sheng_admins::province_admins::sheng_admin_province;
use crate::*;

use super::model::AdminAuthContext;
use super::signature::{build_admin_display_name, parse_sr25519_pubkey_bytes};

/// Phase 2 admin_auth 迁移到 GlobalShard：
/// session 验证 + 用户查找全部从 sharded_store.read_global 同步读取,
/// 不再 lock legacy store 的写锁。write_global(async) 用 tokio::task::spawn
/// 后台执行,不阻塞 auth 返回。
pub(super) fn admin_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    if let Some(token) = bearer_token(headers) {
        let now = Utc::now();
        // ShiAdmin idle 超时(分钟),ShengAdmin 无 idle 限制。
        let shi_idle_timeout_minutes = std::env::var("SFID_ADMIN_IDLE_TIMEOUT_MINUTES")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(10);

        // ── Phase 2:后台节流清理(每 60 秒一次,不阻塞请求) ──
        static LAST_CLEANUP: AtomicI64 = AtomicI64::new(0);
        let last = LAST_CLEANUP.load(Ordering::Relaxed);
        let now_ts = now.timestamp();
        if now_ts - last > 60 {
            LAST_CLEANUP.store(now_ts, Ordering::Relaxed);
            let ss = state.sharded_store.clone();
            let cache = state.sheng_admin_signing_cache.clone();
            tokio::task::spawn(async move {
                if let Ok(evicted) =
                    cleanup_sessions_from_global(&ss, Utc::now(), shi_idle_timeout_minutes).await
                {
                    // ADR-008(2026-05-01)Phase 23e:每省 3-tier,需要按 province 整省驱逐
                    // (退化:本省任一 admin slot 被驱逐 → 本省 cache 全清)。
                    for province in evicted {
                        cache.unload_province(province.as_str());
                    }
                }
            });
        }

        // ── 1. 从 GlobalShard 同步读 session ──
        let session = state
            .sharded_store
            .read_global(|g| g.admin_sessions.get(&token).cloned())
            .map_err(|e| {
                warn!(error = %e, "read_global failed in admin_auth");
                api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e)
            })?;

        let Some(session) = session else {
            return Err(api_error(
                StatusCode::UNAUTHORIZED,
                1002,
                "invalid access token",
            ));
        };

        // ── 2. 验证过期 / idle 超时 ──
        // ShengAdmin 无 idle 限制,仅检查 expire_at(8h);
        // ShiAdmin 额外检查 idle 超时(默认 10 分钟)。
        let idle_expired = session.role == AdminRole::ShiAdmin
            && now > session.last_active_at + Duration::minutes(shi_idle_timeout_minutes);
        if now > session.expire_at || idle_expired {
            // 过期:后台异步删除 session(write_global 是 async)
            let ss = state.sharded_store.clone();
            let token_clone = token.clone();
            tokio::task::spawn(async move {
                let _ = ss
                    .write_global(|g| {
                        g.admin_sessions.remove(&token_clone);
                    })
                    .await;
            });
            return Err(api_error(
                StatusCode::UNAUTHORIZED,
                1002,
                "access token expired",
            ));
        }

        // ── 3. 后台更新 last_active_at(不阻塞返回) ──
        {
            let ss = state.sharded_store.clone();
            let token_clone = token.clone();
            tokio::task::spawn(async move {
                let _ = ss
                    .write_global(|g| {
                        if let Some(s) = g.admin_sessions.get_mut(&token_clone) {
                            s.last_active_at = Utc::now();
                        }
                    })
                    .await;
            });
        }

        let session_pubkey = session.admin_pubkey.clone();

        // ── 4. 查用户信息:优先 GlobalShard,fallback legacy store ──
        // GlobalShard.global_admins 包含 ShengAdmin;
        // ShiAdmin 可能还未同步到 GlobalShard,fallback legacy。
        let user_info = state
            .sharded_store
            .read_global(|g| {
                if let Some(user) = g.global_admins.get(&session_pubkey) {
                    let province = match &user.role {
                        AdminRole::ShengAdmin => g
                            .sheng_admin_province_by_pubkey
                            .get(&session_pubkey)
                            .cloned()
                            .or_else(|| {
                                sheng_admin_province(&session_pubkey).map(|v| v.to_string())
                            }),
                        AdminRole::ShiAdmin => {
                            let creator = &user.created_by;
                            g.sheng_admin_province_by_pubkey
                                .get(creator)
                                .cloned()
                                .or_else(|| sheng_admin_province(creator).map(|v| v.to_string()))
                        }
                    };
                    let city = if user.role == AdminRole::ShiAdmin && !user.city.is_empty() {
                        Some(user.city.clone())
                    } else {
                        None
                    };
                    return Some((
                        user.admin_pubkey.clone(),
                        user.role.clone(),
                        user.status.clone(),
                        user.admin_name.clone(),
                        user.city.clone(),
                        user.created_by.clone(),
                        province,
                        city,
                    ));
                }
                None
            })
            .map_err(|e| {
                warn!(error = %e, "read_global failed for admin user lookup");
                api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e)
            })?;

        // GlobalShard 命中:ShengAdmin(或已同步的 ShiAdmin)
        // 未命中:fallback legacy store(ShiAdmin 可能尚未同步到 GlobalShard)
        let (
            admin_pubkey,
            role,
            status,
            admin_name,
            _city_raw,
            _created_by,
            admin_province,
            admin_city,
        ) = if let Some(info) = user_info {
            info
        } else {
            // fallback:从 legacy store 读(只拿读锁)
            let store = store_read_or_500(state)?;
            let Some(user) = store.admin_users_by_pubkey.get(&session_pubkey) else {
                return Err(api_error(StatusCode::FORBIDDEN, 2002, "admin not found"));
            };
            let province = province_scope_for_role(&store, &user.admin_pubkey, &user.role);
            let city = if user.role == AdminRole::ShiAdmin && !user.city.is_empty() {
                Some(user.city.clone())
            } else {
                None
            };
            (
                user.admin_pubkey.clone(),
                user.role.clone(),
                user.status.clone(),
                user.admin_name.clone(),
                user.city.clone(),
                user.created_by.clone(),
                province,
                city,
            )
        };

        if status != AdminStatus::Active {
            return Err(api_error(StatusCode::FORBIDDEN, 2003, "admin disabled"));
        }

        // 二角色统一:优先使用 admin_name(真实姓名),空则 fallback 到角色默认名
        let display_name = {
            let name = admin_name.trim();
            if !name.is_empty() {
                name.to_string()
            } else {
                build_admin_display_name(&admin_pubkey, &role, admin_province.as_deref())
            }
        };

        return Ok(AdminAuthContext {
            admin_pubkey,
            role,
            admin_name: display_name,
            admin_province,
            admin_city,
        });
    }

    Err(api_error(
        StatusCode::UNAUTHORIZED,
        1002,
        "admin auth required",
    ))
}

/// Phase 2:异步清理 GlobalShard 中的过期 session。
/// 由 admin_auth 里的 60 秒节流触发,后台 tokio::task::spawn 执行。
/// ShengAdmin 无 idle 限制(仅 expire_at),ShiAdmin 额外检查 idle。
async fn cleanup_sessions_from_global(
    store: &std::sync::Arc<crate::store_shards::ShardedStore>,
    now: DateTime<Utc>,
    shi_idle_timeout_minutes: i64,
) -> Result<Vec<String>, String> {
    let mut evicted_provinces: Vec<String> = Vec::new();
    store
        .write_global(|g| {
            let mut evicted_sheng_pubkeys: Vec<String> = Vec::new();
            let mut remaining_sheng_pubkeys: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            g.admin_sessions.retain(|_, session| {
                // expire_at 硬上限对所有角色生效
                if now > session.expire_at {
                    if session.role == AdminRole::ShengAdmin {
                        evicted_sheng_pubkeys.push(session.admin_pubkey.clone());
                    }
                    return false;
                }
                // idle 超时仅 ShiAdmin
                if session.role == AdminRole::ShiAdmin
                    && now > session.last_active_at + Duration::minutes(shi_idle_timeout_minutes)
                {
                    return false;
                }
                if session.role == AdminRole::ShengAdmin {
                    remaining_sheng_pubkeys.insert(session.admin_pubkey.clone());
                }
                true
            });

            let max_sessions = bounded_cache_limit("SFID_ADMIN_SESSION_MAX", 50_000);
            if g.admin_sessions.len() > max_sessions {
                let mut entries = g
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
                let overflow = g.admin_sessions.len() - max_sessions;
                for (token, _, role, pubkey) in entries.into_iter().take(overflow) {
                    g.admin_sessions.remove(&token);
                    if role == AdminRole::ShengAdmin {
                        evicted_sheng_pubkeys.push(pubkey);
                    }
                }
                remaining_sheng_pubkeys.clear();
                for (_, s) in g.admin_sessions.iter() {
                    if s.role == AdminRole::ShengAdmin {
                        remaining_sheng_pubkeys.insert(s.admin_pubkey.clone());
                    }
                }
            }

            for pubkey in evicted_sheng_pubkeys {
                if remaining_sheng_pubkeys.contains(&pubkey) {
                    continue;
                }
                if let Some(province) = g.sheng_admin_province_by_pubkey.get(&pubkey) {
                    evicted_provinces.push(province.clone());
                }
            }
        })
        .await?;
    Ok(evicted_provinces)
}

pub(crate) fn require_admin_any(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    admin_auth(state, headers)
}

pub(crate) fn require_admin_write(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    admin_auth(state, headers)
}

/// 中文注释:require_sheng_admin —— 机构管理类操作只允许 ShengAdmin。
pub(crate) fn require_sheng_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if ctx.role != AdminRole::ShengAdmin {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "sheng admin required",
        ));
    }
    if ctx.admin_province.is_none() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        ));
    }
    Ok(ctx)
}

/// 中文注释:ADR-008 Phase 23e —— 把 (province, admin_pubkey) 的签名 keypair
/// 加载到进程内 cache。失败仅 warn,登录流程继续(运维下次登录会自然重试)。
pub(super) fn bootstrap_sheng_signing_pair(
    state: &AppState,
    admin_pubkey_hex: &str,
    province: &str,
) {
    let Some(pubkey_bytes) = parse_sr25519_pubkey_bytes(admin_pubkey_hex) else {
        tracing::warn!(
            province,
            admin_pubkey = admin_pubkey_hex,
            "bootstrap sheng signer skipped: invalid admin_pubkey hex"
        );
        return;
    };
    match crate::sheng_admins::signing_keys::ensure_signing_keypair(
        state.sheng_admin_signing_cache.as_ref(),
        province,
        &pubkey_bytes,
    ) {
        Ok(pair) => {
            let signing_pubkey = crate::sheng_admins::signing_keys::pair_signing_pubkey_hex(&pair);
            crate::sheng_admins::signing_keys::record_signing_metadata(
                state,
                admin_pubkey_hex,
                signing_pubkey.as_str(),
                Utc::now(),
                false,
            );
            tracing::info!(
                province,
                signing_pubkey = %signing_pubkey,
                "sheng signer ready for province"
            );
        }
        Err(e) => tracing::error!(province, error = %e, "bootstrap sheng signer failed"),
    }
}

pub(super) fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get("authorization")?.to_str().ok()?.trim();
    let token = auth.strip_prefix("Bearer ")?;
    if token.trim().is_empty() {
        return None;
    }
    Some(token.trim().to_string())
}
