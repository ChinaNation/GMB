//! 机构/账户 HTTP handler
//!
//! 中文注释:所有与机构/账户相关的新 API 入口。路由注册在 main.rs。
//!
//! 路由表:
//! - GET    /api/v1/institution/check-name                → check_institution_name
//! - POST   /api/v1/institution/create                    → create_institution
//! - POST   /api/v1/institution/:sfid_id/account/create   → create_account
//! - GET    /api/v1/institution/list                      → list_institutions
//! - GET    /api/v1/institution/:sfid_id                  → get_institution
//! - GET    /api/v1/institution/:sfid_id/accounts         → list_accounts
//! - DELETE /api/v1/institution/:sfid_id/account/:account_name → delete_account
//! - GET    /api/v1/institution/:sfid_id/documents        → list_documents
//! - POST   /api/v1/institution/:sfid_id/documents        → upload_document
//! - GET    /api/v1/institution/:sfid_id/documents/:doc_id/download → download_document
//! - DELETE /api/v1/institution/:sfid_id/documents/:doc_id → delete_document

#![allow(dead_code)]

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::app_core::runtime_ops::append_audit_log;
use crate::institutions::chain::submit_register_account;
use crate::institutions::model::{
    CreateAccountInput, CreateAccountOutput, CreateInstitutionInput, CreateInstitutionOutput,
    InstitutionDetailOutput, InstitutionDocument, InstitutionListRow, MultisigAccount,
    MultisigInstitution, VALID_DOC_TYPES, account_key_to_string,
};
use crate::institutions::service::{
    backfill_public_security_city_codes, derive_category, ensure_account_name_unique,
    ensure_institution_exists, ensure_institution_not_exists, institution_name_exists, institution_name_exists_in_city,
    reconcile_public_security_for_province, validate_account_name, validate_institution_name,
    ReconcileReport, ServiceError,
};
use crate::institutions::store;
use crate::login::require_admin_any;
use crate::models::{ApiResponse, MultisigChainStatus};
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
        .and_then(|r5| if r5.len() >= 5 { Some(r5[2..5].to_string()) } else { None })
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

/// Phase 2 Day 3 Round 2:两段提交 audit_log(机构/账户版)。
/// 先 sharded_store async 写业务数据(已成功),再 legacy store 短锁写审计。
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

