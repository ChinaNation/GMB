//! 公民列表 / 公开身份查询 handlers
//!
//! 中文注释:公民查询能力属于 citizens 模块,不属于权限范围规则。
//! 因此本文件承接后台公民列表和公开身份查询入口。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use crate::*;

pub(crate) async fn admin_list_citizens(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CitizensQuery>,
) -> impl IntoResponse {
    let auth_ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope_province_code = auth_ctx
        .admin_province
        .as_deref()
        .and_then(|name| crate::china::provinces().iter().find(|p| p.name == name))
        .map(|p| p.code.to_string());
    let scope_city_code = auth_ctx
        .admin_city
        .as_deref()
        .and_then(|city_name| {
            auth_ctx
                .admin_province
                .as_deref()
                .and_then(|province_name| {
                    crate::china::provinces()
                        .iter()
                        .find(|p| p.name == province_name)
                        .and_then(|p| p.cities.iter().find(|c| c.name == city_name))
                })
        })
        .map(|c| c.code.to_string());

    let keyword = query.keyword.unwrap_or_default();
    let page_size = query.page_size.or(query.limit).unwrap_or(50).clamp(1, 100);
    if query.offset.unwrap_or(0) > 0 {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "offset pagination is not supported",
        );
    }
    let page = match state.db.list_citizens_exact(
        keyword.as_str(),
        scope_province_code.as_deref(),
        scope_city_code.as_deref(),
        query.cursor.as_deref(),
        page_size,
    ) {
        Ok(v) => v,
        Err(e) if e == "invalid page cursor" => {
            return api_error(StatusCode::BAD_REQUEST, 1001, "invalid page cursor")
        }
        Err(e) => {
            tracing::warn!(error = %e, "admin_list_citizens failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "citizen query failed",
            );
        }
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: page,
    })
    .into_response()
}

pub(crate) async fn public_identity_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PublicIdentitySearchQuery>,
) -> impl IntoResponse {
    // 查询结果仅含公开信息（SFID 码、档案号等），无需 token 认证。
    // 全局 rate limiter 已防滥用。
    // 中文注释:公开查询只返回电子护照绑定后的公开字段。
    let archive_no = query.archive_no.as_deref().map(str::trim).unwrap_or("");
    let identity_code = query.identity_code.as_deref().map(str::trim).unwrap_or("");
    let wallet_pubkey = query.wallet_pubkey.as_deref().map(str::trim).unwrap_or("");
    if archive_no.is_empty() && identity_code.is_empty() && wallet_pubkey.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "archive_no or identity_code or wallet_pubkey is required",
        );
    }

    let actor_ip = actor_ip_from_headers(&headers);
    let request_id = request_id_from_headers(&headers);
    let found = match state.db.with_client({
        let archive_no = archive_no.to_string();
        let identity_code = identity_code.to_string();
        let wallet_pubkey = wallet_pubkey.to_string();
        move |conn| {
            let row = conn
                .query_opt(
                    "SELECT archive_no, sfid_number, wallet_pubkey
                     FROM citizens
                     WHERE bind_status = 'BOUND'
                       AND (
                            ($1::text <> '' AND archive_no = $1)
                            OR ($2::text <> '' AND sfid_number = $2)
                            OR ($3::text <> '' AND lower(wallet_pubkey) = lower($3))
                       )
                     ORDER BY created_at DESC
                     LIMIT 1",
                    &[&archive_no, &identity_code, &wallet_pubkey],
                )
                .map_err(|e| format!("public citizen lookup failed: {e}"))?;
            Ok(row.map(|row| {
                (
                    row.get::<_, Option<String>>(0),
                    row.get::<_, String>(1),
                    row.get::<_, Option<String>>(2),
                )
            }))
        }
    }) {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(error = %err, "public_identity_search failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "identity query failed",
            );
        }
    };
    let output = PublicIdentitySearchOutput {
        found: found.is_some(),
        archive_no: found.as_ref().and_then(|r| r.0.clone()),
        identity_code: found.as_ref().map(|r| r.1.clone()),
        wallet_pubkey: found.as_ref().and_then(|r| r.2.clone()),
    };
    crate::core::runtime_ops::append_audit_log(
        &state,
        "PUBLIC_IDENTITY_SEARCH",
        "public",
        output.wallet_pubkey.clone(),
        format!(
            "found={} archive_no={:?} request_id={:?} actor_ip={:?}",
            output.found, output.archive_no, request_id, actor_ip
        ),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}
