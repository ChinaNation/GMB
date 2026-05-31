//! 管理员安全动作:Passkey 与 WUMIN_QR_V1/sign_request 冷钱包签名。
//!
//! 中文注释:省管理员治理写操作仍在这里直接 apply；省/市管理员业务写操作
//! 只在这里换取一次性安全 grant,再由所属业务模块消费 grant 后落库。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use webauthn_rs::prelude::{PublicKeyCredential, RequestChallengeResponse};

use crate::admins::operators::{
    allocate_next_admin_user_id, can_manage_operator, ensure_city_in_creator_province,
    find_operator_pubkey_by_id, operator_row_from_user, MAX_ADMIN_NAME_CHARS,
};
use crate::admins::passkeys::{
    active_passkeys, cleanup_admin_security_challenges, hash_json, payload_hash_for_text,
    signed_payload_text, update_passkey_usage, verify_cold_wallet_signature, webauthn,
    AdminSignedPayload, ADMIN_ACTION_TTL_SECONDS,
};
use crate::admins::province_admins::sheng_admin_province;
use crate::crypto::pubkey::{normalize_admin_pubkey, same_admin_pubkey};
use crate::login::AdminAuthContext;
use crate::models::{AdminActionChallenge, AdminSecurityGrant, AdminSecurityLevel};
use crate::qr::{build_sign_request, display_field as field};
use crate::*;

const ADMIN_SECURITY_GRANT_TTL_SECONDS: i64 = 120;
pub(crate) const ADMIN_SECURITY_GRANT_HEADER: &str = "x-sfid-security-grant";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminActionType {
    CreateOperator,
    UpdateOperator,
    DeleteOperator,
    CreateShengAdmin,
    UpdateShengAdmin,
    DeleteShengAdmin,
    InstitutionCreate,
    InstitutionUpdate,
    InstitutionCreateAccount,
    InstitutionDeleteAccount,
    InstitutionUploadDocument,
    InstitutionDeleteDocument,
    PublicSecurityReconcile,
    CitizenBindCommit,
    CpmsStatusImportConfirm,
    CpmsIssueInstallCode,
    CpmsRevokeInstallToken,
    CpmsReissueInstallToken,
    CpmsDisableKeys,
    CpmsEnableKeys,
    CpmsRevokeKeys,
    CpmsDeleteKeys,
}

