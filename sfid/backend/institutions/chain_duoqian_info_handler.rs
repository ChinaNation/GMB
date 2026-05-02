//! 机构信息查询 6 个 handler。
//!
//! 全部端点都是只读 + 公开访问(无 admin token):由全局 rate limiter 防滥用。
//! 数据由 SFID 独立维护;链端 / 钱包按需 pull。

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use uuid::Uuid;

use super::dto::{
    AppAccountEntry, AppClearingBankRow, AppClearingBankSearchOutput, AppClearingBankSearchQuery,
    AppInstitutionAccounts, AppInstitutionDetail, AppInstitutionRegistrationCredential,
    AppInstitutionRegistrationInfo, AppInstitutionSearchQuery, AppInstitutionSearchRow,
    EligibleClearingBankRow, EligibleClearingBankSearchQuery,
};
use crate::app_core::chain_runtime::build_institution_registration_info_credential;
use crate::institutions::service::{
    can_delete_account, is_default_account_name, DEFAULT_ACCOUNT_NAMES,
};
use crate::models::{ApiResponse, MultisigChainStatus};
use crate::sfid::province::{province_name_by_code, PROVINCES};
use crate::*;

const MAX_PROVINCE_CHARS: usize = 100;
const MAX_CITY_CHARS: usize = 100;

fn extract_province_code(sfid: &str) -> String {
    sfid.split('-')
        .nth(1)
        .map(|r5| r5[..2.min(r5.len())].to_string())
        .unwrap_or_default()
}

/// 从 sfid_id 解析省代码并映射到省名。
/// 用于 handler 层确定 sharded_store 分片 key。
fn resolve_province_from_sfid_id(sfid_id: &str) -> Option<String> {
    let code = extract_province_code(sfid_id);
    if code.is_empty() {
        return None;
    }
    province_name_by_code(&code).map(|n| n.to_string())
}

// ─── 1. 通用机构搜索 ───────────────────────────────────────────

/// `GET /api/v1/app/institutions/search`
///
/// 区块链/钱包公开搜索机构。SFID 是身份源,链端通过本接口读取机构展示信息。
pub(crate) async fn app_search_institutions(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<AppInstitutionSearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20).clamp(1, 50) as usize;
    let q = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() <= 128)
        .map(|s| s.to_lowercase());

    let mut rows: Vec<AppInstitutionSearchRow> = Vec::new();
    for p in PROVINCES.iter() {
        if rows.len() >= limit {
            break;
        }
        let q_inner = q.clone();
        let read = state
            .sharded_store
            .read_province(p.name, move |shard| {
                let mut local = Vec::new();
                for inst in shard.multisig_institutions.values() {
                    if let Some(ref kw) = q_inner {
                        let sfid_lc = inst.sfid_id.to_lowercase();
                        let name_lc = inst
                            .institution_name
                            .as_deref()
                            .map(|name| name.to_lowercase())
                            .unwrap_or_default();
                        if !sfid_lc.contains(kw) && !name_lc.contains(kw) {
                            continue;
                        }
                    }
                    local.push(AppInstitutionSearchRow {
                        sfid_id: inst.sfid_id.clone(),
                        institution_name: inst.institution_name.clone(),
                        category: inst.category,
                        a3: inst.a3.clone(),
                        province: inst.province.clone(),
                        city: inst.city.clone(),
                        chain_status: inst.chain_status.clone(),
                    });
                }
                local
            })
            .await;
        match read {
            Ok(mut local) => rows.append(&mut local),
            Err(e) => {
                tracing::warn!(province = %p.name, error = %e, "app_search_institutions shard read failed");
            }
        }
    }
    rows.sort_by(|a, b| {
        a.province
            .cmp(&b.province)
            .then(a.city.cmp(&b.city))
            .then(a.sfid_id.cmp(&b.sfid_id))
    });
    if rows.len() > limit {
        rows.truncate(limit);
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}

// ─── 2. 单机构详情 ─────────────────────────────────────────────

