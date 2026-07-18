//! 管理员安全动作:会话 + QR_V1 冷钱包扫码签名(PasskeyColdSign 档)。
//!
//! 管理员治理动作、业务安全授权和短期挑战全部使用结构化表;
//! PasskeyColdSign 档 commit 校验冷钱包签名且 signer 须 ∈ 本机构链上 Active 集合。

use axum::{
    extract::State,
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

use crate::auth::action_sign::{
    hash_json, payload_hash_for_text, signed_payload_text, verify_citizen_wallet_signature,
    AdminSignedPayload, ADMIN_ACTION_TTL_SECONDS,
};
use crate::auth::city_registry_admins::{
    can_manage_city_registry, city_registry_row_from_user_conn,
    count_city_registry_admins_in_city_conn, ensure_city_in_province,
    find_city_registry_by_id_conn, MAX_ADMIN_NAME_CHARS, MAX_CITY_REGISTRY_ADMINS_PER_CITY,
};
use crate::auth::login::AdminAuthContext;
use crate::auth::operation_auth::{
    ensure_action_role_allowed, parse_action_type, AdminActionType, AdminOperationAuth,
};
use crate::auth::repo;
use crate::auth::security_model::{AdminActionChallenge, AdminSecurityGrant};
use crate::core::qr::build_sign_request;
use crate::crypto::pubkey::{normalize_admin_account, same_admin_account};
use crate::*;

const ADMIN_SECURITY_GRANT_TTL_SECONDS: i64 = 120;
pub(crate) const ADMIN_SECURITY_GRANT_HEADER: &str = "x-cid-security-grant";

#[derive(Debug, Deserialize)]
struct InstitutionDeregisterInput {
    cid_number: String,
    account_name: String,
}

/// 注销动作校验通过后解析出的目标(供 apply 写态 + commit 建凭证)。
struct DeregisterTarget {
    cid_number: String,
    account_name: String,
    target_hex: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PrepareAdminActionInput {
    action_type: AdminActionType,
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CommitAdminActionInput {
    action_id: String,
    /// 冷钱包扫码签名(PasskeyColdSign 档必填;Session 动作不走 commit)。
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
    actor_cid_number: String,
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
    actor_cid_number: String,
    sign_request: Option<String>,
    payload_hash: String,
    auth_type: AdminOperationAuth,
    expires_at: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct CityRegistryIdPayload {
    id: u64,
}

struct ActionPreview {
    before_hash: String,
    after_hash: String,
    target: String,
    auth_type: AdminOperationAuth,
}

/// 从节点唯一 active 绑定读取当前机构 CID，并与登录上下文机构严格对齐。
pub(crate) fn actor_cid_number_for_context(
    db: &Db,
    ctx: &AdminAuthContext,
) -> Result<String, axum::response::Response> {
    let binding = repo::active_node_binding(db).map_err(|err| {
        tracing::error!(error = %err, "query active institution binding failed");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "node binding query failed",
        )
    })?;
    let binding =
        binding.ok_or_else(|| api_error(StatusCode::FORBIDDEN, 2002, "node binding missing"))?;
    if binding.institution_code != ctx.institution_code {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            2002,
            "node binding institution mismatch",
        ));
    }
    let cid_number = binding.institution_cid_number;
    if cid_number.is_empty()
        || cid_number.len() > primitives::core_const::CID_NUMBER_MAX_BYTES as usize
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            2002,
            "node binding actor_cid_number missing",
        ));
    }
    Ok(cid_number)
}

