//! 私权机构 HTTP handler。
//!
//! 中文注释:注册局手动新增的私权机构和普通机构精确查询直接读写结构化表。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::admins::actions::require_admin_security_grant;
use crate::admins::login::require_admin_any;
use crate::admins::operation_auth::AdminActionType;
use crate::china::{city_code_by_name, province_code_by_name};
use crate::number::{
    generate_sfid_number, validate_sfid_number_format, GenerateSfidInput, InstitutionCategory,
};
use crate::scope::get_visible_scope;
use crate::subjects::http::{
    extract_city_code, extract_province_code, insert_default_accounts_best_effort,
    service_error_to_response, MAX_CITY_CHARS, MAX_PROVINCE_CHARS,
};
use crate::subjects::model::{
    CreateInstitutionInput, CreateInstitutionOutput, Institution, InstitutionListFilter,
    InstitutionListRow,
};
use crate::subjects::service::{
    derive_category, validate_institution_name, validate_legal_representative_required,
};
use crate::*;

pub(crate) async fn create_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateInstitutionInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let grant_payload = serde_json::json!({
        "subject_property": input.subject_property.clone(),
        "p1": input.p1.clone(),
        "province": input.province.clone(),
        "city": input.city.clone(),
        "institution": input.institution.clone(),
        "institution_name": input.institution_name.clone(),
        "sub_type": input.sub_type.clone(),
        "legal_rep_name": input.legal_rep_name.clone(),
        "legal_rep_sfid_number": input.legal_rep_sfid_number.clone(),
        "legal_rep_photo_path": input.legal_rep_photo_path.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::InstitutionCreate,
        "*",
        Some(&grant_payload),
    ) {
        return resp;
    }
    let scope = get_visible_scope(&ctx);
    let subject_property = input.subject_property.trim().to_string();
    let institution_code = input.institution.trim().to_string();
    let p1 = input.p1.as_deref().unwrap_or("").trim().to_string();
    if subject_property.is_empty() || institution_code.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "subject_property and institution are required",
        );
    }
    if input
        .sub_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some()
    {
        return api_error(StatusCode::BAD_REQUEST, 1001, "创建阶段不接受 sub_type");
    }
    let is_private = matches!(subject_property.as_str(), "S" | "F");
    let is_education_school = institution_code == "JY";
    let allow_missing_name = is_private && !is_education_school;
    if is_education_school && ctx.admin_city.is_none() {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "教育委员会类型学校只能由市级管理员注册",
        );
    }
    let institution_name = match input.institution_name.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => match validate_institution_name(raw) {
            Ok(v) => Some(v),
            Err(e) => return service_error_to_response(e),
        },
        _ if allow_missing_name => None,
        _ => return api_error(StatusCode::BAD_REQUEST, 1001, "学校名称/机构名称不能为空"),
    };
    let province = match scope.locked_province.clone() {
        Some(locked) => {
            if input
                .province
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty() && *v != locked)
                .is_some()
            {
                return api_error(
                    StatusCode::FORBIDDEN,
                    1003,
                    "province out of current admin scope",
                );
            }
            locked
        }
        None => match input
            .province
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            Some(v) => v.to_string(),
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "province is required"),
        },
    };
    if province.chars().count() > MAX_PROVINCE_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province too long");
    }
    let mut city = input.city.trim().to_string();
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
    let category = match derive_category(
        &subject_property,
        &institution_code,
        institution_name.as_deref().unwrap_or(""),
    ) {
        Some(v) => v,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "subject_property/institution combination is not a valid institution",
            )
        }
    };
    if matches!(category, InstitutionCategory::PublicSecurity) {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "公安局由系统按行政区划自动生成,不得手动创建",
        );
    }
    if matches!(category, InstitutionCategory::GovInstitution) && !is_education_school {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "普通公权机构由系统自动生成,仅教育委员会类型学校允许手动注册",
        );
    }
    if matches!(subject_property.as_str(), "S" | "F") && p1 != "0" && p1 != "1" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "P1 非法(仅 0/1)");
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
    if let Some(ref name) = institution_name {
        let conflict = match state.db.institution_name_exists(name, None, None, None) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query institution name failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        };
        if conflict {
            return api_error(StatusCode::CONFLICT, 1007, "该机构名称已被使用");
        }
    }
    for _ in 0..1000u32 {
        let random_account = Uuid::new_v4().to_string();
        let sfid = match generate_sfid_number(GenerateSfidInput {
            account_pubkey: random_account.as_str(),
            subject_property: subject_property.as_str(),
            p1: p1.as_str(),
            province: province.as_str(),
            city: city.as_str(),
            institution: institution_code.as_str(),
        }) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
        };
        let sfid = match validate_sfid_number_format(sfid.as_str()) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg),
        };
        if state
            .db
            .get_institution_with_accounts(&sfid)
            .ok()
            .flatten()
            .is_some()
        {
            continue;
        }
        let inst = Institution {
            sfid_number: sfid.clone(),
            institution_name: institution_name.clone(),
            sfid_name: institution_name.clone(),
            short_name: institution_name.clone(),
            status: "ACTIVE".to_string(),
            category,
            subject_property: subject_property.clone(),
            p1: p1.clone(),
            province: province.clone(),
            city: city.clone(),
            town: String::new(),
            province_code: extract_province_code(&sfid),
            city_code: extract_city_code(&sfid),
            town_code: String::new(),
            institution_code: institution_code.clone(),
            org_code: None,
            sub_type: None,
            parent_sfid_number: None,
            legal_rep_name: Some(legal_rep.name.clone()),
            legal_rep_sfid_number: Some(legal_rep.sfid_number.clone()),
            legal_rep_photo_path: Some(legal_rep.photo_path.clone()),
            legal_rep_photo_name: Some(legal_rep.photo_name.clone()),
            legal_rep_photo_mime: Some(legal_rep.photo_mime.clone()),
            legal_rep_photo_size: Some(legal_rep.photo_size),
            created_by: ctx.admin_pubkey.clone(),
            created_at: Utc::now(),
        };
        if let Err(err) = state.db.upsert_institution_row(&inst) {
            let message = format!("write institution failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
        insert_default_accounts_best_effort(&state, &sfid, &province, &ctx.admin_pubkey).await;
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: CreateInstitutionOutput {
                sfid_number: sfid,
                institution_name,
                category,
            },
        })
        .into_response();
    }
    api_error(
        StatusCode::CONFLICT,
        1005,
        "institution sfid_number collision retry exhausted",
    )
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ListInstitutionQuery {
    pub category: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub q: Option<String>,
    pub cursor: Option<String>,
    pub page_size: Option<usize>,
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
    // 中文注释:JY 学校机构统一收口教育机构 tab(EDUCATION_INSTITUTION),
    // 私权/公权两路列表同步排除,过滤子句见 InstitutionListFilter。
    let filter = match query.category.as_deref() {
        Some("PRIVATE_INSTITUTION") => InstitutionListFilter::Private,
        Some("GOV_INSTITUTION") => InstitutionListFilter::Gov,
        Some("EDUCATION_INSTITUTION") => InstitutionListFilter::Education,
        Some("PUBLIC_SECURITY") => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "public security uses /api/v1/institutions/public-security",
            )
        }
        _ => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "institution category is required",
            )
        }
    };
    let page_size = query.page_size.unwrap_or(50).clamp(1, 100);
    let empty_page = || PageResult::<InstitutionListRow> {
        items: Vec::new(),
        page_size,
        next_cursor: None,
        has_more: false,
        manifest_version: None,
        catalog_status: None,
    };
    if let (Some(locked), Some(requested)) = (&scope.locked_province, &query.province) {
        if locked != requested {
            return Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: empty_page(),
            })
            .into_response();
        }
    }
    if let (Some(locked), Some(requested)) = (&scope.locked_city, &query.city) {
        if locked != requested {
            return Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: empty_page(),
            })
            .into_response();
        }
    }
    let Some(province_name) = scope
        .locked_province
        .as_deref()
        .or(query.province.as_deref())
    else {
        return api_error(StatusCode::FORBIDDEN, 1003, "province scope required");
    };
    let Some(province_code) = province_code_by_name(province_name) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "unknown province");
    };
    let city_code = match scope.locked_city.as_deref().or(query.city.as_deref()) {
        Some(city_name) => match city_code_by_name(province_name, city_name) {
            Some(code) => Some(code),
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "unknown city"),
        },
        None => None,
    };
    let page = match state.db.list_institutions_exact(
        filter,
        province_code,
        city_code,
        query.q.as_deref().unwrap_or(""),
        query.cursor.as_deref(),
        page_size,
    ) {
        Ok(v) => v,
        Err(e) if e == "invalid page cursor" => {
            return api_error(StatusCode::BAD_REQUEST, 1001, "invalid page cursor")
        }
        Err(err) => {
            let message = format!("institution query failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: page,
    })
    .into_response()
}
