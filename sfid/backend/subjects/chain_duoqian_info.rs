//! 机构信息查询(chain pull)。
//!
//! 中文注释:公开只读接口直接查询 `subjects/accounts` 结构化表。

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::response::ApiResponse;
use crate::subjects::http::{MAX_CITY_CHARS, MAX_PROVINCE_CHARS};
use crate::subjects::service::{
    can_delete_account, is_default_account_name, DEFAULT_ACCOUNT_NAMES,
};
use crate::subjects::MultisigChainStatus;
use crate::*;

#[derive(Serialize)]
pub(crate) struct AppInstitutionDetail {
    pub(crate) sfid_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) institution_name: Option<String>,
    pub(crate) category: crate::number::InstitutionCategory,
    pub(crate) subject_property: String,
    pub(crate) p1: String,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) institution_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sub_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_sfid_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AppInstitutionSearchQuery {
    pub q: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Clone)]
pub(crate) struct AppInstitutionSearchRow {
    pub(crate) sfid_number: String,
    pub(crate) institution_name: Option<String>,
    pub(crate) category: crate::number::InstitutionCategory,
    pub(crate) subject_property: String,
    pub(crate) province: String,
    pub(crate) city: String,
}

#[derive(Serialize)]
pub(crate) struct AppAccountEntry {
    pub(crate) account_name: String,
    pub(crate) duoqian_address: Option<String>,
    pub(crate) chain_status: MultisigChainStatus,
    pub(crate) chain_synced_at: Option<DateTime<Utc>>,
    pub(crate) is_default: bool,
    pub(crate) can_delete: bool,
}

#[derive(Serialize)]
pub(crate) struct AppInstitutionAccounts {
    pub(crate) sfid_number: String,
    pub(crate) institution_name: String,
    pub(crate) accounts: Vec<AppAccountEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AppClearingBankSearchQuery {
    pub province: Option<String>,
    pub city: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<u32>,
    pub size: Option<u32>,
}

#[derive(Serialize, Clone)]
pub(crate) struct AppClearingBankRow {
    pub(crate) sfid_number: String,
    pub(crate) institution_name: String,
    pub(crate) subject_property: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sub_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_sfid_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_institution_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_subject_property: Option<String>,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) main_account: Option<String>,
    pub(crate) fee_account: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct AppClearingBankSearchOutput {
    pub(crate) total: usize,
    pub(crate) items: Vec<AppClearingBankRow>,
    pub(crate) page: u32,
    pub(crate) size: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EligibleClearingBankSearchQuery {
    pub q: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Clone)]
pub(crate) struct EligibleClearingBankRow {
    pub(crate) sfid_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) institution_name: Option<String>,
    pub(crate) subject_property: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sub_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_sfid_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_institution_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_subject_property: Option<String>,
    pub(crate) province: String,
    pub(crate) city: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) main_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fee_account: Option<String>,
    pub(crate) main_chain_status: MultisigChainStatus,
}

fn parse_category(value: &str) -> crate::number::InstitutionCategory {
    match value {
        "PUBLIC_SECURITY" => crate::number::InstitutionCategory::PublicSecurity,
        "GOV_INSTITUTION" => crate::number::InstitutionCategory::GovInstitution,
        _ => crate::number::InstitutionCategory::PrivateInstitution,
    }
}

fn parse_account_status(value: &str) -> MultisigChainStatus {
    match value {
        "PENDING_ON_CHAIN" => MultisigChainStatus::PendingOnChain,
        "ACTIVE_ON_CHAIN" => MultisigChainStatus::ActiveOnChain,
        "REVOKED_ON_CHAIN" => MultisigChainStatus::RevokedOnChain,
        _ => MultisigChainStatus::NotOnChain,
    }
}

