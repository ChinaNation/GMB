//! 机构注册 HTTP handler。
//!
//! 本文件只保留跨公权/教育/私权共用的主体注册内核。私权机构入口必须由
//! `private/<type>/` 六类模块传入固定类型规则,不得再由一个 private 总 handler 吞掉。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use std::collections::HashSet;
use uuid::Uuid;

use crate::auth::actions::actor_cid_number_for_context;
use crate::auth::login::require_admin_any;
use crate::auth::login::AdminAuthContext;
use crate::cid::china::{city_code_by_name, province_code_by_name, town_code_by_name};
use crate::cid::code;
use crate::cid::InstitutionCategory;
use crate::crypto::pubkey::normalize_admin_account;
use crate::domains::private::common::resolve_private_type_rule;
use crate::institution::subjects::http::{
    extract_city_code, extract_province_code, service_error_to_response, MAX_CITY_CHARS,
    MAX_PROVINCE_CHARS,
};
use crate::institution::subjects::model::{
    is_education_school_type, CreateInstitutionAdminInput, CreateInstitutionInput,
    CreateInstitutionOutput, Institution, InstitutionListFilter, InstitutionListRow,
};
use crate::institution::subjects::service::{
    derive_category, validate_cid_full_name, validate_cid_short_name,
};
use crate::institution::subjects::unincorporated_org;
use crate::scope::get_visible_scope;
use crate::*;

pub(crate) struct PreparedInstitutionCreate {
    pub(crate) inst: Institution,
    pub(crate) admins: Vec<CreateInstitutionAdminInput>,
}

pub(crate) const PURPOSE_INSTITUTION_CREATE: &str = "institution-create";

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
    let actor_cid_number = match actor_cid_number_for_context(&state.db, &ctx) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let prepared_create =
        match prepare_institution_create_from_input(&state, &ctx, &input, allow_private) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    let inst = prepared_create.inst;
    let normalized_admins = prepared_create.admins;
    if !allow_private
        && inst
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
    let chain =
        match crate::institution::subjects::registration_call::build_create_institution_call_data(
            &state,
            actor_cid_number.as_str(),
            &inst,
            normalized_admins.as_slice(),
        ) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("build institution chain call failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        };
    let prepared_signing = match crate::core::chain_submit::prepare_signing(
        &chain.call_data,
        ctx.admin_account.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "prepare institution create signing failed");
            return api_error(
                StatusCode::BAD_GATEWAY,
                1004,
                "链签名载荷准备失败(链不可用)",
            );
        }
    };
    let issued_at = Utc::now();
    let expires_at =
        issued_at + Duration::seconds(crate::domains::citizens::occupy::SESSION_TTL_SECS);
    let request_id = format!("institution-create-{}", Uuid::new_v4());
    let institution_create_sign_request = match crate::core::qr::build_sign_request_bytes(
        request_id.as_str(),
        issued_at.timestamp(),
        expires_at.timestamp(),
        ctx.admin_account.as_str(),
        &prepared_signing.payload,
        chain.action,
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let session = crate::domains::citizens::occupy::ChainSignSession {
        request_id: request_id.clone(),
        purpose: PURPOSE_INSTITUTION_CREATE.to_string(),
        actor_pubkey: ctx.admin_account.clone(),
        call_data: chain.call_data,
        nonce: prepared_signing.nonce,
        signing_hash: prepared_signing.signing_hash_hex,
        context: serde_json::json!({
            "cid_number": inst.cid_number.clone(),
            "actor_cid_number": actor_cid_number,
            // 这里是签名会话上下文,不是机构业务草稿。只有 submit 链上成功后,
            // 统一 submit 入口才允许把这些字段写入正式查询投影。
            "institution": inst.clone(),
            "admins": normalized_admins.clone(),
        }),
        expires_at,
        consumed_at: None,
    };
    if let Err(err) = state.db.insert_chain_sign_session(&session) {
        tracing::error!(error = %err, "insert institution create chain sign session failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "机构创建链签会话保存失败",
        );
    };
    crate::core::runtime_ops::append_audit_log(
        &state,
        "INSTITUTION_CREATE",
        &ctx.admin_account,
        Some(inst.cid_number.clone()),
        serde_json::json!({
            "cid_number": inst.cid_number.clone(),
            "cid_full_name": inst.cid_full_name.clone().unwrap_or_default(),
            "institution": inst.institution_code.clone(),
            "education_type": inst.education_type.clone(),
            "category": category_text_for_audit(inst.category),
            "province_name": inst.province_name.clone(),
            "city_name": inst.city_name.clone(),
            "town_name": inst.town_name.clone(),
            "private_type": inst.private_type.clone(),
            "partnership_kind": inst.partnership_kind.clone(),
            "parent_cid_number": inst.parent_cid_number.clone(),
            "admins_len": normalized_admins.len(),
        }),
    );
    return Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CreateInstitutionOutput {
            request_id,
            cid_number: inst.cid_number,
            cid_full_name: inst.cid_full_name,
            category: inst.category,
            institution_create_sign_request,
        },
    })
    .into_response();
}

