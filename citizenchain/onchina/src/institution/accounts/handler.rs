//! 机构账户 HTTP handler。
//!
//! 机构账户只读写 `accounts` 结构化表,机构存在性和作用域从
//! `subjects` 表确认。

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Serialize;

use crate::auth::actions::require_admin_security_grant;
use crate::auth::login::require_admin_any;
use crate::auth::operation_auth::AdminActionType;
use crate::institution::subjects::model::{
    CreateAccountInput, CreateAccountOutput, InstitutionAccount,
};
use crate::institution::subjects::service::{
    can_delete_account, is_protocol_account_name, validate_account_name,
};
use crate::scope::get_visible_scope;
use crate::*;

pub(crate) async fn create_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(cid_number): Path<String>,
    Json(input): Json<CreateAccountInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let account_name = match validate_account_name(&input.account_name) {
        Ok(v) => v,
        Err(e) => return crate::institution::subjects::http::service_error_to_response(e),
    };
    let grant_payload = serde_json::json!({
        "target": cid_number.clone(),
        "cid_number": cid_number.clone(),
        "account_name": account_name.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::InstitutionCreateAccount,
        cid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let Some((inst, accounts)) = (match state.db.get_institution_with_accounts(&cid_number) {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query institution failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    let scope = get_visible_scope(&ctx);
    if !scope.includes_province(&inst.province_name)
        || !scope.includes_city(&inst.city_name)
        || !scope.includes_town(&inst.town_name)
    {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "institution out of current admin scope",
        );
    }
    if accounts
        .iter()
        .any(|account| account.account_name == account_name)
    {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "account_name already exists under this institution",
        );
    }
    let now = Utc::now();
    let account = InstitutionAccount {
        cid_number: cid_number.clone(),
        account_name: account_name.clone(),
        account: crate::institution::accounts::derive::derive_account(&cid_number, &account_name),
        created_by: ctx.admin_account.clone(),
        created_at: now,
    };
    if let Err(err) = state.db.upsert_institution_account_row(&account) {
        let message = format!("write account failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    }
    crate::core::runtime_ops::append_audit_log(
        &state,
        "INSTITUTION_ACCOUNT_CREATE",
        &ctx.admin_account,
        Some(cid_number.clone()),
        serde_json::json!({
            "cid_number": cid_number.clone(),
            "account_name": account_name.clone(),
        }),
    );
    let account = account.account.clone();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CreateAccountOutput {
            cid_number,
            account_name,
            account,
        },
    })
    .into_response()
}

pub(crate) async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(cid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some((inst, accounts)) = (match state.db.get_institution_with_accounts(&cid_number) {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query accounts failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    let scope = get_visible_scope(&ctx);
    if !scope.includes_province(&inst.province_name)
        || !scope.includes_city(&inst.city_name)
        || !scope.includes_town(&inst.town_name)
    {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: accounts,
    })
    .into_response()
}

pub(crate) async fn delete_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((cid_number, account_name)): Path<(String, String)>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if is_protocol_account_name(&account_name) {
        return api_error(StatusCode::CONFLICT, 1007, "协议账户不可删除");
    }
    let Some((inst, accounts)) = (match state.db.get_institution_with_accounts(&cid_number) {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query account failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    let scope = get_visible_scope(&ctx);
    if !scope.includes_province(&inst.province_name)
        || !scope.includes_city(&inst.city_name)
        || !scope.includes_town(&inst.town_name)
    {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
    let Some(account) = accounts
        .iter()
        .find(|item| item.account_name == account_name)
        .cloned()
    else {
        return api_error(StatusCode::NOT_FOUND, 1004, "account not found");
    };
    if !can_delete_account(&account) {
        return api_error(StatusCode::CONFLICT, 1007, "协议机构账户不可删除");
    }
    let grant_payload = serde_json::json!({
        "target": cid_number.clone(),
        "cid_number": cid_number.clone(),
        "account_name": account_name.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::InstitutionDeleteAccount,
        cid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    if let Err(err) = state
        .db
        .delete_institution_account_row(&cid_number, &account_name)
    {
        let message = format!("delete account failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    }
    crate::core::runtime_ops::append_audit_log(
        &state,
        "INSTITUTION_ACCOUNT_DELETE",
        &ctx.admin_account,
        Some(cid_number.clone()),
        serde_json::json!({
            "cid_number": cid_number.clone(),
            "account_name": account_name.clone(),
        }),
    );
    #[derive(Serialize)]
    struct DeleteOutput {
        deleted: bool,
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: DeleteOutput { deleted: true },
    })
    .into_response()
}
