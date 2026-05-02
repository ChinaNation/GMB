//! 机构/账户 HTTP handler
//!
//! 中文注释:SFID admin 后台机构/账户的 CRUD + 文档管理。路由注册在 main.rs。
//!
//! ## 当前路由表(admin 端,login 中间件)
//!
//! - GET    /api/v1/institution/check-name                → check_institution_name
//! - POST   /api/v1/institution/create                    → create_institution
//! - POST   /api/v1/institution/:sfid_id/account/create   → create_account
//! - GET    /api/v1/institution/list                      → list_institutions
//! - GET    /api/v1/institution/:sfid_id                  → get_institution
//! - PATCH  /api/v1/institution/:sfid_id                  → update_institution(两步式第二步)
//! - GET    /api/v1/institution/:sfid_id/accounts         → list_accounts
//! - DELETE /api/v1/institution/:sfid_id/account/:account_name → delete_account
//! - GET    /api/v1/institution/:sfid_id/documents        → list_documents
//! - POST   /api/v1/institution/:sfid_id/documents        → upload_document
//! - GET    /api/v1/institution/:sfid_id/documents/:doc_id/download → download_document
//! - DELETE /api/v1/institution/:sfid_id/documents/:doc_id → delete_document
//! - POST   /api/v1/public-security/reconcile             → reconcile_public_security
//!
//! ## 已搬迁(2026-05-01 chain/ 重构)
//!
//! 区块链 / 钱包 pull 用的"机构信息查询"5 个 endpoint(无鉴权)已搬到
//! `crate::chain::institution_info::*`,本文件不再持有。
//!
//! 历史 `POST /api/v1/app/institutions/:sfid_id/chain-sync(sync_institution_chain_state)`
//! 配套 SFID 主动读链场景,2026-05-01 一并下架(SFID 不再读链)。

#![allow(dead_code)]

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::app_core::runtime_ops::append_audit_log;
use crate::institutions::model::{
    account_key_to_string, CreateAccountInput, CreateAccountOutput, CreateInstitutionInput,
    CreateInstitutionOutput, InstitutionDetailOutput, InstitutionDocument, InstitutionListRow,
    MultisigAccount, MultisigInstitution, UpdateInstitutionInput, VALID_DOC_TYPES,
};
use crate::institutions::service::{
    backfill_public_security_city_codes, can_delete_account, derive_category,
    ensure_account_name_unique, ensure_institution_exists, ensure_institution_not_exists,
    institution_name_exists, institution_name_exists_excluding, institution_name_exists_in_city,
    is_default_account_name, reconcile_public_security_for_province, validate_account_name,
    validate_institution_name, validate_sub_type_with_p1, ReconcileReport, ServiceError,
};
use crate::institutions::store;
use crate::login::require_admin_any;
use crate::models::{ApiResponse, InstitutionChainStatus, MultisigChainStatus};
use crate::scope::{filter_by_scope, get_visible_scope};
use crate::sfid::province::{province_name_by_code, PROVINCES};
use crate::sfid::{generate_sfid_code, validate_sfid_id_format, GenerateSfidInput};
use crate::*;

const MAX_PROVINCE_CHARS: usize = 100;
const MAX_CITY_CHARS: usize = 100;

// ─── 辅助 ────────────────────────────────────────────────────────

fn service_error_to_response(e: ServiceError) -> axum::response::Response {
    let status = match e {
        ServiceError::BadInput(_) => StatusCode::BAD_REQUEST,
        ServiceError::NotFound(_) => StatusCode::NOT_FOUND,
        ServiceError::Conflict(_) => StatusCode::CONFLICT,
    };
    let code = match e {
        ServiceError::BadInput(_) => 1001,
        ServiceError::NotFound(_) => 1004,
        ServiceError::Conflict(_) => 1007,
    };
    api_error(status, code, e.message())
}

fn extract_province_code(sfid: &str) -> String {
    sfid.split('-')
        .nth(1)
        .map(|r5| r5[..2.min(r5.len())].to_string())
        .unwrap_or_default()
}