fn validate_initial_admins(
    state: &AppState,
    admins: &[CreateInstitutionAdminInput],
) -> Result<Vec<CreateInstitutionAdminInput>, Response> {
    let mut unique_accounts = HashSet::new();
    let mut normalized = Vec::with_capacity(admins.len());
    for admin in admins {
        let Some(admin_account) = normalize_admin_account(admin.admin_account.as_str()) else {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "admin_account format invalid",
            ));
        };
        if !unique_accounts.insert(admin_account.clone()) {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "管理员账户不能重复",
            ));
        }
        let citizen = state
            .db
            .find_citizen_by_wallet(admin.admin_account.as_str())
            .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, &err))?;
        let family_name = normalize_initial_person_name(admin.family_name.as_deref())
            .or_else(|| {
                citizen.as_ref().and_then(|record| {
                    normalize_initial_person_name(Some(&record.citizen_family_name))
                })
            })
            .unwrap_or_else(|| "管理".to_string());
        let given_name = normalize_initial_person_name(admin.given_name.as_deref())
            .or_else(|| {
                citizen.as_ref().and_then(|record| {
                    normalize_initial_person_name(Some(&record.citizen_given_name))
                })
            })
            .unwrap_or_else(|| "员".to_string());
        normalized.push(CreateInstitutionAdminInput {
            admin_account,
            family_name: Some(family_name),
            given_name: Some(given_name),
        });
    }
    if unique_accounts.len() < 2 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "机构初始管理员至少需要 2 人",
        ));
    }
    Ok(normalized)
}

