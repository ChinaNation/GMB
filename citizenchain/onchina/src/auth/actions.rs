//! 管理员安全动作:会话 + QR_V1 冷钱包扫码签名(PasskeyColdSign 档)。
//!
//! 中文注释:管理员治理动作、业务安全授权和短期挑战全部使用结构化表;
//! PasskeyColdSign 档 commit 校验冷钱包签名且 signer 须 ∈ 本机构链上 Active 集合。

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

use crate::auth::action_sign::{
    hash_json, payload_hash_for_text, signed_payload_text, verify_citizen_wallet_signature,
    AdminSignedPayload, ADMIN_ACTION_TTL_SECONDS,
};
use crate::auth::city_registry_admins::{
    can_manage_city_registry_conn, city_registry_row_from_user_conn,
    count_city_registry_admins_in_city_conn, ensure_city_in_creator_province_conn,
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

/// 注销作用域(必须与链端 public-manage/private-manage 的 SCOPE_INSTITUTION/SCOPE_ACCOUNT 同值)。
const SCOPE_INSTITUTION: u8 = 0;
const SCOPE_ACCOUNT: u8 = 1;

#[derive(Debug, Deserialize)]
struct InstitutionDeregisterInput {
    cid_number: String,
    #[serde(default)]
    account_name: Option<String>,
}

/// 中文注释:注销动作校验通过后解析出的目标(供 apply 写态 + commit 建凭证)。
struct DeregisterTarget {
    cid_number: String,
    account_name: String,
    scope: u8,
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
    sign_request: Option<String>,
    payload_hash: String,
    auth_type: AdminOperationAuth,
    expires_at: i64,
    /// 机构上链创建专用:b.d 携带 propose_create_institution 裸 SCALE call data 的 QR_V1/k=1。
    /// 冷钱包解码逐字段核对后冷签 origin **并由 CitizenWallet 直接提交**;onchina 不提交。
    /// 仅 InstitutionCreate 动作有值,其余动作为 None。
    #[serde(skip_serializing_if = "Option::is_none")]
    institution_create_sign_request: Option<String>,
    /// 机构管理员集合上链专用:b.d 携带 federal_set_city_registry_admins 裸 SCALE call data。
    /// 仅 CreateCityRegistry/DeleteCityRegistry 动作有值,其余为 None;onchina 不提交。
    #[serde(skip_serializing_if = "Option::is_none")]
    admin_set_sign_request: Option<String>,
}

/// InstitutionCreate prepare 载荷:cid_number + 阈值 + 管理员职务/任期表单。
#[derive(Debug, Deserialize)]
struct InstitutionCreatePayload {
    cid_number: String,
    threshold: u32,
    #[serde(default)]
    admins: Vec<crate::institution::subjects::registration_call::AdminProfileFormInput>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CityRegistryIdPayload {
    id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateCityRegistryActionPayload {
    id: u64,
    #[serde(default)]
    admin_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateFederalRegistryActionPayload {
    id: u64,
    admin_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReplaceFederalRegistryActionPayload {
    id: u64,
    admin_account: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateAdminNameInput {
    admin_name: String,
}

struct ActionPreview {
    before_hash: String,
    after_hash: String,
    target: String,
    auth_type: AdminOperationAuth,
}

/// 校验账号 ∈ 本机构链上 Active 管理员集合(冷签 step-up 与替换目标校验共用)。
async fn ensure_pubkey_on_chain_admin(
    account_pubkey: &str,
    message: &'static str,
) -> Result<(), axum::response::Response> {
    use crate::core::chain_runtime;
    let identity = chain_runtime::node_institution_identity().map_err(|err| {
        tracing::error!(error = %err, "node institution identity misconfigured");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "node identity misconfigured",
        )
    })?;
    let onchain =
        chain_runtime::fetch_active_admins_onchain(&identity.admin_pallets, &identity.main_account)
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
async fn ensure_signer_on_chain_admin(signer_pubkey: &str) -> Result<(), axum::response::Response> {
    ensure_pubkey_on_chain_admin(signer_pubkey, "not an on-chain admin").await
}

async fn ensure_replace_target_on_chain_admin(
    input: &ReplaceFederalRegistryActionPayload,
) -> Result<(), axum::response::Response> {
    ensure_pubkey_on_chain_admin(
        input.admin_account.as_str(),
        "replacement admin is not an on-chain admin",
    )
    .await
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
                domain: "cid_admin_governance",
                qr_proto: crate::core::qr::QR_V1,
                action_id: action_id.as_str(),
                action_type: input.action_type.as_str(),
                actor_pubkey: ctx.admin_account.as_str(),
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
                crate::core::qr::ACTION_CID_ADMIN,
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
    // ── 机构上链创建:额外产出 b.d=SCALE call data 的 QR(冷钱包冷签 origin + 提交)。
    //    onchina 不提交 extrinsic。同时把机构/账户链投影置 PENDING_ON_CHAIN。
    let institution_create_sign_request = if input.action_type == AdminActionType::InstitutionCreate
    {
        match build_institution_create_sign_request(
            &state,
            action_id.as_str(),
            now.timestamp(),
            expires_at.timestamp(),
            ctx.admin_account.as_str(),
            &challenge.request_payload,
        ) {
            Ok(v) => Some(v),
            Err(resp) => return resp,
        }
    } else {
        None
    };
    // ── 市注册局管理员集合上链:CreateCityRegistry 并入新管理员、DeleteCityRegistry 移除,
    //    产出 federal_set_city_registry_admins 裸 SCALE call data 的 QR;onchina 不提交。
    let admin_set_sign_request = match input.action_type {
        AdminActionType::CreateCityRegistry | AdminActionType::DeleteCityRegistry => {
            match build_city_registry_admin_set_sign_request(
                &state,
                action_id.as_str(),
                now.timestamp(),
                expires_at.timestamp(),
                ctx.admin_account.as_str(),
                &challenge.request_payload,
                input.action_type == AdminActionType::CreateCityRegistry,
            ) {
                Ok(v) => Some(v),
                Err(resp) => return resp,
            }
        }
        AdminActionType::ReplaceFederalRegistry => {
            match build_federal_registry_replace_sign_request(
                &state,
                action_id.as_str(),
                now.timestamp(),
                expires_at.timestamp(),
                ctx.admin_account.as_str(),
                &challenge.request_payload,
            ) {
                Ok(v) => Some(v),
                Err(resp) => return resp,
            }
        }
        _ => None,
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PrepareAdminActionOutput {
            action_id,
            action_type: input.action_type.as_str().to_string(),
            sign_request,
            payload_hash,
            auth_type: preview.auth_type,
            expires_at: expires_at.timestamp(),
            institution_create_sign_request,
            admin_set_sign_request,
        },
    })
    .into_response()
}

/// 组装市注册局管理员集合上链的 QR_V1/k=1 sign_request(b.d=federal_set_city_registry_admins 裸 SCALE)。
///
/// 中文注释:按 created_by 的联邦作用域解析省/市,取该城市当前 CREG 管理员集合,按 add 增/删 delta,
/// 编码 federal_set call data。onchina 不提交;冷钱包解码核对后冷签 origin 并由 CitizenWallet 提交。
fn build_city_registry_admin_set_sign_request(
    state: &AppState,
    action_id: &str,
    issued_at: i64,
    expires_at: i64,
    actor_pubkey: &str,
    payload: &serde_json::Value,
    add: bool,
) -> Result<String, axum::response::Response> {
    let input: CreateCityRegistryAdminInput =
        serde_json::from_value(payload.clone()).map_err(|_| {
            api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "invalid city registry admin payload",
            )
        })?;
    let Some(new_admin_account) = normalize_admin_account(input.admin_account.as_str()) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_account format invalid",
        ));
    };
    let created_by = match input.created_by.as_deref().map(str::trim) {
        None | Some("") => actor_pubkey.to_string(),
        Some(raw) => match normalize_admin_account(raw) {
            Some(v) => v,
            None => {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "created_by format invalid",
                ))
            }
        },
    };
    let city_name = input.city_name.trim().to_string();
    let chain = state
        .db
        .with_client(move |conn| {
            // 省:created_by(FRG)的联邦作用域单源;市码:china.sqlite 解析并校验属省。
            let province_name =
                repo::province_scope_for_registry_org_conn(conn, created_by.as_str(), "FRG")?
                    .ok_or_else(|| "cannot resolve province from created_by".to_string())?;
            let Some(city_code) =
                crate::cid::china::city_code_by_name(province_name.as_str(), city_name.as_str())
            else {
                return Err("city not found in province".to_string());
            };
            if city_code == "000" {
                return Err("province placeholder city is not allowed".to_string());
            }
            // 当前 CREG 管理员集合(进链账户)。
            let (_total, current) = repo::list_city_registry_admins_by_scope_conn(
                conn,
                province_name.as_str(),
                Some(city_name.as_str()),
                1000,
                0,
            )?;
            let current_accounts: Vec<String> =
                current.iter().map(|u| u.admin_account.clone()).collect();
            crate::institution::admins::admin_set_call::build_city_registry_admin_set_call_data(
                conn,
                province_name.as_str(),
                city_name.as_str(),
                city_code,
                &current_accounts,
                new_admin_account.as_str(),
                add,
            )
        })
        .map_err(admin_action_error)?;
    crate::core::qr::build_sign_request_bytes(
        action_id,
        issued_at,
        expires_at,
        actor_pubkey,
        &chain.call_data,
        chain.action,
    )
}

