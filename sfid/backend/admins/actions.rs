//! 管理员安全动作:Passkey 与 CITIZEN_QR_V1/sign_request 公民钱包签名。
//!
//! 中文注释:管理员治理动作、业务安全授权和短期挑战全部使用结构化表。
//! 本文件不保留旧内存聚合体,也不做旧格式兼容。

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use postgres::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use webauthn_rs::prelude::{PublicKeyCredential, RequestChallengeResponse};

use crate::admins::city_admins::{
    can_manage_city_admin_conn, city_admin_row_from_user_conn, count_city_admins_in_city_conn,
    ensure_city_in_creator_province_conn, find_city_admin_by_id_conn, MAX_ADMIN_NAME_CHARS,
    MAX_CITY_ADMINS_PER_CITY,
};
use crate::admins::federal_admins::federal_admin_province;
use crate::admins::login::AdminAuthContext;
use crate::admins::operation_auth::{
    ensure_action_role_allowed, parse_action_type, AdminActionType, AdminOperationAuth,
};
use crate::admins::passkeys::{
    active_passkeys, hash_json, payload_hash_for_text, signed_payload_text,
    update_passkey_usage_conn, verify_citizen_wallet_signature, webauthn, AdminSignedPayload,
    ADMIN_ACTION_TTL_SECONDS,
};
use crate::admins::repo;
use crate::admins::security_model::{AdminActionChallenge, AdminSecurityGrant};
use crate::core::qr::{build_sign_request, display_account, display_field as field};
use crate::crypto::pubkey::{normalize_admin_pubkey, same_admin_pubkey};
use crate::*;

const ADMIN_SECURITY_GRANT_TTL_SECONDS: i64 = 120;
const MAX_FEDERAL_ADMINS_PER_PROVINCE: usize = 5;
pub(crate) const ADMIN_SECURITY_GRANT_HEADER: &str = "x-sfid-security-grant";

