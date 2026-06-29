use axum::{Json, extract::State, http::HeaderMap, http::StatusCode, response::IntoResponse};
use chrono::Utc;

use crate::auth::repo;
use crate::core::chain_runtime;
use crate::*;

/// 中文注释:Tier1 创世注册局管理员列表(本省 5 人组,「全走链读」决策③)。
///
/// 权威集合在链上 `GenesisAdmins::FederalRegistryProvinceGroups[本省省码]`;本接口直读链上账户,
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
    // 全走链读:本省 Tier1 创世注册局 5 人组的权威账户集合。
    let chain_accounts =
        match chain_runtime::fetch_federal_registry_province_admins(province_code).await {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "chain unreachable listing federal registry admins");
                return api_error(StatusCode::BAD_GATEWAY, 5002, "chain unreachable");
            }
        };
    let now = Utc::now();
    let tier1_code = chain_runtime::TIER1_REGISTRY_CODE.to_string();
    let result = state.db.with_client(move |conn| {
        let mut rows = Vec::with_capacity(chain_accounts.len());
        for account in &chain_accounts {
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
                admin_name: federal_registry_display_name(admin.admin_name.as_str()),
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

fn federal_registry_display_name(raw: &str) -> String {
    let name = raw.trim();
    if name.is_empty() || is_generated_federal_registry_name(name) {
        return "联邦注册局管理员".to_string();
    }
    name.to_string()
}

fn is_generated_federal_registry_name(name: &str) -> bool {
    if !matches!(name.chars().last(), Some('1'..='5')) {
        return false;
    }
    let prefix = &name[..name.len() - 1];
    prefix.ends_with("联邦注册局管理员")
}
