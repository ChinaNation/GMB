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

use crate::auth::actions::require_admin_security_grant;
use crate::auth::login::require_admin_any;
use crate::auth::operation_auth::AdminActionType;
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
    // 机构发起身份只从本节点当前绑定的注册局 CID 读取，不再从环境变量建第二真源。
    let actor_cid_number =
        match crate::domains::citizens::chain_identity::active_registry_cid_number(&state) {
            Ok(value) => value,
            Err(resp) => return resp,
        };
    let grant_payload = serde_json::json!({
        "p1": input.p1.clone(),
        "province_name": input.province_name.clone(),
        "city_name": input.city_name.clone(),
        "town_name": input.town_name.clone(),
        "institution": input.institution.clone(),
        "education_type": input.education_type.clone(),
        "cid_full_name": input.cid_full_name.clone(),
        "cid_short_name": input.cid_short_name.clone(),
        "parent_cid_number": input.parent_cid_number.clone(),
        "private_type": input.private_type.clone(),
        "partnership_kind": input.partnership_kind.clone(),
        "admins": input.admins.clone(),
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
    // 私权入口一旦传入 private_type,机构码/P1/法人资格由后端规则锁定,不信任前端旧字段。
    let institution_code = private_rule
        .map(|rule| rule.institution_code.to_string())
        .unwrap_or_else(|| input.institution.trim().to_string());
    // 机构类别一律由机构码派生。
    let institution = code::institution_code_from_str(&institution_code);
    let p1 = private_rule
        .map(|rule| {
            rule.p1
                .map(str::to_string)
                .unwrap_or_else(|| input.p1.as_deref().unwrap_or("").trim().to_string())
        })
        .unwrap_or_else(|| input.p1.as_deref().unwrap_or("").trim().to_string());
    if institution.is_none() || institution_code.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "institution is required");
    }
    // 私权机构 = 私法人或非法人(由机构码判定)。
    let is_private = institution
        .map(|c| code::is_private_legal_code(&c) || code::is_unincorporated_code(&c))
        .unwrap_or(false);
    // 教育机构(公私大学/学校)走通用路径、免 private_type;基础教育学校(GSCH/SFSC,初/小/中)
    // 需要 education_type 级别,大学(GUN/SUN)不需要。
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
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "必须选择教育级别(初学/小学/中学)",
            );
        };
        if !is_education_school_type(value.as_str()) {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "教育级别仅允许初学/小学/中学",
            );
        }
        Some(value)
    } else {
        if education_type.is_some() {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "education_type 仅基础教育学校(初学/小学/中学)使用",
            );
        }
        None
    };
    if private_rule.is_none()
        && is_private
        && !is_education_institution
        && institution_code != "UNIN"
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "私权机构必须提交 private_type",
        );
    }
    let cid_full_name = match input.cid_full_name.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => match validate_cid_full_name(raw) {
            Ok(v) => Some(v),
            Err(e) => return service_error_to_response(e),
        },
        _ => return api_error(StatusCode::BAD_REQUEST, 1001, "学校全称/机构全称不能为空"),
    };
    let cid_short_name = match input.cid_short_name.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => match validate_cid_short_name(raw) {
            Ok(v) => v,
            Err(e) => return service_error_to_response(e),
        },
        _ => return api_error(StatusCode::BAD_REQUEST, 1001, "学校简称/机构简称不能为空"),
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
    // 机构创建权限统一由省/市 scope 收口:
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
    let category = match derive_category(&institution_code, cid_full_name.as_deref().unwrap_or(""))
    {
        Some(v) => v,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "institution is not a valid institution",
            );
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
            return api_error(StatusCode::BAD_REQUEST, 1001, "town is required");
        };
        let Some(code) = town_code_by_name(&province, &city, raw_town) else {
            return api_error(StatusCode::BAD_REQUEST, 1001, "unknown town");
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
            return api_error(StatusCode::BAD_REQUEST, 1001, "非镇级机构不得提交镇");
        }
        (String::new(), String::new())
    };
    // 手动公权机构按管理员注册局角色 + 机构层级开放:
    // 联邦注册局管理员 → 国家/省/部级(3 字符码);市注册局管理员 → 市/镇级(4 字符码)。
    // 公权教育机构(大学/学校)走教育流程,不受此限。
    if matches!(category, InstitutionCategory::GovInstitution) && !is_education_institution {
        let needs_federal = institution
            .map(|c| code::is_three_char_code(&c))
            .unwrap_or(false);
        let is_federal_admin = scope.locked_city_name.is_none();
        if needs_federal && !is_federal_admin {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "国家/省/部级公权机构由联邦注册局管理员创建",
            );
        }
        if institution_code == "CREG" && !is_federal_admin {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "市注册局只能由联邦注册局管理员创建",
            );
        }
    }
    let normalized_admins = match validate_initial_admins(&input.admins) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if is_private && p1 != "0" && p1 != "1" {
        return api_error(StatusCode::BAD_REQUEST, 1001, "P1 非法(仅 0/1)");
    }
    // ── 非法人挂靠规则:个体经营/无限合伙是独立非法人,教育分校等从属非法人才需要所属法人 ──
    let parent_cid_number = match input
        .parent_cid_number
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(raw) => {
            if !unincorporated_org::requires_parent(institution_code.as_str()) {
                return api_error(StatusCode::BAD_REQUEST, 1001, "该主体类型不接受所属法人");
            }
            Some(raw.to_string())
        }
        None => {
            if unincorporated_org::requires_parent(institution_code.as_str()) {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "非法人必须选择所属法人(私法人或公法人)",
                );
            }
            None
        }
    };
    if let Some(ref parent_cid) = parent_cid_number {
        let Some((parent, _)) = (match state.db.get_institution_with_accounts(parent_cid) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query parent institution failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        }) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "所属法人机构不存在");
        };
        if !unincorporated_org::can_attach_to_parent(parent.institution_code.as_str()) {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                unincorporated_org::parent_subject_requirement_message(),
            );
        }
        if let Some(msg) = unincorporated_org::code_consistency_violation(
            institution_code.as_str(),
            parent.institution_code.as_str(),
        ) {
            return api_error(StatusCode::BAD_REQUEST, 1001, msg);
        }
        let rule = unincorporated_org::parent_locality_rule(parent.institution_code.as_str());
        if let Some(msg) = unincorporated_org::locality_violation(
            rule,
            &parent.province_name,
            &parent.city_name,
            &province,
            &city,
        ) {
            return api_error(StatusCode::BAD_REQUEST, 1001, msg);
        }
        // 盈利属性附属于所属法人:前端推导值必须与父级一致,防客户端漂移
        let expected_p1 =
            unincorporated_org::inherited_p1(parent.institution_code.as_str(), &parent.p1);
        if p1 != expected_p1 {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "非法人盈利属性必须继承所属法人",
            );
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
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        };
        if conflict {
            return api_error(StatusCode::CONFLICT, 1007, "该机构全称已被使用");
        }
    }
    // 随机 UUID 种子 + 1000 次撞号重试 + 格式校验收敛在 cid::seed。
    // 正式机构投影和链确认前待登记区必须同时参与撞号保护；任一查询失败均按已占用处理，
    // 禁止数据库故障时退化为“CID 不存在”。
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
            let pending_exists = state
                .db
                .pending_institution_registration_exists(candidate)?;
            Ok::<bool, String>(formal_exists || pending_exists)
        },
    ) {
        Ok(v) => v,
        Err(crate::cid::SeedCidError::Generate(msg)) => {
            return api_error(StatusCode::BAD_REQUEST, 1001, msg);
        }
        Err(crate::cid::SeedCidError::Validate(msg)) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg);
        }
        Err(crate::cid::SeedCidError::Exhausted) => {
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "institution cid_number collision retry exhausted",
            );
        }
        Err(crate::cid::SeedCidError::Exists(err)) => {
            let message = format!("query institution cid_number failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    {
        let inst = Institution {
            cid_number: cid.clone(),
            cid_full_name: cid_full_name.clone(),
            cid_short_name: Some(cid_short_name.clone()),
            category,
            p1: p1.clone(),
            province_name: province.clone(),
            city_name: city.clone(),
            town_name: town_name.clone(),
            province_code: extract_province_code(&cid),
            city_code: extract_city_code(&cid),
            town_code: town_code.clone(),
            institution_code: institution_code.clone(),
            education_type: education_type.clone(),
            private_type: private_rule.map(|rule| rule.private_type.as_code().to_string()),
            partnership_kind: private_rule
                .and_then(|rule| rule.partnership_kind)
                .map(|kind| kind.as_code().to_string()),
            has_legal_personality: private_rule.map(|rule| rule.has_legal_personality),
            parent_cid_number: parent_cid_number.clone(),
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
        let institution_create_sign_request = match build_institution_create_sign_request(
            &state,
            actor_cid_number.as_str(),
            &inst,
            normalized_admins.as_slice(),
            ctx.admin_account.as_str(),
        ) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        // 链确认前只保存待确认草稿，禁止提前写入 subjects/accounts/admins 正式投影。
        let institution_payload = match serde_json::to_value(&inst) {
            Ok(value) => value,
            Err(err) => {
                let message = format!("encode pending institution failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        };
        let admins_payload = match serde_json::to_value(&normalized_admins) {
            Ok(value) => value,
            Err(err) => {
                let message = format!("encode pending institution admins failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        };
        let pending_cid = cid.clone();
        let pending_actor_cid = actor_cid_number.clone();
        let pending_created_by = ctx.admin_account.clone();
        if let Err(err) = state.db.with_client(move |conn| {
            conn.execute(
                "INSERT INTO pending_institution_registrations (
                    cid_number, institution_payload, admins_payload,
                    actor_cid_number, created_by, created_at
                 ) VALUES ($1, $2, $3, $4, $5, now())
                 ON CONFLICT (cid_number) DO UPDATE SET
                    institution_payload = EXCLUDED.institution_payload,
                    admins_payload = EXCLUDED.admins_payload,
                    actor_cid_number = EXCLUDED.actor_cid_number,
                    created_by = EXCLUDED.created_by,
                    created_at = now()",
                &[
                    &pending_cid,
                    &institution_payload,
                    &admins_payload,
                    &pending_actor_cid,
                    &pending_created_by,
                ],
            )
            .map_err(|e| format!("write pending institution registration failed: {e}"))?;
            Ok(())
        }) {
            let message = format!("write pending institution registration failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
        crate::core::runtime_ops::append_audit_log(
            &state,
            "INSTITUTION_CREATE",
            &ctx.admin_account,
            Some(cid.clone()),
            serde_json::json!({
                "cid_number": cid.clone(),
                "cid_full_name": cid_full_name.clone().unwrap_or_default(),
                "institution": institution_code.clone(),
                "education_type": inst.education_type.clone(),
                "category": category_text_for_audit(category),
                "province_name": province.clone(),
                "city_name": city.clone(),
                "town_name": town_name.clone(),
                "private_type": inst.private_type.clone(),
                "partnership_kind": inst.partnership_kind.clone(),
                "parent_cid_number": parent_cid_number.clone(),
                "admins_len": normalized_admins.len(),
            }),
        );
        Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: CreateInstitutionOutput {
                cid_number: cid,
                cid_full_name,
                category,
                institution_create_sign_request,
            },
        })
        .into_response()
    }
}

fn validate_initial_admins(
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
        normalized.push(CreateInstitutionAdminInput { admin_account });
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

fn build_institution_create_sign_request(
    state: &AppState,
    actor_cid_number: &str,
    inst: &Institution,
    admins: &[CreateInstitutionAdminInput],
    actor_pubkey: &str,
) -> Result<String, Response> {
    let action_id = format!("cid-institution-create-{}", Uuid::new_v4());
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::seconds(120);
    let chain =
        crate::institution::subjects::registration_call::build_create_institution_call_data(
            state,
            actor_cid_number,
            inst,
            admins,
        )
        .map_err(|err| {
            let message = format!("build institution chain call failed: {err}");
            api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str())
        })?;
    crate::core::qr::build_sign_request_bytes(
        action_id.as_str(),
        issued_at.timestamp(),
        expires_at.timestamp(),
        actor_pubkey,
        &chain.call_data,
        chain.action,
    )
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
