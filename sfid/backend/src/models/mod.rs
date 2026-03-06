use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::key_admins::chain_keyring::ChainKeyringState;
use crate::key_admins::chain_proof::SignatureEnvelope;
use crate::login::{AdminSession, LoginChallenge, QrLoginResultRecord};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Store {
    pub(crate) next_seq: u64,
    pub(crate) next_audit_seq: u64,
    pub(crate) pending_by_pubkey: HashMap<String, PendingRequest>,
    pub(crate) bindings_by_pubkey: HashMap<String, BindingRecord>,
    pub(crate) pubkey_by_archive_index: HashMap<String, String>,
    pub(crate) admin_users_by_pubkey: HashMap<String, AdminUser>,
    pub(crate) super_admin_province_by_pubkey: HashMap<String, String>,
    #[serde(skip)]
    pub(crate) login_challenges: HashMap<String, LoginChallenge>,
    #[serde(skip)]
    pub(crate) qr_login_results: HashMap<String, QrLoginResultRecord>,
    #[serde(skip)]
    pub(crate) admin_sessions: HashMap<String, AdminSession>,
    pub(crate) cpms_site_keys: HashMap<String, CpmsSiteKeys>,
    pub(crate) consumed_cpms_register_tokens: HashMap<String, DateTime<Utc>>,
    pub(crate) consumed_qr_ids: HashMap<String, DateTime<Utc>>,
    pub(crate) pending_status_by_archive_no: HashMap<String, CitizenStatus>,
    pub(crate) pending_bind_scan_by_qr_id: HashMap<String, PendingBindScan>,
    pub(crate) generated_sfid_by_pubkey: HashMap<String, String>,
    pub(crate) chain_keyring_state: Option<ChainKeyringState>,
    pub(crate) keyring_rotate_challenges: HashMap<String, KeyringRotateChallenge>,
    pub(crate) audit_logs: Vec<AuditLogEntry>,
    pub(crate) chain_requests_by_key: HashMap<String, ChainRequestReceipt>,
    pub(crate) chain_nonce_seen: HashMap<String, DateTime<Utc>>,
    pub(crate) bind_callback_jobs: Vec<BindCallbackJob>,
    pub(crate) reward_state_by_pubkey: HashMap<String, RewardStateRecord>,
    pub(crate) vote_verify_cache: HashMap<String, VoteVerifyCacheEntry>,
    pub(crate) metrics: ServiceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistedRuntimeMeta {
    pub(crate) version: u32,
    pub(crate) signing_seed_hex: String,
    pub(crate) known_key_seeds: HashMap<String, String>,
    pub(crate) public_key_hex: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminRole {
    KeyAdmin,
    SuperAdmin,
    OperatorAdmin,
    QueryOnly,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CpmsSiteKeys {
    pub(crate) site_sfid: String,
    pub(crate) pubkey_1: String,
    pub(crate) pubkey_2: String,
    pub(crate) pubkey_3: String,
    #[serde(default = "default_cpms_site_status")]
    pub(crate) status: CpmsSiteStatus,
    #[serde(default = "default_cpms_site_version")]
    pub(crate) version: u64,
    #[serde(default)]
    pub(crate) last_register_issued_at: i64,
    #[serde(default)]
    pub(crate) init_qr_payload: Option<String>,
    pub(crate) admin_province: String,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) updated_by: Option<String>,
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
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
}

#[derive(Serialize)]
pub(crate) struct SuperAdminRow {
    pub(crate) id: u64,
    pub(crate) province: String,
    pub(crate) admin_pubkey: String,
    pub(crate) status: AdminStatus,
    pub(crate) built_in: bool,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub(crate) struct CreateOperatorInput {
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
}

#[derive(Deserialize)]
pub(crate) struct ReplaceSuperAdminInput {
    pub(crate) admin_pubkey: String,
}

#[derive(Deserialize)]
pub(crate) struct UpdateOperatorInput {
    pub(crate) admin_pubkey: Option<String>,
    pub(crate) admin_name: Option<String>,
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
}

#[derive(Deserialize)]
pub(crate) struct UpdateCpmsKeysInput {
    pub(crate) pubkey_1: String,
    pub(crate) pubkey_2: String,
    pub(crate) pubkey_3: String,
}

#[derive(Deserialize)]
pub(crate) struct UpdateCpmsSiteStatusInput {
    pub(crate) reason: Option<String>,
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

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CpmsRegisterQrPayload {
    pub(crate) site_sfid: String,
    pub(crate) pubkey_1: String,
    pub(crate) pubkey_2: String,
    pub(crate) pubkey_3: String,
    pub(crate) issued_at: i64,
    pub(crate) checksum_or_signature: String,
    pub(crate) init_qr_payload: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CpmsInstitutionInitQrPayload {
    pub(crate) ver: String,
    pub(crate) issuer_id: String,
    pub(crate) purpose: String,
    pub(crate) site_sfid: String,
    pub(crate) a3: String,
    pub(crate) p1: String,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) institution: String,
    pub(crate) issued_at: i64,
    pub(crate) expire_at: i64,
    pub(crate) qr_id: String,
    pub(crate) sig_alg: String,
    pub(crate) key_id: String,
    pub(crate) key_version: String,
    pub(crate) public_key: String,
    pub(crate) signature: String,
}

#[derive(Serialize)]
pub(crate) struct CpmsRegisterScanOutput {
    pub(crate) site_sfid: String,
    pub(crate) status: &'static str,
    pub(crate) message: &'static str,
}

#[derive(Serialize)]
pub(crate) struct GenerateCpmsInstitutionSfidOutput {
    pub(crate) site_sfid: String,
    pub(crate) issued_at: i64,
    pub(crate) expire_at: i64,
    pub(crate) qr_payload: String,
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
    pub(crate) account_pubkey: String,
    pub(crate) is_bound: bool,
    pub(crate) sfid_code: Option<String>,
    pub(crate) sfid_signature: Option<String>,
    pub(crate) message: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct VoteVerifyInput {
    pub(crate) account_pubkey: String,
    pub(crate) proposal_id: Option<u64>,
    pub(crate) challenge: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct VoteVerifyOutput {
    pub(crate) account_pubkey: String,
    pub(crate) is_bound: bool,
    pub(crate) has_vote_eligibility: bool,
    pub(crate) sfid_code: Option<String>,
    pub(crate) vote_token: Option<SignatureEnvelope>,
    pub(crate) message: String,
}

#[derive(Serialize)]
pub(crate) struct ChainVotersCountOutput {
    pub(crate) total_voters: usize,
    pub(crate) as_of: i64,
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

#[derive(Serialize)]
pub(crate) struct VotePayload {
    pub(crate) kind: &'static str,
    pub(crate) version: &'static str,
    pub(crate) account_pubkey: String,
    pub(crate) sfid_code: String,
    pub(crate) proposal_id: Option<u64>,
    pub(crate) challenge: String,
    pub(crate) iat: i64,
    pub(crate) exp: i64,
    pub(crate) jti: String,
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
