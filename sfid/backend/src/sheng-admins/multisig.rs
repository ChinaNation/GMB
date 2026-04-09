use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::business::scope::in_scope_multisig;
use crate::sfid::{generate_sfid_code, validate_sfid_id_format, GenerateSfidInput};
use crate::*;

use super::institutions::{
    extract_province_code_from_sfid, submit_register_sfid_institution_extrinsic,
};

const MAX_INSTITUTION_NAME_CHARS: usize = 30;
const MAX_INSTITUTION_NAME_BYTES: usize = 128;
const MAX_PROVINCE_CHARS: usize = 100;
const MAX_CITY_CHARS: usize = 100;

pub(crate) async fn generate_multisig_sfid(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GenerateMultisigSfidInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    // ── 校验机构名称 ──
    let institution_name = input.institution_name.trim().to_string();
    if institution_name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "institution_name is required");
    }
    if institution_name.chars().count() > MAX_INSTITUTION_NAME_CHARS {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "institution_name too long (max 30 chars)",
        );
    }
    if institution_name.len() > MAX_INSTITUTION_NAME_BYTES {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "institution_name too long (max 128 bytes)",
        );
    }

    // ── 省级 scope 处理 ──
    let province = match ctx.admin_province.as_deref() {
        Some(scope) => {
            if let Some(raw) = input.province.as_deref() {
                if !raw.trim().is_empty() && raw.trim() != scope {
                    return api_error(
                        StatusCode::FORBIDDEN,
                        1003,
                        "province out of current admin scope",
                    );
                }
            }
            scope.to_string()
        }
        None => match input.province.as_deref() {
            Some(raw) if !raw.trim().is_empty() => raw.trim().to_string(),
            _ => return api_error(StatusCode::BAD_REQUEST, 1001, "province is required"),
        },
    };
    if province.chars().count() > MAX_PROVINCE_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province too long");
    }

    // ── 校验其余输入 ──
    let a3 = input.a3.trim().to_string();
    let mut city = input.city.trim().to_string();
    let institution = input.institution.trim().to_string();
    if a3.is_empty() || institution.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "a3 and institution are required",
        );
    }
    // ShiAdmin 的市由 session 锁定，前端若未传 / 传了不一致，以 session 值为准并拒绝越界
    if ctx.role == AdminRole::ShiAdmin {
        let locked = match ctx.admin_city.as_deref() {
            Some(v) if !v.trim().is_empty() => v.trim().to_string(),
            _ => {
                return api_error(
                    StatusCode::FORBIDDEN,
                    1003,
                    "system admin city is not configured",
                )
            }
        };
        if !city.is_empty() && city != locked {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "city out of current admin scope",
            );
        }
        city = locked;
    }
    if city.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "city is required");
    }
    if city.chars().count() > MAX_CITY_CHARS {
        return api_error(StatusCode::BAD_REQUEST, 1001, "city too long");
    }
    let p1 = input.p1.as_deref().unwrap_or("");

    // ── 生成 SFID（碰撞重试） ──
    for _ in 0..5 {
        let random_account = Uuid::new_v4().to_string();
        let site_sfid = match generate_sfid_code(GenerateSfidInput {
            account_pubkey: random_account.as_str(),
            a3: a3.as_str(),
            p1,
            province: province.as_str(),
            city: city.as_str(),
            institution: institution.as_str(),
        }) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
        };
        let site_sfid = match validate_sfid_id_format(site_sfid.as_str()) {
            Ok(v) => v,
            Err(msg) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, msg),
        };
        let province_code = extract_province_code_from_sfid(&site_sfid);
        let created_at = Utc::now();

        // ── 写入 Store（Pending 状态） ──
        {
            let mut store = match store_write_or_500(&state) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            if store.multisig_sfid_records.contains_key(site_sfid.as_str()) {
                drop(store);
                continue;
            }
            store.multisig_sfid_records.insert(
                site_sfid.clone(),
                MultisigSfidRecord {
                    site_sfid: site_sfid.clone(),
                    a3: a3.clone(),
                    p1: p1.to_string(),
                    province: province.clone(),
                    city: city.clone(),
                    institution_code: institution.clone(),
                    institution_name: institution_name.clone(),
                    province_code: province_code.clone(),
                    chain_tx_hash: None,
                    chain_block_number: None,
                    chain_status: MultisigChainStatus::Pending,
                    created_by: ctx.admin_pubkey.clone(),
                    created_at,
                },
            );
            append_audit_log(
                &mut store,
                "MULTISIG_SFID_GENERATE",
                &ctx.admin_pubkey,
                Some(site_sfid.clone()),
                None,
                "SUCCESS",
                format!(
                    "site_sfid={} a3={} province={} city={} institution={}",
                    site_sfid, a3, province, city, institution,
                ),
            );
        }

        // ── 推送到链上 ──
        match submit_register_sfid_institution_extrinsic(
            &state,
            &site_sfid,
            &institution_name,
        )
        .await
        {
            Ok(receipt) => {
                let mut store = match store_write_or_500(&state) {
                    Ok(v) => v,
                    Err(resp) => return resp,
                };
                if let Some(record) = store.multisig_sfid_records.get_mut(&site_sfid) {
                    record.chain_status = MultisigChainStatus::Registered;
                    record.chain_tx_hash = Some(receipt.tx_hash.clone());
                    record.chain_block_number = Some(receipt.block_number);
                }
                append_audit_log(
                    &mut store,
                    "MULTISIG_SFID_CHAIN_REGISTER",
                    &ctx.admin_pubkey,
                    Some(site_sfid.clone()),
                    None,
                    "SUCCESS",
                    format!(
                        "site_sfid={} tx_hash={} block_number={}",
                        site_sfid, receipt.tx_hash, receipt.block_number,
                    ),
                );
                drop(store);

                return Json(ApiResponse {
                    code: 0,
                    message: "ok".to_string(),
                    data: GenerateMultisigSfidOutput {
                        site_sfid,
                        chain_status: MultisigChainStatus::Registered,
                        chain_tx_hash: Some(receipt.tx_hash),
                        chain_block_number: Some(receipt.block_number),
                    },
                })
                .into_response();
            }
            Err(err) => {
                let mut store = match store_write_or_500(&state) {
                    Ok(v) => v,
                    Err(resp) => return resp,
                };
                if let Some(record) = store.multisig_sfid_records.get_mut(&site_sfid) {
                    record.chain_status = MultisigChainStatus::Failed;
                }
                append_audit_log(
                    &mut store,
                    "MULTISIG_SFID_CHAIN_REGISTER",
                    &ctx.admin_pubkey,
                    Some(site_sfid.clone()),
                    None,
                    "FAILED",
                    format!("site_sfid={} error={}", site_sfid, err),
                );
                drop(store);

                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    &format!("chain register failed: {err}"),
                );
            }
        }
    }

    api_error(
        StatusCode::CONFLICT,
        1005,
        "multisig sfid collision retry exhausted",
    )
}

