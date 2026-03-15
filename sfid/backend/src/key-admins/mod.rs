pub(crate) mod chain_keyring;
pub(crate) mod chain_proof;

use self::chain_keyring::{
    try_derive_pubkey_hex_from_seed, try_load_signing_key_from_seed, verify_rotation_signature,
    ChainKeyringState, KeySlot, RotateMainError, RotateMainRequest,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use schnorrkel::signing_context;
use std::{sync::OnceLock, time::Duration as StdDuration};
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};
use tokio::sync::Mutex as TokioMutex;
use tracing::warn;
use uuid::Uuid;

use blake2::{Blake2b, Digest};
use blake2::digest::consts::U32;

use crate::*;

type Blake2b256 = Blake2b<U32>;

#[derive(Debug, Clone)]
struct BackupSlotMaterial {
    pubkey: String,
    seed_hex: Option<SensitiveSeed>,
}

fn rotate_challenge_ttl_minutes() -> i64 {
    std::env::var("SFID_KEYRING_CHALLENGE_TTL_MINUTES")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(2)
}

fn rotate_challenge_max_active() -> usize {
    std::env::var("SFID_KEYRING_CHALLENGE_MAX_ACTIVE")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(2)
}

fn rotate_chain_submit_timeout() -> StdDuration {
    let seconds = std::env::var("SFID_CHAIN_ROTATE_FINALIZE_TIMEOUT_SECONDS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(90);
    StdDuration::from_secs(seconds)
}

fn rotate_commit_mutex() -> &'static TokioMutex<()> {
    static ROTATE_COMMIT_MUTEX: OnceLock<TokioMutex<()>> = OnceLock::new();
    ROTATE_COMMIT_MUTEX.get_or_init(|| TokioMutex::new(()))
}

fn is_production_mode() -> bool {
    optional_env("SFID_ENV")
        .or_else(|| optional_env("ENV"))
        .map(|v| v.eq_ignore_ascii_case("prod") || v.eq_ignore_ascii_case("production"))
        .unwrap_or(false)
}

fn normalize_pubkey_for_signing(value: &str) -> String {
    let trimmed = value.trim();
    let no_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    format!("0x{}", no_prefix.to_ascii_lowercase())
}

fn rotate_commit_signature_message(challenge_text: &str, new_backup_pubkey: &str) -> String {
    format!(
        "{challenge_text}|phase=commit|new_backup={}",
        normalize_pubkey_for_signing(new_backup_pubkey)
    )
}

fn should_keep_rotate_challenge(challenge: &KeyringRotateChallenge, now: DateTime<Utc>) -> bool {
    if challenge.consumed {
        // Consumed challenges are only retained until natural expiration for brief troubleshooting.
        return challenge.expire_at > now;
    }
    // Unconsumed challenges are kept while active, plus a short post-expiry grace window.
    challenge.expire_at > now - Duration::minutes(10)
}

pub(crate) async fn admin_get_chain_keyring(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = require_key_admin(&state, &headers) {
        return resp;
    }
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some(kr) = store.chain_keyring_state.as_ref() else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            1004,
            "chain keyring not initialized",
        );
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: KeyringStateOutput {
            version: kr.version,
            main_pubkey: kr.main_pubkey.clone(),
            backup_a_pubkey: kr.backup_a_pubkey.clone(),
            backup_b_pubkey: kr.backup_b_pubkey.clone(),
            updated_at: kr.updated_at,
        },
    })
    .into_response()
}