#[derive(Debug, Deserialize)]
pub(crate) struct PrepareAdminActionInput {
    action_type: AdminActionType,
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CommitAdminActionInput {
    action_id: String,
    passkey_assertion: PublicKeyCredential,
    #[serde(default)]
    signer_pubkey: Option<String>,
    #[serde(default)]
    signature: Option<String>,
    #[serde(default)]
    payload_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminSecurityGrantOutput {
    grant_id: String,
    action_type: String,
    auth_type: AdminOperationAuth,
    target: String,
    expires_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum CommitAdminActionOutput {
    Applied(serde_json::Value),
    Grant(AdminSecurityGrantOutput),
}

#[derive(Debug, Serialize)]
pub(crate) struct PrepareAdminActionOutput {
    action_id: String,
    action_type: String,
    webauthn_options: RequestChallengeResponse,
    sign_request: Option<String>,
    payload_hash: String,
    auth_type: AdminOperationAuth,
    expires_at: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct CityAdminIdPayload {
    id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateCityAdminActionPayload {
    id: u64,
    #[serde(default)]
    admin_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CreateFederalAdminActionPayload {
    admin_pubkey: String,
    admin_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateFederalAdminActionPayload {
    id: u64,
    admin_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateAdminNameInput {
    admin_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FederalAdminIdPayload {
    id: u64,
}

struct ActionPreview {
    before_hash: String,
    after_hash: String,
    target: String,
    auth_type: AdminOperationAuth,
    display_fields: Vec<serde_json::Value>,
}

pub(crate) async fn prepare_admin_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PrepareAdminActionInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_action_role_allowed(&ctx, &input.action_type) {
        return resp;
    }
    if input.action_type.is_login_state() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "login state action cannot be prepared",
        );
    }
    let passkeys = match active_passkeys(&state.db, ctx.admin_pubkey.as_str()) {
        Ok(v) => v,
        Err(err) => {
            let message = format!("query passkeys failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    if passkeys.is_empty() {
        return api_error(StatusCode::FORBIDDEN, 1003, "passkey required");
    }
    let preview = match state.db.with_client({
        let ctx = ctx.clone();
        let action_type = input.action_type.clone();
        let payload = input.payload.clone();
        move |conn| preview_action_conn(conn, &ctx, &action_type, &payload)
    }) {
        Ok(v) => v,
        Err(err) => return admin_action_error(err),
    };
    let webauthn = match webauthn() {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let (webauthn_options, webauthn_state) =
        match webauthn.start_passkey_authentication(passkeys.as_slice()) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "start passkey auth failed");
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1503,
                    "passkey auth failed",
                );
            }
        };
    let now = Utc::now();
    let expires_at = now + Duration::seconds(ADMIN_ACTION_TTL_SECONDS);
    let action_id = format!("sfid-admin-action-{}", Uuid::new_v4());
    let province = ctx.admin_province.clone().unwrap_or_default();
    let request_hash = hash_json(&input.payload);
    let (payload_text, payload_hash, sign_request) =
        if preview.auth_type == AdminOperationAuth::PasskeyChallenge {
            let payload_text = signed_payload_text(AdminSignedPayload {
                domain: "sfid_admin_governance",
                qr_proto: crate::core::qr::CITIZEN_QR_V1,
                action_id: action_id.as_str(),
                action_type: input.action_type.as_str(),
                actor_pubkey: ctx.admin_pubkey.as_str(),
                actor_province_name: province.as_str(),
                target: preview.target.as_str(),
                request_hash: request_hash.as_str(),
                before_hash: preview.before_hash.as_str(),
                after_hash: preview.after_hash.as_str(),
                expires_at: expires_at.timestamp(),
            });
            let payload_hash = payload_hash_for_text(payload_text.as_str());
            let sign_request = match build_sign_request(
                action_id.as_str(),
                now.timestamp(),
                expires_at.timestamp(),
                ctx.admin_pubkey.as_str(),
                payload_text.as_str(),
                input.action_type.label(),
                preview.display_fields.clone(),
            ) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            (payload_text, payload_hash, Some(sign_request))
        } else {
            (String::new(), request_hash.clone(), None)
        };
    let challenge = AdminActionChallenge {
        action_id: action_id.clone(),
        action_type: input.action_type.as_str().to_string(),
        actor_pubkey: ctx.admin_pubkey.clone(),
        actor_role: ctx.role.clone(),
        actor_province_name: province,
        actor_city_name: ctx.admin_city.clone(),
        auth_type: preview.auth_type.clone(),
        target: preview.target,
        payload_text,
        payload_hash: payload_hash.clone(),
        before_hash: preview.before_hash,
        after_hash: preview.after_hash,
        request_payload: input.payload,
        webauthn_state,
        issued_at: now,
        expires_at,
        consumed: false,
    };
    if let Err(err) = repo::insert_action_challenge(&state.db, &challenge) {
        let message = format!("insert admin action failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PrepareAdminActionOutput {
            action_id,
            action_type: input.action_type.as_str().to_string(),
            webauthn_options,
            sign_request,
            payload_hash,
            auth_type: preview.auth_type,
            expires_at: expires_at.timestamp(),
        },
    })
    .into_response()
}

pub(crate) async fn commit_admin_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CommitAdminActionInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let webauthn = match webauthn() {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let now = Utc::now();
    let challenge = match state.db.with_client({
        let action_id = input.action_id.clone();
        move |conn| {
            repo::cleanup_security_state_conn(conn, now)?;
            repo::get_action_challenge_conn(conn, action_id.as_str())
        }
    }) {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "admin action not found"),
        Err(err) => {
            let message = format!("query admin action failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };
    if challenge.consumed || now > challenge.expires_at {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "admin action expired",
        );
    }
    if !same_admin_pubkey(challenge.actor_pubkey.as_str(), ctx.admin_pubkey.as_str()) {
        return api_error(StatusCode::FORBIDDEN, 1003, "admin action owner mismatch");
    }
    let action_type = match parse_action_type(challenge.action_type.as_str()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_action_role_allowed(&ctx, &action_type) {
        return resp;
    }
    if action_type.is_login_state() || challenge.auth_type == AdminOperationAuth::LoginState {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "login state action cannot be committed",
        );
    }
    if challenge.auth_type != action_type.auth_type() {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin action auth type mismatch",
        );
    }
    let auth_result = match webauthn
        .finish_passkey_authentication(&input.passkey_assertion, &challenge.webauthn_state)
    {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(error = %err, "finish passkey auth failed");
            return api_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                2004,
                "passkey auth failed",
            );
        }
    };
    if challenge.auth_type == AdminOperationAuth::PasskeyChallenge {
        let signer_pubkey = match input.signer_pubkey.as_deref() {
            Some(v) => v,
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "signer_pubkey is required"),
        };
        let signature = match input.signature.as_deref() {
            Some(v) => v,
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "signature is required"),
        };
        let payload_hash = match input.payload_hash.as_deref() {
            Some(v) => v,
            None => return api_error(StatusCode::BAD_REQUEST, 1001, "payload_hash is required"),
        };
        if let Err(resp) = verify_citizen_wallet_signature(
            ctx.admin_pubkey.as_str(),
            signer_pubkey,
            signature,
            payload_hash,
            challenge.payload_hash.as_str(),
            challenge.payload_text.as_str(),
        ) {
            return resp;
        }
    }
    let result = state.db.with_client({
        let ctx = ctx.clone();
        let challenge = challenge.clone();
        let assertion = input.passkey_assertion.clone();
        move |conn| {
            update_passkey_usage_conn(
                conn,
                ctx.admin_pubkey.as_str(),
                &assertion,
                &auth_result,
                now,
            )
            .map_err(|_| "http:forbidden:passkey owner mismatch".to_string())?;
            repo::cleanup_security_state_conn(conn, now)?;
            let mut current = repo::get_action_challenge_conn(conn, challenge.action_id.as_str())?
                .ok_or_else(|| "http:not_found:admin action not found".to_string())?;
            if current.consumed || now > current.expires_at {
                return Err("http:unprocessable:admin action expired".to_string());
            }
            if action_type.is_governance() {
                recheck_preview_conn(conn, &ctx, &current)?;
                let applied = apply_action_conn(conn, &ctx, &current)?;
                current.consumed = true;
                repo::upsert_action_challenge_conn(conn, &current)?;
                Ok(CommitAdminActionOutput::Applied(applied))
            } else {
                let grant_id = format!("sfid-admin-grant-{}", Uuid::new_v4());
                let grant_expires_at = now + Duration::seconds(ADMIN_SECURITY_GRANT_TTL_SECONDS);
                let grant = AdminSecurityGrant {
                    grant_id: grant_id.clone(),
                    action_type: action_type.as_str().to_string(),
                    actor_pubkey: ctx.admin_pubkey.clone(),
                    actor_role: ctx.role.clone(),
                    actor_province_name: ctx.admin_province.clone().unwrap_or_default(),
                    actor_city_name: ctx.admin_city.clone(),
                    auth_type: current.auth_type.clone(),
                    target: current.target.clone(),
                    payload_hash: hash_json(&current.request_payload),
                    issued_at: now,
                    expires_at: grant_expires_at,
                    consumed: false,
                };
                repo::insert_security_grant_conn(conn, &grant)?;
                current.consumed = true;
                repo::upsert_action_challenge_conn(conn, &current)?;
                Ok(CommitAdminActionOutput::Grant(AdminSecurityGrantOutput {
                    grant_id,
                    action_type: action_type.as_str().to_string(),
                    auth_type: current.auth_type,
                    target: current.target,
                    expires_at: grant_expires_at.timestamp(),
                }))
            }
        }
    });
    let output = match result {
        Ok(v) => v,
        Err(err) => return admin_action_error(err),
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
    })
    .into_response()
}

