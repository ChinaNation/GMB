use super::*;
use crate::login::AdminSession;
use axum::{
    body::to_bytes,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::Response,
};
use chrono::Duration;
use schnorrkel::signing_context;

fn build_test_state() -> AppState {
    std::env::set_var("SFID_CHAIN_TOKEN", "test-chain-token");
    std::env::set_var(
        "SFID_CHAIN_SIGNING_SECRET",
        "test-chain-signing-secret-at-least-32",
    );
    std::env::set_var("SFID_PUBLIC_SEARCH_TOKEN", "test-public-search-token");
    std::env::set_var("SFID_RUNTIME_META_KEY", "test-runtime-meta-key");
    let main_seed = "sfid-dev-master-seed-v1".to_string();
    let main_key = key_admins::chain_keyring::load_signing_key_from_seed(main_seed.as_str());
    let public_key_hex = format!("0x{}", hex::encode(main_key.public.to_bytes()));
    let mut known_key_seeds = HashMap::new();
    known_key_seeds.insert(public_key_hex.clone(), main_seed.clone());
    let state = AppState {
        store: StoreHandle::in_memory(),
        signing_seed_hex: Arc::new(RwLock::new(main_seed)),
        known_key_seeds: Arc::new(RwLock::new(known_key_seeds)),
        request_limits: Arc::new(Mutex::new(HashMap::new())),
        key_id: "sfid-master-v1".to_string(),
        key_version: "v1".to_string(),
        key_alg: "sr25519".to_string(),
        public_key_hex: Arc::new(RwLock::new(public_key_hex)),
    };
    seed_super_admins(&state);
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
    let seed = [seed_byte; 32];
    let mini = schnorrkel::MiniSecretKey::from_bytes(&seed).expect("mini secret key");
    let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Uniform);
    let ctx = signing_context(b"substrate");
    let sig = keypair.sign(ctx.bytes(message.as_bytes()));
    (
        format!("0x{}", hex::encode(keypair.public.to_bytes())),
        format!("0x{}", hex::encode(sig.to_bytes())),
    )
}

fn sign_rotation_challenge(seed_hex: &str, message: &str) -> String {
    let keypair = key_admins::chain_keyring::load_signing_key_from_seed(seed_hex);
    let ctx = signing_context(b"substrate");
    let sig = keypair.sign(ctx.bytes(message.as_bytes()));
    format!("0x{}", hex::encode(sig.to_bytes()))
}

