use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use schnorrkel::{signing_context, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    authz, err, find_admin_by_pubkey, ok, persist_runtime_store, write_audit, ApiError,
    ApiResponse, AppState,
};

pub(crate) const TOKEN_EXPIRES_SECONDS: i64 = 30 * 60;
const CHALLENGE_EXPIRES_SECONDS: i64 = 90;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Session {
    pub(crate) user_id: String,
    pub(crate) role: String,
    pub(crate) expires_at: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct LoginChallenge {
    pub(crate) admin_pubkey: String,
    pub(crate) challenge_payload: String,
    pub(crate) session_id: String,
    pub(crate) expire_at: i64,
    pub(crate) consumed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct QrLoginResult {
    pub(crate) session_id: String,
    pub(crate) access_token: String,
    pub(crate) expires_in: i64,
    pub(crate) user: SessionUser,
    pub(crate) created_at: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SessionUser {
    user_id: String,
    role: String,
}

#[derive(Deserialize)]
struct IdentifyRequest {
    admin_pubkey: String,
}

#[derive(Serialize)]
struct IdentifyData {
    user_id: String,
    role: String,
    status: String,
}

#[derive(Deserialize)]
struct ChallengeRequest {
    admin_pubkey: String,
}

#[derive(Serialize)]
struct ChallengeData {
    challenge_id: String,
    challenge_payload: String,
    nonce: String,
    expire_at: i64,
}

#[derive(Deserialize)]
struct VerifyRequest {
    challenge_id: String,
    admin_pubkey: String,
    signature: String,
}

#[derive(Serialize)]
struct VerifyData {
    access_token: String,
    expires_in: i64,
    user: SessionUser,
}

#[derive(Deserialize)]
struct QrChallengeRequest {
    origin: Option<String>,
    domain: Option<String>,
    session_id: Option<String>,
}

#[derive(Serialize)]
struct QrChallengeData {
    challenge_id: String,
    challenge_payload: String,
    login_qr_payload: String,
    origin: String,
    domain: String,
    session_id: String,
    nonce: String,
    expire_at: i64,
}

#[derive(Deserialize)]
struct QrCompleteRequest {
    challenge_id: String,
    session_id: String,
    admin_pubkey: String,
    signature: String,
}

#[derive(Deserialize)]
struct QrResultQuery {
    challenge_id: String,
    session_id: String,
}

#[derive(Serialize)]
struct QrResultData {
    status: String,
    message: String,
    access_token: Option<String>,
    expires_in: Option<i64>,
    user: Option<SessionUser>,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/auth/identify", post(auth_identify))
        .route("/api/v1/admin/auth/challenge", post(auth_challenge))
        .route("/api/v1/admin/auth/verify", post(auth_verify))
        .route("/api/v1/admin/auth/qr/challenge", post(auth_qr_challenge))
        .route("/api/v1/admin/auth/qr/complete", post(auth_qr_complete))
        .route("/api/v1/admin/auth/qr/result", get(auth_qr_result))
        .route("/api/v1/admin/auth/logout", post(auth_logout))
}

async fn auth_identify(
    State(state): State<AppState>,
    Json(req): Json<IdentifyRequest>,
) -> Result<Json<ApiResponse<IdentifyData>>, (StatusCode, Json<ApiError>)> {
    let admin = find_admin_by_pubkey(&state, &req.admin_pubkey).await?;
    if admin.status != "ACTIVE" {
        write_audit(
            &state,
            None,
            "AUTH_IDENTIFY",
            "ADMIN_USER",
            Some(admin.user_id.clone()),
            "FAILED",
            serde_json::json!({"reason": "inactive"}),
        )
        .await?;
        return Err(err(StatusCode::UNAUTHORIZED, 2002, "admin is not active"));
    }

    write_audit(
        &state,
        Some(admin.user_id.clone()),
        "AUTH_IDENTIFY",
        "ADMIN_USER",
        Some(admin.user_id.clone()),
        "SUCCESS",
        serde_json::json!({}),
    )
    .await?;

    Ok(Json(ok(IdentifyData {
        user_id: admin.user_id,
        role: admin.role,
        status: admin.status,
    })))
}

async fn auth_challenge(
    State(state): State<AppState>,
    Json(req): Json<ChallengeRequest>,
) -> Result<Json<ApiResponse<ChallengeData>>, (StatusCode, Json<ApiError>)> {
    let admin = find_admin_by_pubkey(&state, &req.admin_pubkey).await?;
    if admin.status != "ACTIVE" {
        return Err(err(StatusCode::UNAUTHORIZED, 2002, "admin is not active"));
    }

    let challenge_id = format!("chl_{}", Uuid::new_v4().simple());
    let nonce = Uuid::new_v4().simple().to_string();
    let expire_at = (Utc::now() + Duration::seconds(CHALLENGE_EXPIRES_SECONDS)).timestamp();
    let challenge_payload = format!(
        "cpms-admin-auth-v1|{}|{}|{}|{}",
        challenge_id, req.admin_pubkey, nonce, expire_at
    );

    let challenge = LoginChallenge {
        admin_pubkey: req.admin_pubkey,
        challenge_payload: challenge_payload.clone(),
        session_id: challenge_id.clone(),
        expire_at,
        consumed: false,
    };

    state
        .login_challenges
        .write()
        .await
        .insert(challenge_id.clone(), challenge);
    persist_runtime_store(&state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;

    write_audit(
        &state,
        Some(admin.user_id.clone()),
        "AUTH_CHALLENGE",
        "LOGIN_CHALLENGE",
        Some(challenge_id.clone()),
        "SUCCESS",
        serde_json::json!({"expire_at": expire_at}),
    )
    .await?;

    Ok(Json(ok(ChallengeData {
        challenge_id,
        challenge_payload,
        nonce,
        expire_at,
    })))
}

async fn auth_verify(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<ApiResponse<VerifyData>>, (StatusCode, Json<ApiError>)> {
    let admin = find_admin_by_pubkey(&state, &req.admin_pubkey).await?;
    if admin.status != "ACTIVE" {
        return Err(err(StatusCode::UNAUTHORIZED, 2002, "admin is not active"));
    }

    let now_ts = Utc::now().timestamp();
    let challenge_payload = {
        let mut challenges = state.login_challenges.write().await;
        let challenge = challenges
            .get_mut(&req.challenge_id)
            .ok_or_else(|| err(StatusCode::BAD_REQUEST, 2003, "challenge not found"))?;

        if challenge.admin_pubkey != req.admin_pubkey {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2004,
                "challenge pubkey mismatch",
            ));
        }
        if challenge.consumed {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2005,
                "challenge already consumed",
            ));
        }
        if challenge.expire_at < now_ts {
            return Err(err(StatusCode::BAD_REQUEST, 2006, "challenge expired"));
        }

        challenge.consumed = true;
        challenge.challenge_payload.clone()
    };

    if let Err(reason) =
        verify_challenge_signature(&req.admin_pubkey, &challenge_payload, &req.signature)
    {
        write_audit(
            &state,
            Some(admin.user_id.clone()),
            "AUTH_VERIFY",
            "LOGIN_CHALLENGE",
            Some(req.challenge_id.clone()),
            "FAILED",
            serde_json::json!({"reason": reason}),
        )
        .await?;
        return Err(err(
            StatusCode::UNAUTHORIZED,
            2007,
            "signature verify failed",
        ));
    }

    let access_token = format!("atk_{}", Uuid::new_v4().simple());
    let expires_at = (Utc::now() + Duration::seconds(TOKEN_EXPIRES_SECONDS)).timestamp();
    let session = Session {
        user_id: admin.user_id.clone(),
        role: admin.role.clone(),
        expires_at,
    };
    state
        .sessions
        .write()
        .await
        .insert(access_token.clone(), session);

    write_audit(
        &state,
        Some(admin.user_id.clone()),
        "AUTH_VERIFY",
        "SESSION",
        Some(access_token.clone()),
        "SUCCESS",
        serde_json::json!({"challenge_id": req.challenge_id}),
    )
    .await?;

    Ok(Json(ok(VerifyData {
        access_token,
        expires_in: TOKEN_EXPIRES_SECONDS,
        user: SessionUser {
            user_id: admin.user_id,
            role: admin.role,
        },
    })))
}

async fn auth_qr_challenge(
    State(state): State<AppState>,
    Json(req): Json<QrChallengeRequest>,
) -> Result<Json<ApiResponse<QrChallengeData>>, (StatusCode, Json<ApiError>)> {
    let origin = req.origin.unwrap_or_default().trim().to_string();
    if origin.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "origin is required"));
    }
    let session_id = req.session_id.unwrap_or_default().trim().to_string();
    if session_id.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "session_id is required"));
    }
    let domain = extract_domain_from_origin(&origin)
        .or(req.domain)
        .unwrap_or_default();
    if domain.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "domain is required"));
    }

    let challenge_id = format!("chl_{}", Uuid::new_v4().simple());
    let nonce = Uuid::new_v4().simple().to_string();
    let challenge_token = Uuid::new_v4().simple().to_string();
    let issued_at = Utc::now().timestamp();
    let expire_at = (Utc::now() + Duration::seconds(CHALLENGE_EXPIRES_SECONDS)).timestamp();
    let qr_aud =
        std::env::var("CPMS_LOGIN_QR_AUD").unwrap_or_else(|_| "cpms-local-app".to_string());
    let challenge_payload = format!(
        "WUMINAPP_LOGIN_V1|{}|{}|{}|{}|{}|{}",
        "cpms", qr_aud, challenge_id, challenge_token, nonce, expire_at
    );

    let challenge = LoginChallenge {
        admin_pubkey: String::new(),
        challenge_payload: challenge_payload.clone(),
        session_id: session_id.clone(),
        expire_at,
        consumed: false,
    };
    state
        .login_challenges
        .write()
        .await
        .insert(challenge_id.clone(), challenge);
    persist_runtime_store(&state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;

    let login_qr_payload = serde_json::json!({
        "proto": "WUMINAPP_LOGIN_V1",
        "system": "cpms",
        "request_id": challenge_id,
        "challenge": challenge_token,
        "issued_at": issued_at,
        "expires_at": expire_at,
        "aud": qr_aud,
        "challenge_id": challenge_id,
        "challenge_payload": challenge_payload,
        "session_id": session_id,
        "nonce": nonce,
        "expire_at": expire_at
    })
    .to_string();

    Ok(Json(ok(QrChallengeData {
        challenge_id,
        challenge_payload,
        login_qr_payload,
        origin,
        domain,
        session_id,
        nonce,
        expire_at,
    })))
}