impl AdminActionType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::CreateOperator => "CREATE_OPERATOR",
            Self::UpdateOperator => "UPDATE_OPERATOR",
            Self::DeleteOperator => "DELETE_OPERATOR",
            Self::CreateShengAdmin => "CREATE_SHENG_ADMIN",
            Self::UpdateShengAdmin => "UPDATE_SHENG_ADMIN",
            Self::DeleteShengAdmin => "DELETE_SHENG_ADMIN",
            Self::InstitutionCreate => "INSTITUTION_CREATE",
            Self::InstitutionUpdate => "INSTITUTION_UPDATE",
            Self::InstitutionCreateAccount => "INSTITUTION_CREATE_ACCOUNT",
            Self::InstitutionDeleteAccount => "INSTITUTION_DELETE_ACCOUNT",
            Self::InstitutionUploadDocument => "INSTITUTION_UPLOAD_DOCUMENT",
            Self::InstitutionDeleteDocument => "INSTITUTION_DELETE_DOCUMENT",
            Self::PublicSecurityReconcile => "PUBLIC_SECURITY_RECONCILE",
            Self::CitizenBindCommit => "CITIZEN_BIND_COMMIT",
            Self::CpmsStatusImportConfirm => "CPMS_STATUS_IMPORT_CONFIRM",
            Self::CpmsIssueInstallCode => "CPMS_ISSUE_INSTALL_CODE",
            Self::CpmsRevokeInstallToken => "CPMS_REVOKE_INSTALL_TOKEN",
            Self::CpmsReissueInstallToken => "CPMS_REISSUE_INSTALL_TOKEN",
            Self::CpmsDisableKeys => "CPMS_DISABLE_KEYS",
            Self::CpmsEnableKeys => "CPMS_ENABLE_KEYS",
            Self::CpmsRevokeKeys => "CPMS_REVOKE_KEYS",
            Self::CpmsDeleteKeys => "CPMS_DELETE_KEYS",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::CreateOperator => "新增市级管理员",
            Self::UpdateOperator => "编辑市级管理员",
            Self::DeleteOperator => "删除市级管理员",
            Self::CreateShengAdmin => "新增省级管理员",
            Self::UpdateShengAdmin => "编辑省级管理员",
            Self::DeleteShengAdmin => "删除省级管理员",
            Self::InstitutionCreate => "创建机构",
            Self::InstitutionUpdate => "更新机构",
            Self::InstitutionCreateAccount => "新增机构账户",
            Self::InstitutionDeleteAccount => "删除机构账户",
            Self::InstitutionUploadDocument => "上传机构文档",
            Self::InstitutionDeleteDocument => "删除机构文档",
            Self::PublicSecurityReconcile => "公安局机构对账",
            Self::CitizenBindCommit => "确认电子护照绑定",
            Self::CpmsStatusImportConfirm => "导入 CPMS 年度报告",
            Self::CpmsIssueInstallCode => "签发 CPMS 安装码",
            Self::CpmsRevokeInstallToken => "作废 CPMS 安装令牌",
            Self::CpmsReissueInstallToken => "重新签发 CPMS 安装码",
            Self::CpmsDisableKeys => "禁用 CPMS 授权",
            Self::CpmsEnableKeys => "启用 CPMS 授权",
            Self::CpmsRevokeKeys => "吊销 CPMS 授权",
            Self::CpmsDeleteKeys => "删除 CPMS 授权",
        }
    }

    fn security_level(&self) -> AdminSecurityLevel {
        match self {
            Self::CreateOperator
            | Self::UpdateOperator
            | Self::DeleteOperator
            | Self::CreateShengAdmin
            | Self::UpdateShengAdmin
            | Self::DeleteShengAdmin
            | Self::CpmsIssueInstallCode
            | Self::CpmsRevokeInstallToken
            | Self::CpmsReissueInstallToken
            | Self::CpmsDisableKeys
            | Self::CpmsEnableKeys
            | Self::CpmsRevokeKeys
            | Self::CpmsDeleteKeys => AdminSecurityLevel::Important,
            Self::InstitutionCreate
            | Self::InstitutionUpdate
            | Self::InstitutionCreateAccount
            | Self::InstitutionDeleteAccount
            | Self::InstitutionUploadDocument
            | Self::InstitutionDeleteDocument
            | Self::PublicSecurityReconcile
            | Self::CitizenBindCommit
            | Self::CpmsStatusImportConfirm => AdminSecurityLevel::General,
        }
    }

    fn is_governance(&self) -> bool {
        matches!(
            self,
            Self::CreateOperator
                | Self::UpdateOperator
                | Self::DeleteOperator
                | Self::CreateShengAdmin
                | Self::UpdateShengAdmin
                | Self::DeleteShengAdmin
        )
    }

    fn is_cpms(&self) -> bool {
        matches!(
            self,
            Self::CpmsIssueInstallCode
                | Self::CpmsRevokeInstallToken
                | Self::CpmsReissueInstallToken
                | Self::CpmsDisableKeys
                | Self::CpmsEnableKeys
                | Self::CpmsRevokeKeys
                | Self::CpmsDeleteKeys
        )
    }
}

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
    security_level: AdminSecurityLevel,
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
    security_level: AdminSecurityLevel,
    expires_at: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct OperatorIdPayload {
    id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateOperatorActionPayload {
    id: u64,
    #[serde(default)]
    admin_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CreateShengAdminActionPayload {
    admin_pubkey: String,
    admin_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateShengAdminActionPayload {
    id: u64,
    admin_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ShengAdminIdPayload {
    id: u64,
}

struct ActionPreview {
    before_hash: String,
    after_hash: String,
    target: String,
    security_level: AdminSecurityLevel,
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
    let webauthn = match webauthn() {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let now = Utc::now();
    let expires_at = now + Duration::seconds(ADMIN_ACTION_TTL_SECONDS);
    let action_id = format!("sfid-admin-action-{}", Uuid::new_v4());
    let province = ctx.admin_province.clone().unwrap_or_default();
    let (passkeys, preview) = {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let passkeys = active_passkeys(&store, ctx.admin_pubkey.as_str());
        if passkeys.is_empty() {
            return api_error(StatusCode::FORBIDDEN, 1003, "passkey required");
        }
        let preview = match preview_action(&store, &ctx, &input.action_type, &input.payload) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        (passkeys, preview)
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
    let request_hash = hash_json(&input.payload);
    let (payload_text, payload_hash, sign_request) =
        if preview.security_level == AdminSecurityLevel::Important {
            let payload_text = signed_payload_text(AdminSignedPayload {
                domain: "sfid_admin_governance",
                qr_proto: crate::qr::WUMIN_QR_V1,
                action_id: action_id.as_str(),
                action_type: input.action_type.as_str(),
                actor_pubkey: ctx.admin_pubkey.as_str(),
                actor_province: province.as_str(),
                target: preview.target.as_str(),
                request_hash: request_hash.as_str(),
                before_hash: preview.before_hash.as_str(),
                after_hash: preview.after_hash.as_str(),
                expires_at: expires_at.timestamp(),
            });
            let payload_hash = payload_hash_for_text(payload_text.as_str());
            let mut display_fields = preview.display_fields.clone();
            display_fields.push(field("payload_hash", "负载哈希", payload_hash.as_str()));
            let sign_request = match build_sign_request(
                action_id.as_str(),
                now.timestamp(),
                expires_at.timestamp(),
                ctx.admin_pubkey.as_str(),
                payload_text.as_str(),
                input.action_type.label(),
                display_fields,
            ) {
                Ok(v) => v,
                Err(resp) => return resp,
            };
            (payload_text, payload_hash, Some(sign_request))
        } else {
            (String::new(), request_hash.clone(), None)
        };
    {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        cleanup_admin_security_challenges(&mut store);
        store.admin_action_challenges.insert(
            action_id.clone(),
            AdminActionChallenge {
                action_id: action_id.clone(),
                action_type: input.action_type.as_str().to_string(),
                actor_pubkey: ctx.admin_pubkey.clone(),
                actor_role: ctx.role.clone(),
                actor_province: province,
                actor_city: ctx.admin_city.clone(),
                security_level: preview.security_level.clone(),
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
            },
        );
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
            security_level: preview.security_level,
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
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    cleanup_admin_security_challenges(&mut store);
    let challenge = match store
        .admin_action_challenges
        .get(input.action_id.as_str())
        .cloned()
    {
        Some(v) => v,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "admin action not found"),
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
    if let Err(resp) = update_passkey_usage(
        &mut store,
        ctx.admin_pubkey.as_str(),
        &input.passkey_assertion,
        &auth_result,
        now,
    ) {
        return resp;
    }
    if challenge.security_level == AdminSecurityLevel::Important {
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
        if let Err(resp) = verify_cold_wallet_signature(
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
    let output = if action_type.is_governance() {
        if let Err(resp) = recheck_preview(&store, &ctx, &challenge) {
            return resp;
        }
        let result = match apply_action(&mut store, &ctx, &challenge) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        CommitAdminActionOutput::Applied(result)
    } else {
        let grant_id = format!("sfid-admin-grant-{}", Uuid::new_v4());
        let grant_expires_at = now + Duration::seconds(ADMIN_SECURITY_GRANT_TTL_SECONDS);
        let grant = AdminSecurityGrant {
            grant_id: grant_id.clone(),
            action_type: action_type.as_str().to_string(),
            actor_pubkey: ctx.admin_pubkey.clone(),
            actor_role: ctx.role.clone(),
            actor_province: ctx.admin_province.clone().unwrap_or_default(),
            actor_city: ctx.admin_city.clone(),
            security_level: challenge.security_level.clone(),
            target: challenge.target.clone(),
            payload_hash: hash_json(&challenge.request_payload),
            issued_at: now,
            expires_at: grant_expires_at,
            consumed: false,
        };
        store.admin_security_grants.insert(grant_id.clone(), grant);
        CommitAdminActionOutput::Grant(AdminSecurityGrantOutput {
            grant_id,
            action_type: action_type.as_str().to_string(),
            security_level: challenge.security_level.clone(),
            target: challenge.target.clone(),
            expires_at: grant_expires_at.timestamp(),
        })
    };
    if let Some(challenge_mut) = store
        .admin_action_challenges
        .get_mut(input.action_id.as_str())
    {
        challenge_mut.consumed = true;
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
    let grant_id = headers
        .get(ADMIN_SECURITY_GRANT_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| api_error(StatusCode::FORBIDDEN, 1003, "security grant required"))?
        .to_string();
    let now = Utc::now();
    let mut store = store_write_or_500(state)?;
    cleanup_admin_security_challenges(&mut store);
    let Some(grant) = store.admin_security_grants.get_mut(grant_id.as_str()) else {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "security grant not found",
        ));
    };
    if grant.consumed || now > grant.expires_at {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "security grant expired",
        ));
    }
    if grant.action_type != action_type.as_str() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "security grant action mismatch",
        ));
    }
    if grant.security_level != action_type.security_level() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "security grant level mismatch",
        ));
    }
    if !same_admin_pubkey(grant.actor_pubkey.as_str(), ctx.admin_pubkey.as_str())
        || grant.actor_role != ctx.role
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "security grant owner mismatch",
        ));
    }
    if grant.actor_province != ctx.admin_province.clone().unwrap_or_default()
        || grant.actor_city.as_deref() != ctx.admin_city.as_deref()
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "security grant scope mismatch",
        ));
    }
    let expected_target = normalize_security_target(target);
    if grant.target != expected_target {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "security grant target mismatch",
        ));
    }
    if let Some(payload) = request_payload {
        let request_hash = hash_json(payload);
        if grant.payload_hash != request_hash {
            return Err(api_error(
                StatusCode::FORBIDDEN,
                1003,
                "security grant payload mismatch",
            ));
        }
    }
    grant.consumed = true;
    Ok(())
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