/// 校验账号 ∈ 本机构链上 Active 管理员集合(冷签 step-up 与替换目标校验共用)。
async fn ensure_pubkey_on_chain_admin(
    db: &Db,
    actor_cid_number: &str,
    account_pubkey: &str,
    message: &'static str,
) -> Result<(), axum::response::Response> {
    use crate::core::chain_runtime;
    let binding = repo::active_node_binding(db).map_err(|err| {
        tracing::error!(error = %err, "query node binding failed");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "node binding query failed",
        )
    })?;
    let Some(binding) = binding else {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            2002,
            "node binding missing",
        ));
    };
    if binding.institution_cid_number != actor_cid_number {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            2002,
            "actor_cid_number does not match node binding",
        ));
    }
    let identity = chain_runtime::identity_from_binding_parts(
        &binding.institution_code,
        Some(actor_cid_number),
        binding.frg_province_code.as_deref(),
    )
    .map_err(|err| {
        tracing::error!(error = %err, "node binding is invalid");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "node binding invalid",
        )
    })?;
    let onchain = chain_runtime::fetch_active_admins_onchain(&identity)
        .await
        .map_err(|err| {
            tracing::warn!(error = %err, "chain unreachable during action commit");
            api_error(StatusCode::BAD_GATEWAY, 5002, "chain unreachable")
        })?
        .ok_or_else(|| api_error(StatusCode::FORBIDDEN, 2002, "not an on-chain admin"))?;
    let normalized = chain_runtime::normalize_account_pubkey(account_pubkey)
        .ok_or_else(|| api_error(StatusCode::FORBIDDEN, 2002, message))?;
    if !onchain
        .iter()
        .any(|admin| same_admin_account(admin, normalized.as_str()))
    {
        return Err(api_error(StatusCode::FORBIDDEN, 2002, message));
    }
    Ok(())
}

