use super::*;
use crate::login::AdminSession;
use axum::{
    body::to_bytes,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::Response,
};
use chrono::Duration;
use sp_core::Pair;

fn hex_seed(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

fn rotate_commit_message(challenge_text: &str, new_backup_pubkey: &str) -> String {
    let trimmed = new_backup_pubkey.trim();
    let no_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    format!(
        "{challenge_text}|phase=commit|new_backup=0x{}",
        no_prefix.to_ascii_lowercase()
    )
}

fn build_test_state() -> AppState {
    std::env::set_var("SFID_CHAIN_TOKEN", "test-chain-token");
    std::env::set_var(
        "SFID_CHAIN_SIGNING_SECRET",
        "test-chain-signing-secret-at-least-32",
    );
    std::env::set_var("SFID_PUBLIC_SEARCH_TOKEN", "test-public-search-token");
    let main_seed = hex_seed(0x11);
    let main_key = key_admins::chain_keyring::load_signing_key_from_seed(main_seed.as_str());
    let public_key_hex = format!("0x{}", hex::encode(main_key.public().0));
    let mut known_key_seeds = HashMap::new();
    known_key_seeds.insert(
        public_key_hex.clone(),
        SensitiveSeed::from(main_seed.clone()),
    );
    let state = AppState {
        store: StoreHandle::in_memory(),
        signing_seed_hex: Arc::new(RwLock::new(SensitiveSeed::from(main_seed))),
        known_key_seeds: Arc::new(RwLock::new(known_key_seeds)),
        rate_limit_redis: Arc::new(
            redis::Client::open("redis://127.0.0.1/").expect("test redis url should be valid"),
        ),
        cpms_register_inflight: Arc::new(Mutex::new(HashMap::new())),
        key_id: "sfid-master-v1".to_string(),
        key_version: "v1".to_string(),
        key_alg: "sr25519".to_string(),
        public_key_hex: Arc::new(RwLock::new(public_key_hex)),
        sheng_signer_cache: {
            let seed_bytes = hex::decode(hex_seed(0x11)).expect("test hex seed");
            let mut seed_arr = [0u8; 32];
            seed_arr.copy_from_slice(&seed_bytes);
            Arc::new(
                key_admins::sheng_signer_cache::ShengSignerCache::new_from_seed(&mut seed_arr)
                    .expect("test sheng signer cache init"),
            )
        },
        // 任务卡 `20260410-sfid-store-shard-by-province` Phase 2 Day 2:
        // 测试态使用 mock backend,避免依赖真实 Postgres。
        sharded_store: {
            #[cfg(test)]
            {
                Arc::new(store_shards::ShardedStore::new(
                    Arc::new(store_shards::backend::MockShardBackend::new())
                        as Arc<dyn store_shards::backend::ShardBackend>,
                    false,
                ))
            }
            #[cfg(not(test))]
            {
                unreachable!("main_tests.rs only compiles in test cfg")
            }
        },
    };
    seed_sheng_admins(&state);
    key_admins::seed_chain_keyring(&state);
    key_admins::seed_key_admins(&state);
    seed_demo_record(&state);
    state
}

async fn parse_json(resp: Response) -> serde_json::Value {
    let bytes = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("response body bytes");
    serde_json::from_slice(&bytes).expect("json response")
}

fn sign_with_test_sr25519(seed_byte: u8, message: &str) -> (String, String) {
    let seed = hex_seed(seed_byte);
    let keypair = key_admins::chain_keyring::load_signing_key_from_seed(seed.as_str());
    let sig = keypair.sign(message.as_bytes());
    (
        format!("0x{}", hex::encode(keypair.public().0)),
        format!("0x{}", hex::encode(sig.0)),
    )
}

fn sign_rotation_challenge(seed_hex: &str, message: &str) -> String {
    let keypair = key_admins::chain_keyring::load_signing_key_from_seed(seed_hex);
    let sig = keypair.sign(message.as_bytes());
    format!("0x{}", hex::encode(sig.0))
}

fn setup_rotation_test_state() -> (AppState, HeaderMap, String, String) {
    let state = build_test_state();
    let main_seed = hex_seed(0x21);
    let backup_a_seed = hex_seed(0x22);
    let backup_b_seed = hex_seed(0x23);
    let new_backup_seed = hex_seed(0x24);
    let main_pubkey = key_admins::chain_keyring::derive_pubkey_hex_from_seed(main_seed.as_str());
    let backup_a_pubkey =
        key_admins::chain_keyring::derive_pubkey_hex_from_seed(backup_a_seed.as_str());
    let backup_b_pubkey =
        key_admins::chain_keyring::derive_pubkey_hex_from_seed(backup_b_seed.as_str());

    {
        let mut seed_guard = state
            .signing_seed_hex
            .write()
            .expect("signing seed write lock poisoned");
        *seed_guard = SensitiveSeed::from(main_seed.clone());
    }
    {
        let mut pubkey_guard = state
            .public_key_hex
            .write()
            .expect("public key write lock poisoned");
        *pubkey_guard = main_pubkey.clone();
    }
    {
        let mut known = state
            .known_key_seeds
            .write()
            .expect("known seeds write lock poisoned");
        known.insert(main_pubkey.clone(), SensitiveSeed::from(main_seed.clone()));
        known.insert(
            backup_a_pubkey.clone(),
            SensitiveSeed::from(backup_a_seed.clone()),
        );
        known.insert(
            backup_b_pubkey.clone(),
            SensitiveSeed::from(backup_b_seed.clone()),
        );
        known.insert(
            key_admins::chain_keyring::derive_pubkey_hex_from_seed(new_backup_seed.as_str()),
            SensitiveSeed::from(new_backup_seed.clone()),
        );
    }
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        store.chain_keyring_state = Some(ChainKeyringState::new(
            main_pubkey,
            backup_a_pubkey.clone(),
            backup_b_pubkey,
        ));
        key_admins::sync_key_admin_users(&mut store);
        store.admin_sessions.insert(
            "tok-rotate".to_string(),
            AdminSession {
                token: "tok-rotate".to_string(),
                admin_pubkey: backup_a_pubkey.clone(),
                role: AdminRole::KeyAdmin,
                expire_at: Utc::now() + Duration::hours(1),
                last_active_at: Utc::now(),
            },
        );
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_str("Bearer tok-rotate").expect("header value"),
    );
    (state, headers, backup_a_seed, new_backup_seed)
}

