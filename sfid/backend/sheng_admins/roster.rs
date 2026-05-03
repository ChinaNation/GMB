//! 中文注释:注册局-省级管理员页面的一主两备名册展示。
//!
//! 本文件只负责读取和渲染省管理员三槽状态,不负责链上写操作。真正的
//! “更换省管理员/主备交换”后续如果接入区块链,必须放到独立
//! `chain_replace_admin.rs` 中,避免普通页面查询继续混用 `chain_` 命名。
//!
//! 当前 `fetch_roster` 仍是本地基线读取:main 来自内置省管理员清单,
//! backup_1 / backup_2 暂为空。链上 storage pull 待区块链端主备交换能力
//! 对齐后单独接入。
//!
//! ## 与 `institutions/chain_duoqian_info.rs` 的对齐
//!
//! 公开 endpoint 风格、错误码、ApiResponse wrapper 与 duoqian_info 一致。

#![allow(dead_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use sp_core::Pair;

use crate::crypto::pubkey::same_admin_pubkey;
use crate::login::require_sheng_admin;
use crate::models::ApiResponse;
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

/// 拉取本省 3 槽公钥(slot 顺序固定:[main, backup_1, backup_2])。
///
/// 中文注释:当前只读取 SFID 内置 main 管理员基线,backup 槽位等待后续
/// “更换省管理员/主备交换”链上能力落地后接入真实状态源。
pub(crate) async fn fetch_roster(
    province: &str,
) -> Result<[Option<[u8; 32]>; 3], RosterQueryError> {
    let entry = sheng_admin_mains()
        .iter()
        .find(|p| p.province == province)
        .ok_or(RosterQueryError::UnknownProvince)?;
    let main = pubkey_from_hex(entry.pubkey).ok_or(RosterQueryError::PubkeyDecode)?;
    tracing::warn!(
        province = %province,
        "fetch_roster uses local main baseline; backup slots await chain replacement source"
    );
    Ok([Some(main), None, None])
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminRosterOutput {
    pub(crate) province: String,
    pub(crate) current_admin_pubkey: String,
    pub(crate) current_slot: Option<&'static str>,
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

fn admin_slot_label(idx: usize) -> &'static str {
    match idx {
        0 => "Main",
        1 => "Backup1",
        _ => "Backup2",
    }
}

fn render_admin_roster(
    state: &AppState,
    province: &str,
    current_admin_pubkey: &str,
    slots: [Option<[u8; 32]>; 3],
) -> Result<AdminRosterOutput, axum::response::Response> {
    let store = crate::store_read_or_500(state)?;
    let current_slot = slots.iter().enumerate().find_map(|(idx, slot)| {
        let pubkey = slot.map(|p| format!("0x{}", hex::encode(p)))?;
        if same_admin_pubkey(pubkey.as_str(), current_admin_pubkey) {
            Some(admin_slot_label(idx))
        } else {
            None
        }
    });
    let can_manage_roster = current_slot == Some("Main");

    let entries = slots
        .iter()
        .enumerate()
        .map(|(idx, slot)| {
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
                admin_name: user.map(|user| {
                    if user.admin_name.trim().is_empty() {
                        format!("{province}省级管理员")
                    } else {
                        user.admin_name.clone()
                    }
                }),
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
        current_slot,
        entries,
    })
}

/// `GET /api/v1/admin/sheng-admin/roster`(session 触发型,本省名册)。
///
/// 从登录 session 取 admin_province,直接转 `fetch_roster`。
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
        Ok(slots) => {
            match render_admin_roster(&state, province.as_str(), ctx.admin_pubkey.as_str(), slots) {
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
            crate::api_error(StatusCode::SERVICE_UNAVAILABLE, 1502, "chain pull failed")
        }
    }
}
