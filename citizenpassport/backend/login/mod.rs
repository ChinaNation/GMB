use std::{env, net::SocketAddr};

use axum::{
    extract::{ConnectInfo, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use rand::{rngs::OsRng, RngCore};
use schnorrkel::{signing_context, MiniSecretKey, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    authz,
    common::{err, find_admin_by_pubkey, ok, rate_limit, ApiError, ApiResponse},
    AppState,
};

/// 中文注释：用 CSPRNG 生成 32 字节随机 token，比 UUID 更安全。
fn generate_secure_token(prefix: &str) -> String {
    let mut buf = [0u8; 32];
    OsRng.fill_bytes(&mut buf);
    format!("{}_{}", prefix, hex::encode(buf))
}

/// 管理员 session 空闲过期时间（15 分钟无操作后过期）。
pub(crate) const ADMINS_IDLE_EXPIRES_SECONDS: i64 = 15 * 60;
/// 操作员 session 空闲过期时间（30 分钟无操作后过期）。
pub(crate) const OPERATORS_IDLE_EXPIRES_SECONDS: i64 = 30 * 60;
const CHALLENGE_EXPIRES_SECONDS: i64 = 90;

pub(crate) fn session_ttl_seconds(user_group: &str) -> i64 {
    if user_group == "admins" {
        ADMINS_IDLE_EXPIRES_SECONDS
    } else {
        OPERATORS_IDLE_EXPIRES_SECONDS
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SessionUser {
    user_id: String,
    user_group: String,
    admin_display_name: String,
}

#[derive(Deserialize)]
struct QrChallengeRequest {
    origin: Option<String>,
    domain: Option<String>,
}

#[derive(Serialize)]
struct QrChallengeData {
    challenge_id: String,
    login_qr_payload: String,
    origin: String,
    domain: String,
    session_id: String,
    expire_at: i64,
}

#[derive(Deserialize)]
struct QrCompleteRequest {
    challenge_id: String,
    session_id: String,
    admin_account: String,
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
    expires_in: Option<i64>,
    user: Option<SessionUser>,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/auth/qr/challenge", post(auth_qr_challenge))
        .route("/api/v1/admin/auth/qr/complete", post(auth_qr_complete))
        .route("/api/v1/admin/auth/qr/result", get(auth_qr_result))
        .route("/api/v1/admin/auth/me", get(auth_me))
        .route("/api/v1/admin/auth/logout", post(auth_logout))
}

async fn auth_qr_challenge(
    State(state): State<AppState>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<QrChallengeRequest>,
) -> Result<Json<ApiResponse<QrChallengeData>>, (StatusCode, Json<ApiError>)> {
    rate_limit::check(&state, client_addr, &headers, "auth_qr_challenge", 20, 60).await?;

    let origin = req.origin.unwrap_or_default().trim().to_string();
    if origin.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "origin is required"));
    }
    let session_id = generate_secure_token("sid");
    let domain = extract_domain_from_origin(&origin)
        .or(req.domain)
        .unwrap_or_default();
    if domain.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "domain is required"));
    }

    let challenge_id = format!("chl_{}", Uuid::new_v4().simple());
    let issued_at = Utc::now().timestamp();
    let expire_at = (Utc::now() + Duration::seconds(CHALLENGE_EXPIRES_SECONDS)).timestamp();
    let (sys_pubkey, sys_sig) = build_login_qr_system_signature(
        &state,
        "cpms",
        challenge_id.as_str(),
        issued_at,
        expire_at,
    )
    .await?;

    sqlx::query(
        "INSERT INTO login_challenges (challenge_id, admin_account, session_id, expire_at, consumed, created_at)
         VALUES ($1, '', $2, $3, FALSE, $4)",
    )
    .bind(&challenge_id)
    .bind(&session_id)
    .bind(expire_at)
    .bind(issued_at)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "save challenge failed"))?;

    let login_qr_payload = serde_json::to_string(&crate::qr::LoginChallengeEnvelope::new(
        challenge_id.clone(),
        issued_at,
        expire_at,
        crate::qr::LoginChallengeBody {
            system: "cpms".to_string(),
            sys_pubkey: sys_pubkey.clone(),
            sys_sig: sys_sig.clone(),
        },
    ))
    .unwrap_or_default();

    Ok(Json(ok(QrChallengeData {
        challenge_id,
        login_qr_payload,
        origin,
        domain,
        session_id,
        expire_at,
    })))
}