pub(crate) async fn admin_chain_keyring_rotate_challenge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<KeyringRotateChallengeInput>,
) -> impl IntoResponse {
    let ctx = match require_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.initiator_pubkey.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "initiator_pubkey is required",
        );
    }
    if !input
        .initiator_pubkey
        .trim()
        .eq_ignore_ascii_case(ctx.admin_pubkey.as_str())
    {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "initiator_pubkey must match current key admin",
        );
    }

    let now = Utc::now();
    let expire_at = now + Duration::minutes(rotate_challenge_ttl_minutes());
    let challenge_id = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_keyring_rotate_challenges(&mut store, now);
    let pending_for_initiator = store
        .keyring_rotate_challenges
        .values()
        .filter(|challenge| {
            !challenge.consumed
                && challenge.expire_at > now
                && challenge
                    .initiator_pubkey
                    .eq_ignore_ascii_case(ctx.admin_pubkey.as_str())
        })
        .count();
    let max_active = rotate_challenge_max_active();
    if pending_for_initiator >= max_active {
        return api_error(
            StatusCode::TOO_MANY_REQUESTS,
            1029,
            "too many active rotation challenges",
        );
    }
    let Some(current) = store.chain_keyring_state.as_ref().cloned() else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            1004,
            "chain keyring not initialized",
        );
    };

    // Challenge 阶段只确认“发起者必须是当前备用”。
    let initiator_pubkey = normalize_pubkey_for_signing(input.initiator_pubkey.as_str());
    if !initiator_pubkey.eq_ignore_ascii_case(current.backup_a_pubkey.as_str())
        && !initiator_pubkey.eq_ignore_ascii_case(current.backup_b_pubkey.as_str())
    {
        return map_rotate_main_error(RotateMainError::InitiatorMustBeBackup);
    }

    let challenge_text = format!(
        "sfid-keyring-rotate-v1|challenge_id={}|version={}|initiator={}|nonce={}|iat={}|exp={}|sigfmt=raw-v1",
        challenge_id,
        current.version,
        initiator_pubkey,
        nonce,
        now.timestamp(),
        expire_at.timestamp()
    );

    store.keyring_rotate_challenges.insert(
        challenge_id.clone(),
        KeyringRotateChallenge {
            challenge_id: challenge_id.clone(),
            keyring_version: current.version,
            initiator_pubkey: initiator_pubkey.clone(),
            challenge_text: challenge_text.clone(),
            expire_at,
            verified_at: None,
            consumed: false,
            created_by: ctx.admin_pubkey.clone(),
            created_at: now,
        },
    );
    append_audit_log(
        &mut store,
        "CHAIN_KEYRING_ROTATE_CHALLENGE",
        &ctx.admin_pubkey,
        None,
        None,
        "SUCCESS",
        format!(
            "challenge_id={challenge_id} keyring_version={}",
            current.version
        ),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: KeyringRotateChallengeOutput {
            challenge_id,
            keyring_version: current.version,
            challenge_text,
            expire_at: expire_at.timestamp(),
        },
    })
    .into_response()
}

pub(crate) async fn admin_chain_keyring_rotate_verify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<KeyringRotateVerifyInput>,
) -> impl IntoResponse {
    let ctx = match require_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.challenge_id.trim().is_empty() || input.signature.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id and signature are required",
        );
    }
    let now = Utc::now();
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_keyring_rotate_challenges(&mut store, now);
    let Some(challenge) = store
        .keyring_rotate_challenges
        .get(input.challenge_id.trim())
        .cloned()
    else {
        return api_error(StatusCode::NOT_FOUND, 1004, "rotation challenge not found");
    };
    if challenge.consumed {
        return api_error(
            StatusCode::CONFLICT,
            1007,
            "rotation challenge already consumed",
        );
    }
    if now > challenge.expire_at {
        return api_error(StatusCode::UNAUTHORIZED, 1007, "rotation challenge expired");
    }
    if !challenge
        .initiator_pubkey
        .eq_ignore_ascii_case(ctx.admin_pubkey.as_str())
    {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "rotation challenge owner mismatch",
        );
    }
    if !verify_rotation_signature(
        &challenge.initiator_pubkey,
        &challenge.challenge_text,
        input.signature.trim(),
    ) {
        return api_error(
            StatusCode::UNAUTHORIZED,
            2004,
            "rotation signature verify failed",
        );
    }
    if let Some(challenge_mut) = store
        .keyring_rotate_challenges
        .get_mut(input.challenge_id.trim())
    {
        challenge_mut.verified_at = Some(now);
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: KeyringRotateVerifyOutput {
            challenge_id: challenge.challenge_id,
            initiator_pubkey: challenge.initiator_pubkey,
            keyring_version: challenge.keyring_version,
            verified: true,
            message: "rotation signature verified",
        },
    })
    .into_response()
}

