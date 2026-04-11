use axum::{
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use hex::FromHex;
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};
use tracing::warn;
use uuid::Uuid;

use crate::business::pubkey::same_admin_pubkey;
use crate::business::scope::province_scope_for_role;
use crate::sfid::province::sheng_admin_display_name;
use crate::sfid::province::sheng_admin_province;
use crate::*;

const LOGIN_CHALLENGE_TTL_SECONDS: i64 = 90;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LoginChallenge {
    pub(crate) challenge_id: String,
    pub(crate) admin_pubkey: String,
    pub(crate) challenge_text: String,
    pub(crate) challenge_token: String,
    pub(crate) qr_aud: String,
    pub(crate) qr_origin: String,
    pub(crate) origin: String,
    pub(crate) domain: String,
    pub(crate) session_id: String,
    pub(crate) nonce: String,
    pub(crate) issued_at: DateTime<Utc>,
    pub(crate) expire_at: DateTime<Utc>,
    pub(crate) consumed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminSession {
    pub(crate) token: String,
    pub(crate) admin_pubkey: String,
    pub(crate) role: AdminRole,
    pub(crate) expire_at: DateTime<Utc>,
    #[serde(default = "default_now_utc")]
    pub(crate) last_active_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QrLoginResultRecord {
    pub(crate) session_id: String,
    pub(crate) access_token: String,
    pub(crate) expire_at: DateTime<Utc>,
    pub(crate) admin_pubkey: String,
    pub(crate) role: AdminRole,
    pub(crate) status: AdminStatus,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdminAuthContext {
    pub(crate) admin_pubkey: String,
    pub(crate) role: AdminRole,
    pub(crate) admin_name: String,
    pub(crate) admin_province: Option<String>,
    /// 仅 ShiAdmin 有值：该操作员登记的市（用于多签列表按市过滤、生成时强制锁定）
    pub(crate) admin_city: Option<String>,
}

#[derive(Serialize)]
struct AdminAuthOutput {
    ok: bool,
    admin_pubkey: String,
    role: AdminRole,
    admin_name: String,
    admin_province: Option<String>,
    admin_city: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminIdentifyInput {
    identity_qr: String,
}

#[derive(Serialize)]
struct AdminIdentifyOutput {
    admin_pubkey: String,
    role: AdminRole,
    status: AdminStatus,
    admin_name: String,
    admin_province: Option<String>,
    admin_city: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminChallengeInput {
    admin_pubkey: String,
    origin: Option<String>,
    domain: Option<String>,
    session_id: Option<String>,
}

#[derive(Serialize)]
struct AdminChallengeOutput {
    challenge_id: String,
    challenge_payload: String,
    origin: String,
    domain: String,
    session_id: String,
    nonce: String,
    expire_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct AdminQrChallengeInput {
    pub(crate) origin: Option<String>,
    pub(crate) domain: Option<String>,
    pub(crate) session_id: Option<String>,
}

#[derive(Serialize)]
struct AdminQrChallengeOutput {
    challenge_id: String,
    challenge_payload: String,
    login_qr_payload: String,
    origin: String,
    domain: String,
    session_id: String,
    expire_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct AdminQrCompleteInput {
    #[serde(alias = "request_id", alias = "challenge")]
    pub(crate) challenge_id: String,
    pub(crate) session_id: Option<String>,
    pub(crate) admin_pubkey: String,
    #[serde(default, alias = "pubkey", alias = "public_key")]
    pub(crate) signer_pubkey: Option<String>,
    pub(crate) signature: String,
}

#[derive(Deserialize)]
pub(crate) struct AdminQrResultQuery {
    #[serde(alias = "challenge")]
    pub(crate) challenge_id: String,
    pub(crate) session_id: String,
}

#[derive(Serialize)]
struct AdminQrResultOutput {
    status: String,
    message: String,
    access_token: Option<String>,
    expire_at: Option<i64>,
    admin: Option<AdminIdentifyOutput>,
}

#[derive(Deserialize)]
pub(crate) struct AdminVerifyInput {
    challenge_id: String,
    origin: String,
    domain: Option<String>,
    session_id: String,
    nonce: String,
    signature: String,
}

#[derive(Serialize)]
struct AdminVerifyOutput {
    access_token: String,
    expire_at: i64,
    admin: AdminIdentifyOutput,
}

fn default_now_utc() -> DateTime<Utc> {
    Utc::now()
}

pub(crate) async fn require_admin_session_middleware(
    State(state): State<AppState>,
    request: Request,
    next: middleware::Next,
) -> Response {
    if let Err(resp) = admin_auth(&state, request.headers()) {
        return resp;
    }
    next.run(request).await
}

pub(crate) async fn admin_auth_check(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let ctx = match admin_auth(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminAuthOutput {
            ok: true,
            admin_pubkey: ctx.admin_pubkey,
            role: ctx.role,
            admin_name: ctx.admin_name,
            admin_province: ctx.admin_province,
            admin_city: ctx.admin_city,
        },
    })
    .into_response()
}

/// 主动登出:从 GlobalShard 删除当前 session。
pub(crate) async fn admin_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = match bearer_token(&headers) {
        Some(t) => t,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "missing token"),
    };
    let _ = state
        .sharded_store
        .write_global(|g| {
            g.admin_sessions.remove(&token);
        })
        .await;
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "logged out",
    })
    .into_response()
}

pub(crate) async fn admin_auth_identify(
    State(state): State<AppState>,
    Json(input): Json<AdminIdentifyInput>,
) -> impl IntoResponse {
    let admin_pubkey = parse_admin_identity_qr(&input.identity_qr);
    if admin_pubkey.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "identity_qr is required");
    }

    let store = match state.store.read() {
        Ok(guard) => guard,
        Err(err) => {
            warn!("store read failed in /api/v1/admin/auth/identify: {}", err);
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, "store unavailable");
        }
    };
    let Some(admin) = store.admin_users_by_pubkey.get(&admin_pubkey) else {
        return api_error(StatusCode::FORBIDDEN, 2002, "admin not found");
    };
    if admin.status != AdminStatus::Active {
        return api_error(StatusCode::FORBIDDEN, 2003, "admin disabled");
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminIdentifyOutput {
            admin_pubkey: admin.admin_pubkey.clone(),
            role: admin.role.clone(),
            status: admin.status.clone(),
            admin_name: {
                let province = province_scope_for_role(&store, &admin.admin_pubkey, &admin.role);
                build_admin_display_name_from_user(admin, province.as_deref())
            },
            admin_province: province_scope_for_role(&store, &admin.admin_pubkey, &admin.role),
            admin_city: resolve_admin_city(admin),
        },
    })
    .into_response()
}

