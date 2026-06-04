//! 私权机构 HTTP handler
//!
//! 中文注释:本模块只承载注册局手动新增的私权机构和普通机构精确查询;
//! 确定性公权机构归 gov,账户归 accounts,资料库归 docs,主体详情归 subjects::admin。
//!
//! ## 当前路由表(admin 端,login 中间件)
//!
//! - POST   /api/v1/institution/create                    → create_institution
//! - GET    /api/v1/institution/list                      → list_institutions

#![allow(dead_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::admins::actions::require_admin_security_grant;
use crate::admins::operation_auth::AdminActionType;
use crate::china::{city_code_by_name, province_code_by_name};
use crate::login::require_admin_any;
use crate::models::ApiResponse;
use crate::scope::get_visible_scope;
use crate::sfid_number::{
    generate_sfid_code, validate_sfid_number_format, GenerateSfidInput, InstitutionCategory,
};
use crate::subjects::http::{
    append_inst_audit_log_best_effort, extract_city_code, extract_province_code,
    insert_default_accounts_best_effort, service_error_to_response, MAX_CITY_CHARS,
    MAX_PROVINCE_CHARS,
};
use crate::subjects::model::{
    CreateInstitutionInput, CreateInstitutionOutput, InstitutionListRow, MultisigInstitution,
};
use crate::subjects::service::{
    derive_category, institution_name_exists, institution_name_exists_in_city,
    validate_institution_name,
};
use crate::subjects::InstitutionChainStatus;
use crate::*;