/// `GET /api/v1/app/institutions/:sfid_id`
///
/// 区块链/钱包公开读取机构详情。机构名称变更后,链端重新调用本接口即可更新展示。
///
/// 中文注释:查询与注册分开。本接口不再签发 register_nonce/signature,
/// 链端注册请调用 `/registration-info` 专用接口。
pub(crate) async fn app_get_institution(
    State(state): State<AppState>,
    Path(sfid_id): Path<String>,
) -> impl IntoResponse {
    let province = match resolve_province_from_sfid_id(&sfid_id) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_id",
            )
        }
    };
    let sfid_id_r = sfid_id.clone();
    let read = state
        .sharded_store
        .read_province(&province, move |shard| {
            shard.multisig_institutions.get(&sfid_id_r).cloned()
        })
        .await;
    let inst = match read {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppInstitutionDetail {
            sfid_id: inst.sfid_id,
            institution_name: inst.institution_name,
            category: inst.category,
            a3: inst.a3,
            p1: inst.p1,
            province: inst.province,
            city: inst.city,
            province_code: inst.province_code,
            city_code: inst.city_code,
            institution_code: inst.institution_code,
            sub_type: inst.sub_type,
            parent_sfid_id: inst.parent_sfid_id,
            sfid_finalized: inst.sfid_finalized,
            chain_status: inst.chain_status,
        },
    })
    .into_response()
}

/// `GET /api/v1/app/institutions/:sfid_id/registration-info`
///
/// 链端注册专用接口。业务字段只返回:
/// - `sfid_id`
/// - `institution_name`
/// - `account_names`
///
/// 其余字段放在 `credential` 下,只用于链端验签和防重放。
pub(crate) async fn app_get_institution_registration_info(
    State(state): State<AppState>,
    Path(sfid_id): Path<String>,
) -> impl IntoResponse {
    let sfid_id = sfid_id.trim().to_string();
    if sfid_id.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "sfid_id is required");
    }
    let province = match resolve_province_from_sfid_id(&sfid_id) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_id",
            )
        }
    };
    let sfid_id_r = sfid_id.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_id_r).cloned();
            let mut account_names: Vec<String> = shard
                .multisig_accounts
                .values()
                .filter(|account| account.sfid_id == sfid_id_r)
                .filter(|account| !account.account_name.trim().is_empty())
                .map(|account| account.account_name.clone())
                .collect();
            // 中文注释:签名 payload 中的账户名列表必须稳定排序。默认账户固定排前,
            // 其他账户按名称排序,链端注册时必须使用本接口返回的顺序验签。
            account_names.sort_by(|left, right| {
                let rank = |name: &String| {
                    DEFAULT_ACCOUNT_NAMES
                        .iter()
                        .position(|default_name| *default_name == name.as_str())
                        .unwrap_or(DEFAULT_ACCOUNT_NAMES.len())
                };
                rank(left).cmp(&rank(right)).then(left.cmp(right))
            });
            account_names.dedup();
            (inst, account_names)
        })
        .await;
    let (inst_opt, account_names) = match read_result {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    let inst = match inst_opt {
        Some(i) => i,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    let institution_name = match inst.institution_name.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => {
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "institution_name is required before chain registration",
            )
        }
    };
    if account_names.is_empty() {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "institution account_names are required before chain registration",
        );
    }
    for default_name in DEFAULT_ACCOUNT_NAMES {
        if !account_names
            .iter()
            .any(|account_name| account_name == default_name)
        {
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "default account_names 主账户/费用账户 are required before chain registration",
            );
        }
    }

    let signing_province = inst.province.clone();
    // ADR-008 Phase 23e:每省 3 个 admin slot 都可以签发注册信息凭证。
    // 返回 signer_admin_pubkey,让链端能按 (province, admin_pubkey) 查签名公钥验签。
    let (signer_admin_pubkey, province_pair) = match state
        .sheng_admin_signing_cache
        .any_for_province_with_admin(signing_province.as_str())
    {
        Some(v) => v,
        None => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "province signing key not loaded; ask the province admin to login first",
            )
        }
    };
    let register_nonce = Uuid::new_v4().to_string();
    let credential = match build_institution_registration_info_credential(
        &state,
        sfid_id.as_str(),
        institution_name.as_str(),
        &account_names,
        register_nonce,
        signing_province.as_str(),
        signer_admin_pubkey,
        &province_pair,
    ) {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                &format!("build institution registration credential failed: {e}"),
            )
        }
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppInstitutionRegistrationInfo {
            sfid_id,
            institution_name,
            account_names,
            credential: AppInstitutionRegistrationCredential {
                genesis_hash: credential.genesis_hash,
                register_nonce: credential.register_nonce,
                province: credential.province,
                signer_admin_pubkey: credential.signer_admin_pubkey,
                signature: credential.signature,
                meta: credential.meta,
            },
        },
    })
    .into_response()
}

