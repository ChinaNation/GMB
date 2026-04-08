use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use zeroize::Zeroize;

use crate::key_admins::chain_keyring::ChainKeyringState;
use crate::key_admins::chain_proof::SignatureEnvelope;
use crate::login::{AdminSession, LoginChallenge, QrLoginResultRecord};

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
    pub(crate) pending_by_pubkey: HashMap<String, PendingRequest>,
    pub(crate) bindings_by_pubkey: HashMap<String, BindingRecord>,
    pub(crate) pubkey_by_archive_index: HashMap<String, String>,
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
    /// 绑定 challenge 池
    #[serde(default)]
    pub(crate) citizen_bind_challenges: HashMap<String, CitizenBindChallenge>,
    pub(crate) admin_users_by_pubkey: HashMap<String, AdminUser>,
    pub(crate) super_admin_province_by_pubkey: HashMap<String, String>,
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
    /// 多签机构 SFID 注册记录，key = site_sfid
    #[serde(default)]
    pub(crate) multisig_sfid_records: HashMap<String, MultisigSfidRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminRole {
    KeyAdmin,
    InstitutionAdmin,
    SystemAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CpmsSiteStatus {
    Pending,
    Active,
    Disabled,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CitizenStatus {
    Normal,
    Abnormal,
}

fn default_cpms_site_status() -> CpmsSiteStatus {
    CpmsSiteStatus::Active
}

fn default_cpms_site_version() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminUser {
    pub(crate) id: u64,
    pub(crate) admin_pubkey: String,
    #[serde(default)]
    pub(crate) admin_name: String,
    pub(crate) role: AdminRole,
    pub(crate) status: AdminStatus,
    pub(crate) built_in: bool,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
    /// SystemAdmin 所属的市名称（仅 SystemAdmin 必填，其他角色为空字符串）
    #[serde(default)]
    pub(crate) city: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CpmsSiteKeys {
    pub(crate) site_sfid: String,
    #[serde(default)]
    pub(crate) install_token: String,
    #[serde(default = "default_install_token_status")]
    pub(crate) install_token_status: InstallTokenStatus,
    #[serde(default = "default_cpms_site_status")]
    pub(crate) status: CpmsSiteStatus,
    #[serde(default = "default_cpms_site_version")]
    pub(crate) version: u64,
    #[serde(default)]
    pub(crate) province_code: String,
    pub(crate) admin_province: String,
    #[serde(default)]
    pub(crate) city_name: String,
    #[serde(default)]
    pub(crate) institution_code: String,
    #[serde(default)]
    pub(crate) institution_name: String,
    #[serde(default)]
    pub(crate) qr1_payload: String,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) updated_by: Option<String>,
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum InstallTokenStatus {
    Pending,
    Used,
    Revoked,
}

fn default_install_token_status() -> InstallTokenStatus {
    InstallTokenStatus::Pending
}

/// SFID 端录入的档案记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ImportedArchive {
    pub(crate) archive_no: String,
    /// 以验签通过后的 anon_cert.province_code 为准。
    pub(crate) province_code: String,
    /// 匿名证书 SHA-256 摘要，用于审计。
    pub(crate) anon_cert_hash: String,
    pub(crate) imported_at: DateTime<Utc>,
    #[serde(default = "default_archive_import_status")]
    pub(crate) status: ArchiveImportStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ArchiveImportStatus {
    Active,
    Revoked,
}

fn default_archive_import_status() -> ArchiveImportStatus {
    ArchiveImportStatus::Active
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PendingRequest {
    pub(crate) seq: u64,
    pub(crate) account_pubkey: String,
    pub(crate) admin_province: Option<String>,
    pub(crate) requested_at: DateTime<Utc>,
    pub(crate) callback_url: Option<String>,
    pub(crate) client_request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PendingBindScan {
    pub(crate) qr_id: String,
    pub(crate) archive_no: String,
    pub(crate) site_sfid: String,
    pub(crate) status: CitizenStatus,
    pub(crate) expire_at: i64,
    pub(crate) scanned_at: DateTime<Utc>,
}

// ── 公民身份记录（新模型）──────────────────────────────────────────────

/// 公民身份记录。
///
/// 以自增 ID 为主键，account_pubkey / archive_no / sfid_code 各自唯一（非空时）。
/// 三种状态：
/// - Unbound：只有 pubkey（区块链传入，未绑定档案）
/// - Bound：pubkey + archive_no + sfid_code 三者都有
/// - Unlinked：只有 archive_no + sfid_code（解绑后公钥清除）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CitizenRecord {
    pub(crate) id: u64,
    pub(crate) account_pubkey: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) sfid_signature: Option<String>,
    pub(crate) province_code: Option<String>,
    pub(crate) bound_at: Option<DateTime<Utc>>,
    pub(crate) bound_by: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

impl CitizenRecord {
    pub(crate) fn status(&self) -> CitizenBindStatus {
        match (&self.account_pubkey, &self.archive_no) {
            (Some(_), Some(_)) => CitizenBindStatus::Bound,
            (Some(_), None) => CitizenBindStatus::Unbound,
            (None, Some(_)) => CitizenBindStatus::Unlinked,
            (None, None) => CitizenBindStatus::Unbound,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CitizenBindStatus {
    /// 只有公钥，未绑定档案。
    Unbound,
    /// 三者都有，已绑定。
    Bound,
    /// 解绑后，只有档案号+SFID码，公钥已清除。
    Unlinked,
}

/// 绑定 challenge（公钥签名验证）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CitizenBindChallenge {
    pub(crate) challenge_id: String,
    pub(crate) challenge_text: String,
    pub(crate) account_pubkey: String,
    pub(crate) expire_at: DateTime<Utc>,
    pub(crate) created_at: DateTime<Utc>,
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
pub(crate) struct BindingRecord {
    pub(crate) seq: u64,
    pub(crate) account_pubkey: String,
    pub(crate) archive_index: String,
    pub(crate) birth_date: Option<NaiveDate>,
    pub(crate) citizen_status: CitizenStatus,
    pub(crate) sfid_code: String,
    pub(crate) sfid_signature: String,
    #[serde(default)]
    pub(crate) runtime_bind_binding_id: Option<String>,
    #[serde(default)]
    pub(crate) runtime_bind_bind_nonce: Option<String>,
    #[serde(default)]
    pub(crate) runtime_bind_signature: Option<String>,
    #[serde(default)]
    pub(crate) runtime_bind_key_id: Option<String>,
    #[serde(default)]
    pub(crate) runtime_bind_key_version: Option<String>,
    #[serde(default)]
    pub(crate) runtime_bind_alg: Option<String>,
    #[serde(default)]
    pub(crate) runtime_bind_signer_pubkey: Option<String>,
    pub(crate) bound_at: DateTime<Utc>,
    pub(crate) bound_by: String,
    pub(crate) admin_province: Option<String>,
    pub(crate) client_request_id: Option<String>,
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

#[derive(Debug, Clone)]
pub(crate) struct ChainRequestAuth {
    pub(crate) request_id: String,
    pub(crate) nonce: String,
    pub(crate) timestamp: i64,
}

#[derive(Deserialize)]
pub(crate) struct AuditLogsQuery {
    pub(crate) action: Option<String>,
    pub(crate) actor_pubkey: Option<String>,
    pub(crate) keyword: Option<String>,
    pub(crate) limit: Option<usize>,
}
#[derive(Serialize)]
pub(crate) struct ApiResponse<T: Serialize> {
    pub(crate) code: u32,
    pub(crate) message: String,
    pub(crate) data: T,
}

#[derive(Serialize)]
pub(crate) struct ApiError {
    pub(crate) code: u32,
    pub(crate) message: String,
    pub(crate) trace_id: String,
}

#[derive(Serialize)]
pub(crate) struct HealthData {
    pub(crate) service: &'static str,
    pub(crate) status: &'static str,
    pub(crate) checked_at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BindRequestInput {
    pub(crate) account_pubkey: String,
    pub(crate) callback_url: Option<String>,
    pub(crate) client_request_id: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct BindRequestOutput {
    pub(crate) account_pubkey: String,
    pub(crate) chain_request_id: String,
    pub(crate) status: &'static str,
    pub(crate) message: &'static str,
}

#[derive(Deserialize)]
pub(crate) struct AdminQueryInput {
    pub(crate) account_pubkey: String,
}

#[derive(Serialize)]
pub(crate) struct AdminQueryOutput {
    pub(crate) account_pubkey: String,
    pub(crate) found_pending: bool,
    pub(crate) found_binding: bool,
    pub(crate) archive_index: Option<String>,
    pub(crate) sfid_code: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminBindInput {
    pub(crate) account_pubkey: String,
    pub(crate) archive_index: String,
    pub(crate) qr_id: String,
}

#[derive(Deserialize)]
pub(crate) struct AdminUnbindInput {
    pub(crate) account_pubkey: String,
}

// ── 公民身份绑定接口类型 ──

/// 绑定/解绑 challenge 返回。
#[derive(Serialize)]
pub(crate) struct CitizenBindChallengeOutput {
    pub(crate) challenge_id: String,
    pub(crate) challenge_text: String,
    /// WUMIN_SIGN_V1.0.0 签名请求 JSON（前端直接展示为二维码）。
    pub(crate) sign_request: String,
    pub(crate) expire_at: i64,
}

/// 绑定请求（两种模式）。
#[derive(Deserialize)]
pub(crate) struct CitizenBindInput {
    /// "bind_archive"（全新绑定）或 "bind_pubkey"（重新绑定公钥）
    pub(crate) mode: String,
    /// 用户 SS58 地址（从 WUMIN_USER_V1.0.0 二维码获取）
    pub(crate) user_address: String,
    /// QR4 二维码内容（mode=bind_archive 时必填）
    pub(crate) qr4_payload: Option<String>,
    /// 记录 ID（mode=bind_pubkey 时必填）
    pub(crate) citizen_id: Option<u64>,
    /// challenge ID
    pub(crate) challenge_id: String,
    /// WUMIN_SIGN_V1.0.0 签名结果（hex）
    pub(crate) signature: String,
}

/// 绑定返回。
#[derive(Serialize)]
pub(crate) struct CitizenBindOutput {
    pub(crate) id: u64,
    pub(crate) account_pubkey: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) province_code: Option<String>,
    pub(crate) status: CitizenBindStatus,
}

/// 解绑请求（需要公钥签名确认）。
#[derive(Deserialize)]
pub(crate) struct CitizenUnbindInput {
    pub(crate) citizen_id: u64,
    pub(crate) challenge_id: String,
    pub(crate) signature: String,
}

#[derive(Deserialize)]
pub(crate) struct CitizensQuery {
    pub(crate) keyword: Option<String>,
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PublicIdentitySearchQuery {
    pub(crate) archive_no: Option<String>,
    pub(crate) identity_code: Option<String>,
    pub(crate) account_pubkey: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PublicIdentitySearchOutput {
    pub(crate) found: bool,
    pub(crate) archive_no: Option<String>,
    pub(crate) identity_code: Option<String>,
    pub(crate) account_pubkey: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CitizenRow {
    pub(crate) id: u64,
    pub(crate) account_pubkey: Option<String>,
    pub(crate) archive_no: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) province_code: Option<String>,
    pub(crate) status: CitizenBindStatus,
}

// 保留旧版 CitizenRow 用于兼容旧查询
#[derive(Serialize)]
pub(crate) struct CitizenRowLegacy {
    pub(crate) seq: u64,
    pub(crate) account_pubkey: String,
    pub(crate) archive_index: Option<String>,
    pub(crate) sfid_code: Option<String>,
    pub(crate) citizen_status: Option<CitizenStatus>,
    pub(crate) is_bound: bool,
}

#[derive(Serialize)]
pub(crate) struct AdminBindOutput {
    pub(crate) account_pubkey: String,
    pub(crate) archive_index: String,
    pub(crate) sfid_code: String,
    pub(crate) proof: SignatureEnvelope,
    pub(crate) status: &'static str,
    pub(crate) message: &'static str,
}

#[derive(Deserialize)]
pub(crate) struct AdminGenerateSfidInput {
    pub(crate) account_pubkey: String,
    pub(crate) a3: String,
    pub(crate) p1: Option<String>,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) institution: String,
}

#[derive(Serialize)]
pub(crate) struct AdminGenerateSfidOutput {
    pub(crate) account_pubkey: String,
    pub(crate) sfid_code: String,
}

#[derive(Serialize)]
pub(crate) struct SfidOptionItem {
    pub(crate) label: &'static str,
    pub(crate) value: &'static str,
}

#[derive(Serialize)]
pub(crate) struct SfidProvinceItem {
    pub(crate) name: String,
    pub(crate) code: String,
}

#[derive(Serialize)]
pub(crate) struct SfidCityItem {
    pub(crate) name: String,
    pub(crate) code: String,
}

#[derive(Serialize)]
pub(crate) struct AdminSfidMetaOutput {
    pub(crate) a3_options: Vec<SfidOptionItem>,
    pub(crate) institution_options: Vec<SfidOptionItem>,
    pub(crate) provinces: Vec<SfidProvinceItem>,
    pub(crate) scoped_province: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminSfidCitiesQuery {
    pub(crate) province: String,
}

#[derive(Serialize)]
pub(crate) struct OperatorRow {
    pub(crate) id: u64,
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    pub(crate) role: AdminRole,
    pub(crate) status: AdminStatus,
    pub(crate) built_in: bool,
    pub(crate) created_by: String,
    pub(crate) created_by_name: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) city: String,
}

#[derive(Serialize)]
pub(crate) struct OperatorListOutput {
    pub(crate) total: usize,
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) rows: Vec<OperatorRow>,
}

// 机构管理员对外行（API 序列化）。
//
// SFID 业务语义：机构是永久存在的（43 个省份固定），机构管理员只是当前
// 替机构发声的人；不存在"停用"的机构管理员（被替换即彻底失效）。
// 因此对外暴露的行**不带 status 字段**。
#[derive(Serialize)]
pub(crate) struct SuperAdminRow {
    pub(crate) id: u64,
    pub(crate) province: String,
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    pub(crate) built_in: bool,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub(crate) struct CreateOperatorInput {
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    /// SystemAdmin 所属的市，必填，且必须属于 created_by 对应机构管理员的省份（不可为省辖市）
    pub(crate) city: String,
    /// 可选：指定该 operator 归属的机构管理员 pubkey。
    /// 仅 KeyAdmin 可指定，且必须是已存在的 InstitutionAdmin。
    /// InstitutionAdmin 调用时若指定则必须等于自己 pubkey，否则 403。
    /// 不指定则默认为调用者自身。
    #[serde(default)]
    pub(crate) created_by: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ReplaceSuperAdminInput {
    pub(crate) admin_pubkey: String,
}

#[derive(Deserialize)]
pub(crate) struct ListQuery {
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateOperatorInput {
    pub(crate) admin_pubkey: Option<String>,
    pub(crate) admin_name: Option<String>,
    /// 可选：修改 SystemAdmin 所属的市，必须属于该 operator 所属机构的省份（不可为省辖市）
    #[serde(default)]
    pub(crate) city: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateOperatorStatusInput {
    pub(crate) status: AdminStatus,
}

#[derive(Deserialize)]
pub(crate) struct CpmsRegisterScanInput {
    pub(crate) qr_payload: String,
}

#[derive(Deserialize)]
pub(crate) struct GenerateCpmsInstitutionSfidInput {
    pub(crate) province: Option<String>,
    pub(crate) city: String,
    pub(crate) institution: String,
    #[serde(default)]
    pub(crate) institution_name: Option<String>,
}

/// QR2 注册请求输入。
#[derive(Deserialize)]
pub(crate) struct CpmsRegisterInput {
    pub(crate) qr_payload: String,
}

/// QR4 档案录入输入。
#[derive(Deserialize)]
pub(crate) struct CpmsArchiveImportInput {
    pub(crate) qr_payload: String,
}

#[derive(Deserialize)]
pub(crate) struct UpdateCpmsSiteStatusInput {
    pub(crate) reason: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CpmsKeysListOutput {
    pub(crate) total: usize,
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) rows: Vec<CpmsSiteKeysListRow>,
}

#[derive(Serialize)]
pub(crate) struct CpmsSiteKeysListRow {
    pub(crate) site_sfid: String,
    pub(crate) install_token_status: InstallTokenStatus,
    pub(crate) status: CpmsSiteStatus,
    pub(crate) version: u64,
    pub(crate) province_code: String,
    pub(crate) admin_province: String,
    pub(crate) city_name: String,
    pub(crate) institution_code: String,
    pub(crate) institution_name: String,
    pub(crate) qr1_payload: String,
    pub(crate) created_by: String,
    pub(crate) created_by_name: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_by: Option<String>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub(crate) struct BindScanInput {
    pub(crate) qr_payload: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CitizenQrPayload {
    pub(crate) ver: String,
    pub(crate) issuer_id: String,
    pub(crate) site_sfid: String,
    pub(crate) archive_no: String,
    pub(crate) issued_at: i64,
    pub(crate) expire_at: i64,
    pub(crate) qr_id: String,
    pub(crate) sig_alg: String,
    pub(crate) status: CitizenStatus,
    pub(crate) signature: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CitizenStatusQrPayload {
    pub(crate) ver: String,
    pub(crate) issuer_id: String,
    pub(crate) site_sfid: String,
    pub(crate) archive_no: String,
    pub(crate) status: CitizenStatus,
    pub(crate) issued_at: i64,
    pub(crate) expire_at: i64,
    pub(crate) qr_id: String,
    pub(crate) sig_alg: String,
    pub(crate) signature: String,
}

/// QR2 解析后的注册请求（SFID_CPMS_V1）。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CpmsRegisterReqPayload {
    #[serde(default)]
    pub(crate) proto: String,
    #[serde(alias = "qr_type")]
    pub(crate) r#type: String,
    pub(crate) sfid: String,
    pub(crate) token: String,
    pub(crate) blind: String,
}

/// QR4 解析后的档案业务载荷（SFID_CPMS_V1）。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CpmsArchiveQrPayload {
    #[serde(default)]
    pub(crate) proto: String,
    #[serde(alias = "qr_type")]
    pub(crate) r#type: String,
    pub(crate) prov: String,
    pub(crate) ano: String,
    pub(crate) cs: String,
    pub(crate) ve: bool,
    pub(crate) cert: AnonCert,
    pub(crate) sig: String,
}

/// 匿名证书。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AnonCert {
    pub(crate) prov: String,
    pub(crate) pk: String,
    pub(crate) sig: String,
    #[serde(default)]
    pub(crate) mr: Option<String>,
}

/// 生成 SFID + QR1 的输出。
#[derive(Serialize)]
pub(crate) struct GenerateCpmsInstallOutput {
    pub(crate) site_sfid: String,
    pub(crate) qr1_payload: String,
}

/// 处理 QR2 注册请求后返回 QR3。
#[derive(Serialize)]
pub(crate) struct CpmsRegisterOutput {
    pub(crate) qr3_payload: String,
}

/// 档案录入结果。
#[derive(Serialize)]
pub(crate) struct CpmsArchiveImportOutput {
    pub(crate) archive_no: String,
    pub(crate) province_code: String,
    pub(crate) status: &'static str,
}

#[derive(Serialize)]
pub(crate) struct BindScanOutput {
    pub(crate) site_sfid: String,
    pub(crate) archive_no: String,
    pub(crate) qr_id: String,
    pub(crate) status: CitizenStatus,
    pub(crate) issued_at: i64,
    pub(crate) expire_at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BindResultQuery {
    pub(crate) account_pubkey: String,
}

#[derive(Serialize)]
pub(crate) struct BindResultOutput {
    pub(crate) genesis_hash: String,
    pub(crate) who: String,
    pub(crate) binding_id: String,
    pub(crate) bind_nonce: String,
    pub(crate) signature: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct VoteVerifyInput {
    pub(crate) account_pubkey: String,
    pub(crate) proposal_id: u64,
}

#[derive(Serialize)]
pub(crate) struct VoteVerifyOutput {
    pub(crate) genesis_hash: String,
    pub(crate) who: String,
    pub(crate) binding_id: String,
    pub(crate) proposal_id: u64,
    pub(crate) vote_nonce: String,
    pub(crate) signature: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ChainVotersCountQuery {
    pub(crate) account_pubkey: Option<String>,
    pub(crate) who: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ChainVotersCountOutput {
    pub(crate) genesis_hash: String,
    pub(crate) eligible_total: u64,
    pub(crate) who: String,
    pub(crate) snapshot_nonce: String,
    pub(crate) signature: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ChainBindingValidateInput {
    pub(crate) archive_no: String,
    pub(crate) account_pubkey: String,
}

#[derive(Serialize)]
pub(crate) struct ChainBindingValidateOutput {
    pub(crate) is_bound: bool,
    pub(crate) is_voting_eligible: bool,
    pub(crate) citizen_status: Option<CitizenStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RewardAckStatusInput {
    Success,
    Failed,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RewardAckInput {
    pub(crate) account_pubkey: String,
    pub(crate) callback_id: String,
    pub(crate) status: RewardAckStatusInput,
    pub(crate) reward_tx_hash: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) retry_after_seconds: Option<u64>,
}

#[derive(Serialize)]
pub(crate) struct RewardAckOutput {
    pub(crate) account_pubkey: String,
    pub(crate) callback_id: String,
    pub(crate) reward_status: RewardStatus,
    pub(crate) retry_count: u32,
    pub(crate) next_retry_at: Option<i64>,
    pub(crate) message: String,
}

#[derive(Serialize)]
pub(crate) struct RewardStateOutput {
    pub(crate) account_pubkey: String,
    pub(crate) archive_index: String,
    pub(crate) callback_id: String,
    pub(crate) reward_status: RewardStatus,
    pub(crate) retry_count: u32,
    pub(crate) max_retries: u32,
    pub(crate) reward_tx_hash: Option<String>,
    pub(crate) last_error: Option<String>,
    pub(crate) next_retry_at: Option<i64>,
    pub(crate) updated_at: i64,
    pub(crate) created_at: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RewardStateQuery {
    pub(crate) account_pubkey: String,
}

#[derive(Deserialize)]
pub(crate) struct CpmsStatusScanInput {
    pub(crate) qr_payload: String,
}

#[derive(Serialize)]
pub(crate) struct CpmsStatusScanOutput {
    pub(crate) archive_no: String,
    pub(crate) status: CitizenStatus,
    pub(crate) message: &'static str,
}

#[derive(Serialize)]
pub(crate) struct KeyringStateOutput {
    pub(crate) version: u64,
    pub(crate) main_pubkey: String,
    pub(crate) backup_a_pubkey: String,
    pub(crate) backup_b_pubkey: String,
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

#[derive(Serialize)]
pub(crate) struct BindingPayload {
    pub(crate) kind: &'static str,
    pub(crate) version: &'static str,
    pub(crate) account_pubkey: String,
    pub(crate) archive_index: String,
    pub(crate) sfid_code: String,
    pub(crate) issued_at: i64,
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

#[derive(Serialize)]
pub(crate) struct BindCallbackSignablePayload {
    pub(crate) callback_id: String,
    pub(crate) event: String,
    pub(crate) account_pubkey: String,
    pub(crate) archive_index: String,
    pub(crate) sfid_code: String,
    pub(crate) status: String,
    pub(crate) bound_at: i64,
    pub(crate) proof: SignatureEnvelope,
    pub(crate) client_request_id: Option<String>,
}

// ── 多签管理 ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum MultisigChainStatus {
    Pending,
    Registered,
    Failed,
}

impl Default for MultisigChainStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MultisigSfidRecord {
    pub(crate) site_sfid: String,
    pub(crate) a3: String,
    pub(crate) p1: String,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) institution_code: String,
    pub(crate) institution_name: String,
    pub(crate) province_code: String,
    pub(crate) chain_tx_hash: Option<String>,
    pub(crate) chain_block_number: Option<u64>,
    #[serde(default)]
    pub(crate) chain_status: MultisigChainStatus,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub(crate) struct GenerateMultisigSfidInput {
    pub(crate) a3: String,
    pub(crate) p1: Option<String>,
    pub(crate) province: Option<String>,
    pub(crate) city: String,
    pub(crate) institution: String,
    pub(crate) institution_name: String,
}

#[derive(Serialize)]
pub(crate) struct GenerateMultisigSfidOutput {
    pub(crate) site_sfid: String,
    pub(crate) chain_status: MultisigChainStatus,
    pub(crate) chain_tx_hash: Option<String>,
    pub(crate) chain_block_number: Option<u64>,
}

#[derive(Serialize)]
pub(crate) struct MultisigSfidListRow {
    pub(crate) site_sfid: String,
    pub(crate) a3: String,
    pub(crate) institution_code: String,
    pub(crate) institution_name: String,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) province_code: String,
    pub(crate) chain_status: MultisigChainStatus,
    pub(crate) chain_tx_hash: Option<String>,
    pub(crate) chain_block_number: Option<u64>,
    pub(crate) created_by: String,
    pub(crate) created_by_name: String,
    pub(crate) created_at: String,
}

#[derive(Serialize)]
pub(crate) struct MultisigSfidListOutput {
    pub(crate) total: usize,
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) rows: Vec<MultisigSfidListRow>,
}
