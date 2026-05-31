// 中文注释:SFID 管理员认证与省/市角色测试。
//
// 本文件只保留当前二角色模型相关测试:
//   1. SHENG_ADMIN / SHI_ADMIN QR 登录、签名验签、admin scope。
//   2. SFID/Bind 工具函数。

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

fn build_test_state() -> AppState {
    std::env::set_var("SFID_CHAIN_TOKEN", "test-chain-token");
    std::env::set_var(
        "SFID_CHAIN_SIGNING_SECRET",
        "test-chain-signing-secret-at-least-32",
    );
    std::env::set_var("SFID_PUBLIC_SEARCH_TOKEN", "test-public-search-token");
    // ADR-008 Phase 23e:测试态 SFID main signer 由 env 派生,与 AppState 解耦。
    std::env::set_var("SFID_SIGNING_SEED_HEX", hex_seed(0x11));
    let state = AppState {
        store: StoreHandle::in_memory(),
        rate_limit_redis: Arc::new(
            redis::Client::open("redis://127.0.0.1/").expect("test redis url should be valid"),
        ),
        sharded_store: {
            #[cfg(test)]
            {
                Arc::new(store_shards::ShardedStore::new(Arc::new(
                    store_shards::backend::MockShardBackend::new(),
                )
                    as Arc<dyn store_shards::backend::ShardBackend>))
            }
            #[cfg(not(test))]
            {
                unreachable!("main_tests.rs only compiles in test cfg")
            }
        },
    };
    ensure_builtin_province_admins(&state);
    state
}

fn sample_cpms_site(site_sfid: &str) -> CpmsSiteKeys {
    CpmsSiteKeys {
        site_sfid: site_sfid.to_string(),
        install_token: String::new(),
        install_secret: "0x1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
        install_secret_hash: "0xinstall-secret-hash".to_string(),
        install_token_status: InstallTokenStatus::Pending,
        status: CpmsSiteStatus::Pending,
        version: 1,
        province_code: "GZ".to_string(),
        admin_province: "贵州省".to_string(),
        city_name: "贵阳市".to_string(),
        city_code: "001".to_string(),
        institution_code: "ZF".to_string(),
        institution_name: "贵阳市公安局".to_string(),
        qr1_payload: String::new(),
        cpms_pubkey_hash: None,
        created_by: "0xSUPER".to_string(),
        created_at: Utc::now(),
        updated_by: None,
        updated_at: None,
    }
}

async fn parse_json(resp: Response) -> serde_json::Value {
    let bytes = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("response body bytes");
    serde_json::from_slice(&bytes).expect("json response")
}

#[tokio::test]
async fn cpms_site_snapshot_sync_restores_sharded_runtime_cache() {
    let state = build_test_state();
    let site = sample_cpms_site("SFID-SITE-RESTORE");
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        store
            .cpms_site_keys
            .insert(site.site_sfid.clone(), site.clone());
    }

    sync_cpms_sites_to_sharded(&state).await;

    let restored = state
        .sharded_store
        .read_province("贵州省", |shard| {
            shard.cpms_site_keys.get(&site.site_sfid).cloned()
        })
        .await
        .expect("read province");
    let restored = restored.expect("cpms site restored to sharded cache");
    assert_eq!(restored.site_sfid, site.site_sfid);
    assert_eq!(restored.install_secret, site.install_secret);
}

#[tokio::test]
async fn cpms_pubkey_binding_persists_store_and_runtime_cache() {
    let state = build_test_state();
    let site = sample_cpms_site("SFID-SITE-PUBKEY");
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        store
            .cpms_site_keys
            .insert(site.site_sfid.clone(), site.clone());
    }

    cpms::handler::bind_cpms_pubkey_if_needed(
        &state,
        "贵州省",
        site.site_sfid.as_str(),
        "0xcpms-pubkey-hash",
    )
    .await
    .expect("bind cpms pubkey");

    let stored = {
        let store = state.store.read().expect("store read lock poisoned");
        store
            .cpms_site_keys
            .get(&site.site_sfid)
            .cloned()
            .expect("stored cpms site")
    };
    assert_eq!(
        stored.cpms_pubkey_hash.as_deref(),
        Some("0xcpms-pubkey-hash")
    );
    assert_eq!(stored.status, CpmsSiteStatus::Active);
    assert_eq!(stored.install_token_status, InstallTokenStatus::Used);

    let cached = state
        .sharded_store
        .read_province("贵州省", |shard| {
            shard.cpms_site_keys.get(&site.site_sfid).cloned()
        })
        .await
        .expect("read province")
        .expect("cached cpms site");
    assert_eq!(cached.cpms_pubkey_hash, stored.cpms_pubkey_hash);
    assert_eq!(cached.status, stored.status);
    assert_eq!(cached.install_token_status, stored.install_token_status);

    let err = cpms::handler::bind_cpms_pubkey_if_needed(
        &state,
        "贵州省",
        site.site_sfid.as_str(),
        "0xdifferent-cpms-pubkey-hash",
    )
    .await
    .expect_err("different cpms pubkey hash must be rejected");
    assert_eq!(err.0, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(err.2, "cpms_pubkey does not match installed CPMS");
}