fn extract_city_code(sfid: &str) -> String {
    // r5 = 省代码 2 字符 + 市代码 3 字符
    sfid.split('-')
        .nth(1)
        .and_then(|r5| {
            if r5.len() >= 5 {
                Some(r5[2..5].to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Phase 2 Day 3 Round 2:从 sfid_id 解析省代码并映射到省名。
/// 用于 handler 层确定 sharded_store 分片 key。
fn resolve_province_from_sfid_id(sfid_id: &str) -> Option<String> {
    let code = extract_province_code(sfid_id);
    if code.is_empty() {
        return None;
    }
    province_name_by_code(&code).map(|n| n.to_string())
}

/// 反查 `created_by` pubkey → (管理员姓名, 角色枚举字符串)。
/// 未命中两者均为 `None`(前端显示"未知")。
fn resolve_created_by(state: &AppState, created_by: &str) -> (Option<String>, Option<String>) {
    let norm = match crate::scope::pubkey::normalize_admin_pubkey(created_by) {
        Some(v) => v,
        None => return (None, None),
    };
    let store = match state.store.read() {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "resolve_created_by: store read failed");
            return (None, None);
        }
    };
    for user in store.admin_users_by_pubkey.values() {
        let Some(user_norm) = crate::scope::pubkey::normalize_admin_pubkey(&user.admin_pubkey)
        else {
            continue;
        };
        if user_norm == norm {
            let role_str = match user.role {
                crate::models::AdminRole::KeyAdmin => "KEY_ADMIN",
                crate::models::AdminRole::ShengAdmin => "SHENG_ADMIN",
                crate::models::AdminRole::ShiAdmin => "SHI_ADMIN",
            };
            // 内置 KeyAdmin 默认 admin_name=""(见 key-admins/mod.rs sync_key_admin_users),
            // 此时返回 None 避免前端误判为"未知";role 仍然填
            let name_opt = if user.admin_name.trim().is_empty() {
                None
            } else {
                Some(user.admin_name.clone())
            };
            return (name_opt, Some(role_str.to_string()));
        }
    }
    (None, None)
}

/// 给机构写入 2 条默认账户(`主账户` / `费用账户`)的未上链本地记录。
///
/// 幂等:如果记录已存在,保持原状不覆盖(保留已有 chain_status / 地址)。
/// 错误吞掉(tracing::warn):机构创建事务已成功提交,账户缺失不应影响主流程。
async fn insert_default_accounts_best_effort(
    state: &AppState,
    sfid_id: &str,
    province: &str,
    created_by: &str,
) {
    use crate::institutions::derive::derive_duoqian_address;
    use crate::institutions::service::DEFAULT_ACCOUNT_NAMES;
    let now = chrono::Utc::now();
    let sfid_owned = sfid_id.to_string();
    let creator_owned = created_by.to_string();
    let write_result = state
        .sharded_store
        .write_province(province, move |shard| {
            for name in DEFAULT_ACCOUNT_NAMES {
                let key = account_key_to_string(&sfid_owned, name);
                // DUOQIAN_V1 派生:主账户→OP_MAIN(0x00)、费用账户→OP_FEE(0x01)。
                // 两者 preimage 都不含 account_name,地址由 sfid_id 决定,本地立即确定。
                let addr = derive_duoqian_address(&sfid_owned, name);
                shard
                    .multisig_accounts
                    .entry(key)
                    .or_insert_with(|| MultisigAccount {
                        sfid_id: sfid_owned.clone(),
                        account_name: (*name).to_string(),
                        duoqian_address: addr,
                        chain_status: MultisigChainStatus::NotOnChain,
                        chain_synced_at: None,
                        chain_tx_hash: None,
                        chain_block_number: None,
                        created_by: creator_owned.clone(),
                        created_at: now,
                    });
            }
        })
        .await;
    if let Err(e) = write_result {
        tracing::warn!(
            sfid = sfid_id,
            error = %e,
            "insert_default_accounts shard write failed; institution create already committed"
        );
        return;
    }
    // 同步写全局 store(best-effort)。
    if let Ok(mut store) = state.store.write() {
        for name in DEFAULT_ACCOUNT_NAMES {
            let key = account_key_to_string(sfid_id, name);
            let addr = derive_duoqian_address(sfid_id, name);
            store
                .multisig_accounts
                .entry(key)
                .or_insert_with(|| MultisigAccount {
                    sfid_id: sfid_id.to_string(),
                    account_name: (*name).to_string(),
                    duoqian_address: addr,
                    chain_status: MultisigChainStatus::NotOnChain,
                    chain_synced_at: None,
                    chain_tx_hash: None,
                    chain_block_number: None,
                    created_by: created_by.to_string(),
                    created_at: now,
                });
        }
    }
}

/// Phase 2 Day 3 Round 2:两段提交 audit_log(机构/账户版)。
/// 先 sharded_store async 写业务数据(已成功),再全局 store 短锁写审计。
/// 审计写失败只记 WARN,不影响业务返回。
#[allow(clippy::too_many_arguments)]
fn append_inst_audit_log_best_effort(
    state: &AppState,
    action: &'static str,
    actor_pubkey: &str,
    target_pubkey: Option<String>,
    target_archive_no: Option<String>,
    result: &'static str,
    detail: String,
) {
    match state.store.write() {
        Ok(mut store) => {
            append_audit_log(
                &mut store,
                action,
                actor_pubkey,
                target_pubkey,
                target_archive_no,
                result,
                detail,
            );
        }
        Err(e) => {
            tracing::warn!(action, error = %e, "append_audit_log failed (inst shard write already committed)");
        }
    }
}

// ─── 0. 机构名称查重(私权=全国唯一,公权=同城唯一) ──────────────

pub(crate) async fn check_institution_name(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<CheckNameQuery>,
) -> impl IntoResponse {
    // 需要登录态
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }
    let name = params.name.trim().to_string();
    if name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "name is required");
    }
    let a3 = params.a3.as_deref().unwrap_or("").trim().to_string();
    let city = params.city.as_deref().unwrap_or("").trim().to_string();
    let exists = match state.store.read() {
        Ok(store) => {
            if a3 == "GFR" {
                // 公权机构:同城查重
                if city.is_empty() {
                    return api_error(StatusCode::BAD_REQUEST, 1001, "公权机构查重需要 city 参数");
                }
                institution_name_exists_in_city(&store, &name, &city)
            } else {
                // 私权机构(SFR/FFR)或未指定:全国查重
                institution_name_exists(&store, &name)
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "check_institution_name: store read failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed");
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CheckNameResult { exists },
    })
    .into_response()
}

#[derive(Debug, serde::Deserialize)]
pub struct CheckNameQuery {
    pub name: String,
    pub a3: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Serialize)]
struct CheckNameResult {
    exists: bool,
}

// ─── 1. 创建机构(不上链)─────────────────────────────────────────
//
// 两步式(2026-04-19):
//   - 私权(SFR/FFR):`institution_name` **不传**,仅生成 SFID,name 落库为 None,
//     由详情页 `update_institution` 补填。不再在此校验 sub_type。
//   - 公权(GFR)/公安局:`institution_name` 必传并做同城查重(公权)或"公民安全局"
//     固定名称(公安局);保留原来的行为不改造。

