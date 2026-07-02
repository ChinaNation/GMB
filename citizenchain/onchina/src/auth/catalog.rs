use axum::{extract::State, http::HeaderMap, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use std::collections::BTreeMap;

use crate::auth::repo;
use crate::core::chain_runtime;
use crate::crypto::pubkey::same_admin_account;
use crate::*;

fn balance_lookup_key(account: &str) -> String {
    let trimmed = account.trim();
    trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed)
        .to_ascii_lowercase()
}

fn balance_fen(balances: &BTreeMap<String, Option<String>>, account: &str) -> Option<String> {
    balances
        .get(balance_lookup_key(account).as_str())
        .cloned()
        .flatten()
}

/// 中文注释:Tier1 创世注册局管理员列表(本省 5 人组,「全走链读」决策③)。
///
/// 权威集合在链上 `PublicAdmins::FederalRegistryProvinceGroups[本省省码]`;本接口直读链上账户,
/// 回填本地缓存(缺失即补,保证有本地 id 供换届按 id 定位),再以缓存元数据(自定义名/时间戳)
/// 装配返回。省维度即本节点省(每节点单省);FRG/CREG 节点的管理员省一致,故按 ctx 省读本省组。
pub(crate) async fn list_federal_registry_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some(province) = ctx
        .scope_province_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
    else {
        return api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    };
    let Some(province_code) = chain_runtime::chain_province_code_by_name(&province) else {
        let message = format!("province '{province}' is not a valid chain province");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    };
    // 全走链读:本省 Tier1 创世注册局 5 人组的权威资料集合。
    let chain_profiles =
        match chain_runtime::fetch_federal_registry_province_admin_profiles(province_code).await {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "chain unreachable listing federal registry admins");
                return api_error(StatusCode::BAD_GATEWAY, 5002, "chain unreachable");
            }
        };
    let now = Utc::now();
    let tier1_code = chain_runtime::TIER1_REGISTRY_CODE.to_string();
    let balance_accounts = chain_profiles
        .iter()
        .map(|profile| profile.account_hex.clone())
        .collect::<Vec<_>>();
    let balance_by_account = match chain_runtime::fetch_account_balances_onchain(&balance_accounts)
        .await
    {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(error = %err, "chain balance unavailable listing federal registry admins");
            BTreeMap::new()
        }
    };
    let result = state.db.with_client(move |conn| {
        let mut rows = Vec::with_capacity(chain_profiles.len());
        for profile in &chain_profiles {
            let account = &profile.account_hex;
            // 缓存缺失即补一条 built_in 行(名字空→显示回退),保证有本地 id 供换届定位。
            let admin = match repo::get_admin_by_account_conn(conn, account)? {
                Some(admin) => admin,
                None => {
                    let row = AdminUser {
                        id: repo::next_admin_id_conn(conn)?,
                        admin_account: account.clone(),
                        admin_name: String::new(),
                        institution_code: tier1_code.clone(),
                        built_in: true,
                        created_by: "SYSTEM".to_string(),
                        created_at: now,
                        updated_at: None,
                        city_name: String::new(),
                    };
                    repo::upsert_admin_conn(conn, &row)?;
                    repo::get_admin_by_account_conn(conn, account)?
                        .ok_or_else(|| "federal admin cache backfill lost".to_string())?
                }
            };
            rows.push(FederalRegistryAdminRow {
                id: admin.id,
                province_name: province.clone(),
                admin_account: admin.admin_account,
                admin_name: profile.name.clone(),
                admin_cid_number: profile.admin_cid_number.clone(),
                name: profile.name.clone(),
                admin_role: profile.admin_role.clone(),
                term_start: profile.term_start,
                term_end: profile.term_end,
                source: profile.source,
                source_label: profile.source_label.clone(),
                balance_fen: balance_fen(&balance_by_account, account),
                built_in: admin.built_in,
                created_at: admin.created_at,
                updated_at: admin.updated_at,
            });
        }
        Ok(rows)
    });
    let rows = match result {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query federal registry admins failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}

/// 中文注释:普通机构只读“本机构管理员”列表。
///
/// 权威集合仍来自链上 Active 管理员集合;本地 admins 表只补展示姓名,不能决定准入。
pub(crate) async fn list_own_institution_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let capabilities = crate::platform::capability::capabilities_for(&ctx.institution_code);
    if !capabilities.can_view_own_admins {
        return api_error(StatusCode::FORBIDDEN, 1003, "permission denied");
    }
    let binding = match repo::active_node_binding(&state.db) {
        Ok(Some(binding)) => binding,
        Ok(None) => return api_error(StatusCode::FORBIDDEN, 2002, "not an on-chain admin"),
        Err(err) => {
            let message = format!("query node binding failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    if binding.candidate.institution_code != ctx.institution_code {
        return api_error(StatusCode::FORBIDDEN, 1003, "permission denied");
    }
    let identity = match chain_runtime::identity_from_binding_parts(
        &binding.candidate.institution_code,
        binding.candidate.institution_cid_number.as_deref(),
        binding.candidate.institution_main_account.as_deref(),
        binding.candidate.frg_province_code.as_deref(),
    ) {
        Ok(identity) => identity,
        Err(err) => {
            let message = format!("node binding invalid: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    let chain_profiles = match chain_runtime::fetch_active_admin_profiles_onchain(&identity).await {
        Ok(Some(profiles)) => profiles,
        Ok(None) => return api_error(StatusCode::FORBIDDEN, 2002, "not an on-chain admin"),
        Err(err) => {
            tracing::warn!(error = %err, "chain unreachable listing own institution admins");
            return api_error(StatusCode::BAD_GATEWAY, 5002, "chain unreachable");
        }
    };
    // 中文注释:列表展示也做一次链上 active 复查,避免后台清退窗口内的失效管理员继续读取。
    if !chain_profiles
        .iter()
        .any(|profile| same_admin_account(profile.account_hex.as_str(), ctx.admin_account.as_str()))
    {
        return api_error(StatusCode::FORBIDDEN, 2002, "not an on-chain admin");
    }
    let actor_account = ctx.admin_account.clone();
    let institution_code = ctx.institution_code.clone();
    let cid_short_name = ctx.cid_short_name.clone();
    let balance_accounts = chain_profiles
        .iter()
        .map(|profile| profile.account_hex.clone())
        .collect::<Vec<_>>();
    let balance_by_account = match chain_runtime::fetch_account_balances_onchain(&balance_accounts)
        .await
    {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(error = %err, "chain balance unavailable listing own institution admins");
            BTreeMap::new()
        }
    };
    let result = {
        let mut rows = Vec::with_capacity(chain_profiles.len());
        for profile in chain_profiles {
            let is_self = same_admin_account(profile.account_hex.as_str(), actor_account.as_str());
            let balance = balance_fen(&balance_by_account, profile.account_hex.as_str());
            rows.push(OwnInstitutionAdminRow {
                admin_account: profile.account_hex.clone(),
                admin_name: profile.name.clone(),
                admin_cid_number: profile.admin_cid_number,
                name: profile.name,
                admin_role: profile.admin_role,
                term_start: profile.term_start,
                term_end: profile.term_end,
                source: profile.source,
                source_label: profile.source_label,
                balance_fen: balance,
                is_self,
            });
        }
        Ok::<_, String>(OwnInstitutionAdminListOutput {
            institution_code,
            cid_short_name,
            rows,
        })
    };
    let data = match result {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query own institution admins failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data,
    })
    .into_response()
}