pub(crate) async fn admin_chain_keyring_rotate_commit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<KeyringRotateCommitInput>,
) -> impl IntoResponse {
    let ctx = match require_key_admin(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if input.challenge_id.trim().is_empty()
        || input.signature.trim().is_empty()
        || input.new_backup_pubkey.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, signature, new_backup_pubkey are required",
        );
    }
    // Serialize commit requests per process to avoid local lost-update races.
    let _commit_guard = rotate_commit_mutex().lock().await;

    let now = Utc::now();
    let (
        challenge_id,
        rotate_result,
        promoted_slot,
        new_main_pubkey,
        initiator_seed_hex,
        initiator_pubkey,
        new_backup_pubkey,
        current_state_before,
    ) = {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        cleanup_keyring_rotate_challenges(&mut store, now);
        let Some(challenge) = store
            .keyring_rotate_challenges
            .get(input.challenge_id.trim())
            .cloned()
        else {
            return api_error(StatusCode::NOT_FOUND, 1004, "rotation challenge not found");
        };
        if challenge.consumed {
            return api_error(
                StatusCode::CONFLICT,
                1007,
                "rotation challenge already consumed",
            );
        }
        if now > challenge.expire_at {
            return api_error(StatusCode::UNAUTHORIZED, 1007, "rotation challenge expired");
        }
        if challenge.verified_at.is_none() {
            return api_error(
                StatusCode::CONFLICT,
                1007,
                "rotation challenge not verified",
            );
        }
        if !challenge
            .initiator_pubkey
            .eq_ignore_ascii_case(ctx.admin_pubkey.as_str())
        {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "rotation challenge owner mismatch",
            );
        }
        let commit_message = rotate_commit_signature_message(
            challenge.challenge_text.as_str(),
            input.new_backup_pubkey.as_str(),
        );
        if !verify_rotation_signature(
            &challenge.initiator_pubkey,
            commit_message.as_str(),
            input.signature.trim(),
        ) {
            return api_error(
                StatusCode::UNAUTHORIZED,
                2004,
                "rotation signature verify failed",
            );
        }

        let Some(current) = store.chain_keyring_state.as_ref().cloned() else {
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                1004,
                "chain keyring not initialized",
            );
        };
        if current.version != challenge.keyring_version {
            return api_error(
                StatusCode::CONFLICT,
                1007,
                "chain keyring version changed, retry challenge",
            );
        }
        let current_state_before = current.clone();
        let initiator_seed_hex = {
            let known = match state.known_key_seeds.read() {
                Ok(v) => v,
                Err(_) => {
                    return api_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        1004,
                        "known signer seeds unavailable",
                    )
                }
            };
            let Some(seed) = known
                .iter()
                .find(|(pubkey, _)| {
                    pubkey.eq_ignore_ascii_case(challenge.initiator_pubkey.as_str())
                })
                .map(|(_, seed)| seed.clone())
            else {
                return api_error(
                    StatusCode::UNAUTHORIZED,
                    2004,
                    "initiator signer seed is not present on server",
                );
            };
            seed
        };
        let initiator_seed_pubkey =
            match try_derive_pubkey_hex_from_seed(initiator_seed_hex.expose_secret()) {
                Ok(v) => v,
                Err(err) => {
                    warn!(error = %err, "failed to derive pubkey from initiator seed");
                    return api_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        1004,
                        "invalid initiator signer seed",
                    );
                }
            };
        if !initiator_seed_pubkey.eq_ignore_ascii_case(challenge.initiator_pubkey.as_str()) {
            return api_error(
                StatusCode::UNAUTHORIZED,
                2004,
                "server signer seed does not match initiator_pubkey",
            );
        }
        let rotate_req = RotateMainRequest {
            initiator_pubkey: challenge.initiator_pubkey.clone(),
            new_backup_pubkey: normalize_pubkey_for_signing(input.new_backup_pubkey.as_str()),
        };
        let rotate_result = match current.rotate_main(rotate_req) {
            Ok(v) => v,
            Err(err) => return map_rotate_main_error(err),
        };
        let promoted_slot = rotate_result.promoted_slot.clone();
        let new_main_pubkey = rotate_result.state.main_pubkey.clone();
        (
            challenge.challenge_id,
            rotate_result,
            promoted_slot,
            new_main_pubkey,
            initiator_seed_hex,
            challenge.initiator_pubkey,
            normalize_pubkey_for_signing(input.new_backup_pubkey.as_str()),
            current_state_before,
        )
    };

    let chain_submit_timeout = rotate_chain_submit_timeout();
    let chain_submit = match tokio::time::timeout(
        chain_submit_timeout,
        submit_rotate_sfid_keys_extrinsic(
            initiator_pubkey.as_str(),
            initiator_seed_hex.expose_secret(),
            new_backup_pubkey.as_str(),
        ),
    )
    .await
    {
        Ok(v) => v,
        Err(_) => Err(format!(
            "rotate_sfid_keys submit failed: timed out waiting for finalization after {}s",
            chain_submit_timeout.as_secs()
        )),
    };
    let (chain_tx_hash, block_number, chain_submit_ok, chain_submit_error, response_message) =
        match chain_submit {
            Ok(receipt) => (
                receipt.tx_hash,
                Some(receipt.block_number),
                true,
                None,
                "chain keyring rotation included on chain".to_string(),
            ),
            Err(err) => (
                format!("submit_failed:{}", err),
                None,
                false,
                Some(err.clone()),
                format!(
                    "chain keyring rotation failed on chain, local keyring unchanged: {}",
                    err
                ),
            ),
        };

    let promoted_slot = match promoted_slot {
        KeySlot::Main => "MAIN",
        KeySlot::BackupA => "BACKUP_A",
        KeySlot::BackupB => "BACKUP_B",
    };

    if !chain_submit_ok {
        let mut response_state = current_state_before.clone();
        {
            let mut store = match store_write_or_500(&state) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            if let Some(current) = store.chain_keyring_state.as_ref().cloned() {
                response_state = current;
            }
            append_audit_log(
                &mut store,
                "CHAIN_KEYRING_ROTATE_COMMIT",
                &ctx.admin_pubkey,
                None,
                None,
                "FAILED",
                format!(
                    "challenge_id={} old_main={} new_main={} promoted_slot={} chain_tx_hash={} chain_submit_ok=false error={}",
                    challenge_id,
                    rotate_result.old_main_pubkey,
                    rotate_result.state.main_pubkey,
                    promoted_slot,
                    chain_tx_hash,
                    chain_submit_error.clone().unwrap_or_else(|| "unknown".to_string())
                ),
            );
        }
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: KeyringRotateCommitOutput {
                old_main_pubkey: response_state.main_pubkey.clone(),
                promoted_slot: promoted_slot.to_string(),
                chain_tx_hash,
                block_number,
                chain_submit_ok,
                chain_submit_error,
                version: response_state.version,
                main_pubkey: response_state.main_pubkey,
                backup_a_pubkey: response_state.backup_a_pubkey,
                backup_b_pubkey: response_state.backup_b_pubkey,
                updated_at: response_state.updated_at,
                message: response_message,
            },
        })
        .into_response();
    }

    {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let Some(current) = store.chain_keyring_state.as_ref().cloned() else {
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                1004,
                "chain keyring not initialized",
            );
        };
        if current.version != current_state_before.version {
            if let Some(challenge_mut) = store
                .keyring_rotate_challenges
                .get_mut(input.challenge_id.trim())
            {
                challenge_mut.consumed = true;
            }
            append_audit_log(
                &mut store,
                "CHAIN_KEYRING_ROTATE_COMMIT",
                &ctx.admin_pubkey,
                None,
                None,
                "SKIPPED",
                format!(
                    "challenge_id={} expected_version={} current_version={} old_main={} new_main={} promoted_slot={} chain_tx_hash={} block_number={} chain_submit_ok=true reason=concurrent_rotation_detected",
                    challenge_id,
                    current_state_before.version,
                    current.version,
                    rotate_result.old_main_pubkey,
                    rotate_result.state.main_pubkey,
                    promoted_slot,
                    chain_tx_hash,
                    block_number.unwrap_or_default(),
                ),
            );
            drop(store);
            if let Err(err) = reconcile_main_signer_with_keyring(&state) {
                warn!(
                    error = %err,
                    "failed to reconcile signer after concurrent rotation conflict"
                );
            }
            return api_error(
                StatusCode::CONFLICT,
                1007,
                "concurrent rotation completed, local state refresh required",
            );
        }
    }

    let previous_main_pubkey = match state.public_key_hex.read() {
        Ok(v) => v.clone(),
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "public key unavailable",
            );
        }
    };
    let previous_main_seed = match state.signing_seed_hex.read() {
        Ok(v) => v.clone(),
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "signing seed unavailable",
            );
        }
    };
    if let Err(err) = set_active_main_signer(
        &state,
        new_main_pubkey.as_str(),
        initiator_seed_hex.expose_secret(),
    ) {
        warn!(error = %err, "failed to switch active main signer");
        {
            let mut store = match store_write_or_500(&state) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            append_audit_log(
                &mut store,
                "CHAIN_KEYRING_ROTATE_COMMIT",
                &ctx.admin_pubkey,
                None,
                None,
                "FAILED",
                format!(
                    "challenge_id={} chain_tx_hash={} block_number={} chain_submit_ok=true local_signer_switch_error={}",
                    challenge_id,
                    chain_tx_hash,
                    block_number.unwrap_or_default(),
                    err
                ),
            );
        }
        if let Err(reconcile_err) = reconcile_main_signer_with_keyring(&state) {
            warn!(
                error = %reconcile_err,
                "failed to reconcile signer after local signer switch error"
            );
        }
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "failed to switch active main signer",
        );
    }
    if let Err(err) = persist_runtime_state_checked(&state) {
        warn!(error = %err, "failed to persist runtime signer state");
        if let Err(revert_err) = set_active_main_signer(
            &state,
            previous_main_pubkey.as_str(),
            previous_main_seed.expose_secret(),
        ) {
            warn!(error = %revert_err, "failed to rollback active main signer");
        }
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        append_audit_log(
            &mut store,
            "CHAIN_KEYRING_ROTATE_COMMIT",
            &ctx.admin_pubkey,
            None,
            None,
            "FAILED",
            format!(
                "challenge_id={} chain_tx_hash={} chain_submit_ok=true local_persist_error=failed to persist runtime signer state",
                challenge_id, chain_tx_hash
            ),
        );
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "failed to persist runtime signer state",
        );
    }

    {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        if let Some(challenge_mut) = store
            .keyring_rotate_challenges
            .get_mut(input.challenge_id.trim())
        {
            challenge_mut.consumed = true;
        }
        store.chain_keyring_state = Some(rotate_result.state.clone());
        sync_key_admin_users(&mut store);
        append_audit_log(
            &mut store,
            "CHAIN_KEYRING_ROTATE_COMMIT",
            &ctx.admin_pubkey,
            None,
            None,
            "SUCCESS",
            format!(
                "challenge_id={} old_main={} new_main={} promoted_slot={} chain_tx_hash={} block_number={} chain_submit_ok=true",
                challenge_id,
                rotate_result.old_main_pubkey,
                rotate_result.state.main_pubkey,
                promoted_slot,
                chain_tx_hash,
                block_number.unwrap_or_default()
            ),
        );
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: KeyringRotateCommitOutput {
            old_main_pubkey: rotate_result.old_main_pubkey,
            promoted_slot: promoted_slot.to_string(),
            chain_tx_hash,
            block_number,
            chain_submit_ok,
            chain_submit_error,
            version: rotate_result.state.version,
            main_pubkey: rotate_result.state.main_pubkey,
            backup_a_pubkey: rotate_result.state.backup_a_pubkey,
            backup_b_pubkey: rotate_result.state.backup_b_pubkey,
            updated_at: rotate_result.state.updated_at,
            message: response_message,
        },
    })
    .into_response()
}

