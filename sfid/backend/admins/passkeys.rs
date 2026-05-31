//! 管理员 Passkey 注册与 WebAuthn 工具。
//!
//! 中文注释:本模块只负责浏览器 Passkey 凭据的注册、验证辅助和短期安全挑战清理;
//! 管理员治理动作的业务落库仍归 `admins/actions.rs`。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use webauthn_rs::prelude::{
    AuthenticationResult, CreationChallengeResponse, Passkey, PublicKeyCredential,
    RegisterPublicKeyCredential, Url, Webauthn, WebauthnBuilder,
};

use crate::citizens::binding::pubkey_hex_to_ss58;
use crate::crypto::pubkey::same_admin_pubkey;
use crate::models::{
    AdminPasskeyCredential, AdminPasskeyRegistrationChallenge, AdminPasskeyStatus,
};
use crate::*;

pub(crate) const ADMIN_ACTION_TTL_SECONDS: i64 = 300;

const PROD_PASSKEY_RP_ID: &str = "sfid.crcfrcn.com";
const PROD_PASSKEY_ORIGIN: &str = "https://sfid.crcfrcn.com";
const DEV_PASSKEY_RP_ID: &str = "localhost";
const DEV_PASSKEY_ORIGIN: &str = "http://localhost:5179";
const ENV_PASSKEY_RP_ID: &str = "SFID_PASSKEY_RP_ID";
const ENV_PASSKEY_ORIGIN: &str = "SFID_PASSKEY_ORIGIN";
const ENV_PASSKEY_ALLOWED_ORIGINS: &str = "SFID_PASSKEY_ALLOWED_ORIGINS";
const ADMIN_SIGN_ACTION: &str = "sfid_admin_action";

