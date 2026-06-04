//! 行政区划管理端接口。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::china::provinces;
use crate::number::model::{AdminSfidCitiesQuery, SfidCityItem};
use crate::*;

pub(crate) async fn admin_china_cities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminSfidCitiesQuery>,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if query.province.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "province is required");
    }
    if let Some(scope) = admin_ctx.admin_province.as_deref() {
        if scope != query.province.trim() {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "province out of current admin scope",
            );
        }
    }
    let Some(p) = provinces().iter().find(|p| p.name == query.province.trim()) else {
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
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}