/// 组装联邦注册局替换的 QR_V1/k=1 sign_request(b.d=propose_admin_set_change 裸 SCALE)。
///
/// 中文注释:取全部 FRG 管理员集合,把 id 对应的旧账户换成新账户,编码 genesis
/// propose_admin_set_change(call 0,FRG 内部投票)。onchina 不提交;冷钱包提交。
fn build_federal_registry_replace_sign_request(
    state: &AppState,
    action_id: &str,
    issued_at: i64,
    expires_at: i64,
    actor_pubkey: &str,
    payload: &serde_json::Value,
) -> Result<String, axum::response::Response> {
    let input: ReplaceFederalRegistryActionPayload = serde_json::from_value(payload.clone())
        .map_err(|_| {
            api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "invalid replace federal registry payload",
            )
        })?;
    let Some(new_admin_account) = normalize_admin_account(input.admin_account.as_str()) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_account format invalid",
        ));
    };
    let id = input.id;
    let chain = state
        .db
        .with_client(move |conn| {
            let old_admin = repo::get_admin_by_id_and_registry_org_conn(conn, id, "FRG")?
                .ok_or_else(|| "federal registry admin not found".to_string())?;
            let old_account = old_admin.admin_account.clone();
            let current = repo::list_federal_registry_admins_by_province_conn(conn, None)?;
            let current_accounts: Vec<String> = current
                .iter()
                .map(|(u, _)| u.admin_account.clone())
                .collect();
            crate::institution::admins::admin_set_call::build_federal_registry_admin_set_call_data(
                conn,
                &current_accounts,
                old_account.as_str(),
                new_admin_account.as_str(),
            )
        })
        .map_err(admin_action_error)?;
    crate::core::qr::build_sign_request_bytes(
        action_id,
        issued_at,
        expires_at,
        actor_pubkey,
        &chain.call_data,
        chain.action,
    )
}

