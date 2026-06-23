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
    resolve_legal_representative_scope_for_institution, validate_cid_full_name,
    validate_legal_representative_required,
};
use crate::subjects::uninorg;
use crate::*;

pub(crate) async fn check_cid_full_name(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<CheckNameQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }
    let cid_full_name = params.cid_full_name.trim().to_string();
    if cid_full_name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "cid_full_name is required");
    }
    // 中文注释:公法人/公权机构走"同市同全称"查重;私权机构仍全国查重。
    let is_public_legal = params
        .subject_property
        .as_deref()
        .map(str::trim)
        .map_or(false, |value| value == "G");
    let city = params.city_name.as_deref().unwrap_or("").trim().to_string();
    let exists = if is_public_legal {
        if city.is_empty() {
            return api_error(StatusCode::BAD_REQUEST, 1001, "公权机构查重需要 city 参数");
        }
        let cid_full_name = cid_full_name.clone();
        match state.db.with_client(move |conn| {
            let row = conn
                .query_one(
                    "SELECT EXISTS (
	                        SELECT 1 FROM subjects
	                        WHERE kind = 'PUBLIC' AND cid_full_name = $1 AND city_name = $2
	                     )",
                    &[&cid_full_name, &city],
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
        match state
            .db
            .cid_full_name_exists(&cid_full_name, None, None, None)
        {
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
    pub cid_full_name: String,
    /// 主体属性:G 公法人机构走"同市同全称"查重。
    pub subject_property: Option<String>,
    pub city_name: Option<String>,
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
    Path(cid_number): Path<String>,
    Json(input): Json<UpdateInstitutionInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let grant_payload = serde_json::json!({
        "target": cid_number.clone(),
        "cid_number": cid_number.clone(),
        "cid_full_name": input.cid_full_name.clone(),
        "parent_cid_number": input.parent_cid_number.clone(),
        "legal_rep_name": input.legal_rep_name.clone(),
        "legal_rep_cid_number": input.legal_rep_cid_number.clone(),
        "legal_rep_photo_path": input.legal_rep_photo_path.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::InstitutionUpdate,
        cid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let Some((mut existing, _accounts)) = (match state.db.get_institution_with_accounts(&cid_number)
    {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query institution failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    let old_cid_full_name = existing.cid_full_name.clone().unwrap_or_default();
    let old_parent_cid_number = existing.parent_cid_number.clone().unwrap_or_default();
    let scope = get_visible_scope(&ctx);
    if !scope.includes_province(&existing.province_name)
        || !scope.includes_city(&existing.city_name)
    {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }

    if let Some(raw) = input
        .cid_full_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let new_name = match validate_cid_full_name(raw) {
            Ok(v) => v,
            Err(e) => return service_error_to_response(e),
        };
        let conflict = match state
            .db
            .cid_full_name_exists(&new_name, None, None, Some(&cid_number))
        {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query institution name failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        };
        if conflict {
            return api_error(StatusCode::CONFLICT, 1007, "该机构全称已被使用");
        }
        existing.cid_full_name = Some(new_name);
    }
    if input.parent_cid_number.is_some() {
        let raw = input
            .parent_cid_number
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string();
        if !uninorg::requires_parent(existing.institution_code.as_str()) {
            return api_error(StatusCode::BAD_REQUEST, 1001, "该主体类型不接受所属法人");
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
        if !uninorg::can_attach_to_parent(target.institution_code.as_str()) {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                uninorg::parent_subject_requirement_message(),
            );
        }
        // 中文注释:改挂与创建同源校验(subjects/uninorg 单一权威源):
        // 代码一致性(分校⇔学校本部) + 地域规则 + 盈利属性继承,缺一处就有绕过口。
        if let Some(msg) = uninorg::code_consistency_violation(
            existing.institution_code.as_str(),
            target.institution_code.as_str(),
        ) {
            return api_error(StatusCode::BAD_REQUEST, 1001, msg);
        }
        let rule = uninorg::parent_locality_rule(target.institution_code.as_str());
        if let Some(msg) = uninorg::locality_violation(
            rule,
            &target.province_name,
            &target.city_name,
            &existing.province_name,
            &existing.city_name,
        ) {
            return api_error(StatusCode::BAD_REQUEST, 1001, msg);
        }
        let expected_p1 = uninorg::inherited_p1(target.institution_code.as_str(), &target.p1);
        if existing.p1 != expected_p1 {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "非法人盈利属性必须继承所属法人,该机构盈利属性与新所属法人不一致",
            );
        }
        existing.parent_cid_number = Some(raw);
    }
    let legal_rep = match validate_legal_representative_required(
        input.legal_rep_name.as_deref(),
        input.legal_rep_cid_number.as_deref(),
        input.legal_rep_photo_path.as_deref(),
        input.legal_rep_photo_name.as_deref(),
        input.legal_rep_photo_mime.as_deref(),
        input.legal_rep_photo_size,
    ) {
        Ok(v) => v,
        Err(e) => return service_error_to_response(e),
    };
    let parent_for_legal_rep_scope = match existing.parent_cid_number.as_deref() {
        Some(parent_cid) if !parent_cid.trim().is_empty() => {
            match state.db.get_institution_with_accounts(parent_cid) {
                Ok(v) => v.map(|(parent, _)| parent),
                Err(err) => {
                    let message = format!("query parent institution failed: {err}");
                    return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
                }
            }
        }
        _ => None,
    };
    let legal_rep_scope = resolve_legal_representative_scope_for_institution(
        &existing,
        parent_for_legal_rep_scope.as_ref(),
    );
    match state.db.legal_representative_citizen_exists_in_scope(
        legal_rep.cid_number.as_str(),
        &legal_rep_scope,
    ) {
        Ok(true) => {}
        Ok(false) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                legal_rep_scope.legal_rep_error_message(),
            );
        }
        Err(err) => {
            let message = format!("query legal representative failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }
    existing.legal_rep_name = Some(legal_rep.legal_rep_name);
    existing.legal_rep_cid_number = Some(legal_rep.cid_number);
    existing.legal_rep_photo_path = Some(legal_rep.photo_path);
    existing.legal_rep_photo_name = Some(legal_rep.photo_name);
    existing.legal_rep_photo_mime = Some(legal_rep.photo_mime);
    existing.legal_rep_photo_size = Some(legal_rep.photo_size);
    if let Err(err) = state.db.upsert_institution_row(&existing) {
        let message = format!("update institution failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    }
    crate::core::runtime_ops::append_audit_log(
        &state,
        "INSTITUTION_UPDATE",
        &ctx.admin_account,
        Some(cid_number.clone()),
        serde_json::json!({
            "cid_number": cid_number.clone(),
            "old_cid_full_name": old_cid_full_name,
            "new_cid_full_name": existing.cid_full_name.clone().unwrap_or_default(),
            "old_parent_cid_number": old_parent_cid_number,
            "parent_cid_number": existing.parent_cid_number.clone().unwrap_or_default(),
            "legal_rep_name": existing.legal_rep_name.clone().unwrap_or_default(),
            "legal_rep_cid_number": existing.legal_rep_cid_number.clone().unwrap_or_default(),
        }),
    );
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
    /// 非法人的落位省/市,用于地域预过滤(规则同 subjects/uninorg::parent_locality_rule)。
    pub province_name: Option<String>,
    pub city_name: Option<String>,
    /// 限定父级属性:S=仅私法人(私权入口) / G=仅公法人(公权入口);不传=两者(详情页改挂)。
    pub parent_property: Option<String>,
}

/// 所属法人搜索。SQL 预过滤与 `subjects/uninorg` 规则同源(注释互引):
/// - f_institution=JY → 仅本市法人教育机构(手动 JY 行,G/S);
/// - 其它 → 私法人 S(非学校,全国) ∪ 公法人 G(非学校;手动行/市镇级同市、省级同省、国家级不限);
/// - parent_property 进一步收窄到单边。
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
    let province = query
        .province_name
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    let city = query
        .city_name
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if province.is_empty() || city.is_empty() {
        // 地域预过滤依赖非法人落位省市;缺参直接拒绝,不退化成全国搜索
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "province_name/city_name are required",
        );
    }
    let parent_property = match query.parent_property.as_deref().map(str::trim) {
        None | Some("") => None,
        Some("S") => Some("S".to_string()),
        Some("G") => Some("G".to_string()),
        Some(_) => return api_error(StatusCode::BAD_REQUEST, 1001, "parent_property 仅 S/G"),
    };
    let result = state.db.with_client(move |conn| {
        // 中文注释:候选父级按非法人规则预过滤(与 uninorg::parent_locality_rule 同源)。
        // UNIN 是通用从属,可挂任意法人父级(私法人 S / 公法人 G)。地域规则:学校/大学父级 → 同市,
        // 非学校私法人 → 全国,非学校公法人 → 按机构码行政层级。$4 = parent_property 过滤(S/G)。
        // 主体属性已删除:私法人由机构码集合判定,公法人由 category 判定,层级由 institution_code 集合判定。
        let candidate_clause = "(
                  (s.institution_code IN ('GUN', 'SUN', 'GSCH', 'SFSC')
                   AND s.province_name = $2 AND s.city_name = $3
                   AND ((s.institution_code IN ('SFLP', 'SFGQ', 'SFGF', 'SFGY', 'SFAS', 'SUN', 'SFSC')
                         AND $4::text IS DISTINCT FROM 'G')
                        OR (s.category IN ('GOV_INSTITUTION', 'PUBLIC_SECURITY')
                            AND $4::text IS DISTINCT FROM 'S')))
                  OR (s.institution_code IN ('SFLP', 'SFGQ', 'SFGF', 'SFGY', 'SFAS', 'SUN', 'SFSC')
                      AND s.institution_code NOT IN ('GUN', 'SUN', 'GSCH', 'SFSC')
                      AND $4::text IS DISTINCT FROM 'G')
                  OR (s.category IN ('GOV_INSTITUTION', 'PUBLIC_SECURITY')
                      AND s.institution_code NOT IN ('GUN', 'SUN', 'GSCH', 'SFSC')
                      AND $4::text IS DISTINCT FROM 'S'
                      AND (
                           s.institution_code IN (
                               'PRS','FSC','FIB','FSS','FPR','FRG','MFA','MDF','MHS','MCW','MHU',
                               'MAG','MCM','MFT','MEN','MTR','NLG','NJD','NSP','FAC','FAU','FIV',
                               'NED','NRC','NSN','NRP')
                           OR (s.institution_code IN (
                                   'PGV','PLG','PJD','PSP','PRC','PRB','PDF','PHS','PCW','PHU','PAG',
                                   'PCM','PFT','PEN','PTR','PSN','PRP')
                               AND s.province_name = $2)
                           OR (s.institution_code IN (
                                   'CGOV','CLEG','CSUP','CJUD','CEDU','CSLF','CDEF','CHSC','CCWF',
                                   'CHUD','CAGR','CCOM','CFIN','CENR','CTRN','CREG','CPOL',
                                   'TGOV','TCWF','THUD','TAGR','TFIN','TDEF','THSC','TCOM','TENR','TTRN')
                               AND s.province_name = $2 AND s.city_name = $3)
                      ))
             )";
        let sql = format!(
            "SELECT s.cid_number, s.cid_full_name, s.private_type, s.partnership_kind, s.category,
                    s.p1, s.province_name, s.city_name, COALESCE(s.town_name, '')
             FROM subjects s
             WHERE s.kind IN ('PUBLIC', 'PRIVATE')
               AND s.status = 'ACTIVE'
               AND COALESCE(s.cid_full_name, '') <> ''
               AND {candidate_clause}
               AND (lower(s.cid_number) LIKE '%' || $1 || '%'
                    OR lower(COALESCE(s.cid_full_name, '')) LIKE '%' || $1 || '%'
                    OR lower(COALESCE(s.cid_short_name, '')) LIKE '%' || $1 || '%')
             ORDER BY COALESCE(s.cid_short_name, '') ASC, COALESCE(s.cid_full_name, '') ASC, s.cid_number ASC
             LIMIT 20"
        );
        let rows = conn
            .query(sql.as_str(), &[&q, &province, &city, &parent_property])
            .map_err(|e| format!("query parent institutions failed: {e}"))?;
        let mut output = Vec::with_capacity(rows.len());
        for row in rows {
            let category_text: String = row.get(4);
            let Some(category) = crate::institution_category_from_text(category_text.as_str())
            else {
                continue;
            };
            output.push(ParentInstitutionRow {
                cid_number: row.get(0),
                cid_full_name: row.get(1),
                private_type: row.get(2),
                partnership_kind: row.get(3),
                category,
                p1: row.get(5),
                province_name: row.get(6),
                city_name: row.get(7),
                town_name: row.get(8),
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
    Path(cid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some((inst, accounts)) = (match state.db.get_institution_with_accounts(&cid_number) {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query institution failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    let scope = get_visible_scope(&ctx);
    if !scope.includes_province(&inst.province_name) || !scope.includes_city(&inst.city_name) {
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
/// 本省联邦注册局机构管理员列表,因此这里**不做 scope 校验**(与 get_institution 的唯一区别)。
/// 仍要求已登录管理员;只返回 FEDERAL_REGISTRY 这一个机构。
pub(crate) async fn get_federal_registry(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }
    // 联邦注册局 cid 来自创世常量(china_zf),按 cid_number 直接定位,绕过 scope。
    let Some(cid_number) = crate::gov::service::federal_registry_cid_number() else {
        return api_error(
            StatusCode::NOT_FOUND,
            1004,
            "federal registry not configured",
        );
    };
    let Some((inst, accounts)) = (match state.db.get_institution_with_accounts(cid_number) {
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
