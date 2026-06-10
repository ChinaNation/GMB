//! 主体管理 HTTP handler。
//!
//! 中文注释:跨公权/私权共用的主体查名、详情、更新和父机构查询只读写
//! `subjects/accounts` 结构化表。

use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::admins::actions::require_admin_security_grant;
use crate::admins::login::require_admin_any;
use crate::admins::operation_auth::AdminActionType;
use crate::scope::get_visible_scope;
use crate::subjects::http::{resolve_created_by, service_error_to_response};
use crate::subjects::model::{
    InstitutionDetailOutput, LegalRepresentativePhoto, ParentInstitutionRow, UpdateInstitutionInput,
};
use crate::subjects::service::{
    validate_institution_name, validate_legal_representative_required, validate_sub_type_with_p1,
};
use crate::subjects::uninorg;
use crate::*;

pub(crate) async fn check_institution_name(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<CheckNameQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }
    let name = params.name.trim().to_string();
    if name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "name is required");
    }
    let subject_property = params
        .subject_property
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();
    let city = params.city.as_deref().unwrap_or("").trim().to_string();
    let exists = if subject_property == "G" {
        if city.is_empty() {
            return api_error(StatusCode::BAD_REQUEST, 1001, "公权机构查重需要 city 参数");
        }
        let name = name.clone();
        match state.db.with_client(move |conn| {
            let row = conn
                .query_one(
                    "SELECT EXISTS (
                        SELECT 1 FROM subjects
                        WHERE kind = 'PUBLIC' AND name = $1 AND city = $2
                     )",
                    &[&name, &city],
                )
                .map_err(|e| format!("query city name conflict failed: {e}"))?;
            Ok(row.get(0))
        }) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query institution name failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        }
    } else {
        match state.db.institution_name_exists(&name, None, None, None) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query institution name failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
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
    pub subject_property: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Serialize)]
struct CheckNameResult {
    exists: bool,
}