pub(crate) async fn update_city_admin_login_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(input): Json<UpdateAdminNameInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let action_type = AdminActionType::UpdateCityAdmin;
    if let Err(resp) = ensure_action_role_allowed(&ctx, &action_type) {
        return resp;
    }
    let payload = UpdateCityAdminActionPayload {
        id,
        admin_name: Some(input.admin_name),
    };
    let result = state
        .db
        .with_client(move |conn| apply_update_city_admin_conn(conn, &ctx, &payload));
    let data = match result {
        Ok(v) => v,
        Err(err) => return admin_action_error(err),
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data,
    })
    .into_response()
}

pub(crate) async fn update_federal_admin_login_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(input): Json<UpdateAdminNameInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let action_type = AdminActionType::UpdateFederalAdmin;
    if let Err(resp) = ensure_action_role_allowed(&ctx, &action_type) {
        return resp;
    }
    let payload = UpdateFederalAdminActionPayload {
        id,
        admin_name: input.admin_name,
    };
    let result = state
        .db
        .with_client(move |conn| apply_update_federal_admin_conn(conn, &ctx, &payload));
    let data = match result {
        Ok(v) => v,
        Err(err) => return admin_action_error(err),
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data,
    })
    .into_response()
}

pub(crate) fn require_admin_security_grant(
    state: &AppState,
    headers: &HeaderMap,
    ctx: &AdminAuthContext,
    action_type: AdminActionType,
    target: &str,
    request_payload: Option<&serde_json::Value>,
) -> Result<(), axum::response::Response> {
    if action_type.is_login_state() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "login state action does not accept security grant",
        ));
    }
    let grant_id = headers
        .get(ADMIN_SECURITY_GRANT_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| api_error(StatusCode::FORBIDDEN, 1003, "security grant required"))?
        .to_string();
    let ctx = ctx.clone();
    let target = normalize_security_target(target);
    let request_payload = request_payload.cloned();
    let now = Utc::now();
    let result = state.db.with_client(move |conn| {
        repo::cleanup_security_state_conn(conn, now)?;
        let Some(mut grant) = repo::get_security_grant_conn(conn, grant_id.as_str())? else {
            return Err("http:forbidden:security grant not found".to_string());
        };
        if grant.consumed || now > grant.expires_at {
            return Err("http:forbidden:security grant expired".to_string());
        }
        if grant.action_type != action_type.as_str() {
            return Err("http:forbidden:security grant action mismatch".to_string());
        }
        if grant.auth_type != action_type.auth_type() {
            return Err("http:forbidden:security grant auth type mismatch".to_string());
        }
        if !same_admin_pubkey(grant.actor_pubkey.as_str(), ctx.admin_pubkey.as_str())
            || grant.actor_role != ctx.role
        {
            return Err("http:forbidden:security grant owner mismatch".to_string());
        }
        if grant.actor_province_name != ctx.admin_province.clone().unwrap_or_default()
            || grant.actor_city_name.as_deref() != ctx.admin_city.as_deref()
        {
            return Err("http:forbidden:security grant scope mismatch".to_string());
        }
        if grant.target != target {
            return Err("http:forbidden:security grant target mismatch".to_string());
        }
        if let Some(payload) = request_payload.as_ref() {
            if grant.payload_hash != hash_json(payload) {
                return Err("http:forbidden:security grant payload mismatch".to_string());
            }
        }
        grant.consumed = true;
        repo::insert_security_grant_conn(conn, &grant)
    });
    match result {
        Ok(()) => Ok(()),
        Err(err) => Err(admin_action_error(err)),
    }
}