pub(crate) async fn create_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateInstitutionInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    let a3 = input.a3.trim().to_string();
    let institution_code = input.institution.trim().to_string();
    let p1_input = input.p1.as_deref().unwrap_or("").trim().to_string();
    if a3.is_empty() || institution_code.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "a3 and institution are required",
        );
    }

    // 是否私权两步式流程(此时允许 institution_name 缺失)
    let is_private = matches!(a3.as_str(), "SFR" | "FFR");

    // ── 机构名称:私权两步式允许 None;公权必填 ──
    let institution_name_opt: Option<String> =
        match input.institution_name.as_deref().map(str::trim) {
            Some(raw) if !raw.is_empty() => match validate_institution_name(raw) {
                Ok(v) => Some(v),
                Err(e) => return service_error_to_response(e),
            },
            _ => {
                if !is_private {
                    return api_error(
                        StatusCode::BAD_REQUEST,
                        1001,
                        "institution_name is required for non-private institutions",
                    );
                }
                None
            }
        };

    let province = match scope.locked_province.clone() {
        Some(locked) => {
            if let Some(raw) = input.province.as_deref() {
                if !raw.trim().is_empty() && raw.trim() != locked {
                    return api_error(
                        StatusCode::FORBIDDEN,
                        1003,
                        "province out of current admin scope",
                    );
                }
            }
            locked
        }
        None => match input.province.as_deref() {
            Some(raw) if !raw.trim().is_empty() => raw.trim().to_string(),
            _ => return api_error(StatusCode::BAD_REQUEST, 1001, "province is required"),
        },
    };
    if province.chars().count() > MAX_PROVINCE_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province too long");
    }
    let mut city = input.city.trim().to_string();
    // SHI_ADMIN 锁定市
    if let Some(locked_city) = scope.locked_city.clone() {
        if !city.is_empty() && city != locked_city {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "city out of current admin scope",
            );
        }
        city = locked_city;
    }
    if city.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "city is required");
    }
    if city.chars().count() > MAX_CITY_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "city too long");
    }

    // ── 分类判定 ──
    // 两步式:私权机构 institution_name 为 None 时,derive_category 传空串不影响
    // (classify 仅在 GFR+ZF+"公民安全局" 才返回 PublicSecurity,对私权无副作用)
    let name_for_classify = institution_name_opt.as_deref().unwrap_or("");
    let category = match derive_category(&a3, &institution_code, name_for_classify) {
        Some(c) => c,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "a3/institution combination is not a valid institution",
            )
        }
    };

    // ── A3 ↔ P1 联动硬校验 ──
    // 两步式:SFR/FFR 均允许 P1=0/1,由用户自主选择(sub_type 联动延后到 update_institution)
    if matches!(a3.as_str(), "SFR" | "FFR") {
        if p1_input != "0" && p1_input != "1" {
            return api_error(StatusCode::BAD_REQUEST, 1001, "P1 非法(仅 0/1)");
        }
    }

    // 两步式:创建阶段禁止 sub_type,由详情页 update_institution 设置。
    if input
        .sub_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some()
    {
        return api_error(StatusCode::BAD_REQUEST, 1001, "创建阶段不接受 sub_type");
    }
    let validated_sub_type: Option<String> = None;

    // ── 机构名称唯一校验:公权机构同城唯一,私权若未命名则跳过 ──
    if let Some(ref name) = institution_name_opt {
        let is_public = a3 == "GFR";
        let name_exists = match state.store.read() {
            Ok(store) => {
                if is_public {
                    institution_name_exists_in_city(&store, name, &city)
                } else {
                    institution_name_exists(&store, name)
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "create_institution: store read failed for name check");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed");
            }
        };
        if name_exists {
            if is_public {
                return api_error(StatusCode::CONFLICT, 1007, "该市已存在同名机构");
            }
            return api_error(StatusCode::CONFLICT, 1007, "该机构名称已被使用");
        }
    }

    // ── 生成 sfid_id(碰撞重试)──
    for _ in 0..5 {
        let random_account = Uuid::new_v4().to_string();
        let site_sfid = match generate_sfid_code(GenerateSfidInput {
            account_pubkey: random_account.as_str(),
            a3: a3.as_str(),
            p1: p1_input.as_str(),
            province: province.as_str(),
            city: city.as_str(),
            institution: institution_code.as_str(),
        }) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
        };
        let site_sfid = match validate_sfid_id_format(site_sfid.as_str()) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg),
        };
        let province_code = extract_province_code(&site_sfid);
        let city_code = extract_city_code(&site_sfid);

        // Phase 2 Day 3 Round 2:写 sharded_store 分片
        let inst = MultisigInstitution {
            sfid_id: site_sfid.clone(),
            institution_name: institution_name_opt.clone(),
            category,
            a3: a3.clone(),
            p1: p1_input.clone(),
            province: province.clone(),
            city: city.clone(),
            province_code,
            city_code,
            institution_code: institution_code.clone(),
            sub_type: validated_sub_type.clone(),
            parent_sfid_id: None, // 两步式:FFR 的所属法人由 update_institution 设置
            sfid_finalized: true,
            chain_status: InstitutionChainStatus::NotRegistered,
            chain_tx_hash: None,
            chain_block_number: None,
            chain_synced_at: None,
            created_by: ctx.admin_pubkey.clone(),
            created_at: Utc::now(),
        };
        let sfid_key = site_sfid.clone();
        let inst_for_shard = inst.clone();
        let write_result = state
            .sharded_store
            .write_province(&province, move |shard| {
                if shard.multisig_institutions.contains_key(&sfid_key) {
                    return Err(());
                }
                shard.multisig_institutions.insert(sfid_key, inst_for_shard);
                Ok(())
            })
            .await;
        match write_result {
            Ok(Ok(())) => {}
            Ok(Err(())) => continue, // 碰撞,重试
            Err(e) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    &format!("shard write failed: {e}"),
                );
            }
        }

        // 同步写全局 store,供审计与管理员反查读取同一份机构快照。
        {
            match state.store.write() {
                Ok(mut store) => {
                    store
                        .multisig_institutions
                        .insert(site_sfid.clone(), inst.clone());
                }
                Err(e) => {
                    tracing::warn!(error = %e, "global store mirror failed (institution create, shard already committed)");
                }
            }
        }

        // 审计日志走全局 store 短锁
        append_inst_audit_log_best_effort(
            &state,
            "INSTITUTION_CREATE",
            &ctx.admin_pubkey,
            Some(site_sfid.clone()),
            None,
            "SUCCESS",
            format!(
                "sfid={} name={:?} category={:?} province={} city={}",
                site_sfid, institution_name_opt, category, province, city
            ),
        );

        // ── 自动插入 2 条默认未上链账户(主账户 / 费用账户)──
        // 2026-04-21 统一两步模式:所有机构(公权/私权/公安局)创建时立即生成这两条本地
        // 账户记录,状态 = NotOnChain。链上注册成功后只能由链路同步接口改为 ActiveOnChain。
        insert_default_accounts_best_effort(&state, &site_sfid, &province, &ctx.admin_pubkey).await;

        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: CreateInstitutionOutput {
                sfid_id: site_sfid,
                institution_name: institution_name_opt.clone(),
                category,
            },
        })
        .into_response();
    }

    api_error(
        StatusCode::CONFLICT,
        1005,
        "institution sfid_id collision retry exhausted",
    )
}