async fn auth_qr_complete(
    State(state): State<AppState>,
    Json(req): Json<QrCompleteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    if req.challenge_id.trim().is_empty()
        || req.session_id.trim().is_empty()
        || req.admin_pubkey.trim().is_empty()
        || req.signature.trim().is_empty()
    {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, session_id, admin_pubkey, signature are required",
        ));
    }
    let now_ts = Utc::now().timestamp();
    let challenge_payload = {
        let mut challenges = state.login_challenges.write().await;
        let challenge = challenges
            .get_mut(req.challenge_id.trim())
            .ok_or_else(|| err(StatusCode::BAD_REQUEST, 2003, "challenge not found"))?;
        if challenge.consumed {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2005,
                "challenge already consumed",
            ));
        }
        if challenge.expire_at < now_ts {
            return Err(err(StatusCode::BAD_REQUEST, 2006, "challenge expired"));
        }
        if challenge.session_id != req.session_id.trim() {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2004,
                "challenge session mismatch",
            ));
        }
        challenge.consumed = true;
        challenge.admin_pubkey = req.admin_pubkey.trim().to_string();
        challenge.challenge_payload.clone()
    };

    let admin = find_admin_by_pubkey(&state, req.admin_pubkey.trim()).await?;
    if admin.status != "ACTIVE" {
        return Err(err(StatusCode::UNAUTHORIZED, 2002, "admin is not active"));
    }
    if verify_wumin_login_signature(
        req.admin_pubkey.trim(),
        &challenge_payload,
        req.signature.trim(),
    )
    .is_err()
    {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            2007,
            "signature verify failed",
        ));
    }

    let access_token = format!("atk_{}", Uuid::new_v4().simple());
    let expires_at = (Utc::now() + Duration::seconds(TOKEN_EXPIRES_SECONDS)).timestamp();
    let session = Session {
        user_id: admin.user_id.clone(),
        role: admin.role.clone(),
        expires_at,
    };
    state
        .sessions
        .write()
        .await
        .insert(access_token.clone(), session);
    state.qr_login_results.write().await.insert(
        req.challenge_id.clone(),
        QrLoginResult {
            session_id: req.session_id.trim().to_string(),
            access_token,
            expires_in: TOKEN_EXPIRES_SECONDS,
            user: SessionUser {
                user_id: admin.user_id,
                role: admin.role,
            },
            created_at: now_ts,
        },
    );
    persist_runtime_store(&state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;

    Ok(Json(ok(serde_json::json!({"status": "SUCCESS"}))))
}