async fn auth_qr_complete(
    State(state): State<AppState>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<QrCompleteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    rate_limit::check(&state, client_addr, &headers, "auth_qr_complete", 30, 60).await?;

    if req.challenge_id.trim().is_empty()
        || req.session_id.trim().is_empty()
        || req.admin_account.trim().is_empty()
        || req.signature.trim().is_empty()
    {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, session_id, admin_account, signature are required",
        ));
    }
    let now_ts = Utc::now().timestamp();

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    let row = sqlx::query(
        "SELECT session_id, expire_at, consumed
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
        return Err(err(StatusCode::GONE, 2006, "challenge expired"));
    }

    let challenge_session_id: String = row.get("session_id");
    if challenge_session_id != req.session_id.trim() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            2004,
            "challenge session mismatch",
        ));
    }

    // 先验签和查管理员，全部通过后才消费 challenge（失败时 tx 自动回滚）
    let admin = find_admin_by_pubkey(&state, req.admin_account.trim()).await?;
    if admin.user_group == "operators" {
        crate::archive::ensure_operator_annual_export_unlocked(&state).await?;
    }
    // 重建完整签名原文(包含签名者公钥),与 CitizenWallet 端
    // buildSignatureMessage(kind=login_receipt, principal=pubkey) 一致。
    let verify_message = crate::qr::build_signature_message(
        crate::qr::QrKind::LoginReceipt,
        req.challenge_id.trim(),
        Some("cpms"),
        Some(expire_at),
        req.admin_account.trim(),
    );
    if verify_citizenwallet_login_signature(
        req.admin_account.trim(),
        &verify_message,
        req.signature.trim(),
    )
    .is_err()
    {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            2007,
            "signature verify failed",
        ));
    }

    // 验签通过，消费 challenge
    sqlx::query(
        "UPDATE login_challenges
         SET consumed = TRUE, admin_account = $1
         WHERE challenge_id = $2",
    )
    .bind(req.admin_account.trim())
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
    let ttl = session_ttl_seconds(&admin.user_group);
    let expires_at = (Utc::now() + Duration::seconds(ttl)).timestamp();

    let mut tx2 = state
        .db
        .begin()
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "begin tx failed"))?;

    sqlx::query(
        "INSERT INTO sessions (access_token, user_id, user_group, expires_at, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&access_token)
    .bind(&admin.user_id)
    .bind(&admin.user_group)
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
        "INSERT INTO qr_login_results (challenge_id, session_id, access_token, expires_in, user_id, user_group, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (challenge_id) DO UPDATE SET
           session_id = EXCLUDED.session_id,
           access_token = EXCLUDED.access_token,
           expires_in = EXCLUDED.expires_in,
           user_id = EXCLUDED.user_id,
           user_group = EXCLUDED.user_group,
           created_at = EXCLUDED.created_at",
    )
    .bind(req.challenge_id.trim())
    .bind(req.session_id.trim())
    .bind(&access_token)
    .bind(ttl)
    .bind(&admin.user_id)
    .bind(&admin.user_group)
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
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<QrResultQuery>,
) -> Result<Response, (StatusCode, Json<ApiError>)> {
    rate_limit::check(&state, client_addr, &headers, "auth_qr_result", 120, 60).await?;

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
        "SELECT r.session_id, r.access_token, r.expires_in, r.user_id, r.user_group,
                COALESCE(a.admin_display_name, '') AS admin_display_name
         FROM qr_login_results r
         JOIN admin_users a ON a.user_id = r.user_id
         WHERE r.challenge_id = $1",
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
        let access_token: String = row.get("access_token");
        let expires_in: i64 = row.get("expires_in");
        let user = SessionUser {
            user_id: row.get("user_id"),
            user_group: row.get("user_group"),
            admin_display_name: row.get("admin_display_name"),
        };
        if user.user_group == "operators" {
            crate::archive::ensure_operator_annual_export_unlocked(&state).await?;
        }
        sqlx::query("DELETE FROM qr_login_results WHERE challenge_id = $1 AND session_id = $2")
            .bind(query.challenge_id.trim())
            .bind(query.session_id.trim())
            .execute(&state.db)
            .await
            .map_err(|_| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    5001,
                    "consume qr result failed",
                )
            })?;
        let mut response = Json(ok(QrResultData {
            status: "SUCCESS".to_string(),
            message: "login success".to_string(),
            expires_in: Some(expires_in),
            user: Some(user),
        }))
        .into_response();
        response.headers_mut().insert(
            header::SET_COOKIE,
            session_cookie(&access_token, expires_in)
                .parse()
                .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "set cookie failed"))?,
        );
        return Ok(response);
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
            expires_in: None,
            user: None,
        }))
        .into_response());
    }

    Ok(Json(ok(QrResultData {
        status: "PENDING".to_string(),
        message: "waiting mobile scan".to_string(),
        expires_in: None,
        user: None,
    }))
    .into_response())
}

