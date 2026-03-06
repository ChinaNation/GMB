pub(crate) mod chain_keyring;
pub(crate) mod chain_proof;

use self::chain_keyring::{
    derive_pubkey_hex_from_seed, verify_rotation_signature, ChainKeyringState, KeySlot,
    RotateMainError, RotateMainRequest,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use std::time::Duration as StdDuration;
use tracing::warn;
use uuid::Uuid;

use crate::*;

#[derive(Debug, Clone)]
struct BackupSlotMaterial {
    pubkey: String,
    seed_hex: Option<String>,
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
    if input.initiator_pubkey.trim() != ctx.admin_pubkey {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "initiator_pubkey must match current key admin",
        );
    }

    let now = Utc::now();
    let expire_at = now + Duration::minutes(2);
    let challenge_id = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_keyring_rotate_challenges(&mut store, now);
    let Some(current) = store.chain_keyring_state.as_ref().cloned() else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            1004,
            "chain keyring not initialized",
        );
    };

    // Challenge 阶段只确认“发起者必须是当前备用”。
    let initiator_pubkey = input.initiator_pubkey.trim().to_string();
    if !initiator_pubkey.eq_ignore_ascii_case(current.backup_a_pubkey.as_str())
        && !initiator_pubkey.eq_ignore_ascii_case(current.backup_b_pubkey.as_str())
    {
        return map_rotate_main_error(RotateMainError::InitiatorMustBeBackup);
    }

    let challenge_text = format!(
        "sfid-keyring-rotate-v1|challenge_id={}|version={}|initiator={}|nonce={}|iat={}|exp={}",
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
            initiator_pubkey: input.initiator_pubkey.trim().to_string(),
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
    if challenge.initiator_pubkey != ctx.admin_pubkey {
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
    let now = Utc::now();
    let (challenge_id, rotate_result, promoted_slot, new_main_pubkey, next_version) = {
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
        if challenge.initiator_pubkey != ctx.admin_pubkey {
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
        let initiator_seed_pubkey = derive_pubkey_hex_from_seed(initiator_seed_hex.as_str());
        if !initiator_seed_pubkey.eq_ignore_ascii_case(challenge.initiator_pubkey.as_str()) {
            return api_error(
                StatusCode::UNAUTHORIZED,
                2004,
                "server signer seed does not match initiator_pubkey",
            );
        }
        let rotate_req = RotateMainRequest {
            initiator_pubkey: challenge.initiator_pubkey.clone(),
            new_backup_pubkey: input.new_backup_pubkey.trim().to_string(),
        };
        let rotate_result = match current.rotate_main(rotate_req) {
            Ok(v) => v,
            Err(err) => return map_rotate_main_error(err),
        };
        let promoted_slot = rotate_result.promoted_slot.clone();
        let new_main_pubkey = rotate_result.state.main_pubkey.clone();
        let next_version = rotate_result.state.version;
        if let Some(challenge_mut) = store
            .keyring_rotate_challenges
            .get_mut(input.challenge_id.trim())
        {
            challenge_mut.consumed = true;
        }
        store.chain_keyring_state = Some(rotate_result.state.clone());
        sync_key_admin_users(&mut store);
        if let Err(err) = set_active_main_signer(
            &state,
            new_main_pubkey.as_str(),
            initiator_seed_hex.as_str(),
        ) {
            warn!(error = %err, "failed to switch active main signer");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "failed to switch active main signer",
            );
        }
        (
            challenge.challenge_id,
            rotate_result,
            promoted_slot,
            new_main_pubkey,
            next_version,
        )
    };
    persist_runtime_state(&state);

    let chain_submit = push_main_pubkey_to_chain(new_main_pubkey.as_str(), next_version).await;
    let (chain_tx_hash, chain_submit_ok, chain_submit_error, commit_result, response_message) =
        match chain_submit {
            Ok(tx) => (
                tx,
                true,
                None,
                "SUCCESS",
                "chain keyring rotation committed and submitted to chain".to_string(),
            ),
            Err(err) => (
                format!("submit_failed:{}", err),
                false,
                Some(err.clone()),
                "FAILED",
                format!(
                    "chain keyring rotation committed locally, but chain submit failed: {}",
                    err
                ),
            ),
        };
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
            commit_result,
            format!(
                "challenge_id={} old_main={} new_main={} promoted_slot={:?} chain_tx_hash={} chain_submit_ok={}",
                challenge_id,
                rotate_result.old_main_pubkey,
                rotate_result.state.main_pubkey,
                rotate_result.promoted_slot,
                chain_tx_hash,
                chain_submit_ok
            ),
        );
    }

    let promoted_slot = match promoted_slot {
        KeySlot::Main => "MAIN",
        KeySlot::BackupA => "BACKUP_A",
        KeySlot::BackupB => "BACKUP_B",
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: KeyringRotateCommitOutput {
            old_main_pubkey: rotate_result.old_main_pubkey,
            promoted_slot: promoted_slot.to_string(),
            chain_tx_hash,
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
    store.keyring_rotate_challenges.retain(|_, c| {
        c.expire_at > now - Duration::minutes(10) && (!c.consumed || c.expire_at > now)
    });
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
            String::new()
        });
    if main_seed.is_empty() {
        return;
    }
    if let Err(err) = upsert_seed_for_pubkey(state, main_pubkey.as_str(), main_seed.as_str()) {
        warn!(error = %err, "failed to upsert main key seed while seeding keyring");
    }
    if let Some(seed) = backup_a.seed_hex.as_ref() {
        if let Err(err) = upsert_seed_for_pubkey(state, backup_a.pubkey.as_str(), seed.as_str()) {
            warn!(error = %err, "failed to upsert backup_a key seed while seeding keyring");
        }
    }
    if let Some(seed) = backup_b.seed_hex.as_ref() {
        if let Err(err) = upsert_seed_for_pubkey(state, backup_b.pubkey.as_str(), seed.as_str()) {
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

pub(crate) fn sync_key_admin_users(store: &mut Store) {
    let Some(kr) = store.chain_keyring_state.as_ref().cloned() else {
        return;
    };
    let desired = vec![
        kr.main_pubkey.clone(),
        kr.backup_a_pubkey.clone(),
        kr.backup_b_pubkey.clone(),
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

    let mut next_id = store
        .admin_users_by_pubkey
        .values()
        .map(|u| u.id)
        .max()
        .unwrap_or(0)
        + 1;
    for pubkey in desired {
        if let Some(user) = store.admin_users_by_pubkey.get_mut(&pubkey) {
            user.role = AdminRole::KeyAdmin;
            user.status = AdminStatus::Active;
            user.built_in = true;
            user.created_by = "SYSTEM".to_string();
            continue;
        }
        store.admin_users_by_pubkey.insert(
            pubkey.clone(),
            AdminUser {
                id: next_id,
                admin_pubkey: pubkey,
                admin_name: String::new(),
                role: AdminRole::KeyAdmin,
                status: AdminStatus::Active,
                built_in: true,
                created_by: "SYSTEM".to_string(),
                created_at: Utc::now(),
            },
        );
        next_id += 1;
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
            let pubkey = derive_pubkey_hex_from_seed(trimmed.as_str());
            return BackupSlotMaterial {
                pubkey,
                seed_hex: Some(trimmed),
            };
        }
    }
    if let Ok(pubkey) = std::env::var(pubkey_env) {
        let trimmed = pubkey.trim().to_string();
        if !trimmed.is_empty() {
            let normalized = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                trimmed
            } else {
                format!("0x{}", trimmed)
            };
            return BackupSlotMaterial {
                pubkey: normalized,
                seed_hex: None,
            };
        }
    }
    let digest = blake3::hash(fallback_label.as_bytes());
    BackupSlotMaterial {
        pubkey: format!("0x{}", hex::encode(digest.as_bytes())),
        seed_hex: None,
    }
}

fn upsert_seed_for_pubkey(state: &AppState, pubkey: &str, seed_hex: &str) -> Result<(), String> {
    let mut seeds = state
        .known_key_seeds
        .write()
        .map_err(|_| "known seeds write lock poisoned".to_string())?;
    let target = seeds
        .keys()
        .find(|k| k.eq_ignore_ascii_case(pubkey))
        .cloned()
        .unwrap_or_else(|| pubkey.to_string());
    seeds.insert(target, seed_hex.to_string());
    Ok(())
}

fn set_active_main_signer(
    state: &AppState,
    main_pubkey: &str,
    main_seed_hex: &str,
) -> Result<(), String> {
    {
        let mut seed_guard = state
            .signing_seed_hex
            .write()
            .map_err(|_| "signing seed write lock poisoned".to_string())?;
        *seed_guard = main_seed_hex.to_string();
    }
    {
        let mut pubkey_guard = state
            .public_key_hex
            .write()
            .map_err(|_| "public key write lock poisoned".to_string())?;
        *pubkey_guard = main_pubkey.to_string();
    }
    upsert_seed_for_pubkey(state, main_pubkey, main_seed_hex)?;
    Ok(())
}

async fn push_main_pubkey_to_chain(new_main_pubkey: &str, version: u64) -> Result<String, String> {
    let rpc_url = std::env::var("SFID_CHAIN_RPC_URL")
        .map_err(|_| "SFID_CHAIN_RPC_URL not configured".to_string())?;
    if rpc_url.trim().is_empty() {
        return Err("SFID_CHAIN_RPC_URL is empty".to_string());
    }
    let method = std::env::var("SFID_CHAIN_RPC_METHOD")
        .unwrap_or_else(|_| "sfid_set_main_pubkey".to_string());
    if method.trim().is_empty() {
        return Err("SFID_CHAIN_RPC_METHOD is empty".to_string());
    }

    let ticket = format!("push-{}", Uuid::new_v4());
    let id = format!("sfid-{}", Uuid::new_v4());
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": [new_main_pubkey, version, ticket]
    });

    let client = reqwest::Client::builder()
        .timeout(StdDuration::from_secs(10))
        .build()
        .map_err(|e| format!("build chain rpc client failed: {}", e))?;
    let mut request = client.post(rpc_url.trim()).json(&payload);
    if let Ok(token) = std::env::var("SFID_CHAIN_RPC_TOKEN") {
        if !token.trim().is_empty() {
            request = request.bearer_auth(token.trim());
        }
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("chain rpc request failed: {}", e))?;
    let status = response.status();
    let value: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("parse chain rpc response failed: {}", e))?;
    if !status.is_success() {
        return Err(format!("chain rpc http status {}", status));
    }
    if let Some(err) = value.get("error") {
        return Err(format!("chain rpc returned error: {}", err));
    }
    if let Some(result) = value.get("result") {
        if let Some(tx_hash) = result
            .as_str()
            .map(|v| v.to_string())
            .or_else(|| {
                result
                    .get("tx_hash")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
            })
            .or_else(|| {
                result
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
            })
        {
            return Ok(tx_hash);
        }
        return Ok(format!("accepted:{}", ticket));
    }
    Err("chain rpc result field missing".to_string())
}