pub(crate) async fn admin_auth_challenge(
    State(state): State<AppState>,
    Json(input): Json<AdminChallengeInput>,
) -> impl IntoResponse {
    if input.admin_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "admin_pubkey is required");
    }
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
    let nonce = Uuid::new_v4().to_string();
    let challenge_text = format!(
        "sfid-login|pubkey={}|origin={}|domain={}|session_id={}|nonce={}|iat={}|exp={}",
        input.admin_pubkey,
        origin,
        derived_domain,
        session_id,
        nonce,
        now.timestamp(),
        expire_at.timestamp()
    );

    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);
    let Some(admin) = store.admin_users_by_pubkey.get(&input.admin_pubkey) else {
        return api_error(StatusCode::FORBIDDEN, 2002, "admin not found");
    };
    if admin.status != AdminStatus::Active {
        return api_error(StatusCode::FORBIDDEN, 2003, "admin disabled");
    }

    insert_bounded_map(
        &mut store.login_challenges,
        challenge_id.clone(),
        LoginChallenge {
            challenge_id: challenge_id.clone(),
            admin_pubkey: input.admin_pubkey,
            challenge_text: challenge_text.clone(),
            challenge_token: String::new(),
            qr_aud: String::new(),
            qr_origin: String::new(),
            origin: origin.clone(),
            domain: derived_domain.clone(),
            session_id: session_id.clone(),
            nonce: nonce.clone(),
            issued_at: now,
            expire_at,
            consumed: false,
        },
        bounded_cache_limit("SFID_LOGIN_CHALLENGE_MAX", 20_000),
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminChallengeOutput {
            challenge_id,
            challenge_payload: challenge_text,
            origin,
            domain: derived_domain,
            session_id,
            nonce,
            expire_at: expire_at.timestamp(),
        },
    })
    .into_response()
}