#[derive(Debug, Clone)]
struct PasskeyWebauthnConfig {
    rp_id: String,
    origins: Vec<String>,
    production: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PasskeyStartInput {
    #[serde(default)]
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PasskeyAttestInput {
    registration_id: String,
    credential: RegisterPublicKeyCredential,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PasskeyCompleteInput {
    registration_id: String,
    signer_pubkey: String,
    signature: String,
    payload_hash: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct PasskeyStartOutput {
    registration_id: String,
    public_key_options: CreationChallengeResponse,
    expires_at: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct PasskeyAttestOutput {
    registration_id: String,
    request_id: String,
    sign_request: String,
    payload_hash: String,
    expires_at: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct PasskeyCompleteOutput {
    credential_id: String,
    passkey_count: usize,
}

#[derive(Serialize)]
pub(crate) struct AdminSignedPayload<'a> {
    pub(crate) domain: &'static str,
    pub(crate) qr_proto: &'static str,
    pub(crate) action_id: &'a str,
    pub(crate) action_type: &'a str,
    pub(crate) actor_pubkey: &'a str,
    pub(crate) actor_province: &'a str,
    pub(crate) target: &'a str,
    pub(crate) request_hash: &'a str,
    pub(crate) before_hash: &'a str,
    pub(crate) after_hash: &'a str,
    pub(crate) expires_at: i64,
}

pub(crate) async fn start_passkey_registration(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PasskeyStartInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let webauthn = match webauthn() {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let label = input
        .label
        .unwrap_or_else(|| "管理员 Passkey".to_string())
        .trim()
        .chars()
        .take(80)
        .collect::<String>();
    let label = if label.is_empty() {
        "管理员 Passkey".to_string()
    } else {
        label
    };
    let now = Utc::now();
    let expires_at = now + Duration::seconds(ADMIN_ACTION_TTL_SECONDS);
    let registration_id = format!("admin-passkey-{}", Uuid::new_v4());
    let existing = {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        active_passkey_credentials(&store, ctx.admin_pubkey.as_str())
            .into_iter()
            .map(|record| record.passkey.cred_id().clone())
            .collect::<Vec<_>>()
    };
    let (public_key_options, webauthn_state) = match webauthn.start_passkey_registration(
        user_uuid_for_pubkey(ctx.admin_pubkey.as_str()),
        ctx.admin_pubkey.as_str(),
        ctx.admin_name.as_str(),
        Some(existing),
    ) {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(error = %err, "start passkey registration failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1503,
                "passkey start failed",
            );
        }
    };
    {
        let mut store = match store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        cleanup_admin_security_challenges(&mut store);
        store.admin_passkey_registration_challenges.insert(
            registration_id.clone(),
            AdminPasskeyRegistrationChallenge {
                registration_id: registration_id.clone(),
                admin_pubkey: ctx.admin_pubkey.clone(),
                admin_name: ctx.admin_name.clone(),
                label,
                webauthn_state,
                pending_passkey: None,
                credential_id: None,
                payload_text: None,
                payload_hash: None,
                issued_at: now,
                expires_at,
                consumed: false,
            },
        );
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PasskeyStartOutput {
            registration_id,
            public_key_options,
            expires_at: expires_at.timestamp(),
        },
    })
    .into_response()
}

pub(crate) async fn attest_passkey_registration(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PasskeyAttestInput>,
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
        .admin_passkey_registration_challenges
        .get(input.registration_id.as_str())
        .cloned()
    {
        Some(v) => v,
        None => {
            return api_error(
                StatusCode::NOT_FOUND,
                1004,
                "passkey registration not found",
            )
        }
    };
    if challenge.consumed || now > challenge.expires_at {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "passkey registration expired",
        );
    }
    if !same_admin_pubkey(challenge.admin_pubkey.as_str(), ctx.admin_pubkey.as_str()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "passkey registration owner mismatch",
        );
    }
    let passkey =
        match webauthn.finish_passkey_registration(&input.credential, &challenge.webauthn_state) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "finish passkey registration failed");
                return api_error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    2004,
                    "passkey attest failed",
                );
            }
        };
    let credential_id = credential_id_hex(&passkey);
    if store
        .admin_passkeys_by_credential_id
        .contains_key(credential_id.as_str())
    {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "passkey credential already exists",
        );
    }
    let request_hash = hash_json(&json!({
        "credential_id": credential_id,
        "label": challenge.label,
    }));
    let province = ctx.admin_province.clone().unwrap_or_default();
    let payload_text = signed_payload_text(AdminSignedPayload {
        domain: "sfid_admin_governance",
        qr_proto: crate::qr::WUMIN_QR_V1,
        action_id: challenge.registration_id.as_str(),
        action_type: "PASSKEY_REGISTER",
        actor_pubkey: ctx.admin_pubkey.as_str(),
        actor_province: province.as_str(),
        target: credential_id.as_str(),
        request_hash: request_hash.as_str(),
        before_hash: "none",
        after_hash: request_hash.as_str(),
        expires_at: challenge.expires_at.timestamp(),
    });
    let payload_hash = payload_hash_for_text(payload_text.as_str());
    let sign_request = match build_sign_request(
        challenge.registration_id.as_str(),
        now.timestamp(),
        challenge.expires_at.timestamp(),
        ctx.admin_pubkey.as_str(),
        payload_text.as_str(),
        "绑定管理员 Passkey",
        vec![
            field("action_type", "操作", "绑定 Passkey"),
            field("province", "省份", province.as_str()),
            field("actor_pubkey", "管理员", ctx.admin_pubkey.as_str()),
            field("target", "凭据", credential_id.as_str()),
            field("before_hash", "变更前", "none"),
            field("after_hash", "变更后", request_hash.as_str()),
            field("payload_hash", "负载哈希", payload_hash.as_str()),
        ],
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if let Some(challenge_mut) = store
        .admin_passkey_registration_challenges
        .get_mut(input.registration_id.as_str())
    {
        challenge_mut.pending_passkey = Some(passkey);
        challenge_mut.credential_id = Some(credential_id.clone());
        challenge_mut.payload_text = Some(payload_text);
        challenge_mut.payload_hash = Some(payload_hash.clone());
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PasskeyAttestOutput {
            registration_id: input.registration_id,
            request_id: credential_id_to_request_id(
                challenge.credential_id.as_deref(),
                challenge.registration_id.as_str(),
            ),
            sign_request,
            payload_hash,
            expires_at: challenge.expires_at.timestamp(),
        },
    })
    .into_response()
}

