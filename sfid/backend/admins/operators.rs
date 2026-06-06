//! 联邦管理员查看市级管理员列表与内部变更辅助函数。
//!
//! 中文注释:联邦管理员/市级管理员只通过 admins 结构化表查询同省域操作员,不做全量内存过滤。

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
pub(crate) const MAX_SHI_ADMINS_PER_CITY: usize = 30;

pub(crate) async fn list_operators(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if ctx.role == AdminRole::ShiAdmin && ctx.admin_city.as_deref().unwrap_or("").trim().is_empty()
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
    let actor_province = ctx.admin_province.clone();
    let actor_city = if ctx.role == AdminRole::ShiAdmin {
        ctx.admin_city.clone()
    } else {
        None
    };
    let result = state.db.with_client(move |conn| {
        let province = actor_province
            .as_deref()
            .ok_or_else(|| "admin province scope missing".to_string())?;
        let (total, admins) = repo::list_shi_admins_by_scope_conn(
            conn,
            province,
            actor_city.as_deref(),
            limit,
            offset,
        )?;
        let rows = admins
            .into_iter()
            .map(|user| operator_row_from_user_conn(conn, &user))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(OperatorListOutput {
            total,
            limit,
            offset,
            rows,
        })
    });
    let data = match result {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query operators failed: {err}");
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

pub(crate) fn can_manage_operator_conn(
    conn: &mut Client,
    actor_pubkey: &str,
    actor_province: Option<&str>,
    operator: &AdminUser,
) -> Result<bool, String> {
    if same_admin_pubkey(operator.created_by.as_str(), actor_pubkey) {
        return Ok(true);
    }
    let Some(scope) = actor_province else {
        return Ok(false);
    };
    let operator_scope =
        repo::province_scope_for_role_conn(conn, &operator.admin_pubkey, &operator.role)?;
    Ok(operator_scope.as_deref() == Some(scope))
}

pub(crate) fn find_operator_by_id_conn(
    conn: &mut Client,
    id: u64,
) -> Result<Option<AdminUser>, String> {
    repo::get_admin_by_id_and_role_conn(conn, id, &AdminRole::ShiAdmin)
}

pub(crate) fn operator_row_from_user_conn(
    conn: &mut Client,
    operator: &AdminUser,
) -> Result<OperatorRow, String> {
    Ok(OperatorRow {
        id: operator.id,
        admin_pubkey: operator.admin_pubkey.clone(),
        admin_name: operator.admin_name.clone(),
        role: operator.role.clone(),
        built_in: operator.built_in,
        created_by: operator.created_by.clone(),
        created_by_name: creator_display_name_conn(conn, operator.created_by.as_str())?,
        created_at: operator.created_at,
        city: operator.city.clone(),
    })
}

pub(crate) fn count_shi_admins_in_city_conn(
    conn: &mut Client,
    province: &str,
    city: &str,
) -> Result<usize, String> {
    let city = city.trim();
    repo::count_shi_admins_by_city_conn(conn, province, city)
}

pub(crate) fn creator_display_name_conn(
    conn: &mut Client,
    creator_pubkey: &str,
) -> Result<String, String> {
    let Some(creator) = repo::get_admin_by_pubkey_conn(conn, creator_pubkey)? else {
        return Ok(creator_pubkey.to_string());
    };
    let province = if creator.role == AdminRole::ShengAdmin {
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
        repo::province_scope_for_role_conn(conn, creator_pubkey, &AdminRole::ShengAdmin)
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