pub(crate) async fn admin_auth_verify(
    State(state): State<AppState>,
    Json(input): Json<AdminVerifyInput>,
) -> impl IntoResponse {
    if input.challenge_id.trim().is_empty()
        || input.signature.trim().is_empty()
        || input.origin.trim().is_empty()
        || input.session_id.trim().is_empty()
        || input.nonce.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, origin, session_id, nonce, signature are required",
        );
    }
    let verify_domain = input
        .domain
        .clone()
        .or_else(|| extract_domain_from_origin(&input.origin))
        .unwrap_or_default();
    if verify_domain.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "domain is required");
    }

    let now = Utc::now();
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_expired_challenges(&mut store, now);
    let admin_pubkey = {
        let Some(challenge) = store.login_challenges.get_mut(&input.challenge_id) else {
            return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found");
        };
        if challenge.consumed {
            return api_error(StatusCode::CONFLICT, 1007, "challenge already consumed");
        }
        if now > challenge.expire_at {
            return api_error(StatusCode::UNAUTHORIZED, 1007, "challenge expired");
        }
        if challenge.origin != input.origin
            || challenge.domain != verify_domain
            || challenge.session_id != input.session_id
            || challenge.nonce != input.nonce
        {
            return api_error(StatusCode::UNAUTHORIZED, 2004, "challenge context mismatch");
        }

        // 中文注释：乐观消费——先标记 consumed 再验签，防止并发请求同时通过 consumed 检查。
        // 验签失败时回退 consumed = false。
        challenge.consumed = true;
        let admin_pubkey = challenge.admin_pubkey.clone();
        let challenge_text = challenge.challenge_text.clone();

        if !verify_admin_signature(&admin_pubkey, &challenge_text, input.signature.trim()) {
            if let Some(c) = store.login_challenges.get_mut(&input.challenge_id) {
                c.consumed = false;
            }
            return api_error(StatusCode::UNAUTHORIZED, 2004, "signature verify failed");
        }
        admin_pubkey
    };

    let admin = match store.admin_users_by_pubkey.get(&admin_pubkey) {
        Some(v) => v,
        None => return api_error(StatusCode::FORBIDDEN, 2002, "admin not found"),
    };
    if admin.status != AdminStatus::Active {
        return api_error(StatusCode::FORBIDDEN, 2003, "admin disabled");
    }
    let admin_pubkey = admin.admin_pubkey.clone();
    let admin_role = admin.role.clone();
    let admin_status = admin.status.clone();
    let admin_province = province_scope_for_role(&store, &admin_pubkey, &admin_role);
    let admin_name = build_admin_display_name_from_user(admin, admin_province.as_deref());
    let admin_city = resolve_admin_city(admin);

    let access_token = Uuid::new_v4().to_string();
    let expire_at = now + Duration::hours(8);
    let new_session = AdminSession {
        token: access_token.clone(),
        admin_pubkey: admin_pubkey.clone(),
        role: admin_role.clone(),
        expire_at,
        last_active_at: now,
    };
    store.admin_sessions.insert(access_token.clone(), new_session.clone());
    // 中文注释：先释放写锁，再执行 bootstrap_sheng_signer（含链上推送），避免
    // 跨 await 持有 StoreWriteGuard。
    drop(store);

    // Phase 2 admin_auth 迁移:登录成功后同步写 GlobalShard session
    {
        let ss = state.sharded_store.clone();
        let token_for_shard = access_token.clone();
        tokio::task::spawn(async move {
            let _ = ss
                .write_global(|g| {
                    g.admin_sessions.insert(token_for_shard, new_session);
                })
                .await;
        });
    }

    // 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 步骤 7：
    // 省登录管理员验签成功后确保本省签名密钥就绪。
    if admin_role == AdminRole::ShengAdmin {
        if let Some(province) = admin_province.as_deref() {
            if let Err(e) = crate::key_admins::bootstrap_sheng_signer(
                &state,
                admin_pubkey.as_str(),
                province,
            )
            .await
            {
                tracing::error!(province, error = %e, "BOOTSTRAP FAILED: {}", e);
            } else {
                tracing::info!(province, "sheng signer ready for province");
            }
        }
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminVerifyOutput {
            access_token,
            expire_at: expire_at.timestamp(),
            admin: AdminIdentifyOutput {
                admin_pubkey,
                role: admin_role,
                status: admin_status,
                admin_name,
                admin_province,
                admin_city,
            },
        },
    })
    .into_response()
}

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
    store.admin_sessions.insert(access_token.clone(), new_session_qr.clone());

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
    let bootstrap_province =
        province_scope_for_role(&store, &login_pubkey, &login_role);
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
            if let Err(e) = crate::key_admins::bootstrap_sheng_signer(
                &state,
                bootstrap_pubkey.as_str(),
                province,
            )
            .await
            {
                tracing::error!(province, error = %e, "BOOTSTRAP FAILED (qr): {}", e);
            } else {
                tracing::info!(province, "sheng signer ready for province (qr)");
            }
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