// ── 允许的私法人子类型 ──
const VALID_SUB_TYPES: &[&str] = &[
    "SOLE_PROPRIETORSHIP",
    "PARTNERSHIP",
    "LIMITED_LIABILITY",
    "JOINT_STOCK",
];

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

    // ── 输入校验 ──
    let institution_name = match validate_institution_name(&input.institution_name) {
        Ok(v) => v,
        Err(e) => return service_error_to_response(e),
    };
    let a3 = input.a3.trim().to_string();
    let institution_code = input.institution.trim().to_string();
    let p1_input = input.p1.as_deref().unwrap_or("").trim().to_string();
    if a3.is_empty() || institution_code.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "a3 and institution are required");
    }

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

    let category = match derive_category(&a3, &institution_code, &institution_name) {
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
    if a3 == "SFR" && p1_input != "1" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "私法人(SFR)盈利属性必须为盈利(1)");
    }
    if a3 == "FFR" && p1_input != "0" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "非法人(FFR)盈利属性必须为非盈利(0)");
    }

    // ── 私法人企业类型校验 ──
    let validated_sub_type: Option<String> = if a3 == "SFR" {
        match input.sub_type.as_deref().map(str::trim) {
            Some(v) if VALID_SUB_TYPES.contains(&v) => Some(v.to_string()),
            _ => {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "私法人(SFR)必须选择企业类型(SOLE_PROPRIETORSHIP/PARTNERSHIP/LIMITED_LIABILITY/JOINT_STOCK)",
                );
            }
        }
    } else {
        if input.sub_type.as_deref().map_or(false, |v| !v.trim().is_empty()) {
            return api_error(StatusCode::BAD_REQUEST, 1001, "非 SFR 类型不允许传 sub_type");
        }
        None
    };

    // ── 储备银行(CH)仅股份公司(JOINT_STOCK)可选 ──
    if institution_code == "CH" {
        match validated_sub_type.as_deref() {
            Some("JOINT_STOCK") => {} // OK
            _ => {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "储备银行(CH)仅股份公司可选择",
                );
            }
        }
    }

    // ── 机构名称唯一校验:公权机构同城唯一,私权机构全国唯一 ──
    {
        let is_public = a3 == "GFR";
        let name_exists = match state.store.read() {
            Ok(store) => {
                if is_public {
                    institution_name_exists_in_city(&store, &institution_name, &city)
                } else {
                    institution_name_exists(&store, &institution_name)
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
            institution_name: institution_name.clone(),
            category,
            a3: a3.clone(),
            p1: p1_input.clone(),
            province: province.clone(),
            city: city.clone(),
            province_code,
            city_code,
            institution_code: institution_code.clone(),
            sub_type: validated_sub_type.clone(),
            sfid_finalized: true,
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
                shard
                    .multisig_institutions
                    .insert(sfid_key, inst_for_shard);
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

        // 双写过渡期:sharded_store + legacy store 同步写
        {
            match state.store.write() {
                Ok(mut store) => {
                    store.multisig_institutions.insert(site_sfid.clone(), inst.clone());
                }
                Err(e) => {
                    tracing::warn!(error = %e, "dual-write legacy store failed (institution create, shard already committed)");
                }
            }
        }

        // 审计日志走 legacy store 短锁
        append_inst_audit_log_best_effort(
            &state,
            "INSTITUTION_CREATE",
            &ctx.admin_pubkey,
            Some(site_sfid.clone()),
            None,
            "SUCCESS",
            format!(
                "sfid={} name={} category={:?} province={} city={}",
                site_sfid, institution_name, category, province, city
            ),
        );

        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: CreateInstitutionOutput {
                sfid_id: site_sfid,
                institution_name,
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

// ─── 2. 创建账户(上链)──────────────────────────────────────────

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
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "cannot resolve province from sfid_id"),
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
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &format!("shard read: {e}")),
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

    // ── 写 Pending 状态到分片 ──
    let now = Utc::now();
    let account = MultisigAccount {
        sfid_id: sfid_id.clone(),
        account_name: account_name.clone(),
        duoqian_address: None,
        chain_status: MultisigChainStatus::Pending,
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
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &format!("shard write: {e}"));
    }

    // 双写过渡期:sharded_store + legacy store 同步写
    {
        let acc_key = account_key_to_string(&sfid_id, &account_name);
        match state.store.write() {
            Ok(mut store) => {
                store.multisig_accounts.insert(acc_key, account.clone());
            }
            Err(e) => {
                tracing::warn!(error = %e, "dual-write legacy store failed (account create pending, shard already committed)");
            }
        }
    }

    append_inst_audit_log_best_effort(
        &state,
        "ACCOUNT_CREATE_SUBMIT",
        &ctx.admin_pubkey,
        Some(sfid_id.clone()),
        None,
        "SUCCESS",
        format!("sfid={} account_name={}", sfid_id, account_name),
    );

    // ── 推链 ──
    match submit_register_account(&state, &ctx, &sfid_id, &account_name).await {
        Ok(receipt) => {
            // 更新链状态到分片
            let sfid_id_u = sfid_id.clone();
            let account_name_u = account_name.clone();
            let tx_hash = receipt.tx_hash.clone();
            let block_number = receipt.block_number;
            let duoqian_addr = receipt.duoqian_address.clone();
            let duoqian_addr_shard = duoqian_addr.clone();
            let _ = state
                .sharded_store
                .write_province(&province, move |shard| {
                    let key = account_key_to_string(&sfid_id_u, &account_name_u);
                    if let Some(acc) = shard.multisig_accounts.get_mut(&key) {
                        acc.chain_status = MultisigChainStatus::Registered;
                        acc.chain_tx_hash = Some(tx_hash);
                        acc.chain_block_number = Some(block_number);
                        acc.duoqian_address = duoqian_addr_shard;
                    }
                })
                .await;

            // 双写过渡期:sharded_store + legacy store 同步写
            {
                let acc_key = account_key_to_string(&sfid_id, &account_name);
                match state.store.write() {
                    Ok(mut store) => {
                        if let Some(acc) = store.multisig_accounts.get_mut(&acc_key) {
                            acc.chain_status = MultisigChainStatus::Registered;
                            acc.chain_tx_hash = Some(receipt.tx_hash.clone());
                            acc.chain_block_number = Some(receipt.block_number);
                            acc.duoqian_address = duoqian_addr.clone();
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "dual-write legacy store failed (account chain ok, shard already committed)");
                    }
                }
            }

            append_inst_audit_log_best_effort(
                &state,
                "ACCOUNT_CREATE_CHAIN_OK",
                &ctx.admin_pubkey,
                Some(sfid_id.clone()),
                None,
                "SUCCESS",
                format!(
                    "sfid={} account={} tx={} block={}",
                    sfid_id, account_name, receipt.tx_hash, receipt.block_number
                ),
            );
            Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: CreateAccountOutput {
                    sfid_id,
                    account_name,
                    chain_status: MultisigChainStatus::Registered,
                    chain_tx_hash: Some(receipt.tx_hash),
                    chain_block_number: Some(receipt.block_number),
                    duoqian_address: duoqian_addr,
                },
            })
            .into_response()
        }
        Err(err) => {
            // 标记 Failed 到分片
            let sfid_id_f = sfid_id.clone();
            let account_name_f = account_name.clone();
            let _ = state
                .sharded_store
                .write_province(&province, move |shard| {
                    let key = account_key_to_string(&sfid_id_f, &account_name_f);
                    if let Some(acc) = shard.multisig_accounts.get_mut(&key) {
                        acc.chain_status = MultisigChainStatus::Failed;
                    }
                })
                .await;

            // 双写过渡期:sharded_store + legacy store 同步写
            {
                let acc_key = account_key_to_string(&sfid_id, &account_name);
                match state.store.write() {
                    Ok(mut store) => {
                        if let Some(acc) = store.multisig_accounts.get_mut(&acc_key) {
                            acc.chain_status = MultisigChainStatus::Failed;
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "dual-write legacy store failed (account chain fail, shard already committed)");
                    }
                }
            }

            append_inst_audit_log_best_effort(
                &state,
                "ACCOUNT_CREATE_CHAIN_FAIL",
                &ctx.admin_pubkey,
                Some(sfid_id.clone()),
                None,
                "FAILED",
                format!(
                    "sfid={} account={} error={}",
                    sfid_id, account_name, err
                ),
            );
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("chain register failed: {err}"),
            )
        }
    }
}

