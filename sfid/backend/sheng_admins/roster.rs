//! 中文注释:注册局-省级管理员页面的一主两备名册展示与本地备用槽维护。
//!
//! 本文件负责注册局页面的 roster 查询和 SFID 本地备用管理员录入。真正的
//! “更换省管理员/主备交换”后续如果接入区块链,必须放到独立
//! `chain_replace_admin.rs` 中,避免普通页面查询继续混用 `chain_` 命名。

#![allow(dead_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sp_core::Pair;

use crate::crypto::pubkey::same_admin_pubkey;
use crate::login::{parse_sr25519_pubkey, parse_sr25519_pubkey_bytes, require_sheng_admin};
use crate::models::{
    AdminRole, AdminStatus, AdminUser, ApiResponse, ShengAdminRosterLocal, ShengAdminSlotLocal,
};
use crate::sheng_admins::province_admins::{pubkey_from_hex, sheng_admin_mains};
use crate::AppState;

#[derive(Debug)]
pub(crate) enum RosterQueryError {
    UnknownProvince,
    PubkeyDecode,
}

impl std::fmt::Display for RosterQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RosterQueryError::UnknownProvince => write!(f, "unknown province"),
            RosterQueryError::PubkeyDecode => write!(f, "decode main pubkey hex failed"),
        }
    }
}

impl std::error::Error for RosterQueryError {}

