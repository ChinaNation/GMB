use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use schnorrkel::{signing_context, MiniSecretKey, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{authz, err, find_admin_by_pubkey, ok, write_audit, ApiError, ApiResponse, AppState};
use rand::Rng;

/// 中文注释：用 CSPRNG 生成 32 字节随机 token，比 UUID 更安全。
fn generate_secure_token(prefix: &str) -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill(&mut buf);
    format!("{}_{}", prefix, hex::encode(buf))
}

pub(crate) const TOKEN_EXPIRES_SECONDS: i64 = 30 * 60;
const CHALLENGE_EXPIRES_SECONDS: i64 = 90;

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
    expire_at: i64,
}

#[derive(Deserialize)]
struct QrCompleteRequest {
    #[serde(alias = "challenge")]
    challenge_id: String,
    session_id: String,
    admin_pubkey: String,
    signature: String,
}

#[derive(Deserialize)]
struct QrResultQuery {
    #[serde(alias = "challenge")]
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

    sqlx::query(
        "INSERT INTO login_challenges (challenge_id, admin_pubkey, challenge_payload, session_id, expire_at, consumed, created_at)
         VALUES ($1, $2, $3, $4, $5, FALSE, $6)",
    )
    .bind(&challenge_id)
    .bind(req.admin_pubkey.trim())
    .bind(&challenge_payload)
    .bind(&challenge_id)
    .bind(expire_at)
    .bind(Utc::now().timestamp())
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "save challenge failed"))?;

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
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    let row = sqlx::query(
        "SELECT admin_pubkey, challenge_payload, expire_at, consumed
         FROM login_challenges
         WHERE challenge_id = $1
         FOR UPDATE",
    )
    .bind(req.challenge_id.trim())
    .fetch_optional(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query challenge failed",
        )
    })?
    .ok_or_else(|| err(StatusCode::BAD_REQUEST, 2003, "challenge not found"))?;

    let challenge_pubkey: String = row.get("admin_pubkey");
    if challenge_pubkey != req.admin_pubkey {
        return Err(err(
            StatusCode::BAD_REQUEST,
            2004,
            "challenge pubkey mismatch",
        ));
    }
    let consumed: bool = row.get("consumed");
    if consumed {
        return Err(err(
            StatusCode::BAD_REQUEST,
            2005,
            "challenge already consumed",
        ));
    }
    let expire_at: i64 = row.get("expire_at");
    if expire_at < now_ts {
        return Err(err(StatusCode::BAD_REQUEST, 2006, "challenge expired"));
    }
    let challenge_payload: String = row.get("challenge_payload");

    sqlx::query("UPDATE login_challenges SET consumed = TRUE WHERE challenge_id = $1")
        .bind(req.challenge_id.trim())
        .execute(tx.as_mut())
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "consume challenge failed",
            )
        })?;

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

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

    let access_token = generate_secure_token("atk");
    let expires_at = (Utc::now() + Duration::seconds(TOKEN_EXPIRES_SECONDS)).timestamp();

    sqlx::query(
        "INSERT INTO sessions (access_token, user_id, role, expires_at, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&access_token)
    .bind(&admin.user_id)
    .bind(&admin.role)
    .bind(expires_at)
    .bind(Utc::now().timestamp())
    .execute(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "create session failed",
        )
    })?;

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
    let issued_at = Utc::now().timestamp();
    let expire_at = (Utc::now() + Duration::seconds(CHALLENGE_EXPIRES_SECONDS)).timestamp();
    let challenge_payload = format!(
        "WUMIN_LOGIN_V1.0.0|{}|{}|{}",
        "cpms", challenge_id, expire_at
    );
    let (sys_pubkey, sys_sig) = build_login_qr_system_signature(
        &state,
        "cpms",
        challenge_id.as_str(),
        issued_at,
        expire_at,
    )
    .await?;

    sqlx::query(
        "INSERT INTO login_challenges (challenge_id, admin_pubkey, challenge_payload, session_id, expire_at, consumed, created_at)
         VALUES ($1, '', $2, $3, $4, FALSE, $5)",
    )
    .bind(&challenge_id)
    .bind(&challenge_payload)
    .bind(&session_id)
    .bind(expire_at)
    .bind(issued_at)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "save challenge failed"))?;

    let login_qr_payload = serde_json::json!({
        "proto": "WUMIN_LOGIN_V1.0.0",
        "type": "challenge",
        "system": "cpms",
        "challenge": challenge_id,
        "issued_at": issued_at,
        "expires_at": expire_at,
        "sys_pubkey": sys_pubkey,
        "sys_sig": sys_sig
    })
    .to_string();

    Ok(Json(ok(QrChallengeData {
        challenge_id,
        challenge_payload,
        login_qr_payload,
        origin,
        domain,
        session_id,
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

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    let row = sqlx::query(
        "SELECT challenge_payload, session_id, expire_at, consumed
         FROM login_challenges
         WHERE challenge_id = $1
         FOR UPDATE",
    )
    .bind(req.challenge_id.trim())
    .fetch_optional(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query challenge failed",
        )
    })?
    .ok_or_else(|| err(StatusCode::BAD_REQUEST, 2003, "challenge not found"))?;

    let consumed: bool = row.get("consumed");
    if consumed {
        return Err(err(
            StatusCode::BAD_REQUEST,
            2005,
            "challenge already consumed",
        ));
    }

    let expire_at: i64 = row.get("expire_at");
    if expire_at < now_ts {
        return Err(err(StatusCode::BAD_REQUEST, 2006, "challenge expired"));
    }

    let challenge_session_id: String = row.get("session_id");
    if challenge_session_id != req.session_id.trim() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            2004,
            "challenge session mismatch",
        ));
    }

    let challenge_payload: String = row.get("challenge_payload");

    // 先验签和查管理员，全部通过后才消费 challenge（失败时 tx 自动回滚）
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

    // 验签通过，消费 challenge
    sqlx::query(
        "UPDATE login_challenges
         SET consumed = TRUE, admin_pubkey = $1
         WHERE challenge_id = $2",
    )
    .bind(req.admin_pubkey.trim())
    .bind(req.challenge_id.trim())
    .execute(tx.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "consume challenge failed",
        )
    })?;

    tx.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

    let access_token = generate_secure_token("atk");
    let expires_at = (Utc::now() + Duration::seconds(TOKEN_EXPIRES_SECONDS)).timestamp();

    let mut tx2 = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    sqlx::query(
        "INSERT INTO sessions (access_token, user_id, role, expires_at, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&access_token)
    .bind(&admin.user_id)
    .bind(&admin.role)
    .bind(expires_at)
    .bind(now_ts)
    .execute(tx2.as_mut())
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "create session failed",
        )
    })?;

    sqlx::query(
        "INSERT INTO qr_login_results (challenge_id, session_id, access_token, expires_in, user_id, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (challenge_id) DO UPDATE SET
           session_id = EXCLUDED.session_id,
           access_token = EXCLUDED.access_token,
           expires_in = EXCLUDED.expires_in,
           user_id = EXCLUDED.user_id,
           role = EXCLUDED.role,
           created_at = EXCLUDED.created_at",
    )
    .bind(req.challenge_id.trim())
    .bind(req.session_id.trim())
    .bind(&access_token)
    .bind(TOKEN_EXPIRES_SECONDS)
    .bind(&admin.user_id)
    .bind(&admin.role)
    .bind(now_ts)
    .execute(tx2.as_mut())
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "save qr result failed"))?;

    tx2.commit()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "commit tx failed"))?;

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

    // 清理结果需要幂等即可，使用轻量锁避免高并发下重复清理抖动。
    let _guard = state.qr_result_gc_lock.write().await;
    sqlx::query("DELETE FROM qr_login_results WHERE created_at + 3600 <= $1")
        .bind(now_ts)
        .execute(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "gc qr result failed",
            )
        })?;

    if let Some(row) = sqlx::query(
        "SELECT session_id, access_token, expires_in, user_id, role
         FROM qr_login_results
         WHERE challenge_id = $1",
    )
    .bind(query.challenge_id.trim())
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query qr result failed",
        )
    })? {
        let session_id: String = row.get("session_id");
        if session_id != query.session_id.trim() {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2004,
                "challenge session mismatch",
            ));
        }
        return Ok(Json(ok(QrResultData {
            status: "SUCCESS".to_string(),
            message: "login success".to_string(),
            access_token: Some(row.get("access_token")),
            expires_in: Some(row.get("expires_in")),
            user: Some(SessionUser {
                user_id: row.get("user_id"),
                role: row.get("role"),
            }),
        })));
    }

    let challenge_row =
        sqlx::query("SELECT session_id, expire_at FROM login_challenges WHERE challenge_id = $1")
            .bind(query.challenge_id.trim())
            .fetch_optional(&state.db)
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "query challenge failed",
                )
            })?
            .ok_or_else(|| err(StatusCode::BAD_REQUEST, 2003, "challenge not found"))?;

    let challenge_session_id: String = challenge_row.get("session_id");
    if challenge_session_id != query.session_id.trim() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            2004,
            "challenge session mismatch",
        ));
    }
    let expire_at: i64 = challenge_row.get("expire_at");
    if expire_at < now_ts {
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
    let result = sqlx::query("DELETE FROM sessions WHERE access_token = $1")
        .bind(token)
        .execute(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "delete session failed",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err(err(StatusCode::UNAUTHORIZED, 2001, "invalid token"));
    }

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

pub(crate) fn verify_wumin_login_signature(
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
    pk.verify(signing_context(b"substrate").bytes(challenge_payload.as_bytes()), &sig)
        .map_err(|_| "sr25519 verify failed")
}

pub(crate) async fn build_login_qr_system_signature(
    state: &AppState,
    system: &str,
    challenge: &str,
    issued_at: i64,
    expires_at: i64,
) -> Result<(String, String), (StatusCode, Json<ApiError>)> {
    let keys = crate::initialize::load_qr_sign_keys(state).await?;
    let active = keys.iter().find(|k| k.status == "ACTIVE").ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5002,
            "missing active qr sign key",
        )
    })?;
    if active.secret_bytes.len() != 32 {
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid qr sign secret length",
        ));
    }
    let mini = MiniSecretKey::from_bytes(active.secret_bytes.as_slice()).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid qr sign secret key",
        )
    })?;
    let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);
    let message = format!(
        "WUMIN_LOGIN_V1.0.0|{}|{}|{}|{}|{}",
        system, challenge, issued_at, expires_at, active.pubkey
    );
    let signature = keypair.sign(signing_context(b"substrate").bytes(message.as_bytes()));
    Ok((
        active.pubkey.clone(),
        format!("0x{}", hex::encode(signature.to_bytes())),
    ))
}
