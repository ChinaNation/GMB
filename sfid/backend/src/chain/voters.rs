use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use chrono::Utc;

use crate::*;

pub(crate) async fn chain_voters_count(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let fingerprint = request_fingerprint(&"chain_voters_count");
    let chain_auth =
        match require_chain_request(&mut store, &headers, "chain_voters_count", &fingerprint) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    if let Err(resp) =
        ensure_chain_request_db(&state, "chain_voters_count", &chain_auth, &fingerprint)
    {
        return resp;
    }
    store.metrics.voters_count_total += 1;
    let total_voters = store
        .bindings_by_pubkey
        .values()
        .filter(|b| b.citizen_status == CitizenStatus::Normal)
        .count();
    append_audit_log_with_meta(
        &mut store,
        "CHAIN_VOTERS_COUNT",
        "chain",
        None,
        None,
        Some(chain_auth.request_id),
        actor_ip,
        "SUCCESS",
        format!("total_voters={total_voters}"),
    );
    record_chain_latency(&mut store, started_at);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: ChainVotersCountOutput {
            total_voters,
            as_of: Utc::now().timestamp(),
        },
    })
    .into_response()
}