fn ensure_action_role_allowed(
    ctx: &AdminAuthContext,
    action_type: &AdminActionType,
) -> Result<(), axum::response::Response> {
    if ctx.admin_province.is_none() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        ));
    }
    if (action_type.is_governance() || action_type.is_cpms()) && ctx.role != AdminRole::ShengAdmin {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "sheng admin required",
        ));
    }
    Ok(())
}

fn preview_action(
    store: &Store,
    ctx: &AdminAuthContext,
    action_type: &AdminActionType,
    payload: &serde_json::Value,
) -> Result<ActionPreview, axum::response::Response> {
    match action_type {
        AdminActionType::CreateOperator => {
            let input: CreateOperatorInput = serde_json::from_value(payload.clone())
                .map_err(|_| api_error(StatusCode::BAD_REQUEST, 1001, "invalid create payload"))?;
            let (admin_pubkey, admin_name, city, created_by) =
                validate_create_operator(store, ctx, &input)?;
            let after = json!({
                "role": "SHI_ADMIN",
                "admin_pubkey": admin_pubkey,
                "admin_name": admin_name,
                "city": city,
                "created_by": created_by,
            });
            let after_hash = hash_json(&after);
            Ok(ActionPreview {
                before_hash: "none".to_string(),
                after_hash: after_hash.clone(),
                target: admin_pubkey.clone(),
                security_level: action_type.security_level(),
                display_fields: base_fields(
                    action_type,
                    ctx,
                    admin_pubkey.as_str(),
                    "none",
                    after_hash.as_str(),
                ),
            })
        }
        AdminActionType::UpdateOperator => {
            let input: UpdateOperatorActionPayload = serde_json::from_value(payload.clone())
                .map_err(|_| api_error(StatusCode::BAD_REQUEST, 1001, "invalid update payload"))?;
            let (before, after, target) = preview_update_operator(store, ctx, &input)?;
            let before_hash = hash_serialized(&before);
            let after_hash = hash_serialized(&after);
            Ok(ActionPreview {
                before_hash: before_hash.clone(),
                after_hash: after_hash.clone(),
                target: target.clone(),
                security_level: action_type.security_level(),
                display_fields: base_fields(
                    action_type,
                    ctx,
                    target.as_str(),
                    before_hash.as_str(),
                    after_hash.as_str(),
                ),
            })
        }
        AdminActionType::DeleteOperator => {
            let input: OperatorIdPayload = serde_json::from_value(payload.clone())
                .map_err(|_| api_error(StatusCode::BAD_REQUEST, 1001, "invalid delete payload"))?;
            let operator = require_manageable_operator(store, ctx, input.id)?;
            let before = operator_row_from_user(store, &operator);
            let after =
                json!({ "deleted": true, "id": input.id, "admin_pubkey": operator.admin_pubkey });
            let before_hash = hash_serialized(&before);
            let after_hash = hash_json(&after);
            Ok(ActionPreview {
                before_hash: before_hash.clone(),
                after_hash: after_hash.clone(),
                target: operator.admin_pubkey.clone(),
                security_level: action_type.security_level(),
                display_fields: base_fields(
                    action_type,
                    ctx,
                    operator.admin_pubkey.as_str(),
                    before_hash.as_str(),
                    after_hash.as_str(),
                ),
            })
        }
        AdminActionType::CreateShengAdmin => {
            let input: CreateShengAdminActionPayload = serde_json::from_value(payload.clone())
                .map_err(|_| {
                    api_error(StatusCode::BAD_REQUEST, 1001, "invalid sheng admin payload")
                })?;
            let (_, after, target) = preview_create_sheng_admin(store, ctx, &input)?;
            let after_hash = hash_serialized(&after);
            Ok(ActionPreview {
                before_hash: "none".to_string(),
                after_hash: after_hash.clone(),
                target: target.clone(),
                security_level: action_type.security_level(),
                display_fields: base_fields(
                    action_type,
                    ctx,
                    target.as_str(),
                    "none",
                    after_hash.as_str(),
                ),
            })
        }
        AdminActionType::UpdateShengAdmin => {
            let input: UpdateShengAdminActionPayload = serde_json::from_value(payload.clone())
                .map_err(|_| {
                    api_error(StatusCode::BAD_REQUEST, 1001, "invalid sheng admin payload")
                })?;
            let (before, after, target) = preview_update_sheng_admin(store, ctx, &input)?;
            let before_hash = hash_serialized(&before);
            let after_hash = hash_serialized(&after);
            Ok(ActionPreview {
                before_hash: before_hash.clone(),
                after_hash: after_hash.clone(),
                target: target.clone(),
                security_level: action_type.security_level(),
                display_fields: base_fields(
                    action_type,
                    ctx,
                    target.as_str(),
                    before_hash.as_str(),
                    after_hash.as_str(),
                ),
            })
        }
        AdminActionType::DeleteShengAdmin => {
            let input: ShengAdminIdPayload =
                serde_json::from_value(payload.clone()).map_err(|_| {
                    api_error(StatusCode::BAD_REQUEST, 1001, "invalid sheng admin payload")
                })?;
            let (before, after, target) = preview_delete_sheng_admin(store, ctx, &input)?;
            let before_hash = hash_serialized(&before);
            let after_hash = hash_serialized(&after);
            Ok(ActionPreview {
                before_hash: before_hash.clone(),
                after_hash: after_hash.clone(),
                target: target.clone(),
                security_level: action_type.security_level(),
                display_fields: base_fields(
                    action_type,
                    ctx,
                    target.as_str(),
                    before_hash.as_str(),
                    after_hash.as_str(),
                ),
            })
        }
        _ => preview_security_action(ctx, action_type, payload),
    }
}