pub(crate) async fn app_search_institutions(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<AppInstitutionSearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20).clamp(1, 50) as i64;
    let q = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() <= 128)
        .map(str::to_string);
    let rows = match state.db.with_client(move |conn| {
        let rows = conn
            .query(
                "SELECT sfid_number, name, category, subject_property, province, city
                 FROM subjects
                 WHERE kind IN ('PUBLIC', 'PRIVATE')
                   AND status = 'ACTIVE'
                   AND (
                        $1::text IS NULL
                        OR sfid_number ILIKE '%' || $1 || '%'
                        OR COALESCE(name, '') ILIKE '%' || $1 || '%'
                   )
                 ORDER BY province ASC, city ASC, sfid_number ASC
                 LIMIT $2",
                &[&q, &limit],
            )
            .map_err(|e| format!("search institutions failed: {e}"))?;
        Ok(rows
            .iter()
            .map(|row| AppInstitutionSearchRow {
                sfid_number: row.get(0),
                institution_name: row.get(1),
                category: parse_category(row.get::<_, String>(2).as_str()),
                subject_property: row.get(3),
                province: row.get(4),
                city: row.get(5),
            })
            .collect::<Vec<_>>())
    }) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "app search institutions failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "institution query failed",
            );
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}

pub(crate) async fn app_get_institution(
    State(state): State<AppState>,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let Some((inst, _)) = (match state.db.get_institution_with_accounts(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query institution failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "institution query failed",
            );
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppInstitutionDetail {
            sfid_number: inst.sfid_number,
            institution_name: inst.institution_name,
            category: inst.category,
            subject_property: inst.subject_property,
            p1: inst.p1,
            province: inst.province,
            city: inst.city,
            province_code: inst.province_code,
            city_code: inst.city_code,
            institution_code: inst.institution_code,
            sub_type: inst.sub_type,
            parent_sfid_number: inst.parent_sfid_number,
        },
    })
    .into_response()
}

pub(crate) async fn app_get_institution_registration_info(
    State(state): State<AppState>,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let sfid_number = sfid_number.trim().to_string();
    if sfid_number.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "sfid_number is required");
    }
    let Some((inst, accounts)) = (match state.db.get_institution_with_accounts(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query registration info failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "institution query failed",
            );
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    if inst
        .institution_name
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "institution_name is required before chain registration",
        );
    }
    let mut account_names: Vec<String> = accounts
        .iter()
        .map(|account| account.account_name.clone())
        .filter(|name| !name.trim().is_empty())
        .collect();
    account_names.sort_by(|left, right| {
        let rank = |name: &String| {
            DEFAULT_ACCOUNT_NAMES
                .iter()
                .position(|default_name| *default_name == name.as_str())
                .unwrap_or(DEFAULT_ACCOUNT_NAMES.len())
        };
        rank(left).cmp(&rank(right)).then(left.cmp(right))
    });
    account_names.dedup();
    for default_name in DEFAULT_ACCOUNT_NAMES {
        if !account_names
            .iter()
            .any(|account_name| account_name == default_name)
        {
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "default account_names 主账户/费用账户 are required before chain registration",
            );
        }
    }
    api_error(
        StatusCode::NOT_IMPLEMENTED,
        5001,
        "institution chain registration requires the city-admin signing flow",
    )
}

pub(crate) async fn app_list_accounts(
    State(state): State<AppState>,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let sfid_number = sfid_number.trim().to_string();
    if sfid_number.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "sfid_number is required");
    }
    let Some((inst, accounts)) = (match state.db.get_institution_with_accounts(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query institution accounts failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "accounts query failed",
            );
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    let accounts = accounts
        .iter()
        .map(|account| AppAccountEntry {
            account_name: account.account_name.clone(),
            duoqian_address: account.duoqian_address.clone(),
            chain_status: account.chain_status.clone(),
            chain_synced_at: account.chain_synced_at,
            is_default: is_default_account_name(&account.account_name),
            can_delete: can_delete_account(account),
        })
        .collect::<Vec<_>>();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppInstitutionAccounts {
            sfid_number,
            institution_name: inst.institution_name.unwrap_or_default(),
            accounts,
        },
    })
    .into_response()
}

