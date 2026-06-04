//! 中文注释:进程内 `Store` 聚合体 + 敏感种子封装 + 服务指标 / 链请求回执 /
//! 公民奖励 / 投票验证缓存。
//!
//! 中文注释:本文件维护 `Store` 聚合体类型。运行时短锁仍使用这棵内存对象,
//! 持久化由 `main.rs` 拆成各模块 Store 快照表,不再写整包 runtime JSON。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use zeroize::Zeroize;

use crate::admins::login::{AdminSession, LoginChallenge, QrLoginResultRecord};
use crate::admins::model::AdminUser;
use crate::admins::security_model::{
    AdminActionChallenge, AdminPasskeyCredential, AdminPasskeyRegistrationChallenge,
    AdminSecurityGrant,
};
use crate::audit::AuditLogEntry;
use crate::citizens::model::{
    CitizenBindChallenge, CitizenRecord, CitizenStatus, CpmsStatusExportImportRecord,
};
use crate::cpms::model::CpmsSiteKeys;

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub(crate) struct SensitiveSeed(String);

impl SensitiveSeed {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Exposes the raw seed text for cryptographic operations only.
    /// Never use this in logs, panic messages, or formatted errors.
    #[must_use = "secret material should only be exposed to crypto code paths"]
    pub(crate) fn expose_secret(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for SensitiveSeed {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SensitiveSeed {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl Drop for SensitiveSeed {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for SensitiveSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SensitiveSeed(***)")
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Store {
    pub(crate) next_seq: u64,
    pub(crate) next_audit_seq: u64,
    pub(crate) next_admin_user_id: u64,
    // ── 公民身份记录（新模型）──
    #[serde(default)]
    pub(crate) next_citizen_id: u64,
    #[serde(default)]
    pub(crate) citizen_records: HashMap<u64, CitizenRecord>,
    /// 反向索引：wallet_pubkey → citizen_id
    #[serde(default)]
    pub(crate) citizen_id_by_wallet_pubkey: HashMap<String, u64>,
    /// 反向索引：archive_no → citizen_id
    #[serde(default)]
    pub(crate) citizen_id_by_archive_no: HashMap<String, u64>,
    /// 反向索引：sfid_code → citizen_id
    #[serde(default)]
    pub(crate) citizen_id_by_sfid_code: HashMap<String, u64>,
    /// 绑定 challenge 池
    #[serde(default)]
    pub(crate) citizen_bind_challenges: HashMap<String, CitizenBindChallenge>,
    /// CPMS 年度报告导入幂等记录，key = "{sfid_number}|{export_year}"。
    #[serde(default)]
    pub(crate) cpms_status_export_imports: HashMap<String, CpmsStatusExportImportRecord>,
    pub(crate) admin_users_by_pubkey: HashMap<String, AdminUser>,
    pub(crate) sheng_admin_province_by_pubkey: HashMap<String, String>,
    /// 中文注释:省/市管理员的 Passkey 凭据,只保存服务端可验证的公钥凭据。
    #[serde(default)]
    pub(crate) admin_passkeys_by_credential_id: HashMap<String, AdminPasskeyCredential>,
    /// 中文注释:Passkey 注册挑战必须先完成冷钱包签名确认,再进入 WebAuthn 创建。
    #[serde(default)]
    pub(crate) admin_passkey_registration_challenges:
        HashMap<String, AdminPasskeyRegistrationChallenge>,
    /// 中文注释:管理员 PASSKEY/PASSKEY_CHALLENGE 写操作的短期安全挑战,
    /// 提交后一次性消费;LOGIN_STATE 操作不进入这里。
    #[serde(default)]
    pub(crate) admin_action_challenges: HashMap<String, AdminActionChallenge>,
    /// 中文注释:业务写接口使用的短期一次性授权。前端先按操作类型完成
    /// PASSKEY 或 PASSKEY_CHALLENGE,再把 grant id 放入 x-sfid-security-grant 请求头。
    #[serde(default)]
    pub(crate) admin_security_grants: HashMap<String, AdminSecurityGrant>,
    pub(crate) login_challenges: HashMap<String, LoginChallenge>,
    pub(crate) qr_login_results: HashMap<String, QrLoginResultRecord>,
    pub(crate) admin_sessions: HashMap<String, AdminSession>,
    pub(crate) cpms_site_keys: HashMap<String, CpmsSiteKeys>,
    pub(crate) consumed_qr_ids: HashMap<String, DateTime<Utc>>,
    pub(crate) audit_logs: Vec<AuditLogEntry>,
    pub(crate) chain_requests_by_key: HashMap<String, ChainRequestReceipt>,
    pub(crate) chain_nonce_seen: HashMap<String, DateTime<Utc>>,
    pub(crate) chain_auth_last_cleanup_at: Option<DateTime<Utc>>,
    pub(crate) reward_state_by_pubkey: HashMap<String, RewardStateRecord>,
    pub(crate) vote_verify_cache: HashMap<String, VoteVerifyCacheEntry>,
    pub(crate) metrics: ServiceMetrics,
    /// 机构层(每 sfid_number 唯一),任务卡 2 引入。
    #[serde(default)]
    pub(crate) multisig_institutions: HashMap<String, crate::subjects::MultisigInstitution>,
    /// 账户层(key = "sfid_number|account_name"),任务卡 2 引入。account_name 就是链上 name。
    #[serde(default)]
    pub(crate) multisig_accounts: HashMap<String, crate::subjects::MultisigAccount>,
    /// 机构资料库文档,key = document id(字符串化)。
    #[serde(default)]
    pub(crate) institution_documents: HashMap<String, crate::subjects::InstitutionDocument>,
    /// 文档自增 ID。
    #[serde(default)]
    pub(crate) next_document_id: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ServiceMetrics {
    pub(crate) chain_auth_failures: u64,
    pub(crate) chain_replay_rejects: u64,
    pub(crate) bind_requests_total: u64,
    pub(crate) bind_confirms_total: u64,
    pub(crate) vote_verify_total: u64,
    pub(crate) binding_validate_total: u64,
    pub(crate) voters_count_total: u64,
    pub(crate) chain_request_total: u64,
    pub(crate) chain_request_failed_total: u64,
    pub(crate) chain_latency_samples: Vec<u32>,
    /// Store 持久化失败次数(严重:数据可能丢失)
    #[serde(default)]
    pub(crate) store_persist_failures: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChainRequestReceipt {
    pub(crate) route_key: String,
    pub(crate) request_id: String,
    pub(crate) nonce: String,
    pub(crate) fingerprint: String,
    pub(crate) received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RewardStatus {
    Pending,
    Rewarded,
    RetryWaiting,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RewardStateRecord {
    pub(crate) account_pubkey: String,
    pub(crate) archive_index: String,
    pub(crate) callback_id: String,
    pub(crate) reward_status: RewardStatus,
    pub(crate) retry_count: u32,
    pub(crate) max_retries: u32,
    pub(crate) reward_tx_hash: Option<String>,
    pub(crate) last_error: Option<String>,
    pub(crate) next_retry_at: Option<DateTime<Utc>>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VoteVerifyCacheEntry {
    pub(crate) account_pubkey: String,
    pub(crate) proposal_id: Option<u64>,
    pub(crate) is_bound: bool,
    pub(crate) has_vote_eligibility: bool,
    pub(crate) sfid_code: Option<String>,
    pub(crate) archive_index: Option<String>,
    pub(crate) citizen_status: Option<CitizenStatus>,
    pub(crate) cached_at: DateTime<Utc>,
}
