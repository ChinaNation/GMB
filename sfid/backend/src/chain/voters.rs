use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::chain::runtime_align::build_population_snapshot_credential;
use crate::*;

#[derive(Serialize)]
struct ChainVotersCountFingerprint<'a> {
    route: &'a str,
    request_id: &'a str,
    who: &'a str,
}

pub(crate) async fn chain_voters_count(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ChainVotersCountQuery>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    let who_raw = query.account_pubkey.or(query.who).unwrap_or_default();
    let Some(who) = normalize_account_pubkey(who_raw.as_str()) else {
        return api_error(
            axum::http::StatusCode::BAD_REQUEST,
            1001,
            "account_pubkey is required",
        );
    };
    let request_id_for_fingerprint =
        chain_header_value(&headers, "x-chain-request-id").unwrap_or_default();
    let fingerprint = match request_fingerprint(&ChainVotersCountFingerprint {
        route: "chain_voters_count",
        request_id: request_id_for_fingerprint.as_str(),
        who: who.as_str(),
    }) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let chain_auth =
        match prepare_chain_request(&state, &headers, "chain_voters_count", &fingerprint) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    let request_id = chain_auth.request_id;

    let eligible_total = {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        store.metrics.voters_count_total += 1;
        let total = store
            .bindings_by_pubkey
            .values()
            .filter(|b| b.citizen_status == CitizenStatus::Normal)
            .count() as u64;
        total
    };
    let snapshot = match build_population_snapshot_credential(
        &state,
        who.as_str(),
        eligible_total,
        Uuid::new_v4().to_string(),
    ) {
        Ok(v) => v,
        Err(message) => {
            let detail = format!("snapshot signature sign failed: {message}");
            return api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                detail.as_str(),
            );
        }
    };
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    append_audit_log_with_meta(
        &mut store,
        "CHAIN_VOTERS_COUNT",
        "chain",
        Some(who.clone()),
        None,
        Some(request_id),
        actor_ip,
        "SUCCESS",
        format!("eligible_total={}", snapshot.eligible_total),
    );
    record_chain_latency(&mut store, started_at);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: ChainVotersCountOutput {
            genesis_hash: snapshot.genesis_hash,
            eligible_total: snapshot.eligible_total,
            who: snapshot.who,
            snapshot_nonce: snapshot.snapshot_nonce,
            signature: snapshot.signature,
        },
    })
    .into_response()
}