/// 校验扫码签名 signer ∈ 本机构链上 Active 管理员集合(冷签 step-up,与登录同源)。
async fn ensure_signer_on_chain_admin(
    db: &Db,
    actor_cid_number: &str,
    signer_pubkey: &str,
) -> Result<(), axum::response::Response> {
    ensure_pubkey_on_chain_admin(db, actor_cid_number, signer_pubkey, "not an on-chain admin").await
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
    if input.action_type.auth_type() != AdminOperationAuth::PasskeyColdSign {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "only cold-sign actions can be prepared",
        );
    }
    let actor_cid_number = match actor_cid_number_for_context(&state.db, &ctx) {
        Ok(value) => value,
        Err(resp) => return resp,
    };
    let preview = match state.db.with_client({
        let ctx = ctx.clone();
        let action_type = input.action_type.clone();
        let payload = input.payload.clone();
        move |conn| preview_action_conn(conn, &ctx, &action_type, &payload)
    }) {
        Ok(v) => v,
        Err(err) => return admin_action_error(err),
    };
    let now = Utc::now();
    let expires_at = now + Duration::seconds(ADMIN_ACTION_TTL_SECONDS);
    let action_id = format!("cid-admin-action-{}", Uuid::new_v4());
    let province = ctx.scope_province_name.clone().unwrap_or_default();
    let request_hash = hash_json(&input.payload);
    let (payload_text, payload_hash, sign_request) =
        if preview.auth_type == AdminOperationAuth::PasskeyColdSign {
            let payload_text = signed_payload_text(AdminSignedPayload {
                domain: "onchina_admin_governance",
                qr_proto: crate::core::qr::QR_V1,
                action_id: action_id.as_str(),
                action_type: input.action_type.as_str(),
                actor_pubkey: ctx.admin_account.as_str(),
                actor_cid_number: actor_cid_number.as_str(),
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
                ctx.admin_account.as_str(),
                payload_text.as_str(),
                crate::core::qr::action_onchina_admin(),
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
        actor_account: ctx.admin_account.clone(),
        actor_institution_code: ctx.institution_code.clone(),
        actor_cid_number: actor_cid_number.clone(),
        actor_province_name: province,
        actor_city_name: ctx.scope_city_name.clone(),
        auth_type: preview.auth_type.clone(),
        target: preview.target,
        payload_text,
        payload_hash: payload_hash.clone(),
        before_hash: preview.before_hash,
        after_hash: preview.after_hash,
        request_payload: input.payload,
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
            actor_cid_number,
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
    let now = Utc::now();
    let challenge = match state.db.with_client({
        let action_id = input.action_id.clone();
        let actor_account = ctx.admin_account.clone();
        move |conn| {
            repo::cleanup_security_state_conn(conn, now)?;
            repo::get_action_challenge_conn(conn, action_id.as_str(), actor_account.as_str())
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
    if !same_admin_account(challenge.actor_account.as_str(), ctx.admin_account.as_str()) {
        return api_error(StatusCode::FORBIDDEN, 1003, "admin action owner mismatch");
    }
    let actor_cid_number = match actor_cid_number_for_context(&state.db, &ctx) {
        Ok(value) => value,
        Err(resp) => return resp,
    };
    if challenge.actor_cid_number != actor_cid_number {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin action actor_cid_number mismatch",
        );
    }
    let action_type = match parse_action_type(challenge.action_type.as_str()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Err(resp) = ensure_action_role_allowed(&ctx, &action_type) {
        return resp;
    }
    if action_type.is_session() || challenge.auth_type == AdminOperationAuth::Session {
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
    // ── 冷签 step-up:冷钱包扫码签名 + signer ∈ 本机构链上 Active 集合。
    //    所有可 commit 动作(Session 已在上方拒绝)一律走此校验。
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
        ctx.admin_account.as_str(),
        signer_pubkey,
        signature,
        payload_hash,
        challenge.payload_hash.as_str(),
        challenge.payload_text.as_str(),
    ) {
        return resp;
    }
    if let Err(resp) =
        ensure_signer_on_chain_admin(&state.db, &actor_cid_number, signer_pubkey).await
    {
        return resp;
    }
    // 闭包 move 会拿走 action_type,克隆一份供注销动作的 commit 后处理用。
    let action_type_for_credential = action_type.clone();
    let result = state.db.with_client({
        let ctx = ctx.clone();
        let challenge = challenge.clone();
        move |conn| {
            repo::cleanup_security_state_conn(conn, now)?;
            let mut current = repo::get_action_challenge_conn(
                conn,
                challenge.action_id.as_str(),
                ctx.admin_account.as_str(),
            )?
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
                let grant_id = format!("cid-admin-grant-{}", Uuid::new_v4());
                let grant_expires_at = now + Duration::seconds(ADMIN_SECURITY_GRANT_TTL_SECONDS);
                let grant = AdminSecurityGrant {
                    grant_id: grant_id.clone(),
                    action_type: action_type.as_str().to_string(),
                    actor_account: ctx.admin_account.clone(),
                    actor_institution_code: ctx.institution_code.clone(),
                    actor_cid_number: current.actor_cid_number.clone(),
                    actor_province_name: ctx.scope_province_name.clone().unwrap_or_default(),
                    actor_city_name: ctx.scope_city_name.clone(),
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
                    actor_cid_number: current.actor_cid_number,
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
    // ── 自定义账户注销动作：apply 已写 ISSUED 行，此处生成凭证并回填签名字段。
    //    签发失败则删除该无签名 ISSUED 行,保持一致(不留无签名残行)。
    if action_type_for_credential == AdminActionType::InstitutionAccountDeregister {
        if let CommitAdminActionOutput::Applied(ref value) = output {
            let cid_number = value
                .get("cid_number")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let account_name = value
                .get("account_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let target_hex = value
                .get("target_account")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let nonce = value
                .get("deregister_nonce")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let drop_issued = |state: &AppState, nonce: String| {
                let _ = state.db.with_client(move |conn| {
                    repo::delete_deregistration_by_nonce_conn(conn, &nonce)
                });
            };
            let Some(target_32) = crate::auth::login::parse_sr25519_pubkey_bytes(&target_hex)
            else {
                drop_issued(&state, nonce);
                return admin_action_error("http:internal:target account parse failed".to_string());
            };
            let credential_actor_cid_number = match repo::active_node_binding(&state.db) {
                Ok(Some(binding)) => {
                    if binding.institution_cid_number.trim().is_empty() {
                        drop_issued(&state, nonce);
                        return admin_action_error(
                            "http:internal:active binding cid_number is required".to_string(),
                        );
                    }
                    binding.institution_cid_number
                }
                Ok(None) => {
                    drop_issued(&state, nonce);
                    return admin_action_error(
                        "http:internal:active institution binding is required".to_string(),
                    );
                }
                Err(err) => {
                    drop_issued(&state, nonce);
                    return admin_action_error(format!(
                        "http:internal:query active institution binding failed: {err}"
                    ));
                }
            };
            let cred = match crate::core::chain_runtime::build_institution_deregistration_credential(
                &state,
                &credential_actor_cid_number,
                &cid_number,
                &account_name,
                &target_32,
                nonce.clone(),
            ) {
                Ok(c) => c,
                Err(err) => {
                    drop_issued(&state, nonce);
                    return admin_action_error(format!(
                        "issue deregistration credential failed: {err}"
                    ));
                }
            };
            if let Err(err) = state.db.with_client({
                let nonce = nonce.clone();
                let signature = cred.signature.clone();
                let issuer_cid = cred.credential_issuer_cid_number.clone();
                let signer_pubkey = cred.credential_signer_pubkey.clone();
                move |conn| {
                    repo::set_deregistration_credential_conn(
                        conn,
                        &nonce,
                        &signature,
                        &issuer_cid,
                        &signer_pubkey,
                    )
                }
            }) {
                return admin_action_error(format!(
                    "persist deregistration credential failed: {err}"
                ));
            }
        }
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: output,
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
    if action_type.is_session() {
        return ensure_action_role_allowed(ctx, &action_type);
    }
    if action_type.auth_type() == AdminOperationAuth::Passkey {
        crate::auth::passkey::require_passkey_assertion(state, headers, &ctx.admin_account)?;
        return ensure_action_role_allowed(ctx, &action_type);
    }
    consume_admin_security_grant(state, headers, ctx, action_type, target, request_payload)
        .map(|_| ())
}

pub(crate) fn consume_admin_security_grant(
    state: &AppState,
    headers: &HeaderMap,
    ctx: &AdminAuthContext,
    action_type: AdminActionType,
    target: &str,
    request_payload: Option<&serde_json::Value>,
) -> Result<AdminSecurityGrant, axum::response::Response> {
    if action_type.is_session() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "security grant is not available for session action",
        ));
    }
    // PasskeyColdSign 档:先消费 passkey 断言(fail-closed,绝不降档),再消费冷签 grant。
    crate::auth::passkey::require_passkey_assertion(state, headers, &ctx.admin_account)?;
    if action_type.auth_type() == AdminOperationAuth::Passkey {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "security grant is not available for passkey action",
        ));
    }
    let actor_cid_number = actor_cid_number_for_context(&state.db, ctx)?;
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
        let Some(mut grant) =
            repo::get_security_grant_conn(conn, grant_id.as_str(), ctx.admin_account.as_str())?
        else {
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
        if !same_admin_account(grant.actor_account.as_str(), ctx.admin_account.as_str())
            || grant.actor_institution_code != ctx.institution_code
            || grant.actor_cid_number != actor_cid_number
        {
            return Err("http:forbidden:security grant owner mismatch".to_string());
        }
        if grant.actor_province_name != ctx.scope_province_name.clone().unwrap_or_default()
            || grant.actor_city_name.as_deref() != ctx.scope_city_name.as_deref()
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
        let consumed = grant.clone();
        grant.consumed = true;
        repo::insert_security_grant_conn(conn, &grant)?;
        Ok(consumed)
    });
    match result {
        Ok(grant) => Ok(grant),
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
        AdminActionType::CreateCityRegistry => {
            let input: CreateCityRegistryAdminInput = serde_json::from_value(payload.clone())
                .map_err(|_| "http:bad_request:invalid create payload".to_string())?;
            let (admin_account, admin_name, city, created_by) =
                validate_create_city_registry_conn(conn, ctx, &input)?;
            let after = json!({
                "institution_code": "CREG",
                "admin_account": admin_account,
                "admin_name": admin_name,
                "city_name": city,
                "created_by": created_by,
            });
            let after_hash = hash_json(&after);
            Ok(ActionPreview {
                before_hash: "none".to_string(),
                after_hash: after_hash.clone(),
                target: admin_account.clone(),
                auth_type: action_type.auth_type(),
            })
        }
        AdminActionType::DeleteCityRegistry => {
            let input: CityRegistryIdPayload = serde_json::from_value(payload.clone())
                .map_err(|_| "http:bad_request:invalid delete payload".to_string())?;
            let city_registry = require_manageable_city_registry_conn(conn, ctx, input.id)?;
            let before = city_registry_row_from_user_conn(conn, &city_registry)?;
            let after = json!({ "deleted": true, "id": input.id, "admin_account": city_registry.admin_account });
            let before_hash = hash_serialized(&before);
            let after_hash = hash_json(&after);
            Ok(ActionPreview {
                before_hash,
                after_hash,
                target: city_registry.admin_account,
                auth_type: action_type.auth_type(),
            })
        }
        AdminActionType::InstitutionAccountDeregister => {
            let target = validate_institution_deregister_conn(conn, ctx, action_type, payload)?;
            let after = json!({
                "deregister": true,
                "cid_number": target.cid_number,
                "account_name": target.account_name,
            });
            Ok(ActionPreview {
                before_hash: "none".to_string(),
                after_hash: hash_json(&after),
                target: target.target_hex.clone(),
                auth_type: action_type.auth_type(),
            })
        }
        AdminActionType::NodeBindingUnbind => preview_node_binding_unbind_conn(conn, action_type),
        AdminActionType::InstitutionCreate => {
            precheck_institution_create_scope(ctx, payload)?;
            preview_security_action(action_type, payload)
        }
        AdminActionType::InstitutionCreateAccount
        | AdminActionType::InstitutionDeleteAccount
        | AdminActionType::InstitutionDeleteDocument => {
            precheck_institution_target_scope_conn(conn, ctx, payload)?;
            preview_security_action(action_type, payload)
        }
        _ => preview_security_action(action_type, payload),
    }
}

/// 对已存在机构的特殊操作(建/删账户、删机构文档)在 prepare 阶段预检管辖权,
/// 与 accounts handler 的 get_visible_scope 校验等价。文档删除流程的业务 handler 自身不含
/// 省/市校验,此预检即为其唯一管辖权闸:越权管理员拿不到一次性 grant,无法跨省操作他机构。
fn precheck_institution_target_scope_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let cid_number = payload
        .get("target")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("cid_number").and_then(|v| v.as_str()))
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "http:bad_request:cid_number is required".to_string())?;
    let Some((inst, _accounts)) = Db::get_institution_with_accounts_conn(conn, cid_number)? else {
        return Err("http:not_found:institution not found".to_string());
    };
    let scope = crate::scope::rules::get_visible_scope(ctx);
    if !scope.includes_province(&inst.province_name)
        || !scope.includes_city(&inst.city_name)
        || !scope.includes_town(&inst.town_name)
    {
        return Err("http:forbidden:institution out of current admin scope".to_string());
    }
    Ok(())
}

/// 新建机构在 prepare 阶段预检省/市/镇管辖权,与 create_institution_inner 的
/// locked_province/locked_city 校验逐字段等价:scope 锁定省/市/镇时,申报省/市/镇必须留空或
/// 等于锁定值(留空交业务 handler 用锁定值回填),不会比 handler 更严而误拒。
/// 管理员阈值由链端按严格多数自动计算,prepare 不再接收 threshold 字段。
fn precheck_institution_create_scope(
    ctx: &AdminAuthContext,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let scope = crate::scope::rules::get_visible_scope(ctx);
    check_locked_field(
        scope.locked_province_name.as_deref(),
        payload.get("province_name").and_then(|v| v.as_str()),
        "province",
    )?;
    check_locked_field(
        scope.locked_city_name.as_deref(),
        payload.get("city_name").and_then(|v| v.as_str()),
        "city",
    )?;
    check_locked_field(
        scope.locked_town_name.as_deref(),
        payload.get("town_name").and_then(|v| v.as_str()),
        "town",
    )?;
    let admins = payload
        .get("admins")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "http:bad_request:admins is required".to_string())?;
    if admins.len() < 2 {
        return Err(
            "http:bad_request:institution admins must contain at least two accounts".to_string(),
        );
    }
    let mut normalized_accounts: Vec<String> = Vec::with_capacity(admins.len());
    for item in admins {
        let raw = item
            .get("admin_account")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| "http:bad_request:admin_account is required".to_string())?;
        let Some(normalized) = normalize_admin_account(raw) else {
            return Err("http:bad_request:admin_account format invalid".to_string());
        };
        if normalized_accounts
            .iter()
            .any(|account| account.eq_ignore_ascii_case(normalized.as_str()))
        {
            return Err("http:bad_request:duplicate admin_account".to_string());
        }
        normalized_accounts.push(normalized);
    }
    Ok(())
}