fn preview_security_action(
    ctx: &AdminAuthContext,
    action_type: &AdminActionType,
    payload: &serde_json::Value,
) -> Result<ActionPreview, axum::response::Response> {
    ensure_action_role_allowed(ctx, action_type)?;
    let target = payload
        .get("target")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("sfid_number").and_then(|v| v.as_str()))
        .or_else(|| payload.get("site_sfid").and_then(|v| v.as_str()))
        .or_else(|| payload.get("challenge_id").and_then(|v| v.as_str()))
        .unwrap_or("*");
    let target = normalize_security_target(target);
    let request_hash = hash_json(payload);
    Ok(ActionPreview {
        before_hash: "security-grant".to_string(),
        after_hash: request_hash.clone(),
        target: target.clone(),
        security_level: action_type.security_level(),
        display_fields: base_fields(
            action_type,
            ctx,
            target.as_str(),
            "security-grant",
            request_hash.as_str(),
        ),
    })
}

fn base_fields(
    action_type: &AdminActionType,
    ctx: &AdminAuthContext,
    target: &str,
    before_hash: &str,
    after_hash: &str,
) -> Vec<serde_json::Value> {
    vec![
        field("action_type", "操作", action_type.label()),
        field(
            "province",
            "省份",
            ctx.admin_province.as_deref().unwrap_or_default(),
        ),
        field("actor_pubkey", "管理员", ctx.admin_pubkey.as_str()),
        field("target", "目标", target),
        field("before_hash", "变更前", before_hash),
        field("after_hash", "变更后", after_hash),
    ]
}