// ─── 0. 机构名称查重(私权=全国唯一,公权=同城唯一) ──────────────

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
        "a3": input.a3.clone(),
        "p1": input.p1.clone(),
        "province": input.province.clone(),
        "city": input.city.clone(),
        "institution": input.institution.clone(),
        "institution_name": input.institution_name.clone(),
        "source": input.source.clone(),
        "institution_level": input.institution_level.clone(),
        "sub_type": input.sub_type.clone(),
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

    let a3 = input.a3.trim().to_string();
    let institution_code = input.institution.trim().to_string();
    let p1_input = input.p1.as_deref().unwrap_or("").trim().to_string();
    if a3.is_empty() || institution_code.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "a3 and institution are required",
        );
    }

    // 是否私权两步式流程(此时允许 institution_name 缺失)
    let is_private = matches!(a3.as_str(), "SFR" | "FFR");
    let is_education_school = institution_code == "JY";
    let allow_missing_name = is_private && !is_education_school;

    // 中文注释:手动创建不得伪造自动目录字段。学校只是 JY 类型机构,
    // 不是一个需要绑定父学校 SFID 的独立学校内部组织对象。
    if input.source.is_some() || input.institution_level.is_some() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "source/institution_level 仅系统自动目录可写",
        );
    }
    if is_education_school && ctx.admin_city.is_none() {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "教育委员会类型学校只能由市级管理员注册",
        );
    }

    // ── 机构名称:私权两步式允许 None;公权必填 ──
    let institution_name_opt: Option<String> =
        match input.institution_name.as_deref().map(str::trim) {
            Some(raw) if !raw.is_empty() => match validate_institution_name(raw) {
                Ok(v) => Some(v),
                Err(e) => return service_error_to_response(e),
            },
            _ => {
                if !allow_missing_name {
                    return api_error(StatusCode::BAD_REQUEST, 1001, "学校名称/机构名称不能为空");
                }
                None
            }
        };

    let province = match scope.locked_province.clone() {
        Some(locked) => {
            if let Some(raw) = input.province.as_deref() {
                if !raw.trim().is_empty() && raw.trim() != locked {
                    return api_error(
                        StatusCode::FORBIDDEN,
                        1003,
                        "province out of current admin scope",
                    );
                }
            }
            locked
        }
        None => match input.province.as_deref() {
            Some(raw) if !raw.trim().is_empty() => raw.trim().to_string(),
            _ => return api_error(StatusCode::BAD_REQUEST, 1001, "province is required"),
        },
    };
    if province.chars().count() > MAX_PROVINCE_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province too long");
    }
    let mut city = input.city.trim().to_string();
    // SHI_ADMIN 锁定市
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

    // ── 分类判定 ──
    // 两步式:私权机构 institution_name 为 None 时,derive_category 传空串不影响
    // (classify 仅在 GFR+ZF+"公民安全局" 才返回 PublicSecurity,对私权无副作用)
    let name_for_classify = institution_name_opt.as_deref().unwrap_or("");
    let category = match derive_category(&a3, &institution_code, name_for_classify) {
        Some(c) => c,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "a3/institution combination is not a valid institution",
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

    // ── A3 ↔ P1 联动硬校验 ──
    // 两步式:SFR/FFR 均允许 P1=0/1,由用户自主选择(sub_type 联动延后到 update_institution)
    if matches!(a3.as_str(), "SFR" | "FFR") {
        if p1_input != "0" && p1_input != "1" {
            return api_error(StatusCode::BAD_REQUEST, 1001, "P1 非法(仅 0/1)");
        }
    }

    // 两步式:创建阶段禁止 sub_type,由详情页 update_institution 设置。
    if input
        .sub_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some()
    {
        return api_error(StatusCode::BAD_REQUEST, 1001, "创建阶段不接受 sub_type");
    }
    let validated_sub_type: Option<String> = None;

    // ── 机构名称唯一校验:公权机构同城唯一,私权若未命名则跳过 ──
    if let Some(ref name) = institution_name_opt {
        let is_public = a3 == "GFR";
        let name_exists = match state.store.read() {
            Ok(store) => {
                if is_public {
                    institution_name_exists_in_city(&store, name, &city)
                } else {
                    institution_name_exists(&store, name)
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "create_institution: store read failed for name check");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed");
            }
        };
        if name_exists {
            if is_public {
                return api_error(StatusCode::CONFLICT, 1007, "该市已存在同名机构");
            }
            return api_error(StatusCode::CONFLICT, 1007, "该机构名称已被使用");
        }
    }

    // ── 生成 sfid_number(碰撞重试,1000 次保护栏)──
    //
    // n9 桶 = 10⁹,单 (a3, 省, 市, 机构, year) 5 元组共享。最坏情况下
    // 单省 1.5 亿人口仅占桶 15%,1000 次都撞概率 ≈ 0.15^1000 ≈ 10⁻⁸²⁴。
    // 1000 次保护栏的实际作用是防极端饱和(桶填到 99% 以上)与代码 bug 死循环。
    for _ in 0..1000u32 {
        let random_account = Uuid::new_v4().to_string();
        let site_sfid = match generate_sfid_code(GenerateSfidInput {
            account_pubkey: random_account.as_str(),
            a3: a3.as_str(),
            p1: p1_input.as_str(),
            province: province.as_str(),
            city: city.as_str(),
            institution: institution_code.as_str(),
        }) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
        };
        let site_sfid = match validate_sfid_number_format(site_sfid.as_str()) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg),
        };
        let province_code = extract_province_code(&site_sfid);
        let city_code = extract_city_code(&site_sfid);

        // 中文注释:写入进程内省分片缓存。
        let inst = MultisigInstitution {
            sfid_number: site_sfid.clone(),
            institution_name: institution_name_opt.clone(),
            category,
            source: None,
            institution_level: None,
            a3: a3.clone(),
            p1: p1_input.clone(),
            province: province.clone(),
            city: city.clone(),
            province_code,
            city_code,
            institution_code: institution_code.clone(),
            sub_type: validated_sub_type.clone(),
            parent_sfid_number: None, // 两步式:FFR 的所属法人由 update_institution 设置
            chain_status: InstitutionChainStatus::NotRegistered,
            chain_tx_hash: None,
            chain_block_number: None,
            chain_synced_at: None,
            created_by: ctx.admin_pubkey.clone(),
            created_at: Utc::now(),
        };
        let sfid_key = site_sfid.clone();
        let inst_for_shard = inst.clone();
        let write_result = state
            .sharded_store
            .write_province(&province, move |shard| {
                if shard.multisig_institutions.contains_key(&sfid_key) {
                    return Err(());
                }
                shard.multisig_institutions.insert(sfid_key, inst_for_shard);
                Ok(())
            })
            .await;
        match write_result {
            Ok(Ok(())) => {}
            Ok(Err(())) => continue, // 碰撞,重试
            Err(e) => {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    &format!("shard write failed: {e}"),
                );
            }
        }
        if let Err(e) = state.store.upsert_institution_row(&inst) {
            tracing::error!(error = %e, "institution row upsert failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "institution row write failed",
            );
        }

        // 同步写模块 Store 快照,供审计与管理员反查读取同一份机构快照。
        {
            match state.store.write() {
                Ok(mut store) => {
                    store
                        .multisig_institutions
                        .insert(site_sfid.clone(), inst.clone());
                }
                Err(e) => {
                    tracing::warn!(error = %e, "module store snapshot write failed (institution create, shard already committed)");
                }
            }
        }

        // 审计日志走模块 Store 快照短锁。
        append_inst_audit_log_best_effort(
            &state,
            "INSTITUTION_CREATE",
            &ctx.admin_pubkey,
            Some(site_sfid.clone()),
            None,
            "SUCCESS",
            format!(
                "sfid={} name={:?} category={:?} province={} city={}",
                site_sfid, institution_name_opt, category, province, city
            ),
        );

        // ── 自动插入 2 条默认未上链账户(主账户 / 费用账户)──
        // 2026-04-21 统一两步模式:所有机构(公权/私权/公安局)创建时立即生成这两条本地
        // 账户记录,状态 = NotOnChain。链上注册成功后只能由链路同步接口改为 ActiveOnChain。
        insert_default_accounts_best_effort(&state, &site_sfid, &province, &ctx.admin_pubkey).await;

        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: CreateInstitutionOutput {
                sfid_number: site_sfid,
                institution_name: institution_name_opt.clone(),
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

// ─── 1b. 更新机构详情(两步式第二步)────────────────────────────
//
// PATCH /api/v1/institution/:sfid_number
//   body: { institution_name?: string, sub_type?: string | null }
//
// 仅允许编辑 institution_name 与 sub_type,其他(A3/P1/institution_code/省市)一律
// 不可变 — 那是 SFID 派生的基础属性,创建时已固化。
//
// 校验:
//   - scope:KEY=全国 / SHENG=本省 / SHI=本市
//   - institution_name:格式 + 全国唯一(排除自身 sfid_number)
//   - sub_type:与 (a3, p1) 联动(仅 SFR 可设,FFR/GFR 不得设)

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ListInstitutionQuery {
    pub category: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    /// 精确搜索关键字:匹配 institution_name 或 sfid_number。
    /// 为空或缺省时返回空页,避免管理员登录后拉取大范围数据。
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
    let category = match query.category.as_deref() {
        Some("PRIVATE_INSTITUTION") | Some("GOV_INSTITUTION") => query.category.as_deref(),
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
    let empty_page = || PageResult::<InstitutionListRow> {
        items: Vec::new(),
        page_size: query.page_size.unwrap_or(50).clamp(1, 100),
        next_cursor: None,
        has_more: false,
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
    let province = scope
        .locked_province
        .as_deref()
        .or(query.province.as_deref());
    let city = scope.locked_city.as_deref().or(query.city.as_deref());
    let Some(province_name) = province else {
        return api_error(StatusCode::FORBIDDEN, 1003, "province scope required");
    };
    let Some(province_code) = province_code_by_name(province_name) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "unknown province");
    };
    let city_code = match city {
        Some(city_name) => match city_code_by_name(province_name, city_name) {
            Some(code) => Some(code),
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "unknown city"),
        },
        None => None,
    };
    let page_size = query.page_size.unwrap_or(50).clamp(1, 100);
    let page = match state.store.list_institutions_exact(
        category,
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
        Err(e) => {
            tracing::warn!(error = %e, "list_institutions failed");
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
        data: page,
    })
    .into_response()
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ListPublicSecurityQuery {
    pub cursor: Option<String>,
    pub page_size: Option<usize>,
}