// 说明：原 `setup_bind_confirm_test_state` 辅助函数及对应的 `bind_confirm_*` 测试
// 依赖已废弃的 `operate::binding::admin_bind_confirm` + `AdminBindInput` + 基于
// `archive_index + qr_id` 的旧绑定流程。当前绑定流程已重构为 `citizen_bind`
// （challenge + signature 模式），整个旧测试路径不再可复用，已整体删除。
// 新流程的测试请在 `operate::binding::citizen_bind` 的调用侧按新入参重写。

#[tokio::test]
async fn keyring_rotate_challenge_rejects_main_initiator() {
    let (state, headers, _, _) = setup_rotation_test_state();
    let main_pubkey = state
        .public_key_hex
        .read()
        .expect("public key read lock poisoned")
        .clone();
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        let session = store
            .admin_sessions
            .get_mut("tok-rotate")
            .expect("rotation session should exist");
        session.admin_pubkey = main_pubkey.clone();
    }

    let challenge_resp = key_admins::admin_chain_keyring_rotate_challenge(
        State(state),
        headers,
        Json(KeyringRotateChallengeInput {
            initiator_pubkey: main_pubkey,
        }),
    )
    .await
    .into_response();
    assert_eq!(challenge_resp.status(), StatusCode::FORBIDDEN);
    let body = parse_json(challenge_resp).await;
    assert_eq!(
        body["message"].as_str(),
        Some("rotation initiator must be backup key")
    );
}

// 说明：原 `bind_confirm_requires_pre_generated_sfid` 与
// `bind_confirm_consumes_pre_generated_sfid_without_fallback` 两个测试已整体删除。
// 原因：对应的 `admin_bind_confirm(AdminBindInput { account_pubkey, archive_index, qr_id })`
// 接口已被 `citizen_bind(CitizenBindInput { user_address, challenge_id, signature })`
// 新流程替代，旧测试的业务前提与 API 形态均已不存在，无法简单修补。
// 新流程的对应测试请在业务路径稳定后按 challenge + signature 模式补回。