fn preview_action_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    action_type: &AdminActionType,
    payload: &serde_json::Value,
) -> Result<ActionPreview, String> {
    match action_type {
        AdminActionType::CreateCityAdmin => {
            let input: CreateCityAdminInput = serde_json::from_value(payload.clone())
                .map_err(|_| "http:bad_request:invalid create payload".to_string())?;
            let (admin_pubkey, admin_name, city, created_by) =
                validate_create_city_admin_conn(conn, ctx, &input)?;
            let after = json!({
                "role": "CITY_ADMIN",
                "admin_pubkey": admin_pubkey,
                "admin_name": admin_name,
                "city_name": city,
                "created_by": created_by,
            });
            let after_hash = hash_json(&after);
            Ok(ActionPreview {
                before_hash: "none".to_string(),
                after_hash: after_hash.clone(),
                target: admin_pubkey.clone(),
                auth_type: action_type.auth_type(),
                display_fields: base_fields(action_type, ctx, admin_pubkey.as_str()),
            })
        }
        AdminActionType::UpdateCityAdmin => {
            Err("http:bad_request:update city_admin is login state action".to_string())
        }
        AdminActionType::DeleteCityAdmin => {
            let input: CityAdminIdPayload = serde_json::from_value(payload.clone())
                .map_err(|_| "http:bad_request:invalid delete payload".to_string())?;
            let city_admin = require_manageable_city_admin_conn(conn, ctx, input.id)?;
            let before = city_admin_row_from_user_conn(conn, &city_admin)?;
            let after =
                json!({ "deleted": true, "id": input.id, "admin_pubkey": city_admin.admin_pubkey });
            let before_hash = hash_serialized(&before);
            let after_hash = hash_json(&after);
            Ok(ActionPreview {
                before_hash,
                after_hash,
                target: city_admin.admin_pubkey,
                auth_type: action_type.auth_type(),
                display_fields: base_fields(
                    action_type,
                    ctx,
                    after["admin_pubkey"].as_str().unwrap_or("*"),
                ),
            })
        }
        AdminActionType::CreateFederalAdmin => {
            let input: CreateFederalAdminActionPayload = serde_json::from_value(payload.clone())
                .map_err(|_| "http:bad_request:invalid federal admin payload".to_string())?;
            let (admin_pubkey, admin_name, province) =
                validate_create_federal_admin_conn(conn, ctx, &input)?;
            let after = json!({
                "role": "FEDERAL_ADMIN",
                "province_name": province,
                "admin_pubkey": admin_pubkey.clone(),
                "admin_name": admin_name,
                "created_by": ctx.admin_pubkey,
            });
            let after_hash = hash_serialized(&after);
            Ok(ActionPreview {
                before_hash: "none".to_string(),
                after_hash,
                target: admin_pubkey.clone(),
                auth_type: action_type.auth_type(),
                display_fields: base_fields(action_type, ctx, admin_pubkey.as_str()),
            })
        }
        AdminActionType::UpdateFederalAdmin => {
            Err("http:bad_request:update federal admin is login state action".to_string())
        }
        AdminActionType::DeleteFederalAdmin => {
            let input: FederalAdminIdPayload = serde_json::from_value(payload.clone())
                .map_err(|_| "http:bad_request:invalid federal admin payload".to_string())?;
            let (before, after, target) = preview_delete_federal_admin_conn(conn, ctx, &input)?;
            Ok(ActionPreview {
                before_hash: hash_serialized(&before),
                after_hash: hash_serialized(&after),
                target: target.clone(),
                auth_type: action_type.auth_type(),
                display_fields: base_fields(action_type, ctx, target.as_str()),
            })
        }
        _ => preview_security_action(ctx, action_type, payload),
    }
}