/// scope 锁定某行政维度时,申报值必须留空(交 handler 回填)或逐字等于锁定值,
/// 否则视为越权。锁定为 None(该档不限此维度)时不校验。
fn check_locked_field(
    locked: Option<&str>,
    requested: Option<&str>,
    field: &str,
) -> Result<(), String> {
    if let (Some(locked), Some(requested)) =
        (locked, requested.map(str::trim).filter(|v| !v.is_empty()))
    {
        if requested != locked {
            return Err(format!("http:forbidden:{field} out of current admin scope"));
        }
    }
    Ok(())
}

fn preview_security_action(
    action_type: &AdminActionType,
    payload: &serde_json::Value,
) -> Result<ActionPreview, String> {
    let target = payload
        .get("target")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("cid_number").and_then(|v| v.as_str()))
        .or_else(|| payload.get("cid_number").and_then(|v| v.as_str()))
        .or_else(|| payload.get("challenge_id").and_then(|v| v.as_str()))
        .unwrap_or("*");
    let target = normalize_security_target(target);
    let request_hash = hash_json(payload);
    Ok(ActionPreview {
        before_hash: "security-grant".to_string(),
        after_hash: request_hash.clone(),
        target: target.clone(),
        auth_type: action_type.auth_type(),
    })
}

fn validate_create_city_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &CreateCityRegistryAdminInput,
) -> Result<(String, String, String, String), String> {
    let Some(admin_account) = normalize_admin_account(input.admin_account.as_str()) else {
        return Err("http:bad_request:admin_account format invalid".to_string());
    };
    let admin_name = validate_admin_name(input.admin_name.as_str())?;
    let created_by = match input.created_by.as_deref().map(str::trim) {
        None | Some("") => ctx.admin_account.clone(),
        Some(raw) => {
            let Some(normalized) = normalize_admin_account(raw) else {
                return Err("http:bad_request:created_by format invalid".to_string());
            };
            if !same_admin_account(normalized.as_str(), ctx.admin_account.as_str()) {
                return Err(
                    "http:forbidden:FederalRegistry can only create city registry admins under itself"
                        .to_string(),
                );
            }
            normalized
        }
    };
    if let Some(existing) = repo::resolve_admin_account_key_conn(conn, admin_account.as_str())? {
        let institution_code = repo::get_admin_by_account_conn(conn, existing.as_str())?
            .map(|v| v.institution_code)
            .unwrap_or_else(|| "CREG".to_string());
        return Err(duplicate_admin_account_error(&institution_code));
    }
    let province_name = ctx
        .scope_province_name
        .as_deref()
        .ok_or_else(|| "http:forbidden:admin province scope missing".to_string())?;
    let (province, city) = ensure_city_in_province(province_name, input.city_name.as_str())
        .map_err(response_to_string)?;
    if count_city_registry_admins_in_city_conn(conn, province.as_str(), city.as_str())?
        >= MAX_CITY_REGISTRY_ADMINS_PER_CITY
    {
        return Err("http:conflict:city admin city limit reached".to_string());
    }
    Ok((admin_account, admin_name, city, created_by))
}