/// Phase 2 admin_auth 迁移到 GlobalShard：
/// session 验证 + 用户查找全部从 sharded_store.read_global 同步读取,
/// 不再 lock legacy store 的写锁。write_global(async) 用 tokio::task::spawn
/// 后台执行,不阻塞 auth 返回。
fn admin_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    if let Some(token) = bearer_token(headers) {
        let now = Utc::now();
        // ShiAdmin idle 超时(分钟),KeyAdmin/ShengAdmin 无 idle 限制
        let shi_idle_timeout_minutes = std::env::var("SFID_ADMIN_IDLE_TIMEOUT_MINUTES")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(10);

        // ── Phase 2:后台节流清理(每 60 秒一次,不阻塞请求) ──
        static LAST_CLEANUP: AtomicI64 = AtomicI64::new(0);
        let last = LAST_CLEANUP.load(Ordering::Relaxed);
        let now_ts = now.timestamp();
        if now_ts - last > 60 {
            LAST_CLEANUP.store(now_ts, Ordering::Relaxed);
            let ss = state.sharded_store.clone();
            let cache = state.sheng_signer_cache.clone();
            tokio::task::spawn(async move {
                if let Ok(evicted) =
                    cleanup_sessions_from_global(&ss, Utc::now(), shi_idle_timeout_minutes).await
                {
                    for province in evicted {
                        cache.unload_province(province.as_str());
                    }
                }
            });
        }

        // ── 1. 从 GlobalShard 同步读 session ──
        let session = state
            .sharded_store
            .read_global(|g| g.admin_sessions.get(&token).cloned())
            .map_err(|e| {
                warn!(error = %e, "read_global failed in admin_auth");
                api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e)
            })?;

        let Some(session) = session else {
            return Err(api_error(
                StatusCode::UNAUTHORIZED,
                1002,
                "invalid access token",
            ));
        };

        // ── 2. 验证过期 / idle 超时 ──
        // KeyAdmin/ShengAdmin 无 idle 限制,仅检查 expire_at(8h);
        // ShiAdmin 额外检查 idle 超时(默认 10 分钟)。
        let idle_expired = session.role == AdminRole::ShiAdmin
            && now > session.last_active_at + Duration::minutes(shi_idle_timeout_minutes);
        if now > session.expire_at || idle_expired {
            // 过期:后台异步删除 session(write_global 是 async)
            let ss = state.sharded_store.clone();
            let token_clone = token.clone();
            tokio::task::spawn(async move {
                let _ = ss
                    .write_global(|g| {
                        g.admin_sessions.remove(&token_clone);
                    })
                    .await;
            });
            return Err(api_error(
                StatusCode::UNAUTHORIZED,
                1002,
                "access token expired",
            ));
        }

        // ── 3. 后台更新 last_active_at(不阻塞返回) ──
        {
            let ss = state.sharded_store.clone();
            let token_clone = token.clone();
            tokio::task::spawn(async move {
                let _ = ss
                    .write_global(|g| {
                        if let Some(s) = g.admin_sessions.get_mut(&token_clone) {
                            s.last_active_at = Utc::now();
                        }
                    })
                    .await;
            });
        }

        let session_pubkey = session.admin_pubkey.clone();

        // ── 4. 查用户信息:优先 GlobalShard,fallback legacy store ──
        // GlobalShard.global_admins 包含 KeyAdmin + ShengAdmin;
        // ShiAdmin 可能还未同步到 GlobalShard,fallback legacy。
        let user_info = state
            .sharded_store
            .read_global(|g| {
                if let Some(user) = g.global_admins.get(&session_pubkey) {
                    let province = match &user.role {
                        AdminRole::KeyAdmin => None,
                        AdminRole::ShengAdmin => g
                            .sheng_admin_province_by_pubkey
                            .get(&session_pubkey)
                            .cloned()
                            .or_else(|| {
                                sheng_admin_province(&session_pubkey).map(|v| v.to_string())
                            }),
                        AdminRole::ShiAdmin => {
                            let creator = &user.created_by;
                            g.sheng_admin_province_by_pubkey
                                .get(creator)
                                .cloned()
                                .or_else(|| {
                                    sheng_admin_province(creator).map(|v| v.to_string())
                                })
                        }
                    };
                    let city = if user.role == AdminRole::ShiAdmin && !user.city.is_empty() {
                        Some(user.city.clone())
                    } else {
                        None
                    };
                    return Some((
                        user.admin_pubkey.clone(),
                        user.role.clone(),
                        user.status.clone(),
                        user.admin_name.clone(),
                        user.city.clone(),
                        user.created_by.clone(),
                        province,
                        city,
                    ));
                }
                None
            })
            .map_err(|e| {
                warn!(error = %e, "read_global failed for admin user lookup");
                api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, &e)
            })?;

        // GlobalShard 命中:KeyAdmin / ShengAdmin(或已同步的 ShiAdmin)
        // 未命中:fallback legacy store(ShiAdmin 可能尚未同步到 GlobalShard)
        let (admin_pubkey, role, status, admin_name, _city_raw, _created_by, admin_province, admin_city) =
            if let Some(info) = user_info {
                info
            } else {
                // fallback:从 legacy store 读(只拿读锁)
                let store = store_read_or_500(state)?;
                let Some(user) = store.admin_users_by_pubkey.get(&session_pubkey) else {
                    return Err(api_error(StatusCode::FORBIDDEN, 2002, "admin not found"));
                };
                let province =
                    province_scope_for_role(&store, &user.admin_pubkey, &user.role);
                let city = if user.role == AdminRole::ShiAdmin && !user.city.is_empty() {
                    Some(user.city.clone())
                } else {
                    None
                };
                (
                    user.admin_pubkey.clone(),
                    user.role.clone(),
                    user.status.clone(),
                    user.admin_name.clone(),
                    user.city.clone(),
                    user.created_by.clone(),
                    province,
                    city,
                )
            };

        if status != AdminStatus::Active {
            return Err(api_error(StatusCode::FORBIDDEN, 2003, "admin disabled"));
        }

        // 三角色统一:优先使用 admin_name(真实姓名),空则 fallback 到角色默认名
        let display_name = {
            let name = admin_name.trim();
            if !name.is_empty() {
                name.to_string()
            } else {
                build_admin_display_name(&admin_pubkey, &role, admin_province.as_deref())
            }
        };

        return Ok(AdminAuthContext {
            admin_pubkey,
            role,
            admin_name: display_name,
            admin_province,
            admin_city,
        });
    }

    Err(api_error(
        StatusCode::UNAUTHORIZED,
        1002,
        "admin auth required",
    ))
}

