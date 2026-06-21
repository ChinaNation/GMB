//! 机构注册 HTTP handler。
//!
//! 中文注释:本文件只保留跨公权/教育/私权共用的主体注册内核。私权机构入口必须由
//! `private/<type>/` 六类模块传入固定类型规则,不得再由一个 private 总 handler 吞掉。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
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
use crate::private::common::resolve_private_type_rule;
use crate::scope::get_visible_scope;
use crate::subjects::http::{
    extract_city_code, extract_province_code, insert_default_accounts_best_effort,
    service_error_to_response, MAX_CITY_CHARS, MAX_PROVINCE_CHARS,
};
use crate::subjects::model::{
    is_education_school_type, CreateInstitutionInput, CreateInstitutionOutput, Institution,
    InstitutionListFilter, InstitutionListRow,
};
use crate::subjects::service::{
    derive_category, resolve_legal_representative_scope_for_codes,
    validate_legal_representative_required, validate_sfid_full_name,
};
use crate::subjects::uninorg;
use crate::*;

pub(crate) async fn create_institution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateInstitutionInput>,
) -> impl IntoResponse {
    create_institution_inner(state, headers, input, false).await
}

pub(crate) async fn create_private_institution(
    state: AppState,
    headers: HeaderMap,
    input: CreateInstitutionInput,
) -> Response {
    create_institution_inner(state, headers, input, true).await
}