async fn auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<ApiError>)> {
    let token = authz::session_token(&headers)?;
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
        return Err(err(StatusCode::UNAUTHORIZED, 2001, "invalid session"));
    }

    let mut response = Json(ok(serde_json::json!({"status": "SIGNED_OUT"}))).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        clear_session_cookie().parse().map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "clear cookie failed",
            )
        })?,
    );
    Ok(response)
}

async fn auth_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<SessionUser>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_auth(&state, &headers).await?;
    let admin = crate::common::find_admin_by_user_id(&state, &ctx.user_id).await?;
    Ok(Json(ok(SessionUser {
        user_id: ctx.user_id,
        user_group: ctx.user_group,
        admin_display_name: admin.admin_display_name,
    })))
}

fn session_cookie(access_token: &str, _max_age: i64) -> String {
    let secure = if cookie_secure_enabled() {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Strict{}",
        authz::SESSION_COOKIE_NAME,
        access_token,
        secure
    )
}

fn clear_session_cookie() -> String {
    let secure = if cookie_secure_enabled() {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{}=; Path=/; HttpOnly; SameSite=Strict{}; Max-Age=0",
        authz::SESSION_COOKIE_NAME,
        secure
    )
}

fn cookie_secure_enabled() -> bool {
    env::var("CPMS_COOKIE_SECURE")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
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

pub(crate) fn verify_citizenwallet_login_signature(
    admin_account: &str,
    signed_message: &str,
    signature: &str,
) -> Result<(), &'static str> {
    let pubkey_bytes =
        crate::common::decode_bytes(admin_account).ok_or("invalid admin_account encoding")?;
    if pubkey_bytes.len() != 32 {
        return Err("invalid admin_account length");
    }
    let sig_bytes = crate::common::decode_bytes(signature).ok_or("invalid signature encoding")?;
    if sig_bytes.len() != 64 {
        return Err("invalid signature length");
    }

    let pk = PublicKey::from_bytes(&pubkey_bytes).map_err(|_| "invalid sr25519 public key")?;
    let sig = Signature::from_bytes(&sig_bytes).map_err(|_| "invalid sr25519 signature")?;
    pk.verify(
        signing_context(b"substrate").bytes(signed_message.as_bytes()),
        &sig,
    )
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
    let _ = issued_at; // 统一签名原文不再包含 issued_at
    let message = crate::qr::build_signature_message(
        crate::qr::QrKind::LoginChallenge,
        challenge,
        Some(system),
        Some(expires_at),
        &active.pubkey,
    );
    let signature = keypair.sign(signing_context(b"substrate").bytes(message.as_bytes()));
    Ok((
        active.pubkey.clone(),
        format!("0x{}", hex::encode(signature.to_bytes())),
    ))
}