fn validate_create_operator(
    store: &Store,
    ctx: &AdminAuthContext,
    input: &CreateOperatorInput,
) -> Result<(String, String, String, String), axum::response::Response> {
    if input.admin_pubkey.trim().is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_pubkey is required",
        ));
    }
    let Some(admin_pubkey) = normalize_admin_pubkey(input.admin_pubkey.as_str()) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_pubkey format invalid",
        ));
    };
    let admin_name = input.admin_name.trim().to_string();
    if admin_name.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_name is required",
        ));
    }
    if admin_name.chars().count() > MAX_ADMIN_NAME_CHARS {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_name too long",
        ));
    }
    let created_by = match input.created_by.as_deref().map(str::trim) {
        None | Some("") => ctx.admin_pubkey.clone(),
        Some(raw) => {
            let Some(normalized) = normalize_admin_pubkey(raw) else {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "created_by format invalid",
                ));
            };
            if !same_admin_pubkey(normalized.as_str(), ctx.admin_pubkey.as_str()) {
                return Err(api_error(
                    StatusCode::FORBIDDEN,
                    1003,
                    "ShengAdmin can only create operators under itself",
                ));
            }
            normalized
        }
    };
    if store
        .admin_users_by_pubkey
        .keys()
        .any(|existing| same_admin_pubkey(existing.as_str(), admin_pubkey.as_str()))
    {
        return Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "operator already exists",
        ));
    }
    let city = ensure_city_in_creator_province(store, created_by.as_str(), input.city.as_str())?;
    Ok((admin_pubkey, admin_name, city, created_by))
}

fn require_manageable_operator(
    store: &Store,
    ctx: &AdminAuthContext,
    id: u64,
) -> Result<AdminUser, axum::response::Response> {
    let Some(pubkey) = find_operator_pubkey_by_id(store, id) else {
        return Err(api_error(StatusCode::NOT_FOUND, 1004, "operator not found"));
    };
    let Some(operator) = store.admin_users_by_pubkey.get(pubkey.as_str()).cloned() else {
        return Err(api_error(StatusCode::NOT_FOUND, 1004, "operator not found"));
    };
    if !can_manage_operator(
        store,
        ctx.admin_pubkey.as_str(),
        ctx.admin_province.as_deref(),
        &operator,
    ) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province operators",
        ));
    }
    Ok(operator)
}

fn preview_update_operator(
    store: &Store,
    ctx: &AdminAuthContext,
    input: &UpdateOperatorActionPayload,
) -> Result<(OperatorRow, OperatorRow, String), axum::response::Response> {
    let mut operator = require_manageable_operator(store, ctx, input.id)?;
    let before = operator_row_from_user(store, &operator);
    // 中文注释:市级管理员地址和市归属属于身份根,编辑时只允许调整姓名。
    if let Some(next_name) = input.admin_name.as_deref() {
        let name = next_name.trim();
        if name.is_empty() {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "admin_name is invalid",
            ));
        }
        if name.chars().count() > MAX_ADMIN_NAME_CHARS {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "admin_name too long",
            ));
        }
        operator.admin_name = name.to_string();
    }
    let after = operator_row_from_user(store, &operator);
    Ok((before, after, operator.admin_pubkey))
}

fn validate_sheng_admin_name(name: &str) -> Result<String, axum::response::Response> {
    let name = name.trim();
    if name.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_name is required",
        ));
    }
    if name.chars().count() > MAX_ADMIN_NAME_CHARS {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_name too long",
        ));
    }
    Ok(name.to_string())
}

fn find_sheng_admin_pubkey_by_id(store: &Store, id: u64) -> Option<String> {
    store
        .admin_users_by_pubkey
        .values()
        .find(|u| u.id == id && u.role == AdminRole::ShengAdmin)
        .map(|u| u.admin_pubkey.clone())
}

fn sheng_admin_scope(store: &Store, admin_pubkey: &str) -> Option<String> {
    store
        .sheng_admin_province_by_pubkey
        .iter()
        .find(|(pubkey, _)| same_admin_pubkey(pubkey.as_str(), admin_pubkey))
        .map(|(_, province)| province.clone())
        .or_else(|| sheng_admin_province(admin_pubkey).map(|v| v.to_string()))
}

fn require_actor_province(ctx: &AdminAuthContext) -> Result<String, axum::response::Response> {
    ctx.admin_province
        .clone()
        .ok_or_else(|| api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing"))
}

fn actor_is_initial_sheng_admin(ctx: &AdminAuthContext) -> bool {
    let Some(province) = ctx.admin_province.as_deref() else {
        return false;
    };
    sheng_admin_province(ctx.admin_pubkey.as_str())
        .map(|built_in_province| built_in_province == province)
        .unwrap_or(false)
}

fn sheng_admin_row_value(
    store: &Store,
    admin: &AdminUser,
) -> Result<serde_json::Value, axum::response::Response> {
    let province = sheng_admin_scope(store, admin.admin_pubkey.as_str())
        .ok_or_else(|| api_error(StatusCode::CONFLICT, 1005, "sheng admin province missing"))?;
    serde_json::to_value(ShengAdminRow {
        id: admin.id,
        province,
        admin_pubkey: admin.admin_pubkey.clone(),
        admin_name: admin.admin_name.clone(),
        built_in: admin.built_in,
        created_at: admin.created_at,
        updated_at: admin.updated_at,
    })
    .map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1503,
            "encode sheng admin failed",
        )
    })
}

fn validate_create_sheng_admin(
    store: &Store,
    ctx: &AdminAuthContext,
    input: &CreateShengAdminActionPayload,
) -> Result<(String, String, String), axum::response::Response> {
    let province = require_actor_province(ctx)?;
    let Some(admin_pubkey) = normalize_admin_pubkey(input.admin_pubkey.as_str()) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_pubkey format invalid",
        ));
    };
    let admin_name = validate_sheng_admin_name(input.admin_name.as_str())?;
    if store
        .admin_users_by_pubkey
        .keys()
        .any(|existing| same_admin_pubkey(existing.as_str(), admin_pubkey.as_str()))
    {
        return Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "sheng admin already exists",
        ));
    }
    Ok((admin_pubkey, admin_name, province))
}