#[test]
fn module_snapshots_preserve_cross_request_runtime_state() {
    let now = chrono::Utc::now();
    let mut store = Store::default();
    store.citizen_bind_challenges.insert(
        "bind-1".to_string(),
        CitizenBindChallenge {
            challenge_id: "bind-1".to_string(),
            challenge_text: "sfid-citizen-bind-v1|bind-1".to_string(),
            mode: "create".to_string(),
            citizen_id: None,
            archive_no: "ARCHIVE-1".to_string(),
            wallet_address: "5CTestAddress".to_string(),
            wallet_pubkey: "0xabc".to_string(),
            wallet_sig_alg: "sr25519".to_string(),
            citizen_status: CitizenStatus::Normal,
            voting_eligible: true,
            archive_valid_from: "2026-05-24".to_string(),
            archive_valid_until: "2036-05-23".to_string(),
            status_updated_at: 1_779_580_800,
            province_code: "GD".to_string(),
            city_code: "4401".to_string(),
            expire_at: now + Duration::minutes(5),
            created_at: now,
        },
    );
    store.login_challenges.insert(
        "login-1".to_string(),
        LoginChallenge {
            challenge_id: "login-1".to_string(),
            admin_pubkey: "0xadmin".to_string(),
            challenge_text: "challenge".to_string(),
            challenge_token: String::new(),
            qr_aud: String::new(),
            qr_origin: String::new(),
            origin: "http://localhost".to_string(),
            domain: "localhost".to_string(),
            session_id: "sid-1".to_string(),
            nonce: "nonce-1".to_string(),
            issued_at: now,
            expire_at: now + Duration::minutes(5),
            consumed: false,
        },
    );
    store.admin_sessions.insert(
        "token-1".to_string(),
        AdminSession {
            token: "token-1".to_string(),
            admin_pubkey: "0xadmin".to_string(),
            role: AdminRole::ShengAdmin,
            expire_at: now + Duration::hours(1),
            last_active_at: now,
        },
    );

    let mut restored = Store::default();
    CitizenStoreSnapshot::from_store(&store).apply_to(&mut restored);
    OpsStoreSnapshot::from_store(&store).apply_to(&mut restored);

    assert!(restored.citizen_bind_challenges.contains_key("bind-1"));
    assert!(restored.login_challenges.contains_key("login-1"));
    assert!(restored.admin_sessions.contains_key("token-1"));
}

/// 构造 QR 登录完整签名消息：challenge_payload 末尾补 principal(pubkey hex 去 0x 小写)。
fn qr_login_sign_message(challenge_payload: &str, pubkey_hex: &str) -> String {
    let pp = pubkey_hex
        .strip_prefix("0x")
        .or_else(|| pubkey_hex.strip_prefix("0X"))
        .unwrap_or(pubkey_hex)
        .to_lowercase();
    format!("{challenge_payload}{pp}")
}

fn sign_with_test_sr25519(seed_byte: u8, message: &str) -> (String, String) {
    let seed = hex_seed(seed_byte);
    let keypair = crypto::sr25519::load_signing_key_from_seed(seed.as_str());
    let sig = keypair.sign(message.as_bytes());
    (
        format!("0x{}", hex::encode(keypair.public().0)),
        format!("0x{}", hex::encode(sig.0)),
    )
}