/// 机构自定义命名账户注销校验。协议账户永久存在，机构本身没有注销路径。
fn validate_institution_deregister_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    action_type: &AdminActionType,
    payload: &serde_json::Value,
) -> Result<DeregisterTarget, String> {
    let input: InstitutionDeregisterInput = serde_json::from_value(payload.clone())
        .map_err(|_| "http:bad_request:invalid deregister payload".to_string())?;
    let cid_number = input.cid_number.trim().to_string();
    if cid_number.is_empty() {
        return Err("http:bad_request:cid_number is required".to_string());
    }
    let Some((inst, accounts)) = Db::get_institution_with_accounts_conn(conn, &cid_number)? else {
        return Err("http:not_found:institution not found".to_string());
    };
    // 管辖:发起注册局管理员的可见域必须覆盖该机构所在省/市。
    let visible = crate::scope::rules::get_visible_scope(ctx);
    if !visible.includes_province(&inst.province_name)
        || !visible.includes_city(&inst.city_name)
        || !visible.includes_town(&inst.town_name)
    {
        return Err("http:forbidden:out of admin scope".to_string());
    }
    let account_name = match action_type {
        AdminActionType::InstitutionAccountDeregister => input.account_name.trim().to_string(),
        _ => return Err("http:bad_request:not a deregister action".to_string()),
    };
    let Some(kind) = primitives::account_derive::institution_kind_by_name(
        cid_number.as_bytes(),
        account_name.as_bytes(),
    ) else {
        return Err("http:bad_request:account_name is required".to_string());
    };
    if !kind.is_closable_institution_account() {
        return Err("http:forbidden:protocol institution account cannot be closed".to_string());
    }
    // 账户存在性只按 CID + account_name 查找；链上关闭时由 runtime 再核对账户归属。
    let _account = accounts
        .iter()
        .find(|a| a.account_name == account_name)
        .ok_or_else(|| "http:not_found:account not found".to_string())?;
    // target = derive_account(cid, account_name)(与链端 derive_account 同源,= propose_close 所关账户)。
    let target_hex =
        crate::institution::accounts::derive::derive_account(&cid_number, &account_name)
            .ok_or_else(|| "http:internal:derive target account failed".to_string())?;
    Ok(DeregisterTarget {
        cid_number,
        account_name,
        target_hex,
    })
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

fn require_manageable_city_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    id: u64,
) -> Result<AdminUser, String> {
    let city_registry = find_city_registry_by_id_conn(conn, id)?
        .ok_or_else(|| "http:not_found:city admin not found".to_string())?;
    if !can_manage_city_registry(ctx.scope_province_name.as_deref(), &city_registry) {
        return Err("http:forbidden:cannot manage other province city registry admins".to_string());
    }
    Ok(city_registry)
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
        AdminActionType::CreateCityRegistry => {
            let input: CreateCityRegistryAdminInput =
                serde_json::from_value(challenge.request_payload.clone())
                    .map_err(|_| "http:bad_request:invalid create payload".to_string())?;
            apply_create_city_registry_conn(conn, ctx, &input)
        }
        AdminActionType::DeleteCityRegistry => {
            let input: CityRegistryIdPayload =
                serde_json::from_value(challenge.request_payload.clone())
                    .map_err(|_| "http:bad_request:invalid delete payload".to_string())?;
            apply_delete_city_registry_conn(conn, ctx, &input)
        }
        AdminActionType::InstitutionAccountDeregister => {
            let target = validate_institution_deregister_conn(
                conn,
                ctx,
                &action_type,
                &challenge.request_payload,
            )?;
            let nonce = format!("dereg-{}", Uuid::new_v4().simple());
            // 凭证字段先空占位，commit 层生成签名后原子回填。
            repo::insert_deregistration_issued_conn(
                conn,
                &target.cid_number,
                &target.account_name,
                &target.target_hex,
                &nonce,
                "",
                "",
                ctx.admin_account.as_str(),
            )?;
            Ok(json!({
                "cid_number": target.cid_number,
                "account_name": target.account_name,
                "target_account": target.target_hex,
                "deregister_nonce": nonce,
            }))
        }
        AdminActionType::NodeBindingUnbind => apply_node_binding_unbind_conn(conn),
        _ => Err(
            "http:bad_request:business action cannot be applied by admin governance endpoint"
                .to_string(),
        ),
    }
}