pub(crate) fn cleanup_keyring_rotate_challenges(store: &mut Store, now: DateTime<Utc>) {
    store
        .keyring_rotate_challenges
        .retain(|_, challenge| should_keep_rotate_challenge(challenge, now));
}

pub(crate) fn map_rotate_main_error(err: RotateMainError) -> axum::response::Response {
    match err {
        RotateMainError::InitiatorMustBeBackup => api_error(
            StatusCode::FORBIDDEN,
            1003,
            "rotation initiator must be backup key",
        ),
        RotateMainError::NewBackupPubkeyRequired => api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "new_backup_pubkey is required",
        ),
        RotateMainError::NewBackupPubkeyConflict => api_error(
            StatusCode::CONFLICT,
            1007,
            "new_backup_pubkey conflicts with current keyring",
        ),
    }
}

pub(crate) fn seed_chain_keyring(state: &AppState) {
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(_) => return,
    };
    if store.chain_keyring_state.is_some() {
        return;
    }
    let main_pubkey = state
        .public_key_hex
        .read()
        .map(|v| v.clone())
        .unwrap_or_else(|_| {
            warn!("public key read lock poisoned while seeding keyring");
            String::new()
        });
    if main_pubkey.is_empty() {
        return;
    }
    let mut backup_a = resolve_backup_slot(
        "SFID_BACKUP_A_SEED_HEX",
        "SFID_BACKUP_A_PUBKEY",
        "sfid-dev-backup-a",
    );
    let mut backup_b = resolve_backup_slot(
        "SFID_BACKUP_B_SEED_HEX",
        "SFID_BACKUP_B_PUBKEY",
        "sfid-dev-backup-b",
    );
    if backup_a.pubkey.eq_ignore_ascii_case(main_pubkey.as_str()) {
        backup_a = resolve_backup_slot(
            "SFID_BACKUP_A_SEED_HEX_ALT",
            "SFID_BACKUP_A_PUBKEY_ALT",
            "sfid-dev-backup-a-alt",
        );
    }
    if backup_b.pubkey.eq_ignore_ascii_case(main_pubkey.as_str())
        || backup_b
            .pubkey
            .eq_ignore_ascii_case(backup_a.pubkey.as_str())
    {
        backup_b = resolve_backup_slot(
            "SFID_BACKUP_B_SEED_HEX_ALT",
            "SFID_BACKUP_B_PUBKEY_ALT",
            "sfid-dev-backup-b-alt",
        );
    }

    let main_seed = state
        .signing_seed_hex
        .read()
        .map(|v| v.clone())
        .unwrap_or_else(|_| {
            warn!("signing seed read lock poisoned while seeding keyring");
            SensitiveSeed::default()
        });
    if main_seed.expose_secret().is_empty() {
        return;
    }
    if let Err(err) = upsert_seed_for_pubkey(state, main_pubkey.as_str(), main_seed.expose_secret())
    {
        warn!(error = %err, "failed to upsert main key seed while seeding keyring");
    }
    if let Some(seed) = backup_a.seed_hex.as_ref() {
        if let Err(err) =
            upsert_seed_for_pubkey(state, backup_a.pubkey.as_str(), seed.expose_secret())
        {
            warn!(error = %err, "failed to upsert backup_a key seed while seeding keyring");
        }
    }
    if let Some(seed) = backup_b.seed_hex.as_ref() {
        if let Err(err) =
            upsert_seed_for_pubkey(state, backup_b.pubkey.as_str(), seed.expose_secret())
        {
            warn!(error = %err, "failed to upsert backup_b key seed while seeding keyring");
        }
    }

    store.chain_keyring_state = Some(ChainKeyringState::new(
        main_pubkey,
        backup_a.pubkey,
        backup_b.pubkey,
    ));
}

