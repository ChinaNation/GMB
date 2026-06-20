//! 联邦管理员查看市管理员列表与内部变更辅助函数。
//!
//! 中文注释:联邦管理员/市管理员只通过 admins 结构化表查询同省域市管理员,不做全量内存过滤。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use postgres::Client;

use crate::admins::repo;
use crate::crypto::pubkey::same_admin_pubkey;
use crate::*;

pub(crate) const MAX_ADMIN_NAME_CHARS: usize = 200;
pub(crate) const MAX_CITY_ADMINS_PER_CITY: usize = 30;

pub(crate) async fn list_city_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if ctx.role == AdminRole::CityAdmin && ctx.admin_city.as_deref().unwrap_or("").trim().is_empty()
    {
        return api_error(StatusCode::FORBIDDEN, 1003, "admin city scope missing");
    }
    if ctx
        .admin_province
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    }
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0);
    let actor_province_name = ctx.admin_province.clone();
    let actor_city_name = if ctx.role == AdminRole::CityAdmin {
        ctx.admin_city.clone()
    } else {
        None
    };
    let result = state.db.with_client(move |conn| {
        let province = actor_province_name
            .as_deref()
            .ok_or_else(|| "admin province scope missing".to_string())?;
        let (total, admins) = repo::list_city_admins_by_scope_conn(
            conn,
            province,
            actor_city_name.as_deref(),
            limit,
            offset,
        )?;
        let rows = admins
            .into_iter()
            .map(|user| city_admin_row_from_user_conn(conn, &user))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CityAdminListOutput {
            total,
            limit,
            offset,
            rows,
        })
    });
    let data = match result {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query city admins failed: {err}");
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

pub(crate) fn can_manage_city_admin_conn(
    conn: &mut Client,
    actor_pubkey: &str,
    actor_province_name: Option<&str>,
    city_admin: &AdminUser,
) -> Result<bool, String> {
    if same_admin_pubkey(city_admin.created_by.as_str(), actor_pubkey) {
        return Ok(true);
    }
    let Some(scope) = actor_province_name else {
        return Ok(false);
    };
    let city_admin_scope =
        repo::province_scope_for_role_conn(conn, &city_admin.admin_pubkey, &city_admin.role)?;
    Ok(city_admin_scope.as_deref() == Some(scope))
}

pub(crate) fn find_city_admin_by_id_conn(
    conn: &mut Client,
    id: u64,
) -> Result<Option<AdminUser>, String> {
    repo::get_admin_by_id_and_role_conn(conn, id, &AdminRole::CityAdmin)
}

pub(crate) fn city_admin_row_from_user_conn(
    conn: &mut Client,
    city_admin: &AdminUser,
) -> Result<CityAdminRow, String> {
    Ok(CityAdminRow {
        id: city_admin.id,
        admin_pubkey: city_admin.admin_pubkey.clone(),
        admin_name: city_admin.admin_name.clone(),
        role: city_admin.role.clone(),
        built_in: city_admin.built_in,
        created_by: city_admin.created_by.clone(),
        created_by_name: creator_display_name_conn(conn, city_admin.created_by.as_str())?,
        created_at: city_admin.created_at,
        city: city_admin.city.clone(),
    })
}

pub(crate) fn count_city_admins_in_city_conn(
    conn: &mut Client,
    province: &str,
    city: &str,
) -> Result<usize, String> {
    let city = city.trim();
    repo::count_city_admins_by_city_conn(conn, province, city)
}

pub(crate) fn creator_display_name_conn(
    conn: &mut Client,
    creator_pubkey: &str,
) -> Result<String, String> {
    let Some(creator) = repo::get_admin_by_pubkey_conn(conn, creator_pubkey)? else {
        return Ok(creator_pubkey.to_string());
    };
    let province = if creator.role == AdminRole::FederalAdmin {
        repo::province_scope_for_role_conn(conn, &creator.admin_pubkey, &creator.role)?
    } else {
        None
    };
    Ok(build_admin_display_name(
        creator_pubkey,
        &creator.role,
        province.as_deref(),
    ))
}

pub(crate) fn ensure_city_in_creator_province_conn(
    conn: &mut Client,
    creator_pubkey: &str,
    city: &str,
) -> Result<(String, String), axum::response::Response> {
    let city = city.trim();
    if city.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, "city is required"));
    }
    let province_name =
        repo::province_scope_for_role_conn(conn, creator_pubkey, &AdminRole::FederalAdmin)
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
    let Some(city_code) = crate::china::city_code_by_name(province_name.as_str(), city) else {
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