/// Phase 2:异步清理 GlobalShard 中的过期 session。
/// 由 admin_auth 里的 60 秒节流触发,后台 tokio::task::spawn 执行。
/// KeyAdmin/ShengAdmin 无 idle 限制(仅 expire_at),ShiAdmin 额外检查 idle。
async fn cleanup_sessions_from_global(
    store: &std::sync::Arc<crate::store_shards::ShardedStore>,
    now: DateTime<Utc>,
    shi_idle_timeout_minutes: i64,
) -> Result<Vec<String>, String> {
    let mut evicted_provinces: Vec<String> = Vec::new();
    store
        .write_global(|g| {
            let mut evicted_sheng_pubkeys: Vec<String> = Vec::new();
            let mut remaining_sheng_pubkeys: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            g.admin_sessions.retain(|_, session| {
                // expire_at 硬上限对所有角色生效
                if now > session.expire_at {
                    if session.role == AdminRole::ShengAdmin {
                        evicted_sheng_pubkeys.push(session.admin_pubkey.clone());
                    }
                    return false;
                }
                // idle 超时仅 ShiAdmin
                if session.role == AdminRole::ShiAdmin
                    && now > session.last_active_at + Duration::minutes(shi_idle_timeout_minutes)
                {
                    return false;
                }
                if session.role == AdminRole::ShengAdmin {
                    remaining_sheng_pubkeys.insert(session.admin_pubkey.clone());
                }
                true
            });

            let max_sessions = bounded_cache_limit("SFID_ADMIN_SESSION_MAX", 50_000);
            if g.admin_sessions.len() > max_sessions {
                let mut entries = g
                    .admin_sessions
                    .iter()
                    .map(|(token, session)| {
                        (
                            token.clone(),
                            session.last_active_at,
                            session.role.clone(),
                            session.admin_pubkey.clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                entries.sort_by_key(|(_, last_active, _, _)| *last_active);
                let overflow = g.admin_sessions.len() - max_sessions;
                for (token, _, role, pubkey) in entries.into_iter().take(overflow) {
                    g.admin_sessions.remove(&token);
                    if role == AdminRole::ShengAdmin {
                        evicted_sheng_pubkeys.push(pubkey);
                    }
                }
                remaining_sheng_pubkeys.clear();
                for (_, s) in g.admin_sessions.iter() {
                    if s.role == AdminRole::ShengAdmin {
                        remaining_sheng_pubkeys.insert(s.admin_pubkey.clone());
                    }
                }
            }

            for pubkey in evicted_sheng_pubkeys {
                if remaining_sheng_pubkeys.contains(&pubkey) {
                    continue;
                }
                if let Some(province) = g.sheng_admin_province_by_pubkey.get(&pubkey) {
                    evicted_provinces.push(province.clone());
                }
            }
        })
        .await?;
    Ok(evicted_provinces)
}

pub(crate) fn require_admin_any(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    admin_auth(state, headers)
}

pub(crate) fn require_admin_write(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    admin_auth(state, headers)
}

pub(crate) fn require_institution_or_key_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if !matches!(ctx.role, AdminRole::ShengAdmin | AdminRole::KeyAdmin) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "institution admin or key admin required",
        ));
    }
    if ctx.role == AdminRole::ShengAdmin && ctx.admin_province.is_none() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        ));
    }
    Ok(ctx)
}

