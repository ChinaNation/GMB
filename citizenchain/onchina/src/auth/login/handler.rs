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

use crate::auth::repo;
use crate::*;

use super::guards::{admin_auth, bearer_token};
use super::model::*;
use super::onchain_gate;
use super::signature::{
    build_admin_name_from_user, extract_domain_from_origin, parse_admin_identity_qr,
    verify_admin_signature,
};
use super::LOGIN_SIGN_REQUEST_TTL_SECONDS;

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
    let capabilities = crate::platform::capability::capabilities_for(&ctx.institution_code);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminAuthOutput {
            ok: true,
            admin_account: ctx.admin_account,
            institution_code: ctx.institution_code,
            admin_level: ctx.admin_level,
            capabilities,
            admin_name: ctx.admin_name,
            scope_province_name: ctx.scope_province_name,
            scope_city_name: ctx.scope_city_name,
            scope_town_name: ctx.scope_town_name,
            cid_short_name: ctx.cid_short_name,
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
    let admin_account = parse_admin_identity_qr(&input.identity_qr);
    if admin_account.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "identity_qr is required");
    }

    let admin = match repo::get_admin_by_account(&state.db, admin_account.as_str()) {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::FORBIDDEN, 2002, "admin not found"),
        Err(err) => {
            let message = format!("query admin failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    let admin_account_for_scope = admin.admin_account.clone();
    let institution_code_for_scope = admin.institution_code.clone();
    let (province, scope_city_name, scope_town_name) = match state.db.with_client(move |conn| {
        repo::derive_admin_scope_conn(
            conn,
            admin_account_for_scope.as_str(),
            institution_code_for_scope.as_str(),
        )
    }) {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query admin scope failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    if province.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return api_error(StatusCode::FORBIDDEN, 2002, "admin province scope missing");
    }
    let cid_short_name = repo::resolve_home_cid_short_name(
        &state.db,
        &admin.institution_code,
        province.as_deref(),
        scope_city_name.as_deref(),
    )
    .unwrap_or(None);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminIdentifyOutput {
            admin_account: admin.admin_account.clone(),
            institution_code: admin.institution_code.clone(),
            admin_level: crate::core::chain_runtime::admin_level_label_for(&admin.institution_code),
            capabilities: crate::platform::capability::capabilities_for(&admin.institution_code),
            admin_name: build_admin_name_from_user(&admin, province.as_deref()),
            scope_province_name: province,
            scope_city_name,
            scope_town_name,
            cid_short_name,
        },
    })
    .into_response()
}

pub(crate) async fn admin_auth_challenge(
    State(state): State<AppState>,
    Json(input): Json<AdminChallengeInput>,
) -> impl IntoResponse {
    if input.admin_account.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "admin_account is required");
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
    let expire_at = now + Duration::seconds(LOGIN_SIGN_REQUEST_TTL_SECONDS);
    let challenge_id = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().to_string();
    let challenge_text = format!(
        "onchina-login|pubkey={}|origin={}|domain={}|session_id={}|nonce={}|iat={}|exp={}",
        input.admin_account,
        origin,
        derived_domain,
        session_id,
        nonce,
        now.timestamp(),
        expire_at.timestamp()
    );

    if let Err(err) = repo::insert_login_sign_request(
        &state.db,
        &LoginSignRequest {
            challenge_id: challenge_id.clone(),
            admin_account: input.admin_account,
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
        let Some(mut challenge) = repo::get_login_sign_request_conn(conn, challenge_id.as_str())?
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
        let admin_account = challenge.admin_account.clone();
        let challenge_text = challenge.challenge_text.clone();
        repo::update_login_sign_request_conn(conn, &challenge)?;

        if !verify_admin_signature(&admin_account, &challenge_text, signature.as_str()) {
            challenge.consumed = false;
            repo::update_login_sign_request_conn(conn, &challenge)?;
            return Err("http:unprocessable:login signature verify failed".to_string());
        }

        // 中文注释:membership 真源切到链上集合(见 onchain_gate),此处只回已验签 pubkey。
        Ok(admin_account)
    });

    let verified_pubkey = match result {
        Ok(v) => v,
        Err(err) if err == "http:not_found:challenge not found" => {
            return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found");
        }
        Err(err) if err == "http:conflict:challenge already consumed" => {
            return api_error(StatusCode::CONFLICT, 1007, "challenge already consumed");
        }
        Err(err) if err == "http:gone:challenge expired" => {
            return api_error(StatusCode::GONE, 1007, "challenge expired");
        }
        Err(err) if err == "http:unprocessable:challenge context mismatch" => {
            return api_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                2004,
                "challenge context mismatch",
            );
        }
        Err(err) if err == "http:unprocessable:login signature verify failed" => {
            return api_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                2004,
                "login signature verify failed",
            );
        }
        Err(err) => {
            let message = format!("verify login failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };

    // 链上集合鉴权 + 落本地元数据 + 签发会话。
    let outcome =
        match onchain_gate::issue_session_after_onchain_gate(&state, &verified_pubkey, now).await {
            Ok(v) => v,
            Err(err) => return onchain_gate::gate_error_response(err),
        };
    let onchain_gate::GateOutcome::Session {
        access_token,
        expire_at,
        admin,
    } = outcome
    else {
        return api_error(StatusCode::CONFLICT, 1007, "node binding required");
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

pub(crate) async fn admin_auth_confirm_node_binding(
    State(state): State<AppState>,
    Json(input): Json<NodeBindingConfirmInput>,
) -> impl IntoResponse {
    let now = Utc::now();
    let (access_token, expire_at, admin) =
        match onchain_gate::confirm_node_binding_after_onchain_gate(
            &state,
            input.binding_challenge_id.as_str(),
            input.candidate_id.as_str(),
            now,
        )
        .await
        {
            Ok(v) => v,
            Err(err) => return onchain_gate::gate_error_response(err),
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
