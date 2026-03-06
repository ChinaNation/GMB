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

    let keyword = query.keyword.unwrap_or_default().trim().to_lowercase();
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0);

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut rows: Vec<CitizenRow> = Vec::new();

    if auth_ctx.role != AdminRole::QueryOnly {
        for pending in store.pending_by_pubkey.values() {
            if store
                .bindings_by_pubkey
                .contains_key(&pending.account_pubkey)
            {
                continue;
            }
            if !in_scope_pending(pending, auth_ctx.admin_province.as_deref()) {
                continue;
            }
            rows.push(CitizenRow {
                seq: pending.seq,
                account_pubkey: pending.account_pubkey.clone(),
                archive_index: None,
                sfid_code: store
                    .generated_sfid_by_pubkey
                    .get(&pending.account_pubkey)
                    .cloned(),
                citizen_status: None,
                is_bound: false,
            });
        }
    }

    for b in store.bindings_by_pubkey.values() {
        if !in_scope(b, auth_ctx.admin_province.as_deref()) {
            continue;
        }
        rows.push(CitizenRow {
            seq: b.seq,
            account_pubkey: b.account_pubkey.clone(),
            archive_index: Some(b.archive_index.clone()),
            sfid_code: Some(b.sfid_code.clone()),
            citizen_status: Some(b.citizen_status.clone()),
            is_bound: true,
        });
    }

    rows.sort_by_key(|r| r.seq);

    if auth_ctx.role == AdminRole::QueryOnly && keyword.is_empty() {
        rows.clear();
    }

    if !keyword.is_empty() {
        rows.retain(|r| {
            r.account_pubkey.to_lowercase().contains(&keyword)
                || r.archive_index
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

pub(crate) async fn admin_query_by_pubkey(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(input): Query<AdminQueryInput>,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    if input.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }

    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let pending = store
        .pending_by_pubkey
        .get(&input.account_pubkey)
        .filter(|p| in_scope_pending(p, admin_ctx.admin_province.as_deref()))
        .is_some();
    let binding = store
        .bindings_by_pubkey
        .get(&input.account_pubkey)
        .filter(|b| in_scope(b, admin_ctx.admin_province.as_deref()));

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminQueryOutput {
            account_pubkey: input.account_pubkey,
            found_pending: pending,
            found_binding: binding.is_some(),
            archive_index: binding.map(|b| b.archive_index.clone()),
            sfid_code: binding.map(|b| b.sfid_code.clone()),
        },
    })
    .into_response()
}

pub(crate) async fn public_identity_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PublicIdentitySearchQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_public_search_auth(&headers) {
        return resp;
    }
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
            store.bindings_by_pubkey.get(account_pubkey).cloned()
        } else if !archive_no.is_empty() {
            store
                .bindings_by_pubkey
                .values()
                .find(|b| b.archive_index == archive_no)
                .cloned()
        } else {
            store
                .bindings_by_pubkey
                .values()
                .find(|b| b.sfid_code == identity_code)
                .cloned()
        }
    };
    let output = PublicIdentitySearchOutput {
        found: found.is_some(),
        archive_no: found.as_ref().map(|b| b.archive_index.clone()),
        identity_code: found.as_ref().map(|b| b.sfid_code.clone()),
        account_pubkey: found.as_ref().map(|b| b.account_pubkey.clone()),
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