fn preview_node_binding_unbind_conn(
    conn: &mut Client,
    action_type: &AdminActionType,
) -> Result<ActionPreview, String> {
    let Some(binding) = repo::get_active_node_binding_conn(conn)? else {
        return Err("http:conflict:node binding missing".to_string());
    };
    let binding_id = binding.binding_id.clone();
    let candidate_id = binding.candidate_id.clone();
    let institution_code = binding.institution_code.clone();
    let after = json!({
        "unbind": true,
        "binding_id": binding_id,
        "candidate_id": candidate_id,
        "institution_code": institution_code,
    });
    Ok(ActionPreview {
        before_hash: hash_serialized(&binding),
        after_hash: hash_json(&after),
        target: binding.candidate_id,
        auth_type: action_type.auth_type(),
    })
}

fn apply_node_binding_unbind_conn(conn: &mut Client) -> Result<serde_json::Value, String> {
    let Some(binding) = repo::get_active_node_binding_conn(conn)? else {
        return Err("http:conflict:node binding missing".to_string());
    };
    let changed = repo::deactivate_active_node_binding_conn(conn)?;
    if changed == 0 {
        return Err("http:conflict:node binding already inactive".to_string());
    }
    let removed_sessions = repo::delete_all_admin_sessions_conn(conn)?;
    Ok(json!({
        "binding_id": binding.binding_id,
        "candidate_id": binding.candidate_id,
        "institution_code": binding.institution_code,
        "status": "INACTIVE",
        "removed_sessions": removed_sessions,
    }))
}