/// 组装机构上链创建的 QR_V1/k=1 sign_request(b.d=propose_create_institution 裸 SCALE call data)。
///
/// 中文注释:同一事务里组装 call data + 置机构/账户链投影 PENDING_ON_CHAIN;
/// 失败回滚不留半态。call data 含注册局凭证(register_nonce/signature/issuer/scope),
/// 冷钱包解码逐字段核对后冷签 origin,由 CitizenWallet 提交;onchina 绝不提交。
fn build_institution_create_sign_request(
    state: &AppState,
    action_id: &str,
    issued_at: i64,
    expires_at: i64,
    actor_pubkey: &str,
    payload: &serde_json::Value,
) -> Result<String, axum::response::Response> {
    let input: InstitutionCreatePayload =
        serde_json::from_value(payload.clone()).map_err(|_| {
            api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "invalid institution create payload",
            )
        })?;
    let cid_number = input.cid_number.trim().to_string();
    let chain = state
        .db
        .with_client({
            let state = state.clone();
            let cid_number = cid_number.clone();
            let admins = input.admins.clone();
            move |conn| {
                let data = crate::institution::subjects::registration_call::build_create_institution_call_data(
                    &state,
                    conn,
                    cid_number.as_str(),
                    input.threshold,
                    &admins,
                )?;
                set_institution_chain_status_conn(conn, cid_number.as_str(), "PENDING_ON_CHAIN")?;
                Ok(data)
            }
        })
        .map_err(admin_action_error)?;
    crate::core::qr::build_sign_request_bytes(
        action_id,
        issued_at,
        expires_at,
        actor_pubkey,
        &chain.call_data,
        chain.action,
    )
}

