//! 管理员二维码登录 handler。
//!
//! 只承接 WUMIN_QR_V1 登录挑战生成、手机扫码完成、网页轮询结果;普通 challenge
//! 登录仍在 `handler.rs`。

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use tracing::warn;
use uuid::Uuid;

use crate::crypto::pubkey::same_admin_pubkey;
use crate::scope::admin_province::province_scope_for_role;
use crate::*;

use super::guards::bootstrap_sheng_signing_pair;
use super::model::*;
use super::signature::{
    build_admin_display_name, build_admin_display_name_from_user, build_login_qr_system_signature,
    cleanup_expired_challenges, extract_domain_from_origin, resolve_admin_city,
    resolve_admin_pubkey_key, verify_admin_signature,
};
use super::LOGIN_CHALLENGE_TTL_SECONDS;

pub(crate) async fn admin_auth_qr_challenge(
    State(state): State<AppState>,
    Json(input): Json<AdminQrChallengeInput>,
) -> impl IntoResponse {
    let origin = input.origin.unwrap_or_default().trim().to_string();
    if origin.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "origin is required");
    }
    let session_id = input.session_id.unwrap_or_default().trim().to_string();
    if session_id.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "session_id is required");
    }
    let derived_domain = extract_domain_from_origin(&origin)
        .or_else(|| input.domain.clone())
        .unwrap_or_default();
    if derived_domain.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "domain is required");
    }

    let now = Utc::now();
    let expire_at = now + Duration::seconds(LOGIN_CHALLENGE_TTL_SECONDS);
    let challenge_id = Uuid::new_v4().to_string();
    // challenge_text:客户端签 login_receipt 时的原文(与 wumin 端的
    // buildSignatureMessage(kind=login_receipt, ...) 拼接规则保持一致)。
    // 注意 <principal> 位置由客户端签名时填入自己的 pubkey,后端验证时同样
    // 以客户端 pubkey 为 principal 重新拼接。这里保存的 challenge_text 仅作
    // 回放保护用的唯一 token,实际验证在 admin_auth_qr_complete 中重建。
    let challenge_text = format!(
        "{}|{}|{}|{}|{}|",
        crate::qr::WUMIN_QR_V1,
        crate::qr::QrKind::LoginReceipt.wire(),
        challenge_id,
        "sfid",
        expire_at.timestamp()
    );
    let (sys_pubkey, sys_sig) = match build_login_qr_system_signature(
        &state,
        "sfid",
        challenge_id.as_str(),
        now.timestamp(),
        expire_at.timestamp(),
    ) {
        Ok(v) => v,
        Err(err) => {
            warn!(error = %err, "build sfid login qr system signature failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "build login qr signature failed",
            );
        }
    };
    let login_qr_payload = serde_json::to_string(&crate::qr::LoginChallengeEnvelope::new(
        challenge_id.clone(),
        now.timestamp(),
        expire_at.timestamp(),
        crate::qr::LoginChallengeBody {
            system: "sfid".to_string(),
            sys_pubkey: sys_pubkey.clone(),
            sys_sig: sys_sig.clone(),
        },
    ))
    .unwrap_or_default();

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);
    insert_bounded_map(
        &mut store.login_challenges,
        challenge_id.clone(),
        LoginChallenge {
            challenge_id: challenge_id.clone(),
            admin_pubkey: String::new(),
            challenge_text: challenge_text.clone(),
            challenge_token: String::new(),
            qr_aud: String::new(),
            qr_origin: String::new(),
            origin: origin.clone(),
            domain: derived_domain.clone(),
            session_id: session_id.clone(),
            nonce: String::new(),
            issued_at: now,
            expire_at,
            consumed: false,
        },
        bounded_cache_limit("SFID_LOGIN_CHALLENGE_MAX", 20_000),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminQrChallengeOutput {
            challenge_id,
            challenge_payload: challenge_text,
            login_qr_payload,
            origin,
            domain: derived_domain,
            session_id,
            expire_at: expire_at.timestamp(),
        },
    })
    .into_response()
}