pub(crate) fn seed_key_admins(state: &AppState) {
    let mut store = match state.store.write() {
        Ok(v) => v,
        Err(_) => return,
    };
    sync_key_admin_users(&mut store);
}

pub(crate) fn reconcile_main_signer_with_keyring(state: &AppState) -> Result<bool, String> {
    let keyring_main = {
        let store = state
            .store
            .read()
            .map_err(|_| "store read lock poisoned".to_string())?;
        let Some(kr) = store.chain_keyring_state.as_ref() else {
            return Ok(false);
        };
        normalize_pubkey_for_signing(kr.main_pubkey.as_str())
    };
    let active_main = {
        let pubkey = state
            .public_key_hex
            .read()
            .map_err(|_| "public key read lock poisoned".to_string())?;
        normalize_pubkey_for_signing(pubkey.as_str())
    };
    if keyring_main.eq_ignore_ascii_case(active_main.as_str()) {
        return Ok(false);
    }

    let signer_seed = {
        let known = state
            .known_key_seeds
            .read()
            .map_err(|_| "known seeds read lock poisoned".to_string())?;
        known
            .iter()
            .find(|(pubkey, _)| pubkey.eq_ignore_ascii_case(keyring_main.as_str()))
            .map(|(_, seed)| seed.clone())
            .ok_or_else(|| "keyring main signer seed is missing".to_string())?
    };
    set_active_main_signer(state, keyring_main.as_str(), signer_seed.expose_secret())?;
    persist_runtime_state_checked(state)?;
    Ok(true)
}

