// 中文注释:ADR-008 Phase 23e(2026-05-01)整体重写。
//
// 历史 main_tests.rs(966 行)包含大量 KEY_ADMIN keyring rotate / chain_keyring
// / signing_seed_hex / known_key_seeds 测试,这些路径都已随 phase23e 删除。
// 本文件保留:
//   1. 三省 3-tier 签名 keypair 隔离测试(`sheng_signing_3tier_isolation`,
//      ADR-008 决议核心保证)
//   2. SHENG_ADMIN / SHI_ADMIN QR 登录、签名验签、admin scope、SFID/Bind 工具
//      函数等与 KEY_ADMIN 解耦的旧测试
//
// 删除的测试:
//   - keyring_rotate_*(整个 rotate 流程)
//   - sync_key_admin_users_keeps_monotonic_ids
//   - validate_active_main_signer_with_keyring_rejects_runtime_mismatch
//   - require_admin_any_should_allow_all_three_roles 中的 KeyAdmin session 注入

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
        cpms_register_inflight: Arc::new(Mutex::new(HashMap::new())),
        sheng_signer_cache: Arc::new(sheng_admins::signing_cache::ShengSigningCache::new()),
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
    state
}

async fn parse_json(resp: Response) -> serde_json::Value {
    let bytes = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("response body bytes");
    serde_json::from_slice(&bytes).expect("json response")
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

#[tokio::test]
async fn require_admin_any_should_allow_remaining_two_roles() {
    // ADR-008 Phase 23e:KEY_ADMIN 整角色废止,本测试仅覆盖 ShengAdmin / ShiAdmin。
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
        qr3_payload: None,
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
fn sensitive_seed_debug_remains_redacted() {
    let raw_seed = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let seed = SensitiveSeed::from(raw_seed);
    assert_eq!(format!("{seed:?}"), "SensitiveSeed(***)");
}

/// 中文注释:ADR-008 Phase 23e 核心保证 —— 三省 / 三 slot 各自独立 keypair。
///
/// 模拟三个不同 (province, admin_pubkey) 进 cache,断言:
///   1. 各自从 cache 取出的 Pair 公钥唯一
///   2. `any_for_province` 返回的是本省内某一 slot
///   3. `unload_province` 清掉本省所有 slot,不影响其他省
#[test]
fn sheng_signing_3tier_isolation() {
    let cache = sheng_admins::signing_cache::ShengSigningCache::new();

    // 安徽省 main / backup_1 / backup_2(三 slot 模拟)
    let ah_main_seed = hex_seed(0x41);
    let ah_b1_seed = hex_seed(0x42);
    let ah_b2_seed = hex_seed(0x43);
    let ah_main_pair = crypto::sr25519::load_signing_key_from_seed(&ah_main_seed);
    let ah_b1_pair = crypto::sr25519::load_signing_key_from_seed(&ah_b1_seed);
    let ah_b2_pair = crypto::sr25519::load_signing_key_from_seed(&ah_b2_seed);
    let ah_main_pk: [u8; 32] = ah_main_pair.public().0;
    let ah_b1_pk: [u8; 32] = ah_b1_pair.public().0;
    let ah_b2_pk: [u8; 32] = ah_b2_pair.public().0;

    cache.load("安徽省".to_string(), ah_main_pk, ah_main_pair.clone());
    cache.load("安徽省".to_string(), ah_b1_pk, ah_b1_pair.clone());
    cache.load("安徽省".to_string(), ah_b2_pk, ah_b2_pair.clone());

    // 广东省 main(独立省)
    let gd_seed = hex_seed(0x51);
    let gd_pair = crypto::sr25519::load_signing_key_from_seed(&gd_seed);
    let gd_pk: [u8; 32] = gd_pair.public().0;
    cache.load("广东省".to_string(), gd_pk, gd_pair.clone());

    // 1. 三 slot 公钥彼此不等
    assert_ne!(ah_main_pk, ah_b1_pk);
    assert_ne!(ah_b1_pk, ah_b2_pk);
    assert_ne!(ah_main_pk, ah_b2_pk);
    assert_ne!(ah_main_pk, gd_pk);

    // 2. 精确取(province, admin_pubkey)各自命中
    let got_main = cache.get("安徽省", &ah_main_pk).expect("ah main present");
    let got_b1 = cache.get("安徽省", &ah_b1_pk).expect("ah b1 present");
    let got_b2 = cache.get("安徽省", &ah_b2_pk).expect("ah b2 present");
    assert_eq!(got_main.public().0, ah_main_pk);
    assert_eq!(got_b1.public().0, ah_b1_pk);
    assert_eq!(got_b2.public().0, ah_b2_pk);

    // 3. 跨省隔离:用安徽 admin pubkey 在广东省 cache 里取不到
    assert!(cache.get("广东省", &ah_main_pk).is_none());

    // 4. any_for_province 返回的是本省任一 slot
    let any_ah = cache.any_for_province("安徽省").expect("ah any present");
    let any_pk = any_ah.public().0;
    assert!(
        any_pk == ah_main_pk || any_pk == ah_b1_pk || any_pk == ah_b2_pk,
        "any_for_province must return one of the loaded ah slots"
    );

    // 5. active_count 反映总 slot 数
    assert_eq!(cache.active_count(), 4);

    // 6. 驱逐安徽省 → 安徽 3 slot 全清,广东省不受影响
    cache.unload_province("安徽省");
    assert!(cache.get("安徽省", &ah_main_pk).is_none());
    assert!(cache.get("安徽省", &ah_b1_pk).is_none());
    assert!(cache.get("安徽省", &ah_b2_pk).is_none());
    assert!(cache.any_for_province("安徽省").is_none());
    let still_gd = cache.get("广东省", &gd_pk).expect("gd still present");
    assert_eq!(still_gd.public().0, gd_pk);
    assert_eq!(cache.active_count(), 1);
}