pub(crate) async fn admin_auth_qr_complete(
    State(state): State<AppState>,
    Json(input): Json<AdminQrCompleteInput>,
) -> impl IntoResponse {
    if input.challenge_id.trim().is_empty()
        || input.admin_pubkey.trim().is_empty()
        || input.signature.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, admin_pubkey, signature are required",
        );
    }

    let now = Utc::now();
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);

    let (challenge_text, session_id, challenge_expire_at) = {
        let Some(challenge) = store.login_challenges.get_mut(&input.challenge_id) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found");
        };
        if challenge.consumed {
            return api_error(StatusCode::CONFLICT, 1007, "challenge already consumed");
        }
        if let Some(client_sid) = input.session_id.as_ref() {
            if challenge.session_id != client_sid.trim() {
                return api_error(StatusCode::FORBIDDEN, 1003, "challenge session mismatch");
            }
        }
        if now > challenge.expire_at {
            return api_error(StatusCode::UNAUTHORIZED, 1007, "challenge expired");
        }
        (
            challenge.challenge_text.clone(),
            challenge.session_id.clone(),
            challenge.expire_at.timestamp(),
        )
    };

    let login_pubkey_raw = input.admin_pubkey.trim().to_string();
    let signer_pubkey = input
        .signer_pubkey
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let verify_pubkey = signer_pubkey
        .clone()
        .unwrap_or_else(|| login_pubkey_raw.clone());
    let login_pubkey = resolve_admin_pubkey_key(&store, &login_pubkey_raw)
        .or_else(|| {
            signer_pubkey
                .as_ref()
                .and_then(|spk| resolve_admin_pubkey_key(&store, spk))
        })
        .unwrap_or_else(|| login_pubkey_raw.clone());
    if !same_admin_pubkey(login_pubkey.as_str(), verify_pubkey.as_str()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "signer_pubkey must match admin_pubkey",
        );
    }
    // 重建完整签名原文(包含签名者公钥作为 principal),与 wumin 端
    // buildSignatureMessage(kind=login_receipt, principal=pubkey) 一致。
    // challenge_text 仅用于回放保护,不直接用于签名验证。
    let verify_message = crate::qr::build_signature_message(
        crate::qr::QrKind::LoginReceipt,
        &input.challenge_id,
        Some("sfid"),
        Some(challenge_expire_at),
        &verify_pubkey,
    );
    let _ = challenge_text; // 不再用于签名验证
    if !verify_admin_signature(&verify_pubkey, &verify_message, input.signature.trim()) {
        warn!(
            challenge = %input.challenge_id,
            admin_pubkey = %login_pubkey_raw,
            signer_pubkey = %verify_pubkey,
            "qr login signature verify failed"
        );
        return api_error(StatusCode::UNAUTHORIZED, 2004, "signature verify failed");
    }
    let Some(admin) = store.admin_users_by_pubkey.get(&login_pubkey) else {
        return api_error(StatusCode::FORBIDDEN, 2002, "admin not found");
    };
    if admin.status != AdminStatus::Active {
        return api_error(StatusCode::FORBIDDEN, 2003, "admin disabled");
    }
    let login_role = admin.role.clone();
    let login_status = admin.status.clone();

    if let Some(challenge) = store.login_challenges.get_mut(&input.challenge_id) {
        challenge.consumed = true;
        challenge.admin_pubkey = login_pubkey.clone();
    }

    let access_token = Uuid::new_v4().to_string();
    let expire_at = now + Duration::hours(8);
    let new_session_qr = AdminSession {
        token: access_token.clone(),
        admin_pubkey: login_pubkey.clone(),
        role: login_role.clone(),
        expire_at,
        last_active_at: now,
    };
    store
        .admin_sessions
        .insert(access_token.clone(), new_session_qr.clone());

    // Phase 2 admin_auth 迁移:QR 登录成功后同步写 GlobalShard session
    {
        let ss = state.sharded_store.clone();
        let token_for_shard = access_token.clone();
        tokio::task::spawn(async move {
            let _ = ss
                .write_global(|g| {
                    g.admin_sessions.insert(token_for_shard, new_session_qr);
                })
                .await;
        });
    }

    // 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 步骤 7：
    // 为省登录管理员 bootstrap 本省签名密钥（需要 provinces 映射）。
    let bootstrap_pubkey = login_pubkey.clone();
    let bootstrap_role = login_role.clone();
    let bootstrap_province = province_scope_for_role(&store, &login_pubkey, &login_role);
    store.qr_login_results.insert(
        input.challenge_id.clone(),
        QrLoginResultRecord {
            session_id,
            access_token: access_token.clone(),
            expire_at,
            admin_pubkey: login_pubkey,
            role: login_role,
            status: login_status,
            created_at: now,
        },
    );
    drop(store);

    if bootstrap_role == AdminRole::ShengAdmin {
        if let Some(province) = bootstrap_province.as_deref() {
            bootstrap_sheng_signing_pair(&state, bootstrap_pubkey.as_str(), province);
        }
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "qr login complete",
    })
    .into_response()
}