pub(crate) fn sync_key_admin_users(store: &mut Store) {
    let Some(kr) = store.chain_keyring_state.as_ref().cloned() else {
        return;
    };
    let desired = vec![
        normalize_pubkey_for_signing(kr.main_pubkey.as_str()),
        normalize_pubkey_for_signing(kr.backup_a_pubkey.as_str()),
        normalize_pubkey_for_signing(kr.backup_b_pubkey.as_str()),
    ];

    let stale: Vec<String> = store
        .admin_users_by_pubkey
        .iter()
        .filter(|(_, user)| user.role == AdminRole::KeyAdmin)
        .filter(|(pubkey, _)| !desired.iter().any(|d| d.eq_ignore_ascii_case(pubkey)))
        .map(|(pubkey, _)| pubkey.clone())
        .collect();
    for pubkey in stale {
        store.admin_users_by_pubkey.remove(&pubkey);
    }

    if store.next_admin_user_id == 0 {
        store.next_admin_user_id = store
            .admin_users_by_pubkey
            .values()
            .map(|u| u.id)
            .max()
            .unwrap_or(0)
            + 1;
    }
    for pubkey in desired {
        let now = Utc::now();
        if let Some(existing_key) = store
            .admin_users_by_pubkey
            .keys()
            .find(|k| k.eq_ignore_ascii_case(pubkey.as_str()))
            .cloned()
        {
            if existing_key != pubkey {
                if let Some(mut user) = store.admin_users_by_pubkey.remove(existing_key.as_str()) {
                    user.admin_pubkey = pubkey.clone();
                    user.role = AdminRole::KeyAdmin;
                    user.status = AdminStatus::Active;
                    user.built_in = true;
                    user.created_by = "SYSTEM".to_string();
                    user.updated_at = Some(now);
                    store.admin_users_by_pubkey.insert(pubkey.clone(), user);
                }
            } else if let Some(user) = store.admin_users_by_pubkey.get_mut(pubkey.as_str()) {
                user.admin_pubkey = pubkey.clone();
                user.role = AdminRole::KeyAdmin;
                user.status = AdminStatus::Active;
                user.built_in = true;
                user.created_by = "SYSTEM".to_string();
                user.updated_at = Some(now);
            }
            continue;
        }
        let next_id = store.next_admin_user_id;
        store.next_admin_user_id = store.next_admin_user_id.saturating_add(1);
        store.admin_users_by_pubkey.insert(
            pubkey.clone(),
            AdminUser {
                id: next_id,
                admin_pubkey: pubkey.clone(),
                admin_name: String::new(),
                role: AdminRole::KeyAdmin,
                status: AdminStatus::Active,
                built_in: true,
                created_by: "SYSTEM".to_string(),
                created_at: now,
                updated_at: Some(now),
            },
        );
    }
}