// ─── 3. 列出机构(按 scope 过滤)──────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ListInstitutionQuery {
    pub category: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
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

    let query_category = query.category.clone();
    let query_province = query.province.clone();
    let query_city = query.city.clone();
    let scope_clone = scope.clone();
    let mut rows: Vec<InstitutionListRow> = Vec::new();
    for prov in &target_provinces {
        let q_cat = query_category.clone();
        let q_prov = query_province.clone();
        let q_city = query_city.clone();
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
                    let account_count = account_counts.get(inst.sfid_id.as_str()).copied().unwrap_or(0);
                    province_rows.push(InstitutionListRow {
                        sfid_id: inst.sfid_id.clone(),
                        institution_name: inst.institution_name.clone(),
                        category: inst.category,
                        a3: inst.a3.clone(),
                        p1: inst.p1.clone(),
                        province: inst.province.clone(),
                        city: inst.city.clone(),
                        institution_code: inst.institution_code.clone(),
                        sub_type: inst.sub_type.clone(),
                        account_count,
                        created_at: inst.created_at,
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

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
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
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "cannot resolve province from sfid_id"),
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
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &format!("shard read: {e}")),
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
        data: InstitutionDetailOutput {
            institution: inst,
            accounts,
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
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "cannot resolve province from sfid_id"),
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
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &format!("shard read: {e}")),
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

    // Phase 2 Day 3 Round 2:从 sfid_id 解析省份,操作 sharded_store
    let province = match resolve_province_from_sfid_id(&sfid_id) {
        Some(p) => p,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "cannot resolve province from sfid_id"),
    };

    // 先读:校验机构存在 + scope
    let sfid_id_r = sfid_id.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            shard.multisig_institutions.get(&sfid_id_r).cloned()
        })
        .await;
    let inst = match read_result {
        Ok(Some(i)) => i,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &format!("shard read: {e}")),
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
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
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &format!("shard write: {e}")),
        Ok(Some(_)) => {}
    }

    // 双写过渡期:sharded_store + legacy store 同步写
    {
        let acc_key = account_key_to_string(&sfid_id, &account_name);
        match state.store.write() {
            Ok(mut store) => {
                store.multisig_accounts.remove(&acc_key);
            }
            Err(e) => {
                tracing::warn!(error = %e, "dual-write legacy store failed (account delete, shard already committed)");
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
    match query.province.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
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
                crate::sfid::province::PROVINCES.iter().map(|p| p.name.to_string()).collect()
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
    let stored_name = format!("{}_{}.{}", Utc::now().format("%Y%m%d%H%M%S"), Uuid::new_v4().as_simple(), file_ext);
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
    let encoded_name: String = doc.file_name.bytes().map(|b| {
        if b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_' {
            String::from(b as char)
        } else {
            format!("%{b:02X}")
        }
    }).collect();
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