pub(crate) async fn admin_auth_qr_result(
    State(state): State<AppState>,
    Query(query): Query<AdminQrResultQuery>,
) -> impl IntoResponse {
    if query.challenge_id.trim().is_empty() || query.session_id.trim().is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id and session_id are required",
        );
    }

    let now = Utc::now();
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);

    if let Some(result) = store.qr_login_results.get(query.challenge_id.trim()) {
        if result.session_id != query.session_id.trim() {
            return api_error(StatusCode::FORBIDDEN, 1003, "challenge session mismatch");
        }
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: AdminQrResultOutput {
                status: "SUCCESS".to_string(),
                message: "login success".to_string(),
                access_token: Some(result.access_token.clone()),
                expire_at: Some(result.expire_at.timestamp()),
                admin: Some(AdminIdentifyOutput {
                    admin_pubkey: result.admin_pubkey.clone(),
                    role: result.role.clone(),
                    status: result.status.clone(),
                    admin_name: {
                        if let Some(admin_user) =
                            store.admin_users_by_pubkey.get(&result.admin_pubkey)
                        {
                            let province =
                                province_scope_for_role(&store, &result.admin_pubkey, &result.role);
                            build_admin_display_name_from_user(admin_user, province.as_deref())
                        } else {
                            let province =
                                province_scope_for_role(&store, &result.admin_pubkey, &result.role);
                            build_admin_display_name(
                                &result.admin_pubkey,
                                &result.role,
                                province.as_deref(),
                            )
                        }
                    },
                    admin_province: province_scope_for_role(
                        &store,
                        &result.admin_pubkey,
                        &result.role,
                    ),
                    admin_city: store
                        .admin_users_by_pubkey
                        .get(&result.admin_pubkey)
                        .and_then(resolve_admin_city),
                }),
            },
        })
        .into_response();
    }

    let Some(challenge) = store.login_challenges.get(query.challenge_id.trim()) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found");
    };
    if challenge.session_id != query.session_id.trim() {
        return api_error(StatusCode::FORBIDDEN, 1003, "challenge session mismatch");
    }
    if now > challenge.expire_at {
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: AdminQrResultOutput {
                status: "EXPIRED".to_string(),
                message: "challenge expired".to_string(),
                access_token: None,
                expire_at: None,
                admin: None,
            },
        })
        .into_response();
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminQrResultOutput {
            status: "PENDING".to_string(),
            message: "waiting mobile scan".to_string(),
            access_token: None,
            expire_at: None,
            admin: None,
        },
    })
    .into_response()
}