// ─── 1b. 更新机构详情(两步式第二步)────────────────────────────
//
// PATCH /api/v1/institution/:sfid_id
//   body: { institution_name?: string, sub_type?: string | null }
//
// 仅允许编辑 institution_name 与 sub_type,其他(A3/P1/institution_code/省市)一律
// 不可变 — 那是 SFID 派生的基础属性,创建时已固化。
//
// 校验:
//   - scope:KEY=全国 / SHENG=本省 / SHI=本市
//   - institution_name:格式 + 全国唯一(排除自身 sfid_id)
//   - sub_type:与 (a3, p1) 联动(仅 SFR 可设,FFR/GFR 不得设)

pub(crate) async fn update_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_id): Path<String>,
    Json(input): Json<UpdateInstitutionInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    let province = match resolve_province_from_sfid_id(&sfid_id) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_id",
            )
        }
    };

    // 读取机构,做 scope 校验并缓存 a3/p1
    let sfid_id_r = sfid_id.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            shard.multisig_institutions.get(&sfid_id_r).cloned()
        })
        .await;
    let existing = match read_result {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    if !scope.includes_province(&existing.province) || !scope.includes_city(&existing.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }

    // ── 校验并构造新值 ──
    let new_name: Option<String> = match input.institution_name.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => match validate_institution_name(raw) {
            Ok(v) => Some(v),
            Err(e) => return service_error_to_response(e),
        },
        _ => None, // 字段缺省 → 不更新 name
    };
    let sub_type_change_requested = input.sub_type.is_some();
    let new_sub_type: Option<String> = if sub_type_change_requested {
        match validate_sub_type_with_p1(&existing.a3, &existing.p1, input.sub_type.as_deref()) {
            Ok(v) => v,
            Err(e) => return service_error_to_response(e),
        }
    } else {
        existing.sub_type.clone()
    };

    // ── parent_sfid_id:仅 FFR(非法人)可设置,必须指向已存在的 SFR/GFR ──
    let parent_change_requested = input.parent_sfid_id.is_some();
    let new_parent: Option<String> = if parent_change_requested {
        let raw = input
            .parent_sfid_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string();
        if existing.a3 != "FFR" {
            return api_error(StatusCode::BAD_REQUEST, 1001, "仅非法人(FFR)可设置所属法人");
        }
        if raw.is_empty() {
            // FFR 明确传空串 → 允许清除?两步式第二步"必填"语义下不允许,直接拒
            return api_error(StatusCode::BAD_REQUEST, 1001, "所属法人不能为空");
        }
        // 校验目标机构存在 + a3 ∈ {SFR, GFR}
        let target_province = match resolve_province_from_sfid_id(&raw) {
            Some(p) => p,
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "所属法人 sfid_id 格式无效"),
        };
        let raw_clone = raw.clone();
        let target_read = state
            .sharded_store
            .read_province(&target_province, move |shard| {
                shard.multisig_institutions.get(&raw_clone).cloned()
            })
            .await;
        let target_inst = match target_read {
            Ok(Some(v)) => v,
            Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "所属法人机构不存在"),
            Err(e) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    &format!("shard read: {e}"),
                )
            }
        };
        if target_inst.a3 != "SFR" && target_inst.a3 != "GFR" {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "所属法人必须是私法人(SFR)或公法人(GFR)",
            );
        }
        Some(raw)
    } else {
        existing.parent_sfid_id.clone()
    };

    // 全国唯一校验(仅在真正要更新 name 时做)
    if let Some(ref name) = new_name {
        let conflict = match state.store.read() {
            Ok(store) => institution_name_exists_excluding(&store, name, Some(&sfid_id)),
            Err(e) => {
                tracing::warn!(error = %e, "update_institution: store read failed for name check");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed");
            }
        };
        if conflict {
            return api_error(StatusCode::CONFLICT, 1007, "该机构名称已被使用");
        }
    }

    // 构造更新后的实例(clone existing + overlay)
    let mut updated = existing.clone();
    if let Some(name) = new_name.clone() {
        updated.institution_name = Some(name);
    }
    if sub_type_change_requested {
        updated.sub_type = new_sub_type.clone();
    }
    if parent_change_requested {
        updated.parent_sfid_id = new_parent.clone();
    }

    // 写分片
    let sfid_id_w = sfid_id.clone();
    let updated_shard = updated.clone();
    if let Err(e) = state
        .sharded_store
        .write_province(&province, move |shard| {
            shard
                .multisig_institutions
                .insert(sfid_id_w.clone(), updated_shard);
        })
        .await
    {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            &format!("shard write: {e}"),
        );
    }

    // 同步写全局 store。
    {
        match state.store.write() {
            Ok(mut store) => {
                store
                    .multisig_institutions
                    .insert(sfid_id.clone(), updated.clone());
            }
            Err(e) => {
                tracing::warn!(error = %e, "global store mirror failed (institution update)");
            }
        }
    }

    append_inst_audit_log_best_effort(
        &state,
        "INSTITUTION_UPDATE",
        &ctx.admin_pubkey,
        Some(sfid_id.clone()),
        None,
        "SUCCESS",
        format!(
            "sfid={} name={:?} sub_type={:?} parent={:?}",
            sfid_id, updated.institution_name, updated.sub_type, updated.parent_sfid_id,
        ),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: updated,
    })
    .into_response()
}

// ─── 2. 创建账户(只登记 SFID 账户名称,不上链)──────────────────────