fn query_clearing_rows(
    state: &AppState,
    province: Option<String>,
    city: Option<String>,
    keyword: Option<String>,
    active_only: bool,
    limit: Option<i64>,
) -> Result<Vec<EligibleClearingBankRow>, String> {
    state.db.with_client(move |conn| {
        let limit = limit.unwrap_or(10_000);
        let rows = conn
            .query(
                "SELECT s.sfid_number, s.name, s.subject_property, s.sub_type, s.parent_sfid_number,
                        p.name, p.subject_property, s.province, s.city,
                        main.duoqian_address, main.chain_status, fee.duoqian_address
                 FROM subjects s
                 LEFT JOIN subjects p ON p.sfid_number = s.parent_sfid_number
                 LEFT JOIN accounts main
                   ON main.sfid_number = s.sfid_number AND main.account_name = '主账户'
                 LEFT JOIN accounts fee
                   ON fee.sfid_number = s.sfid_number AND fee.account_name = '费用账户'
                 WHERE s.kind = 'PRIVATE'
                   AND s.status = 'ACTIVE'
                   AND (
                        (s.subject_property = 'S' AND s.sub_type = 'JOINT_STOCK')
                        OR (s.subject_property = 'F' AND p.subject_property = 'S' AND p.sub_type = 'JOINT_STOCK')
                   )
                   AND ($1::text IS NULL OR s.province = $1)
                   AND ($2::text IS NULL OR s.city = $2)
                   AND (
                        $3::text IS NULL
                        OR s.sfid_number ILIKE '%' || $3 || '%'
                        OR COALESCE(s.name, '') ILIKE '%' || $3 || '%'
                   )
                   AND (
                        $4::bool = false
                        OR (main.chain_status = 'ACTIVE_ON_CHAIN' AND main.duoqian_address IS NOT NULL)
                   )
                 ORDER BY s.province ASC, s.city ASC, s.sfid_number ASC
                 LIMIT $5",
                &[&province, &city, &keyword, &active_only, &limit],
            )
            .map_err(|e| format!("query clearing banks failed: {e}"))?;
        Ok(rows
            .iter()
            .map(|row| {
                let main_status: Option<String> = row.get(10);
                EligibleClearingBankRow {
                    sfid_number: row.get(0),
                    institution_name: row.get(1),
                    subject_property: row.get(2),
                    sub_type: row.get(3),
                    parent_sfid_number: row.get(4),
                    parent_institution_name: row.get(5),
                    parent_subject_property: row.get(6),
                    province: row.get(7),
                    city: row.get(8),
                    main_account: row.get(9),
                    fee_account: row.get(11),
                    main_chain_status: main_status
                        .as_deref()
                        .map(parse_account_status)
                        .unwrap_or(MultisigChainStatus::NotOnChain),
                }
            })
            .collect::<Vec<_>>())
    })
}

pub(crate) async fn app_search_clearing_banks(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<AppClearingBankSearchQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(20).clamp(1, 100);
    let province = match query
        .province
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(v) if v.len() > MAX_PROVINCE_CHARS => {
            return api_error(StatusCode::BAD_REQUEST, 1001, "province too long")
        }
        Some(v) => Some(v.to_string()),
        None => None,
    };
    let city = match query
        .city
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(v) if v.len() > MAX_CITY_CHARS => {
            return api_error(StatusCode::BAD_REQUEST, 1001, "city too long")
        }
        Some(v) => Some(v.to_string()),
        None => None,
    };
    let keyword = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let rows = match query_clearing_rows(&state, province, city, keyword, true, None) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query clearing banks failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "clearing query failed",
            );
        }
    };
    let total = rows.len();
    let start = ((page.saturating_sub(1)) as usize) * (size as usize);
    let items = rows
        .into_iter()
        .skip(start)
        .take(size as usize)
        .map(|row| AppClearingBankRow {
            sfid_number: row.sfid_number,
            institution_name: row.institution_name.unwrap_or_default(),
            subject_property: row.subject_property,
            sub_type: row.sub_type,
            parent_sfid_number: row.parent_sfid_number,
            parent_institution_name: row.parent_institution_name,
            parent_subject_property: row.parent_subject_property,
            province: row.province,
            city: row.city,
            main_account: row.main_account,
            fee_account: row.fee_account,
        })
        .collect::<Vec<_>>();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppClearingBankSearchOutput {
            total,
            items,
            page,
            size,
        },
    })
    .into_response()
}

pub(crate) async fn app_search_eligible_clearing_banks(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<EligibleClearingBankSearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20).clamp(1, 50) as i64;
    let keyword = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty() && v.len() <= 64)
        .map(str::to_string);
    let rows = match query_clearing_rows(&state, None, None, keyword, false, Some(limit)) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query eligible clearing banks failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "clearing query failed",
            );
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}
