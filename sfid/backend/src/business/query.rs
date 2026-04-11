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
    let _auth_ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let keyword = query.keyword.unwrap_or_default().trim().to_lowercase();
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0);

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut rows: Vec<CitizenRow> = Vec::new();

    // 从 citizen_records 构建列表
    for record in store.citizen_records.values() {
        rows.push(CitizenRow {
            id: record.id,
            account_pubkey: record.account_pubkey.clone(),
            account_address: record.account_address.clone(),
            archive_no: record.archive_no.clone(),
            sfid_code: record.sfid_code.clone(),
            province_code: record.province_code.clone(),
            status: record.status(),
        });
    }

    rows.sort_by_key(|r| r.id);

    if !keyword.is_empty() {
        rows.retain(|r| {
            r.account_address
                .as_ref()
                .map(|v| v.to_lowercase().contains(&keyword))
                .unwrap_or(false)
                || r.archive_no
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&keyword))
                    .unwrap_or(false)
                || r.sfid_code
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&keyword))
                    .unwrap_or(false)
        });
    }
    let rows = rows
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
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
    // 中文注释:legacy bindings_by_pubkey 已删除,改为从 citizen_records 查询。
    let archive_no = query.archive_no.as_deref().map(str::trim).unwrap_or("");
    let identity_code = query.identity_code.as_deref().map(str::trim).unwrap_or("");
    let account_pubkey = query.account_pubkey.as_deref().map(str::trim).unwrap_or("");
    if archive_no.is_empty() && identity_code.is_empty() && account_pubkey.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "archive_no or identity_code or account_pubkey is required",
        );
    }

    let actor_ip = actor_ip_from_headers(&headers);
    let request_id = request_id_from_headers(&headers);
    let found = {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        if !account_pubkey.is_empty() {
            store.citizen_id_by_pubkey.get(account_pubkey)
                .and_then(|cid| store.citizen_records.get(cid))
                .cloned()
        } else if !archive_no.is_empty() {
            store.citizen_id_by_archive_no.get(archive_no)
                .and_then(|cid| store.citizen_records.get(cid))
                .cloned()
        } else {
            store.citizen_records.values()
                .find(|r| r.sfid_code.as_deref() == Some(identity_code))
                .cloned()
        }
    };
    let output = PublicIdentitySearchOutput {
        found: found.is_some(),
        archive_no: found.as_ref().and_then(|r| r.archive_no.clone()),
        identity_code: found.as_ref().and_then(|r| r.sfid_code.clone()),
        account_pubkey: found.as_ref().and_then(|r| r.account_pubkey.clone()),
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    append_audit_log_with_meta(
        &mut store,
        "PUBLIC_IDENTITY_SEARCH",
        "public",
        output.account_pubkey.clone(),
        output.archive_no.clone(),
        request_id,
        actor_ip,
        "SUCCESS",
        format!("found={}", output.found),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}