fn setup_rotation_test_state() -> (AppState, HeaderMap, String, String) {
    let state = build_test_state();
    let main_seed = "sfid-test-main-seed";
    let backup_a_seed = "sfid-test-backup-a-seed";
    let backup_b_seed = "sfid-test-backup-b-seed";
    let new_backup_seed = "sfid-test-backup-c-seed";
    let main_pubkey = key_admins::chain_keyring::derive_pubkey_hex_from_seed(main_seed);
    let backup_a_pubkey = key_admins::chain_keyring::derive_pubkey_hex_from_seed(backup_a_seed);
    let backup_b_pubkey = key_admins::chain_keyring::derive_pubkey_hex_from_seed(backup_b_seed);

    {
        let mut seed_guard = state
            .signing_seed_hex
            .write()
            .expect("signing seed write lock poisoned");
        *seed_guard = main_seed.to_string();
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
        known.insert(main_pubkey.clone(), main_seed.to_string());
        known.insert(backup_a_pubkey.clone(), backup_a_seed.to_string());
        known.insert(backup_b_pubkey.clone(), backup_b_seed.to_string());
        known.insert(
            key_admins::chain_keyring::derive_pubkey_hex_from_seed(new_backup_seed),
            new_backup_seed.to_string(),
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
    (
        state,
        headers,
        backup_a_seed.to_string(),
        new_backup_seed.to_string(),
    )
}

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
async fn keyring_rotate_commit_reports_chain_submit_failure_without_blocking_local_rotation() {
    let previous_rpc_url = std::env::var("SFID_CHAIN_RPC_URL").ok();
    std::env::remove_var("SFID_CHAIN_RPC_URL");
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
    let signature = sign_rotation_challenge(backup_a_seed.as_str(), challenge_text.as_str());

    let verify_resp = key_admins::admin_chain_keyring_rotate_verify(
        State(state.clone()),
        headers.clone(),
        Json(KeyringRotateVerifyInput {
            challenge_id: challenge_id.clone(),
            signature: signature.clone(),
        }),
    )
    .await
    .into_response();
    assert_eq!(verify_resp.status(), StatusCode::OK);

    let new_backup_pubkey =
        key_admins::chain_keyring::derive_pubkey_hex_from_seed(new_backup_seed.as_str());
    let backup_a_pubkey =
        key_admins::chain_keyring::derive_pubkey_hex_from_seed(backup_a_seed.as_str());
    let commit_resp = key_admins::admin_chain_keyring_rotate_commit(
        State(state),
        headers,
        Json(KeyringRotateCommitInput {
            challenge_id,
            signature,
            new_backup_pubkey,
        }),
    )
    .await
    .into_response();

    if let Some(value) = previous_rpc_url {
        std::env::set_var("SFID_CHAIN_RPC_URL", value);
    }

    assert_eq!(commit_resp.status(), StatusCode::OK);
    let body = parse_json(commit_resp).await;
    assert_eq!(body["data"]["chain_submit_ok"].as_bool(), Some(false));
    assert_eq!(
        body["data"]["main_pubkey"].as_str(),
        Some(backup_a_pubkey.as_str())
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
async fn qr_login_super_admin_keeps_write_permission() {
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
                role: AdminRole::SuperAdmin,
                status: AdminStatus::Active,
                built_in: false,
                created_by: "TEST".to_string(),
                created_at: Utc::now(),
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
        Some("SUPER_ADMIN")
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
                role: AdminRole::SuperAdmin,
                status: AdminStatus::Active,
                built_in: false,
                created_by: "TEST".to_string(),
                created_at: Utc::now(),
            },
        );
        store.admin_users_by_pubkey.insert(
            signer_pubkey.clone(),
            AdminUser {
                id: 2002,
                admin_pubkey: signer_pubkey.clone(),
                admin_name: String::new(),
                role: AdminRole::OperatorAdmin,
                status: AdminStatus::Active,
                built_in: false,
                created_by: "TEST".to_string(),
                created_at: Utc::now(),
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
fn require_super_or_operator_or_key_admin_should_allow_expected_roles() {
    let state = build_test_state();
    let (super_pubkey, key_pubkey) = {
        let store = state.store.read().expect("store read lock poisoned");
        let super_pubkey = store
            .admin_users_by_pubkey
            .values()
            .find(|u| u.role == AdminRole::SuperAdmin)
            .map(|u| u.admin_pubkey.clone())
            .expect("super admin exists");
        let key_pubkey = store
            .admin_users_by_pubkey
            .values()
            .find(|u| u.role == AdminRole::KeyAdmin)
            .map(|u| u.admin_pubkey.clone())
            .expect("key admin exists");
        (super_pubkey, key_pubkey)
    };
    let operator_pubkey = "0xTEST_OPERATOR_ADMIN".to_string();
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        store.admin_users_by_pubkey.insert(
            operator_pubkey.clone(),
            AdminUser {
                id: 9_999,
                admin_pubkey: operator_pubkey.clone(),
                admin_name: "测试操作员".to_string(),
                role: AdminRole::OperatorAdmin,
                status: AdminStatus::Active,
                built_in: false,
                created_by: super_pubkey.clone(),
                created_at: Utc::now(),
            },
        );
        store.admin_sessions.insert(
            "tok-super".to_string(),
            AdminSession {
                token: "tok-super".to_string(),
                admin_pubkey: super_pubkey.clone(),
                role: AdminRole::SuperAdmin,
                expire_at: Utc::now() + Duration::hours(1),
                last_active_at: Utc::now(),
            },
        );
        store.admin_sessions.insert(
            "tok-operator".to_string(),
            AdminSession {
                token: "tok-operator".to_string(),
                admin_pubkey: operator_pubkey.clone(),
                role: AdminRole::OperatorAdmin,
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
        store.admin_sessions.insert(
            "tok-query".to_string(),
            AdminSession {
                token: "tok-query".to_string(),
                admin_pubkey: "query-only".to_string(),
                role: AdminRole::QueryOnly,
                expire_at: Utc::now() + Duration::hours(1),
                last_active_at: Utc::now(),
            },
        );
    }

    for token in ["tok-super", "tok-operator", "tok-key"] {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {token}")).expect("header value"),
        );
        assert!(require_super_or_operator_or_key_admin(&state, &headers).is_ok());
    }

    let mut query_headers = HeaderMap::new();
    query_headers.insert(
        "authorization",
        HeaderValue::from_str("Bearer tok-query").expect("header value"),
    );
    assert!(require_super_or_operator_or_key_admin(&state, &query_headers).is_err());
}

#[test]
fn parse_sr25519_pubkey_accepts_0x_prefix() {
    let key = "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    let parsed = login::parse_sr25519_pubkey(key).expect("parse pubkey");
    assert_eq!(
        parsed,
        "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
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
fn pending_scope_requires_province_when_admin_is_scoped() {
    let pending = PendingRequest {
        seq: 1,
        account_pubkey: "0xP".to_string(),
        admin_province: None,
        requested_at: Utc::now(),
        callback_url: None,
        client_request_id: None,
    };
    assert!(!in_scope_pending(&pending, Some("中枢省")));

    let claimed = PendingRequest {
        admin_province: Some("中枢省".to_string()),
        ..pending
    };
    assert!(in_scope_pending(&claimed, Some("中枢省")));
    assert!(!in_scope_pending(&claimed, Some("岭南省")));
}

#[test]
fn cpms_site_scope_must_match_admin_province() {
    let site = CpmsSiteKeys {
        site_sfid: "SFID-SITE-001".to_string(),
        pubkey_1: "0x1".to_string(),
        pubkey_2: "0x2".to_string(),
        pubkey_3: "0x3".to_string(),
        status: CpmsSiteStatus::Active,
        version: 1,
        last_register_issued_at: Utc::now().timestamp(),
        init_qr_payload: None,
        admin_province: "贵州省".to_string(),
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
