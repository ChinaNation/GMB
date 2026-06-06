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
use uuid::Uuid;

use crate::admins::repo;
use crate::*;

use super::guards::{admin_auth, bearer_token};
use super::model::*;
use super::signature::{
    build_admin_display_name_from_user, extract_domain_from_origin, parse_admin_identity_qr,
    resolve_admin_city, verify_admin_signature,
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
            passkey_bound: ctx.passkey_bound,
        },
    })
    .into_response()
}

/// 主动登出:从结构化会话表删除当前 session。
pub(crate) async fn admin_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = match bearer_token(&headers) {
        Some(t) => t,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "missing token"),
    };
    if let Err(err) = repo::delete_admin_session(&state.db, token.as_str()) {
        let message = format!("delete session failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    }
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

    let admin = match repo::get_admin_by_pubkey(&state.db, admin_pubkey.as_str()) {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::FORBIDDEN, 2002, "admin not found"),
        Err(err) => {
            let message = format!("query admin failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    let province = match repo::province_scope_for_role(&state.db, &admin.admin_pubkey, &admin.role)
    {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query admin scope failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    let passkey_bound = match repo::admin_has_active_passkey(&state.db, &admin.admin_pubkey) {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query passkey failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminIdentifyOutput {
            admin_pubkey: admin.admin_pubkey.clone(),
            role: admin.role.clone(),
            admin_name: build_admin_display_name_from_user(&admin, province.as_deref()),
            admin_province: province,
            admin_city: resolve_admin_city(&admin),
            passkey_bound,
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

    match repo::get_admin_by_pubkey(&state.db, input.admin_pubkey.as_str()) {
        Ok(Some(_)) => {}
        Ok(None) => return api_error(StatusCode::FORBIDDEN, 2002, "admin not found"),
        Err(err) => {
            let message = format!("query admin failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    }
    if let Err(err) = repo::insert_login_challenge(
        &state.db,
        &LoginChallenge {
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
    ) {
        let message = format!("insert challenge failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    }

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
    let challenge_id = input.challenge_id.trim().to_string();
    let signature = input.signature.trim().to_string();
    let origin = input.origin.clone();
    let session_id = input.session_id.clone();
    let nonce = input.nonce.clone();
    let verify_domain_for_db = verify_domain.clone();
    let result = state.db.with_client(move |conn| {
        repo::cleanup_login_state_conn(conn, now)?;
        let Some(mut challenge) = repo::get_login_challenge_conn(conn, challenge_id.as_str())?
        else {
            return Err("http:not_found:challenge not found".to_string());
        };
        if challenge.consumed {
            return Err("http:conflict:challenge already consumed".to_string());
        }
        if now > challenge.expire_at {
            return Err("http:gone:challenge expired".to_string());
        }
        if challenge.origin != origin
            || challenge.domain != verify_domain_for_db
            || challenge.session_id != session_id
            || challenge.nonce != nonce
        {
            return Err("http:unprocessable:challenge context mismatch".to_string());
        }

        // 中文注释:乐观消费——先标记 consumed 再验签,防止并发请求同时通过 consumed 检查。
        // 验签失败时回退 consumed = false。
        challenge.consumed = true;
        let admin_pubkey = challenge.admin_pubkey.clone();
        let challenge_text = challenge.challenge_text.clone();
        repo::update_login_challenge_conn(conn, &challenge)?;

        if !verify_admin_signature(&admin_pubkey, &challenge_text, signature.as_str()) {
            challenge.consumed = false;
            repo::update_login_challenge_conn(conn, &challenge)?;
            return Err("http:unprocessable:signature verify failed".to_string());
        }

        let admin = repo::get_admin_by_pubkey_conn(conn, admin_pubkey.as_str())?
            .ok_or_else(|| "http:forbidden:admin not found".to_string())?;
        let admin_role = admin.role.clone();
        let admin_province =
            repo::province_scope_for_role_conn(conn, &admin.admin_pubkey, &admin.role)?;
        let admin_name = build_admin_display_name_from_user(&admin, admin_province.as_deref());
        let admin_city = resolve_admin_city(&admin);
        let passkey_bound = repo::admin_has_active_passkey_conn(conn, &admin.admin_pubkey)?;
        let access_token = Uuid::new_v4().to_string();
        let expire_at = now + Duration::hours(8);
        let new_session = AdminSession {
            token: access_token.clone(),
            admin_pubkey: admin.admin_pubkey.clone(),
            role: admin_role.clone(),
            expire_at,
            last_active_at: now,
        };
        repo::insert_admin_session_conn(conn, &new_session)?;
        Ok((
            access_token,
            expire_at,
            AdminIdentifyOutput {
                admin_pubkey: admin.admin_pubkey,
                role: admin_role,
                admin_name,
                admin_province,
                admin_city,
                passkey_bound,
            },
        ))
    });

    let (access_token, expire_at, admin) = match result {
        Ok(v) => v,
        Err(err) if err == "http:not_found:challenge not found" => {
            return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found")
        }
        Err(err) if err == "http:conflict:challenge already consumed" => {
            return api_error(StatusCode::CONFLICT, 1007, "challenge already consumed")
        }
        Err(err) if err == "http:gone:challenge expired" => {
            return api_error(StatusCode::GONE, 1007, "challenge expired")
        }
        Err(err) if err == "http:unprocessable:challenge context mismatch" => {
            return api_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                2004,
                "challenge context mismatch",
            )
        }
        Err(err) if err == "http:unprocessable:signature verify failed" => {
            return api_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                2004,
                "signature verify failed",
            )
        }
        Err(err) if err == "http:forbidden:admin not found" => {
            return api_error(StatusCode::FORBIDDEN, 2002, "admin not found")
        }
        Err(err) => {
            let message = format!("verify login failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminVerifyOutput {
            access_token,
            expire_at: expire_at.timestamp(),
            admin,
        },
    })
    .into_response()
}