pub(crate) async fn create_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_id): Path<String>,
    Json(input): Json<CreateAccountInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    let account_name = match validate_account_name(&input.account_name) {
        Ok(v) => v,
        Err(e) => return service_error_to_response(e),
    };

    // Phase 2 Day 3 Round 2:从 sfid_id 解析省份,读 sharded_store
    let province = match resolve_province_from_sfid_id(&sfid_id) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_id",
            )
        }
    };

    // ── 机构存在 + scope 校验 + 账户名唯一性(从分片读)──
    let sfid_id_r = sfid_id.clone();
    let account_name_r = account_name.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_id_r);
            let inst_clone = inst.cloned();
            let acc_key = account_key_to_string(&sfid_id_r, &account_name_r);
            let acc_exists = shard.multisig_accounts.contains_key(&acc_key);
            (inst_clone, acc_exists)
        })
        .await;
    let (inst_opt, acc_exists) = match read_result {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    let inst = match inst_opt {
        Some(i) => i,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "institution out of current admin scope",
        );
    }
    if acc_exists {
        return service_error_to_response(ServiceError::Conflict(
            "account_name already exists under this institution",
        ));
    }

    // ── 写 NotOnChain 本地记录,**不触链** ──
    // SFID 只登记账户名称;账户激活必须来自链上机构注册/新增账户交易的同步结果。
    // 地址按 DUOQIAN_V1 本地派生(账户名 = "主账户"/"费用账户"/其他,分别走 0x00/0x01/0x05),
    // 不等链上 receipt;链上同步时会再次断言地址一致。
    let now = Utc::now();
    let account = MultisigAccount {
        sfid_id: sfid_id.clone(),
        account_name: account_name.clone(),
        duoqian_address: crate::institutions::derive::derive_duoqian_address(
            &sfid_id,
            &account_name,
        ),
        chain_status: MultisigChainStatus::NotOnChain,
        chain_synced_at: None,
        chain_tx_hash: None,
        chain_block_number: None,
        created_by: ctx.admin_pubkey.clone(),
        created_at: now,
    };
    let acc_for_shard = account.clone();
    let sfid_id_w = sfid_id.clone();
    let account_name_w = account_name.clone();
    if let Err(e) = state
        .sharded_store
        .write_province(&province, move |shard| {
            let key = account_key_to_string(&sfid_id_w, &account_name_w);
            shard.multisig_accounts.insert(key, acc_for_shard);
        })
        .await
    {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            &format!("shard write: {e}"),
        );
    }

    // 同步写全局 store,保持后台反查与审计读取口径一致。
    {
        let acc_key = account_key_to_string(&sfid_id, &account_name);
        match state.store.write() {
            Ok(mut store) => {
                store.multisig_accounts.insert(acc_key, account.clone());
            }
            Err(e) => {
                tracing::warn!(error = %e, "global store mirror failed (account create, shard already committed)");
            }
        }
    }

    append_inst_audit_log_best_effort(
        &state,
        "ACCOUNT_CREATE_NOT_ON_CHAIN",
        &ctx.admin_pubkey,
        Some(sfid_id.clone()),
        None,
        "SUCCESS",
        format!("sfid={} account_name={}", sfid_id, account_name),
    );

    let duoqian_address = account.duoqian_address.clone();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CreateAccountOutput {
            sfid_id,
            account_name,
            chain_status: MultisigChainStatus::NotOnChain,
            chain_synced_at: None,
            chain_tx_hash: None,
            chain_block_number: None,
            duoqian_address,
        },
    })
    .into_response()
}

// ─── 3. 列出机构(按 scope 过滤)──────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ListInstitutionQuery {
    pub category: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    /// 模糊搜索关键字:匹配 institution_name 或 sfid_id 子串(大小写不敏感)。
    /// 为空或缺省时返回 scope 范围内全量。
    /// scope(密钥=全国 / 省级=本省 / 市级=本市)由上游 `filter_by_scope` 自动保证。
    pub q: Option<String>,
}

