//! 主体 HTTP handler 共享辅助函数。
//!
//! 这里只放跨公权、私权、账户、资料库和主体详情共用的 HTTP 辅助。
//! 数据读取写入统一走结构化表,不保留旧缓存回填逻辑。

use axum::http::StatusCode;

use crate::auth::login::AdminAuthContext;
use crate::auth::repo;
use crate::crypto::pubkey::normalize_admin_account;
use crate::institution::subjects::model::Institution;
use crate::institution::subjects::service::ServiceError;
use crate::{api_error, AppState};

pub(crate) const MAX_PROVINCE_CHARS: usize = 100;
pub(crate) const MAX_CITY_CHARS: usize = 100;

pub(crate) fn service_error_to_response(e: ServiceError) -> axum::response::Response {
    let status = match e {
        ServiceError::BadInput(_) => StatusCode::BAD_REQUEST,
        ServiceError::NotFound(_) => StatusCode::NOT_FOUND,
        ServiceError::Conflict(_) => StatusCode::CONFLICT,
    };
    let code = match e {
        ServiceError::BadInput(_) => 1001,
        ServiceError::NotFound(_) => 1004,
        ServiceError::Conflict(_) => 1007,
    };
    api_error(status, code, e.message())
}

pub(crate) fn extract_province_code(cid: &str) -> String {
    cid.split('-')
        .next()
        .map(|r5| r5[..2.min(r5.len())].to_string())
        .unwrap_or_default()
}

pub(crate) fn extract_city_code(cid: &str) -> String {
    cid.split('-')
        .next()
        .and_then(|r5| {
            if r5.len() >= 5 {
                Some(r5[2..5].to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// 机构可见性闸,与全仓 get_visible_scope/includes_* 同一 fail-closed 语义——
/// scope 省/市缺省即空可见域,拒绝任何机构。登录守卫已先拒空 scope 会话,此处为纵深一致性,
/// 不再因 scope 字段为 None 而 fail-open 放行。
pub(crate) fn ensure_institution_visible_to_admin(
    inst: &Institution,
    ctx: &AdminAuthContext,
) -> Result<(), axum::response::Response> {
    let scope = crate::scope::get_visible_scope(ctx);
    if !scope.includes_province(&inst.province_name) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "province out of scope",
        ));
    }
    if !scope.includes_city(&inst.city_name) {
        return Err(api_error(StatusCode::FORBIDDEN, 1003, "city out of scope"));
    }
    if !scope.includes_town(&inst.town_name) {
        return Err(api_error(StatusCode::FORBIDDEN, 1003, "town out of scope"));
    }
    Ok(())
}

pub(crate) fn resolve_created_by(
    state: &AppState,
    created_by: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let Some(norm) = normalize_admin_account(created_by) else {
        return (None, None, None);
    };
    let result = state.db.with_client(move |conn| {
        let Some(admin) = repo::get_admin_by_account_conn(conn, norm.as_str())? else {
            return Ok((None, None, None));
        };
        let institution_code = admin.institution_code.clone();
        let family_name = admin.family_name.trim().to_string();
        let given_name = admin.given_name.trim().to_string();
        Ok((
            (!family_name.is_empty()).then_some(family_name),
            (!given_name.is_empty()).then_some(given_name),
            Some(institution_code),
        ))
    });
    result.unwrap_or((None, None, None))
}