async fn auth_qr_result(
    State(state): State<AppState>,
    Query(query): Query<QrResultQuery>,
) -> Result<Json<ApiResponse<QrResultData>>, (StatusCode, Json<ApiError>)> {
    if query.challenge_id.trim().is_empty() || query.session_id.trim().is_empty() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id and session_id are required",
        ));
    }
    let now_ts = Utc::now().timestamp();
    let changed = {
        let mut results = state.qr_login_results.write().await;
        let before = results.len();
        results.retain(|_, v| v.created_at + 3600 > now_ts);
        before != results.len()
    };
    if changed {
        persist_runtime_store(&state)
            .await
            .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;
    }

    if let Some(result) = state
        .qr_login_results
        .read()
        .await
        .get(query.challenge_id.trim())
        .cloned()
    {
        if result.session_id != query.session_id.trim() {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2004,
                "challenge session mismatch",
            ));
        }
        return Ok(Json(ok(QrResultData {
            status: "SUCCESS".to_string(),
            message: "login success".to_string(),
            access_token: Some(result.access_token),
            expires_in: Some(result.expires_in),
            user: Some(result.user),
        })));
    }

    let challenge_map = state.login_challenges.read().await;
    let Some(challenge) = challenge_map.get(query.challenge_id.trim()) else {
        return Err(err(StatusCode::BAD_REQUEST, 2003, "challenge not found"));
    };
    if challenge.session_id != query.session_id.trim() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            2004,
            "challenge session mismatch",
        ));
    }
    if challenge.expire_at < now_ts {
        return Ok(Json(ok(QrResultData {
            status: "EXPIRED".to_string(),
            message: "challenge expired".to_string(),
            access_token: None,
            expires_in: None,
            user: None,
        })));
    }

    Ok(Json(ok(QrResultData {
        status: "PENDING".to_string(),
        message: "waiting mobile scan".to_string(),
        access_token: None,
        expires_in: None,
        user: None,
    })))
}

