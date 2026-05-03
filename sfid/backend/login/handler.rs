//! 管理员普通登录与会话接口 handler。
//!
//! 覆盖 auth/check、logout、identity identify、challenge、verify;扫码登录单独放
//! `qr_login.rs`,避免普通登录和二维码登录继续堆在同一个文件。

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use tracing::warn;
use uuid::Uuid;

use crate::scope::admin_province::province_scope_for_role;
use crate::*;

use super::guards::{admin_auth, bearer_token, bootstrap_sheng_signing_pair};
use super::model::*;
use super::signature::{
    build_admin_display_name_from_user, cleanup_expired_challenges, extract_domain_from_origin,
    parse_admin_identity_qr, resolve_admin_city, verify_admin_signature,
};
use super::LOGIN_CHALLENGE_TTL_SECONDS;

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
    store
        .admin_sessions
        .insert(access_token.clone(), new_session.clone());
    // 中文注释:先释放写锁,再执行省管理员签名密钥本地 bootstrap,
    // 避免跨后续异步写 GlobalShard 时持有 StoreWriteGuard。
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

    // ADR-008(2026-05-01)Phase 23e:省登录管理员 3-tier 自治,首次登录时通过
    // `sheng_admins::signing_keys::ensure_signing_keypair` 把本 (province, admin_pubkey)
    // 的签名 keypair 加载到进程内 cache。本流程只处理 SFID 本地 seed,
    // 不负责省管理员链上更换。
    if admin_role == AdminRole::ShengAdmin {
        if let Some(province) = admin_province.as_deref() {
            bootstrap_sheng_signing_pair(&state, admin_pubkey.as_str(), province);
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
