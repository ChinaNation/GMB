//! 主体 HTTP handler 共享辅助函数。
//!
//! 中文注释:这里只放跨公权、私权、账户、资料库和主体详情共用的 HTTP 辅助。
//! 数据读取写入统一走结构化表,不保留旧缓存回填逻辑。

use axum::http::StatusCode;

use crate::admins::login::AdminAuthContext;
use crate::admins::repo;
use crate::crypto::pubkey::normalize_admin_pubkey;
use crate::subjects::model::Institution;
use crate::subjects::service::{build_default_accounts, ServiceError};
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

pub(crate) fn extract_province_code(sfid: &str) -> String {
    sfid.split('-')
        .next()
        .map(|r5| r5[..2.min(r5.len())].to_string())
        .unwrap_or_default()
}

pub(crate) fn extract_city_code(sfid: &str) -> String {
    sfid.split('-')
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

pub(crate) fn ensure_institution_visible_to_admin(
    inst: &Institution,
    ctx: &AdminAuthContext,
) -> Result<(), axum::response::Response> {
    if let Some(ref locked_province) = ctx.admin_province {
        if inst.province != *locked_province {
            return Err(api_error(
                StatusCode::FORBIDDEN,
                1003,
                "province out of scope",
            ));
        }
    }
    if let Some(ref locked_city) = ctx.admin_city {
        if inst.city != *locked_city {
            return Err(api_error(StatusCode::FORBIDDEN, 1003, "city out of scope"));
        }
    }
    Ok(())
}

pub(crate) fn resolve_created_by(
    state: &AppState,
    created_by: &str,
) -> (Option<String>, Option<String>) {
    let Some(norm) = normalize_admin_pubkey(created_by) else {
        return (None, None);
    };
    let result = state.db.with_client(move |conn| {
        let Some(admin) = repo::get_admin_by_pubkey_conn(conn, norm.as_str())? else {
            return Ok((None, None));
        };
        let role_str = match admin.role {
            crate::admins::model::AdminRole::ShengAdmin => "FEDERAL_ADMIN",
            crate::admins::model::AdminRole::ShiAdmin => "SHI_ADMIN",
        };
        let name_opt = if admin.admin_name.trim().is_empty() {
            None
        } else {
            Some(admin.admin_name)
        };
        Ok((name_opt, Some(role_str.to_string())))
    });
    result.unwrap_or((None, None))
}

pub(crate) async fn insert_default_accounts_best_effort(
    state: &AppState,
    sfid_number: &str,
    _province: &str,
    created_by: &str,
) {
    for account in build_default_accounts(sfid_number, created_by) {
        if let Err(err) = state.db.upsert_institution_account_row(&account) {
            tracing::warn!(
                sfid = sfid_number,
                account = %account.account_name,
                error = %err,
                "default account upsert failed"
            );
        }
    }
}