pub(crate) fn require_key_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    let ctx = admin_auth(state, headers)?;
    if ctx.role != AdminRole::KeyAdmin {
        return Err(api_error(StatusCode::FORBIDDEN, 1003, "key admin required"));
    }
    Ok(ctx)
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get("authorization")?.to_str().ok()?.trim();
    let token = auth.strip_prefix("Bearer ")?;
    if token.trim().is_empty() {
        return None;
    }
    Some(token.trim().to_string())
}

pub(crate) fn verify_admin_signature(
    admin_pubkey: &str,
    message: &str,
    signature_text: &str,
) -> bool {
    let Some(pubkey_bytes) = parse_sr25519_pubkey_bytes(admin_pubkey) else {
        return false;
    };
    let normalized_sig = strip_0x_prefix(signature_text);
    let sig_bytes = match Vec::from_hex(normalized_sig) {
        Ok(v) if v.len() == 64 => v,
        _ => return false,
    };
    let sig_arr: [u8; 64] = match sig_bytes.as_slice().try_into() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let pubkey = match Sr25519PublicKey::from_bytes(&pubkey_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let signature = match Sr25519Signature::from_bytes(&sig_arr) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let ctx = signing_context(b"substrate");
    if pubkey
        .verify(ctx.bytes(message.as_bytes()), &signature)
        .is_ok()
    {
        return true;
    }
    let wrapped = format!("<Bytes>{}</Bytes>", message);
    pubkey
        .verify(ctx.bytes(wrapped.as_bytes()), &signature)
        .is_ok()
}

fn build_login_qr_system_signature(
    state: &AppState,
    system: &str,
    challenge: &str,
    issued_at: i64,
    expires_at: i64,
) -> Result<(String, String), String> {
    let sys_pubkey = state
        .public_key_hex
        .read()
        .map_err(|_| "public key read lock poisoned".to_string())?
        .clone();
    let seed = state
        .signing_seed_hex
        .read()
        .map_err(|_| "signing seed read lock poisoned".to_string())?
        .clone();
    let message = crate::qr::build_signature_message(
        crate::qr::QrKind::LoginChallenge,
        challenge,
        Some(system),
        Some(expires_at),
        &sys_pubkey,
    );
    let _ = issued_at; // 统一签名原文不再包含 issued_at
    let signer =
        crate::key_admins::chain_keyring::try_load_signing_key_from_seed(seed.expose_secret())?;
    let signature = signer.sign(message.as_bytes());
    Ok((sys_pubkey, format!("0x{}", hex::encode(signature.0))))
}

/// 解析 Sr25519 公钥，返回统一格式 `0x` + 64 位小写 hex。
pub(crate) fn parse_sr25519_pubkey(admin_pubkey: &str) -> Option<String> {
    let raw = admin_pubkey
        .trim()
        .strip_prefix("0x")
        .or_else(|| admin_pubkey.trim().strip_prefix("0X"))
        .unwrap_or(admin_pubkey.trim());
    if raw.len() == 64 && raw.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(format!("0x{}", raw.to_ascii_lowercase()));
    }
    None
}

