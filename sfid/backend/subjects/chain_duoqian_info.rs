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
use uuid::Uuid;

use crate::core::chain_runtime::build_institution_registration_credential;
use crate::core::response::ApiResponse;
use crate::subjects::service::{
    can_delete_account, default_account_names_for_institution, is_default_account_name,
};
use crate::subjects::MultisigChainStatus;
use crate::*;

#[derive(Serialize)]
pub(crate) struct AppInstitutionDetail {
    pub(crate) sfid_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sfid_full_name: Option<String>,
    pub(crate) category: crate::number::InstitutionCategory,
    pub(crate) subject_property: String,
    pub(crate) p1: String,
    pub(crate) province_name: String,
    pub(crate) city_name: String,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) institution_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) private_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) partnership_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) has_legal_personality: Option<bool>,
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
    pub(crate) sfid_full_name: Option<String>,
    pub(crate) category: crate::number::InstitutionCategory,
    pub(crate) subject_property: String,
    pub(crate) province_name: String,
    pub(crate) city_name: String,
}

#[derive(Serialize)]
pub(crate) struct AppAccountEntry {
    pub(crate) account_name: String,
    pub(crate) duoqian_account: Option<String>,
    pub(crate) chain_status: MultisigChainStatus,
    pub(crate) chain_synced_at: Option<DateTime<Utc>>,
    pub(crate) is_default: bool,
    pub(crate) can_delete: bool,
}

#[derive(Serialize)]
pub(crate) struct AppInstitutionAccounts {
    pub(crate) sfid_number: String,
    pub(crate) sfid_full_name: String,
    pub(crate) accounts: Vec<AppAccountEntry>,
}

#[derive(Serialize)]
pub(crate) struct AppInstitutionRegistrationCredential {
    pub(crate) genesis_hash: String,
    pub(crate) register_nonce: String,
    pub(crate) issuer_sfid_number: String,
    pub(crate) issuer_main_account: String,
    pub(crate) signer_pubkey: String,
    pub(crate) scope_province_name: String,
    pub(crate) scope_city_name: String,
    pub(crate) signature: String,
    pub(crate) meta: crate::core::chain_runtime::RuntimeSignatureMeta,
}

#[derive(Serialize)]
pub(crate) struct AppInstitutionRegistrationInfo {
    pub(crate) sfid_number: String,
    pub(crate) sfid_full_name: String,
    pub(crate) account_names: Vec<String>,
    pub(crate) credential: AppInstitutionRegistrationCredential,
}

fn parse_category(value: &str) -> crate::number::InstitutionCategory {
    match value {
        "PUBLIC_SECURITY" => crate::number::InstitutionCategory::PublicSecurity,
        "GOV_INSTITUTION" => crate::number::InstitutionCategory::GovInstitution,
        _ => crate::number::InstitutionCategory::PrivateInstitution,
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
                "SELECT sfid_number, name, category, subject_property, province_name, city_name
                 FROM subjects
                 WHERE kind IN ('PUBLIC', 'PRIVATE')
                   AND status = 'ACTIVE'
                   AND (
                        $1::text IS NULL
                        OR sfid_number ILIKE '%' || $1 || '%'
                        OR COALESCE(name, '') ILIKE '%' || $1 || '%'
                   )
                 ORDER BY province_name ASC, city_name ASC, sfid_number ASC
                 LIMIT $2",
                &[&q, &limit],
            )
            .map_err(|e| format!("search institutions failed: {e}"))?;
        Ok(rows
            .iter()
            .map(|row| AppInstitutionSearchRow {
                sfid_number: row.get(0),
                sfid_full_name: row.get(1),
                category: parse_category(row.get::<_, String>(2).as_str()),
                subject_property: row.get(3),
                province_name: row.get(4),
                city_name: row.get(5),
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
            sfid_full_name: inst.sfid_full_name,
            category: inst.category,
            subject_property: inst.subject_property,
            p1: inst.p1,
            province_name: inst.province_name,
            city_name: inst.city_name,
            province_code: inst.province_code,
            city_code: inst.city_code,
            institution_code: inst.institution_code,
            private_type: inst.private_type,
            partnership_kind: inst.partnership_kind,
            has_legal_personality: inst.has_legal_personality,
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
        .sfid_full_name
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "sfid_full_name is required before chain registration",
        );
    }
    let required_default_names = default_account_names_for_institution(&inst);
    let mut account_names: Vec<String> = accounts
        .iter()
        .map(|account| account.account_name.clone())
        .filter(|name| !name.trim().is_empty())
        .collect();
    account_names.sort_by(|left, right| {
        let rank = |name: &String| {
            required_default_names
                .iter()
                .position(|default_name| *default_name == name.as_str())
                .unwrap_or(required_default_names.len())
        };
        rank(left).cmp(&rank(right)).then(left.cmp(right))
    });
    account_names.dedup();
    for default_name in required_default_names {
        if !account_names
            .iter()
            .any(|account_name| account_name == default_name)
        {
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "default account_names are required before chain registration",
            );
        }
    }
    let sfid_full_name = inst.sfid_full_name.unwrap_or_default();
    let credential = match build_institution_registration_credential(
        &state,
        &sfid_number,
        &sfid_full_name,
        &account_names,
        Uuid::new_v4().to_string(),
        &inst.province_name,
        &inst.city_name,
    ) {
        Ok(v) => v,
        Err(message) => {
            let detail = format!("institution registration credential sign failed: {message}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str());
        }
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppInstitutionRegistrationInfo {
            sfid_number,
            sfid_full_name,
            account_names,
            credential: AppInstitutionRegistrationCredential {
                genesis_hash: credential.genesis_hash,
                register_nonce: credential.register_nonce,
                issuer_sfid_number: credential.issuer_sfid_number,
                issuer_main_account: credential.issuer_main_account,
                signer_pubkey: credential.signer_pubkey,
                scope_province_name: credential.scope_province_name,
                scope_city_name: credential.scope_city_name,
                signature: credential.signature,
                meta: credential.meta,
            },
        },
    })
    .into_response()
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
            duoqian_account: account.duoqian_account.clone(),
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
            sfid_full_name: inst.sfid_full_name.unwrap_or_default(),
            accounts,
        },
    })
    .into_response()
}