/// 拉取本省主管理员公钥。
///
/// 中文注释:main 来自 SFID 内置省级管理员基线;backup 槽位由
/// `Store.sheng_admin_rosters` 本地保存,后续链上主备交换能力落地后再接真源。
pub(crate) async fn fetch_roster(province: &str) -> Result<[u8; 32], RosterQueryError> {
    let entry = sheng_admin_mains()
        .iter()
        .find(|p| p.province == province)
        .ok_or(RosterQueryError::UnknownProvince)?;
    pubkey_from_hex(entry.pubkey).ok_or(RosterQueryError::PubkeyDecode)
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminRosterOutput {
    pub(crate) province: String,
    pub(crate) current_admin_pubkey: String,
    pub(crate) entries: Vec<AdminRosterEntry>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminRosterEntry {
    /// 前端业务槽位:"Main" / "Backup1" / "Backup2"。
    pub(crate) slot: &'static str,
    /// 0x 小写 hex(32 字节);未设置时为 null。
    pub(crate) admin_pubkey: Option<String>,
    pub(crate) admin_name: Option<String>,
    /// UNSET / NOT_INITIALIZED / GENERATED / GENERATED_NOT_LOADED。
    pub(crate) signing_status: &'static str,
    pub(crate) signing_pubkey: Option<String>,
    pub(crate) signing_created_at: Option<String>,
    pub(crate) cache_loaded: bool,
    pub(crate) can_operate_signing: bool,
    pub(crate) can_manage_roster: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SetBackupAdminInput {
    pub(crate) slot: String,
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
}

fn admin_slot_label(idx: usize) -> &'static str {
    match idx {
        0 => "Main",
        1 => "Backup1",
        _ => "Backup2",
    }
}

fn local_slot_by_index(local: &ShengAdminRosterLocal, idx: usize) -> Option<&ShengAdminSlotLocal> {
    match idx {
        1 => local.backup_1.as_ref(),
        2 => local.backup_2.as_ref(),
        _ => None,
    }
}

fn render_admin_roster(
    state: &AppState,
    province: &str,
    current_admin_pubkey: &str,
    main: [u8; 32],
) -> Result<AdminRosterOutput, axum::response::Response> {
    let store = crate::store_read_or_500(state)?;
    let local = store
        .sheng_admin_rosters
        .get(province)
        .cloned()
        .unwrap_or_default();
    let slots: [Option<[u8; 32]>; 3] = [
        Some(main),
        local
            .backup_1
            .as_ref()
            .and_then(|slot| parse_sr25519_pubkey_bytes(slot.admin_pubkey.as_str())),
        local
            .backup_2
            .as_ref()
            .and_then(|slot| parse_sr25519_pubkey_bytes(slot.admin_pubkey.as_str())),
    ];
    let current_admin_slot = slots.iter().enumerate().find_map(|(idx, slot)| {
        let pubkey = slot.map(|p| format!("0x{}", hex::encode(p)))?;
        if same_admin_pubkey(pubkey.as_str(), current_admin_pubkey) {
            Some(admin_slot_label(idx))
        } else {
            None
        }
    });
    let can_manage_roster = current_admin_slot == Some("Main");

    let entries = slots
        .iter()
        .enumerate()
        .map(|(idx, slot)| {
            let local_slot = local_slot_by_index(&local, idx);
            let Some(admin_bytes) = slot else {
                return AdminRosterEntry {
                    slot: admin_slot_label(idx),
                    admin_pubkey: None,
                    admin_name: None,
                    signing_status: "UNSET",
                    signing_pubkey: None,
                    signing_created_at: None,
                    cache_loaded: false,
                    can_operate_signing: false,
                    can_manage_roster,
                };
            };
            let admin_pubkey = format!("0x{}", hex::encode(admin_bytes));
            let user = store
                .admin_users_by_pubkey
                .iter()
                .find(|(key, _)| same_admin_pubkey(key.as_str(), admin_pubkey.as_str()))
                .map(|(_, user)| user);
            let cache_pair = state.sheng_admin_signing_cache.get(province, admin_bytes);
            let cache_loaded = cache_pair.is_some();
            let signing_pubkey = user
                .and_then(|user| user.signing_pubkey.clone())
                .or_else(|| {
                    cache_pair
                        .as_ref()
                        .map(|pair| format!("0x{}", hex::encode(pair.public().0)))
                });
            let signing_created_at = user
                .and_then(|user| user.signing_created_at)
                .map(|dt| dt.to_rfc3339());
            let signing_status = match (signing_pubkey.is_some(), cache_loaded) {
                (false, _) => "NOT_INITIALIZED",
                (true, true) => "GENERATED",
                (true, false) => "GENERATED_NOT_LOADED",
            };
            AdminRosterEntry {
                slot: admin_slot_label(idx),
                admin_pubkey: Some(admin_pubkey.clone()),
                admin_name: user
                    .map(|user| {
                        if user.admin_name.trim().is_empty() {
                            format!("{province}省级管理员")
                        } else {
                            user.admin_name.clone()
                        }
                    })
                    .or_else(|| local_slot.map(|slot| slot.admin_name.clone())),
                signing_status,
                signing_pubkey,
                signing_created_at,
                cache_loaded,
                can_operate_signing: same_admin_pubkey(admin_pubkey.as_str(), current_admin_pubkey),
                can_manage_roster,
            }
        })
        .collect();

    Ok(AdminRosterOutput {
        province: province.to_string(),
        current_admin_pubkey: current_admin_pubkey.to_string(),
        entries,
    })
}

/// `GET /api/v1/admin/sheng-admin/roster`。
pub(crate) async fn list_roster_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.admin_province.clone() else {
        return crate::api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    };
    match fetch_roster(province.as_str()).await {
        Ok(main) => {
            match render_admin_roster(&state, province.as_str(), ctx.admin_pubkey.as_str(), main) {
                Ok(data) => Json(ApiResponse {
                    code: 0,
                    message: "ok".to_string(),
                    data,
                })
                .into_response(),
                Err(resp) => resp,
            }
        }
        Err(err) => {
            tracing::warn!(province = %province, error = %err, "fetch_roster failed");
            crate::api_error(StatusCode::SERVICE_UNAVAILABLE, 1502, "roster unavailable")
        }
    }
}

/// `POST /api/v1/admin/sheng-admin/backup`。
///
/// 中文注释:主管理员新增/设置本省备用管理员。前端必须扫码得到账户并转成
/// 0x 公钥后提交;本接口只承接扫码后的规范化账户。
pub(crate) async fn set_backup_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SetBackupAdminInput>,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.admin_province.clone() else {
        return crate::api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    };
    let backup_slot = match input.slot.as_str() {
        "Backup1" => "Backup1",
        "Backup2" => "Backup2",
        _ => {
            return crate::api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "slot must be Backup1/Backup2",
            )
        }
    };
    let admin_name = input.admin_name.trim();
    if admin_name.is_empty() {
        return crate::api_error(StatusCode::BAD_REQUEST, 1001, "admin_name is required");
    }
    if admin_name.chars().count() > 200 {
        return crate::api_error(StatusCode::BAD_REQUEST, 1001, "admin_name too long");
    }
    let Some(admin_pubkey) = parse_sr25519_pubkey(input.admin_pubkey.as_str()) else {
        return crate::api_error(StatusCode::BAD_REQUEST, 1001, "invalid admin_pubkey");
    };

    let main = match fetch_roster(province.as_str()).await {
        Ok(v) => format!("0x{}", hex::encode(v)),
        Err(err) => {
            tracing::warn!(province = %province, error = %err, "fetch_roster failed before set backup");
            return crate::api_error(StatusCode::SERVICE_UNAVAILABLE, 1502, "roster unavailable");
        }
    };
    if !same_admin_pubkey(main.as_str(), ctx.admin_pubkey.as_str()) {
        return crate::api_error(StatusCode::FORBIDDEN, 1003, "main sheng admin required");
    }
    if same_admin_pubkey(main.as_str(), admin_pubkey.as_str()) {
        return crate::api_error(
            StatusCode::CONFLICT,
            1005,
            "backup admin must differ from main admin",
        );
    }

    let now = Utc::now();
    let new_user = {
        let mut store = match crate::store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let local = store
            .sheng_admin_rosters
            .entry(province.clone())
            .or_insert_with(ShengAdminRosterLocal::default);
        let other = match backup_slot {
            "Backup1" => local.backup_2.as_ref(),
            _ => local.backup_1.as_ref(),
        };
        if other
            .map(|slot| same_admin_pubkey(slot.admin_pubkey.as_str(), admin_pubkey.as_str()))
            .unwrap_or(false)
        {
            return crate::api_error(StatusCode::CONFLICT, 1005, "backup admin duplicated");
        }

        let slot_record = ShengAdminSlotLocal {
            admin_pubkey: admin_pubkey.clone(),
            admin_name: admin_name.to_string(),
            created_by: ctx.admin_pubkey.clone(),
            created_at: now,
            updated_at: Some(now),
        };
        match backup_slot {
            "Backup1" => local.backup_1 = Some(slot_record),
            _ => local.backup_2 = Some(slot_record),
        }

        let user = if let Some(user) = store.admin_users_by_pubkey.get_mut(admin_pubkey.as_str()) {
            if user.role != AdminRole::ShengAdmin {
                return crate::api_error(
                    StatusCode::CONFLICT,
                    1005,
                    "admin_pubkey already used by non-sheng admin",
                );
            }
            user.admin_name = admin_name.to_string();
            user.status = AdminStatus::Active;
            user.updated_at = Some(now);
            user.clone()
        } else {
            let id = store.next_admin_user_id;
            store.next_admin_user_id += 1;
            let user = AdminUser {
                id,
                admin_pubkey: admin_pubkey.clone(),
                admin_name: admin_name.to_string(),
                role: AdminRole::ShengAdmin,
                status: AdminStatus::Active,
                built_in: false,
                created_by: ctx.admin_pubkey.clone(),
                created_at: now,
                updated_at: Some(now),
                city: String::new(),
                encrypted_signing_privkey: None,
                signing_pubkey: None,
                signing_created_at: None,
            };
            store
                .admin_users_by_pubkey
                .insert(admin_pubkey.clone(), user.clone());
            user
        };
        store
            .sheng_admin_province_by_pubkey
            .insert(admin_pubkey.clone(), province.clone());
        user
    };

    let province_for_shard = province.clone();
    let pubkey_for_shard = admin_pubkey.clone();
    let _ = state
        .sharded_store
        .write_global(|g| {
            g.global_admins
                .insert(pubkey_for_shard.clone(), new_user.clone());
            g.sheng_admin_province_by_pubkey
                .insert(pubkey_for_shard, province_for_shard);
        })
        .await;

    match fetch_roster(province.as_str()).await {
        Ok(main) => {
            match render_admin_roster(&state, province.as_str(), ctx.admin_pubkey.as_str(), main) {
                Ok(data) => Json(ApiResponse {
                    code: 0,
                    message: "ok".to_string(),
                    data,
                })
                .into_response(),
                Err(resp) => resp,
            }
        }
        Err(_) => crate::api_error(StatusCode::SERVICE_UNAVAILABLE, 1502, "roster unavailable"),
    }
}