#[tokio::test]
async fn keyring_rotate_commit_requires_prior_verify() {
    let (state, headers, backup_a_seed, new_backup_seed) = setup_rotation_test_state();
    let challenge_resp = key_admins::admin_chain_keyring_rotate_challenge(
        State(state.clone()),
        headers.clone(),
        Json(KeyringRotateChallengeInput {
            initiator_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                backup_a_seed.as_str(),
            ),
        }),
    )
    .await
    .into_response();
    assert_eq!(challenge_resp.status(), StatusCode::OK);
    let challenge_json = parse_json(challenge_resp).await;
    let challenge_id = challenge_json["data"]["challenge_id"]
        .as_str()
        .expect("challenge_id")
        .to_string();
    let challenge_text = challenge_json["data"]["challenge_text"]
        .as_str()
        .expect("challenge_text")
        .to_string();

    let commit_resp = key_admins::admin_chain_keyring_rotate_commit(
        State(state),
        headers,
        Json(KeyringRotateCommitInput {
            challenge_id,
            signature: sign_rotation_challenge(backup_a_seed.as_str(), challenge_text.as_str()),
            new_backup_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                new_backup_seed.as_str(),
            ),
        }),
    )
    .await
    .into_response();
    assert_eq!(commit_resp.status(), StatusCode::CONFLICT);
    let body = parse_json(commit_resp).await;
    assert_eq!(
        body["message"].as_str(),
        Some("rotation challenge not verified")
    );
}

#[tokio::test]
async fn keyring_rotate_commit_reports_chain_submit_failure_and_keeps_local_state_unchanged() {
    let previous_ws_url = std::env::var("SFID_CHAIN_WS_URL").ok();
    std::env::remove_var("SFID_CHAIN_WS_URL");
    let (state, headers, backup_a_seed, new_backup_seed) = setup_rotation_test_state();
    let challenge_resp = key_admins::admin_chain_keyring_rotate_challenge(
        State(state.clone()),
        headers.clone(),
        Json(KeyringRotateChallengeInput {
            initiator_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                backup_a_seed.as_str(),
            ),
        }),
    )
    .await
    .into_response();
    assert_eq!(challenge_resp.status(), StatusCode::OK);
    let challenge_json = parse_json(challenge_resp).await;
    let challenge_id = challenge_json["data"]["challenge_id"]
        .as_str()
        .expect("challenge_id")
        .to_string();
    let challenge_text = challenge_json["data"]["challenge_text"]
        .as_str()
        .expect("challenge_text")
        .to_string();
    let verify_signature = sign_rotation_challenge(backup_a_seed.as_str(), challenge_text.as_str());

    let verify_resp = key_admins::admin_chain_keyring_rotate_verify(
        State(state.clone()),
        headers.clone(),
        Json(KeyringRotateVerifyInput {
            challenge_id: challenge_id.clone(),
            signature: verify_signature,
        }),
    )
    .await
    .into_response();
    assert_eq!(verify_resp.status(), StatusCode::OK);

    let new_backup_pubkey =
        key_admins::chain_keyring::derive_pubkey_hex_from_seed(new_backup_seed.as_str());
    let backup_a_pubkey =
        key_admins::chain_keyring::derive_pubkey_hex_from_seed(backup_a_seed.as_str());
    let commit_message = rotate_commit_message(challenge_text.as_str(), new_backup_pubkey.as_str());
    let commit_signature = sign_rotation_challenge(backup_a_seed.as_str(), commit_message.as_str());
    let commit_resp = key_admins::admin_chain_keyring_rotate_commit(
        State(state.clone()),
        headers,
        Json(KeyringRotateCommitInput {
            challenge_id,
            signature: commit_signature,
            new_backup_pubkey: new_backup_pubkey.clone(),
        }),
    )
    .await
    .into_response();

    if let Some(value) = previous_ws_url {
        std::env::set_var("SFID_CHAIN_WS_URL", value);
    }

    assert_eq!(commit_resp.status(), StatusCode::OK);
    let body = parse_json(commit_resp).await;
    assert_eq!(body["data"]["chain_submit_ok"].as_bool(), Some(false));
    assert!(body["data"]["block_number"].is_null());
    assert_eq!(
        body["data"]["main_pubkey"].as_str(),
        body["data"]["old_main_pubkey"].as_str()
    );
    assert_ne!(
        body["data"]["main_pubkey"].as_str(),
        Some(backup_a_pubkey.as_str())
    );
    let persisted_main = {
        let store = state.store.read().expect("store read lock poisoned");
        store
            .chain_keyring_state
            .as_ref()
            .expect("keyring state")
            .main_pubkey
            .clone()
    };
    assert_eq!(
        body["data"]["main_pubkey"].as_str(),
        Some(persisted_main.as_str())
    );
}