fn normalize_initial_person_name(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

pub(crate) fn prepare_institution_create_from_input(
    state: &AppState,
    ctx: &AdminAuthContext,
    input: &CreateInstitutionInput,
    allow_private: bool,
) -> Result<PreparedInstitutionCreate, Response> {
    if !allow_private
        && input
            .private_type
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .is_some()
    {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "私权机构必须使用 /api/v1/private/<type> 创建",
        ));
    }
    let scope = get_visible_scope(ctx);
    let private_rule = match input
        .private_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(private_type) => {
            match resolve_private_type_rule(private_type, input.partnership_kind.as_deref()) {
                Ok(v) => Some(v),
                Err(msg) => return Err(api_error(StatusCode::BAD_REQUEST, 1001, msg)),
            }
        }
        None => None,
    };
    let institution_code = private_rule
        .map(|rule| rule.institution_code.to_string())
        .unwrap_or_else(|| input.institution.trim().to_string());
    let institution = code::institution_code_from_str(&institution_code);
    let p1 = private_rule
        .map(|rule| {
            rule.p1
                .map(str::to_string)
                .unwrap_or_else(|| input.p1.as_deref().unwrap_or("").trim().to_string())
        })
        .unwrap_or_else(|| input.p1.as_deref().unwrap_or("").trim().to_string());
    if institution.is_none() || institution_code.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "institution is required",
        ));
    }
    let is_private = institution
        .map(|c| code::is_private_legal_code(&c) || code::is_unincorporated_code(&c))
        .unwrap_or(false);
    let is_education_institution = institution
        .map(|c| code::is_education_institution_code(&c))
        .unwrap_or(false);
    let requires_education_level = institution
        .map(|c| code::requires_education_level(&c))
        .unwrap_or(false);
    let education_type = input
        .education_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let education_type = if requires_education_level {
        let Some(value) = education_type else {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "必须选择教育级别(初学/小学/中学)",
            ));
        };
        if !is_education_school_type(value.as_str()) {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "教育级别仅允许初学/小学/中学",
            ));
        }
        Some(value)
    } else {
        if education_type.is_some() {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "education_type 仅基础教育学校(初学/小学/中学)使用",
            ));
        }
        None
    };
    if private_rule.is_none()
        && is_private
        && !is_education_institution
        && institution_code != "UNIN"
    {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "私权机构必须提交 private_type",
        ));
    }
    let cid_full_name = match input.cid_full_name.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => match validate_cid_full_name(raw) {
            Ok(v) => Some(v),
            Err(e) => return Err(service_error_to_response(e)),
        },
        _ => {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "学校全称/机构全称不能为空",
            ))
        }
    };
    let cid_short_name = match input.cid_short_name.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => match validate_cid_short_name(raw) {
            Ok(v) => v,
            Err(e) => return Err(service_error_to_response(e)),
        },
        _ => {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "学校简称/机构简称不能为空",
            ))
        }
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
                return Err(api_error(
                    StatusCode::FORBIDDEN,
                    1003,
                    "province out of current admin scope",
                ));
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
            None => {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "province is required",
                ))
            }
        },
    };
    if province.chars().count() > MAX_PROVINCE_CHARS {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "province too long",
        ));
    }
    let mut city = input.city_name.trim().to_string();
    if let Some(locked_city_name) = scope.locked_city_name.clone() {
        if !city.is_empty() && city != locked_city_name {
            return Err(api_error(
                StatusCode::FORBIDDEN,
                1003,
                "city out of current admin scope",
            ));
        }
        city = locked_city_name;
    }
    if city.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, "city is required"));
    }
    if city.chars().count() > MAX_CITY_CHARS {
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, "city too long"));
    }
    let category = match derive_category(&institution_code, cid_full_name.as_deref().unwrap_or(""))
    {
        Some(v) => v,
        None => {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "institution is not a valid institution",
            ));
        }
    };
    let is_town_public_institution = matches!(
        institution,
        Some(code) if code::admin_level(&code) == Some(code::AdminLevel::Town)
    );
    let (town_name, town_code) = if is_town_public_institution {
        let Some(raw_town) = input
            .town_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            return Err(api_error(StatusCode::BAD_REQUEST, 1001, "town is required"));
        };
        let Some(code) = town_code_by_name(&province, &city, raw_town) else {
            return Err(api_error(StatusCode::BAD_REQUEST, 1001, "unknown town"));
        };
        (raw_town.to_string(), code.to_string())
    } else {
        if input
            .town_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .is_some()
        {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "非镇级机构不得提交镇",
            ));
        }
        (String::new(), String::new())
    };
    if matches!(category, InstitutionCategory::GovInstitution) && !is_education_institution {
        let needs_federal = institution
            .map(|c| code::is_three_char_code(&c))
            .unwrap_or(false);
        let is_federal_admin = scope.locked_city_name.is_none();
        if needs_federal && !is_federal_admin {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "国家/省/部级公权机构由联邦注册局管理员创建",
            ));
        }
        if institution_code == "CREG" && !is_federal_admin {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "市注册局只能由联邦注册局管理员创建",
            ));
        }
    }
    let normalized_admins = validate_initial_admins(state, &input.admins)?;
    if is_private && p1 != "0" && p1 != "1" {
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, "P1 非法(仅 0/1)"));
    }
    let parent_cid_number = match input
        .parent_cid_number
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(raw) => {
            if !unincorporated_org::requires_parent(institution_code.as_str()) {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "该主体类型不接受所属法人",
                ));
            }
            Some(raw.to_string())
        }
        None => {
            if unincorporated_org::requires_parent(institution_code.as_str()) {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "非法人必须选择所属法人(私法人或公法人)",
                ));
            }
            None
        }
    };
    if let Some(ref parent_cid) = parent_cid_number {
        let Some((parent, _)) = (match state.db.get_institution_with_accounts(parent_cid) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query parent institution failed: {err}");
                return Err(api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    message.as_str(),
                ));
            }
        }) else {
            return Err(api_error(StatusCode::NOT_FOUND, 1004, "所属法人机构不存在"));
        };
        if !unincorporated_org::can_attach_to_parent(parent.institution_code.as_str()) {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                unincorporated_org::parent_subject_requirement_message(),
            ));
        }
        if let Some(msg) = unincorporated_org::code_consistency_violation(
            institution_code.as_str(),
            parent.institution_code.as_str(),
        ) {
            return Err(api_error(StatusCode::BAD_REQUEST, 1001, msg));
        }
        let rule = unincorporated_org::parent_locality_rule(parent.institution_code.as_str());
        if let Some(msg) = unincorporated_org::locality_violation(
            rule,
            &parent.province_name,
            &parent.city_name,
            &province,
            &city,
        ) {
            return Err(api_error(StatusCode::BAD_REQUEST, 1001, msg));
        }
        let expected_p1 =
            unincorporated_org::inherited_p1(parent.institution_code.as_str(), &parent.p1);
        if p1 != expected_p1 {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "非法人盈利属性必须继承所属法人",
            ));
        }
    }
    if let Some(ref cid_full_name_value) = cid_full_name {
        let conflict = match state
            .db
            .cid_full_name_exists(cid_full_name_value, None, None, None)
        {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query cid_full_name failed: {err}");
                return Err(api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    message.as_str(),
                ));
            }
        };
        if conflict {
            return Err(api_error(StatusCode::CONFLICT, 1007, "该机构全称已被使用"));
        }
    }
    let cid = match crate::cid::dynamic_institution_cid(
        province.as_str(),
        city.as_str(),
        institution_code.as_str(),
        p1.as_str(),
        |candidate| {
            let formal_exists = state
                .db
                .get_institution_with_accounts(candidate)
                .map(|value| value.is_some())?;
            Ok::<bool, String>(formal_exists)
        },
    ) {
        Ok(v) => v,
        Err(crate::cid::SeedCidError::Generate(msg)) => {
            return Err(api_error(StatusCode::BAD_REQUEST, 1001, msg));
        }
        Err(crate::cid::SeedCidError::Validate(msg)) => {
            return Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg));
        }
        Err(crate::cid::SeedCidError::Exhausted) => {
            return Err(api_error(
                StatusCode::CONFLICT,
                1005,
                "institution cid_number collision retry exhausted",
            ));
        }
        Err(crate::cid::SeedCidError::Exists(err)) => {
            let message = format!("query institution cid_number failed: {err}");
            return Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                message.as_str(),
            ));
        }
    };
    let inst = Institution {
        cid_number: cid,
        cid_full_name,
        cid_short_name: Some(cid_short_name.clone()),
        category,
        p1: p1.clone(),
        province_name: province.clone(),
        city_name: city.clone(),
        town_name,
        province_code: String::new(),
        city_code: String::new(),
        town_code: town_code.clone(),
        institution_code: institution_code.clone(),
        education_type,
        private_type: private_rule.map(|rule| rule.private_type.as_code().to_string()),
        partnership_kind: private_rule
            .and_then(|rule| rule.partnership_kind)
            .map(|kind| kind.as_code().to_string()),
        has_legal_personality: private_rule.map(|rule| rule.has_legal_personality),
        parent_cid_number,
        legal_representative_name: None,
        legal_representative_cid_number: None,
        legal_representative_account: None,
        legal_representative_photo_path: None,
        legal_representative_photo_name: None,
        legal_representative_photo_mime: None,
        legal_representative_photo_size: None,
        created_by: ctx.admin_account.clone(),
        created_at: Utc::now(),
    };
    let mut inst = inst;
    inst.province_code = extract_province_code(&inst.cid_number);
    inst.city_code = extract_city_code(&inst.cid_number);
    Ok(PreparedInstitutionCreate {
        inst,
        admins: normalized_admins,
    })
}

fn category_text_for_audit(category: InstitutionCategory) -> &'static str {
    match category {
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
    // 本路列表只服务运行期创建机构(私权/教育/手动公权)。镇级公权机构会携带
    // town_code,非镇级机构 town_code 为空;本列表仍按省/市收口,详情展示再读 town_code。
    // JY 教育机构统一收口教育机构 tab(EDUCATION_FORM),私权/公权两路列表同步排除,
    // 过滤子句见 InstitutionListFilter。
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
        Some("EDUCATION_FORM") => InstitutionListFilter::Education,
        _ => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "institution category is required",
            );
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
    let city_code = match scope
        .locked_city_name
        .as_deref()
        .or(query.city_name.as_deref())
    {
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
            return api_error(StatusCode::BAD_REQUEST, 1001, "invalid page cursor");
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
