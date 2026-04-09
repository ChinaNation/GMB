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
use tracing::warn;
use uuid::Uuid;

use crate::business::pubkey::same_admin_pubkey;
use crate::business::scope::province_scope_for_role;
use crate::sfid::province::sheng_admin_display_name;
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
    store.admin_sessions.insert(
        access_token.clone(),
        AdminSession {
            token: access_token.clone(),
            admin_pubkey: admin_pubkey.clone(),
            role: admin_role.clone(),
            expire_at,
            last_active_at: now,
        },
    );

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
    let challenge_text = format!(
        "WUMIN_LOGIN_V1.0.0|{}|{}|{}",
        "sfid",
        challenge_id,
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
    let login_qr_payload = serde_json::json!({
        "proto": "WUMIN_LOGIN_V1.0.0",
        "type": "challenge",
        "system": "sfid",
        "challenge": challenge_id,
        "issued_at": now.timestamp(),
        "expires_at": expire_at.timestamp(),
        "sys_pubkey": sys_pubkey,
        "sys_sig": sys_sig
    })
    .to_string();

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

    let (challenge_text, session_id) = {
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
    if !verify_admin_signature(&verify_pubkey, &challenge_text, input.signature.trim()) {
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
    store.admin_sessions.insert(
        access_token.clone(),
        AdminSession {
            token: access_token.clone(),
            admin_pubkey: login_pubkey.clone(),
            role: login_role.clone(),
            expire_at,
            last_active_at: now,
        },
    );
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

fn admin_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminAuthContext, axum::response::Response> {
    if let Some(token) = bearer_token(headers) {
        let mut store = match store_write_or_500(state) {
            Ok(v) => v,
            Err(resp) => return Err(resp),
        };
        let now = Utc::now();
        let idle_timeout_minutes = std::env::var("SFID_ADMIN_IDLE_TIMEOUT_MINUTES")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(10);
        cleanup_admin_sessions(&mut store, now, idle_timeout_minutes);
        let (session_pubkey, _session_role) = {
            let Some(session) = store.admin_sessions.get_mut(&token) else {
                return Err(api_error(
                    StatusCode::UNAUTHORIZED,
                    1002,
                    "invalid access token",
                ));
            };
            if now > session.expire_at
                || now > session.last_active_at + Duration::minutes(idle_timeout_minutes)
            {
                store.admin_sessions.remove(&token);
                return Err(api_error(
                    StatusCode::UNAUTHORIZED,
                    1002,
                    "access token expired",
                ));
            }
            session.last_active_at = now;
            (session.admin_pubkey.clone(), session.role.clone())
        };
        let Some(admin_user) = store.admin_users_by_pubkey.get(&session_pubkey) else {
            return Err(api_error(StatusCode::FORBIDDEN, 2002, "admin not found"));
        };
        if admin_user.status != AdminStatus::Active {
            return Err(api_error(StatusCode::FORBIDDEN, 2003, "admin disabled"));
        }
        let admin_province =
            province_scope_for_role(&store, &admin_user.admin_pubkey, &admin_user.role);
        // 只对 ShiAdmin 暴露 city（其他角色底层字段为空字符串）
        let admin_city = if admin_user.role == AdminRole::ShiAdmin && !admin_user.city.is_empty()
        {
            Some(admin_user.city.clone())
        } else {
            None
        };
        return Ok(AdminAuthContext {
            admin_pubkey: admin_user.admin_pubkey.clone(),
            role: admin_user.role.clone(),
            admin_name: build_admin_display_name_from_user(admin_user, admin_province.as_deref()),
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
    let normalized_sig = normalize_hex(signature_text);
    let sig_bytes = match Vec::from_hex(&normalized_sig) {
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
    let message = format!(
        "WUMIN_LOGIN_V1.0.0|{}|{}|{}|{}|{}",
        system, challenge, issued_at, expires_at, sys_pubkey
    );
    let signer =
        crate::key_admins::chain_keyring::try_load_signing_key_from_seed(seed.expose_secret())?;
    let signature = signer.sign(message.as_bytes());
    Ok((sys_pubkey, format!("0x{}", hex::encode(signature.0))))
}

pub(crate) fn parse_sr25519_pubkey(admin_pubkey: &str) -> Option<String> {
    let normalized = normalize_hex(admin_pubkey);
    if normalized.len() == 64 && normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(normalized);
    }
    None
}

pub(crate) fn parse_sr25519_pubkey_bytes(admin_pubkey: &str) -> Option<[u8; 32]> {
    if let Some(hex_pubkey) = parse_sr25519_pubkey(admin_pubkey) {
        let bytes = Vec::from_hex(&hex_pubkey).ok()?;
        let arr: [u8; 32] = bytes.as_slice().try_into().ok()?;
        return Some(arr);
    }
    None
}

fn normalize_hex(value: &str) -> String {
    value
        .trim()
        .strip_prefix("0x")
        .or_else(|| value.trim().strip_prefix("0X"))
        .unwrap_or(value.trim())
        .to_string()
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

fn cleanup_admin_sessions(store: &mut Store, now: DateTime<Utc>, idle_timeout_minutes: i64) {
    store.admin_sessions.retain(|_, session| {
        now <= session.expire_at
            && now <= session.last_active_at + Duration::minutes(idle_timeout_minutes)
    });
    let max_sessions = bounded_cache_limit("SFID_ADMIN_SESSION_MAX", 50_000);
    if store.admin_sessions.len() > max_sessions {
        let mut entries = store
            .admin_sessions
            .iter()
            .map(|(token, session)| (token.clone(), session.last_active_at))
            .collect::<Vec<_>>();
        entries.sort_by_key(|(_, last_active)| *last_active);
        let overflow = store.admin_sessions.len() - max_sessions;
        for (token, _) in entries.into_iter().take(overflow) {
            store.admin_sessions.remove(&token);
        }
    }
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
    if admin.role == AdminRole::ShiAdmin {
        let name = admin.admin_name.trim();
        if !name.is_empty() {
            return name.to_string();
        }
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