pub(crate) async fn list_institutions(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<ListInstitutionQuery>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    // Phase 2 Day 3 Round 2:从 sharded_store 遍历分片收集机构列表
    // scope 确定需要扫描哪些省:ShengAdmin/ShiAdmin 只看锁定省,KeyAdmin 全省
    let target_provinces: Vec<String> = if let Some(ref locked) = scope.locked_province {
        vec![locked.clone()]
    } else {
        PROVINCES.iter().map(|p| p.name.to_string()).collect()
    };

    // 预先从全局 store 构建 admin pubkey → (name, role_str) 反查表
    // 用于 InstitutionListRow.created_by_name / created_by_role 填充
    // 归一化 key:去 0x 前缀 + 转小写,匹配 created_by 的格式
    // value = (可选姓名, 角色):name 为空视为 None(内置 KeyAdmin 默认空名)
    let admin_lookup: std::collections::HashMap<String, (Option<String>, &'static str)> =
        match state.store.read() {
            Ok(store) => store
                .admin_users_by_pubkey
                .values()
                .filter_map(|u| {
                    let key = crate::scope::pubkey::normalize_admin_pubkey(&u.admin_pubkey)?;
                    let role_str: &'static str = match u.role {
                        crate::models::AdminRole::KeyAdmin => "KEY_ADMIN",
                        crate::models::AdminRole::ShengAdmin => "SHENG_ADMIN",
                        crate::models::AdminRole::ShiAdmin => "SHI_ADMIN",
                    };
                    let name_opt = if u.admin_name.trim().is_empty() {
                        None
                    } else {
                        Some(u.admin_name.clone())
                    };
                    Some((key, (name_opt, role_str)))
                })
                .collect(),
            Err(e) => {
                tracing::warn!(error = %e, "list_institutions: admin_users lookup read failed");
                std::collections::HashMap::new()
            }
        };

    let query_category = query.category.clone();
    let query_province = query.province.clone();
    let query_city = query.city.clone();
    // 模糊关键字统一小写参与比较
    let query_q: Option<String> = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase());
    let scope_clone = scope.clone();
    let mut rows: Vec<InstitutionListRow> = Vec::new();
    for prov in &target_provinces {
        let q_cat = query_category.clone();
        let q_prov = query_province.clone();
        let q_city = query_city.clone();
        let q_kw = query_q.clone();
        let sc = scope_clone.clone();
        let read_result = state
            .sharded_store
            .read_province(prov, move |shard| {
                // 预计算 sfid_id → 账户数,避免 O(n*m) 遍历
                let account_counts: std::collections::HashMap<&str, usize> = {
                    let mut map = std::collections::HashMap::new();
                    for acc in shard.multisig_accounts.values() {
                        *map.entry(acc.sfid_id.as_str()).or_default() += 1;
                    }
                    map
                };
                let mut province_rows: Vec<InstitutionListRow> = Vec::new();
                for inst in shard.multisig_institutions.values() {
                    if !sc.includes_province(&inst.province) || !sc.includes_city(&inst.city) {
                        continue;
                    }
                    // 中文注释:任务卡 6 bug fix — 前端传的是 SCREAMING_SNAKE_CASE
                    if let Some(ref cat) = q_cat {
                        let target = match cat.trim().to_ascii_uppercase().as_str() {
                            "PUBLIC_SECURITY" | "PUBLICSECURITY" => {
                                Some(crate::sfid::InstitutionCategory::PublicSecurity)
                            }
                            "GOV_INSTITUTION" | "GOVINSTITUTION" => {
                                Some(crate::sfid::InstitutionCategory::GovInstitution)
                            }
                            "PRIVATE_INSTITUTION" | "PRIVATEINSTITUTION" => {
                                Some(crate::sfid::InstitutionCategory::PrivateInstitution)
                            }
                            _ => None,
                        };
                        match target {
                            Some(t) if inst.category == t => {}
                            _ => continue,
                        }
                    }
                    if q_prov.as_deref().map_or(false, |p| inst.province != p) {
                        continue;
                    }
                    if q_city.as_deref().map_or(false, |c| inst.city != c) {
                        continue;
                    }
                    // 模糊关键字:匹配 sfid_id 子串或 institution_name 子串(大小写不敏感)
                    if let Some(ref kw) = q_kw {
                        let sfid_lc = inst.sfid_id.to_lowercase();
                        let name_lc = inst
                            .institution_name
                            .as_deref()
                            .map(|n| n.to_lowercase())
                            .unwrap_or_default();
                        if !sfid_lc.contains(kw) && !name_lc.contains(kw) {
                            continue;
                        }
                    }
                    let account_count = account_counts
                        .get(inst.sfid_id.as_str())
                        .copied()
                        .unwrap_or(0);
                    province_rows.push(InstitutionListRow {
                        sfid_id: inst.sfid_id.clone(),
                        institution_name: inst.institution_name.clone(),
                        category: inst.category,
                        // above: institution_name 为 Option<String>;两步式私权机构未命名时为 None
                        a3: inst.a3.clone(),
                        p1: inst.p1.clone(),
                        province: inst.province.clone(),
                        city: inst.city.clone(),
                        institution_code: inst.institution_code.clone(),
                        sub_type: inst.sub_type.clone(),
                        parent_sfid_id: inst.parent_sfid_id.clone(),
                        chain_status: inst.chain_status.clone(),
                        account_count,
                        created_at: inst.created_at,
                        // 两个字段由外层循环根据 admin_lookup 反查填充(无法跨 shard 闭包传入)
                        created_by_name: None,
                        created_by_role: Some(inst.created_by.clone()),
                    });
                }
                province_rows
            })
            .await;
        match read_result {
            Ok(prows) => rows.extend(prows),
            Err(e) => {
                tracing::warn!(province = %prov, error = %e, "shard read failed in list_institutions");
            }
        }
    }
    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // ── 反查 created_by → (姓名, 角色) ──
    // 上面 shard 闭包里临时把 created_by pubkey 塞进了 created_by_role,这里归一化
    // + 查 admin_lookup;未命中则两字段均置 None(前端显示"未知")
    for row in rows.iter_mut() {
        let raw_created_by = row.created_by_role.take();
        let Some(raw) = raw_created_by else { continue };
        let Some(norm) = crate::scope::pubkey::normalize_admin_pubkey(&raw) else {
            continue;
        };
        if let Some((name_opt, role)) = admin_lookup.get(&norm) {
            row.created_by_name = name_opt.clone();
            row.created_by_role = Some((*role).to_string());
        }
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}

// ─── 3b. 法人机构搜索(FFR 详情页"所属法人"选择器用)───────────
//
// GET /api/v1/institution/search-parents?q=关键字
//   按 sfid_id 子串(大小写不敏感)或 institution_name 子串模糊匹配,
//   仅返回 a3 ∈ {SFR, GFR} 且 institution_name 已补填的机构,
//   最多返回 20 条。**全国范围**可选(FFR 非法人可跨省挂靠法人)。

#[derive(Debug, serde::Deserialize)]
pub(crate) struct SearchParentsQuery {
    pub q: Option<String>,
}

pub(crate) async fn search_parent_institutions(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<SearchParentsQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }
    let q = query.q.as_deref().unwrap_or("").trim().to_lowercase();
    if q.is_empty() {
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: Vec::<crate::institutions::model::ParentInstitutionRow>::new(),
        })
        .into_response();
    }

    let mut hits: Vec<crate::institutions::model::ParentInstitutionRow> = Vec::new();
    const LIMIT: usize = 20;
    for p in PROVINCES {
        if hits.len() >= LIMIT {
            break;
        }
        let q_clone = q.clone();
        let need = LIMIT - hits.len();
        let read_result = state
            .sharded_store
            .read_province(p.name, move |shard| {
                let mut local: Vec<crate::institutions::model::ParentInstitutionRow> = Vec::new();
                for inst in shard.multisig_institutions.values() {
                    if local.len() >= need {
                        break;
                    }
                    // 仅法人(SFR/GFR)且已命名
                    if inst.a3 != "SFR" && inst.a3 != "GFR" {
                        continue;
                    }
                    let name = match &inst.institution_name {
                        Some(n) if !n.trim().is_empty() => n.clone(),
                        _ => continue,
                    };
                    let sfid_lc = inst.sfid_id.to_lowercase();
                    let name_lc = name.to_lowercase();
                    if !sfid_lc.contains(&q_clone) && !name_lc.contains(&q_clone) {
                        continue;
                    }
                    local.push(crate::institutions::model::ParentInstitutionRow {
                        sfid_id: inst.sfid_id.clone(),
                        institution_name: name,
                        a3: inst.a3.clone(),
                        sub_type: inst.sub_type.clone(),
                        category: inst.category,
                        province: inst.province.clone(),
                        city: inst.city.clone(),
                    });
                }
                local
            })
            .await;
        match read_result {
            Ok(mut v) => hits.append(&mut v),
            Err(e) => {
                tracing::warn!(province = %p.name, error = %e, "search_parents shard read failed");
            }
        }
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: hits,
    })
    .into_response()
}

// ─── 4. 机构详情 ─────────────────────────────────────────────────

