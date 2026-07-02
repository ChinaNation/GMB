//! 管理员敏感动作的扫码签名挑战与一次性安全授权模型。
//!
//! 这些结构服务机构管理员安全动作,因此归属 `admins`。
//! step-up = 会话 + 冷钱包扫码签名。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::operation_auth::AdminOperationAuth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminActionChallenge {
    pub(crate) action_id: String,
    pub(crate) action_type: String,
    pub(crate) actor_account: String,
    pub(crate) actor_institution_code: String,
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
    pub(crate) actor_institution_code: String,
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
