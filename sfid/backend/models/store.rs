//! 中文注释:进程内 `Store` 聚合体 + 敏感种子封装 + 服务指标 / 审计 / 链请求回执 /
//! 异步绑定回调 / 公民奖励 / 投票验证缓存。
//!
//! 中文注释:本文件维护 `Store` 这棵进程内状态树。业务模型类型只引用对应
//! 功能模块,不在 `models` 里复制定义。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use zeroize::Zeroize;

use crate::login::{AdminSession, LoginChallenge, QrLoginResultRecord};

use crate::citizens::model::{
    CitizenBindChallenge, CitizenRecord, CitizenStatus, ImportedArchive, PendingBindScan,
};
use crate::cpms::model::CpmsSiteKeys;

/// 中文注释:历史 `make_signature_envelope` 已下线,本结构仅保留作为
/// `BindCallbackPayload.proof / callback_attestation` 字段类型(目前由
/// runtime_align 单边产出,未实际填充)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SignatureEnvelope {
    pub(crate) key_id: String,
    pub(crate) key_version: String,
    pub(crate) alg: String,
    pub(crate) payload: String,
    pub(crate) signature_hex: String,
}

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
    /// 反向索引：pubkey → citizen_id
    #[serde(default)]
    pub(crate) citizen_id_by_pubkey: HashMap<String, u64>,
    /// 反向索引：archive_no → citizen_id
    #[serde(default)]
    pub(crate) citizen_id_by_archive_no: HashMap<String, u64>,
    /// 反向索引：sfid_code → citizen_id
    #[serde(default)]
    pub(crate) citizen_id_by_sfid_code: HashMap<String, u64>,
    /// 绑定 challenge 池
    #[serde(default)]
    pub(crate) citizen_bind_challenges: HashMap<String, CitizenBindChallenge>,
    pub(crate) admin_users_by_pubkey: HashMap<String, super::role::AdminUser>,
    pub(crate) sheng_admin_province_by_pubkey: HashMap<String, String>,
    /// 中文注释:省管理员一主两备的 SFID 本地备用槽记录。
    /// main 仍来自内置省级管理员基线;backup_1 / backup_2 先在 SFID 本地保存,
    /// 后续链上更换省管理员能力落地后再对齐链上真相。
    #[serde(default)]
    pub(crate) sheng_admin_rosters: HashMap<String, ShengAdminRosterLocal>,
    pub(crate) login_challenges: HashMap<String, LoginChallenge>,
    pub(crate) qr_login_results: HashMap<String, QrLoginResultRecord>,
    pub(crate) admin_sessions: HashMap<String, AdminSession>,
    pub(crate) cpms_site_keys: HashMap<String, CpmsSiteKeys>,
    /// 已录入的档案记录，key = archive_no。
    pub(crate) imported_archives: HashMap<String, ImportedArchive>,
    pub(crate) consumed_cpms_register_tokens: HashMap<String, DateTime<Utc>>,
    pub(crate) consumed_qr_ids: HashMap<String, DateTime<Utc>>,
    pub(crate) pending_status_by_archive_no: HashMap<String, CitizenStatus>,
    pub(crate) pending_bind_scan_by_qr_id: HashMap<String, PendingBindScan>,
    pub(crate) generated_sfid_by_pubkey: HashMap<String, String>,
    /// RSABSSA 匿名证书签发 RSA 私钥 PEM（自动生成，持久化）。
    #[serde(default)]
    pub(crate) anon_rsa_private_key_pem: Option<String>,
    pub(crate) audit_logs: Vec<AuditLogEntry>,
    pub(crate) chain_requests_by_key: HashMap<String, ChainRequestReceipt>,
    pub(crate) chain_nonce_seen: HashMap<String, DateTime<Utc>>,
    pub(crate) chain_auth_last_cleanup_at: Option<DateTime<Utc>>,
    pub(crate) pending_bind_last_cleanup_at: Option<DateTime<Utc>>,
    pub(crate) bind_callback_jobs: Vec<BindCallbackJob>,
    pub(crate) reward_state_by_pubkey: HashMap<String, RewardStateRecord>,
    pub(crate) vote_verify_cache: HashMap<String, VoteVerifyCacheEntry>,
    pub(crate) metrics: ServiceMetrics,
    /// 机构层(每 sfid_id 唯一),任务卡 2 引入。
    #[serde(default)]
    pub(crate) multisig_institutions: HashMap<String, crate::institutions::MultisigInstitution>,
    /// 账户层(key = "sfid_id|account_name"),任务卡 2 引入。account_name 就是链上 name。
    #[serde(default)]
    pub(crate) multisig_accounts: HashMap<String, crate::institutions::MultisigAccount>,
    /// 机构资料库文档,key = document id(字符串化)。
    #[serde(default)]
    pub(crate) institution_documents: HashMap<String, crate::institutions::InstitutionDocument>,
    /// 文档自增 ID。
    #[serde(default)]
    pub(crate) next_document_id: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ShengAdminRosterLocal {
    pub(crate) backup_1: Option<ShengAdminSlotLocal>,
    pub(crate) backup_2: Option<ShengAdminSlotLocal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShengAdminSlotLocal {
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

// 中文注释:旧签名轮换 DTO 已删除。省级 3-tier 名册由 chain runtime 上
// `ShengAdmins` storage 持有真相,SFID 不再维护本地签名轮换流程。

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuditLogEntry {
    pub(crate) seq: u64,
    pub(crate) action: String,
    pub(crate) actor_pubkey: String,
    pub(crate) target_pubkey: Option<String>,
    pub(crate) target_archive_no: Option<String>,
    #[serde(default)]
    pub(crate) request_id: Option<String>,
    #[serde(default)]
    pub(crate) actor_ip: Option<String>,
    pub(crate) result: String,
    pub(crate) detail: String,
    pub(crate) created_at: DateTime<Utc>,
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
    pub(crate) bind_callback_success_total: u64,
    pub(crate) bind_callback_retry_total: u64,
    pub(crate) bind_callback_failed_total: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BindCallbackJob {
    pub(crate) callback_id: String,
    pub(crate) callback_url: String,
    pub(crate) payload: BindCallbackPayload,
    pub(crate) attempts: u32,
    pub(crate) max_attempts: u32,
    pub(crate) next_attempt_at: DateTime<Utc>,
    pub(crate) last_error: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BindCallbackPayload {
    pub(crate) callback_id: String,
    pub(crate) event: String,
    pub(crate) account_pubkey: String,
    pub(crate) archive_index: String,
    pub(crate) sfid_code: String,
    pub(crate) status: String,
    pub(crate) bound_at: i64,
    pub(crate) proof: SignatureEnvelope,
    pub(crate) client_request_id: Option<String>,
    pub(crate) callback_attestation: SignatureEnvelope,
}