pub(crate) async fn complete_passkey_registration(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PasskeyCompleteInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
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
        .admin_passkey_registration_challenges
        .get(input.registration_id.as_str())
        .cloned()
    {
        Some(v) => v,
        None => {
            return api_error(
                StatusCode::NOT_FOUND,
                1004,
                "passkey registration not found",
            )
        }
    };
    if challenge.consumed || now > challenge.expires_at {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "passkey registration expired",
        );
    }
    if !same_admin_pubkey(challenge.admin_pubkey.as_str(), ctx.admin_pubkey.as_str()) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "passkey registration owner mismatch",
        );
    }
    let Some(passkey) = challenge.pending_passkey.clone() else {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "passkey attestation required first",
        );
    };
    let Some(credential_id) = challenge.credential_id.clone() else {
        return api_error(StatusCode::CONFLICT, 1005, "passkey credential missing");
    };
    let payload_text = challenge.payload_text.clone().unwrap_or_default();
    let payload_hash = challenge.payload_hash.clone().unwrap_or_default();
    if let Err(resp) = verify_cold_wallet_signature(
        ctx.admin_pubkey.as_str(),
        input.signer_pubkey.as_str(),
        input.signature.as_str(),
        input.payload_hash.as_str(),
        payload_hash.as_str(),
        payload_text.as_str(),
    ) {
        return resp;
    }
    // 中文注释:更新密钥采用替换语义,同一管理员只保留一个有效 Passkey。
    store.admin_passkeys_by_credential_id.retain(|_, record| {
        !same_admin_pubkey(record.admin_pubkey.as_str(), ctx.admin_pubkey.as_str())
    });
    store.admin_passkeys_by_credential_id.insert(
        credential_id.clone(),
        AdminPasskeyCredential {
            credential_id: credential_id.clone(),
            admin_pubkey: ctx.admin_pubkey.clone(),
            label: challenge.label,
            passkey,
            status: AdminPasskeyStatus::Active,
            created_at: now,
            last_used_at: None,
        },
    );
    if let Some(challenge_mut) = store
        .admin_passkey_registration_challenges
        .get_mut(input.registration_id.as_str())
    {
        challenge_mut.consumed = true;
    }
    append_audit_log(
        &mut store,
        "ADMIN_PASSKEY_REGISTER",
        &ctx.admin_pubkey,
        Some(ctx.admin_pubkey.clone()),
        None,
        "SUCCESS",
        format!("credential_id={credential_id}"),
    );
    let count = active_passkey_credentials(&store, ctx.admin_pubkey.as_str()).len();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PasskeyCompleteOutput {
            credential_id,
            passkey_count: count,
        },
    })
    .into_response()
}

pub(crate) fn webauthn() -> Result<Webauthn, axum::response::Response> {
    let config = passkey_webauthn_config().map_err(|err| {
        tracing::error!(error = %err, "passkey configuration invalid");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1503,
            "passkey configuration invalid",
        )
    })?;
    build_webauthn(&config).map_err(|err| {
        tracing::error!(error = %err, "webauthn build failed");
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1503,
            "passkey configuration invalid",
        )
    })
}

pub(crate) fn validate_passkey_configuration() -> Result<(), String> {
    let config = passkey_webauthn_config()?;
    build_webauthn(&config).map(|_| ())
}

fn passkey_webauthn_config() -> Result<PasskeyWebauthnConfig, String> {
    let production = is_production_env();
    let rp_id = optional_env(ENV_PASSKEY_RP_ID).unwrap_or_else(|| {
        if production {
            PROD_PASSKEY_RP_ID.to_string()
        } else {
            DEV_PASSKEY_RP_ID.to_string()
        }
    });
    let default_origin = if production {
        PROD_PASSKEY_ORIGIN
    } else {
        DEV_PASSKEY_ORIGIN
    };
    let origins = configured_passkey_origins(default_origin);
    let config = PasskeyWebauthnConfig {
        rp_id,
        origins,
        production,
    };
    validate_passkey_config(&config)?;
    Ok(config)
}

fn configured_passkey_origins(default_origin: &str) -> Vec<String> {
    let configured = optional_env(ENV_PASSKEY_ORIGIN)
        .into_iter()
        .chain(
            optional_env(ENV_PASSKEY_ALLOWED_ORIGINS)
                .into_iter()
                .flat_map(|raw| {
                    raw.split(',')
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                }),
        )
        .collect::<Vec<_>>();
    let origins = if configured.is_empty() {
        vec![default_origin.to_string()]
    } else {
        configured
    };
    origins.into_iter().fold(Vec::new(), |mut acc, origin| {
        if !acc.iter().any(|existing| existing == &origin) {
            acc.push(origin);
        }
        acc
    })
}

