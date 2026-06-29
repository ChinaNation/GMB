use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};

use crate::auth::repo;
use crate::*;

/// 中文注释:联邦注册局管理员列表。联邦管理员看全量并按「省份」列区分;
/// 市注册局管理员只看本省联邦管理员,避免市侧获得无关省域目录。
pub(crate) async fn list_federal_registry_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if ctx
        .scope_province_name
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
    let scope_province = ctx.scope_province_name.clone();
    let province_filter = if ctx.institution_code == "FRG" {
        None
    } else {
        scope_province
    };
    let result = state.db.with_client(move |conn| {
        let rows =
            repo::list_federal_registry_admins_by_province_conn(conn, province_filter.as_deref())?
                .into_iter()
                .map(|(admin, province)| FederalRegistryAdminRow {
                    id: admin.id,
                    province_name: province,
                    admin_account: admin.admin_account,
                    admin_name: federal_registry_display_name(admin.admin_name.as_str()),
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
            let message = format!("query federal registry admins failed: {err}");
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
