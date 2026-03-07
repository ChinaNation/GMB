use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::chain::runtime_align::{build_vote_credential, RuntimeVoteCredential};
use crate::*;

const MAX_VOTE_REVALIDATION_ATTEMPTS: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
enum VoteDecisionSource {
    CacheHit,
    BindingState,
    Unbound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct VoteDecision {
    source: VoteDecisionSource,
    is_bound: bool,
    has_vote_eligibility: bool,
    sfid_code: Option<String>,
    archive_index: Option<String>,
    citizen_status: Option<CitizenStatus>,
}

impl VoteDecision {
    fn from_cache(cache: VoteVerifyCacheEntry) -> Self {
        Self {
            source: VoteDecisionSource::CacheHit,
            is_bound: cache.is_bound,
            has_vote_eligibility: cache.has_vote_eligibility,
            sfid_code: cache.sfid_code,
            archive_index: cache.archive_index,
            citizen_status: cache.citizen_status,
        }
    }

    fn from_binding(binding: BindingRecord) -> Self {
        let eligible = binding.citizen_status == CitizenStatus::Normal;
        Self {
            source: VoteDecisionSource::BindingState,
            is_bound: true,
            has_vote_eligibility: eligible,
            sfid_code: Some(binding.sfid_code),
            archive_index: Some(binding.archive_index),
            citizen_status: Some(binding.citizen_status),
        }
    }

    fn unbound() -> Self {
        Self {
            source: VoteDecisionSource::Unbound,
            is_bound: false,
            has_vote_eligibility: false,
            sfid_code: None,
            archive_index: None,
            citizen_status: None,
        }
    }

    fn same_outcome(&self, other: &Self) -> bool {
        self.is_bound == other.is_bound
            && self.has_vote_eligibility == other.has_vote_eligibility
            && self.sfid_code == other.sfid_code
            && self.archive_index == other.archive_index
            && self.citizen_status == other.citizen_status
    }

    fn audit_detail(&self, attempt: usize) -> String {
        let detail = match self.source {
            VoteDecisionSource::CacheHit => "vote eligibility cache hit".to_string(),
            VoteDecisionSource::BindingState | VoteDecisionSource::Unbound => {
                format!("eligible={}", self.has_vote_eligibility)
            }
        };
        if attempt > 1 {
            format!("{detail}; revalidated_attempt={attempt}")
        } else {
            detail
        }
    }

