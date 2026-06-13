//! 公权机构目录 —— wuminapp 公民端匿名只读 BFF。
//!
//! 数据来自 SFID 自有 Postgres 确定性目录(subjects + gov + accounts),**与链交互无关**。
//! 复用领域查询 `Db::list_official_institutions_scope`(逻辑留 gov),本层只做:
//!   ① 去鉴权 / 去 scope 锁(公民可查任意省/市);
//!   ② 映射公开 DTO 白名单(丢弃管理员/运营/PII 字段);
//!   ③ 批量补 custom_account_names(单查 accounts 表,不动领域查询)。
//!
//! 路由(挂 app_routes,非 admin):
//!   GET /api/v1/app/public-institutions?province=&city=&q=&org_code=&cursor=&page_size=
//!   GET /api/v1/app/public-institutions/version?province=&city=

use std::collections::HashMap;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::china::{city_code_by_name, province_code_by_name};
use crate::core::response::{ApiResponse, PageResult};
use crate::gov::service::{
    current_gov_manifest_version, gov_manifest_key, GovTargetKind, OfficialReconcileScope,
};
use crate::number::InstitutionCategory;
use crate::subjects::InstitutionListRow;
use crate::*;

/// 公权机构目录公开行(白名单 DTO)。
///
/// 安全红线:**显式不含** created_by_name / created_by_role / cpms_status /
/// install_token_status / identity_service_status / private_type / partnership_kind。
/// 新增字段前必须确认其可公开。
#[derive(Debug, Serialize)]
pub(crate) struct PublicInstitutionRow {
    pub sfid_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub institution_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sfid_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    pub status: String,
    pub category: InstitutionCategory,
    pub subject_property: String,
    pub p1: String,
    pub province: String,
    pub city: String,
    #[serde(default)]
    pub town: String,
    pub institution_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_legal_personality: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_sfid_number: Option<String>,
    pub account_count: usize,
    /// 自定义账户名(op_tag=0x06)。主/费可本地派生不带;绝大多数机构为空。
    pub custom_account_names: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl PublicInstitutionRow {
    /// 从领域行映射公开行;只取白名单字段,管理员/运营/PII 字段一律丢弃。
    fn from_list_row(row: InstitutionListRow, custom_account_names: Vec<String>) -> Self {
        Self {
            sfid_number: row.sfid_number,
            institution_name: row.institution_name,
            sfid_name: row.sfid_name,
            short_name: row.short_name,
            status: row.status,
            category: row.category,
            subject_property: row.subject_property,
            p1: row.p1,
            province: row.province,
            city: row.city,
            town: row.town,
            institution_code: row.institution_code,
            org_code: row.org_code,
            has_legal_personality: row.has_legal_personality,
            parent_sfid_number: row.parent_sfid_number,
            account_count: row.account_count,
            custom_account_names,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct PublicInstitutionListQuery {
    pub province: Option<String>,
    pub city: Option<String>,
    pub q: Option<String>,
    pub org_code: Option<String>,
    pub cursor: Option<String>,
    pub page_size: Option<usize>,
}

/// GET /api/v1/app/public-institutions —— 匿名公权机构目录(按省必填、市可选)。
pub(crate) async fn list_public_institutions(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<PublicInstitutionListQuery>,
) -> impl IntoResponse {
    let Some(province) = query
        .province
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province required");
    };
    let Some(province_code) = province_code_by_name(province) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "unknown province");
    };
    let city = query
        .city
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let city_code = match city {
        Some(name) => match city_code_by_name(province, name) {
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

    let page = match state.db.list_official_institutions_scope(
        province_code,
        city_code,
        keyword,
        org_code,
        offset,
        page_size,
    ) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "public institution list failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "public institution query failed",
            );
        }
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
    let manifest_version = resolve_manifest_version(&state, &directory_scope);

    let sfid_numbers: Vec<String> =
        page.items.iter().map(|r| r.sfid_number.clone()).collect();
    let custom_map = custom_account_names_for(&state, province_code, &sfid_numbers)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "custom account names query failed; default empty");
            HashMap::new()
        });

    let items: Vec<PublicInstitutionRow> = page
        .items
        .into_iter()
        .map(|row| {
            let names = custom_map.get(&row.sfid_number).cloned().unwrap_or_default();
            PublicInstitutionRow::from_list_row(row, names)
        })
        .collect();

    let data = PageResult::<PublicInstitutionRow> {
        items,
        page_size: page.page_size,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
        manifest_version,
        catalog_status: Some("OK".to_string()),
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data,
    })
    .into_response()
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct PublicInstitutionVersionQuery {
    pub province: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Serialize)]
struct PublicInstitutionVersion {
    province: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manifest_version: Option<String>,
}

