//! 联邦注册局管理员查看市注册局管理员列表与内部变更辅助函数。
//!
//! 中文注释:联邦注册局管理员/市注册局管理员只通过 admins 结构化表查询同省域市注册局管理员,不做全量内存过滤。

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use postgres::Client;

use crate::auth::login::build_admin_name_from_user;
use crate::auth::repo;
use crate::crypto::pubkey::same_admin_account;
use crate::*;

pub(crate) const MAX_ADMIN_NAME_CHARS: usize = 200;
pub(crate) const MAX_CITY_REGISTRY_ADMINS_PER_CITY: usize = 30;

pub(crate) async fn list_city_registry_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if crate::core::chain_runtime::is_subordinate_registry(&ctx.institution_code)
        && ctx
            .scope_city_name
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return api_error(StatusCode::FORBIDDEN, 1003, "admin city scope missing");
    }
    if ctx
        .scope_province_name
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    }
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0);
    let actor_province_name = ctx.scope_province_name.clone();
    let actor_city_name =
        if crate::core::chain_runtime::is_subordinate_registry(&ctx.institution_code) {
            ctx.scope_city_name.clone()
        } else {
            None
        };
    let result = state.db.with_client(move |conn| {
        // 省作用域校验保留(缺省即登录投影错误);列表按机构码 + 市过滤(每节点单省,省隐含)。
        actor_province_name
            .as_deref()
            .ok_or_else(|| "admin province scope missing".to_string())?;
        let (total, admins) = repo::list_city_registry_admins_by_scope_conn(
            conn,
            actor_city_name.as_deref(),
            limit,
            offset,
        )?;
        let rows = admins
            .into_iter()
            .map(|user| city_registry_row_from_user_conn(conn, &user))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CityRegistryAdminListOutput {
            total,
            limit,
            offset,
            rows,
        })
    });
    let data = match result {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query city registry admins failed: {err}");
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

pub(crate) fn can_manage_city_registry_conn(
    conn: &mut Client,
    actor_account: &str,
    actor_province_name: Option<&str>,
    city_registry: &AdminUser,
) -> Result<bool, String> {
    if same_admin_account(city_registry.created_by.as_str(), actor_account) {
        return Ok(true);
    }
    let Some(scope) = actor_province_name else {
        return Ok(false);
    };
    let city_registry_scope = repo::province_scope_for_registry_org_conn(
        conn,
        &city_registry.admin_account,
        &city_registry.institution_code,
    )?;
    Ok(city_registry_scope.as_deref() == Some(scope))
}

pub(crate) fn find_city_registry_by_id_conn(
    conn: &mut Client,
    id: u64,
) -> Result<Option<AdminUser>, String> {
    repo::get_admin_by_id_and_registry_org_conn(conn, id, "CREG")
}

pub(crate) fn city_registry_row_from_user_conn(
    conn: &mut Client,
    city_registry: &AdminUser,
) -> Result<CityRegistryAdminRow, String> {
    Ok(CityRegistryAdminRow {
        id: city_registry.id,
        admin_account: city_registry.admin_account.clone(),
        admin_name: city_registry_display_name(city_registry),
        institution_code: city_registry.institution_code.clone(),
        built_in: city_registry.built_in,
        created_by: city_registry.created_by.clone(),
        created_by_name: creator_admin_name_conn(conn, city_registry.created_by.as_str())?,
        created_at: city_registry.created_at,
        city_name: city_registry.city_name.clone(),
    })
}

fn city_registry_display_name(city_registry: &AdminUser) -> String {
    let name = city_registry.admin_name.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    let city = city_registry.city_name.trim();
    if city.is_empty() {
        return "市注册局管理员".to_string();
    }
    let suffix = if city.ends_with('市') { "" } else { "市" };
    format!("{city}{suffix}注册局管理员")
}

pub(crate) fn count_city_registry_admins_in_city_conn(
    conn: &mut Client,
    province: &str,
    city: &str,
) -> Result<usize, String> {
    let _ = province;
    let city = city.trim();
    repo::count_city_registry_admins_by_city_conn(conn, city)
}

pub(crate) fn creator_admin_name_conn(
    conn: &mut Client,
    creator_account: &str,
) -> Result<String, String> {
    let Some(creator) = repo::get_admin_by_account_conn(conn, creator_account)? else {
        return Ok("未知注册局管理员".to_string());
    };
    let province = if crate::core::chain_runtime::is_tier1_registry(&creator.institution_code) {
        repo::province_scope_for_registry_org_conn(
            conn,
            &creator.admin_account,
            &creator.institution_code,
        )?
    } else {
        None
    };
    Ok(build_admin_name_from_user(&creator, province.as_deref()))
}

pub(crate) fn ensure_city_in_creator_province_conn(
    conn: &mut Client,
    creator_account: &str,
    city: &str,
) -> Result<(String, String), axum::response::Response> {
    let city = city.trim();
    if city.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, "city is required"));
    }
    let province_name = repo::province_scope_for_registry_org_conn(conn, creator_account, "FRG")
        .map_err(|err| {
            let message = format!("query creator province failed: {err}");
            api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str())
        })?
        .ok_or_else(|| {
            api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from created_by",
            )
        })?;
    let Some(city_code) = crate::cid::china::city_code_by_name(province_name.as_str(), city) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "city not found in province",
        ));
    };
    if city_code == "000" {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "province placeholder city (000) is not allowed",
        ));
    }
    Ok((province_name, city.to_string()))
}
