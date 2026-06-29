//! 管理员二维码登录 handler。
//!
//! 只承接 QR_V1 登录签名请求生成、手机扫码签名、网页轮询结果;普通登录仍在 `handler.rs`。

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use tracing::warn;
use uuid::Uuid;

use crate::auth::repo;
use crate::crypto::pubkey::same_admin_account;
use crate::*;

use super::model::*;
use super::onchain_gate;
use super::signature::{
    build_admin_name, build_admin_name_from_user, build_login_qr_system_signature,
    extract_domain_from_origin, resolve_scope_city_name, verify_admin_signature,
};
use super::LOGIN_SIGN_REQUEST_TTL_SECONDS;

pub(crate) async fn admin_auth_qr_sign_request(
    State(state): State<AppState>,
    Json(input): Json<AdminQrSignRequestInput>,
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
    let expire_at = now + Duration::seconds(LOGIN_SIGN_REQUEST_TTL_SECONDS);
    let challenge_id = Uuid::new_v4().to_string();
    // challenge_text:客户端生成 k=2 登录签名响应时的原文(与 CitizenWallet 端的
    // buildSignatureMessage(kind=signResponse, ...) 拼接规则保持一致)。
    // 注意 <principal> 位置由客户端签名时填入自己的 pubkey,后端验证时同样
    // 以客户端 pubkey 为 principal 重新拼接。这里保存的 challenge_text 仅作
    // 回放保护用的唯一 token,实际验证在 admin_auth_qr_complete 中重建。
    let challenge_text = format!(
        "{}|{}|{}|{}|{}|",
        crate::core::qr::QR_V1,
        crate::core::qr::QrKind::SignResponse.code(),
        challenge_id,
        "cid",
        expire_at.timestamp()
    );
    let (sys_pubkey, sys_sig) = match build_login_qr_system_signature(
        &state,
        "cid",
        challenge_id.as_str(),
        now.timestamp(),
        expire_at.timestamp(),
    ) {
        Ok(v) => v,
        Err(err) => {
            warn!(error = %err, "build cid login qr system signature failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "build login qr signature failed",
            );
        }
    };
    let login_qr_payload = serde_json::to_string(&crate::core::qr::SignRequestEnvelope::new(
        challenge_id.clone(),
        now.timestamp(),
        expire_at.timestamp(),
        crate::core::qr::login_request_body("cid", &sys_pubkey, &sys_sig),
    ))
    .unwrap_or_default();

    if let Err(err) = repo::insert_login_sign_request(
        &state.db,
        &LoginSignRequest {
            challenge_id: challenge_id.clone(),
            admin_account: String::new(),
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
    ) {
        let message = format!("insert qr sign request failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminQrSignRequestOutput {
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
        || input.admin_account.trim().is_empty()
        || input.signature.trim().is_empty()
    {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, admin_account, signature are required",
        );
    }

    let now = Utc::now();
    let challenge_id = input.challenge_id.trim().to_string();
    let client_session_id = input.session_id.clone();
    let login_pubkey_raw = input.admin_account.trim().to_string();
    let signer_pubkey = input
        .signer_pubkey
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let signature = input.signature.trim().to_string();
    let result = state.db.with_client(move |conn| {
        repo::cleanup_login_state_conn(conn, now)?;
        let Some(mut challenge) = repo::get_login_sign_request_conn(conn, challenge_id.as_str())?
        else {
            return Err("http:not_found:sign request not found".to_string());
        };
        if challenge.consumed {
            return Err("http:conflict:sign request already consumed".to_string());
        }
        if let Some(client_sid) = client_session_id.as_ref() {
            if challenge.session_id != client_sid.trim() {
                return Err("http:forbidden:sign request session mismatch".to_string());
            }
        }
        if now > challenge.expire_at {
            return Err("http:gone:sign request expired".to_string());
        }
        let session_id = challenge.session_id.clone();
        let challenge_expire_at = challenge.expire_at.timestamp();
        let verify_pubkey = signer_pubkey
            .clone()
            .unwrap_or_else(|| login_pubkey_raw.clone());
        let login_pubkey = repo::resolve_admin_account_key_conn(conn, login_pubkey_raw.as_str())?
            .or_else(|| {
                signer_pubkey.as_ref().and_then(|spk| {
                    repo::resolve_admin_account_key_conn(conn, spk)
                        .ok()
                        .flatten()
                })
            })
            .unwrap_or_else(|| login_pubkey_raw.clone());
        if !same_admin_account(login_pubkey.as_str(), verify_pubkey.as_str()) {
            return Err("http:forbidden:signer_pubkey must match admin_account".to_string());
        }
        // 中文注释:重建完整签名原文,与 CitizenWallet 端 k=2 登录签名响应规则一致。
        let verify_message = crate::core::qr::build_signature_message(
            crate::core::qr::QrKind::SignResponse,
            challenge_id.as_str(),
            Some("cid"),
            Some(challenge_expire_at),
            &verify_pubkey,
        );
        if !verify_admin_signature(&verify_pubkey, &verify_message, signature.as_str()) {
            warn!(
                request = %challenge_id,
                admin_account = %login_pubkey_raw,
                signer_pubkey = %verify_pubkey,
                "qr login signature verify failed"
            );
            return Err("http:unprocessable:login signature verify failed".to_string());
        }
        // 中文注释:membership 真源切到链上集合(见 onchain_gate),此处只消费挑战并回已验签 pubkey。
        challenge.consumed = true;
        challenge.admin_account = login_pubkey.clone();
        repo::update_login_sign_request_conn(conn, &challenge)?;
        Ok((session_id, login_pubkey))
    });

    let (session_id, verified_pubkey) = match result {
        Ok(v) => v,
        Err(err) if err == "http:not_found:sign request not found" => {
            return api_error(StatusCode::NOT_FOUND, 1004, "sign request not found");
        }
        Err(err) if err == "http:conflict:sign request already consumed" => {
            return api_error(StatusCode::CONFLICT, 1007, "sign request already consumed");
        }
        Err(err) if err == "http:forbidden:sign request session mismatch" => {
            return api_error(StatusCode::FORBIDDEN, 1003, "sign request session mismatch");
        }
        Err(err) if err == "http:gone:sign request expired" => {
            return api_error(StatusCode::GONE, 1007, "sign request expired");
        }
        Err(err) if err == "http:forbidden:signer_pubkey must match admin_account" => {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "signer_pubkey must match admin_account",
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
            let message = format!("complete qr login failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };

    // 链上集合鉴权 + 落本地元数据 + 签发会话。
    let (access_token, expire_at, output) =
        match onchain_gate::issue_session_after_onchain_gate(&state, &verified_pubkey, now).await {
            Ok(v) => v,
            Err(err) => return onchain_gate::gate_error_response(err),
        };

    // 写入 QR 轮询结果,供网页端取回 access_token。
    let challenge_id_for_result = input.challenge_id.trim().to_string();
    let qr_result = QrLoginResultRecord {
        session_id,
        access_token: access_token.clone(),
        expire_at,
        admin_account: output.admin_account.clone(),
        institution_code: output.institution_code.clone(),
        created_at: now,
    };
    if let Err(err) = state.db.with_client(move |conn| {
        repo::insert_qr_login_result_conn(conn, challenge_id_for_result.as_str(), &qr_result)
    }) {
        let message = format!("persist qr login result failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
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
    let challenge_id = query.challenge_id.trim().to_string();
    let session_id = query.session_id.trim().to_string();
    let result = state.db.with_client(move |conn| {
        repo::cleanup_login_state_conn(conn, now)?;
        let result = repo::get_qr_login_result_conn(conn, challenge_id.as_str())?;
        let challenge = repo::get_login_sign_request_conn(conn, challenge_id.as_str())?;
        Ok((result, challenge))
    });
    let (qr_result, challenge) = match result {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query qr login result failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };

    if let Some(result) = qr_result {
        if result.session_id != query.session_id.trim() {
            return api_error(StatusCode::FORBIDDEN, 1003, "challenge session mismatch");
        }
        let admin = match repo::get_admin_by_account(&state.db, &result.admin_account) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query admin failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        };
        let province = match repo::province_scope_for_registry_org(
            &state.db,
            &result.admin_account,
            &result.institution_code,
        ) {
            Ok(v) => v,
            Err(err) => {
                let message = format!("query admin scope failed: {err}");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
            }
        };
        if province.as_deref().map(str::trim).unwrap_or("").is_empty() {
            return api_error(StatusCode::FORBIDDEN, 2002, "admin province scope missing");
        }
        let scope_city_name = admin.as_ref().and_then(resolve_scope_city_name);
        let cid_short_name = repo::resolve_home_cid_short_name(
            &state.db,
            &result.institution_code,
            province.as_deref(),
            scope_city_name.as_deref(),
        )
        .unwrap_or(None);
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: AdminQrResultOutput {
                status: "SUCCESS".to_string(),
                message: "login success".to_string(),
                access_token: Some(result.access_token.clone()),
                expire_at: Some(result.expire_at.timestamp()),
                admin: Some(AdminIdentifyOutput {
                    admin_account: result.admin_account.clone(),
                    institution_code: result.institution_code.clone(),
                    admin_level: crate::core::chain_runtime::admin_level_label_for(
                        &result.institution_code,
                    ),
                    capabilities: crate::platform::capability::capabilities_for(
                        &result.institution_code,
                    ),
                    admin_name: admin
                        .as_ref()
                        .map(|v| build_admin_name_from_user(v, province.as_deref()))
                        .unwrap_or_else(|| {
                            build_admin_name(
                                &result.admin_account,
                                &result.institution_code,
                                province.as_deref(),
                            )
                        }),
                    scope_province_name: province,
                    scope_city_name,
                    scope_town_name: if result.institution_code == "FRG" {
                        None
                    } else {
                        crate::core::chain_runtime::node_scope_town()
                    },
                    cid_short_name,
                }),
            },
        })
        .into_response();
    }

    let Some(challenge) = challenge else {
        return api_error(StatusCode::NOT_FOUND, 1004, "challenge not found");
    };
    if challenge.session_id != session_id {
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