/// 在已有连接上把机构主体 + 其全部账户的链投影状态置为指定值(创建上链 PENDING/回写复用)。
fn set_institution_chain_status_conn(
    conn: &mut Client,
    cid_number: &str,
    status: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE subjects SET chain_status = $2, updated_at = now() WHERE cid_number = $1",
        &[&cid_number, &status],
    )
    .map_err(|e| format!("update subjects chain_status failed: {e}"))?;
    conn.execute(
        "UPDATE accounts SET chain_status = $2 WHERE cid_number = $1",
        &[&cid_number, &status],
    )
    .map_err(|e| format!("update accounts chain_status failed: {e}"))?;
    Ok(())
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
    if let Err(resp) = ensure_signer_on_chain_admin(signer_pubkey).await {
        return resp;
    }
    if action_type == AdminActionType::ReplaceFederalRegistry {
        let input: ReplaceFederalRegistryActionPayload =
            match serde_json::from_value(challenge.request_payload.clone()) {
                Ok(v) => v,
                Err(_) => {
                    return api_error(
                        StatusCode::BAD_REQUEST,
                        1001,
                        "invalid federal admin replacement payload",
                    )
                }
            };
        if let Err(resp) = ensure_replace_target_on_chain_admin(&input).await {
            return resp;
        }
    }
    // 中文注释:闭包 move 会拿走 action_type,克隆一份供注销动作的 commit 后处理用。
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
    // ── 注销动作:apply 已写 ISSUED 行(conn 级);此处(有 state)建凭证 + 回填 signature/issuer。
    //    签发失败则删除该无签名 ISSUED 行,保持一致(不留无签名残行)。
    if matches!(
        action_type_for_credential,
        AdminActionType::InstitutionDeregister | AdminActionType::InstitutionAccountDeregister
    ) {
        if let CommitAdminActionOutput::Applied(ref value) = output {
            let scope = value.get("scope").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
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
            let cred = match crate::core::chain_runtime::build_institution_deregistration_credential(
                &state,
                scope,
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
                let issuer_cid = cred.issuer_cid_number.clone();
                let issuer_main = cred.issuer_main_account.clone();
                let signer_pubkey = cred.signer_pubkey.clone();
                move |conn| {
                    repo::set_deregistration_credential_conn(
                        conn,
                        &nonce,
                        &signature,
                        &issuer_cid,
                        &issuer_main,
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

pub(crate) async fn update_city_registry_login_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(input): Json<UpdateAdminNameInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let action_type = AdminActionType::UpdateCityRegistry;
    if let Err(resp) = ensure_action_role_allowed(&ctx, &action_type) {
        return resp;
    }
    // Passkey 重要档 step-up:消费 passkey 断言(fail-closed)。
    if let Err(resp) =
        crate::auth::passkey::require_passkey_assertion(&state, &headers, &ctx.admin_account)
    {
        return resp;
    }
    let payload = UpdateCityRegistryActionPayload {
        id,
        admin_name: Some(input.admin_name),
    };
    let result = state
        .db
        .with_client(move |conn| apply_update_city_registry_conn(conn, &ctx, &payload));
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

pub(crate) async fn update_federal_registry_login_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(input): Json<UpdateAdminNameInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let action_type = AdminActionType::UpdateFederalRegistry;
    if let Err(resp) = ensure_action_role_allowed(&ctx, &action_type) {
        return resp;
    }
    // Passkey 重要档 step-up:消费 passkey 断言(fail-closed)。
    if let Err(resp) =
        crate::auth::passkey::require_passkey_assertion(&state, &headers, &ctx.admin_account)
    {
        return resp;
    }
    let payload = UpdateFederalRegistryActionPayload {
        id,
        admin_name: input.admin_name,
    };
    let result = state
        .db
        .with_client(move |conn| apply_update_federal_registry_conn(conn, &ctx, &payload));
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
    // Session 档不要求扫码签名 grant——会话已是链上已证管理员,仅按动作角色边界
    // (联邦/省 scope)放行,不消费一次性 grant。Passkey / PasskeyColdSign 档继续校验 grant。
    if action_type.is_session() {
        return ensure_action_role_allowed(ctx, &action_type);
    }
    // Passkey / PasskeyColdSign 档:先消费 passkey 断言(fail-closed,绝不降档)。
    crate::auth::passkey::require_passkey_assertion(state, headers, &ctx.admin_account)?;
    // Passkey 档(重要操作)到此按角色边界放行;PasskeyColdSign 档继续校验冷签 grant。
    if action_type.auth_type() == AdminOperationAuth::Passkey {
        return ensure_action_role_allowed(ctx, &action_type);
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
        AdminActionType::UpdateCityRegistry => {
            Err("http:bad_request:update city_registry is login state action".to_string())
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
        AdminActionType::UpdateFederalRegistry => {
            Err("http:bad_request:update federal admin is login state action".to_string())
        }
        AdminActionType::ReplaceFederalRegistry => {
            let input: ReplaceFederalRegistryActionPayload =
                serde_json::from_value(payload.clone())
                    .map_err(|_| "http:bad_request:invalid federal admin payload".to_string())?;
            let (before, after, target) = preview_replace_federal_registry_conn(conn, ctx, &input)?;
            Ok(ActionPreview {
                before_hash: hash_serialized(&before),
                after_hash: hash_serialized(&after),
                target: target.clone(),
                auth_type: action_type.auth_type(),
            })
        }
        AdminActionType::InstitutionDeregister | AdminActionType::InstitutionAccountDeregister => {
            let target = validate_institution_deregister_conn(conn, ctx, action_type, payload)?;
            let after = json!({
                "deregister": true,
                "cid_number": target.cid_number,
                "account_name": target.account_name,
                "scope": target.scope,
            });
            Ok(ActionPreview {
                before_hash: "none".to_string(),
                after_hash: hash_json(&after),
                target: target.target_hex.clone(),
                auth_type: action_type.auth_type(),
            })
        }
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

/// 中文注释:对已存在机构的特殊操作(建/删账户、删机构文档)在 prepare 阶段预检管辖权,
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

/// 中文注释:新建机构在 prepare 阶段预检省/市/镇管辖权,与 create_institution_inner 的
/// locked_province/locked_city 校验逐字段等价:scope 锁定省/市/镇时,申报省/市/镇必须留空或
/// 等于锁定值(留空交业务 handler 用锁定值回填),不会比 handler 更严而误拒。
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
    Ok(())
}

/// 中文注释:scope 锁定某行政维度时,申报值必须留空(交 handler 回填)或逐字等于锁定值,
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
    let (province, city) =
        ensure_city_in_creator_province_conn(conn, created_by.as_str(), input.city_name.as_str())
            .map_err(response_to_string)?;
    if count_city_registry_admins_in_city_conn(conn, province.as_str(), city.as_str())?
        >= MAX_CITY_REGISTRY_ADMINS_PER_CITY
    {
        return Err("http:conflict:city admin city limit reached".to_string());
    }
    Ok((admin_account, admin_name, city, created_by))
}

/// 中文注释:机构/账户注销校验。conn 级(查存+管辖+派生),不触签名(签名在 commit 层)。
/// 创世/治理机构由链端 `is_genesis_protected`/org 闸权威拒;此处 created_by='SYSTEM' 是 CID 侧
/// 纵深 + UX(不让根基机构进入注销流程）。
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
    // 拒根基:创世/官方机构(行政区生成、created_by=SYSTEM)永不可注销。
    if inst.created_by.trim().eq_ignore_ascii_case("SYSTEM") {
        return Err("http:forbidden:cannot deregister official institution".to_string());
    }
    let (account_name, scope) = match action_type {
        AdminActionType::InstitutionDeregister => (
            crate::institution::subjects::service::ACCOUNT_NAME_MAIN.to_string(),
            SCOPE_INSTITUTION,
        ),
        AdminActionType::InstitutionAccountDeregister => {
            let name = input
                .account_name
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| "http:bad_request:account_name is required".to_string())?
                .to_string();
            if name == crate::institution::subjects::service::ACCOUNT_NAME_MAIN {
                return Err(
                    "http:bad_request:use InstitutionDeregister for the main account".to_string(),
                );
            }
            (name, SCOPE_ACCOUNT)
        }
        _ => return Err("http:bad_request:not a deregister action".to_string()),
    };
    // 账户查存 + 链上活跃。
    let account = accounts
        .iter()
        .find(|a| a.account_name == account_name)
        .ok_or_else(|| "http:not_found:account not found".to_string())?;
    if account.chain_status
        != crate::institution::subjects::model::MultisigChainStatus::ActiveOnChain
    {
        return Err("http:unprocessable:account not active on chain".to_string());
    }
    // target = derive_account(cid, account_name)(与链端 derive_account 同源,= propose_close 所关账户)。
    let target_hex =
        crate::institution::accounts::derive::derive_account(&cid_number, &account_name)
            .ok_or_else(|| "http:internal:derive target account failed".to_string())?;
    Ok(DeregisterTarget {
        cid_number,
        account_name,
        scope,
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
    if !can_manage_city_registry_conn(
        conn,
        ctx.admin_account.as_str(),
        ctx.scope_province_name.as_deref(),
        &city_registry,
    )? {
        return Err("http:forbidden:cannot manage other province city registry admins".to_string());
    }
    Ok(city_registry)
}

fn preview_update_city_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &UpdateCityRegistryActionPayload,
) -> Result<(CityRegistryAdminRow, CityRegistryAdminRow, String), String> {
    let mut city_registry = require_manageable_city_registry_conn(conn, ctx, input.id)?;
    let before = city_registry_row_from_user_conn(conn, &city_registry)?;
    if let Some(next_name) = input.admin_name.as_deref() {
        city_registry.admin_name = validate_admin_name(next_name)?;
    }
    let after = city_registry_row_from_user_conn(conn, &city_registry)?;
    Ok((before, after, city_registry.admin_account))
}

fn validate_replacement_federal_registry_account_conn(
    conn: &mut Client,
    current: &AdminUser,
    input: &ReplaceFederalRegistryActionPayload,
) -> Result<String, String> {
    let Some(admin_account) = normalize_admin_account(input.admin_account.as_str()) else {
        return Err("http:bad_request:admin_account format invalid".to_string());
    };
    if same_admin_account(admin_account.as_str(), current.admin_account.as_str()) {
        return Err("http:bad_request:replacement admin_account must be different".to_string());
    }
    if let Some(existing) = repo::resolve_admin_account_key_conn(conn, admin_account.as_str())? {
        let institution_code = repo::get_admin_by_account_conn(conn, existing.as_str())?
            .map(|v| v.institution_code)
            .unwrap_or_else(|| "FRG".to_string());
        return Err(duplicate_admin_account_error(&institution_code));
    }
    Ok(admin_account)
}

fn find_federal_registry_by_id_conn(
    conn: &mut Client,
    id: u64,
) -> Result<Option<AdminUser>, String> {
    repo::get_admin_by_id_and_registry_org_conn(conn, id, "FRG")
}

fn require_manageable_federal_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    id: u64,
) -> Result<(AdminUser, String), String> {
    let actor_province_name = ctx
        .scope_province_name
        .clone()
        .ok_or_else(|| "http:forbidden:admin province scope missing".to_string())?;
    let admin = find_federal_registry_by_id_conn(conn, id)?
        .ok_or_else(|| "http:not_found:federal admin not found".to_string())?;
    let target_province = repo::province_scope_for_registry_org_conn(
        conn,
        &admin.admin_account,
        &admin.institution_code,
    )?
    .ok_or_else(|| "http:conflict:federal admin province missing".to_string())?;
    if target_province != actor_province_name {
        return Err(
            "http:forbidden:cannot manage other province federal registry admins".to_string(),
        );
    }
    Ok((admin, target_province))
}

fn federal_registry_row_value(
    admin: &AdminUser,
    province_name: String,
) -> Result<serde_json::Value, String> {
    serde_json::to_value(FederalRegistryAdminRow {
        id: admin.id,
        province_name,
        admin_account: admin.admin_account.clone(),
        admin_name: admin.admin_name.clone(),
        built_in: admin.built_in,
        created_at: admin.created_at,
        updated_at: admin.updated_at,
    })
    .map_err(|e| format!("encode federal admin failed: {e}"))
}

fn preview_replace_federal_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &ReplaceFederalRegistryActionPayload,
) -> Result<(serde_json::Value, serde_json::Value, String), String> {
    let (admin, province) = require_manageable_federal_registry_conn(conn, ctx, input.id)?;
    let replacement_account =
        validate_replacement_federal_registry_account_conn(conn, &admin, input)?;
    let before = federal_registry_row_value(&admin, province.clone())?;
    let after = json!({
        "replaced": true,
        "id": input.id,
        "province_name": province,
        "old_admin_account": admin.admin_account.clone(),
        "new_admin_account": replacement_account,
    });
    Ok((before, after, admin.admin_account))
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
        AdminActionType::ReplaceFederalRegistry => {
            let input: ReplaceFederalRegistryActionPayload =
                serde_json::from_value(challenge.request_payload.clone())
                    .map_err(|_| "http:bad_request:invalid federal admin payload".to_string())?;
            apply_replace_federal_registry_conn(conn, ctx, &input)
        }
        AdminActionType::InstitutionDeregister | AdminActionType::InstitutionAccountDeregister => {
            let target = validate_institution_deregister_conn(
                conn,
                ctx,
                &action_type,
                &challenge.request_payload,
            )?;
            let nonce = format!("dereg-{}", Uuid::new_v4().simple());
            // 中文注释:issuer 三字段空占位,commit 层建凭证后回填 signature + issuer(同源)。
            repo::insert_deregistration_issued_conn(
                conn,
                &target.cid_number,
                &target.account_name,
                target.scope,
                &target.target_hex,
                &nonce,
                "",
                "",
                "",
                ctx.admin_account.as_str(),
            )?;
            Ok(json!({
                "cid_number": target.cid_number,
                "account_name": target.account_name,
                "scope": target.scope,
                "target_account": target.target_hex,
                "deregister_nonce": nonce,
            }))
        }
        _ => Err(
            "http:bad_request:business action cannot be applied by admin governance endpoint"
                .to_string(),
        ),
    }
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
    repo::upsert_admin_conn(conn, &row, None)?;
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

fn apply_update_city_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &UpdateCityRegistryActionPayload,
) -> Result<serde_json::Value, String> {
    let (before, after, _) = preview_update_city_registry_conn(conn, ctx, input)?;
    let mut next = require_manageable_city_registry_conn(conn, ctx, input.id)?;
    next.admin_name = after.admin_name;
    next.updated_at = Some(Utc::now());
    repo::upsert_admin_conn(conn, &next, None)?;
    let _ = before;
    serde_json::to_value(city_registry_row_from_user_conn(conn, &next)?)
        .map_err(|e| format!("encode city admin failed: {e}"))
}

fn apply_update_federal_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &UpdateFederalRegistryActionPayload,
) -> Result<serde_json::Value, String> {
    let (mut admin, province) = require_manageable_federal_registry_conn(conn, ctx, input.id)?;
    admin.admin_name = validate_admin_name(input.admin_name.as_str())?;
    admin.updated_at = Some(Utc::now());
    repo::upsert_admin_conn(conn, &admin, Some(province.as_str()))?;
    federal_registry_row_value(&admin, province)
}