#[tokio::test]
async fn keyring_rotate_verify_rejects_expired_challenge() {
    let (state, headers, backup_a_seed, _) = setup_rotation_test_state();
    let challenge_resp = key_admins::admin_chain_keyring_rotate_challenge(
        State(state.clone()),
        headers.clone(),
        Json(KeyringRotateChallengeInput {
            initiator_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                backup_a_seed.as_str(),
            ),
        }),
    )
    .await
    .into_response();
    assert_eq!(challenge_resp.status(), StatusCode::OK);
    let challenge_json = parse_json(challenge_resp).await;
    let challenge_id = challenge_json["data"]["challenge_id"]
        .as_str()
        .expect("challenge_id")
        .to_string();
    let challenge_text = challenge_json["data"]["challenge_text"]
        .as_str()
        .expect("challenge_text")
        .to_string();
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        let entry = store
            .keyring_rotate_challenges
            .get_mut(&challenge_id)
            .expect("challenge exists");
        entry.expire_at = Utc::now() - Duration::minutes(3);
    }

    let verify_resp = key_admins::admin_chain_keyring_rotate_verify(
        State(state),
        headers,
        Json(KeyringRotateVerifyInput {
            challenge_id,
            signature: sign_rotation_challenge(backup_a_seed.as_str(), challenge_text.as_str()),
        }),
    )
    .await
    .into_response();
    assert_eq!(verify_resp.status(), StatusCode::UNAUTHORIZED);
    let body = parse_json(verify_resp).await;
    assert_eq!(body["message"].as_str(), Some("rotation challenge expired"));
}

#[tokio::test]
async fn keyring_rotate_commit_rejects_reused_verify_signature() {
    let (state, headers, backup_a_seed, new_backup_seed) = setup_rotation_test_state();
    let challenge_resp = key_admins::admin_chain_keyring_rotate_challenge(
        State(state.clone()),
        headers.clone(),
        Json(KeyringRotateChallengeInput {
            initiator_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                backup_a_seed.as_str(),
            ),
        }),
    )
    .await
    .into_response();
    assert_eq!(challenge_resp.status(), StatusCode::OK);
    let challenge_json = parse_json(challenge_resp).await;
    let challenge_id = challenge_json["data"]["challenge_id"]
        .as_str()
        .expect("challenge_id")
        .to_string();
    let challenge_text = challenge_json["data"]["challenge_text"]
        .as_str()
        .expect("challenge_text")
        .to_string();

    let verify_signature = sign_rotation_challenge(backup_a_seed.as_str(), challenge_text.as_str());
    let verify_resp = key_admins::admin_chain_keyring_rotate_verify(
        State(state.clone()),
        headers.clone(),
        Json(KeyringRotateVerifyInput {
            challenge_id: challenge_id.clone(),
            signature: verify_signature.clone(),
        }),
    )
    .await
    .into_response();
    assert_eq!(verify_resp.status(), StatusCode::OK);

    let commit_resp = key_admins::admin_chain_keyring_rotate_commit(
        State(state),
        headers,
        Json(KeyringRotateCommitInput {
            challenge_id,
            signature: verify_signature,
            new_backup_pubkey: key_admins::chain_keyring::derive_pubkey_hex_from_seed(
                new_backup_seed.as_str(),
            ),
        }),
    )
    .await
    .into_response();
    assert_eq!(commit_resp.status(), StatusCode::UNAUTHORIZED);
    let body = parse_json(commit_resp).await;
    assert_eq!(
        body["message"].as_str(),
        Some("rotation signature verify failed")
    );
}