async fn auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let token = authz::bearer_token(&headers)?;
    let removed = state.sessions.write().await.remove(&token);
    if removed.is_none() {
        return Err(err(StatusCode::UNAUTHORIZED, 2001, "invalid token"));
    }
    persist_runtime_store(&state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;

    Ok(Json(ok(serde_json::json!({"status": "SIGNED_OUT"}))))
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

pub(crate) fn verify_challenge_signature(
    admin_pubkey: &str,
    challenge_payload: &str,
    signature: &str,
) -> Result<(), &'static str> {
    crate::verify_signature_with_context(
        admin_pubkey,
        challenge_payload,
        signature,
        b"CPMS-ADMIN-AUTH-V1",
    )
}

fn verify_wumin_login_signature(
    admin_pubkey: &str,
    challenge_payload: &str,
    signature: &str,
) -> Result<(), &'static str> {
    let pubkey_bytes = crate::decode_bytes(admin_pubkey).ok_or("invalid admin_pubkey encoding")?;
    if pubkey_bytes.len() != 32 {
        return Err("invalid admin_pubkey length");
    }
    let sig_bytes = crate::decode_bytes(signature).ok_or("invalid signature encoding")?;
    if sig_bytes.len() != 64 {
        return Err("invalid signature length");
    }

    let pk = PublicKey::from_bytes(&pubkey_bytes).map_err(|_| "invalid sr25519 public key")?;
    let sig = Signature::from_bytes(&sig_bytes).map_err(|_| "invalid sr25519 signature")?;
    let ctx = signing_context(b"substrate");
    if pk
        .verify(ctx.bytes(challenge_payload.as_bytes()), &sig)
        .is_ok()
    {
        return Ok(());
    }
    let wrapped = format!("<Bytes>{}</Bytes>", challenge_payload);
    pk.verify(ctx.bytes(wrapped.as_bytes()), &sig)
        .map_err(|_| "sr25519 verify failed")
}
