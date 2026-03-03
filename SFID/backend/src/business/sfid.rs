use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::sfid_tool::{generate_sfid_code, GenerateSfidInput};
use crate::*;

pub(crate) async fn admin_generate_sfid(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminGenerateSfidInput>,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_write(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.account_pubkey.trim().is_empty()
        || input.a3.trim().is_empty()
        || input.province.trim().is_empty()
        || input.city.trim().is_empty()
        || input.institution.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "account_pubkey, a3, province, city, institution are required",
        );
    }
    if let Some(scope) = admin_ctx.admin_province.as_deref() {
        if input.province.trim() != scope {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "province out of current admin scope",
            );
        }
    }

    let sfid_code = match generate_sfid_code(GenerateSfidInput {
        account_pubkey: input.account_pubkey.trim(),
        a3: input.a3.trim(),
        p1: input.p1.as_deref().unwrap_or(""),
        province: input.province.trim(),
        city: input.city.trim(),
        institution: input.institution.trim(),
    }) {
        Ok(v) => v,
        Err(msg) => return api_error(StatusCode::BAD_REQUEST, 1001, msg),
    };

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if store
        .bindings_by_pubkey
        .get(input.account_pubkey.trim())
        .is_some()
    {
        return api_error(StatusCode::CONFLICT, 3002, "citizen already bound");
    }
    let account_pubkey = input.account_pubkey.trim().to_string();
    let pending_scope = match store.pending_by_pubkey.get(&account_pubkey) {
        Some(pending) => pending.admin_province.clone(),
        None => return api_error(StatusCode::NOT_FOUND, 1004, "pending citizen not found"),
    };
    if let Some(scope) = admin_ctx.admin_province.as_deref() {
        if let Some(bound_scope) = pending_scope.as_deref() {
            if bound_scope != scope {
                return api_error(
                    StatusCode::FORBIDDEN,
                    1003,
                    "cannot manage other province citizens",
                );
            }
        } else if let Some(pending) = store.pending_by_pubkey.get_mut(&account_pubkey) {
            pending.admin_province = Some(scope.to_string());
        }
    }
    if !store.pending_by_pubkey.contains_key(&account_pubkey) {
        return api_error(StatusCode::NOT_FOUND, 1004, "pending citizen not found");
    }
    store
        .generated_sfid_by_pubkey
        .insert(account_pubkey.clone(), sfid_code.clone());

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminGenerateSfidOutput {
            account_pubkey,
            sfid_code,
        },
    })
    .into_response()
}

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
