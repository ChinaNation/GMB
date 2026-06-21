//! 投票引擎核心数据结构与常量定义。
//!
//! 包括:提案 ID 别名、提案/投票各类计数 struct、提案状态阶段常量、
//! 内部提案互斥锁结构、执行重试状态、清理阶段枚举等。

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub const PROPOSAL_KIND_INTERNAL: u8 = 0;
pub const PROPOSAL_KIND_JOINT: u8 = 1;

// ──────────────────────────────────────────────────────────────────
// ORG 类型常量(共用,internal-vote / joint-vote / 业务 pallet 都引用)
// ──────────────────────────────────────────────────────────────────

/// 治理机构:国储会
pub const ORG_NRC: u8 = 0;
/// 治理机构:省储会
pub const ORG_PRC: u8 = 1;
/// 治理机构:省储行
pub const ORG_PRB: u8 = 2;
/// 注册多签:个人多签账户(管理员由 admins-change 提供,动态阈值由 internal-vote 保存)
pub const ORG_REN: u8 = 3;
/// 注册多签:公权机构账户(政府/教育/司法/立法/监察)
pub const ORG_PUP: u8 = 4;
/// 注册多签:其他机构账户(公司/银行/基金/...)
pub const ORG_OTH: u8 = 5;

/// 是否为内部投票支持的 org。
pub fn is_valid_org(org: u8) -> bool {
    matches!(
        org,
        ORG_NRC | ORG_PRC | ORG_PRB | ORG_REN | ORG_PUP | ORG_OTH
    )
}

/// 是否为注册多签动态账户 org。
///
/// 中文注释：REN 只代表个人多签；PUP/OTH 代表机构账户，不能互相代替。
pub fn is_registered_multisig_org(org: u8) -> bool {
    matches!(org, ORG_REN | ORG_PUP | ORG_OTH)
}

/// 治理机构(NRC/PRC/PRB)的固定制度阈值。
/// 中文注释:三类治理机构阈值是永久治理常量,不读取注册多签账户配置。
pub fn fixed_governance_pass_threshold(org: u8) -> Option<u32> {
    use primitives::count_const::{
        NRC_INTERNAL_THRESHOLD, PRB_INTERNAL_THRESHOLD, PRC_INTERNAL_THRESHOLD,
    };
    match org {
        ORG_NRC => Some(NRC_INTERNAL_THRESHOLD),
        ORG_PRC => Some(PRC_INTERNAL_THRESHOLD),
        ORG_PRB => Some(PRB_INTERNAL_THRESHOLD),
        _ => None,
    }
}

/// 内部投票 pallet 的 stage(单阶段提案)。
pub const STAGE_INTERNAL: u8 = 0;
/// 联合投票 pallet 的内部投票阶段(jointinternal):国储会/省储会/省储行管理员加权投票。
pub const STAGE_JOINT: u8 = 1;
/// 联合投票 pallet 的联合公投阶段(jointreferendum):内部投票阶段未全票通过或超时进入,
/// 由 SFID 持有者按 >50% 严格多数投票。
///
/// 注意:这是联合投票的第二阶段,与独立的 citizen-vote pallet(pallet_index=24)
/// 是两个不同概念。citizen-vote pallet 用于公民选举/公投等多模式投票(Phase 3 接业务)。
pub const STAGE_REFERENDUM: u8 = 2;

pub const STATUS_VOTING: u8 = 0;
pub const STATUS_PASSED: u8 = 1;
pub const STATUS_REJECTED: u8 = 2;
/// 提案已执行完成（终态）。消费模块在业务逻辑成功后推进到该状态。
pub const STATUS_EXECUTED: u8 = 3;
/// 投票通过但业务执行失败（终态）。只由投票引擎在重试耗尽、超时或业务永久失败时写入。
pub const STATUS_EXECUTION_FAILED: u8 = 4;

/// 业务模块统一执行结果。
///
/// 中文注释：业务模块只表达“业务动作执行结果”，不再直接改写提案状态。
/// 投票引擎根据该结果统一维护 PASSED / EXECUTED / EXECUTION_FAILED 状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ProposalExecutionOutcome {
    /// 不是本模块提案。
    Ignored,
    /// 业务执行成功。
    Executed,
    /// 暂时失败，保留 PASSED 并允许管理员手动重试。
    RetryableFailed,
    /// 确定不可执行，进入 EXECUTION_FAILED 终态。
    FatalFailed,
}

/// 业务模块对 `PASSED` 重试提案是否允许管理员提前取消的决策。
///
/// 中文注释：`MODULE_TAG` 只用于路由识别，不能作为权限凭据；因此取消必须由真正
/// 认领该提案的 callback 显式返回 `Allow`，默认实现一律 `Ignored`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ProposalCancelDecision {
    /// 不是本模块提案。
    Ignored,
    /// 本模块确认该提案已不可执行，允许进入 EXECUTION_FAILED 终态。
    Allow,
}