    fn to_cache_entry(
        &self,
        account_pubkey: String,
        proposal_id: Option<u64>,
    ) -> Option<VoteVerifyCacheEntry> {
        if matches!(self.source, VoteDecisionSource::CacheHit) {
            return None;
        }
        Some(VoteVerifyCacheEntry {
            account_pubkey,
            proposal_id,
            is_bound: self.is_bound,
            has_vote_eligibility: self.has_vote_eligibility,
            sfid_code: self.sfid_code.clone(),
            archive_index: self.archive_index.clone(),
            citizen_status: self.citizen_status.clone(),
            cached_at: Utc::now(),
        })
    }
}

fn vote_verify_message(is_bound: bool, has_vote_eligibility: bool) -> String {
    if has_vote_eligibility {
        "pubkey bound and vote eligible".to_string()
    } else if is_bound {
        "pubkey bound but not vote eligible".to_string()
    } else {
        "pubkey not bound, no vote eligibility".to_string()
    }
}

fn build_vote_output(
    account_pubkey: String,
    is_bound: bool,
    has_vote_eligibility: bool,
    sfid_code: Option<String>,
    proposal_id: u64,
    credential: Option<RuntimeVoteCredential>,
) -> VoteVerifyOutput {
    let (sfid_hash, proposal_id, vote_nonce, signature, key_id, key_version, alg) =
        if let Some(v) = credential {
            (
                Some(v.sfid_hash),
                Some(v.proposal_id),
                Some(v.vote_nonce),
                Some(v.signature),
                Some(v.meta.key_id),
                Some(v.meta.key_version),
                Some(v.meta.alg),
            )
        } else {
            (None, Some(proposal_id), None, None, None, None, None)
        };
    VoteVerifyOutput {
        account_pubkey,
        is_bound,
        has_vote_eligibility,
        sfid_code,
        sfid_hash,
        proposal_id,
        vote_nonce,
        signature,
        key_id,
        key_version,
        alg,
        message: vote_verify_message(is_bound, has_vote_eligibility),
    }
}

fn load_vote_decision(store: &Store, cache_key: &str, account_pubkey: &str) -> VoteDecision {
    if let Some(cache) = store.vote_verify_cache.get(cache_key).cloned() {
        return VoteDecision::from_cache(cache);
    }
    load_vote_decision_live(store, account_pubkey)
}

fn load_vote_decision_live(store: &Store, account_pubkey: &str) -> VoteDecision {
    if let Some(binding) = store.bindings_by_pubkey.get(account_pubkey).cloned() {
        return VoteDecision::from_binding(binding);
    }
    VoteDecision::unbound()
}

pub(crate) async fn verify_vote_eligibility(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<VoteVerifyInput>,
) -> impl IntoResponse {
    let started_at = Utc::now();
    let actor_ip = actor_ip_from_headers(&headers);
    let mut input = input;
    let Some(account_pubkey) = normalize_account_pubkey(input.account_pubkey.as_str()) else {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is invalid");
    };
    input.account_pubkey = account_pubkey.clone();
    let proposal_id = input.proposal_id;
    let fingerprint = request_fingerprint(&input);
    let chain_auth = match prepare_chain_request(&state, &headers, "vote_verify", &fingerprint) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let request_id = chain_auth.request_id;
    let cache_key = vote_cache_key(&account_pubkey, Some(proposal_id));
    for attempt in 1..=MAX_VOTE_REVALIDATION_ATTEMPTS {
        let decision = {
            let mut store = match store_write_or_500(&state) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            cleanup_vote_cache(&mut store, Utc::now());
            if attempt == 1 {
                store.metrics.vote_verify_total += 1;
            }
            load_vote_decision(&store, cache_key.as_str(), account_pubkey.as_str())
        };

        let vote_credential = if decision.has_vote_eligibility {
            let Some(sfid_code) = decision.sfid_code.as_deref() else {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1004,
                    "vote credential build failed",
                );
            };
            match build_vote_credential(
                &state,
                &account_pubkey,
                sfid_code,
                proposal_id,
                Uuid::new_v4().to_string(),
            ) {
                Ok(token) => Some(token),
                Err(message) => {
                    let detail = format!("vote credential sign failed: {message}");
                    return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, detail.as_str());
                }
            }
        } else {
            None
        };

        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        cleanup_vote_cache(&mut store, Utc::now());
        let latest_live = load_vote_decision_live(&store, account_pubkey.as_str());
        if latest_live.same_outcome(&decision) {
            if let Some(entry) =
                latest_live.to_cache_entry(account_pubkey.clone(), Some(proposal_id))
            {
                insert_bounded_map(
                    &mut store.vote_verify_cache,
                    cache_key.clone(),
                    entry,
                    bounded_cache_limit("SFID_VOTE_VERIFY_CACHE_MAX", 50_000),
                );
            }
            append_audit_log_with_meta(
                &mut store,
                "CHAIN_VOTE_VERIFY",
                "chain",
                Some(account_pubkey.clone()),
                latest_live.archive_index.clone(),
                Some(request_id.clone()),
                actor_ip.clone(),
                "SUCCESS",
                if matches!(decision.source, VoteDecisionSource::CacheHit) {
                    format!("vote eligibility cache hit; live_revalidated_attempt={attempt}")
                } else {
                    latest_live.audit_detail(attempt)
                },
            );
            record_chain_latency(&mut store, started_at);
            let output = build_vote_output(
                account_pubkey.clone(),
                latest_live.is_bound,
                latest_live.has_vote_eligibility,
                latest_live.sfid_code.clone(),
                proposal_id,
                vote_credential,
            );
            return Json(ApiResponse {
                code: 0,
                message: "ok".to_string(),
                data: output,
            })
            .into_response();
        }
        store.vote_verify_cache.remove(&cache_key);
    }

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    append_audit_log_with_meta(
        &mut store,
        "CHAIN_VOTE_VERIFY",
        "chain",
        Some(account_pubkey),
        None,
        Some(request_id),
        actor_ip,
        "FAILED",
        "vote eligibility changed during signing window".to_string(),
    );
    record_chain_latency(&mut store, started_at);
    api_error(
        StatusCode::CONFLICT,
        3010,
        "vote eligibility changed, retry request",
    )
}
