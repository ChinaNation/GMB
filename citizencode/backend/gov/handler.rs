//! 公权机构 HTTP handler
//!
//! 中文注释:本模块只承载确定性公权机构目录,包括公安局目录、公民宪法机构目录
//! 和公安局 reconcile;手动机构新增归 private,账户归 accounts,资料库归 docs。
//!
//! ## 当前路由表(admin 端,login 中间件)
//!
//! - POST   /api/v1/public-security/reconcile             → reconcile_public_security
//! - GET    /api/v1/institutions/public-security           → list_public_security_institutions
//! - GET    /api/v1/institutions/official                  → list_official_institutions

#![allow(dead_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::admins::actions::require_admin_security_grant;
use crate::admins::login::{require_admin_any, require_federal_registry};
use crate::admins::operation_auth::AdminActionType;
use crate::china::{city_code_by_name, province_code_by_name};
use crate::core::response::ApiResponse;
use crate::gov::service::{
    check_gov_catalog_db, current_gov_manifest_version, gov_manifest_key,
    reconcile_public_security_for_province_db, GovTargetKind, OfficialReconcileScope,
    ReconcileReport,
};
use crate::scope::get_visible_scope;
use crate::subjects::model::InstitutionListRow;
use crate::*;

// ─── 0. 机构名称查重(私权=全国唯一,公权=同城唯一) ──────────────

fn manifest_version_for_scope(
    state: &AppState,
    scope: &OfficialReconcileScope,
    kind: GovTargetKind,
    province_code: &str,
) -> Option<String> {
    current_gov_manifest_version(&state.db, gov_manifest_key(scope, kind).as_str())
        .or_else(|| {
            current_gov_manifest_version(
                &state.db,
                gov_manifest_key(scope, GovTargetKind::All).as_str(),
            )
        })
        .or_else(|| {
            let province_scope = OfficialReconcileScope::Province {
                province_code: province_code.to_string(),
            };
            current_gov_manifest_version(
                &state.db,
                gov_manifest_key(&province_scope, kind).as_str(),
            )
        })
        .or_else(|| {
            let province_scope = OfficialReconcileScope::Province {
                province_code: province_code.to_string(),
            };
            current_gov_manifest_version(
                &state.db,
                gov_manifest_key(&province_scope, GovTargetKind::All).as_str(),
            )
        })
        .or_else(|| {
            current_gov_manifest_version(
                &state.db,
                gov_manifest_key(&OfficialReconcileScope::All, GovTargetKind::All).as_str(),
            )
        })
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ListPublicSecurityQuery {
    pub cursor: Option<String>,
    pub page_size: Option<usize>,
}

/// GET /api/v1/institutions/public-security
///
/// 中文注释:公安局是按 cid 省市代码确定性生成的机构,不是普通公权机构搜索结果。
/// 该接口不接收搜索词:联邦注册局管理员返回本省全部市公安局,市注册局管理员返回本市公安局。
pub(crate) async fn list_public_security_institutions(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<ListPublicSecurityQuery>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.scope_province_name.as_deref() else {
        return api_error(StatusCode::FORBIDDEN, 1003, "province scope required");
    };
    let Some(province_code) = province_code_by_name(province) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "unknown province");
    };
    let city_code = match ctx.scope_city_name.as_deref() {
        Some(city) => match city_code_by_name(province, city) {
            Some(code) => Some(code),
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "unknown city"),
        },
        None => None,
    };
    let page_size = query.page_size.unwrap_or(300).clamp(1, 300);
    let offset = match query
        .cursor
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(raw) => match raw.parse::<usize>() {
            Ok(v) => v,
            Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid page cursor"),
        },
        None => 0,
    };

    let directory_scope = match city_code {
        Some(code) => OfficialReconcileScope::City {
            province_code: province_code.to_string(),
            city_code: code.to_string(),
        },
        None => OfficialReconcileScope::Province {
            province_code: province_code.to_string(),
        },
    };
    let mut page =
        match state
            .db
            .list_public_security_scope(province_code, city_code, offset, page_size)
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "public security list failed");
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "public security query failed",
                );
            }
        };
    page.manifest_version = manifest_version_for_scope(
        &state,
        &directory_scope,
        GovTargetKind::PublicSecurity,
        province_code,
    );
    page.catalog_status = Some("OK".to_string());
    if page.items.is_empty() {
        match check_gov_catalog_db(&state.db, directory_scope, GovTargetKind::PublicSecurity) {
            Ok(report) if !report.ok => {
                return api_error(
                    StatusCode::CONFLICT,
                    1005,
                    "deterministic public security directory is not initialized",
                );
            }
            Err(e) => {
                tracing::warn!(error = %e, "public security directory check failed");
            }
            _ => {}
        }
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: page,
    })
    .into_response()
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ListOfficialInstitutionQuery {
    pub province_name: Option<String>,
    pub city_name: Option<String>,
    pub q: Option<String>,
    pub org_code: Option<String>,
    pub cursor: Option<String>,
    pub page_size: Option<usize>,
}