fn preview_security_action(
    ctx: &AdminAuthContext,
    action_type: &AdminActionType,
    payload: &serde_json::Value,
) -> Result<ActionPreview, String> {
    let target = payload
        .get("target")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("sfid_number").and_then(|v| v.as_str()))
        .or_else(|| payload.get("sfid_number").and_then(|v| v.as_str()))
        .or_else(|| payload.get("challenge_id").and_then(|v| v.as_str()))
        .unwrap_or("*");
    let target = normalize_security_target(target);
    let request_hash = hash_json(payload);
    Ok(ActionPreview {
        before_hash: "security-grant".to_string(),
        after_hash: request_hash.clone(),
        target: target.clone(),
        auth_type: action_type.auth_type(),
        display_fields: base_fields(action_type, ctx, target.as_str()),
    })
}

fn base_fields(
    action_type: &AdminActionType,
    ctx: &AdminAuthContext,
    target: &str,
) -> Vec<serde_json::Value> {
    vec![
        field("action_type", "操作", action_type.label()),
        field(
            "province_name",
            "省份",
            ctx.admin_province.as_deref().unwrap_or_default(),
        ),
        field(
            "actor_pubkey",
            "管理员",
            display_account(ctx.admin_pubkey.as_str()).as_str(),
        ),
        field("target", "目标", display_account(target).as_str()),
    ]
}

fn validate_create_city_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &CreateCityAdminInput,
) -> Result<(String, String, String, String), String> {
    let Some(admin_pubkey) = normalize_admin_pubkey(input.admin_pubkey.as_str()) else {
        return Err("http:bad_request:admin_pubkey format invalid".to_string());
    };
    let admin_name = validate_admin_name(input.admin_name.as_str())?;
    let created_by = match input.created_by.as_deref().map(str::trim) {
        None | Some("") => ctx.admin_pubkey.clone(),
        Some(raw) => {
            let Some(normalized) = normalize_admin_pubkey(raw) else {
                return Err("http:bad_request:created_by format invalid".to_string());
            };
            if !same_admin_pubkey(normalized.as_str(), ctx.admin_pubkey.as_str()) {
                return Err(
                    "http:forbidden:FederalAdmin can only create city admins under itself"
                        .to_string(),
                );
            }
            normalized
        }
    };
    if let Some(existing) = repo::resolve_admin_pubkey_key_conn(conn, admin_pubkey.as_str())? {
        let role = repo::get_admin_by_pubkey_conn(conn, existing.as_str())?
            .map(|v| v.role)
            .unwrap_or(AdminRole::CityAdmin);
        return Err(duplicate_admin_pubkey_error(&role));
    }
    let (province, city) =
        ensure_city_in_creator_province_conn(conn, created_by.as_str(), input.city.as_str())
            .map_err(response_to_string)?;
    if count_city_admins_in_city_conn(conn, province.as_str(), city.as_str())?
        >= MAX_CITY_ADMINS_PER_CITY
    {
        return Err("http:conflict:city admin city limit reached".to_string());
    }
    Ok((admin_pubkey, admin_name, city, created_by))
}

fn validate_admin_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("http:bad_request:admin_name is required".to_string());
    }
    if name.chars().count() > MAX_ADMIN_NAME_CHARS {
        return Err("http:bad_request:admin_name too long".to_string());
    }
    Ok(name.to_string())
}