pub(crate) fn parse_sr25519_pubkey_bytes(admin_pubkey: &str) -> Option<[u8; 32]> {
    if let Some(hex_pubkey) = parse_sr25519_pubkey(admin_pubkey) {
        // hex::decode 不接受 0x 前缀，去掉后解码
        let bytes = Vec::from_hex(strip_0x_prefix(&hex_pubkey)).ok()?;
        let arr: [u8; 32] = bytes.as_slice().try_into().ok()?;
        return Some(arr);
    }
    None
}

/// 去掉 0x/0X 前缀，仅用于 hex::decode 前的临时处理，不用于存储。
fn strip_0x_prefix(value: &str) -> &str {
    value
        .trim()
        .strip_prefix("0x")
        .or_else(|| value.trim().strip_prefix("0X"))
        .unwrap_or(value.trim())
}

fn resolve_admin_pubkey_key(store: &Store, candidate: &str) -> Option<String> {
    store
        .admin_users_by_pubkey
        .keys()
        .find(|pubkey| same_admin_pubkey(pubkey.as_str(), candidate))
        .cloned()
}

fn parse_admin_identity_qr(identity_qr: &str) -> String {
    let trimmed = identity_qr.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('{') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(v) = value
                .get("admin_pubkey")
                .or_else(|| value.get("pubkey"))
                .and_then(|v| v.as_str())
            {
                return v.trim().to_string();
            }
        }
    }
    trimmed.to_string()
}

fn extract_domain_from_origin(origin: &str) -> Option<String> {
    let trimmed = origin.trim();
    if trimmed.is_empty() {
        return None;
    }
    let no_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let host_port = no_scheme.split('/').next().unwrap_or("");
    if host_port.is_empty() {
        return None;
    }
    let domain = host_port.split(':').next().unwrap_or("");
    if domain.is_empty() {
        return None;
    }
    Some(domain.to_string())
}

fn cleanup_expired_challenges(store: &mut Store, now: DateTime<Utc>) {
    store.login_challenges.retain(|_, c| {
        c.expire_at > now - Duration::minutes(10) && (!c.consumed || c.expire_at > now)
    });
    store.qr_login_results.retain(|_, r| {
        r.created_at > now - Duration::hours(1) && r.expire_at > now - Duration::minutes(10)
    });
}