/// GET /api/v1/institutions/official
///
/// 中文注释:公权机构目录和公安局一样是确定性列表,进入市详情时直接展示。
/// `q` 只作为已展示列表的过滤条件,不能再作为是否返回数据的前提。
pub(crate) async fn list_official_institutions(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<ListOfficialInstitutionQuery>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);
    let empty_page = || PageResult::<InstitutionListRow> {
        items: Vec::new(),
        page_size: query.page_size.unwrap_or(300).clamp(1, 300),
        next_cursor: None,
        has_more: false,
        manifest_version: None,
        catalog_status: None,
    };
    if let (Some(locked), Some(requested)) = (&scope.locked_province_name, &query.province_name) {
        if locked != requested {
            return Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: empty_page(),
            })
            .into_response();
        }
    }
    if let (Some(locked), Some(requested)) = (&scope.locked_city_name, &query.city_name) {
        if locked != requested {
            return Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: empty_page(),
            })
            .into_response();
        }
    }
    let Some(province) = scope
        .locked_province_name
        .clone()
        .or_else(|| query.province_name.clone())
    else {
        return api_error(StatusCode::FORBIDDEN, 1003, "province scope required");
    };
    let city = scope
        .locked_city_name
        .clone()
        .or_else(|| query.city_name.clone());
    let Some(province_code) = province_code_by_name(&province) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "unknown province");
    };
    let city_code = match city.as_deref() {
        Some(city_name) => match city_code_by_name(&province, city_name) {
            Some(code) => Some(code),
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "unknown city"),
        },
        None => None,
    };
    let page_size = query.page_size.unwrap_or(300).clamp(1, 300);
    let offset = match query
        .cursor
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(raw) => match raw.parse::<usize>() {
            Ok(v) => v,
            Err(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid page cursor"),
        },
        None => 0,
    };

    let keyword = query.q.as_deref().map(str::trim).unwrap_or("");
    let org_code = query.org_code.as_deref().map(str::trim).unwrap_or("");
    let directory_scope = match city_code {
        Some(code) => OfficialReconcileScope::City {
            province_code: province_code.to_string(),
            city_code: code.to_string(),
        },
        None => OfficialReconcileScope::Province {
            province_code: province_code.to_string(),
        },
    };
    let mut page = match state.db.list_official_institutions_scope(
        province_code,
        city_code,
        keyword,
        org_code,
        offset,
        page_size,
    ) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "official institution list failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "official institution query failed",
            );
        }
    };
    page.manifest_version = manifest_version_for_scope(
        &state,
        &directory_scope,
        GovTargetKind::Official,
        province_code,
    );
    page.catalog_status = Some("OK".to_string());
    if page.items.is_empty() {
        match check_gov_catalog_db(&state.db, directory_scope, GovTargetKind::Official) {
            Ok(report) if !report.ok => {
                return api_error(
                    StatusCode::CONFLICT,
                    1005,
                    "deterministic gov directory is not initialized",
                );
            }
            Err(e) => {
                tracing::warn!(error = %e, "official directory check failed");
            }
            _ => {}
        }
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: page,
    })
    .into_response()
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ReconcilePublicSecurityQuery {
    pub province_name: Option<String>,
}

pub(crate) async fn reconcile_public_security(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<ReconcilePublicSecurityQuery>,
) -> impl IntoResponse {
    let ctx = match require_federal_registry(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let grant_target = query
        .province_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("*")
        .to_string();
    let grant_payload = serde_json::json!({
        "target": grant_target.clone(),
        "province_name": query.province_name.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::PublicSecurityReconcile,
        grant_target.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let scope = get_visible_scope(&ctx);

    let mut reports: Vec<ReconcileReport> = Vec::new();
    match query
        .province_name
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
            let report = match reconcile_public_security_for_province_db(
                &state.db,
                province,
                ctx.admin_account.as_str(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(error = %e, "public security reconcile failed");
                    return api_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        1004,
                        "public security reconcile failed",
                    );
                }
            };
            reports.push(report);
        }
        None => {
            // 中文注释:scope.provinces 正常必非空;若为空则按全国省份执行显式对账。
            let target_provinces: Vec<String> = if scope.provinces.is_empty() {
                crate::china::provinces()
                    .iter()
                    .map(|p| p.name.to_string())
                    .collect()
            } else {
                scope.provinces.clone()
            };
            for province in target_provinces {
                let report = match reconcile_public_security_for_province_db(
                    &state.db,
                    province.as_str(),
                    ctx.admin_account.as_str(),
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!(error = %e, "public security reconcile failed");
                        return api_error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            1004,
                            "public security reconcile failed",
                        );
                    }
                };
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