pub(crate) async fn upload_legal_representative_photo(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }

    let mut file_name: Option<String> = None;
    let mut file_mime: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name().unwrap_or("") != "file" {
            continue;
        }
        file_name = field.file_name().map(|v| v.to_string());
        file_mime = field.content_type().map(|v| v.to_string());
        match field.bytes().await {
            Ok(bytes) => file_data = Some(bytes.to_vec()),
            Err(e) => {
                let message = format!("读取证件照失败: {e}");
                return api_error(StatusCode::BAD_REQUEST, 1001, message.as_str());
            }
        }
    }

    let file_name = match file_name
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "file is required"),
    };
    let file_data = match file_data.filter(|v| !v.is_empty()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "file is empty"),
    };
    if file_data.len() > crate::subjects::service::MAX_LEGAL_REP_PHOTO_BYTES as usize {
        return api_error(StatusCode::BAD_REQUEST, 1001, "证件照不能超过 5MB");
    }
    let mime = file_mime.unwrap_or_else(|| "application/octet-stream".to_string());
    let ext = match mime.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => return api_error(StatusCode::BAD_REQUEST, 1001, "证件照只支持 JPEG/PNG/WebP"),
    };
    let doc_dir = format!("data/legal-rep-photos/{}", Utc::now().format("%Y%m"));
    if let Err(e) = std::fs::create_dir_all(&doc_dir) {
        tracing::error!(error = %e, "create legal representative photo dir failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "create dir failed");
    }
    let stored_name = format!(
        "{}_{}.{}",
        Utc::now().format("%Y%m%d%H%M%S"),
        Uuid::new_v4().as_simple(),
        ext
    );
    let stored_path = format!("{doc_dir}/{stored_name}");
    if let Err(e) = std::fs::write(&stored_path, &file_data) {
        tracing::error!(error = %e, "write legal representative photo failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "write file failed");
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: LegalRepresentativePhoto {
            file_path: stored_path,
            file_name,
            mime_type: mime,
            file_size: file_data.len() as u64,
        },
    })
    .into_response()
}

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
        "legal_rep_name": input.legal_rep_name.clone(),
        "legal_rep_sfid_number": input.legal_rep_sfid_number.clone(),
        "legal_rep_photo_path": input.legal_rep_photo_path.clone(),
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
    let Some((mut existing, _accounts)) =
        (match state.db.get_institution_with_accounts(&sfid_number) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query institution failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        })
    else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    let scope = get_visible_scope(&ctx);
    if !scope.includes_province(&existing.province) || !scope.includes_city(&existing.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }

    if let Some(raw) = input
        .institution_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let new_name = match validate_institution_name(raw) {
            Ok(v) => v,
            Err(e) => return service_error_to_response(e),
        };
        let conflict =
            match state
                .db
                .institution_name_exists(&new_name, None, None, Some(&sfid_number))
            {
                Ok(v) => v,
                Err(err) => {
                    let message = format!("query institution name failed: {err}");
                    return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
                }
            };
        if conflict {
            return api_error(StatusCode::CONFLICT, 1007, "该机构名称已被使用");
        }
        existing.institution_name = Some(new_name);
    }
    if input.sub_type.is_some() {
        existing.sub_type = match validate_sub_type_with_p1(
            &existing.subject_property,
            &existing.p1,
            input.sub_type.as_deref(),
        ) {
            Ok(v) => v,
            Err(e) => return service_error_to_response(e),
        };
    }
    if input.parent_sfid_number.is_some() {
        let raw = input
            .parent_sfid_number
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string();
        if !uninorg::requires_parent(existing.subject_property.as_str()) {
            return api_error(StatusCode::BAD_REQUEST, 1001, "仅非法人(F)可设置所属法人");
        }
        if raw.is_empty() {
            return api_error(StatusCode::BAD_REQUEST, 1001, "所属法人不能为空");
        }
        let Some((target, _)) = (match state.db.get_institution_with_accounts(&raw) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query parent institution failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        }) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "所属法人机构不存在");
        };
        if !uninorg::can_attach_to_parent_subject(target.subject_property.as_str()) {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                uninorg::parent_subject_requirement_message(),
            );
        }
        existing.parent_sfid_number = Some(raw);
    }
    let legal_rep = match validate_legal_representative_required(
        input.legal_rep_name.as_deref(),
        input.legal_rep_sfid_number.as_deref(),
        input.legal_rep_photo_path.as_deref(),
        input.legal_rep_photo_name.as_deref(),
        input.legal_rep_photo_mime.as_deref(),
        input.legal_rep_photo_size,
    ) {
        Ok(v) => v,
        Err(e) => return service_error_to_response(e),
    };
    match state
        .db
        .legal_representative_citizen_exists(legal_rep.sfid_number.as_str())
    {
        Ok(true) => {}
        Ok(false) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "法定代表人身份ID必须选择正常状态公民",
            )
        }
        Err(err) => {
            let message = format!("query legal representative failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }
    existing.legal_rep_name = Some(legal_rep.name);
    existing.legal_rep_sfid_number = Some(legal_rep.sfid_number);
    existing.legal_rep_photo_path = Some(legal_rep.photo_path);
    existing.legal_rep_photo_name = Some(legal_rep.photo_name);
    existing.legal_rep_photo_mime = Some(legal_rep.photo_mime);
    existing.legal_rep_photo_size = Some(legal_rep.photo_size);
    if let Err(err) = state.db.upsert_institution_row(&existing) {
        let message = format!("update institution failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: existing,
    })
    .into_response()
}

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
            data: Vec::<ParentInstitutionRow>::new(),
        })
        .into_response();
    }
    let result = state.db.with_client(move |conn| {
        let rows = conn
            .query(
                "SELECT sfid_number, name, subject_property, sub_type, category, province, city, COALESCE(town, '')
                 FROM subjects
                 WHERE kind IN ('PUBLIC', 'PRIVATE')
                   AND subject_property IN ('S', 'G')
                   AND name IS NOT NULL
                   AND (lower(sfid_number) LIKE '%' || $1 || '%'
                        OR lower(name) LIKE '%' || $1 || '%')
                 ORDER BY name ASC, sfid_number ASC
                 LIMIT 20",
                &[&q],
            )
            .map_err(|e| format!("query parent institutions failed: {e}"))?;
        let mut output = Vec::with_capacity(rows.len());
        for row in rows {
            let category_text: String = row.get(4);
            let Some(category) = crate::institution_category_from_text(category_text.as_str())
            else {
                continue;
            };
            output.push(ParentInstitutionRow {
                sfid_number: row.get(0),
                institution_name: row.get(1),
                subject_property: row.get(2),
                sub_type: row.get(3),
                category,
                province: row.get(5),
                city: row.get(6),
                town: row.get(7),
            });
        }
        Ok(output)
    });
    let hits = match result {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query parent institutions failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: hits,
    })
    .into_response()
}

pub(crate) async fn get_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some((inst, accounts)) = (match state.db.get_institution_with_accounts(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query institution failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    let scope = get_visible_scope(&ctx);
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
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

/// 联邦注册局机构详情(只读)。
/// 联邦注册局是全国唯一机构(位于中枢省),所有省份管理员都需要进入它的机构详情页查看
/// 本省联邦管理员列表,因此这里**不做 scope 校验**(与 get_institution 的唯一区别)。
/// 仍要求已登录管理员;只返回 FEDERAL_REGISTRY 这一个机构。
pub(crate) async fn get_federal_registry(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }
    // 联邦注册局 sfid 来自创世常量(china_zf),按 sfid_number 直接定位,绕过 org_code 与 scope。
    let Some(sfid_number) = crate::gov::service::federal_registry_sfid_number() else {
        return api_error(StatusCode::NOT_FOUND, 1004, "federal registry not configured");
    };
    let Some((inst, accounts)) = (match state.db.get_institution_with_accounts(sfid_number) {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query federal registry failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "federal registry not found");
    };
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