fn preview_create_sheng_admin(
    store: &Store,
    ctx: &AdminAuthContext,
    input: &CreateShengAdminActionPayload,
) -> Result<(serde_json::Value, serde_json::Value, String), axum::response::Response> {
    let (admin_pubkey, admin_name, province) = validate_create_sheng_admin(store, ctx, input)?;
    let before = json!({ "exists": false, "admin_pubkey": admin_pubkey.clone() });
    let after = json!({
        "role": "SHENG_ADMIN",
        "province": province,
        "admin_pubkey": admin_pubkey.clone(),
        "admin_name": admin_name,
        "created_by": ctx.admin_pubkey,
    });
    Ok((before, after, admin_pubkey))
}

fn require_manageable_sheng_admin(
    store: &Store,
    ctx: &AdminAuthContext,
    id: u64,
) -> Result<(AdminUser, String), axum::response::Response> {
    let actor_province = require_actor_province(ctx)?;
    let Some(pubkey) = find_sheng_admin_pubkey_by_id(store, id) else {
        return Err(api_error(
            StatusCode::NOT_FOUND,
            1004,
            "sheng admin not found",
        ));
    };
    let Some(admin) = store.admin_users_by_pubkey.get(pubkey.as_str()).cloned() else {
        return Err(api_error(
            StatusCode::NOT_FOUND,
            1004,
            "sheng admin not found",
        ));
    };
    let target_province = sheng_admin_scope(store, admin.admin_pubkey.as_str())
        .ok_or_else(|| api_error(StatusCode::CONFLICT, 1005, "sheng admin province missing"))?;
    if target_province != actor_province {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "cannot manage other province sheng admins",
        ));
    }
    Ok((admin, target_province))
}

fn preview_update_sheng_admin(
    store: &Store,
    ctx: &AdminAuthContext,
    input: &UpdateShengAdminActionPayload,
) -> Result<(serde_json::Value, serde_json::Value, String), axum::response::Response> {
    let (mut admin, _) = require_manageable_sheng_admin(store, ctx, input.id)?;
    let before = sheng_admin_row_value(store, &admin)?;
    admin.admin_name = validate_sheng_admin_name(input.admin_name.as_str())?;
    let after = sheng_admin_row_value(store, &admin)?;
    Ok((before, after, admin.admin_pubkey))
}

fn preview_delete_sheng_admin(
    store: &Store,
    ctx: &AdminAuthContext,
    input: &ShengAdminIdPayload,
) -> Result<(serde_json::Value, serde_json::Value, String), axum::response::Response> {
    if !actor_is_initial_sheng_admin(ctx) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "initial sheng admin required",
        ));
    }
    let (admin, province) = require_manageable_sheng_admin(store, ctx, input.id)?;
    if same_admin_pubkey(admin.admin_pubkey.as_str(), ctx.admin_pubkey.as_str()) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "initial sheng admin cannot delete itself",
        ));
    }
    if admin.built_in || sheng_admin_province(admin.admin_pubkey.as_str()).is_some() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "built-in sheng admin cannot be deleted",
        ));
    }
    let before = sheng_admin_row_value(store, &admin)?;
    let after = json!({
        "deleted": true,
        "id": input.id,
        "province": province,
        "admin_pubkey": admin.admin_pubkey.clone(),
    });
    Ok((before, after, admin.admin_pubkey))
}

fn recheck_preview(
    store: &Store,
    ctx: &AdminAuthContext,
    challenge: &AdminActionChallenge,
) -> Result<(), axum::response::Response> {
    let action_type = parse_action_type(challenge.action_type.as_str())?;
    let preview = preview_action(store, ctx, &action_type, &challenge.request_payload)?;
    if preview.before_hash != challenge.before_hash {
        return Err(api_error(
            StatusCode::CONFLICT,
            1005,
            "admin action state changed",
        ));
    }
    Ok(())
}

fn parse_action_type(action_type: &str) -> Result<AdminActionType, axum::response::Response> {
    match action_type {
        "CREATE_OPERATOR" => Ok(AdminActionType::CreateOperator),
        "UPDATE_OPERATOR" => Ok(AdminActionType::UpdateOperator),
        "DELETE_OPERATOR" => Ok(AdminActionType::DeleteOperator),
        "CREATE_SHENG_ADMIN" => Ok(AdminActionType::CreateShengAdmin),
        "UPDATE_SHENG_ADMIN" => Ok(AdminActionType::UpdateShengAdmin),
        "DELETE_SHENG_ADMIN" => Ok(AdminActionType::DeleteShengAdmin),
        "INSTITUTION_CREATE" => Ok(AdminActionType::InstitutionCreate),
        "INSTITUTION_UPDATE" => Ok(AdminActionType::InstitutionUpdate),
        "INSTITUTION_CREATE_ACCOUNT" => Ok(AdminActionType::InstitutionCreateAccount),
        "INSTITUTION_DELETE_ACCOUNT" => Ok(AdminActionType::InstitutionDeleteAccount),
        "INSTITUTION_UPLOAD_DOCUMENT" => Ok(AdminActionType::InstitutionUploadDocument),
        "INSTITUTION_DELETE_DOCUMENT" => Ok(AdminActionType::InstitutionDeleteDocument),
        "PUBLIC_SECURITY_RECONCILE" => Ok(AdminActionType::PublicSecurityReconcile),
        "CITIZEN_BIND_COMMIT" => Ok(AdminActionType::CitizenBindCommit),
        "CPMS_STATUS_IMPORT_CONFIRM" => Ok(AdminActionType::CpmsStatusImportConfirm),
        "CPMS_ISSUE_INSTALL_CODE" => Ok(AdminActionType::CpmsIssueInstallCode),
        "CPMS_REVOKE_INSTALL_TOKEN" => Ok(AdminActionType::CpmsRevokeInstallToken),
        "CPMS_REISSUE_INSTALL_TOKEN" => Ok(AdminActionType::CpmsReissueInstallToken),
        "CPMS_DISABLE_KEYS" => Ok(AdminActionType::CpmsDisableKeys),
        "CPMS_ENABLE_KEYS" => Ok(AdminActionType::CpmsEnableKeys),
        "CPMS_REVOKE_KEYS" => Ok(AdminActionType::CpmsRevokeKeys),
        "CPMS_DELETE_KEYS" => Ok(AdminActionType::CpmsDeleteKeys),
        _ => Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "unknown action_type",
        )),
    }
}

