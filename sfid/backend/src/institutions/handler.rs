//! 机构/账户 HTTP handler
//!
//! 中文注释:所有与机构/账户相关的新 API 入口。路由注册在 main.rs。
//!
//! 路由表:
//! - POST   /api/v1/institution/create                    → create_institution
//! - POST   /api/v1/institution/:sfid_id/account/create   → create_account
//! - GET    /api/v1/institution/list                      → list_institutions
//! - GET    /api/v1/institution/:sfid_id                  → get_institution
//! - GET    /api/v1/institution/:sfid_id/accounts         → list_accounts
//! - DELETE /api/v1/institution/:sfid_id/account/:account_name → delete_account

#![allow(dead_code)]

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
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
    InstitutionDetailOutput, InstitutionListRow, MultisigAccount, MultisigInstitution,
};
use crate::institutions::service::{
    backfill_public_security_city_codes, derive_category, ensure_account_name_unique,
    ensure_institution_exists, ensure_institution_not_exists,
    reconcile_public_security_for_province, validate_account_name, validate_institution_name,
    ReconcileReport, ServiceError,
};
use crate::institutions::store;
use crate::login::require_admin_any;
use crate::models::{ApiResponse, MultisigChainStatus};
use crate::scope::{filter_by_scope, get_visible_scope};
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

        let mut store_guard = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        if store::contains_institution(&store_guard, &site_sfid) {
            drop(store_guard);
            continue;
        }

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
            created_by: ctx.admin_pubkey.clone(),
            created_at: Utc::now(),
        };
        store::insert_institution(&mut store_guard, inst.clone());

        append_audit_log(
            &mut store_guard,
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
        drop(store_guard);

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

    // ── 机构存在 + scope 校验 + 账户名唯一性 ──
    {
        let store_guard = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let inst = match store::get_institution(&store_guard, &sfid_id) {
            Some(i) => i.clone(),
            None => {
                return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
            }
        };
        if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "institution out of current admin scope",
            );
        }
        if let Err(e) = ensure_account_name_unique(&store_guard, &sfid_id, &account_name) {
            return service_error_to_response(e);
        }
    }

    // ── 写 Pending 状态 ──
    let now = Utc::now();
    {
        let mut store_guard = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
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
        store::insert_account(&mut store_guard, account);
        append_audit_log(
            &mut store_guard,
            "ACCOUNT_CREATE_SUBMIT",
            &ctx.admin_pubkey,
            Some(sfid_id.clone()),
            None,
            "SUCCESS",
            format!("sfid={} account_name={}", sfid_id, account_name),
        );
    }

    // ── 推链 ──
    match submit_register_account(&state, &sfid_id, &account_name).await {
        Ok(receipt) => {
            let mut store_guard = match store_write_or_500(&state) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            store::update_account_chain(&mut store_guard, &sfid_id, &account_name, |acc| {
                acc.chain_status = MultisigChainStatus::Registered;
                acc.chain_tx_hash = Some(receipt.tx_hash.clone());
                acc.chain_block_number = Some(receipt.block_number);
            });
            append_audit_log(
                &mut store_guard,
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
            drop(store_guard);
            Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: CreateAccountOutput {
                    sfid_id,
                    account_name,
                    chain_status: MultisigChainStatus::Registered,
                    chain_tx_hash: Some(receipt.tx_hash),
                    chain_block_number: Some(receipt.block_number),
                    duoqian_address: None, // 由前端或后续任务卡从链事件回填
                },
            })
            .into_response()
        }
        Err(err) => {
            let mut store_guard = match store_write_or_500(&state) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            store::update_account_chain(&mut store_guard, &sfid_id, &account_name, |acc| {
                acc.chain_status = MultisigChainStatus::Failed;
            });
            append_audit_log(
                &mut store_guard,
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
            drop(store_guard);
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

    let store_guard = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let mut rows: Vec<InstitutionListRow> = store::all_institutions(&store_guard)
        .into_iter()
        .filter(|inst| scope.includes_province(&inst.province) && scope.includes_city(&inst.city))
        // 中文注释:任务卡 6 bug fix — 前端传的是 SCREAMING_SNAKE_CASE
        // (`PUBLIC_SECURITY` / `GOV_INSTITUTION` / `PRIVATE_INSTITUTION`),
        // 统一转成枚举比较,不再做字符串格式化兼容。
        .filter(|inst| match query.category.as_deref() {
            None => true,
            Some(cat) => {
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
                    Some(t) => inst.category == t,
                    None => false,
                }
            }
        })
        .filter(|inst| query.province.as_deref().map_or(true, |p| inst.province == p))
        .filter(|inst| query.city.as_deref().map_or(true, |c| inst.city == c))
        .map(|inst| InstitutionListRow {
            sfid_id: inst.sfid_id.clone(),
            institution_name: inst.institution_name.clone(),
            category: inst.category,
            a3: inst.a3.clone(),
            p1: inst.p1.clone(),
            province: inst.province.clone(),
            city: inst.city.clone(),
            institution_code: inst.institution_code.clone(),
            account_count: store::count_accounts_of_institution(&store_guard, &inst.sfid_id),
            created_at: inst.created_at,
        })
        .collect();
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

    let store_guard = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inst = match store::get_institution(&store_guard, &sfid_id) {
        Some(i) => i.clone(),
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
    let accounts = store::accounts_of_institution(&store_guard, &sfid_id);
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

    let store_guard = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inst = match store::get_institution(&store_guard, &sfid_id) {
        Some(i) => i.clone(),
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
    let accounts = store::accounts_of_institution(&store_guard, &sfid_id);
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

    let mut store_guard = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inst = match store::get_institution(&store_guard, &sfid_id) {
        Some(i) => i.clone(),
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
    if store::remove_account(&mut store_guard, &sfid_id, &account_name).is_none() {
        return api_error(StatusCode::NOT_FOUND, 1004, "account not found");
    }
    append_audit_log(
        &mut store_guard,
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