fn require_manageable_city_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    id: u64,
) -> Result<AdminUser, String> {
    let city_admin = find_city_admin_by_id_conn(conn, id)?
        .ok_or_else(|| "http:not_found:city admin not found".to_string())?;
    if !can_manage_city_admin_conn(
        conn,
        ctx.admin_pubkey.as_str(),
        ctx.admin_province.as_deref(),
        &city_admin,
    )? {
        return Err("http:forbidden:cannot manage other province city admins".to_string());
    }
    Ok(city_admin)
}

fn preview_update_city_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &UpdateCityAdminActionPayload,
) -> Result<(CityAdminRow, CityAdminRow, String), String> {
    let mut city_admin = require_manageable_city_admin_conn(conn, ctx, input.id)?;
    let before = city_admin_row_from_user_conn(conn, &city_admin)?;
    if let Some(next_name) = input.admin_name.as_deref() {
        city_admin.admin_name = validate_admin_name(next_name)?;
    }
    let after = city_admin_row_from_user_conn(conn, &city_admin)?;
    Ok((before, after, city_admin.admin_pubkey))
}

fn validate_create_federal_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &CreateFederalAdminActionPayload,
) -> Result<(String, String, String), String> {
    let province = ctx
        .admin_province
        .clone()
        .ok_or_else(|| "http:forbidden:admin province scope missing".to_string())?;
    let Some(admin_pubkey) = normalize_admin_pubkey(input.admin_pubkey.as_str()) else {
        return Err("http:bad_request:admin_pubkey format invalid".to_string());
    };
    let admin_name = validate_admin_name(input.admin_name.as_str())?;
    if let Some(existing) = repo::resolve_admin_pubkey_key_conn(conn, admin_pubkey.as_str())? {
        let role = repo::get_admin_by_pubkey_conn(conn, existing.as_str())?
            .map(|v| v.role)
            .unwrap_or(AdminRole::FederalAdmin);
        return Err(duplicate_admin_pubkey_error(&role));
    }
    if count_federal_admins_in_province_conn(conn, province.as_str())?
        >= MAX_FEDERAL_ADMINS_PER_PROVINCE
    {
        return Err("http:conflict:federal admin province limit reached".to_string());
    }
    Ok((admin_pubkey, admin_name, province))
}

fn count_federal_admins_in_province_conn(
    conn: &mut Client,
    province: &str,
) -> Result<usize, String> {
    repo::count_federal_admins_by_province_conn(conn, province)
}

fn find_federal_admin_by_id_conn(conn: &mut Client, id: u64) -> Result<Option<AdminUser>, String> {
    repo::get_admin_by_id_and_role_conn(conn, id, &AdminRole::FederalAdmin)
}

fn require_manageable_federal_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    id: u64,
) -> Result<(AdminUser, String), String> {
    let actor_province_name = ctx
        .admin_province
        .clone()
        .ok_or_else(|| "http:forbidden:admin province scope missing".to_string())?;
    let admin = find_federal_admin_by_id_conn(conn, id)?
        .ok_or_else(|| "http:not_found:federal admin not found".to_string())?;
    let target_province =
        repo::province_scope_for_role_conn(conn, &admin.admin_pubkey, &admin.role)?
            .ok_or_else(|| "http:conflict:federal admin province missing".to_string())?;
    if target_province != actor_province_name {
        return Err("http:forbidden:cannot manage other province federal admins".to_string());
    }
    Ok((admin, target_province))
}

fn actor_is_initial_federal_admin(ctx: &AdminAuthContext) -> bool {
    let Some(province) = ctx.admin_province.as_deref() else {
        return false;
    };
    federal_admin_province(ctx.admin_pubkey.as_str())
        .map(|built_in_province| built_in_province == province)
        .unwrap_or(false)
}

fn federal_admin_row_value(
    admin: &AdminUser,
    province: String,
) -> Result<serde_json::Value, String> {
    serde_json::to_value(FederalAdminRow {
        id: admin.id,
        province,
        admin_pubkey: admin.admin_pubkey.clone(),
        admin_name: admin.admin_name.clone(),
        built_in: admin.built_in,
        created_at: admin.created_at,
        updated_at: admin.updated_at,
    })
    .map_err(|e| format!("encode federal admin failed: {e}"))
}