#[tokio::test]
async fn qr_login_non_admin_should_be_rejected() {
    let state = build_test_state();

    let challenge_resp = login::admin_auth_qr_challenge(
        State(state.clone()),
        Json(login::AdminQrChallengeInput {
            origin: Some("http://127.0.0.1:5179".to_string()),
            domain: None,
            session_id: Some("sid-query-test".to_string()),
        }),
    )
    .await
    .into_response();
    assert_eq!(challenge_resp.status(), StatusCode::OK);
    let challenge_json = parse_json(challenge_resp).await;
    let challenge_id = challenge_json["data"]["challenge_id"]
        .as_str()
        .expect("challenge_id")
        .to_string();
    let session_id = challenge_json["data"]["session_id"]
        .as_str()
        .expect("session_id")
        .to_string();
    let challenge_payload = challenge_json["data"]["challenge_payload"]
        .as_str()
        .expect("challenge_payload")
        .to_string();

    let (query_pubkey, signature) = sign_with_test_sr25519(11, &challenge_payload);
    let complete_resp = login::admin_auth_qr_complete(
        State(state.clone()),
        Json(login::AdminQrCompleteInput {
            challenge_id: challenge_id.clone(),
            session_id: Some(session_id.clone()),
            admin_pubkey: query_pubkey,
            signer_pubkey: None,
            signature,
        }),
    )
    .await
    .into_response();
    assert_eq!(complete_resp.status(), StatusCode::FORBIDDEN);
    let body = parse_json(complete_resp).await;
    assert_eq!(body["message"].as_str(), Some("admin not found"));
}

#[tokio::test]
async fn qr_login_sheng_admin_keeps_write_permission() {
    let state = build_test_state();

    let challenge_resp = login::admin_auth_qr_challenge(
        State(state.clone()),
        Json(login::AdminQrChallengeInput {
            origin: Some("http://127.0.0.1:5179".to_string()),
            domain: None,
            session_id: Some("sid-admin-test".to_string()),
        }),
    )
    .await
    .into_response();
    assert_eq!(challenge_resp.status(), StatusCode::OK);
    let challenge_json = parse_json(challenge_resp).await;
    let challenge_id = challenge_json["data"]["challenge_id"]
        .as_str()
        .expect("challenge_id")
        .to_string();
    let session_id = challenge_json["data"]["session_id"]
        .as_str()
        .expect("session_id")
        .to_string();
    let challenge_payload = challenge_json["data"]["challenge_payload"]
        .as_str()
        .expect("challenge_payload")
        .to_string();

    let (admin_pubkey, signature) = sign_with_test_sr25519(22, &challenge_payload);
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        store.admin_users_by_pubkey.insert(
            admin_pubkey.clone(),
            AdminUser {
                id: 999,
                admin_pubkey: admin_pubkey.clone(),
                admin_name: String::new(),
                role: AdminRole::ShengAdmin,
                status: AdminStatus::Active,
                built_in: false,
                created_by: "TEST".to_string(),
                created_at: Utc::now(),
                updated_at: None,
                city: String::new(),
                encrypted_signing_privkey: None,
                signing_pubkey: None,
                signing_created_at: None,
            },
        );
    }
    let complete_resp = login::admin_auth_qr_complete(
        State(state.clone()),
        Json(login::AdminQrCompleteInput {
            challenge_id: challenge_id.clone(),
            session_id: Some(session_id.clone()),
            admin_pubkey,
            signer_pubkey: None,
            signature,
        }),
    )
    .await
    .into_response();
    assert_eq!(complete_resp.status(), StatusCode::OK);

    let result_resp = login::admin_auth_qr_result(
        State(state.clone()),
        Query(login::AdminQrResultQuery {
            challenge_id,
            session_id,
        }),
    )
    .await
    .into_response();
    assert_eq!(result_resp.status(), StatusCode::OK);
    let result_json = parse_json(result_resp).await;
    let token = result_json["data"]["access_token"]
        .as_str()
        .expect("access token");
    assert_eq!(
        result_json["data"]["admin"]["role"].as_str(),
        Some("SHENG_ADMIN")
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", token)).expect("header value"),
    );
    assert!(require_admin_write(&state, &headers).is_ok());
}