// ─── 3. 机构账户列表(脱敏) ─────────────────────────────────

/// `GET /api/v1/app/institutions/:sfid_id/accounts`
///
/// 只返回机构名/账户名/多签地址/链上状态,不暴露管理员/创建人等敏感字段。
pub(crate) async fn app_list_accounts(
    State(state): State<AppState>,
    Path(sfid_id): Path<String>,
) -> impl IntoResponse {
    let sfid_id = sfid_id.trim().to_string();
    if sfid_id.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "sfid_id is required");
    }
    let province = match resolve_province_from_sfid_id(&sfid_id) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_id",
            )
        }
    };
    let sfid_id_r = sfid_id.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_id_r).cloned();
            let accounts: Vec<AppAccountEntry> = shard
                .multisig_accounts
                .values()
                .filter(|a| a.sfid_id == sfid_id_r)
                .map(|a| AppAccountEntry {
                    account_name: a.account_name.clone(),
                    duoqian_address: a.duoqian_address.clone(),
                    chain_status: a.chain_status.clone(),
                    chain_synced_at: a.chain_synced_at,
                    is_default: is_default_account_name(&a.account_name),
                    can_delete: can_delete_account(a),
                })
                .collect();
            (inst, accounts)
        })
        .await;
    let (inst_opt, accounts) = match read_result {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    let inst = match inst_opt {
        Some(i) => i,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppInstitutionAccounts {
            sfid_id,
            institution_name: inst.institution_name.unwrap_or_default(),
            accounts,
        },
    })
    .into_response()
}

// ─── 4. 已激活清算行搜索(分页) ─────────────────────────────

