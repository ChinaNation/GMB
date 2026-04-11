// 中文注释:Phase 2 Day 1 —— 分片数据结构定义。
//
// 本文件只定义 StoreShard(每省一份)和 GlobalShard(跨省共享)两个
// 结构体。它们对应 impl.md 第 3.3 / 3.4 节的字段清单。字段类型统一
// 引用 `crate::models::*` / `crate::institutions::*` 下已有的类型,
// 不新建任何数据结构、不改动 Phase 1 代码。
//
// 这两个结构体必须:
//   1. `Serialize + Deserialize`:走 Postgres JSONB 持久化;
//   2. `Default`:允许从无到有按需创建空分片;
//   3. `Clone + Debug`:方便测试快照、日志打印。
//
// 注意:`cpms_pending_registrations` 字段在当前代码里没有独立类型,
// impl.md 的伪代码里虽然列出了它,但实际 Store 中并不存在
// `PendingCpmsRegistration`。Day 1 按「实际代码为准」的铁律,
// 暂不加该字段,等 Day 3 handler 改造时若确实需要再回填。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::institutions::{MultisigAccount, MultisigInstitution};
use crate::key_admins::chain_keyring::ChainKeyringState;
use crate::login::{AdminSession, LoginChallenge, QrLoginResultRecord};
use crate::models::{
    AdminUser, AuditLogEntry, BindCallbackJob, ChainRequestReceipt, CitizenBindChallenge,
    CitizenRecord, CitizenStatus, CpmsSiteKeys, ImportedArchive, KeyringRotateChallenge,
    PendingBindScan, RewardStateRecord, ServiceMetrics, VoteVerifyCacheEntry,
};

/// 省级分片:按 province 名切分的业务数据。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct StoreShard {
    /// 分片 key(省名)
    pub(crate) province: String,

    // ── 本省管理员(仅 ShiAdmin;ShengAdmin/KeyAdmin 本体在 GlobalShard)──
    pub(crate) local_admins: HashMap<String, AdminUser>,

    // ── 本省机构(两层模型)──
    pub(crate) multisig_institutions: HashMap<String, MultisigInstitution>,
    pub(crate) multisig_accounts: HashMap<String, MultisigAccount>,

    // ── 本省 CPMS 站点 ──
    pub(crate) cpms_site_keys: HashMap<String, CpmsSiteKeys>,

    // ── 本省 citizen 记录 ──
    pub(crate) next_citizen_id: u64,
    pub(crate) citizen_records: HashMap<u64, CitizenRecord>,
    pub(crate) citizen_id_by_pubkey: HashMap<String, u64>,
    pub(crate) citizen_id_by_archive_no: HashMap<String, u64>,
    pub(crate) pubkey_by_archive_index: HashMap<String, String>,

    // ── 本省 citizen 绑定流程 ──
    pub(crate) citizen_bind_challenges: HashMap<String, CitizenBindChallenge>,
    pub(crate) pending_bind_scan_by_qr_id: HashMap<String, PendingBindScan>,

    // ── 本省档案导入 ──
    pub(crate) imported_archives: HashMap<String, ImportedArchive>,
    pub(crate) pending_status_by_archive_no: HashMap<String, CitizenStatus>,

    // ── 本省 SFID 生成历史 ──
    pub(crate) generated_sfid_by_pubkey: HashMap<String, String>,

    // ── 本省回调任务 ──
    pub(crate) bind_callback_jobs: Vec<BindCallbackJob>,

    // ── 本省奖励状态 ──
    pub(crate) reward_state_by_pubkey: HashMap<String, RewardStateRecord>,

    /// 版本号:每次 write_province 递增,用于冲突检测与持久化比对。
    pub(crate) version: u64,
}

/// 全局分片:跨省共享、登录路由、审计与幂等池等。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct GlobalShard {
    // ── KEY_ADMIN 密钥环状态 ──
    pub(crate) chain_keyring_state: Option<ChainKeyringState>,
    pub(crate) keyring_rotate_challenges: HashMap<String, KeyringRotateChallenge>,

    /// 全局管理员索引:KeyAdmin + ShengAdmin 本体(含 encrypted_signing_privkey)。
    /// ShiAdmin 不进这里,存在对应省的 `StoreShard.local_admins`。
    pub(crate) global_admins: HashMap<String, AdminUser>,

    /// 省份路由索引:ShengAdmin pubkey → province。登录后用它快速把请求路由到分片。
    pub(crate) sheng_admin_province_by_pubkey: HashMap<String, String>,

    // ── 登录 challenge + session ──
    pub(crate) login_challenges: HashMap<String, LoginChallenge>,
    pub(crate) qr_login_results: HashMap<String, QrLoginResultRecord>,
    pub(crate) admin_sessions: HashMap<String, AdminSession>,

    // ── 全局幂等池 ──
    pub(crate) consumed_qr_ids: HashMap<String, DateTime<Utc>>,
    pub(crate) consumed_cpms_register_tokens: HashMap<String, DateTime<Utc>>,

    // ── 审计日志(大表,将来可能迁 ClickHouse)──
    pub(crate) audit_logs: Vec<AuditLogEntry>,

    // ── 链请求幂等 ──
    pub(crate) chain_requests_by_key: HashMap<String, ChainRequestReceipt>,
    pub(crate) chain_nonce_seen: HashMap<String, DateTime<Utc>>,

    /// RSABSSA 匿名证书签发 RSA 私钥 PEM(启动时自动生成并持久化)。
    pub(crate) anon_rsa_private_key_pem: Option<String>,

    // ── 清理时间戳 ──
    pub(crate) chain_auth_last_cleanup_at: Option<DateTime<Utc>>,
    pub(crate) pending_bind_last_cleanup_at: Option<DateTime<Utc>>,

    // ── 服务指标 ──
    pub(crate) metrics: ServiceMetrics,

    // ── 全局递增计数器 ──
    pub(crate) next_seq: u64,
    pub(crate) next_audit_seq: u64,
    pub(crate) next_admin_user_id: u64,

    // ── 投票资格缓存 ──
    pub(crate) vote_verify_cache: HashMap<String, VoteVerifyCacheEntry>,

    /// 版本号:每次 write_global 递增。
    pub(crate) version: u64,
}