fn apply_replace_federal_registry_conn(
    conn: &mut Client,
    ctx: &AdminAuthContext,
    input: &ReplaceFederalRegistryActionPayload,
) -> Result<serde_json::Value, String> {
    let (mut admin, province) = require_manageable_federal_registry_conn(conn, ctx, input.id)?;
    let old_account = admin.admin_account.clone();
    let replacement_account =
        validate_replacement_federal_registry_account_conn(conn, &admin, input)?;
    repo::delete_admin_runtime_state_conn(conn, old_account.as_str())?;
    for mut city_registry in
        repo::list_city_registry_admins_by_creator_conn(conn, old_account.as_str())?
    {
        city_registry.created_by = replacement_account.clone();
        city_registry.updated_at = Some(Utc::now());
        repo::upsert_admin_conn(conn, &city_registry, None)?;
    }
    admin.admin_account = replacement_account.clone();
    admin.admin_name = String::new();
    admin.built_in = false;
    admin.created_by = ctx.admin_account.clone();
    admin.updated_at = Some(Utc::now());
    conn.execute(
        "UPDATE admins
         SET admin_account = $1, admin_name = $2, built_in = $3, created_by = $4, updated_at = $5
         WHERE admin_id = $6",
        &[
            &admin.admin_account,
            &admin.admin_name,
            &admin.built_in,
            &admin.created_by,
            &admin.updated_at,
            &(admin.id as i64),
        ],
    )
    .map_err(|e| format!("replace federal admin failed: {e}"))?;
    federal_registry_row_value(&admin, province)
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
    if institution_code == "FRG" {
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