/// `GET /api/v1/app/clearing-banks/search`
///
/// 语义(ADR-007):
/// - 仅返回**资格白名单**机构 ∩ **主账户已激活上链**(`ActiveOnChain`)
/// - 资格白名单:(SFR ∧ sub_type=JOINT_STOCK) ∨ (FFR ∧ parent.SFR ∧ parent.JOINT_STOCK)
/// - 默认跨全国 43 省;传 `province` 限定单省
/// - 主账户/费用账户地址从 `MultisigAccount` 对应 `(sfid_id, "主账户" | "费用账户")` 反查
///
/// 跨省 parent 解析:
/// - FFR 候选的 parent_sfid_id 可能在另一省 shard,采用 2 轮跨省读取:
///   - 第 1 轮:跨 43 省构建全国"SFR ∧ JOINT_STOCK"的 sfid_id 集合
///   - 第 2 轮:扫描候选时,FFR 仅当 parent_sfid_id ∈ 第 1 轮集合时通过
pub(crate) async fn app_search_clearing_banks(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<AppClearingBankSearchQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(20).clamp(1, 100);

    let keyword_lc: Option<String> = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase());

    let target_provinces: Vec<String> = match query.province.as_deref() {
        Some(p) if !p.trim().is_empty() => {
            if p.len() > MAX_PROVINCE_CHARS {
                return api_error(StatusCode::BAD_REQUEST, 1001, "province too long");
            }
            vec![p.trim().to_string()]
        }
        _ => PROVINCES.iter().map(|p| p.name.to_string()).collect(),
    };
    if let Some(ref c) = query.city {
        if c.len() > MAX_CITY_CHARS {
            return api_error(StatusCode::BAD_REQUEST, 1001, "city too long");
        }
    }

    // 第 1 轮:跨全国 43 省构建"SFR ∧ JOINT_STOCK"的快查表 sfid_id → (institution_name, a3)。
    // FFR 候选过滤时用此表判定 parent 是否合规。
    // 第 1 轮永远跨全国(不受 query.province 限制),因为 FFR 的 parent 可能在外省。
    let mut sfr_jointstock_lookup: std::collections::HashMap<String, (Option<String>, String)> =
        std::collections::HashMap::new();
    for p in PROVINCES.iter() {
        let read_result = state
            .sharded_store
            .read_province(p.name, move |shard| {
                let mut hits: Vec<(String, Option<String>, String)> = Vec::new();
                for inst in shard.multisig_institutions.values() {
                    if inst.a3 == "SFR" && inst.sub_type.as_deref() == Some("JOINT_STOCK") {
                        hits.push((
                            inst.sfid_id.clone(),
                            inst.institution_name.clone(),
                            inst.a3.clone(),
                        ));
                    }
                }
                hits
            })
            .await;
        match read_result {
            Ok(hits) => {
                for (sid, name, a3) in hits {
                    sfr_jointstock_lookup.insert(sid, (name, a3));
                }
            }
            Err(e) => {
                tracing::warn!(province = %p.name, error = %e, "app_search_clearing_banks: stage1 SFR-JOINT_STOCK collect failed");
            }
        }
    }

    let q_city = query.city.clone();
    let lookup = std::sync::Arc::new(sfr_jointstock_lookup);
    let mut all_rows: Vec<AppClearingBankRow> = Vec::new();
    // 第 2 轮:跨目标省扫描候选,过滤资格白名单 + 主账户已激活。
    //
    // 注意:历史上这里还做"已加入清算网络"过滤(走 ClearingBankNodes watcher 缓存),
    // 现已下线(SFID 不再读链,链端 chain pull 才是真源)。如果 wuminapp 需要进一步
    // 过滤"已声明清算节点"的机构,自己去链上 storage 查 OffchainTransaction::ClearingBankNodes。
    for prov in &target_provinces {
        let q_city_inner = q_city.clone();
        let q_kw_inner = keyword_lc.clone();
        let lookup_inner = lookup.clone();
        let read_result = state
            .sharded_store
            .read_province(prov, move |shard| {
                let mut main_addr: std::collections::HashMap<String, String> =
                    std::collections::HashMap::new();
                let mut fee_addr: std::collections::HashMap<String, String> =
                    std::collections::HashMap::new();
                for acc in shard.multisig_accounts.values() {
                    if acc.chain_status != MultisigChainStatus::ActiveOnChain {
                        continue;
                    }
                    let Some(ref addr) = acc.duoqian_address else {
                        continue;
                    };
                    match acc.account_name.as_str() {
                        "主账户" => {
                            main_addr.insert(acc.sfid_id.clone(), addr.clone());
                        }
                        "费用账户" => {
                            fee_addr.insert(acc.sfid_id.clone(), addr.clone());
                        }
                        _ => {}
                    }
                }

                let mut prov_rows: Vec<AppClearingBankRow> = Vec::new();
                for inst in shard.multisig_institutions.values() {
                    if !main_addr.contains_key(&inst.sfid_id) {
                        continue;
                    }

                    let (eligible, parent_info): (bool, Option<(String, Option<String>, String)>) =
                        match inst.a3.as_str() {
                            "SFR" => (inst.sub_type.as_deref() == Some("JOINT_STOCK"), None),
                            "FFR" => match inst.parent_sfid_id.as_deref() {
                                Some(pid) => match lookup_inner.get(pid) {
                                    Some((p_name, p_a3)) => (
                                        true,
                                        Some((pid.to_string(), p_name.clone(), p_a3.clone())),
                                    ),
                                    None => (false, None),
                                },
                                None => (false, None),
                            },
                            _ => (false, None),
                        };
                    if !eligible {
                        continue;
                    }

                    if let Some(ref c) = q_city_inner {
                        if inst.city != *c {
                            continue;
                        }
                    }
                    if let Some(ref kw) = q_kw_inner {
                        let sfid_lc = inst.sfid_id.to_lowercase();
                        let name_lc = inst
                            .institution_name
                            .as_deref()
                            .map(|n| n.to_lowercase())
                            .unwrap_or_default();
                        if !sfid_lc.contains(kw) && !name_lc.contains(kw) {
                            continue;
                        }
                    }

                    let (parent_sfid_id, parent_institution_name, parent_a3) = match parent_info {
                        Some((pid, pname, pa3)) => (Some(pid), pname, Some(pa3)),
                        None => (None, None, None),
                    };
                    prov_rows.push(AppClearingBankRow {
                        sfid_id: inst.sfid_id.clone(),
                        institution_name: inst.institution_name.clone().unwrap_or_default(),
                        a3: inst.a3.clone(),
                        sub_type: inst.sub_type.clone(),
                        parent_sfid_id,
                        parent_institution_name,
                        parent_a3,
                        province: inst.province.clone(),
                        city: inst.city.clone(),
                        main_account: main_addr.get(&inst.sfid_id).cloned(),
                        fee_account: fee_addr.get(&inst.sfid_id).cloned(),
                    });
                }
                prov_rows
            })
            .await;
        match read_result {
            Ok(rows) => all_rows.extend(rows),
            Err(e) => {
                tracing::warn!(province = %prov, error = %e, "app_search_clearing_banks: stage2 read failed");
            }
        }
    }

    all_rows.sort_by(|a, b| {
        a.province
            .cmp(&b.province)
            .then(a.city.cmp(&b.city))
            .then(a.sfid_id.cmp(&b.sfid_id))
    });

    let total = all_rows.len();
    let start = ((page.saturating_sub(1)) as usize) * (size as usize);
    let items = if start >= total {
        Vec::new()
    } else {
        let end = (start + size as usize).min(total);
        all_rows[start..end].to_vec()
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AppClearingBankSearchOutput {
            total,
            items,
            page,
            size,
        },
    })
    .into_response()
}

