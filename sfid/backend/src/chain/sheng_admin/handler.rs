//! 中文注释:`chain/sheng_admin/` 模块的 HTTP handler。
//!
//! 提供两类 endpoint:
//!
//! 1. **公开**(链反向调,无 session,但走全局 rate limit):
//!    - `GET /api/v1/chain/sheng-admin/list?province=AH`
//! 2. **session 触发型**(需 ShengAdmin session):
//!    - 实际 add/remove backup handler 各自定义在 `add_backup.rs` /
//!      `remove_backup.rs`,仅在 `main.rs` 路由表中挂载;本文件只导出公开
//!      `list_roster_public`。
//!
//! ## 与 `chain/institution_info/handler.rs` 的对齐
//!
//! 公开 endpoint 风格、错误码、ApiResponse wrapper 与 institution_info 一致。

#![allow(dead_code)]

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::chain::sheng_admin::query::fetch_roster;
use crate::login::require_sheng_admin;
use crate::models::ApiResponse;
use crate::sfid::province::province_name_by_code;
use crate::AppState;

/// `GET /api/v1/chain/sheng-admin/list?province=AH`
///
/// 入参 `province` 是 2 字母省 code(如 `AH`)。返回当前 3 槽公钥。
#[derive(Debug, Deserialize)]
pub(crate) struct ListRosterQuery {
    pub(crate) province: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ListRosterRow {
    /// 槽位标签:"MAIN" / "BACKUP_1" / "BACKUP_2"
    pub(crate) slot: &'static str,
    /// 0x 小写 hex(32 字节);未占用为 null。
    pub(crate) pubkey: Option<String>,
}

/// 中文注释:把 3 槽数组渲染成 ApiResponse rows 的共享 helper。
fn render_roster(slots: [Option<[u8; 32]>; 3]) -> Vec<ListRosterRow> {
    vec![
        ListRosterRow {
            slot: "MAIN",
            pubkey: slots[0].map(|p| format!("0x{}", hex::encode(p))),
        },
        ListRosterRow {
            slot: "BACKUP_1",
            pubkey: slots[1].map(|p| format!("0x{}", hex::encode(p))),
        },
        ListRosterRow {
            slot: "BACKUP_2",
            pubkey: slots[2].map(|p| format!("0x{}", hex::encode(p))),
        },
    ]
}

/// `GET /api/v1/admin/sheng-admin/roster`(session 触发型,本省名册)。
///
/// 从登录 session 取 admin_province,直接转 `fetch_roster`。
pub(crate) async fn list_roster_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.admin_province.clone() else {
        return crate::api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        );
    };
    match fetch_roster(province.as_str()).await {
        Ok(slots) => Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: render_roster(slots),
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(province = %province, error = %err, "fetch_roster failed");
            crate::api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                1502,
                "chain pull failed",
            )
        }
    }
}

/// 公开 endpoint(链反向调用):无 session,仅走全局 rate limit。
pub(crate) async fn list_roster_public(
    Query(q): Query<ListRosterQuery>,
) -> impl IntoResponse {
    let code = q.province.trim().to_ascii_uppercase();
    if code.is_empty() {
        return crate::api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "province query is required",
        );
    }
    let Some(province_name) = province_name_by_code(code.as_str()) else {
        return crate::api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "unknown province code",
        );
    };
    match fetch_roster(province_name).await {
        Ok(slots) => Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: render_roster(slots),
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(province = %province_name, error = %err, "fetch_roster failed");
            crate::api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                1502,
                "chain pull failed",
            )
        }
    }
}