fn validate_passkey_config(config: &PasskeyWebauthnConfig) -> Result<(), String> {
    if config.rp_id.trim().is_empty() {
        return Err(format!("{ENV_PASSKEY_RP_ID} must not be empty"));
    }
    if config.origins.is_empty() {
        return Err("passkey origins must not be empty".to_string());
    }
    for origin in &config.origins {
        let parsed = Url::parse(origin).map_err(|_| format!("passkey origin invalid: {origin}"))?;
        match parsed.scheme() {
            "http" | "https" => {}
            other => return Err(format!("passkey origin scheme unsupported: {other}")),
        }
        if config.production && parsed.scheme() != "https" {
            return Err("production passkey origin must use https".to_string());
        }
    }
    if config.production {
        if config.rp_id != PROD_PASSKEY_RP_ID {
            return Err(format!(
                "production {ENV_PASSKEY_RP_ID} must be {PROD_PASSKEY_RP_ID}"
            ));
        }
        if config.origins.len() != 1 || config.origins[0] != PROD_PASSKEY_ORIGIN {
            return Err(format!(
                "production passkey origin must be exactly {PROD_PASSKEY_ORIGIN}"
            ));
        }
    }
    Ok(())
}

fn build_webauthn(config: &PasskeyWebauthnConfig) -> Result<Webauthn, String> {
    let origins = config
        .origins
        .iter()
        .map(|origin| Url::parse(origin).map_err(|_| format!("passkey origin invalid: {origin}")))
        .collect::<Result<Vec<_>, _>>()?;
    let Some(primary_origin) = origins.first() else {
        return Err("passkey origins must not be empty".to_string());
    };
    let mut builder = WebauthnBuilder::new(config.rp_id.as_str(), primary_origin)
        .map_err(|err| format!("webauthn builder failed: {err}"))?;
    for origin in origins.iter().skip(1) {
        builder = builder.append_allowed_origin(origin);
    }
    builder
        .rp_name("SFID")
        .build()
        .map_err(|err| format!("webauthn build failed: {err}"))
}

fn is_production_env() -> bool {
    optional_env("SFID_ENV")
        .or_else(|| optional_env("ENV"))
        .map(|value| value.eq_ignore_ascii_case("prod") || value.eq_ignore_ascii_case("production"))
        .unwrap_or(false)
}

fn optional_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn active_passkeys(store: &Store, admin_pubkey: &str) -> Vec<Passkey> {
    active_passkey_credentials(store, admin_pubkey)
        .into_iter()
        .map(|record| record.passkey.clone())
        .collect()
}

fn active_passkey_credentials<'a>(
    store: &'a Store,
    admin_pubkey: &str,
) -> Vec<&'a AdminPasskeyCredential> {
    store
        .admin_passkeys_by_credential_id
        .values()
        .filter(|record| record.status == AdminPasskeyStatus::Active)
        .filter(|record| same_admin_pubkey(record.admin_pubkey.as_str(), admin_pubkey))
        .collect()
}

fn user_uuid_for_pubkey(pubkey: &str) -> Uuid {
    let digest = Sha256::digest(pubkey.as_bytes());
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(bytes)
}

fn credential_id_hex(passkey: &Passkey) -> String {
    format!("0x{}", hex::encode(passkey.cred_id().as_ref()))
}

fn credential_id_to_request_id(_credential_id: Option<&str>, registration_id: &str) -> String {
    registration_id.to_string()
}

pub(crate) fn cleanup_admin_security_challenges(store: &mut Store) {
    let now = Utc::now();
    store
        .admin_passkey_registration_challenges
        .retain(|_, c| !c.consumed && c.expires_at > now);
    store
        .admin_action_challenges
        .retain(|_, c| !c.consumed && c.expires_at > now);
    store
        .admin_security_grants
        .retain(|_, c| !c.consumed && c.expires_at > now);
}

pub(crate) fn signed_payload_text(payload: AdminSignedPayload<'_>) -> String {
    serde_json::to_string(&payload).unwrap_or_default()
}

pub(crate) fn payload_hash_for_text(text: &str) -> String {
    format!("0x{}", hex::encode(Sha256::digest(text.as_bytes())))
}