/// 内部提案互斥类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum InternalProposalMutexKind {
    /// 普通内部治理事项，允许同账户多个普通事项并行。
    Regular,
    /// 管理员集合变更，同账户下必须独占。
    AdminSetMutationExclusive,
}

/// 同一治理账户下的互斥状态。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct InternalProposalMutexState {
    /// 当前占用管理员集合变更独占锁的提案。
    pub admin_set_mutation_proposal: Option<u64>,
    /// 当前普通活跃提案数量。
    pub regular_active_count: u32,
}

impl InternalProposalMutexState {
    pub(crate) fn is_empty(&self) -> bool {
        self.admin_set_mutation_proposal.is_none() && self.regular_active_count == 0
    }
}

/// proposal_id 到互斥锁的反向绑定，用于终态/阶段切换时释放锁。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct InternalProposalMutexBinding<AccountId> {
    pub org: u8,
    pub institution: AccountId,
    pub kind: InternalProposalMutexKind,
}

/// 自动执行失败后的统一重试状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ExecutionRetryState<BlockNumber> {
    /// 已失败的手动执行次数。自动执行失败不计入该次数。
    pub manual_attempts: u8,
    /// 第一次自动执行失败所在区块。
    pub first_auto_failed_at: BlockNumber,
    /// 超过该区块仍未执行成功，则自动转 EXECUTION_FAILED。
    pub retry_deadline: BlockNumber,
    /// 最近一次手动执行尝试所在区块。
    pub last_attempt_at: Option<BlockNumber>,
}

/// 中文注释：事项模块接入联合投票时，统一由投票引擎创建提案。
/// 人口快照、联合签名、投票资格和计票数据只允许在 votingengine/joint-vote 内处理。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Proposal<BlockNumber, AccountId> {
    /// 提案类型：内部投票/联合投票
    pub kind: u8,
    /// 当前所处投票阶段：内部/联合/公民
    pub stage: u8,
    /// 当前提案状态：投票中/通过/否决
    pub status: u8,
    /// 仅内部投票使用：机构类型（国储会/省储会/省储行）
    pub internal_org: Option<u8>,
    /// 仅内部投票使用：多签账户（全链唯一）
    pub internal_institution: Option<AccountId>,
    /// 本阶段起始区块
    pub start: BlockNumber,
    /// 本阶段截止区块（超过则超时）
    pub end: BlockNumber,
    /// 联合公投阶段的可投票总人数（由外部资格系统给出）
    pub citizen_eligible_total: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VoteCountU32 {
    /// 赞成票
    pub yes: u32,
    /// 反对票
    pub no: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VoteCountU64 {
    /// 赞成票
    pub yes: u64,
    /// 反对票
    pub no: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum PendingCleanupStage {
    AdminSnapshots,
    InternalVotes,
    JointAdminVotes,
    JointInstitutionVotes,
    JointInstitutionTallies,
    CitizenVotes,
    VoteCredentials,
    /// 清理大对象存储（ProposalObject + ProposalObjectMeta）。
    ProposalObject,
    /// 清理业务数据（ProposalData + ProposalMeta）和核心数据（Proposals + Tallies）。
    /// 这是清理流程的最后一步，单次完成。
    FinalCleanup,
}

/// 提案辅助元数据（由投票引擎统一存储，替代各业务模块的 ProposalCreatedAt / ProposalPassedAt）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ProposalMetadata<BlockNumber> {
    /// 提案创建时的区块号
    pub created_at: BlockNumber,
    /// 提案通过时的区块号（未通过时为 None）
    pub passed_at: Option<BlockNumber>,
}

/// 提案对象层元数据：记录统一对象存储的类型、长度与哈希。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ProposalObjectMetadata<Hash> {
    /// 对象类型，由业务模块自行定义并在解码时识别。
    pub kind: u8,
    /// 对象字节长度，便于链上/链下快速判断对象规模。
    pub object_len: u32,
    /// 对象内容哈希，用于执行和审计时做一致性校验。
    pub object_hash: Hash,
}

/// 提案展示号(双层 ID 设计)。
///
/// 主键 `proposal_id: u64` 是纯单调全局递增,实质无上限。
/// 展示号则按"年份 + 年内序号"组合,与主键解耦,通过单独的
/// `ProposalDisplayId` storage map 反查。
///
/// 客户端渲染走 `2026-#000123` 类格式;链上和后端在创建提案时同步写入。
/// 改展示格式(比如 `2026Q1-001234` 季度制)只动渲染层,主键和存储不动。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ProposalDisplayMeta {
    /// 创建年份(UTC 公历)。
    pub year: u16,
    /// 年内序号(每年从 0 重置)。u32 上限 42.9 亿/年,实质无上限。
    pub seq_in_year: u32,
}