pub(crate) async fn get_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_id): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    // Phase 2 Day 3 Round 2:从 sfid_id 解析省份,读 sharded_store
    let province = match resolve_province_from_sfid_id(&sfid_id) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_id",
            )
        }
    };
    let sfid_id_r = sfid_id.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_id_r).cloned();
            let accounts: Vec<MultisigAccount> = shard
                .multisig_accounts
                .values()
                .filter(|a| a.sfid_id == sfid_id_r)
                .cloned()
                .collect();
            (inst, accounts)
        })
        .await;
    let (inst_opt, accounts) = match read_result {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    let inst = match inst_opt {
        Some(i) => i,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }

    // 反查 created_by → 管理员姓名 + 角色
    let (created_by_name, created_by_role) = resolve_created_by(&state, &inst.created_by);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: InstitutionDetailOutput {
            institution: inst,
            accounts,
            created_by_name,
            created_by_role,
        },
    })
    .into_response()
}

// ─── 5. 机构下账户列表 ───────────────────────────────────────────

pub(crate) async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_id): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    // Phase 2 Day 3 Round 2:从 sfid_id 解析省份,读 sharded_store
    let province = match resolve_province_from_sfid_id(&sfid_id) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_id",
            )
        }
    };
    let sfid_id_r = sfid_id.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_id_r).cloned();
            let accounts: Vec<MultisigAccount> = shard
                .multisig_accounts
                .values()
                .filter(|a| a.sfid_id == sfid_id_r)
                .cloned()
                .collect();
            (inst, accounts)
        })
        .await;
    let (inst_opt, accounts) = match read_result {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    let inst = match inst_opt {
        Some(i) => i,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: accounts,
    })
    .into_response()
}


// ─── 5b/扫码支付/清算行候选搜索 已搬 chain/institution_info/ ─────────
//
// 5 个 endpoint 全部移到 chain/institution_info/handler.rs:
//   - app_search_institutions / app_get_institution / app_list_accounts
//   - app_search_clearing_banks / app_search_eligible_clearing_banks
//
// 历史 sync_institution_chain_state(POST /app/institutions/:sfid_id/chain-sync)
// 0 caller,与 SFID 不再读链铁律冲突,2026-05-01 一并下架。
//
// 调用入口现走 `crate::chain::institution_info::*` 重新导出。


// ─── 6. 删除账户(软删,不触链)──────────────────────────────────

pub(crate) async fn delete_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((sfid_id, account_name)): Path<(String, String)>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    // 默认账户(主账户 / 费用账户)保护:禁止删除
    // 这两个账户每家机构都自动生成,绑定业务语义(主账户 = Role::Main, 费用账户 = Role::Fee),
    // 不允许从 sfid 系统层面移除;只有删除整个 SFID 时才随机构一起消失。
    if is_default_account_name(&account_name) {
        return api_error(
            StatusCode::CONFLICT,
            1007,
            "默认账户(主账户/费用账户)不可删除",
        );
    }

    // Phase 2 Day 3 Round 2:从 sfid_id 解析省份,操作 sharded_store
    let province = match resolve_province_from_sfid_id(&sfid_id) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_id",
            )
        }
    };

    // 先读:校验机构存在 + scope + 账户状态。SFID 不能删除仍在链上的账户名称。
    let sfid_id_r = sfid_id.clone();
    let account_name_r = account_name.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_id_r).cloned();
            let account = shard
                .multisig_accounts
                .get(&account_key_to_string(&sfid_id_r, &account_name_r))
                .cloned();
            (inst, account)
        })
        .await;
    let (inst, account) = match read_result {
        Ok((Some(i), Some(a))) => (i, a),
        Ok((None, _)) => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
        Ok((Some(_), None)) => return api_error(StatusCode::NOT_FOUND, 1004, "account not found"),
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
    if !can_delete_account(&account) {
        return api_error(
            StatusCode::CONFLICT,
            1007,
            "账户仍在链上或处于上链中,不能在 SFID 系统删除",
        );
    }

    // 写:删除账户
    let sfid_id_w = sfid_id.clone();
    let account_name_w = account_name.clone();
    let remove_result = state
        .sharded_store
        .write_province(&province, move |shard| {
            let key = account_key_to_string(&sfid_id_w, &account_name_w);
            shard.multisig_accounts.remove(&key)
        })
        .await;
    match remove_result {
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "account not found"),
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard write: {e}"),
            )
        }
        Ok(Some(_)) => {}
    }

    // 同步写全局 store,供审计与管理员反查读取同一份账户快照。
    {
        let acc_key = account_key_to_string(&sfid_id, &account_name);
        match state.store.write() {
            Ok(mut store) => {
                store.multisig_accounts.remove(&acc_key);
            }
            Err(e) => {
                tracing::warn!(error = %e, "global store mirror failed (account delete, shard already committed)");
            }
        }
    }

    append_inst_audit_log_best_effort(
        &state,
        "ACCOUNT_DELETE",
        &ctx.admin_pubkey,
        Some(sfid_id.clone()),
        None,
        "SUCCESS",
        format!("sfid={} account={}", sfid_id, account_name),
    );
    #[derive(Serialize)]
    struct Ok {
        deleted: bool,
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: Ok { deleted: true },
    })
    .into_response()
}

// ─── 任务卡 6:公安局对账 ─────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ReconcilePublicSecurityQuery {
    pub province: Option<String>,
}

/// POST /api/v1/public-security/reconcile?province=安徽省
///
/// 中文注释:按 sfid 工具的权威市清单对账该省的公安局机构(增/删/改名)。
/// 进入公安局省详情页前调用,确保数据跟市清单同步。
/// scope 校验:ShengAdmin/ShiAdmin 只能对自己省对账;KeyAdmin 可对任意省,
/// 省参数为空时对 43 省全量对账。
pub(crate) async fn reconcile_public_security(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<ReconcilePublicSecurityQuery>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    let mut store_guard = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    backfill_public_security_city_codes(&mut store_guard);

    let mut reports: Vec<ReconcileReport> = Vec::new();
    match query
        .province
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(province) => {
            if !scope.includes_province(province) {
                return api_error(
                    StatusCode::FORBIDDEN,
                    1003,
                    "province out of current admin scope",
                );
            }
            let report = reconcile_public_security_for_province(
                &mut store_guard,
                province,
                ctx.admin_pubkey.as_str(),
            );
            reports.push(report);
        }
        None => {
            // 无省参数:KeyAdmin 全省对账,其他角色按 scope.provinces 限制
            let target_provinces: Vec<String> = if scope.provinces.is_empty() {
                crate::sfid::province::PROVINCES
                    .iter()
                    .map(|p| p.name.to_string())
                    .collect()
            } else {
                scope.provinces.clone()
            };
            for province in target_provinces {
                let report = reconcile_public_security_for_province(
                    &mut store_guard,
                    province.as_str(),
                    ctx.admin_pubkey.as_str(),
                );
                reports.push(report);
            }
        }
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: reports,
    })
    .into_response()
}