/// 中文注释：清理过期/空闲超时的 admin session。
///
/// 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 步骤 8：
/// 返回本次被驱逐的 ShengAdmin session 所属 province 列表，供外层调用
/// `state.sheng_signer_cache.unload_province` 释放内存 Pair。
#[allow(dead_code)]
fn cleanup_admin_sessions(
    store: &mut Store,
    now: DateTime<Utc>,
    idle_timeout_minutes: i64,
) -> Vec<String> {
    let mut evicted_sheng_provinces: Vec<String> = Vec::new();
    let mut remaining_sheng_pubkeys: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // 先收集"被驱逐"的 sheng admin pubkey。
    let mut evicted_sheng_pubkeys: Vec<String> = Vec::new();
    store.admin_sessions.retain(|_, session| {
        let keep = now <= session.expire_at
            && now <= session.last_active_at + Duration::minutes(idle_timeout_minutes);
        if !keep && session.role == AdminRole::ShengAdmin {
            evicted_sheng_pubkeys.push(session.admin_pubkey.clone());
        }
        if keep && session.role == AdminRole::ShengAdmin {
            remaining_sheng_pubkeys.insert(session.admin_pubkey.clone());
        }
        keep
    });

    let max_sessions = bounded_cache_limit("SFID_ADMIN_SESSION_MAX", 50_000);
    if store.admin_sessions.len() > max_sessions {
        let mut entries = store
            .admin_sessions
            .iter()
            .map(|(token, session)| {
                (token.clone(), session.last_active_at, session.role.clone(), session.admin_pubkey.clone())
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|(_, last_active, _, _)| *last_active);
        let overflow = store.admin_sessions.len() - max_sessions;
        for (token, _, role, pubkey) in entries.into_iter().take(overflow) {
            store.admin_sessions.remove(&token);
            if role == AdminRole::ShengAdmin {
                evicted_sheng_pubkeys.push(pubkey);
            }
        }
        // 重新计算 remaining_sheng_pubkeys
        remaining_sheng_pubkeys.clear();
        for (_, s) in store.admin_sessions.iter() {
            if s.role == AdminRole::ShengAdmin {
                remaining_sheng_pubkeys.insert(s.admin_pubkey.clone());
            }
        }
    }

    // 只有当该 sheng admin 所有 session 都被清掉时，才驱逐本省 cache。
    for pubkey in evicted_sheng_pubkeys {
        if remaining_sheng_pubkeys.contains(&pubkey) {
            continue;
        }
        if let Some(province) = store.sheng_admin_province_by_pubkey.get(&pubkey) {
            evicted_sheng_provinces.push(province.clone());
        }
    }
    evicted_sheng_provinces
}

pub(crate) fn build_admin_display_name(
    admin_pubkey: &str,
    role: &AdminRole,
    admin_province: Option<&str>,
) -> String {
    if *role == AdminRole::ShengAdmin {
        if let Some(province) = admin_province {
            return format!("{province}机构管理员");
        }
    }
    if let Some(name) = sheng_admin_display_name(admin_pubkey) {
        return name;
    }
    match role {
        AdminRole::KeyAdmin => "密钥管理员".to_string(),
        AdminRole::ShiAdmin => "系统管理员".to_string(),
        AdminRole::ShengAdmin => "机构管理员".to_string(),
    }
}

pub(crate) fn build_admin_display_name_from_user(
    admin: &AdminUser,
    admin_province: Option<&str>,
) -> String {
    // 三角色统一:优先使用 admin_name(真实姓名),空则 fallback 到角色默认名
    let name = admin.admin_name.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    build_admin_display_name(&admin.admin_pubkey, &admin.role, admin_province)
}

/// 仅 ShiAdmin 暴露 admin_city，其他角色或空字符串一律返回 None。
pub(crate) fn resolve_admin_city(admin: &AdminUser) -> Option<String> {
    if admin.role == AdminRole::ShiAdmin && !admin.city.trim().is_empty() {
        Some(admin.city.clone())
    } else {
        None
    }
}