fn resolve_backup_slot(
    seed_env: &str,
    pubkey_env: &str,
    fallback_label: &str,
) -> BackupSlotMaterial {
    if let Ok(seed) = std::env::var(seed_env) {
        let trimmed = seed.trim().to_string();
        if !trimmed.is_empty() {
            let pubkey = try_derive_pubkey_hex_from_seed(trimmed.as_str())
                .unwrap_or_else(|err| panic!("{seed_env} is invalid: {err}"));
            return BackupSlotMaterial {
                pubkey,
                seed_hex: Some(SensitiveSeed::from(trimmed)),
            };
        }
    }
    if let Ok(pubkey) = std::env::var(pubkey_env) {
        let trimmed = pubkey.trim().to_string();
        if !trimmed.is_empty() {
            let normalized = normalize_pubkey_for_signing(trimmed.as_str());
            return BackupSlotMaterial {
                pubkey: normalized,
                seed_hex: None,
            };
        }
    }
    if is_production_mode() {
        panic!("{seed_env} or {pubkey_env} must be configured in production mode (SFID_ENV=prod)");
    }
    let digest = Blake2b256::digest(fallback_label.as_bytes());
    BackupSlotMaterial {
        pubkey: format!("0x{}", hex::encode(digest)),
        seed_hex: None,
    }
}

