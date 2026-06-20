//! 管理员 Passkey 注册与 WebAuthn 工具。
//!
//! 中文注释:Passkey 凭据、注册挑战和一次性安全动作均落到 admins 结构化表。
//! 本模块只保留 WebAuthn、公民钱包签名校验和哈希工具,不再维护进程内状态。

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

use crate::admins::repo;
use crate::admins::security_model::{
    AdminPasskeyCredential, AdminPasskeyRegistrationChallenge, AdminPasskeyStatus,
};
use crate::core::qr::{build_sign_request, display_account, display_field as field};
use crate::crypto::pubkey::same_admin_pubkey;
use crate::*;

pub(crate) const ADMIN_ACTION_TTL_SECONDS: i64 = 300;

const PROD_PASSKEY_RP_ID: &str = "sfid.crcfrcn.com";
const PROD_PASSKEY_ORIGIN: &str = "https://sfid.crcfrcn.com";
const DEV_PASSKEY_RP_ID: &str = "localhost";
const DEV_PASSKEY_ORIGIN: &str = "http://localhost:5179";
const ENV_PASSKEY_RP_ID: &str = "SFID_PASSKEY_RP_ID";
const ENV_PASSKEY_ORIGIN: &str = "SFID_PASSKEY_ORIGIN";
const ENV_PASSKEY_ALLOWED_ORIGINS: &str = "SFID_PASSKEY_ALLOWED_ORIGINS";

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
pub(crate) struct PasskeyConfirmInput {
    registration_id: String,
    signer_pubkey: String,
    signature: String,
    payload_hash: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct PasskeyStartOutput {
    registration_id: String,
    request_id: String,
    sign_request: String,
    payload_hash: String,
    expires_at: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct PasskeyConfirmOutput {
    registration_id: String,
    public_key_options: CreationChallengeResponse,
    expires_at: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PasskeyCompleteInput {
    registration_id: String,
    credential: RegisterPublicKeyCredential,
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
    pub(crate) actor_province_name: &'a str,
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
    let request_hash = hash_json(&json!({
        "admin_pubkey": ctx.admin_pubkey.as_str(),
        "label": label.as_str(),
        "registration_id": registration_id.as_str(),
    }));
    let province = ctx.admin_province.clone().unwrap_or_default();
    let payload_text = signed_payload_text(AdminSignedPayload {
        domain: "sfid_admin_governance",
        qr_proto: crate::core::qr::WUMIN_QR_V1,
        action_id: registration_id.as_str(),
        action_type: "PASSKEY_REGISTER",
        actor_pubkey: ctx.admin_pubkey.as_str(),
        actor_province_name: province.as_str(),
        target: ctx.admin_pubkey.as_str(),
        request_hash: request_hash.as_str(),
        before_hash: "none",
        after_hash: request_hash.as_str(),
        expires_at: expires_at.timestamp(),
    });
    let payload_hash = payload_hash_for_text(payload_text.as_str());
    let sign_request = match build_sign_request(
        registration_id.as_str(),
        now.timestamp(),
        expires_at.timestamp(),
        ctx.admin_pubkey.as_str(),
        payload_text.as_str(),
        "更新管理员 Passkey",
        vec![
            field("action_type", "操作", "更新 Passkey"),
            field("province", "省份", province.as_str()),
            field(
                "actor_pubkey",
                "管理员",
                display_account(ctx.admin_pubkey.as_str()).as_str(),
            ),
            field(
                "target",
                "目标账户",
                display_account(ctx.admin_pubkey.as_str()).as_str(),
            ),
        ],
    ) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    if let Err(err) = repo::insert_passkey_challenge(
        &state.db,
        &AdminPasskeyRegistrationChallenge {
            registration_id: registration_id.clone(),
            admin_pubkey: ctx.admin_pubkey.clone(),
            admin_name: ctx.admin_name.clone(),
            label,
            webauthn_state: None,
            payload_text,
            payload_hash: payload_hash.clone(),
            citizen_wallet_confirmed: false,
            issued_at: now,
            expires_at,
            consumed: false,
        },
    ) {
        let message = format!("insert passkey challenge failed: {err}");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PasskeyStartOutput {
            registration_id: registration_id.clone(),
            request_id: registration_id,
            sign_request,
            payload_hash,
            expires_at: expires_at.timestamp(),
        },
    })
    .into_response()
}

pub(crate) async fn confirm_passkey_registration(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PasskeyConfirmInput>,
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
    let registration_id = input.registration_id.trim().to_string();
    let result = state.db.with_client(move |conn| {
        repo::cleanup_security_state_conn(conn, now)?;
        let Some(mut challenge) = repo::get_passkey_challenge_conn(conn, registration_id.as_str())?
        else {
            return Err("http:not_found:passkey registration not found".to_string());
        };
        if challenge.consumed || now > challenge.expires_at {
            return Err("http:unprocessable:passkey registration expired".to_string());
        }
        if !same_admin_pubkey(challenge.admin_pubkey.as_str(), ctx.admin_pubkey.as_str()) {
            return Err("http:forbidden:passkey registration owner mismatch".to_string());
        }
        verify_citizen_wallet_signature(
            ctx.admin_pubkey.as_str(),
            input.signer_pubkey.as_str(),
            input.signature.as_str(),
            input.payload_hash.as_str(),
            challenge.payload_hash.as_str(),
            challenge.payload_text.as_str(),
        )
        .map_err(|_| "http:unprocessable:signature verify failed".to_string())?;
        let existing = repo::active_passkey_credentials_conn(conn, ctx.admin_pubkey.as_str())?
            .into_iter()
            .map(|record| record.passkey.cred_id().clone())
            .collect::<Vec<_>>();
        let (public_key_options, webauthn_state) = webauthn
            .start_passkey_registration(
                user_uuid_for_pubkey(ctx.admin_pubkey.as_str()),
                ctx.admin_pubkey.as_str(),
                ctx.admin_name.as_str(),
                Some(existing),
            )
            .map_err(|err| format!("start passkey registration failed: {err}"))?;
        challenge.webauthn_state = Some(webauthn_state);
        challenge.citizen_wallet_confirmed = true;
        repo::upsert_passkey_challenge_conn(conn, &challenge)?;
        Ok((public_key_options, challenge.expires_at))
    });

    let (public_key_options, expires_at) = match result {
        Ok(v) => v,
        Err(err) if err == "http:not_found:passkey registration not found" => {
            return api_error(
                StatusCode::NOT_FOUND,
                1004,
                "passkey registration not found",
            )
        }
        Err(err) if err == "http:unprocessable:passkey registration expired" => {
            return api_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                2004,
                "passkey registration expired",
            )
        }
        Err(err) if err == "http:forbidden:passkey registration owner mismatch" => {
            return api_error(
                StatusCode::FORBIDDEN,
                1003,
                "passkey registration owner mismatch",
            )
        }
        Err(err) if err == "http:unprocessable:signature verify failed" => {
            return api_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                2004,
                "signature verify failed",
            )
        }
        Err(err) => {
            let message = format!("confirm passkey failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PasskeyConfirmOutput {
            registration_id: input.registration_id,
            public_key_options,
            expires_at: expires_at.timestamp(),
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
    let registration_id = input.registration_id.trim().to_string();
    let challenge = match state.db.with_client(move |conn| {
        repo::cleanup_security_state_conn(conn, now)?;
        repo::get_passkey_challenge_conn(conn, registration_id.as_str())
    }) {
        Ok(Some(v)) => v,
        Ok(None) => {
            return api_error(
                StatusCode::NOT_FOUND,
                1004,
                "passkey registration not found",
            )
        }
        Err(err) => {
            let message = format!("query passkey challenge failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
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
    if !challenge.citizen_wallet_confirmed {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "citizen wallet confirmation required first",
        );
    }
    let Some(webauthn_state) = challenge.webauthn_state.clone() else {
        return api_error(
            StatusCode::CONFLICT,
            1005,
            "passkey registration confirmation missing",
        );
    };
    let passkey = match webauthn().and_then(|w| {
        w.finish_passkey_registration(&input.credential, &webauthn_state)
            .map_err(|err| {
                tracing::warn!(error = %err, "finish passkey registration failed");
                api_error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    2004,
                    "passkey registration failed",
                )
            })
    }) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let credential_id = credential_id_hex(&passkey);
    let label = challenge.label.clone();
    let result = state.db.with_client({
        let ctx_pubkey = ctx.admin_pubkey.clone();
        let credential_id = credential_id.clone();
        let registration_id = input.registration_id.clone();
        move |conn| {
            if repo::get_passkey_credential_conn(conn, credential_id.as_str())?.is_some() {
                return Err("http:conflict:passkey credential already exists".to_string());
            }
            repo::revoke_active_passkeys_for_admin_conn(conn, ctx_pubkey.as_str())?;
            repo::upsert_passkey_credential_conn(
                conn,
                &AdminPasskeyCredential {
                    credential_id: credential_id.clone(),
                    admin_pubkey: ctx_pubkey.clone(),
                    label,
                    passkey,
                    status: AdminPasskeyStatus::Active,
                    created_at: now,
                    last_used_at: None,
                },
            )?;
            let Some(mut challenge) =
                repo::get_passkey_challenge_conn(conn, registration_id.as_str())?
            else {
                return Err("http:not_found:passkey registration not found".to_string());
            };
            challenge.consumed = true;
            repo::upsert_passkey_challenge_conn(conn, &challenge)?;
            repo::active_passkey_credentials_conn(conn, ctx_pubkey.as_str()).map(|v| v.len())
        }
    });
    let count = match result {
        Ok(v) => v,
        Err(err) if err == "http:conflict:passkey credential already exists" => {
            return api_error(
                StatusCode::CONFLICT,
                1005,
                "passkey credential already exists",
            )
        }
        Err(err) if err == "http:not_found:passkey registration not found" => {
            return api_error(
                StatusCode::NOT_FOUND,
                1004,
                "passkey registration not found",
            )
        }
        Err(err) => {
            let message = format!("complete passkey failed: {err}");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str());
        }
    };

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

pub(crate) fn active_passkeys(db: &Db, admin_pubkey: &str) -> Result<Vec<Passkey>, String> {
    let admin_pubkey = admin_pubkey.to_string();
    db.with_client(move |conn| {
        repo::active_passkey_credentials_conn(conn, admin_pubkey.as_str()).map(|rows| {
            rows.into_iter()
                .map(|record| record.passkey)
                .collect::<Vec<_>>()
        })
    })
}

pub(crate) fn update_passkey_usage_conn(
    conn: &mut postgres::Client,
    admin_pubkey: &str,
    assertion: &PublicKeyCredential,
    auth_result: &AuthenticationResult,
    now: DateTime<Utc>,
) -> Result<(), axum::response::Response> {
    let credential_id = format!("0x{}", hex::encode(assertion.raw_id.as_ref()));
    let Some(mut record) = repo::get_passkey_credential_conn(conn, credential_id.as_str())
        .map_err(|err| {
            let message = format!("query passkey credential failed: {err}");
            api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str())
        })?
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
    repo::upsert_passkey_credential_conn(conn, &record).map_err(|err| {
        let message = format!("update passkey usage failed: {err}");
        api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, message.as_str())
    })
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

pub(crate) fn verify_citizen_wallet_signature(
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
    if !crate::admins::login::verify_admin_signature(signer_pubkey, payload_text, signature) {
        return Err(api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            2004,
            "signature verify failed",
        ));
    }
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