fn apply_action(
    store: &mut Store,
    ctx: &AdminAuthContext,
    challenge: &AdminActionChallenge,
) -> Result<serde_json::Value, axum::response::Response> {
    let action_type = parse_action_type(challenge.action_type.as_str())?;
    match action_type {
        AdminActionType::CreateOperator => {
            let input: CreateOperatorInput =
                serde_json::from_value(challenge.request_payload.clone()).map_err(|_| {
                    api_error(StatusCode::BAD_REQUEST, 1001, "invalid create payload")
                })?;
            let (admin_pubkey, admin_name, city, created_by) =
                validate_create_operator(store, ctx, &input)?;
            let now = Utc::now();
            let row = AdminUser {
                id: allocate_next_admin_user_id(store),
                admin_pubkey: admin_pubkey.clone(),
                admin_name,
                role: AdminRole::ShiAdmin,
                built_in: false,
                created_by,
                created_at: now,
                updated_at: Some(now),
                city,
            };
            store
                .admin_users_by_pubkey
                .insert(admin_pubkey, row.clone());
            append_audit_log(
                store,
                "OPERATOR_CREATE",
                &ctx.admin_pubkey,
                Some(row.admin_pubkey.clone()),
                None,
                "SUCCESS",
                format!("operator_id={} created_by={}", row.id, row.created_by),
            );
            serde_json::to_value(operator_row_from_user(store, &row)).map_err(|_| {
                api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    1503,
                    "encode operator failed",
                )
            })
        }
        AdminActionType::UpdateOperator => {
            let input: UpdateOperatorActionPayload =
                serde_json::from_value(challenge.request_payload.clone()).map_err(|_| {
                    api_error(StatusCode::BAD_REQUEST, 1001, "invalid update payload")
                })?;
            apply_update_operator(store, ctx, &input)
        }
        AdminActionType::DeleteOperator => {
            let input: OperatorIdPayload =
                serde_json::from_value(challenge.request_payload.clone()).map_err(|_| {
                    api_error(StatusCode::BAD_REQUEST, 1001, "invalid delete payload")
                })?;
            let operator = require_manageable_operator(store, ctx, input.id)?;
            let pubkey = operator.admin_pubkey.clone();
            store.admin_users_by_pubkey.remove(pubkey.as_str());
            store.admin_sessions.retain(|_, session| {
                !same_admin_pubkey(session.admin_pubkey.as_str(), pubkey.as_str())
            });
            store.admin_passkeys_by_credential_id.retain(|_, record| {
                !same_admin_pubkey(record.admin_pubkey.as_str(), pubkey.as_str())
            });
            append_audit_log(
                store,
                "OPERATOR_DELETE",
                &ctx.admin_pubkey,
                Some(pubkey.clone()),
                None,
                "SUCCESS",
                format!(
                    "operator_id={} created_by={}",
                    operator.id, operator.created_by
                ),
            );
            Ok(json!({ "deleted": true, "admin_pubkey": pubkey }))
        }
        AdminActionType::CreateShengAdmin => {
            let input: CreateShengAdminActionPayload =
                serde_json::from_value(challenge.request_payload.clone()).map_err(|_| {
                    api_error(StatusCode::BAD_REQUEST, 1001, "invalid sheng admin payload")
                })?;
            apply_create_sheng_admin(store, ctx, &input)
        }
        AdminActionType::UpdateShengAdmin => {
            let input: UpdateShengAdminActionPayload =
                serde_json::from_value(challenge.request_payload.clone()).map_err(|_| {
                    api_error(StatusCode::BAD_REQUEST, 1001, "invalid sheng admin payload")
                })?;
            apply_update_sheng_admin(store, ctx, &input)
        }
        AdminActionType::DeleteShengAdmin => {
            let input: ShengAdminIdPayload =
                serde_json::from_value(challenge.request_payload.clone()).map_err(|_| {
                    api_error(StatusCode::BAD_REQUEST, 1001, "invalid sheng admin payload")
                })?;
            apply_delete_sheng_admin(store, ctx, &input)
        }
        _ => Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "business action cannot be applied by admin governance endpoint",
        )),
    }
}

fn apply_update_operator(
    store: &mut Store,
    ctx: &AdminAuthContext,
    input: &UpdateOperatorActionPayload,
) -> Result<serde_json::Value, axum::response::Response> {
    let operator = require_manageable_operator(store, ctx, input.id)?;
    let (_, after, _) = preview_update_operator(store, ctx, input)?;
    let mut next = operator;
    next.admin_name = after.admin_name;
    next.updated_at = Some(Utc::now());
    let current_pubkey = next.admin_pubkey.clone();
    store
        .admin_users_by_pubkey
        .insert(next.admin_pubkey.clone(), next.clone());
    append_audit_log(
        store,
        "OPERATOR_UPDATE",
        &ctx.admin_pubkey,
        Some(next.admin_pubkey.clone()),
        None,
        "SUCCESS",
        format!(
            "operator_id={} pubkey={} name_updated=true",
            next.id, current_pubkey
        ),
    );
    serde_json::to_value(operator_row_from_user(store, &next)).map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1503,
            "encode operator failed",
        )
    })
}

