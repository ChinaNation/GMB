use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};

use crate::admins::repo;
use crate::*;

/// 中文注释:二角色均可访问联邦管理员列表,但只能看自己所在省域。
pub(crate) async fn list_federal_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if ctx
        .admin_province
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return api_error(
            axum::http::StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        );
    }
    let scope_province = ctx.admin_province.clone();
    let result = state.db.with_client(move |conn| {
        let rows = repo::list_federal_admins_by_province_conn(conn, scope_province.as_deref())?
            .into_iter()
            .map(|(admin, province)| FederalAdminRow {
                id: admin.id,
                province: province.clone(),
                admin_pubkey: admin.admin_pubkey,
                admin_name: if admin.admin_name.is_empty() {
                    format!("{province}联邦管理员")
                } else {
                    admin.admin_name
                },
                built_in: admin.built_in,
                created_at: admin.created_at,
                updated_at: admin.updated_at,
            })
            .collect::<Vec<_>>();
        Ok(rows)
    });
    let rows = match result {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query federal admins failed: {err}");
            return api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                message.as_str(),
            );
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}