fn preview_delete_federal_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &FederalAdminIdPayload,
) -> Result<(serde_json::Value, serde_json::Value, String), String> {
    if !actor_is_initial_federal_admin(ctx) {
        return Err("http:forbidden:initial federal admin required".to_string());
    }
    let (admin, province) = require_manageable_federal_admin_conn(conn, ctx, input.id)?;
    if same_admin_pubkey(admin.admin_pubkey.as_str(), ctx.admin_pubkey.as_str()) {
        return Err("http:forbidden:initial federal admin cannot delete itself".to_string());
    }
    if admin.built_in || federal_admin_province(admin.admin_pubkey.as_str()).is_some() {
        return Err("http:forbidden:built-in federal admin cannot be deleted".to_string());
    }
    let before = federal_admin_row_value(&admin, province.clone())?;
    let after = json!({
        "deleted": true,
        "id": input.id,
        "province_name": province,
        "admin_pubkey": admin.admin_pubkey.clone(),
    });
    Ok((before, after, admin.admin_pubkey))
}

fn recheck_preview_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    challenge: &AdminActionChallenge,
) -> Result<(), String> {
    let action_type = parse_action_type(challenge.action_type.as_str())
        .map_err(|_| "http:bad_request:unknown action_type".to_string())?;
    let preview = preview_action_conn(conn, ctx, &action_type, &challenge.request_payload)?;
    if preview.before_hash != challenge.before_hash {
        return Err("http:conflict:admin action state changed".to_string());
    }
    Ok(())
}

fn apply_action_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    challenge: &AdminActionChallenge,
) -> Result<serde_json::Value, String> {
    let action_type = parse_action_type(challenge.action_type.as_str())
        .map_err(|_| "http:bad_request:unknown action_type".to_string())?;
    match action_type {
        AdminActionType::CreateCityAdmin => {
            let input: CreateCityAdminInput =
                serde_json::from_value(challenge.request_payload.clone())
                    .map_err(|_| "http:bad_request:invalid create payload".to_string())?;
            apply_create_city_admin_conn(conn, ctx, &input)
        }
        AdminActionType::DeleteCityAdmin => {
            let input: CityAdminIdPayload =
                serde_json::from_value(challenge.request_payload.clone())
                    .map_err(|_| "http:bad_request:invalid delete payload".to_string())?;
            apply_delete_city_admin_conn(conn, ctx, &input)
        }
        AdminActionType::CreateFederalAdmin => {
            let input: CreateFederalAdminActionPayload =
                serde_json::from_value(challenge.request_payload.clone())
                    .map_err(|_| "http:bad_request:invalid federal admin payload".to_string())?;
            apply_create_federal_admin_conn(conn, ctx, &input)
        }
        AdminActionType::DeleteFederalAdmin => {
            let input: FederalAdminIdPayload =
                serde_json::from_value(challenge.request_payload.clone())
                    .map_err(|_| "http:bad_request:invalid federal admin payload".to_string())?;
            apply_delete_federal_admin_conn(conn, ctx, &input)
        }
        _ => Err(
            "http:bad_request:business action cannot be applied by admin governance endpoint"
                .to_string(),
        ),
    }
}

fn apply_create_city_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &CreateCityAdminInput,
) -> Result<serde_json::Value, String> {
    let (admin_pubkey, admin_name, city, created_by) =
        validate_create_city_admin_conn(conn, ctx, input)?;
    let now = Utc::now();
    let row = AdminUser {
        id: repo::next_admin_id_conn(conn)?,
        admin_pubkey: admin_pubkey.clone(),
        admin_name,
        role: AdminRole::CityAdmin,
        built_in: false,
        created_by,
        created_at: now,
        updated_at: Some(now),
        city,
    };
    repo::upsert_admin_conn(conn, &row, None)?;
    serde_json::to_value(city_admin_row_from_user_conn(conn, &row)?)
        .map_err(|e| format!("encode city admin failed: {e}"))
}

fn apply_delete_city_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &CityAdminIdPayload,
) -> Result<serde_json::Value, String> {
    let city_admin = require_manageable_city_admin_conn(conn, ctx, input.id)?;
    let pubkey = city_admin.admin_pubkey.clone();
    repo::delete_admin_runtime_state_conn(conn, pubkey.as_str())?;
    conn.execute(
        "DELETE FROM admins WHERE lower(admin_pubkey) = lower($1)",
        &[&pubkey],
    )
    .map_err(|e| format!("delete city admin failed: {e}"))?;
    Ok(json!({ "deleted": true, "admin_pubkey": pubkey }))
}

