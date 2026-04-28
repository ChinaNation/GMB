pub(crate) mod chain_keyring;
pub(crate) mod chain_proof;
pub(crate) mod chain_sheng_signing;
pub(crate) mod rsa_blind;
pub(crate) mod sheng_signer_cache;
pub(crate) mod signer_router;

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
use reqwest::Client as HttpClient;
use serde_json::json;
use sp_core::Pair;
use std::hash::Hasher;
use std::{sync::OnceLock, time::Duration as StdDuration};
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};
use tokio::sync::Mutex as TokioMutex;
use tracing::warn;
use twox_hash::XxHash64;
use uuid::Uuid;

use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};

use crate::*;

type Blake2b256 = Blake2b<U32>;

#[derive(Debug, Clone)]
struct BackupSlotMaterial {
    pubkey: String,
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

fn twox_128(input: &[u8]) -> [u8; 16] {
    let mut h1 = XxHash64::with_seed(0);
    h1.write(input);
    let mut h2 = XxHash64::with_seed(1);
    h2.write(input);

    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&h1.finish().to_le_bytes());
    out[8..].copy_from_slice(&h2.finish().to_le_bytes());
    out
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
    // 从 admin_users_by_pubkey 反查三个密钥管理员的名字
    let lookup_name = |pubkey: &str| -> String {
        store
            .admin_users_by_pubkey
            .iter()
            .find(|(k, _)| crate::business::pubkey::same_admin_pubkey(k.as_str(), pubkey))
            .map(|(_, u)| u.admin_name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "密钥管理员".to_string())
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: KeyringStateOutput {
            version: kr.version,
            main_pubkey: kr.main_pubkey.clone(),
            main_name: lookup_name(&kr.main_pubkey),
            backup_a_pubkey: kr.backup_a_pubkey.clone(),
            backup_a_name: lookup_name(&kr.backup_a_pubkey),
            backup_b_pubkey: kr.backup_b_pubkey.clone(),
            backup_b_name: lookup_name(&kr.backup_b_pubkey),
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
                    "initiator backup private key is not present on server; rotate_sfid_keys must be submitted by backup_1 or backup_2",
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
            if let Err(err) = validate_active_main_signer_with_keyring(&state) {
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
        if let Err(reconcile_err) = validate_active_main_signer_with_keyring(&state) {
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
        // 设置新备用管理员的姓名
        if let Some(name) = input
            .new_backup_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
        {
            let normalized_backup = normalize_pubkey_for_signing(input.new_backup_pubkey.as_str());
            if let Some(user) = store
                .admin_users_by_pubkey
                .iter_mut()
                .find(|(k, _)| k.eq_ignore_ascii_case(normalized_backup.as_str()))
                .map(|(_, v)| v)
            {
                user.admin_name = name.trim().to_string();
            }
        }
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
    let mut backup_a = resolve_backup_slot("SFID_BACKUP_A_PUBKEY", "sfid-dev-backup-a");
    let mut backup_b = resolve_backup_slot("SFID_BACKUP_B_PUBKEY", "sfid-dev-backup-b");
    if backup_a.pubkey.eq_ignore_ascii_case(main_pubkey.as_str()) {
        backup_a = resolve_backup_slot("SFID_BACKUP_A_PUBKEY_ALT", "sfid-dev-backup-a-alt");
    }
    if backup_b.pubkey.eq_ignore_ascii_case(main_pubkey.as_str())
        || backup_b
            .pubkey
            .eq_ignore_ascii_case(backup_a.pubkey.as_str())
    {
        backup_b = resolve_backup_slot("SFID_BACKUP_B_PUBKEY_ALT", "sfid-dev-backup-b-alt");
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

pub(crate) fn validate_active_main_signer_with_keyring(state: &AppState) -> Result<(), String> {
    let keyring = {
        let store = state
            .store
            .read()
            .map_err(|_| "store read lock poisoned".to_string())?;
        let Some(kr) = store.chain_keyring_state.as_ref().cloned() else {
            return Err("chain keyring not initialized".to_string());
        };
        kr
    };
    let signing_seed = state
        .signing_seed_hex
        .read()
        .map_err(|_| "signing seed read lock poisoned".to_string())?
        .clone();
    let derived_main = try_derive_pubkey_hex_from_seed(signing_seed.expose_secret())
        .map_err(|e| format!("local signing seed is invalid: {e}"))?;
    let active_main = normalize_pubkey_for_signing(
        state
            .public_key_hex
            .read()
            .map_err(|_| "public key read lock poisoned".to_string())?
            .as_str(),
    );

    if !derived_main.eq_ignore_ascii_case(active_main.as_str()) {
        return Err(format!(
            "local signing seed derives {derived_main}, but active signer state is {active_main}"
        ));
    }

    let keyring_main = normalize_pubkey_for_signing(keyring.main_pubkey.as_str());
    if derived_main.eq_ignore_ascii_case(keyring.backup_a_pubkey.as_str())
        || derived_main.eq_ignore_ascii_case(keyring.backup_b_pubkey.as_str())
    {
        return Err(format!(
            "local signing key {derived_main} matches a backup pubkey, but sfid must only hold the current chain main private key"
        ));
    }
    if !derived_main.eq_ignore_ascii_case(keyring_main.as_str()) {
        return Err(format!(
            "local signing key {derived_main} does not match chain main pubkey {keyring_main}"
        ));
    }

    Ok(())
}

pub(crate) async fn sync_chain_keyring_from_chain(state: &AppState) -> Result<bool, String> {
    let chain_state = fetch_chain_keyring_from_chain().await?;
    let mut store = state
        .store
        .write()
        .map_err(|_| "store write lock poisoned".to_string())?;
    let changed = match store.chain_keyring_state.as_ref() {
        Some(current)
            if current
                .main_pubkey
                .eq_ignore_ascii_case(chain_state.main_pubkey.as_str())
                && current
                    .backup_a_pubkey
                    .eq_ignore_ascii_case(chain_state.backup_a_pubkey.as_str())
                && current
                    .backup_b_pubkey
                    .eq_ignore_ascii_case(chain_state.backup_b_pubkey.as_str()) =>
        {
            false
        }
        _ => true,
    };
    if changed {
        // 中文注释：链上三把公钥是最终真相，本地缓存只做镜像，不允许反向覆盖链上状态。
        store.chain_keyring_state = Some(chain_state);
        sync_key_admin_users(&mut store);
    }
    Ok(changed)
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
                city: String::new(),
                encrypted_signing_privkey: None,
                signing_pubkey: None,
                signing_created_at: None,
            },
        );
    }
}

fn resolve_backup_slot(pubkey_env: &str, fallback_label: &str) -> BackupSlotMaterial {
    if let Ok(pubkey) = std::env::var(pubkey_env) {
        let trimmed = pubkey.trim().to_string();
        if !trimmed.is_empty() {
            let normalized = normalize_pubkey_for_signing(trimmed.as_str());
            return BackupSlotMaterial { pubkey: normalized };
        }
    }
    if is_production_mode() {
        panic!("{pubkey_env} must be configured in production mode (SFID_ENV=prod)");
    }
    let digest = Blake2b256::digest(fallback_label.as_bytes());
    BackupSlotMaterial {
        pubkey: format!("0x{}", hex::encode(digest)),
    }
}

fn replace_active_main_seed(state: &AppState, pubkey: &str, seed_hex: &str) -> Result<(), String> {
    let mut seeds = state
        .known_key_seeds
        .write()
        .map_err(|_| "known seeds write lock poisoned".to_string())?;
    seeds.clear();
    seeds.insert(
        normalize_pubkey_for_signing(pubkey),
        SensitiveSeed::from(seed_hex.to_string()),
    );
    Ok(())
}

fn set_active_main_signer(
    state: &AppState,
    main_pubkey: &str,
    main_seed_hex: &str,
) -> Result<(), String> {
    use zeroize::Zeroize;
    let normalized_main_pubkey = normalize_pubkey_for_signing(main_pubkey);

    // 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 步骤 10：
    // SFID MAIN 轮换时，级联用新 wrap key 重新加密所有 sheng admin 的私钥种子，
    // 然后同步更新进程内 cache 的 sfid_main_signer + wrap_key。
    let hex_trim = main_seed_hex
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    let new_seed_bytes =
        hex::decode(hex_trim).map_err(|e| format!("new main seed hex decode failed: {e}"))?;
    if new_seed_bytes.len() != 32 {
        return Err("new main seed must decode to 32 bytes".to_string());
    }
    let mut new_seed_arr = [0u8; 32];
    new_seed_arr.copy_from_slice(&new_seed_bytes);

    let new_wrap = sheng_signer_cache::derive_wrap_key(&new_seed_arr)
        .map_err(|e| format!("derive new wrap key failed: {e}"))?;
    let old_wrap = state
        .sheng_signer_cache
        .current_wrap_key()
        .map_err(|e| format!("read old wrap key failed: {e}"))?;

    // 读所有 sheng admin 的密文，在内存中完成批量重加密，然后一次性写回。
    let mut re_encrypted: Vec<(String, String)> = Vec::new();
    {
        let store = state
            .store
            .read()
            .map_err(|_| "store read lock poisoned".to_string())?;
        for (pubkey, user) in store.admin_users_by_pubkey.iter() {
            if user.role != AdminRole::ShengAdmin {
                continue;
            }
            let Some(enc) = user.encrypted_signing_privkey.as_deref() else {
                continue;
            };
            let mut plain = match sheng_signer_cache::decrypt_with_wrap(&old_wrap, enc) {
                Ok(v) => v,
                Err(e) => {
                    new_seed_arr.zeroize();
                    return Err(format!("decrypt sheng signer for {pubkey} failed: {e}"));
                }
            };
            let new_ct = match sheng_signer_cache::encrypt_with_wrap(&new_wrap, &plain) {
                Ok(v) => v,
                Err(e) => {
                    plain.zeroize();
                    new_seed_arr.zeroize();
                    return Err(format!("re-encrypt sheng signer for {pubkey} failed: {e}"));
                }
            };
            plain.zeroize();
            re_encrypted.push((pubkey.clone(), new_ct));
        }
    }

    {
        let mut store = state
            .store
            .write()
            .map_err(|_| "store write lock poisoned".to_string())?;
        for (pubkey, new_ct) in re_encrypted {
            if let Some(user) = store.admin_users_by_pubkey.get_mut(&pubkey) {
                user.encrypted_signing_privkey = Some(new_ct);
            }
        }
    }

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
    replace_active_main_seed(state, normalized_main_pubkey.as_str(), main_seed_hex)?;

    // 同步进程内 cache（sfid_main_signer Pair + wrap key）。rotate_main_seed 会
    // zeroize 传入的 seed array。
    if let Err(e) = state.sheng_signer_cache.rotate_main_seed(&mut new_seed_arr) {
        new_seed_arr.zeroize();
        return Err(format!("rotate sheng signer cache main seed failed: {e}"));
    }
    new_seed_arr.zeroize();
    Ok(())
}

/// 暴露给其他模块（如 chain::balance）按需调用 state_getStorage。
pub(crate) async fn call_chain_state_get_storage(
    storage_key_hex: &str,
) -> Result<Option<String>, String> {
    let result = chain_rpc_call("state_getStorage", json!([storage_key_hex])).await?;
    Ok(result.as_str().map(str::to_string))
}

async fn chain_rpc_call(
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let client = HttpClient::new();
    let url = crate::chain::url::chain_http_url()?;
    let response = client
        .post(url)
        .json(&json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
        .send()
        .await
        .map_err(|e| format!("chain rpc request failed: {e}"))?;
    let status = response.status();
    let payload = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("decode chain rpc response failed: {e}"))?;
    if !status.is_success() {
        return Err(format!("chain rpc returned status {status}"));
    }
    if let Some(err) = payload.get("error") {
        return Err(format!("chain rpc returned error: {err}"));
    }
    Ok(payload["result"].clone())
}

fn parse_chain_account_storage(raw: Option<&str>, field: &str) -> Result<String, String> {
    let Some(raw) = raw else {
        return Err(format!("chain storage `{field}` is empty"));
    };
    let bytes = hex::decode(raw.trim_start_matches("0x"))
        .map_err(|_| format!("chain storage `{field}` is not valid hex"))?;
    let account = match bytes.len() {
        32 => bytes,
        33 if bytes.first().copied() == Some(1) => bytes[1..33].to_vec(),
        _ => {
            return Err(format!(
                "chain storage `{field}` has unexpected AccountId encoding length {}",
                bytes.len()
            ))
        }
    };
    Ok(format!("0x{}", hex::encode(account)))
}

async fn fetch_chain_keyring_from_chain() -> Result<ChainKeyringState, String> {
    async fn fetch_pubkey(storage_name: &str) -> Result<String, String> {
        let storage_key = format!(
            "0x{}{}",
            hex::encode(twox_128(b"SfidCodeAuth")),
            hex::encode(twox_128(storage_name.as_bytes()))
        );
        let raw = chain_rpc_call("state_getStorage", json!([storage_key])).await?;
        parse_chain_account_storage(raw.as_str(), storage_name)
    }

    let main_pubkey = fetch_pubkey("SfidMainAccount").await?;
    let backup_a_pubkey = fetch_pubkey("SfidBackupAccount1").await?;
    let backup_b_pubkey = fetch_pubkey("SfidBackupAccount2").await?;
    Ok(ChainKeyringState::new(
        main_pubkey,
        backup_a_pubkey,
        backup_b_pubkey,
    ))
}

#[derive(Debug, Clone)]
struct ChainRotateReceipt {
    tx_hash: String,
    block_number: u64,
}

fn parse_account_id32(pubkey: &str) -> Result<[u8; 32], String> {
    parse_sr25519_pubkey_bytes(pubkey).ok_or_else(|| "invalid sr25519 account pubkey".to_string())
}

async fn submit_rotate_sfid_keys_extrinsic(
    initiator_pubkey: &str,
    initiator_seed_hex: &str,
    new_backup_pubkey: &str,
) -> Result<ChainRotateReceipt, String> {
    // 中文注释：与 institutions.rs 中 register_sfid_institution 相同，
    // PoW 链下 subxt 0.43 默认行为(从 finalized 读 nonce/取 era birth/等 finalize)全部踩坑，
    // 必须做三件事：① legacy RPC 取 best+pool 视图 nonce ② immortal era ③ 只等 InBestBlock。
    // 详见 ADR `04-decisions/sfid/2026-04-07-subxt-0.43-pow-chain-quirks.md`。
    let ws_url = crate::chain::url::chain_ws_url()
        .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    let client = OnlineClient::<PolkadotConfig>::from_url(ws_url.clone())
        .await
        .map_err(|e| {
            format!("rotate_sfid_keys submit failed: chain websocket connect failed: {e}")
        })?;
    // ① legacy RPC client，用于显式取 nonce
    let rpc_client = subxt::backend::rpc::RpcClient::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: legacy rpc connect failed: {e}"))?;
    let legacy_rpc = subxt::backend::legacy::LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    let signer_account = AccountId32(
        parse_account_id32(initiator_pubkey)
            .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?,
    );
    let chain_nonce = legacy_rpc
        .system_account_next_index(&signer_account)
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: fetch account nonce failed: {e}"))?;
    let new_backup_account = parse_account_id32(new_backup_pubkey)
        .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    let payload = tx(
        "SfidCodeAuth",
        "rotate_sfid_keys",
        vec![Value::from_bytes(new_backup_account)],
    );
    // ② immortal + 显式 nonce
    let params = subxt::config::DefaultExtrinsicParamsBuilder::<PolkadotConfig>::new()
        .immortal()
        .nonce(chain_nonce)
        .build();
    let mut partial_tx = client
        .tx()
        .create_partial(&payload, &signer_account, params)
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: build extrinsic failed: {e}"))?;
    let signing_key = try_load_signing_key_from_seed(initiator_seed_hex)
        .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    let signature = signing_key.sign(&partial_tx.signer_payload()).0;
    let tx = partial_tx
        .sign_with_account_and_signature(&signer_account, &MultiSignature::Sr25519(signature));
    let tx_hash = format!("0x{}", hex::encode(tx.hash().as_ref()));

    let mut submitted = tx
        .submit_and_watch()
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: submit_and_watch failed: {e}"))?;
    // ③ 只等 InBestBlock
    let in_block = tokio::time::timeout(std::time::Duration::from_secs(120), async {
        use subxt::tx::TxStatus;
        loop {
            match submitted.next().await {
                Some(Ok(TxStatus::InBestBlock(b))) => return Ok::<_, String>(b),
                Some(Ok(TxStatus::InFinalizedBlock(b))) => return Ok(b),
                Some(Ok(TxStatus::Error { message }))
                | Some(Ok(TxStatus::Invalid { message }))
                | Some(Ok(TxStatus::Dropped { message })) => {
                    return Err(format!("tx pool reported: {message}"));
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Err(format!("tx watch stream error: {e}")),
                None => return Err("tx watch stream closed unexpectedly".to_string()),
            }
        }
    })
    .await
    .map_err(|_| {
        "rotate_sfid_keys submit failed: timed out waiting for in-block inclusion".to_string()
    })?
    .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
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

/// 中文注释：省登录管理员登录成功后，确保本省签名私钥在进程内缓存就绪。
///
/// 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 步骤 7。
///
/// 行为：
/// 1. 若数据库里该管理员已有 `encrypted_signing_privkey` → 解密 → 构造 Pair → 载入
///    cache。
/// 2. 否则（首次登录）→ 随机生成 32 字节 seed → 派生 Pair → 加密持久化 → 推链
///    `SfidCodeAuth::set_sheng_signing_pubkey(province, Some(pubkey))` → 载入 cache。
///
/// 失败时返回 Err，登录流程应记录 warn 但继续颁发 access_token（等下次登录重试
/// bootstrap，或由运维手工处理）。
pub(crate) async fn bootstrap_sheng_signer(
    state: &AppState,
    admin_pubkey: &str,
    province: &str,
) -> Result<(), String> {
    use self::chain_sheng_signing::submit_set_sheng_signing_pubkey_with_client;
    use sp_core::sr25519;
    use subxt::backend::legacy::LegacyRpcMethods;
    use subxt::{OnlineClient, PolkadotConfig};
    use zeroize::Zeroizing;

    // 1. 先尝试读取现有密文,顺便校验 admin 存在。
    // 中文注释:pre-check admin_users_by_pubkey 是为了 Issue 1 幂等性修复:
    // 如果 admin 已经不存在(比如 session token 过期瞬间刚好被 replace_sheng_admin
    // 清掉),在推链消耗 nonce 之前就退出,避免"链上已写新 pubkey 但本地 Store
    // 找不到 admin 落盘"的不可恢复窗口。
    let existing_encrypted: Option<String> = {
        let store = state.store.read().map_err(|e| format!("store read: {e}"))?;
        let user = store
            .admin_users_by_pubkey
            .get(admin_pubkey)
            .ok_or_else(|| "admin user not found (bootstrap pre-check)".to_string())?;
        user.encrypted_signing_privkey.clone()
    };

    if let Some(enc) = existing_encrypted {
        // 中文注释:Issue 2 修复 —— seed 用 Zeroizing 包装,离开作用域自动 zeroize,
        // 即使 panic 或 task 被取消也不会残留明文种子。
        let seed_arr = Zeroizing::new(
            state
                .sheng_signer_cache
                .decrypt_seed(enc.as_str())
                .map_err(|e| format!("decrypt existing sheng signer failed: {e}"))?,
        );
        let pair = sr25519::Pair::from_seed(&*seed_arr);
        state
            .sheng_signer_cache
            .load_province(province.to_string(), pair);
        tracing::info!(
            province,
            "sheng signer cache populated from existing ciphertext"
        );
        return Ok(());
    }

    // 2. 首次登录：生成新 keypair。
    // 中文注释:Issue 2 修复 —— 用 Zeroizing 包装,所有路径(包括 .await 期间被
    // cancel、panic 展开、提前 return)都自动 zeroize 明文种子。
    let mut seed_arr: Zeroizing<[u8; 32]> = Zeroizing::new([0u8; 32]);
    getrandom::getrandom(seed_arr.as_mut_slice()).map_err(|e| format!("rng: {e}"))?;
    let pair = sr25519::Pair::from_seed(&*seed_arr);
    let new_pubkey: [u8; 32] = pair.public().0;
    let encrypted = state
        .sheng_signer_cache
        .encrypt_seed(&*seed_arr)
        .map_err(|e| format!("encrypt new sheng signer failed: {e}"))?;

    // 3. 推链 set_sheng_signing_pubkey(province, Some(new_pubkey))。
    let ws_url =
        crate::chain::url::chain_ws_url().map_err(|e| format!("resolve ws url failed: {e}"))?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.clone())
        .await
        .map_err(|e| format!("chain connect failed: {e}"))?;
    let rpc_client = subxt::backend::rpc::RpcClient::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("legacy rpc connect failed: {e}"))?;
    let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);
    let main_pair = state.sheng_signer_cache.sfid_main_signer();
    let tx_hash = submit_set_sheng_signing_pubkey_with_client(
        &client,
        &legacy_rpc,
        &main_pair,
        province,
        Some(new_pubkey),
    )
    .await
    .map_err(|e| format!("submit set_sheng_signing_pubkey failed: {e}"))?;

    // 4. 写 Store（drop guard 触发持久化）。
    // 中文注释:Issue 1 幂等性修复 —— 步骤 1 已经 pre-check admin 存在;正常情况下
    // 这里 get_mut 不会 None。若真的 None(极端 race:pre-check 后到这里的几毫秒
    // 内 admin 被删),我们记一条 ERROR 日志但不 panic,上游交易已成功,下次 admin
    // 恢复后会走 "already encrypted" 分支处理。
    {
        let mut store = state
            .store
            .write()
            .map_err(|e| format!("store write: {e}"))?;
        match store.admin_users_by_pubkey.get_mut(admin_pubkey) {
            Some(user) => {
                user.encrypted_signing_privkey = Some(encrypted);
                user.signing_pubkey = Some(hex::encode(new_pubkey));
                user.signing_created_at = Some(chrono::Utc::now());
            }
            None => {
                tracing::error!(
                    province,
                    admin_pubkey,
                    tx_hash = %tx_hash,
                    "admin user disappeared between pre-check and store write; chain already committed new signing pubkey, manual reconciliation required"
                );
                return Err(
                    "admin user removed during bootstrap; chain tx committed but store write failed"
                        .to_string(),
                );
            }
        }
    }

    // 5. 载入 cache。
    state
        .sheng_signer_cache
        .load_province(province.to_string(), pair);
    tracing::info!(
        province,
        tx_hash = %tx_hash,
        "new sheng signer generated, persisted and on-chain registered"
    );
    Ok(())
}
