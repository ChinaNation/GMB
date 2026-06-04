//! 主体管理 HTTP handler
//!
//! 中文注释:本模块只承载跨公权/私权共用的主体名称检查、详情、更新和父机构查询;
//! 公权目录归 gov,私权新增/列表归 private,账户归 accounts,资料库归 docs。
//!
//! ## 当前路由表(admin 端,login 中间件)
//!
//! - GET    /api/v1/institution/check-name                → check_institution_name
//! - GET    /api/v1/institution/:sfid_number                  → get_institution
//! - PATCH  /api/v1/institution/:sfid_number                  → update_institution(两步式第二步)

#![allow(dead_code)]

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::admins::actions::require_admin_security_grant;
use crate::admins::operation_auth::AdminActionType;
use crate::china::provinces;
use crate::login::require_admin_any;
use crate::models::ApiResponse;
use crate::scope::get_visible_scope;
use crate::subjects::http::{
    append_inst_audit_log_best_effort, resolve_created_by, resolve_province_from_sfid_number,
    service_error_to_response,
};
use crate::subjects::model::{InstitutionDetailOutput, MultisigAccount, UpdateInstitutionInput};
use crate::subjects::service::{
    institution_name_exists, institution_name_exists_excluding, institution_name_exists_in_city,
    validate_institution_name, validate_sub_type_with_p1,
};
use crate::subjects::uninorg;
use crate::*;

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
//   - 普通私权(SFR/FFR):`institution_name` **不传**,仅生成 SFID,name 落库为 None,
//     由详情页 `update_institution` 补填。不再在此校验 sub_type。
//   - 教育委员会(JY):手动新增时登记的是学校这个机构,必须填写学校名称。
//   - 普通公权机构/公安局:不走手动创建入口,由自动目录或公安局独立对账维护。

pub(crate) async fn update_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
    Json(input): Json<UpdateInstitutionInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let grant_payload = serde_json::json!({
        "target": sfid_number.clone(),
        "sfid_number": sfid_number.clone(),
        "institution_name": input.institution_name.clone(),
        "sub_type": input.sub_type.clone(),
        "parent_sfid_number": input.parent_sfid_number.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::InstitutionUpdate,
        sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let scope = get_visible_scope(&ctx);

    let province = match resolve_province_from_sfid_number(&sfid_number) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_number",
            )
        }
    };

    // 读取机构,做 scope 校验并缓存 a3/p1
    let sfid_number_r = sfid_number.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            shard.multisig_institutions.get(&sfid_number_r).cloned()
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

    // ── parent_sfid_number:仅 FFR(非法人)可设置,必须指向已存在的法人主体 ──
    let parent_change_requested = input.parent_sfid_number.is_some();
    let new_parent: Option<String> = if parent_change_requested {
        let raw = input
            .parent_sfid_number
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string();
        if !uninorg::requires_parent(existing.a3.as_str()) {
            return api_error(StatusCode::BAD_REQUEST, 1001, "仅非法人(FFR)可设置所属法人");
        }
        if raw.is_empty() {
            // FFR 明确传空串 → 允许清除?两步式第二步"必填"语义下不允许,直接拒
            return api_error(StatusCode::BAD_REQUEST, 1001, "所属法人不能为空");
        }
        // 校验目标机构存在,并由 uninorg 统一判断其是否可作为非法人所属法人。
        let target_province = match resolve_province_from_sfid_number(&raw) {
            Some(p) => p,
            None => {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "所属法人 sfid_number 格式无效",
                )
            }
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
        if !uninorg::can_attach_to_parent_a3(target_inst.a3.as_str()) {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                uninorg::parent_a3_requirement_message(),
            );
        }
        Some(raw)
    } else {
        existing.parent_sfid_number.clone()
    };

    // 全国唯一校验(仅在真正要更新 name 时做)
    if let Some(ref name) = new_name {
        let conflict = match state.store.read() {
            Ok(store) => institution_name_exists_excluding(&store, name, Some(&sfid_number)),
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
        updated.parent_sfid_number = new_parent.clone();
    }

    // 写分片
    let sfid_number_w = sfid_number.clone();
    let updated_shard = updated.clone();
    if let Err(e) = state
        .sharded_store
        .write_province(&province, move |shard| {
            shard
                .multisig_institutions
                .insert(sfid_number_w.clone(), updated_shard);
        })
        .await
    {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            &format!("shard write: {e}"),
        );
    }
    if let Err(e) = state.store.upsert_institution_row(&updated) {
        tracing::error!(error = %e, "institution row upsert failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "institution row write failed",
        );
    }

    // 同步写模块 Store 快照。
    {
        match state.store.write() {
            Ok(mut store) => {
                store
                    .multisig_institutions
                    .insert(sfid_number.clone(), updated.clone());
            }
            Err(e) => {
                tracing::warn!(error = %e, "module store snapshot write failed (institution update)");
            }
        }
    }

    append_inst_audit_log_best_effort(
        &state,
        "INSTITUTION_UPDATE",
        &ctx.admin_pubkey,
        Some(sfid_number.clone()),
        None,
        "SUCCESS",
        format!(
            "sfid={} name={:?} sub_type={:?} parent={:?}",
            sfid_number, updated.institution_name, updated.sub_type, updated.parent_sfid_number,
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
            data: Vec::<crate::subjects::model::ParentInstitutionRow>::new(),
        })
        .into_response();
    }

    let mut hits: Vec<crate::subjects::model::ParentInstitutionRow> = Vec::new();
    const LIMIT: usize = 20;
    for p in provinces() {
        if hits.len() >= LIMIT {
            break;
        }
        let q_clone = q.clone();
        let need = LIMIT - hits.len();
        let read_result = state
            .sharded_store
            .read_province(p.name, move |shard| {
                let mut local: Vec<crate::subjects::model::ParentInstitutionRow> = Vec::new();
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
                    let sfid_lc = inst.sfid_number.to_lowercase();
                    let name_lc = name.to_lowercase();
                    if !sfid_lc.contains(&q_clone) && !name_lc.contains(&q_clone) {
                        continue;
                    }
                    local.push(crate::subjects::model::ParentInstitutionRow {
                        sfid_number: inst.sfid_number.clone(),
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
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    // 中文注释:从 sfid_number 解析省份后读取进程内分片缓存。
    let province = match resolve_province_from_sfid_number(&sfid_number) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_number",
            )
        }
    };
    let sfid_number_r = sfid_number.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_number_r).cloned();
            let accounts: Vec<MultisigAccount> = shard
                .multisig_accounts
                .values()
                .filter(|a| a.sfid_number == sfid_number_r)
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