#[tokio::test]
async fn qr_login_rejects_signer_admin_mismatch() {
    let state = build_test_state();

    let challenge_resp = login::admin_auth_qr_challenge(
        State(state.clone()),
        Json(login::AdminQrChallengeInput {
            origin: Some("http://127.0.0.1:5179".to_string()),
            domain: None,
            session_id: Some("sid-mismatch-test".to_string()),
        }),
    )
    .await
    .into_response();
    assert_eq!(challenge_resp.status(), StatusCode::OK);
    let challenge_json = parse_json(challenge_resp).await;
    let challenge_id = challenge_json["data"]["challenge_id"]
        .as_str()
        .expect("challenge_id")
        .to_string();
    let session_id = challenge_json["data"]["session_id"]
        .as_str()
        .expect("session_id")
        .to_string();
    let challenge_payload = challenge_json["data"]["challenge_payload"]
        .as_str()
        .expect("challenge_payload")
        .to_string();

    let (login_pubkey, _) = sign_with_test_sr25519(31, &challenge_payload);
    let (signer_pubkey, signer_signature) = sign_with_test_sr25519(32, &challenge_payload);
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        store.admin_users_by_pubkey.insert(
            login_pubkey.clone(),
            AdminUser {
                id: 2001,
                admin_pubkey: login_pubkey.clone(),
                admin_name: String::new(),
                role: AdminRole::ShengAdmin,
                status: AdminStatus::Active,
                built_in: false,
                created_by: "TEST".to_string(),
                created_at: Utc::now(),
                updated_at: None,
                city: String::new(),
                encrypted_signing_privkey: None,
                signing_pubkey: None,
                signing_created_at: None,
            },
        );
        store.admin_users_by_pubkey.insert(
            signer_pubkey.clone(),
            AdminUser {
                id: 2002,
                admin_pubkey: signer_pubkey.clone(),
                admin_name: String::new(),
                role: AdminRole::ShiAdmin,
                status: AdminStatus::Active,
                built_in: false,
                created_by: "TEST".to_string(),
                created_at: Utc::now(),
                updated_at: None,
                city: String::new(),
                encrypted_signing_privkey: None,
                signing_pubkey: None,
                signing_created_at: None,
            },
        );
    }

    let complete_resp = login::admin_auth_qr_complete(
        State(state),
        Json(login::AdminQrCompleteInput {
            challenge_id,
            session_id: Some(session_id),
            admin_pubkey: login_pubkey,
            signer_pubkey: Some(signer_pubkey),
            signature: signer_signature,
        }),
    )
    .await
    .into_response();
    assert_eq!(complete_resp.status(), StatusCode::FORBIDDEN);
    let body = parse_json(complete_resp).await;
    assert_eq!(
        body["message"].as_str(),
        Some("signer_pubkey must match admin_pubkey")
    );
}

#[test]
fn require_admin_any_should_allow_all_three_roles() {
    let state = build_test_state();
    let (institution_pubkey, key_pubkey) = {
        let store = state.store.read().expect("store read lock poisoned");
        let institution_pubkey = store
            .admin_users_by_pubkey
            .values()
            .find(|u| u.role == AdminRole::ShengAdmin)
            .map(|u| u.admin_pubkey.clone())
            .expect("institution admin exists");
        let key_pubkey = store
            .admin_users_by_pubkey
            .values()
            .find(|u| u.role == AdminRole::KeyAdmin)
            .map(|u| u.admin_pubkey.clone())
            .expect("key admin exists");
        (institution_pubkey, key_pubkey)
    };
    let system_pubkey = "0xTEST_SHI_ADMIN".to_string();
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        store.admin_users_by_pubkey.insert(
            system_pubkey.clone(),
            AdminUser {
                id: 9_999,
                admin_pubkey: system_pubkey.clone(),
                admin_name: "测试系统管理员".to_string(),
                role: AdminRole::ShiAdmin,
                status: AdminStatus::Active,
                built_in: false,
                created_by: institution_pubkey.clone(),
                created_at: Utc::now(),
                updated_at: None,
                city: String::new(),
                encrypted_signing_privkey: None,
                signing_pubkey: None,
                signing_created_at: None,
            },
        );
        store.admin_sessions.insert(
            "tok-institution".to_string(),
            AdminSession {
                token: "tok-institution".to_string(),
                admin_pubkey: institution_pubkey.clone(),
                role: AdminRole::ShengAdmin,
                expire_at: Utc::now() + Duration::hours(1),
                last_active_at: Utc::now(),
            },
        );
        store.admin_sessions.insert(
            "tok-system".to_string(),
            AdminSession {
                token: "tok-system".to_string(),
                admin_pubkey: system_pubkey.clone(),
                role: AdminRole::ShiAdmin,
                expire_at: Utc::now() + Duration::hours(1),
                last_active_at: Utc::now(),
            },
        );
        store.admin_sessions.insert(
            "tok-key".to_string(),
            AdminSession {
                token: "tok-key".to_string(),
                admin_pubkey: key_pubkey.clone(),
                role: AdminRole::KeyAdmin,
                expire_at: Utc::now() + Duration::hours(1),
                last_active_at: Utc::now(),
            },
        );
    }

    for token in ["tok-institution", "tok-system", "tok-key"] {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {token}")).expect("header value"),
        );
        assert!(require_admin_any(&state, &headers).is_ok());
    }
}

