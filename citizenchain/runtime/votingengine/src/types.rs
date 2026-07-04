//! 投票引擎核心数据结构与常量定义。
//!
//! 包括:提案 ID 别名、提案/投票各类计数 struct、提案状态阶段常量、
//! 内部提案互斥锁结构、执行重试状态、清理阶段枚举等。

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::{BoundedVec, ConstU32};
use scale_info::TypeInfo;

pub const PROPOSAL_KIND_INTERNAL: u8 = 0;
pub const PROPOSAL_KIND_JOINT: u8 = 1;
/// 立法投票 pallet 的提案类型(ADR-027)。立法机构专属:单院/两院内部表决 + 特别案强制公投。
pub const PROPOSAL_KIND_LEGISLATION: u8 = 2;
/// 选举投票 pallet 的提案类型。只承载选举流程,职位规则由业务模块/选举法解释后传入快照。
pub const PROPOSAL_KIND_ELECTION: u8 = 3;
// 机构码治理分类(全链唯一真源 = primitives::cid::code,见铁律)
// 治理统一用 CID 机构码 [u8; 4]。
// 这里整体 re-export,使 internal-vote / joint-vote / 业务 pallet 仍可写
// `votingengine::types::{InstitutionCode, NRC, fixed_governance_pass_threshold, ...}`。
pub use primitives::cid::code::{
    code_bytes, fixed_governance_pass_threshold, institution_code_from_cid_number,
    is_fixed_governance_code, is_institution_code, is_personal_code, is_public_legal_code,
    is_registered_multisig_code, is_valid_governance_code, InstitutionCode, FRG, NJD, NRC, PMUL,
    PRB, PRC,
};

/// 机构 CID 号链上固定上限。所有机构类主体以 CID 作为唯一身份真源。
pub type CidNumber = BoundedVec<u8, ConstU32<{ primitives::core_const::CID_NUMBER_MAX_BYTES }>>;

/// 单个提案最多关联的机构 CID 数。联合投票需要覆盖 NRC + PRC + PRB 全体机构。
pub const MAX_PROPOSAL_SUBJECT_CIDS: u32 = 256;

/// 提案关联机构 CID 集合。个人多签没有 CID,该集合必须为空。
pub type ProposalSubjectCidNumbers = BoundedVec<CidNumber, ConstU32<MAX_PROPOSAL_SUBJECT_CIDS>>;

/// 内部投票 pallet 的 stage(单阶段提案)。
pub const STAGE_INTERNAL: u8 = 0;
/// 联合投票 pallet 的内部投票阶段(jointinternal):国家储委会/省储委会/省储行管理员加权投票。
pub const STAGE_JOINT: u8 = 1;
/// 联合投票 pallet 的联合公投阶段(jointreferendum):内部投票阶段未全票通过或超时进入,
/// 由 CID 持有者按 >50% 严格多数投票。
///
/// 注意:这是联合投票的第二阶段,与独立的 election-vote pallet(pallet_index=24)
/// 是两个不同概念。election-vote pallet 用于选举公职人员(普选 + 机构成员互选)。
pub const STAGE_REFERENDUM: u8 = 2;

/// 立法投票内部表决阶段(legislation-vote,ADR-027):立法机构议员/委员一人一票。
/// 单院 = 一段;两院 = 众议会段→参议会段在本 stage 内顺序推进(by current_house)。
pub const STAGE_LEG_HOUSE: u8 = 10;
/// 立法投票强制公投阶段(legislation-vote,特别案/核心修宪):内部全过后强制进入。
/// 与联合投票 `STAGE_REFERENDUM` 区分,阈值为宪法 ≥70% 参与 + ≥70% 赞成。
pub const STAGE_LEG_REFERENDUM: u8 = 11;
/// 立法投票行政签署阶段(legislation-vote,ADR-027,宪法第45/46/100/106条):
/// 非特别案内部全过后,由机构法定代表人签署——市长(市)/省长(省)/总统(国)。
/// 市行政区单院无救济;省行政区/国家否决或超时进入 `STAGE_LEG_OVERRIDE`。特别案不经此阶段。
pub const STAGE_LEG_SIGN: u8 = 12;
/// 立法投票三人会签救济阶段(省行政区/国家):立法院院长 + 参议长 + 众议长共同签署。
/// 三人全签同意 → 生效;任一否决或会签超时 → 否决。
pub const STAGE_LEG_OVERRIDE: u8 = 13;
/// 立法投票护宪大法官终审阶段(ADR-027 修订 2026-06-25,宪法第21条):**仅修宪(tier=宪法)**。
/// 修宪在现有流程(重要案 总统签署后 / 特别案 公投后)通过后,最后进入护宪大法官表决:
/// 7 名护宪大法官中 4 名及以上赞成 → 生效;未获 4 名及以上赞成或 30 天超时 → 否决。
pub const STAGE_LEG_CONSTITUTION_GUARD: u8 = 14;