#[test]
fn admin_pubkey_duplicate_error_codes_are_role_specific() {
    // 中文注释:管理员公钥全局唯一,重复冲突必须暴露已存在角色而不是笼统 409。
    assert_eq!(
        sfid_error_code(
            StatusCode::CONFLICT,
            "admin pubkey already exists as sheng admin"
        ),
        "SFID_ADMIN_PUBKEY_EXISTS_AS_SHENG_ADMIN"
    );
    assert_eq!(
        sfid_error_code(
            StatusCode::CONFLICT,
            "admin pubkey already exists as shi admin"
        ),
        "SFID_ADMIN_PUBKEY_EXISTS_AS_SHI_ADMIN"
    );
    assert_eq!(
        sfid_error_code(StatusCode::CONFLICT, "sheng admin province limit reached"),
        "SFID_ADMIN_SHENG_ADMIN_PROVINCE_LIMIT_REACHED"
    );
    assert_eq!(
        sfid_error_code(StatusCode::CONFLICT, "shi admin city limit reached"),
        "SFID_ADMIN_SHI_ADMIN_CITY_LIMIT_REACHED"
    );
    assert_eq!(
        sfid_error_code(StatusCode::INTERNAL_SERVER_ERROR, "store persist failed"),
        "SFID_STORE_PERSIST_FAILED"
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

    let (query_pubkey, _) = sign_with_test_sr25519(11, "dummy");
    let login_msg = qr_login_sign_message(&challenge_payload, &query_pubkey);
    let (_, signature) = sign_with_test_sr25519(11, &login_msg);
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

    let (admin_pubkey, _) = sign_with_test_sr25519(22, "dummy");
    let login_msg = qr_login_sign_message(&challenge_payload, &admin_pubkey);
    let (_, signature) = sign_with_test_sr25519(22, &login_msg);
    {
        let mut store = state.store.write().expect("store write lock poisoned");
        store.admin_users_by_pubkey.insert(
            admin_pubkey.clone(),
            AdminUser {
                id: 999,
                admin_pubkey: admin_pubkey.clone(),
                admin_name: String::new(),
                role: AdminRole::ShengAdmin,
                built_in: false,
                created_by: "TEST".to_string(),
                created_at: Utc::now(),
                updated_at: None,
                city: String::new(),
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

    // admin_auth_qr_complete 用 tokio::task::spawn 异步写 sharded_store,
    // 测试里 spawn 可能尚未执行,需手动同步写入以避免竞态 401。
    {
        let store = state.store.read().expect("store read lock poisoned");
        if let Some(session) = store.admin_sessions.get(token) {
            let s = session.clone();
            let t = token.to_string();
            state
                .sharded_store
                .write_global_sync(|g| {
                    g.admin_sessions.insert(t, s);
                })
                .expect("write_global_sync");
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", token)).expect("header value"),
    );
    assert!(require_admin_any(&state, &headers).is_ok());
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
                built_in: false,
                created_by: "TEST".to_string(),
                created_at: Utc::now(),
                updated_at: None,
                city: String::new(),
            },
        );
        store.admin_users_by_pubkey.insert(
            signer_pubkey.clone(),
            AdminUser {
                id: 2002,
                admin_pubkey: signer_pubkey.clone(),
                admin_name: String::new(),
                role: AdminRole::ShiAdmin,
                built_in: false,
                created_by: "TEST".to_string(),
                created_at: Utc::now(),
                updated_at: None,
                city: String::new(),
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

#[tokio::test]
async fn require_admin_any_should_allow_remaining_two_roles() {
    // 中文注释:当前只覆盖 ShengAdmin / ShiAdmin 两个管理员角色。
    let state = build_test_state();
    let institution_pubkey = {
        let store = state.store.read().expect("store read lock poisoned");
        store
            .admin_users_by_pubkey
            .values()
            .find(|u| u.role == AdminRole::ShengAdmin)
            .map(|u| u.admin_pubkey.clone())
            .expect("institution admin exists")
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
                built_in: false,
                created_by: institution_pubkey.clone(),
                created_at: Utc::now(),
                updated_at: None,
                city: String::new(),
            },
        );
        let sessions = [
            (
                "tok-institution",
                institution_pubkey.clone(),
                AdminRole::ShengAdmin,
            ),
            ("tok-system", system_pubkey.clone(), AdminRole::ShiAdmin),
        ];
        for (tok, pubkey, role) in &sessions {
            let session = AdminSession {
                token: tok.to_string(),
                admin_pubkey: pubkey.clone(),
                role: role.clone(),
                expire_at: Utc::now() + Duration::hours(1),
                last_active_at: Utc::now(),
            };
            store
                .admin_sessions
                .insert(tok.to_string(), session.clone());
            state
                .sharded_store
                .write_global_sync(|g| {
                    g.admin_sessions.insert(tok.to_string(), session);
                })
                .expect("write_global_sync");
        }
    }

    for token in ["tok-institution", "tok-system"] {
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
    assert!(login::verify_admin_signature(&pubkey, message, &signature));
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
        install_secret: "0x11".to_string(),
        install_secret_hash: "0x22".to_string(),
        install_token_status: InstallTokenStatus::Pending,
        status: CpmsSiteStatus::Active,
        version: 1,
        province_code: "GZ".to_string(),
        admin_province: "贵州省".to_string(),
        city_name: "贵阳市".to_string(),
        city_code: "001".to_string(),
        institution_code: "ZF".to_string(),
        institution_name: "贵阳市政府".to_string(),
        qr1_payload: String::new(),
        cpms_pubkey_hash: None,
        created_by: "0xSUPER".to_string(),
        created_at: Utc::now(),
        updated_by: None,
        updated_at: None,
    };
    assert!(in_scope_cpms_site(&site, Some("贵州省")));
    assert!(!in_scope_cpms_site(&site, Some("中枢省")));
}

#[test]
fn sensitive_seed_debug_remains_redacted() {
    let raw_seed = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let seed = SensitiveSeed::from(raw_seed);
    assert_eq!(format!("{seed:?}"), "SensitiveSeed(***)");
}