fn apply_create_sheng_admin(
    store: &mut Store,
    ctx: &AdminAuthContext,
    input: &CreateShengAdminActionPayload,
) -> Result<serde_json::Value, axum::response::Response> {
    let (admin_pubkey, admin_name, province) = validate_create_sheng_admin(store, ctx, input)?;
    let now = Utc::now();
    let row = AdminUser {
        id: allocate_next_admin_user_id(store),
        admin_pubkey: admin_pubkey.clone(),
        admin_name,
        role: AdminRole::ShengAdmin,
        built_in: false,
        created_by: ctx.admin_pubkey.clone(),
        created_at: now,
        updated_at: Some(now),
        city: String::new(),
    };
    store
        .admin_users_by_pubkey
        .insert(admin_pubkey.clone(), row.clone());
    store
        .sheng_admin_province_by_pubkey
        .insert(admin_pubkey.clone(), province.clone());
    append_audit_log(
        store,
        "SHENG_ADMIN_CREATE",
        &ctx.admin_pubkey,
        Some(admin_pubkey.clone()),
        None,
        "SUCCESS",
        format!("sheng_admin_id={} province={province}", row.id),
    );
    sheng_admin_row_value(store, &row)
}

fn apply_update_sheng_admin(
    store: &mut Store,
    ctx: &AdminAuthContext,
    input: &UpdateShengAdminActionPayload,
) -> Result<serde_json::Value, axum::response::Response> {
    let (admin, _) = require_manageable_sheng_admin(store, ctx, input.id)?;
    let admin_name = validate_sheng_admin_name(input.admin_name.as_str())?;
    {
        let Some(row) = store
            .admin_users_by_pubkey
            .get_mut(admin.admin_pubkey.as_str())
        else {
            return Err(api_error(
                StatusCode::NOT_FOUND,
                1004,
                "sheng admin not found",
            ));
        };
        row.admin_name = admin_name;
        row.updated_at = Some(Utc::now());
    }
    let updated = store
        .admin_users_by_pubkey
        .get(admin.admin_pubkey.as_str())
        .cloned()
        .ok_or_else(|| api_error(StatusCode::NOT_FOUND, 1004, "sheng admin not found"))?;
    append_audit_log(
        store,
        "SHENG_ADMIN_UPDATE",
        &ctx.admin_pubkey,
        Some(updated.admin_pubkey.clone()),
        None,
        "SUCCESS",
        format!("sheng_admin_id={} name_updated=true", updated.id),
    );
    sheng_admin_row_value(store, &updated)
}

fn apply_delete_sheng_admin(
    store: &mut Store,
    ctx: &AdminAuthContext,
    input: &ShengAdminIdPayload,
) -> Result<serde_json::Value, axum::response::Response> {
    let (_, _, _) = preview_delete_sheng_admin(store, ctx, input)?;
    let Some(pubkey) = find_sheng_admin_pubkey_by_id(store, input.id) else {
        return Err(api_error(
            StatusCode::NOT_FOUND,
            1004,
            "sheng admin not found",
        ));
    };
    let removed = store
        .admin_users_by_pubkey
        .remove(pubkey.as_str())
        .ok_or_else(|| api_error(StatusCode::NOT_FOUND, 1004, "sheng admin not found"))?;
    store
        .sheng_admin_province_by_pubkey
        .retain(|candidate, _| !same_admin_pubkey(candidate.as_str(), pubkey.as_str()));
    store
        .admin_sessions
        .retain(|_, session| !same_admin_pubkey(session.admin_pubkey.as_str(), pubkey.as_str()));
    store
        .admin_passkeys_by_credential_id
        .retain(|_, record| !same_admin_pubkey(record.admin_pubkey.as_str(), pubkey.as_str()));
    store
        .admin_passkey_registration_challenges
        .retain(|_, challenge| {
            !same_admin_pubkey(challenge.admin_pubkey.as_str(), pubkey.as_str())
        });
    store.admin_action_challenges.retain(|_, challenge| {
        !same_admin_pubkey(challenge.actor_pubkey.as_str(), pubkey.as_str())
    });
    store
        .admin_security_grants
        .retain(|_, grant| !same_admin_pubkey(grant.actor_pubkey.as_str(), pubkey.as_str()));
    // 中文注释:删除新增省管理员时,其名下市管理员交回当前初始省管理员,避免省域归属断链。
    for operator in store.admin_users_by_pubkey.values_mut() {
        if operator.role == AdminRole::ShiAdmin
            && same_admin_pubkey(operator.created_by.as_str(), pubkey.as_str())
        {
            operator.created_by = ctx.admin_pubkey.clone();
            operator.updated_at = Some(Utc::now());
        }
    }
    append_audit_log(
        store,
        "SHENG_ADMIN_DELETE",
        &ctx.admin_pubkey,
        Some(pubkey.clone()),
        None,
        "SUCCESS",
        format!("sheng_admin_id={} reassigned_city_admins=true", removed.id),
    );
    Ok(json!({ "deleted": true, "admin_pubkey": pubkey }))
}