// ─── 5. 候选清算行搜索(可未激活) ────────────────────────────

/// `GET /api/v1/app/clearing-banks/eligible-search?q=<keyword>&limit=<N>`
///
/// 语义:
/// - 仅按资格白名单过滤(SFR + JOINT_STOCK 或 FFR + 合规 parent)
/// - 不要求主账户已激活上链(节点桌面 UI"添加清算行"用,可能正在创建中)
/// - 单页返回(无分页),`limit` 默认 20 上限 50
/// - 不按 province/city 过滤(sfid_id 全局唯一,精确定位)
pub(crate) async fn app_search_eligible_clearing_banks(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<EligibleClearingBankSearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20).clamp(1, 50) as usize;

    let keyword_lc: Option<String> = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() <= 64)
        .map(|s| s.to_lowercase());

    // 第 1 轮:全国扫 SFR-JOINT_STOCK 集合(用于 FFR 候选 parent 校验)
    let mut sfr_jointstock_lookup: std::collections::HashMap<String, (Option<String>, String)> =
        std::collections::HashMap::new();
    for p in PROVINCES.iter() {
        let read_result = state
            .sharded_store
            .read_province(p.name, move |shard| {
                let mut hits: Vec<(String, Option<String>, String)> = Vec::new();
                for inst in shard.multisig_institutions.values() {
                    if inst.a3 == "SFR" && inst.sub_type.as_deref() == Some("JOINT_STOCK") {
                        hits.push((
                            inst.sfid_id.clone(),
                            inst.institution_name.clone(),
                            inst.a3.clone(),
                        ));
                    }
                }
                hits
            })
            .await;
        match read_result {
            Ok(hits) => {
                for (sid, name, a3) in hits {
                    sfr_jointstock_lookup.insert(sid, (name, a3));
                }
            }
            Err(e) => {
                tracing::warn!(province = %p.name, error = %e, "app_search_eligible_clearing_banks: stage1 collect failed");
            }
        }
    }

    let lookup = std::sync::Arc::new(sfr_jointstock_lookup);
    let mut all_rows: Vec<EligibleClearingBankRow> = Vec::new();
    // 第 2 轮:跨全国 43 省扫候选(本接口不接受 province 过滤参数)
    for p in PROVINCES.iter() {
        if all_rows.len() >= limit {
            break;
        }
        let q_kw_inner = keyword_lc.clone();
        let lookup_inner = lookup.clone();
        let read_result = state
            .sharded_store
            .read_province(p.name, move |shard| {
                let mut main_addr: std::collections::HashMap<
                    String,
                    (Option<String>, MultisigChainStatus),
                > = std::collections::HashMap::new();
                let mut fee_addr: std::collections::HashMap<String, String> =
                    std::collections::HashMap::new();
                for acc in shard.multisig_accounts.values() {
                    match acc.account_name.as_str() {
                        "主账户" => {
                            main_addr.insert(
                                acc.sfid_id.clone(),
                                (acc.duoqian_address.clone(), acc.chain_status.clone()),
                            );
                        }
                        "费用账户" => {
                            if let Some(ref addr) = acc.duoqian_address {
                                fee_addr.insert(acc.sfid_id.clone(), addr.clone());
                            }
                        }
                        _ => {}
                    }
                }

                let mut prov_rows: Vec<EligibleClearingBankRow> = Vec::new();
                for inst in shard.multisig_institutions.values() {
                    let (eligible, parent_info): (bool, Option<(String, Option<String>, String)>) =
                        match inst.a3.as_str() {
                            "SFR" => (inst.sub_type.as_deref() == Some("JOINT_STOCK"), None),
                            "FFR" => match inst.parent_sfid_id.as_deref() {
                                Some(pid) => match lookup_inner.get(pid) {
                                    Some((p_name, p_a3)) => (
                                        true,
                                        Some((pid.to_string(), p_name.clone(), p_a3.clone())),
                                    ),
                                    None => (false, None),
                                },
                                None => (false, None),
                            },
                            _ => (false, None),
                        };
                    if !eligible {
                        continue;
                    }

                    if let Some(ref kw) = q_kw_inner {
                        let sfid_lc = inst.sfid_id.to_lowercase();
                        let name_lc = inst
                            .institution_name
                            .as_deref()
                            .map(|n| n.to_lowercase())
                            .unwrap_or_default();
                        if !sfid_lc.contains(kw) && !name_lc.contains(kw) {
                            continue;
                        }
                    }

                    let (parent_sfid_id, parent_institution_name, parent_a3) = match parent_info {
                        Some((pid, pname, pa3)) => (Some(pid), pname, Some(pa3)),
                        None => (None, None, None),
                    };
                    let (main_account_addr, main_chain_status) = match main_addr.get(&inst.sfid_id)
                    {
                        Some((addr, status)) => (addr.clone(), status.clone()),
                        None => (None, MultisigChainStatus::NotOnChain),
                    };
                    prov_rows.push(EligibleClearingBankRow {
                        sfid_id: inst.sfid_id.clone(),
                        institution_name: inst.institution_name.clone(),
                        a3: inst.a3.clone(),
                        sub_type: inst.sub_type.clone(),
                        parent_sfid_id,
                        parent_institution_name,
                        parent_a3,
                        province: inst.province.clone(),
                        city: inst.city.clone(),
                        main_account: main_account_addr,
                        fee_account: fee_addr.get(&inst.sfid_id).cloned(),
                        main_chain_status,
                    });
                }
                prov_rows
            })
            .await;
        match read_result {
            Ok(rows) => all_rows.extend(rows),
            Err(e) => {
                tracing::warn!(province = %p.name, error = %e, "app_search_eligible_clearing_banks: stage2 read failed");
            }
        }
    }

    all_rows.sort_by(|a, b| {
        a.province
            .cmp(&b.province)
            .then(a.city.cmp(&b.city))
            .then(a.sfid_id.cmp(&b.sfid_id))
    });
    if all_rows.len() > limit {
        all_rows.truncate(limit);
    }

    // 直接返 Vec(顶层信封 ApiResponse.data),节点客户端反序列化时
    // 必须按 `data: Vec<Row>` 的形态读取,不要套 `data.items`。
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: all_rows,
    })
    .into_response()
}
