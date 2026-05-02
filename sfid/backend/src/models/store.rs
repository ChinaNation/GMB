//! 中文注释:进程内 `Store` 聚合体 + 敏感种子封装 + 服务指标 / 审计 / 链请求回执 /
//! 异步绑定回调 / 公民奖励 / 投票验证缓存 / Keyring 轮换会话 / 机构与账户链上状态。
//!
//! 内容统一来自 phase23a 拆分前的 `models/mod.rs`,本文件维护 `Store` 这棵
//! 进程内状态树的全部字段类型;子领域 DTO(citizen/cpms/role/meta)单独成文件。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use zeroize::Zeroize;

use crate::key_admins::chain_keyring::ChainKeyringState;
use crate::login::{AdminSession, LoginChallenge, QrLoginResultRecord};

use super::citizen::{
    CitizenBindChallenge, CitizenRecord, CitizenStatus, ImportedArchive, PendingBindScan,
};
use super::cpms::CpmsSiteKeys;

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
    pub(crate) chain_keyring_state: Option<ChainKeyringState>,
    pub(crate) keyring_rotate_challenges: HashMap<String, KeyringRotateChallenge>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeyringRotateChallenge {
    pub(crate) challenge_id: String,
    pub(crate) keyring_version: u64,
    pub(crate) initiator_pubkey: String,
    pub(crate) challenge_text: String,
    pub(crate) expire_at: DateTime<Utc>,
    pub(crate) verified_at: Option<DateTime<Utc>>,
    pub(crate) consumed: bool,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
}

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

// 中文注释:ChainRequestAuth 配套于已下架的 chain HMAC 鉴权(prepare_chain_request),
// 2026-05-01 一并下架。

#[derive(Deserialize)]
pub(crate) struct AuditLogsQuery {
    pub(crate) action: Option<String>,
    pub(crate) actor_pubkey: Option<String>,
    pub(crate) keyword: Option<String>,
    pub(crate) limit: Option<usize>,
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

// ── Keyring 轮换接口类型(管理员密钥环) ─────────────────

#[derive(Serialize)]
pub(crate) struct KeyringStateOutput {
    pub(crate) version: u64,
    pub(crate) main_pubkey: String,
    pub(crate) main_name: String,
    pub(crate) backup_a_pubkey: String,
    pub(crate) backup_a_name: String,
    pub(crate) backup_b_pubkey: String,
    pub(crate) backup_b_name: String,
    pub(crate) updated_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct KeyringRotateChallengeInput {
    pub(crate) initiator_pubkey: String,
}

#[derive(Serialize)]
pub(crate) struct KeyringRotateChallengeOutput {
    pub(crate) challenge_id: String,
    pub(crate) keyring_version: u64,
    pub(crate) challenge_text: String,
    pub(crate) expire_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct KeyringRotateCommitInput {
    pub(crate) challenge_id: String,
    pub(crate) signature: String,
    pub(crate) new_backup_pubkey: String,
    /// 新备用管理员姓名(必填)
    #[serde(default)]
    pub(crate) new_backup_name: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct KeyringRotateVerifyInput {
    pub(crate) challenge_id: String,
    pub(crate) signature: String,
}

#[derive(Serialize)]
pub(crate) struct KeyringRotateVerifyOutput {
    pub(crate) challenge_id: String,
    pub(crate) initiator_pubkey: String,
    pub(crate) keyring_version: u64,
    pub(crate) verified: bool,
    pub(crate) message: &'static str,
}

#[derive(Serialize)]
pub(crate) struct KeyringRotateCommitOutput {
    pub(crate) old_main_pubkey: String,
    pub(crate) promoted_slot: String,
    pub(crate) chain_tx_hash: String,
    pub(crate) block_number: Option<u64>,
    pub(crate) chain_submit_ok: bool,
    pub(crate) chain_submit_error: Option<String>,
    pub(crate) version: u64,
    pub(crate) main_pubkey: String,
    pub(crate) backup_a_pubkey: String,
    pub(crate) backup_b_pubkey: String,
    pub(crate) updated_at: i64,
    pub(crate) message: String,
}

// ── 多签管理:机构 / 账户链上状态 ─────────────────────

/// 机构链上注册状态。
///
/// 中文注释:SFID 系统只记录链上同步回来的机构状态,不主动创建或注销链上机构。
/// 创建 SFID 时默认为 `NotRegistered`;链上注册/注销成功后由受信任同步接口更新。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum InstitutionChainStatus {
    NotRegistered,
    PendingRegister,
    Registered,
    RevokedOnChain,
}

impl Default for InstitutionChainStatus {
    fn default() -> Self {
        Self::NotRegistered
    }
}

/// 机构账户链上状态。
///
/// 中文注释:账户是否激活只以链上事实为准。SFID 创建账户时只是登记
/// `(sfid_id, account_name)`,默认 `NotOnChain`;链上机构注册或新增账户成功后,
/// 由同步接口写成 `ActiveOnChain`;链上注销后写成 `RevokedOnChain`。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum MultisigChainStatus {
    NotOnChain,
    PendingOnChain,
    ActiveOnChain,
    RevokedOnChain,
}

impl Default for MultisigChainStatus {
    fn default() -> Self {
        Self::NotOnChain
    }
}