pub(crate) async fn list_multisig_sfids(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let scope = ctx.admin_province.as_deref();
    // ShiAdmin 需要进一步按市收敛：只能看到自己所属市的记录。
    let city_scope: Option<&str> = if ctx.role == AdminRole::ShiAdmin {
        ctx.admin_city.as_deref()
    } else {
        None
    };
    let mut rows: Vec<MultisigSfidListRow> = store
        .multisig_sfid_records
        .values()
        .filter(|r| in_scope_multisig(r, scope))
        .filter(|r| match city_scope {
            Some(city) => r.city == city,
            None => true,
        })
        .map(|r| {
            let created_by_name = store
                .admin_users_by_pubkey
                .get(&r.created_by)
                .map(|u| u.admin_name.clone())
                .unwrap_or_else(|| r.created_by.clone());
            MultisigSfidListRow {
                site_sfid: r.site_sfid.clone(),
                a3: r.a3.clone(),
                institution_code: r.institution_code.clone(),
                institution_name: r.institution_name.clone(),
                province: r.province.clone(),
                city: r.city.clone(),
                province_code: r.province_code.clone(),
                chain_status: r.chain_status.clone(),
                chain_tx_hash: r.chain_tx_hash.clone(),
                chain_block_number: r.chain_block_number,
                created_by: r.created_by.clone(),
                created_by_name,
                created_at: r.created_at.to_rfc3339(),
            }
        })
        .collect();

    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total = rows.len();
    let limit = query.limit.unwrap_or(100).min(500);
    let offset = query.offset.unwrap_or(0).min(total);
    let paged: Vec<MultisigSfidListRow> = rows.into_iter().skip(offset).take(limit).collect();

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: MultisigSfidListOutput {
            total,
            limit,
            offset,
            rows: paged,
        },
    })
    .into_response()
}

/// `DELETE /api/v1/admin/multisig-sfids/:site_sfid`
///
/// 删除一条多签注册机构 SFID 记录。
/// 业务规则：
/// 1. 仅允许删除**未成功上链**的记录（`chain_status != Registered`）；
///    `Registered` 的记录代表该 SFID 已在链上被多签账户引用，强删会造成链上/本地状态分裂，直接拒绝。
/// 2. 省级作用域按 `admin_province` 过滤；ShiAdmin 额外按 `admin_city` 过滤。
/// 3. 只允许调用者看到的记录被删。
pub(crate) async fn delete_multisig_sfid(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_sfid): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let target_sfid = site_sfid.trim().to_string();
    if target_sfid.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "site_sfid is required");
    }

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let record = match store.multisig_sfid_records.get(&target_sfid) {
        Some(r) => r.clone(),
        None => return api_error(StatusCode::NOT_FOUND, 1004, "multisig sfid not found"),
    };

    // 省级作用域校验
    if !in_scope_multisig(&record, ctx.admin_province.as_deref()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "multisig sfid out of current admin scope",
        );
    }
    // ShiAdmin 额外按市校验
    if ctx.role == AdminRole::ShiAdmin {
        let city_scope = ctx.admin_city.as_deref().unwrap_or("");
        if city_scope.is_empty() || record.city != city_scope {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "multisig sfid out of current admin city scope",
            );
        }
    }

    // 链上已注册的不能删
    if record.chain_status == MultisigChainStatus::Registered {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "cannot delete a multisig sfid already registered on chain",
        );
    }

    store.multisig_sfid_records.remove(&target_sfid);
    append_audit_log(
        &mut store,
        "MULTISIG_SFID_DELETE",
        &ctx.admin_pubkey,
        Some(target_sfid.clone()),
        None,
        "SUCCESS",
        format!(
            "site_sfid={} chain_status={:?}",
            target_sfid, record.chain_status
        ),
    );
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: target_sfid,
    })
    .into_response()
}