#[test]
fn parse_sr25519_pubkey_accepts_0x_prefix() {
    let key = "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    let parsed = login::parse_sr25519_pubkey(key).expect("parse pubkey");
    assert_eq!(
        parsed,
        "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
    );
}

#[test]
fn verify_admin_signature_accepts_0x_signature_prefix_for_sr25519() {
    let message = "sfid-qr-login|origin=http://127.0.0.1:5179|domain=127.0.0.1|session_id=sid|nonce=n|iat=1|exp=2";
    let (pubkey, signature) = sign_with_test_sr25519(33, message);
    assert!(verify_admin_signature(&pubkey, message, &signature));
}

#[test]
fn parse_sr25519_pubkey_bytes_rejects_non_hex_pubkey() {
    assert!(
        login::parse_sr25519_pubkey_bytes("5D4Y9fP2U8NDDw7X9W7N6wA6ZwZP3oYfgho2dQ4q8W35bLoA")
            .is_none()
    );
}

#[test]
fn normalize_account_pubkey_requires_32_byte_hex() {
    let raw = "AABBCCDDEEFF00112233445566778899AABBCCDDEEFF00112233445566778899";
    assert_eq!(
        normalize_account_pubkey(raw),
        Some("0xaabbccddeeff00112233445566778899aabbccddeeff00112233445566778899".to_string())
    );
    assert_eq!(
        normalize_account_pubkey(format!("0x{raw}").as_str()),
        Some("0xaabbccddeeff00112233445566778899aabbccddeeff00112233445566778899".to_string())
    );
    assert!(normalize_account_pubkey("5D4Y9fP2U8NDDw7X9W7N6wA6ZwZP3oYfgho2dQ4q8W35bLoA").is_none());
    assert!(normalize_account_pubkey("hello_world").is_none());
}

#[test]
fn cpms_site_scope_must_match_admin_province() {
    let site = CpmsSiteKeys {
        site_sfid: "SFID-SITE-001".to_string(),
        install_token: "test-token".to_string(),
        install_token_status: InstallTokenStatus::Pending,
        status: CpmsSiteStatus::Active,
        version: 1,
        province_code: "GZ".to_string(),
        admin_province: "贵州省".to_string(),
        city_name: "贵阳市".to_string(),
        institution_code: "ZF".to_string(),
        institution_name: "贵阳市政府".to_string(),
        qr1_payload: String::new(),
        created_by: "0xSUPER".to_string(),
        created_at: Utc::now(),
        updated_by: None,
        updated_at: None,
    };
    assert!(in_scope_cpms_site(&site, Some("贵州省")));
    assert!(!in_scope_cpms_site(&site, Some("中枢省")));
}

#[test]
fn validate_bind_callback_url_rejects_localhost_and_private_literals() {
    let localhost = validate_bind_callback_url("https://localhost/callback");
    assert!(localhost.is_err());
    let private_ip = validate_bind_callback_url("https://192.168.1.8/callback");
    assert!(private_ip.is_err());
}

#[test]
fn chain_signature_payload_and_hash_are_deterministic() {
    let payload = chain_signature_payload("vote_verify", "req-1", "nonce-1", 1731000000, "fp-123");
    let sig_a = chain_signature_hex("secret-a", payload.as_str());
    let sig_b = chain_signature_hex("secret-a", payload.as_str());
    let sig_c = chain_signature_hex("secret-b", payload.as_str());
    assert!(constant_time_eq_hex(sig_a.as_str(), sig_b.as_str()));
    assert!(!constant_time_eq_hex(sig_a.as_str(), sig_c.as_str()));
}