/// 选举投票普选阶段(election-vote):公民按行政区/机构范围投票选人。
pub const STAGE_ELECTION_POPULAR: u8 = 20;
/// 选举投票互选阶段(election-vote):机构现任成员/管理员在成员快照内互选。
pub const STAGE_ELECTION_MUTUAL: u8 = 21;

pub const STATUS_VOTING: u8 = 0;
pub const STATUS_PASSED: u8 = 1;
pub const STATUS_REJECTED: u8 = 2;
/// 提案已执行完成（终态）。消费模块在业务逻辑成功后推进到该状态。
pub const STATUS_EXECUTED: u8 = 3;
/// 投票通过但业务执行失败（终态）。只由投票引擎在重试耗尽、超时或业务永久失败时写入。
pub const STATUS_EXECUTION_FAILED: u8 = 4;
// 立法表决阈值(公民宪法第45/46条,ADR-027)。全整数运算,按宪法精确取端点。
// 5 类提案:常规/常规教育/重要/重要教育/特别。教育变体阈值同非教育同级,
// 仅提案机构与表决院路由不同。投票引擎侧用 u8 表决类型解耦,值与 legislation-yuan::VoteType::as_u8 对齐。
/// 常规案(>80% 参与,≥60% 赞成)
pub const LEG_VOTE_REGULAR: u8 = 0;
/// 常规教育案(教委会;阈值同常规案)
pub const LEG_VOTE_REGULAR_EDU: u8 = 1;
/// 重要案(>90% 参与,≥70% 赞成)
pub const LEG_VOTE_MAJOR: u8 = 2;
/// 重要教育案(教委会;阈值同重要案)
pub const LEG_VOTE_MAJOR_EDU: u8 = 3;
/// 特别案(全员参与,≥70% 赞成 + 强制公投)
pub const LEG_VOTE_SPECIAL: u8 = 4;

/// 单部法律最多院数(单院 1 / 两院 2 / 留余量)。立法投票与立法院模块共享此上限(单一真源)。
pub const MAX_LEGISLATION_HOUSES: u32 = 4;

/// 立法内部表决期满计票:按现任议员/委员快照总数 `total` + 已投 `yes`/`no` 判定是否通过。
/// "参与表决"= casted = yes+no;赞成率基数为 casted(参与表决者),非 total。
pub fn legislation_house_final_passed(vote_type: u8, total: u32, yes: u32, no: u32) -> bool {
    let casted = yes.saturating_add(no);
    if total == 0 || casted == 0 {
        return false;
    }
    let (total, yes, casted) = (u64::from(total), u64::from(yes), u64::from(casted));
    match vote_type {
        // 常规案/常规教育案:>80% 参与 且 ≥60% 赞成(参与者基数)
        LEG_VOTE_REGULAR | LEG_VOTE_REGULAR_EDU => {
            casted * 100 > total * 80 && yes * 100 >= casted * 60
        }
        // 重要案/重要教育案:>90% 参与 且 ≥70% 赞成
        LEG_VOTE_MAJOR | LEG_VOTE_MAJOR_EDU => {
            casted * 100 > total * 90 && yes * 100 >= casted * 70
        }
        // 特别案内部:全员参与 且 ≥70% 赞成
        LEG_VOTE_SPECIAL => casted == total && yes * 100 >= total * 70,
        _ => false,
    }
}

/// 立法内部表决提前判定(只做绝对安全的提前决,避免误判):
/// - 全员已投 → 立即按期满规则判定;
/// - 反对票已使赞成不可能达标 → 提前否决;
/// - 其余 → None(继续等票,期满再计)。
pub fn legislation_house_decided(vote_type: u8, total: u32, yes: u32, no: u32) -> Option<bool> {
    let casted = yes.saturating_add(no);
    if total == 0 {
        return Some(false);
    }
    if casted >= total {
        return Some(legislation_house_final_passed(vote_type, total, yes, no));
    }
    let (total_u, no_u) = (u64::from(total), u64::from(no));
    // 反对票上限:超过即赞成永不可能达标(即使剩余全投赞成)。
    let reject_locked = match vote_type {
        // 常规系:需赞成≥60%参与 → 反对>40% 即锁死
        LEG_VOTE_REGULAR | LEG_VOTE_REGULAR_EDU => no_u * 100 > total_u * 40,
        // 重要系:需赞成≥70%参与 → 反对>30% 即锁死
        LEG_VOTE_MAJOR | LEG_VOTE_MAJOR_EDU => no_u * 100 > total_u * 30,
        // 特别案:赞成需≥70%(全员)→ 反对>30% 即锁死
        LEG_VOTE_SPECIAL => no_u * 100 > total_u * 30,
        _ => true,
    };
    if reject_locked {
        Some(false)
    } else {
        None
    }
}