/// GET /api/v1/app/public-institutions/version —— 某省/市目录版本(增量同步比对用)。
///
/// 中文注释:客户端按省(可选市)低频查版本,变化才重拉该省份目录(省份有界,确定
/// 性目录极少变),兑现 ADR-018 §九 懒同步。
pub(crate) async fn public_institutions_version(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<PublicInstitutionVersionQuery>,
) -> impl IntoResponse {
    let Some(province) = query
        .province
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province required");
    };
    let Some(province_code) = province_code_by_name(province) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "unknown province");
    };
    let city = query
        .city
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let city_code = match city {
        Some(name) => match city_code_by_name(province, name) {
            Some(code) => Some(code),
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "unknown city"),
        },
        None => None,
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
    let data = PublicInstitutionVersion {
        province: province.to_string(),
        city: city.map(str::to_string),
        manifest_version: resolve_manifest_version(&state, &directory_scope),
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data,
    })
    .into_response()
}

/// 取目录 manifest_version:精确 scope 命中优先,回退 All kind。
fn resolve_manifest_version(
    state: &AppState,
    scope: &OfficialReconcileScope,
) -> Option<String> {
    current_gov_manifest_version(
        &state.db,
        gov_manifest_key(scope, GovTargetKind::Official).as_str(),
    )
    .or_else(|| {
        current_gov_manifest_version(
            &state.db,
            gov_manifest_key(scope, GovTargetKind::All).as_str(),
        )
    })
}

/// 批量查机构自定义账户名(op_tag=0x06,即非 5 保留名)。单查 accounts 表,
/// 不动领域查询;空列表短路。
fn custom_account_names_for(
    state: &AppState,
    p_code: &str,
    sfid_numbers: &[String],
) -> Result<HashMap<String, Vec<String>>, String> {
    if sfid_numbers.is_empty() {
        return Ok(HashMap::new());
    }
    let p_code = p_code.to_string();
    let sfids: Vec<String> = sfid_numbers.to_vec();
    let reserved: Vec<String> = crate::accounts::derive::RESERVED_ACCOUNT_NAMES
        .iter()
        .map(|s| s.to_string())
        .collect();
    state.db.with_client(move |conn| {
        let rows = conn
            .query(
                "SELECT sfid_number, account_name
                 FROM accounts
                 WHERE p_code = $1
                   AND sfid_number = ANY($2)
                   AND account_name <> ALL($3)
                 ORDER BY sfid_number ASC, account_name ASC",
                &[&p_code, &sfids, &reserved],
            )
            .map_err(|e| format!("custom account names query failed: {e}"))?;
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for row in rows {
            let sfid: String = row.get(0);
            let name: String = row.get(1);
            map.entry(sfid).or_default().push(name);
        }
        Ok(map)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_list_row() -> InstitutionListRow {
        InstitutionListRow {
            sfid_number: "AH001-ZF000-123456789-2026".to_string(),
            institution_name: Some("安徽省人民政府".to_string()),
            sfid_name: Some("安徽省国民政府".to_string()),
            short_name: Some("皖府".to_string()),
            status: "ACTIVE".to_string(),
            category: InstitutionCategory::GovInstitution,
            subject_property: "G".to_string(),
            p1: "0".to_string(),
            province: "安徽".to_string(),
            city: "合肥".to_string(),
            town: String::new(),
            institution_code: "ZF".to_string(),
            org_code: None,
            private_type: None,
            partnership_kind: None,
            has_legal_personality: Some(true),
            parent_sfid_number: None,
            account_count: 2,
            cpms_status: Some("INSTALLED".to_string()),
            install_token_status: Some("ISSUED".to_string()),
            identity_service_status: Some("ON".to_string()),
            created_at: Utc::now(),
            created_by_name: Some("张三管理员".to_string()),
            created_by_role: Some("FEDERAL_ADMIN".to_string()),
        }
    }

    #[test]
    fn public_dto_excludes_sensitive_admin_fields() {
        let public = PublicInstitutionRow::from_list_row(
            sample_list_row(),
            vec!["业务专户A".to_string()],
        );
        let json = serde_json::to_string(&public).expect("serialize public row");
        for forbidden in [
            "created_by",
            "张三管理员",
            "FEDERAL_ADMIN",
            "cpms_status",
            "INSTALLED",
            "install_token_status",
            "identity_service_status",
            "private_type",
            "partnership_kind",
        ] {
            assert!(
                !json.contains(forbidden),
                "公开 DTO 泄露敏感字段: {forbidden}"
            );
        }
        assert!(json.contains("custom_account_names"));
        assert!(json.contains("业务专户A"));
        assert!(json.contains("安徽省人民政府"));
    }

    #[test]
    fn public_dto_keeps_directory_fields() {
        let public =
            PublicInstitutionRow::from_list_row(sample_list_row(), Vec::new());
        assert_eq!(public.sfid_number, "AH001-ZF000-123456789-2026");
        assert_eq!(public.account_count, 2);
        assert_eq!(public.institution_code, "ZF");
        assert!(public.custom_account_names.is_empty());
    }
}
