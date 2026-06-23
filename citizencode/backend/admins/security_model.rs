//! 中文注释:管理员 Passkey、公民钱包确认和一次性安全授权模型。
//!
//! 这些结构只服务联邦注册局管理员/市注册局管理员安全动作,因此归属 `admins`,不再放在全局
//! `models` 目录里。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{Passkey, PasskeyAuthentication, PasskeyRegistration};

use crate::admins::model::RegistryOrgCode;
use crate::admins::operation_auth::AdminOperationAuth;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminPasskeyStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminPasskeyCredential {
    pub(crate) credential_id: String,
    pub(crate) admin_account: String,
    pub(crate) label: String,
    pub(crate) passkey: Passkey,
    pub(crate) status: AdminPasskeyStatus,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminPasskeyRegistrationChallenge {
    pub(crate) registration_id: String,
    pub(crate) admin_account: String,
    pub(crate) admin_name: String,
    pub(crate) label: String,
    /// 中文注释:公民钱包确认通过后才生成并保存 WebAuthn registration state。
    #[serde(default)]
    pub(crate) webauthn_state: Option<PasskeyRegistration>,
    pub(crate) payload_text: String,
    pub(crate) payload_hash: String,
    #[serde(default)]
    pub(crate) citizen_wallet_confirmed: bool,
    pub(crate) issued_at: DateTime<Utc>,
    pub(crate) expires_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) consumed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminActionChallenge {
    pub(crate) action_id: String,
    pub(crate) action_type: String,
    pub(crate) actor_account: String,
    pub(crate) actor_registry_org_code: RegistryOrgCode,
    pub(crate) actor_province_name: String,
    #[serde(default)]
    pub(crate) actor_city_name: Option<String>,
    pub(crate) auth_type: AdminOperationAuth,
    pub(crate) target: String,
    pub(crate) payload_text: String,
    pub(crate) payload_hash: String,
    pub(crate) before_hash: String,
    pub(crate) after_hash: String,
    pub(crate) request_payload: serde_json::Value,
    pub(crate) webauthn_state: PasskeyAuthentication,
    pub(crate) issued_at: DateTime<Utc>,
    pub(crate) expires_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) consumed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminSecurityGrant {
    pub(crate) grant_id: String,
    pub(crate) action_type: String,
    pub(crate) actor_account: String,
    pub(crate) actor_registry_org_code: RegistryOrgCode,
    pub(crate) actor_province_name: String,
    #[serde(default)]
    pub(crate) actor_city_name: Option<String>,
    pub(crate) auth_type: AdminOperationAuth,
    pub(crate) target: String,
    pub(crate) payload_hash: String,
    pub(crate) issued_at: DateTime<Utc>,
    pub(crate) expires_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) consumed: bool,
}