#[test]
fn chain_request_requires_replay_headers() {
    std::env::set_var("SFID_CHAIN_TOKEN", "test-chain-token");
    std::env::set_var(
        "SFID_CHAIN_SIGNING_SECRET",
        "test-chain-signing-secret-at-least-32",
    );
    let mut store = Store::default();
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-chain-token",
        HeaderValue::from_static("test-chain-token"),
    );
    assert!(require_chain_request(&mut store, &headers, "vote_verify", "fp").is_err());
}

#[test]
fn chain_request_rejects_duplicate_nonce() {
    std::env::set_var("SFID_CHAIN_TOKEN", "test-chain-token");
    std::env::set_var(
        "SFID_CHAIN_SIGNING_SECRET",
        "test-chain-signing-secret-at-least-32",
    );
    let mut store = Store::default();
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-chain-token",
        HeaderValue::from_static("test-chain-token"),
    );
    headers.insert("x-chain-request-id", HeaderValue::from_static("req-1"));
    headers.insert("x-chain-nonce", HeaderValue::from_static("nonce-1"));
    let ts = Utc::now().timestamp();
    headers.insert(
        "x-chain-timestamp",
        HeaderValue::from_str(&ts.to_string()).expect("header value"),
    );
    let sig_payload = chain_signature_payload("vote_verify", "req-1", "nonce-1", ts, "fp-1");
    let sig = chain_signature_hex(
        "test-chain-signing-secret-at-least-32",
        sig_payload.as_str(),
    );
    headers.insert(
        "x-chain-signature",
        HeaderValue::from_str(sig.as_str()).expect("header value"),
    );
    assert!(require_chain_request(&mut store, &headers, "vote_verify", "fp-1").is_ok());

    let mut second_headers = headers.clone();
    second_headers.insert("x-chain-request-id", HeaderValue::from_static("req-2"));
    let sig_payload_2 = chain_signature_payload("vote_verify", "req-2", "nonce-1", ts, "fp-2");
    let sig2 = chain_signature_hex(
        "test-chain-signing-secret-at-least-32",
        sig_payload_2.as_str(),
    );
    second_headers.insert(
        "x-chain-signature",
        HeaderValue::from_str(sig2.as_str()).expect("header value"),
    );
    assert!(require_chain_request(&mut store, &second_headers, "vote_verify", "fp-2").is_err());
}

// 中文注释:legacy chain_bind_result_reuses_persisted_runtime_credential 测试已删除
// (依赖 bindings_by_pubkey + get_bind_result,均已清除)。

#[test]
fn sync_key_admin_users_keeps_monotonic_ids() {
    let state = build_test_state();
    let replacement_seed = hex_seed(0x77);
    let replacement_pubkey =
        key_admins::chain_keyring::derive_pubkey_hex_from_seed(replacement_seed.as_str());
    let old_max_id;
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        old_max_id = store
            .admin_users_by_pubkey
            .values()
            .map(|u| u.id)
            .max()
            .expect("at least one admin user");
        store.next_admin_user_id = old_max_id + 1;

        let mut keyring = store
            .chain_keyring_state
            .as_ref()
            .cloned()
            .expect("keyring exists");
        let removed_pubkey = keyring.backup_b_pubkey.clone();
        store.admin_users_by_pubkey.remove(&removed_pubkey);
        keyring.backup_b_pubkey = replacement_pubkey.clone();
        store.chain_keyring_state = Some(keyring);
        key_admins::sync_key_admin_users(&mut store);

        let inserted = store
            .admin_users_by_pubkey
            .get(&replacement_pubkey)
            .expect("replacement key admin inserted");
        assert_eq!(inserted.id, old_max_id + 1);
        assert_eq!(store.next_admin_user_id, old_max_id + 2);
    }
}

#[test]
fn validate_active_main_signer_with_keyring_rejects_runtime_mismatch() {
    let (state, _, _, _) = setup_rotation_test_state();

    {
        let mut pubkey_guard = state
            .public_key_hex
            .write()
            .expect("public key write lock poisoned");
        *pubkey_guard = "0xdeadbeef".to_string();
    }
    let err = key_admins::validate_active_main_signer_with_keyring(&state)
        .expect_err("mismatched runtime signer should be rejected");
    assert!(err.contains("active signer state"));
}

#[test]
fn sensitive_seed_debug_remains_redacted() {
    let raw_seed = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let seed = SensitiveSeed::from(raw_seed);
    assert_eq!(format!("{seed:?}"), "SensitiveSeed(***)");
}