fn upsert_seed_for_pubkey(state: &AppState, pubkey: &str, seed_hex: &str) -> Result<(), String> {
    let mut seeds = state
        .known_key_seeds
        .write()
        .map_err(|_| "known seeds write lock poisoned".to_string())?;
    let normalized = normalize_pubkey_for_signing(pubkey);
    let target = seeds
        .keys()
        .find(|k| k.eq_ignore_ascii_case(normalized.as_str()))
        .cloned()
        .unwrap_or(normalized);
    seeds.insert(target, SensitiveSeed::from(seed_hex.to_string()));
    Ok(())
}

fn set_active_main_signer(
    state: &AppState,
    main_pubkey: &str,
    main_seed_hex: &str,
) -> Result<(), String> {
    let normalized_main_pubkey = normalize_pubkey_for_signing(main_pubkey);
    {
        let mut seed_guard = state
            .signing_seed_hex
            .write()
            .map_err(|_| "signing seed write lock poisoned".to_string())?;
        *seed_guard = SensitiveSeed::from(main_seed_hex.to_string());
    }
    {
        let mut pubkey_guard = state
            .public_key_hex
            .write()
            .map_err(|_| "public key write lock poisoned".to_string())?;
        *pubkey_guard = normalized_main_pubkey.clone();
    }
    upsert_seed_for_pubkey(state, normalized_main_pubkey.as_str(), main_seed_hex)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct ChainRotateReceipt {
    tx_hash: String,
    block_number: u64,
}

fn normalize_chain_ws_url(input: &str) -> String {
    if let Some(rest) = input.strip_prefix("http://") {
        return format!("ws://{rest}");
    }
    if let Some(rest) = input.strip_prefix("https://") {
        return format!("wss://{rest}");
    }
    input.to_string()
}

fn resolve_chain_ws_url() -> Result<String, String> {
    let ws_url = std::env::var("SFID_CHAIN_WS_URL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::env::var("SFID_CHAIN_RPC_URL")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })
        .ok_or_else(|| "SFID_CHAIN_RPC_URL or SFID_CHAIN_WS_URL not configured".to_string())?;
    Ok(normalize_chain_ws_url(ws_url.as_str()))
}

fn parse_account_id32(pubkey: &str) -> Result<[u8; 32], String> {
    parse_sr25519_pubkey_bytes(pubkey).ok_or_else(|| "invalid sr25519 account pubkey".to_string())
}

async fn submit_rotate_sfid_keys_extrinsic(
    initiator_pubkey: &str,
    initiator_seed_hex: &str,
    new_backup_pubkey: &str,
) -> Result<ChainRotateReceipt, String> {
    let ws_url =
        resolve_chain_ws_url().map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    let client = OnlineClient::<PolkadotConfig>::from_url(ws_url)
        .await
        .map_err(|e| {
            format!("rotate_sfid_keys submit failed: chain websocket connect failed: {e}")
        })?;

    let signer_account = AccountId32(
        parse_account_id32(initiator_pubkey)
            .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?,
    );
    let new_backup_account = parse_account_id32(new_backup_pubkey)
        .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    let payload = tx(
        "SfidCodeAuth",
        "rotate_sfid_keys",
        vec![Value::from_bytes(new_backup_account)],
    );
    let mut partial_tx = client
        .tx()
        .create_partial(&payload, &signer_account, Default::default())
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: build extrinsic failed: {e}"))?;
    let signing_key = try_load_signing_key_from_seed(initiator_seed_hex)
        .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    let signature = signing_key
        .sign(signing_context(b"substrate").bytes(&partial_tx.signer_payload()))
        .to_bytes();
    let tx = partial_tx
        .sign_with_account_and_signature(&signer_account, &MultiSignature::Sr25519(signature));
    let tx_hash = format!("0x{}", hex::encode(tx.hash().as_ref()));

    let in_block = tx
        .submit_and_watch()
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: submit_and_watch failed: {e}"))?
        .wait_for_finalized()
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: wait_for_finalized failed: {e}"))?;
    in_block
        .wait_for_success()
        .await
        .map_err(|e| format!("rotate_sfid_keys included failed: {e}"))?;

    let block = client
        .blocks()
        .at(in_block.block_hash())
        .await
        .map_err(|e| format!("rotate_sfid_keys included failed: fetch block failed: {e}"))?;
    let block_number =
        block.number().to_string().parse::<u64>().map_err(|e| {
            format!("rotate_sfid_keys included failed: parse block number failed: {e}")
        })?;

    Ok(ChainRotateReceipt {
        tx_hash,
        block_number,
    })
}