/// 立法公投期满计票(特别案/核心修宪,宪法 ≥70% 参与 + ≥70% 赞成)。
/// `eligible` = 作用域内拥有投票权的公民总数(人口快照);`yes`/`no` = 已投票数。
pub fn legislation_referendum_final_passed(eligible: u64, yes: u64, no: u64) -> bool {
    let casted = yes.saturating_add(no);
    if eligible == 0 || casted == 0 {
        return false;
    }
    // 参与率 ≥70% 且 赞成率(参与者基数)≥70%
    casted.saturating_mul(100) >= eligible.saturating_mul(70)
        && yes.saturating_mul(100) >= casted.saturating_mul(70)
}

/// 业务模块统一执行结果。
///
/// 业务模块只表达“业务动作执行结果”，不再直接改写提案状态。
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
/// `MODULE_TAG` 只用于路由识别，不能作为权限凭据；因此取消必须由真正
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

/// 提案主体键。机构类主体用 CID,个人多签用账户。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ProposalSubject<AccountId> {
    /// 机构类主体:公权、私权、非法人组织等全部以 CID 为唯一身份真源。
    InstitutionCid(CidNumber),
    /// 个人多签主体:个人多签没有 CID,继续以个人多签账户作为主体。
    PersonalAccount(AccountId),
}

/// 同一提案主体下的互斥状态。
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
    pub subject: ProposalSubject<AccountId>,
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

/// 事项模块接入联合投票时，统一由投票引擎创建提案。
/// 人口快照、联合签名、投票资格和计票数据只允许在 votingengine/joint-vote 内处理。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Proposal<BlockNumber, AccountId> {
    /// 提案类型：内部投票/联合投票
    pub kind: u8,
    /// 当前所处投票阶段：内部/联合/公民
    pub stage: u8,
    /// 当前提案状态：投票中/通过/否决
    pub status: u8,
    /// 仅内部投票使用:机构码。该字段只用于分类、路由和规则判断,不是主体身份真源。
    pub internal_code: Option<InstitutionCode>,
    /// 投票/执行账户上下文。机构身份真源见 `subject_cid_numbers`。
    pub account_context: Option<AccountId>,
    /// 机构类提案关联的机构 CID 集合。个人多签没有 CID,该集合为空。
    pub subject_cid_numbers: ProposalSubjectCidNumbers,
    /// 本阶段起始区块
    pub start: BlockNumber,
    /// 本阶段截止区块（超过则超时）
    pub end: BlockNumber,
    /// 联合公投阶段的可投票总人数（由外部资格系统给出）
    pub citizen_eligible_total: u64,
}

impl<BlockNumber, AccountId: Clone> Proposal<BlockNumber, AccountId> {
    /// 返回用于活跃上限和互斥锁的提案主体键。
    pub fn subject_keys(&self) -> sp_std::vec::Vec<ProposalSubject<AccountId>> {
        if !self.subject_cid_numbers.is_empty() {
            return self
                .subject_cid_numbers
                .iter()
                .cloned()
                .map(ProposalSubject::InstitutionCid)
                .collect();
        }
        if self.internal_code == Some(PMUL) {
            if let Some(account) = self.account_context.clone() {
                return sp_std::vec![ProposalSubject::PersonalAccount(account)];
            }
        }
        sp_std::vec::Vec::new()
    }
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
    /// 联合投票公投阶段投票账本(ReferendumVotesByAccount)分块清理。
    JointReferendumVotes,
    /// 立法投票内部表决投票账本(LegHouseVotesByAdmin)分块清理。
    LegislationHouseVotes,
    /// 立法投票公投账本(LegReferendumVotesByAccount)分块清理。
    LegislationReferendumVotes,
    /// 选举投票投票账本(ElectionVotesByVoter)分块清理。
    ElectionVotes,
    /// 选举投票选民快照(ElectionVoters)分块清理。
    ElectionVoters,
    /// 选举投票候选人票数(ElectionCandidateTallies)分块清理。
    ElectionTallies,
    /// 选举投票元数据/候选人/结果等小存储清理。
    ElectionCandidates,
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