pub(crate) fn hash_json(value: &serde_json::Value) -> String {
    let encoded = serde_json::to_vec(value).unwrap_or_default();
    format!("0x{}", hex::encode(Sha256::digest(&encoded)))
}

pub(crate) fn field(key: &str, label: &str, value: &str) -> serde_json::Value {
    json!({ "key": key, "label": label, "value": value })
}

pub(crate) fn build_sign_request(
    request_id: &str,
    issued_at: i64,
    expires_at: i64,
    actor_pubkey: &str,
    payload_text: &str,
    summary: &str,
    fields: Vec<serde_json::Value>,
) -> Result<String, axum::response::Response> {
    let Some(address) = pubkey_hex_to_ss58(actor_pubkey) else {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "actor pubkey cannot be encoded as SS58",
        ));
    };
    let sign_request = json!({
        "proto": crate::qr::WUMIN_QR_V1,
        "kind": "sign_request",
        "id": request_id,
        "issued_at": issued_at,
        "expires_at": expires_at,
        "body": {
            "address": address,
            "pubkey": actor_pubkey,
            "sig_alg": "sr25519",
            "payload_hex": format!("0x{}", hex::encode(payload_text.as_bytes())),
            "display": {
                "action": ADMIN_SIGN_ACTION,
                "summary": summary,
                "fields": fields,
            }
        }
    });
    serde_json::to_string(&sign_request).map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1503,
            "encode sign request failed",
        )
    })
}

pub(crate) fn verify_cold_wallet_signature(
    expected_actor_pubkey: &str,
    signer_pubkey: &str,
    signature: &str,
    submitted_payload_hash: &str,
    expected_payload_hash: &str,
    payload_text: &str,
) -> Result<(), axum::response::Response> {
    if !same_admin_pubkey(expected_actor_pubkey, signer_pubkey) {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "signer pubkey mismatch",
        ));
    }
    if submitted_payload_hash.trim().to_lowercase() != expected_payload_hash {
        return Err(api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "payload hash mismatch",
        ));
    }
    if !crate::login::verify_admin_signature(signer_pubkey, payload_text, signature) {
        return Err(api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "signature verify failed",
        ));
    }
    Ok(())
}

pub(crate) fn update_passkey_usage(
    store: &mut Store,
    admin_pubkey: &str,
    assertion: &PublicKeyCredential,
    auth_result: &AuthenticationResult,
    now: DateTime<Utc>,
) -> Result<(), axum::response::Response> {
    let credential_id = format!("0x{}", hex::encode(assertion.raw_id.as_ref()));
    let Some(record) = store
        .admin_passkeys_by_credential_id
        .get_mut(credential_id.as_str())
    else {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "passkey credential not registered",
        ));
    };
    if record.status != AdminPasskeyStatus::Active
        || !same_admin_pubkey(record.admin_pubkey.as_str(), admin_pubkey)
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "passkey owner mismatch",
        ));
    }
    let _ = record.passkey.update_credential(auth_result);
    record.last_used_at = Some(now);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_localhost_passkey_config_builds() {
        let config = PasskeyWebauthnConfig {
            rp_id: DEV_PASSKEY_RP_ID.to_string(),
            origins: vec![DEV_PASSKEY_ORIGIN.to_string()],
            production: false,
        };

        validate_passkey_config(&config).expect("dev localhost passkey config should be valid");
        build_webauthn(&config).expect("dev localhost passkey webauthn should build");
    }

    #[test]
    fn production_passkey_config_rejects_localhost() {
        let config = PasskeyWebauthnConfig {
            rp_id: DEV_PASSKEY_RP_ID.to_string(),
            origins: vec![DEV_PASSKEY_ORIGIN.to_string()],
            production: true,
        };

        assert!(validate_passkey_config(&config).is_err());
    }

    #[test]
    fn production_passkey_config_accepts_official_domain() {
        let config = PasskeyWebauthnConfig {
            rp_id: PROD_PASSKEY_RP_ID.to_string(),
            origins: vec![PROD_PASSKEY_ORIGIN.to_string()],
            production: true,
        };

        validate_passkey_config(&config).expect("production passkey config should be valid");
        build_webauthn(&config).expect("production passkey webauthn should build");
    }
}