fn apply_create_city_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &CreateCityRegistryAdminInput,
) -> Result<serde_json::Value, String> {
    let (admin_account, admin_name, city, created_by) =
        validate_create_city_registry_conn(conn, ctx, input)?;
    let now = Utc::now();
    let row = AdminUser {
        id: repo::next_admin_id_conn(conn)?,
        admin_account: admin_account.clone(),
        admin_name,
        institution_code: "CREG".to_string(),
        built_in: false,
        created_by,
        created_at: now,
        updated_at: Some(now),
        city_name: city,
    };
    repo::upsert_admin_conn(conn, &row)?;
    serde_json::to_value(city_registry_row_from_user_conn(conn, &row)?)
        .map_err(|e| format!("encode city admin failed: {e}"))
}

fn apply_delete_city_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &CityRegistryIdPayload,
) -> Result<serde_json::Value, String> {
    let city_registry = require_manageable_city_registry_conn(conn, ctx, input.id)?;
    let admin_account = city_registry.admin_account.clone();
    repo::delete_admin_runtime_state_conn(conn, admin_account.as_str())?;
    conn.execute(
        "DELETE FROM admins WHERE lower(admin_account) = lower($1)",
        &[&admin_account],
    )
    .map_err(|e| format!("delete city admin failed: {e}"))?;
    Ok(json!({ "deleted": true, "admin_account": admin_account }))
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

fn duplicate_admin_account_error(institution_code: &str) -> String {
    if crate::core::chain_runtime::is_tier1_registry(institution_code) {
        "http:conflict:admin admin_account already exists as federal admin"
    } else {
        "http:conflict:admin admin_account already exists as city admin"
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
