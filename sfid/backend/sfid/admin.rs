use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use super::{generate_sfid_code, GenerateSfidInput};
use crate::sfid::province::provinces;
use crate::*;

// 中文注释:legacy admin_generate_sfid 已删除(依赖 pending_by_pubkey / bindings_by_pubkey)。
// SFID 生成走 institutions::handler 模块的两层模型。

pub(crate) async fn admin_sfid_meta(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scoped = admin_ctx.admin_province.clone();
    let mut provinces_rows: Vec<SfidProvinceItem> = provinces()
        .iter()
        .filter(|p| scoped.as_deref().map(|v| v == p.name).unwrap_or(true))
        .map(|p| SfidProvinceItem {
            name: p.name.to_string(),
            code: p.code.to_string(),
        })
        .collect();
    provinces_rows.sort_by(|a, b| a.code.cmp(&b.code));
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminSfidMetaOutput {
            a3_options: vec![
                SfidOptionItem {
                    label: "公民人",
                    value: "GMR",
                },
                SfidOptionItem {
                    label: "自然人",
                    value: "ZRR",
                },
                SfidOptionItem {
                    label: "智能人",
                    value: "ZNR",
                },
                SfidOptionItem {
                    label: "公法人",
                    value: "GFR",
                },
                SfidOptionItem {
                    label: "私法人",
                    value: "SFR",
                },
                SfidOptionItem {
                    label: "非法人",
                    value: "FFR",
                },
            ],
            institution_options: vec![
                SfidOptionItem {
                    label: "中国",
                    value: "ZG",
                },
                SfidOptionItem {
                    label: "政府",
                    value: "ZF",
                },
                SfidOptionItem {
                    label: "立法院",
                    value: "LF",
                },
                SfidOptionItem {
                    label: "司法院",
                    value: "SF",
                },
                SfidOptionItem {
                    label: "监察院",
                    value: "JC",
                },
                SfidOptionItem {
                    label: "公民教育委员会",
                    value: "JY",
                },
                SfidOptionItem {
                    label: "公民储备委员会",
                    value: "CB",
                },
                SfidOptionItem {
                    label: "公民储备银行",
                    value: "CH",
                },
                SfidOptionItem {
                    label: "他国",
                    value: "TG",
                },
            ],
            provinces: provinces_rows,
            scoped_province: scoped,
        },
    })
    .into_response()
}

pub(crate) async fn admin_sfid_cities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminSfidCitiesQuery>,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_write(&state, &headers) {
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
