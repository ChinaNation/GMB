//! 行政区划管理员只读接口。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::admins::login::AdminAuthContext;
use crate::china::{province_code_by_name, provinces};
use crate::number::model::{AdminSfidCitiesQuery, SfidCityItem};
use crate::*;

fn ok<T: Serialize>(data: T) -> axum::response::Response {
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data,
    })
    .into_response()
}

fn trimmed(value: &str, field: &str) -> Result<String, axum::response::Response> {
    let value = value.trim();
    if value.is_empty() {
        let message = format!("{field} is required");
        return Err(api_error(StatusCode::BAD_REQUEST, 1001, message.as_str()));
    }
    Ok(value.to_string())
}

fn ensure_province_scope(
    ctx: &AdminAuthContext,
    province_name: &str,
) -> Result<(), axum::response::Response> {
    let scope = ctx
        .admin_province
        .as_deref()
        .ok_or_else(|| api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing"))?;
    if scope != province_name {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "province out of current admin scope",
        ));
    }
    province_code_by_name(province_name)
        .map(|_| ())
        .ok_or_else(|| api_error(StatusCode::NOT_FOUND, 1004, "province not found"))
}

pub(crate) async fn admin_china_cities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminSfidCitiesQuery>,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let province_name = match trimmed(&query.province, "province") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_province_scope(&admin_ctx, &province_name) {
        return resp;
    }
    let Some(p) = provinces().iter().find(|p| p.name == province_name) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "province not found");
    };
    let mut rows: Vec<SfidCityItem> = p
        .cities
        .iter()
        .map(|c| SfidCityItem {
            name: c.name.to_string(),
            code: c.code.to_string(),
        })
        .collect();
    rows.sort_by(|a, b| a.code.cmp(&b.code));
    ok(rows)
}