fn apply_update_city_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &UpdateCityAdminActionPayload,
) -> Result<serde_json::Value, String> {
    let (before, after, _) = preview_update_city_admin_conn(conn, ctx, input)?;
    let mut next = require_manageable_city_admin_conn(conn, ctx, input.id)?;
    next.admin_name = after.admin_name;
    next.updated_at = Some(Utc::now());
    repo::upsert_admin_conn(conn, &next, None)?;
    let _ = before;
    serde_json::to_value(city_admin_row_from_user_conn(conn, &next)?)
        .map_err(|e| format!("encode city admin failed: {e}"))
}

fn apply_create_federal_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &CreateFederalAdminActionPayload,
) -> Result<serde_json::Value, String> {
    let (admin_pubkey, admin_name, province) =
        validate_create_federal_admin_conn(conn, ctx, input)?;
    let now = Utc::now();
    let row = AdminUser {
        id: repo::next_admin_id_conn(conn)?,
        admin_pubkey: admin_pubkey.clone(),
        admin_name,
        role: AdminRole::FederalAdmin,
        built_in: false,
        created_by: ctx.admin_pubkey.clone(),
        created_at: now,
        updated_at: Some(now),
        city: String::new(),
    };
    repo::upsert_admin_conn(conn, &row, Some(province.as_str()))?;
    federal_admin_row_value(&row, province)
}

fn apply_update_federal_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &UpdateFederalAdminActionPayload,
) -> Result<serde_json::Value, String> {
    let (mut admin, province) = require_manageable_federal_admin_conn(conn, ctx, input.id)?;
    admin.admin_name = validate_admin_name(input.admin_name.as_str())?;
    admin.updated_at = Some(Utc::now());
    repo::upsert_admin_conn(conn, &admin, Some(province.as_str()))?;
    federal_admin_row_value(&admin, province)
}

fn apply_delete_federal_admin_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &FederalAdminIdPayload,
) -> Result<serde_json::Value, String> {
    let (_, _, _) = preview_delete_federal_admin_conn(conn, ctx, input)?;
    let (removed, _) = require_manageable_federal_admin_conn(conn, ctx, input.id)?;
    let pubkey = removed.admin_pubkey.clone();
    repo::delete_admin_runtime_state_conn(conn, pubkey.as_str())?;
    for mut city_admin in repo::list_city_admins_by_creator_conn(conn, pubkey.as_str())? {
        city_admin.created_by = ctx.admin_pubkey.clone();
        city_admin.updated_at = Some(Utc::now());
        repo::upsert_admin_conn(conn, &city_admin, None)?;
    }
    conn.execute(
        "DELETE FROM admins WHERE lower(admin_pubkey) = lower($1)",
        &[&pubkey],
    )
    .map_err(|e| format!("delete federal admin failed: {e}"))?;
    Ok(json!({ "deleted": true, "admin_pubkey": pubkey }))
}

fn hash_serialized<T: Serialize>(value: &T) -> String {
    let encoded = serde_json::to_vec(value).unwrap_or_default();
    format!("0x{}", hex::encode(Sha256::digest(&encoded)))
}

fn normalize_security_target(target: &str) -> String {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        "*".to_string()
    } else {
        trimmed.to_string()
    }
}

fn duplicate_admin_pubkey_error(role: &AdminRole) -> String {
    match role {
        AdminRole::FederalAdmin => "http:conflict:admin pubkey already exists as federal admin",
        AdminRole::CityAdmin => "http:conflict:admin pubkey already exists as city admin",
    }
    .to_string()
}

fn response_to_string(_resp: axum::response::Response) -> String {
    "http:bad_request:invalid request".to_string()
}

fn admin_action_error(err: String) -> axum::response::Response {
    if let Some(message) = err.strip_prefix("http:bad_request:") {
        return api_error(StatusCode::BAD_REQUEST, 1001, message);
    }
    if let Some(message) = err.strip_prefix("http:forbidden:") {
        return api_error(StatusCode::FORBIDDEN, 1003, message);
    }
    if let Some(message) = err.strip_prefix("http:not_found:") {
        return api_error(StatusCode::NOT_FOUND, 1004, message);
    }
    if let Some(message) = err.strip_prefix("http:conflict:") {
        return api_error(StatusCode::CONFLICT, 1005, message);
    }
    if let Some(message) = err.strip_prefix("http:unprocessable:") {
        return api_error(StatusCode::UNPROCESSABLE_ENTITY, 2004, message);
    }
    let message = format!("admin action failed: {err}");
    api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str())
}