async fn create_institution_inner(
    state: AppState,
    headers: HeaderMap,
    input: CreateInstitutionInput,
    allow_private: bool,
) -> Response {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if !allow_private
        && input
            .private_type
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .is_some()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "私权机构必须使用 /api/v1/private/<type> 创建",
        );
    }
    let grant_payload = serde_json::json!({
        "subject_property": input.subject_property.clone(),
        "p1": input.p1.clone(),
        "province_name": input.province_name.clone(),
        "city_name": input.city_name.clone(),
        "institution": input.institution.clone(),
        "education_type": input.education_type.clone(),
        "sfid_full_name": input.sfid_full_name.clone(),
        "parent_sfid_number": input.parent_sfid_number.clone(),
        "private_type": input.private_type.clone(),
        "partnership_kind": input.partnership_kind.clone(),
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
    let private_rule = match input
        .private_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(private_type) => {
            match resolve_private_type_rule(private_type, input.partnership_kind.as_deref()) {
                Ok(v) => Some(v),
                Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
            }
        }
        None => None,
    };
    // 中文注释:私权入口一旦传入 private_type,主体属性、机构码、P1 与法人资格全部由后端规则锁定,
    // 不信任前端同时提交的旧 subject_property / institution / p1。
    let subject_property = private_rule
        .map(|rule| rule.subject_property.to_string())
        .unwrap_or_else(|| input.subject_property.trim().to_string());
    let institution_code = private_rule
        .map(|rule| rule.institution_code.to_string())
        .unwrap_or_else(|| input.institution.trim().to_string());
    let p1 = private_rule
        .map(|rule| rule.p1.to_string())
        .unwrap_or_else(|| input.p1.as_deref().unwrap_or("").trim().to_string());
    if subject_property.is_empty() || institution_code.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "subject_property and institution are required",
        );
    }
    let is_private = matches!(subject_property.as_str(), "S" | "F");
    let is_education_school = institution_code == "JY";
    let education_type = input
        .education_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let education_type = if is_education_school {
        match subject_property.as_str() {
            "G" | "S" => {
                let Some(value) = education_type else {
                    return api_error(StatusCode::BAD_REQUEST, 1001, "必须选择教育机构类型");
                };
                if !is_education_school_type(value.as_str()) {
                    return api_error(
                        StatusCode::BAD_REQUEST,
                        1001,
                        "教育机构类型仅允许初学/小学/中学/大学",
                    );
                }
                Some(value)
            }
            "F" => {
                if education_type.is_some() {
                    return api_error(
                        StatusCode::BAD_REQUEST,
                        1001,
                        "F+JY 分支机构不使用教育阶段分类",
                    );
                }
                None
            }
            _ => None,
        }
    } else {
        if education_type.is_some() {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "education_type 仅允许教育机构使用",
            );
        }
        None
    };
    if private_rule.is_none() && is_private && !is_education_school && institution_code != "ZG" {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "私权机构必须提交 private_type",
        );
    }
    let sfid_full_name = match input.sfid_full_name.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => match validate_sfid_full_name(raw) {
            Ok(v) => Some(v),
            Err(e) => return service_error_to_response(e),
        },
        _ => return api_error(StatusCode::BAD_REQUEST, 1001, "学校名称/机构名称不能为空"),
    };
    let province = match scope.locked_province_name.clone() {
        Some(locked) => {
            if input
                .province_name
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
            .province_name
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
    let mut city = input.city_name.trim().to_string();
    // 中文注释:机构创建权限统一由省/市 scope 收口:
    // 联邦注册局机构管理员 locked_province_name=本省且 locked_city_name=None,可在本省任意市创建;
    // 市注册局机构管理员同时锁定省和市,只能创建本市机构。
    if let Some(locked_city_name) = scope.locked_city_name.clone() {
        if !city.is_empty() && city != locked_city_name {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "city out of current admin scope",
            );
        }
        city = locked_city_name;
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
        sfid_full_name.as_deref().unwrap_or(""),
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
    // 中文注释:手动公权机构开放 ZF/LF/SF/JC 四类(教育委员会 JY 走教育 tab 学校流程);
    // 公民储备委员会/省储行不开放手动创建,创世目录已确定性生成。
    if matches!(category, InstitutionCategory::GovInstitution)
        && !is_education_school
        && !matches!(institution_code.as_str(), "ZF" | "LF" | "SF" | "JC")
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "手动公权机构仅允许 ZF/LF/SF/JC,公民储备委员会/省储行由系统生成",
        );
    }
    if matches!(subject_property.as_str(), "S" | "F") && p1 != "0" && p1 != "1" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "P1 非法(仅 0/1)");
    }
    // ── 非法人挂靠规则:个体经营/无限合伙是独立非法人,教育分校等从属非法人才需要所属法人 ──
    let parent_sfid_number = match input
        .parent_sfid_number
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(raw) => {
            if !uninorg::requires_parent(subject_property.as_str(), institution_code.as_str()) {
                return api_error(StatusCode::BAD_REQUEST, 1001, "该主体类型不接受所属法人");
            }
            Some(raw.to_string())
        }
        None => {
            if uninorg::requires_parent(subject_property.as_str(), institution_code.as_str()) {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "非法人必须选择所属法人(私法人或公法人)",
                );
            }
            None
        }
    };
    let mut parent_for_legal_rep_scope: Option<Institution> = None;
    if let Some(ref parent_sfid) = parent_sfid_number {
        let Some((parent, _)) = (match state.db.get_institution_with_accounts(parent_sfid) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query parent institution failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        }) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "所属法人机构不存在");
        };
        if !uninorg::can_attach_to_parent_subject(parent.subject_property.as_str()) {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                uninorg::parent_subject_requirement_message(),
            );
        }
        if let Some(msg) = uninorg::code_consistency_violation(
            institution_code.as_str(),
            parent.institution_code.as_str(),
            parent.org_code.as_deref(),
        ) {
            return api_error(StatusCode::BAD_REQUEST, 1001, msg);
        }
        let rule = uninorg::parent_locality_rule(
            parent.subject_property.as_str(),
            parent.institution_code.as_str(),
            parent.org_code.as_deref(),
        );
        if let Some(msg) = uninorg::locality_violation(
            rule,
            &parent.province_name,
            &parent.city_name,
            &province,
            &city,
        ) {
            return api_error(StatusCode::BAD_REQUEST, 1001, msg);
        }
        // 盈利属性附属于所属法人:前端推导值必须与父级一致,防客户端漂移
        let expected_p1 = uninorg::inherited_p1(parent.subject_property.as_str(), &parent.p1);
        if p1 != expected_p1 {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "非法人盈利属性必须继承所属法人",
            );
        }
        parent_for_legal_rep_scope = Some(parent);
    }
    let Some(province_code_for_legal_rep) = province_code_by_name(&province) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "unknown province");
    };
    let Some(city_code_for_legal_rep) = city_code_by_name(&province, &city) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "unknown city");
    };
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
    let legal_rep_scope = resolve_legal_representative_scope_for_codes(
        subject_property.as_str(),
        institution_code.as_str(),
        None,
        education_type.as_deref(),
        province_code_for_legal_rep,
        city_code_for_legal_rep,
        parent_for_legal_rep_scope.as_ref(),
    );
    match state.db.legal_representative_citizen_exists_in_scope(
        legal_rep.sfid_number.as_str(),
        &legal_rep_scope,
    ) {
        Ok(true) => {}
        Ok(false) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                legal_rep_scope.legal_rep_error_message(),
            )
        }
        Err(err) => {
            let message = format!("query legal representative failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }
    if let Some(ref name) = sfid_full_name {
        let conflict = match state.db.sfid_full_name_exists(name, None, None, None) {
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
            province_name: province.as_str(),
            city_name: city.as_str(),
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
            sfid_full_name: sfid_full_name.clone(),
            sfid_short_name: sfid_full_name.clone(),
            status: "ACTIVE".to_string(),
            category,
            subject_property: subject_property.clone(),
            p1: p1.clone(),
            province_name: province.clone(),
            city_name: city.clone(),
            town_name: String::new(),
            province_code: extract_province_code(&sfid),
            city_code: extract_city_code(&sfid),
            town_code: String::new(),
            institution_code: institution_code.clone(),
            org_code: None,
            education_type: education_type.clone(),
            private_type: private_rule.map(|rule| rule.private_type.as_code().to_string()),
            partnership_kind: private_rule
                .and_then(|rule| rule.partnership_kind)
                .map(|kind| kind.as_code().to_string()),
            has_legal_personality: private_rule.map(|rule| rule.has_legal_personality),
            parent_sfid_number: parent_sfid_number.clone(),
            legal_rep_name: Some(legal_rep.name.clone()),
            legal_rep_sfid_number: Some(legal_rep.sfid_number.clone()),
            legal_rep_photo_path: Some(legal_rep.photo_path.clone()),
            legal_rep_photo_name: Some(legal_rep.photo_name.clone()),
            legal_rep_photo_mime: Some(legal_rep.photo_mime.clone()),
            legal_rep_photo_size: Some(legal_rep.photo_size),
            created_by: ctx.admin_account.clone(),
            created_at: Utc::now(),
        };
        if let Err(err) = state.db.upsert_institution_row(&inst) {
            let message = format!("write institution failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
        insert_default_accounts_best_effort(&state, &inst, &ctx.admin_account).await;
        crate::core::runtime_ops::append_audit_log(
            &state,
            "INSTITUTION_CREATE",
            &ctx.admin_account,
            Some(sfid.clone()),
            serde_json::json!({
                "sfid_number": sfid.clone(),
                "sfid_full_name": sfid_full_name.clone().unwrap_or_default(),
                "subject_property": subject_property.clone(),
                "institution": institution_code.clone(),
                "education_type": inst.education_type.clone(),
                "category": category_text_for_audit(category),
                "province_name": province.clone(),
                "city_name": city.clone(),
                "private_type": inst.private_type.clone(),
                "partnership_kind": inst.partnership_kind.clone(),
                "parent_sfid_number": parent_sfid_number.clone(),
            }),
        );
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: CreateInstitutionOutput {
                sfid_number: sfid,
                sfid_full_name,
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

fn category_text_for_audit(category: InstitutionCategory) -> &'static str {
    match category {
        InstitutionCategory::PublicSecurity => "PUBLIC_SECURITY",
        InstitutionCategory::GovInstitution => "GOV_INSTITUTION",
        InstitutionCategory::PrivateInstitution => "PRIVATE_INSTITUTION",
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ListInstitutionQuery {
    pub category: Option<String>,
    pub private_type: Option<String>,
    pub province_name: Option<String>,
    pub city_name: Option<String>,
    pub q: Option<String>,
    pub cursor: Option<String>,
    pub page_size: Option<usize>,
}

pub(crate) async fn list_institutions(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<ListInstitutionQuery>,
) -> impl IntoResponse {
    list_institutions_inner(state, headers, query, false).await
}

pub(crate) async fn list_private_institutions(
    state: AppState,
    headers: HeaderMap,
    query: ListInstitutionQuery,
) -> Response {
    list_institutions_inner(state, headers, query, true).await
}

async fn list_institutions_inner(
    state: AppState,
    headers: HeaderMap,
    query: ListInstitutionQuery,
    allow_private: bool,
) -> Response {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);
    // 中文注释:JY 教育机构统一收口教育机构 tab(EDUCATION_INSTITUTION),
    // 私权/公权两路列表同步排除,过滤子句见 InstitutionListFilter。
    let filter = match query.category.as_deref() {
        Some("PRIVATE_INSTITUTION") => {
            if !allow_private {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "私权机构必须使用 /api/v1/private/<type> 查询",
                );
            }
            InstitutionListFilter::Private
        }
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
    let Some(province_name) = scope
        .locked_province_name
        .as_deref()
        .or(query.province_name.as_deref())
    else {
        return api_error(StatusCode::FORBIDDEN, 1003, "province scope required");
    };
    let Some(province_code) = province_code_by_name(province_name) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "unknown province");
    };
    let city_code = match scope.locked_city_name.as_deref().or(query.city_name.as_deref()) {
        Some(city_name) => match city_code_by_name(province_name, city_name) {
            Some(code) => Some(code),
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "unknown city"),
        },
        None => None,
    };
    let page = match state.db.list_institutions_exact(
        filter,
        query.private_type.as_deref(),
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