// ─── 资料库:机构文档 CRUD ──────────────────────────────────────

/// GET /api/v1/institution/:sfid_id/documents — 列出某机构的所有文档
pub(crate) async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_id): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut docs: Vec<&InstitutionDocument> = store
        .institution_documents
        .values()
        .filter(|d| d.sfid_id == sfid_id)
        .collect();
    docs.sort_by(|a, b| b.uploaded_at.cmp(&a.uploaded_at));
    let owned: Vec<InstitutionDocument> = docs.into_iter().cloned().collect();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: owned,
    })
    .into_response()
}

/// POST /api/v1/institution/:sfid_id/documents — 上传文档(multipart/form-data)
/// 字段: file(文件), doc_type(文档类型)
pub(crate) async fn upload_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_id): Path<String>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // 校验机构存在 + scope 权限(KEY_ADMIN 全局,SHENG 本省,SHI 本市)
    {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let inst = match store.multisig_institutions.get(&sfid_id) {
            Some(v) => v,
            None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
        };
        if let Some(ref locked_province) = ctx.admin_province {
            if inst.province != *locked_province {
                return api_error(StatusCode::FORBIDDEN, 1003, "province out of scope");
            }
        }
        if let Some(ref locked_city) = ctx.admin_city {
            if inst.city != *locked_city {
                return api_error(StatusCode::FORBIDDEN, 1003, "city out of scope");
            }
        }
    }

    let mut file_name: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut doc_type: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                match field.bytes().await {
                    Ok(bytes) => file_data = Some(bytes.to_vec()),
                    Err(e) => {
                        return api_error(
                            StatusCode::BAD_REQUEST,
                            1001,
                            &format!("读取文件失败: {e}"),
                        );
                    }
                }
            }
            "doc_type" => {
                if let Ok(text) = field.text().await {
                    doc_type = Some(text.trim().to_string());
                }
            }
            _ => {}
        }
    }

    let file_name = match file_name.filter(|s| !s.trim().is_empty()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "file is required"),
    };
    let file_data = match file_data.filter(|d| !d.is_empty()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "file is empty"),
    };
    let doc_type = match doc_type.filter(|s| !s.trim().is_empty()) {
        Some(v) if VALID_DOC_TYPES.contains(&v.as_str()) => v,
        _ => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "doc_type is required and must be one of: 公司章程/营业许可证/股东会决议/法人授权书/其他",
            );
        }
    };

    // 10MB 限制
    if file_data.len() > 10 * 1024 * 1024 {
        return api_error(StatusCode::BAD_REQUEST, 1001, "文件大小不能超过 10MB");
    }

    // 写文件到 data/documents/{sfid_id}/
    let doc_dir = format!("data/documents/{sfid_id}");
    if let Err(e) = std::fs::create_dir_all(&doc_dir) {
        tracing::error!(error = %e, "create document dir failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "create dir failed");
    }
    let file_ext = std::path::Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let stored_name = format!(
        "{}_{}.{}",
        Utc::now().format("%Y%m%d%H%M%S"),
        Uuid::new_v4().as_simple(),
        file_ext
    );
    let stored_path = format!("{doc_dir}/{stored_name}");
    if let Err(e) = std::fs::write(&stored_path, &file_data) {
        tracing::error!(error = %e, "write document file failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "write file failed");
    }

    let file_size = file_data.len() as u64;
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let doc_id = store.next_document_id;
    store.next_document_id += 1;
    let doc = InstitutionDocument {
        id: doc_id,
        sfid_id: sfid_id.clone(),
        file_name,
        doc_type,
        file_size,
        file_path: stored_path,
        uploaded_by: ctx.admin_pubkey.clone(),
        uploaded_at: Utc::now(),
    };
    store
        .institution_documents
        .insert(doc_id.to_string(), doc.clone());
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: doc,
    })
    .into_response()
}

/// GET /api/v1/institution/:sfid_id/documents/:doc_id/download — 下载文档
pub(crate) async fn download_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((sfid_id, doc_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let doc = match store.institution_documents.get(&doc_id) {
        Some(d) if d.sfid_id == sfid_id => d.clone(),
        _ => return api_error(StatusCode::NOT_FOUND, 1004, "document not found"),
    };
    drop(store);

    let file_bytes = match std::fs::read(&doc.file_path) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, path = %doc.file_path, "read document file failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "read file failed");
        }
    };
    // RFC 5987 percent-encode 文件名
    let encoded_name: String = doc
        .file_name
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_' {
                String::from(b as char)
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename*=UTF-8''{encoded_name}"),
            ),
        ],
        Body::from(file_bytes),
    )
        .into_response()
}

/// DELETE /api/v1/institution/:sfid_id/documents/:doc_id — 删除文档
pub(crate) async fn delete_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((sfid_id, doc_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // scope 权限校验
    {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        if let Some(inst) = store.multisig_institutions.get(&sfid_id) {
            if let Some(ref locked_province) = ctx.admin_province {
                if inst.province != *locked_province {
                    return api_error(StatusCode::FORBIDDEN, 1003, "province out of scope");
                }
            }
            if let Some(ref locked_city) = ctx.admin_city {
                if inst.city != *locked_city {
                    return api_error(StatusCode::FORBIDDEN, 1003, "city out of scope");
                }
            }
        }
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let doc = match store.institution_documents.get(&doc_id) {
        Some(d) if d.sfid_id == sfid_id => d.clone(),
        _ => return api_error(StatusCode::NOT_FOUND, 1004, "document not found"),
    };
    store.institution_documents.remove(&doc_id);
    drop(store);

    // 尝试删除文件(best-effort)
    if let Err(e) = std::fs::remove_file(&doc.file_path) {
        tracing::warn!(error = %e, path = %doc.file_path, "remove document file failed");
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "deleted",
    })
    .into_response()
}
