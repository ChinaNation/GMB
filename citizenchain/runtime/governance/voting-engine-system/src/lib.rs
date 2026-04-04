//! # 投票引擎系统 (voting-engine-system)
//!
//! 治理投票基础设施模块，统一承载三类投票流程：
//! - **内部投票**（INTERNAL）：机构内部管理员按阈值投票。
//! - **联合机构投票**（JOINT）：国储会/省储会/省储行管理员按票权加权投票。
//! - **公民投票**（CITIZEN）：SFID 持有者按 >50% 严格多数投票。
//!
//! 通过 trait 为上层治理模块提供标准化能力：
//! - `InternalVoteEngine` / `JointVoteEngine`：提案创建和投票。
//! - `JointVoteResultCallback`：联合提案终结后回调目标治理模块。
//! - 自动超时结算、原子终结+回调一致性、90 天延迟分块清理。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod active_proposal_limit;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod citizen_vote;
pub mod internal_vote;
pub mod joint_vote;
pub mod proposal_cleanup;
pub mod weights;

pub use citizen_vote::{SfidEligibility, VoteCredentialCleanup};
pub use internal_vote::ORG_DUOQIAN;
pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

pub type InstitutionPalletId = [u8; 48];

/// 国储会 InstitutionPalletId（从 CHINA_CB 第一条记录派生）。
/// 公共函数，供 internal_vote、joint_vote 等子模块共用。
pub fn nrc_pallet_id_bytes() -> Option<InstitutionPalletId> {
    use primitives::china::china_cb::{
        shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
    };
    CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
}

pub const PROPOSAL_KIND_INTERNAL: u8 = 0;
pub const PROPOSAL_KIND_JOINT: u8 = 1;

pub const STAGE_INTERNAL: u8 = 0;
pub const STAGE_JOINT: u8 = 1;
pub const STAGE_CITIZEN: u8 = 2;

pub const STATUS_VOTING: u8 = 0;
pub const STATUS_PASSED: u8 = 1;
pub const STATUS_REJECTED: u8 = 2;
/// 提案已执行完成（终态）。消费模块在业务逻辑成功后调用 set_status_and_emit 设置。
pub const STATUS_EXECUTED: u8 = 3;
/// 投票通过但业务执行失败（终态）。由消费模块回调在 set_status_and_emit 事务内覆盖。
pub const STATUS_EXECUTION_FAILED: u8 = 4;

/// 中文注释：事项模块接入联合投票时，统一由投票引擎创建提案并写入人口快照。
pub trait JointVoteEngine<AccountId> {
    fn create_joint_proposal(
        who: AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
    ) -> Result<u64, DispatchError>;

    fn cleanup_joint_proposal(_proposal_id: u64) {}
}

/// 中文注释：事项模块接入内部投票时，统一由投票引擎创建提案并返回真实提案ID。
pub trait InternalVoteEngine<AccountId> {
    fn create_internal_proposal(
        who: AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError>;

    fn cast_internal_vote(
        who: AccountId,
        proposal_id: u64,
        approve: bool,
    ) -> Result<(), DispatchError>;

    fn cleanup_internal_proposal(_proposal_id: u64) {}
}

impl<AccountId> InternalVoteEngine<AccountId> for () {
    fn create_internal_proposal(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("InternalVoteEngineNotConfigured"))
    }

    fn cast_internal_vote(
        _who: AccountId,
        _proposal_id: u64,
        _approve: bool,
    ) -> Result<(), DispatchError> {
        Err(DispatchError::Other("InternalVoteEngineNotConfigured"))
    }
}

/// 中文注释：公民总人口快照验签接口（由 runtime 对接 SFID 系统）。
pub trait PopulationSnapshotVerifier<AccountId, Nonce, Signature> {
    fn verify_population_snapshot(
        who: &AccountId,
        eligible_total: u64,
        nonce: &Nonce,
        signature: &Signature,
    ) -> bool;
}

impl<AccountId, Nonce, Signature> PopulationSnapshotVerifier<AccountId, Nonce, Signature> for () {
    fn verify_population_snapshot(
        _who: &AccountId,
        _eligible_total: u64,
        _nonce: &Nonce,
        _signature: &Signature,
    ) -> bool {
        false
    }
}

pub trait JointVoteResultCallback {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult;
}

impl JointVoteResultCallback for () {
    fn on_joint_vote_finalized(_vote_proposal_id: u64, _approved: bool) -> DispatchResult {
        Ok(())
    }
}

/// 中文注释：内部管理员动态提供器（可由其他治理模块提供最新管理员集合）。
pub trait InternalAdminProvider<AccountId> {
    fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId) -> bool;
}

impl<AccountId> InternalAdminProvider<AccountId> for () {
    fn is_internal_admin(_org: u8, _institution: InstitutionPalletId, _who: &AccountId) -> bool {
        false
    }
}

/// 内部管理员总人数提供器。
/// 联合投票会根据“剩余管理员数是否还能让赞成票达到阈值”来自动判定机构反对。
pub trait InternalAdminCountProvider {
    fn admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32>;
}

impl InternalAdminCountProvider for () {
    fn admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32> {
        match org {
            internal_vote::ORG_NRC | internal_vote::ORG_PRC => {
                use primitives::china::china_cb::{
                    shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
                };
                CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok())
            }
            internal_vote::ORG_PRB => {
                use primitives::china::china_ch::{
                    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
                };
                CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok())
            }
            _ => None,
        }
    }
}

/// 内部投票阈值动态提供器。
/// 治理机构（NRC/PRC/PRB）返回硬编码阈值，注册多签机构（ORG_DUOQIAN）从链上存储动态读取。
pub trait InternalThresholdProvider {
    fn pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32>;
}

/// 默认实现：仅支持治理机构的硬编码阈值。
impl InternalThresholdProvider for () {
    fn pass_threshold(org: u8, _institution: InstitutionPalletId) -> Option<u32> {
        internal_vote::governance_org_pass_threshold(org)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Proposal<BlockNumber> {
    /// 提案类型：内部投票/联合投票
    pub kind: u8,
    /// 当前所处投票阶段：内部/联合/公民
    pub stage: u8,
    /// 当前提案状态：投票中/通过/否决
    pub status: u8,
    /// 仅内部投票使用：机构类型（国储会/省储会/省储行）
    pub internal_org: Option<u8>,
    /// 仅内部投票使用：机构 shenfen_id 标识（全链唯一）
    pub internal_institution: Option<InstitutionPalletId>,
    /// 本阶段起始区块
    pub start: BlockNumber,
    /// 本阶段截止区块（超过则超时）
    pub end: BlockNumber,
    /// 公民投票阶段的可投票总人数（由外部资格系统给出）
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

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::UnixTime;
    use frame_support::{
        pallet_prelude::*,
        storage::{with_transaction, TransactionOutcome},
        Blake2_128Concat,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Hash, One, Saturating};
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxVoteNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxVoteSignatureLength: Get<u32>;

        /// 每个区块自动处理的“到期提案”上限，避免 on_initialize 无界增长。
        #[pallet::constant]
        type MaxAutoFinalizePerBlock: Get<u32>;

        /// 单个到期区块允许挂载的提案 ID 上限，避免 expiry 桶无界增长。
        #[pallet::constant]
        type MaxProposalsPerExpiry: Get<u32>;

        /// 每个区块最多执行多少个清理步骤，避免历史提案清理拖垮 on_initialize。
        #[pallet::constant]
        type MaxCleanupStepsPerBlock: Get<u32>;

        /// 每个清理步骤最多删除多少条前缀项。
        #[pallet::constant]
        type CleanupKeysPerStep: Get<u32>;

        /// 提案业务数据最大长度（字节），各业务模块序列化后的数据不超过此限制。
        #[pallet::constant]
        type MaxProposalDataLen: Get<u32>;

        /// 提案大对象数据最大长度（字节），用于 runtime wasm 等大载荷。
        #[pallet::constant]
        type MaxProposalObjectLen: Get<u32>;

        type SfidEligibility: SfidEligibility<Self::AccountId, Self::Hash>;
        type PopulationSnapshotVerifier: PopulationSnapshotVerifier<
            Self::AccountId,
            VoteNonceOf<Self>,
            VoteSignatureOf<Self>,
        >;

        type JointVoteResultCallback: JointVoteResultCallback;
        type InternalAdminProvider: InternalAdminProvider<Self::AccountId>;
        type InternalAdminCountProvider: InternalAdminCountProvider;
        /// 内部投票阈值动态提供器（治理机构硬编码，注册多签动态读取）。
        type InternalThresholdProvider: InternalThresholdProvider;

        /// 时间源，用于提案 ID 编码年份。
        type TimeProvider: frame_support::traits::UnixTime;

        type WeightInfo: crate::weights::WeightInfo;
    }

    use crate::weights::WeightInfo;

    pub type VoteNonceOf<T> = BoundedVec<u8, <T as Config>::MaxVoteNonceLength>;
    pub type VoteSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxVoteSignatureLength>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 当前提案年份（用于年度计数器重置）。
    #[pallet::storage]
    pub type CurrentProposalYear<T> = StorageValue<_, u16, ValueQuery>;

    /// 当前年份内的提案计数器（每年从 0 开始）。
    #[pallet::storage]
    pub type YearProposalCounter<T> = StorageValue<_, u32, ValueQuery>;

    /// 兼容性别名：返回下一个 proposal_id（年份 × 1,000,000 + 计数器）。
    /// 仅供外部查询使用（如 App 扫描提案范围）。
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, u64, ValueQuery>;

    /// 全局提案表：proposal_id → 提案元数据（类型/阶段/状态/起止区块/机构等）。
    /// 由 `create_internal_proposal` 写入，`set_status_and_emit` 更新状态，超时清理自动删除。
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, Proposal<BlockNumberFor<T>>, OptionQuery>;

    /// 以“阶段截止区块”索引提案，用于 on_initialize 自动超时结算。
    #[pallet::storage]
    #[pallet::getter(fn proposals_by_expiry)]
    pub type ProposalsByExpiry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<u64, <T as Config>::MaxProposalsPerExpiry>,
        ValueQuery,
    >;

    /// 自动结算游标：记录上个区块未处理完的过期桶。
    #[pallet::storage]
    #[pallet::getter(fn pending_expiry_bucket)]
    pub type PendingExpiryBucket<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

    /// 分块清理游标：按提案维度逐步清理历史投票状态，避免 finalize 路径单次无界删除。
    #[pallet::storage]
    #[pallet::getter(fn pending_cleanup_stage)]
    pub type PendingProposalCleanups<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, PendingCleanupStage, OptionQuery>;

    /// 内部投票记录：(proposal_id, 管理员公钥) → 赞成/反对。防止同一管理员重复投票。
    #[pallet::storage]
    pub type InternalVotesByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn internal_tally)]
    pub type InternalTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU32, ValueQuery>;

    /// 联合投票——管理员级记录：(proposal_id, (机构, 管理员公钥)) → 赞成/反对。
    /// 防止同一管理员在同一机构内重复投票。
    #[pallet::storage]
    pub type JointVotesByAdmin<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        (InstitutionPalletId, T::AccountId),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn joint_institution_tally)]
    pub type JointInstitutionTallies<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        InstitutionPalletId,
        VoteCountU32,
        ValueQuery,
    >;

    /// 联合投票——机构级汇总：(proposal_id, 机构) → 该机构内部投票的最终结果（赞成/反对）。
    /// 机构内部达到阈值后写入，用于联合阶段权重汇总。
    #[pallet::storage]
    pub type JointVotesByInstitution<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        InstitutionPalletId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn joint_tally)]
    pub type JointTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU32, ValueQuery>;

    /// 公民投票记录：(proposal_id, 公民身份绑定哈希) → 赞成/反对。
    /// 每个公民身份只能投一次，由绑定哈希防重。
    #[pallet::storage]
    pub type CitizenVotesByBindingId<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, T::Hash, bool, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn citizen_tally)]
    pub type CitizenTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU64, ValueQuery>;

    /// 中文注释：总人口快照 nonce 防重放（全局维度，防止跨提案重放）。
    #[pallet::storage]
    #[pallet::getter(fn used_population_snapshot_nonce)]
    pub type UsedPopulationSnapshotNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 提案业务数据（由各业务模块序列化后写入，投票引擎统一存储和清理）。
    #[pallet::storage]
    #[pallet::getter(fn proposal_data)]
    pub type ProposalData<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BoundedVec<u8, T::MaxProposalDataLen>, OptionQuery>;

    /// 提案对象层元数据（对象类型 / 长度 / 哈希），由投票引擎统一存储和清理。
    #[pallet::storage]
    #[pallet::getter(fn proposal_object_meta)]
    pub type ProposalObjectMeta<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ProposalObjectMetadata<T::Hash>, OptionQuery>;

    /// 提案对象层原始数据（例如 runtime wasm），由投票引擎统一存储和清理。
    #[pallet::storage]
    #[pallet::getter(fn proposal_object)]
    pub type ProposalObject<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BoundedVec<u8, T::MaxProposalObjectLen>, OptionQuery>;

    /// 提案辅助元数据（创建时间、通过时间，由投票引擎统一存储和清理）。
    #[pallet::storage]
    #[pallet::getter(fn proposal_meta)]
    pub type ProposalMeta<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ProposalMetadata<BlockNumberFor<T>>, OptionQuery>;

    /// 延迟清理队列：按清理到期区块索引待清理的 proposal_id 列表。
    #[pallet::storage]
    pub type CleanupQueue<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<u64, ConstU32<50>>,
        ValueQuery,
    >;

    /// 每个机构的活跃提案 ID 列表（全局管控，不区分提案类型，上限 10 个）。
    #[pallet::storage]
    #[pallet::getter(fn active_proposals_by_institution)]
    pub type ActiveProposalsByInstitution<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        InstitutionPalletId,
        BoundedVec<u64, ConstU32<{ crate::active_proposal_limit::MAX_ACTIVE_PROPOSALS }>>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 中文注释：提案已创建，记录类型、阶段和截止区块。
        ProposalCreated {
            proposal_id: u64,
            kind: u8,
            stage: u8,
            end: BlockNumberFor<T>,
        },
        /// 中文注释：联合投票阶段非全票通过或超时，提案推进到公民投票阶段。
        ProposalAdvancedToCitizen {
            proposal_id: u64,
            citizen_end: BlockNumberFor<T>,
            eligible_total: u64,
        },
        /// 中文注释：提案终结，status 为最终状态（PASSED/REJECTED/EXECUTED/EXECUTION_FAILED）。
        ProposalFinalized {
            proposal_id: u64,
            status: u8,
        },
        /// 中文注释：内部投票已投出一票。
        InternalVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 中文注释：联合投票中某机构管理员已投出一票。
        JointAdminVoteCast {
            proposal_id: u64,
            institution: InstitutionPalletId,
            who: T::AccountId,
            approve: bool,
        },
        /// 中文注释：联合投票中某机构已形成最终结果（赞成/反对）。
        JointInstitutionVoteFinalized {
            proposal_id: u64,
            institution: InstitutionPalletId,
            approved: bool,
        },
        /// 中文注释：公民投票已投出一票（binding_id 为 SFID 哈希）。
        CitizenVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            binding_id: T::Hash,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 中文注释：提案不存在或已被清理。
        ProposalNotFound,
        /// 中文注释：提案类型与当前操作不匹配（内部/联合）。
        InvalidProposalKind,
        /// 中文注释：提案所处阶段与当前操作不匹配（内部/联合/公民）。
        InvalidProposalStage,
        /// 中文注释：提案状态不允许当前操作（例如已终结的提案不可投票）。
        InvalidProposalStatus,
        /// 中文注释：内部投票的机构类型不合法。
        InvalidInternalOrg,
        /// 中文注释：机构标识不属于任何已知类型（NRC/PRC/PRB/多签）。
        InvalidInstitution,
        /// 中文注释：调用者无权执行此操作（非管理员或外部 extrinsic 直接调用）。
        NoPermission,
        /// 中文注释：投票已截止（当前区块 > end）。
        VoteClosed,
        /// 中文注释：提案尚未到期，不可手动触发超时结算。
        VoteNotExpired,
        /// 中文注释：同一身份已对该提案投过票。
        AlreadyVoted,
        /// 中文注释：SFID 资格校验未通过（binding_id 未绑定或不匹配）。
        SfidNotEligible,
        /// 中文注释：SFID 投票凭证验签失败或已被消费。
        InvalidSfidVoteCredential,
        /// 中文注释：公民投票总分母未设置（eligible_total == 0）。
        CitizenEligibleTotalNotSet,
        /// 中文注释：人口快照参数无效（nonce 为空/已使用/签名验证失败）。
        InvalidPopulationSnapshot,
        /// 中文注释：提案已终结，不可重复结算。
        ProposalAlreadyFinalized,
        /// 中文注释：提案 ID 分配溢出（年内超过 999,999 或数学溢出）。
        ProposalIdOverflow,
        /// 中文注释：单个到期区块的提案数超出上限。
        TooManyProposalsAtExpiry,
        /// 中文注释：该机构活跃提案数已达上限（10 个），需等待现有提案完成后再发起。
        ActiveProposalLimitReached,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::zero();
            let max_auto_finalize = T::MaxAutoFinalizePerBlock::get() as usize;

            if max_auto_finalize > 0 {
                let db_weight = T::DbWeight::get();
                weight = weight.saturating_add(db_weight.reads(1));
                let mut budget = max_auto_finalize;
                let pending = PendingExpiryBucket::<T>::get();

                if let Some(expiry) = pending {
                    if expiry <= n {
                        let (processed, has_remaining, processed_weight) =
                            Self::auto_finalize_expiry_bucket(expiry, n, budget);
                        weight = weight.saturating_add(processed_weight);
                        budget = budget.saturating_sub(processed);
                        if has_remaining {
                            PendingExpiryBucket::<T>::put(expiry);
                            weight = weight.saturating_add(db_weight.writes(1));
                            return weight.saturating_add(Self::process_pending_cleanup_steps());
                        }
                        PendingExpiryBucket::<T>::kill();
                        weight = weight.saturating_add(db_weight.writes(1));
                    }
                }

                if budget > 0 {
                    let (_processed, has_remaining, processed_weight) =
                        Self::auto_finalize_expiry_bucket(n, n, budget);
                    weight = weight.saturating_add(processed_weight);
                    if has_remaining {
                        PendingExpiryBucket::<T>::put(n);
                        weight = weight.saturating_add(db_weight.writes(1));
                    }
                }
            }

            weight = weight.saturating_add(Self::process_pending_cleanup_steps());

            // 处理延迟清理队列：清理 90 天前完成的提案的全部数据
            weight = weight.saturating_add(proposal_cleanup::process_cleanup_queue::<T>(n));

            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads(1))] // 非零 weight，防止零成本 spam
        pub fn create_internal_proposal(
            origin: OriginFor<T>,
            _org: u8,
            _institution: InstitutionPalletId,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            // 中文注释：内部提案只能由业务模块经 InternalVoteEngine trait 创建，
            // 禁止直接发 extrinsic 绕过业务动作映射和上层副作用。
            Err(Error::<T>::NoPermission.into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads(1))] // 非零 weight，防止零成本 spam
        pub fn create_joint_proposal(
            origin: OriginFor<T>,
            _eligible_total: u64,
            _snapshot_nonce: VoteNonceOf<T>,
            _signature: VoteSignatureOf<T>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            // 中文注释：联合投票提案只能由事项模块通过 JointVoteEngine trait 创建；
            // 禁止外部直接调用，避免产生“无事项映射”的悬空联合提案。
            Err(Error::<T>::NoPermission.into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 4))]
        pub fn internal_vote(
            origin: OriginFor<T>,
            _proposal_id: u64,
            _approve: bool,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            // 中文注释：内部投票只能由事项模块通过 InternalVoteEngine trait 转发。
            Err(Error::<T>::NoPermission.into())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::joint_vote())]
        pub fn joint_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            institution: InstitutionPalletId,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_joint_vote(who, proposal_id, institution, approve)
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::citizen_vote())]
        pub fn citizen_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            binding_id: T::Hash,
            nonce: VoteNonceOf<T>,
            signature: VoteSignatureOf<T>,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_citizen_vote(who, proposal_id, binding_id, nonce, signature, approve)
        }

        #[pallet::call_index(5)]
        #[pallet::weight(
            T::WeightInfo::finalize_proposal_internal()
                .max(T::WeightInfo::finalize_proposal_joint())
                .max(T::WeightInfo::finalize_proposal_citizen())
        )]
        pub fn finalize_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            let actual_weight = match proposal.stage {
                STAGE_INTERNAL => {
                    Self::do_finalize_internal_timeout(&proposal, proposal_id)?;
                    T::WeightInfo::finalize_proposal_internal()
                }
                STAGE_JOINT => {
                    Self::do_finalize_joint_timeout(&proposal, proposal_id)?;
                    T::WeightInfo::finalize_proposal_joint()
                }
                STAGE_CITIZEN => {
                    Self::do_finalize_citizen_timeout(&proposal, proposal_id)?;
                    T::WeightInfo::finalize_proposal_citizen()
                }
                _ => return Err(Error::<T>::InvalidProposalStage.into()),
            };

            Ok(Some(actual_weight).into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn schedule_proposal_expiry(
            proposal_id: u64,
            end: BlockNumberFor<T>,
        ) -> DispatchResult {
            // end 表示“最后一个仍可投票区块”，因此超时结算应在 end+1 触发。
            let expiry = end.saturating_add(One::one());
            ProposalsByExpiry::<T>::try_mutate(expiry, |ids| {
                ids.try_push(proposal_id)
                    .map_err(|_| Error::<T>::TooManyProposalsAtExpiry.into())
            })
        }

        fn auto_finalize_expiry_bucket(
            expiry: BlockNumberFor<T>,
            now: BlockNumberFor<T>,
            max_count: usize,
        ) -> (usize, bool, Weight) {
            let db_weight = T::DbWeight::get();
            let mut weight = db_weight.reads_writes(1, 1);
            let mut proposal_ids = ProposalsByExpiry::<T>::take(expiry);
            if proposal_ids.is_empty() {
                return (0, false, weight);
            }

            let process_count = core::cmp::min(max_count, proposal_ids.len());
            let mut retry_ids = Vec::new();
            for proposal_id in proposal_ids.drain(..process_count) {
                weight = weight.saturating_add(db_weight.reads(1));
                let Some(proposal) = Proposals::<T>::get(proposal_id) else {
                    continue;
                };
                if proposal.status != STATUS_VOTING || proposal.end >= now {
                    continue;
                }

                let finalize_result = match proposal.stage {
                    STAGE_INTERNAL => Self::do_finalize_internal_timeout(&proposal, proposal_id),
                    STAGE_JOINT => Self::do_finalize_joint_timeout(&proposal, proposal_id),
                    STAGE_CITIZEN => Self::do_finalize_citizen_timeout(&proposal, proposal_id),
                    _ => Ok(()),
                };
                if finalize_result.is_err() {
                    // 中文注释：终结失败时必须保留自动重试索引，
                    // 避免提案状态仍是 Voting，但后续再也不会被 on_initialize 处理。
                    retry_ids.push(proposal_id);
                }
            }
            for proposal_id in retry_ids {
                proposal_ids
                    .try_push(proposal_id)
                    .expect("retry ids come from the drained expiry bucket and must fit");
            }

            let has_remaining = !proposal_ids.is_empty();
            if has_remaining {
                ProposalsByExpiry::<T>::insert(expiry, proposal_ids);
                weight = weight.saturating_add(db_weight.writes(1));
            }

            let per_finalize_weight = T::WeightInfo::finalize_proposal_internal()
                .max(T::WeightInfo::finalize_proposal_joint())
                .max(T::WeightInfo::finalize_proposal_citizen());
            let finalize_weight = per_finalize_weight.saturating_mul(process_count as u64);
            weight = weight.saturating_add(finalize_weight);

            (process_count, has_remaining, weight)
        }

        /// 分配提案 ID：`年份 × 1,000,000 + 年内计数器`。
        /// 每年计数器自动重置。例如：2026000000, 2026000001, ..., 2027000000, ...
        pub(crate) fn allocate_proposal_id() -> Result<u64, DispatchError> {
            let now_ms = T::TimeProvider::now().as_millis();
            // 毫秒 → 秒 → 年份（UTC）
            let secs = (now_ms / 1000) as i64;
            let year = Self::unix_seconds_to_year(secs);

            let stored_year = CurrentProposalYear::<T>::get();
            let counter = if stored_year != year {
                // 新的一年，重置计数器
                CurrentProposalYear::<T>::put(year);
                YearProposalCounter::<T>::put(1u32);
                0u32
            } else {
                let c = YearProposalCounter::<T>::get();
                ensure!(c < 999_999, Error::<T>::ProposalIdOverflow);
                YearProposalCounter::<T>::put(c + 1);
                c
            };

            let id = (year as u64)
                .checked_mul(1_000_000)
                .and_then(|base| base.checked_add(counter as u64))
                .ok_or(Error::<T>::ProposalIdOverflow)?;

            // 更新 NextProposalId（兼容外部查询）
            let next = id.checked_add(1).ok_or(Error::<T>::ProposalIdOverflow)?;
            NextProposalId::<T>::put(next);

            Ok(id)
        }

        /// Unix 秒数转年份（简化算法，不需要精确到天）。
        fn unix_seconds_to_year(secs: i64) -> u16 {
            // 1970-01-01 起算，每年约 31,556,952 秒（365.2425 天）
            const SECS_PER_YEAR: i64 = 31_556_952;
            let year = 1970 + (secs / SECS_PER_YEAR);
            year as u16
        }

        pub(crate) fn ensure_open_proposal(
            proposal_id: u64,
        ) -> Result<Proposal<BlockNumberFor<T>>, DispatchError> {
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(
                proposal.status == STATUS_VOTING,
                Error::<T>::InvalidProposalStatus
            );
            ensure!(
                <frame_system::Pallet<T>>::block_number() <= proposal.end,
                Error::<T>::VoteClosed
            );

            Ok(proposal)
        }

        /// 更新提案状态并发出 ProposalFinalized 事件。
        /// 消费模块在执行成功后调用此方法设置 STATUS_EXECUTED。
        pub fn set_status_and_emit(proposal_id: u64, status: u8) -> DispatchResult {
            with_transaction(|| {
                let (kind, institution) = match Proposals::<T>::try_mutate(
                    proposal_id,
                    |maybe| -> Result<(u8, Option<InstitutionPalletId>), DispatchError> {
                        let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                        let kind = proposal.kind;
                        let inst = proposal.internal_institution;
                        proposal.status = status;
                        Ok((kind, inst))
                    },
                ) {
                    Ok(v) => v,
                    Err(err) => return TransactionOutcome::Rollback(Err(err)),
                };

                // 提案结束（通过或拒绝），立即释放活跃提案名额
                if status != STATUS_VOTING {
                    if let Some(inst) = institution {
                        active_proposal_limit::remove_active_proposal::<T>(inst, proposal_id);
                    }
                }

                Self::deposit_event(Event::<T>::ProposalFinalized {
                    proposal_id,
                    status,
                });

                if kind == PROPOSAL_KIND_JOINT && status != STATUS_VOTING {
                    if let Err(err) = T::JointVoteResultCallback::on_joint_vote_finalized(
                        proposal_id,
                        status == STATUS_PASSED,
                    ) {
                        // 中文注释：联合投票结果必须由业务模块成功消费后，
                        // 才允许投票引擎把提案标记为最终状态。
                        return TransactionOutcome::Rollback(Err(err));
                    }
                }

                // 终态转换时自动注册 90 天延迟清理（PASSED/REJECTED/EXECUTED 均触发）。
                // 同一提案可能多次进入终态（PASSED → EXECUTED），schedule_cleanup
                // 内部用 try_push 保证幂等，重复注册不会导致重复清理。
                if status != STATUS_VOTING {
                    let now = frame_system::Pallet::<T>::block_number();
                    let _ = proposal_cleanup::schedule_cleanup::<T>(proposal_id, now);
                }

                TransactionOutcome::Commit(Ok(()))
            })
        }

        /// 低级状态覆盖：仅供 on_joint_vote_finalized 回调在同一事务内纠正状态。
        /// 不发事件、不注册清理——这些由外层 set_status_and_emit 统一处理。
        pub fn override_proposal_status(proposal_id: u64, new_status: u8) -> DispatchResult {
            Proposals::<T>::try_mutate(proposal_id, |maybe| {
                let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                proposal.status = new_status;
                Ok(())
            })
        }

        fn process_pending_cleanup_steps() -> Weight {
            let max_steps = T::MaxCleanupStepsPerBlock::get() as usize;
            if max_steps == 0 {
                return Weight::zero();
            }

            let cleanup_limit = T::CleanupKeysPerStep::get().max(1);
            let db_weight = T::DbWeight::get();
            let mut weight = Weight::zero();
            // 每步的最大 weight 上界：cleanup_limit 次读 + cleanup_limit 次写 + 固定开销
            let max_weight_per_step =
                db_weight.reads_writes(u64::from(cleanup_limit) + 2, u64::from(cleanup_limit) + 2);

            for _ in 0..max_steps {
                let Some((proposal_id, stage)) = PendingProposalCleanups::<T>::iter().next() else {
                    break;
                };
                weight = weight.saturating_add(db_weight.reads(1));

                let (next_stage, _actual_weight) =
                    Self::process_pending_cleanup_step(proposal_id, stage, cleanup_limit);
                // 使用预估最大值而非实际值，确保 on_initialize 不超出声明的 weight
                weight = weight.saturating_add(max_weight_per_step);

                match next_stage {
                    Some(next) if next != stage => {
                        PendingProposalCleanups::<T>::insert(proposal_id, next);
                        weight = weight.saturating_add(db_weight.writes(1));
                    }
                    Some(_) => {}
                    None => {
                        PendingProposalCleanups::<T>::remove(proposal_id);
                        weight = weight.saturating_add(db_weight.writes(1));
                    }
                }
            }

            weight
        }

        fn process_pending_cleanup_step(
            proposal_id: u64,
            stage: PendingCleanupStage,
            cleanup_limit: u32,
        ) -> (Option<PendingCleanupStage>, Weight) {
            let db_weight = T::DbWeight::get();

            match stage {
                PendingCleanupStage::InternalVotes => {
                    let result =
                        InternalVotesByAccount::<T>::clear_prefix(proposal_id, cleanup_limit, None);
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::InternalVotes)
                    } else {
                        Some(PendingCleanupStage::JointAdminVotes) // 继续下一阶段
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointAdminVotes => {
                    let result =
                        JointVotesByAdmin::<T>::clear_prefix(proposal_id, cleanup_limit, None);
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::JointAdminVotes)
                    } else {
                        Some(PendingCleanupStage::JointInstitutionVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointInstitutionVotes => {
                    let result = JointVotesByInstitution::<T>::clear_prefix(
                        proposal_id,
                        cleanup_limit,
                        None,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::JointInstitutionVotes)
                    } else {
                        Some(PendingCleanupStage::JointInstitutionTallies)
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointInstitutionTallies => {
                    let result = JointInstitutionTallies::<T>::clear_prefix(
                        proposal_id,
                        cleanup_limit,
                        None,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::JointInstitutionTallies)
                    } else {
                        Some(PendingCleanupStage::CitizenVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::CitizenVotes => {
                    let result = CitizenVotesByBindingId::<T>::clear_prefix(
                        proposal_id,
                        cleanup_limit,
                        None,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::CitizenVotes)
                    } else {
                        Some(PendingCleanupStage::VoteCredentials)
                    };
                    (next, weight)
                }
                PendingCleanupStage::VoteCredentials => {
                    let result = T::SfidEligibility::cleanup_vote_credentials_chunk(
                        proposal_id,
                        cleanup_limit,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.removed));
                    let next = if result.has_remaining {
                        Some(PendingCleanupStage::VoteCredentials)
                    } else {
                        Some(PendingCleanupStage::ProposalObject)
                    };
                    (next, weight)
                }
                PendingCleanupStage::ProposalObject => {
                    ProposalObject::<T>::remove(proposal_id);
                    ProposalObjectMeta::<T>::remove(proposal_id);
                    let weight = db_weight.writes(2);
                    (Some(PendingCleanupStage::FinalCleanup), weight)
                }
                PendingCleanupStage::FinalCleanup => {
                    // 清理核心数据 + 业务数据（单次完成）
                    Proposals::<T>::remove(proposal_id);
                    InternalTallies::<T>::remove(proposal_id);
                    JointTallies::<T>::remove(proposal_id);
                    CitizenTallies::<T>::remove(proposal_id);
                    ProposalData::<T>::remove(proposal_id);
                    ProposalMeta::<T>::remove(proposal_id);
                    let weight = db_weight.writes(6);
                    (None, weight) // 全部完成
                }
            }
        }
    }

    // ──── 统一提案数据存储公共接口 ────

    impl<T: Config> Pallet<T> {
        /// 存储提案业务数据（由业务模块在创建提案时调用）。
        pub fn store_proposal_data(proposal_id: u64, data: sp_std::vec::Vec<u8>) -> DispatchResult {
            let bounded: BoundedVec<u8, T::MaxProposalDataLen> = data
                .try_into()
                .map_err(|_| DispatchError::Other("ProposalDataTooLarge"))?;
            ProposalData::<T>::insert(proposal_id, bounded);
            Ok(())
        }

        /// 读取提案业务数据。
        pub fn get_proposal_data(proposal_id: u64) -> Option<sp_std::vec::Vec<u8>> {
            ProposalData::<T>::get(proposal_id).map(|v| v.into_inner())
        }

        /// 存储提案大对象（例如 runtime wasm）。
        pub fn store_proposal_object(
            proposal_id: u64,
            kind: u8,
            data: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let object_len = u32::try_from(data.len())
                .map_err(|_| DispatchError::Other("ProposalObjectTooLarge"))?;
            let object_hash = T::Hashing::hash(&data);
            let bounded: BoundedVec<u8, T::MaxProposalObjectLen> = data
                .try_into()
                .map_err(|_| DispatchError::Other("ProposalObjectTooLarge"))?;
            ProposalObject::<T>::insert(proposal_id, bounded);
            ProposalObjectMeta::<T>::insert(
                proposal_id,
                ProposalObjectMetadata {
                    kind,
                    object_len,
                    object_hash,
                },
            );
            Ok(())
        }

        /// 读取提案大对象原始数据。
        pub fn get_proposal_object(proposal_id: u64) -> Option<sp_std::vec::Vec<u8>> {
            ProposalObject::<T>::get(proposal_id).map(|v| v.into_inner())
        }

        /// 读取提案对象层元数据。
        pub fn get_proposal_object_meta(
            proposal_id: u64,
        ) -> Option<ProposalObjectMetadata<T::Hash>> {
            ProposalObjectMeta::<T>::get(proposal_id)
        }

        /// 删除提案对象层数据与元数据。
        pub fn remove_proposal_object(proposal_id: u64) {
            ProposalObject::<T>::remove(proposal_id);
            ProposalObjectMeta::<T>::remove(proposal_id);
        }

        /// 存储提案辅助元数据（创建时间）。
        pub fn store_proposal_meta(proposal_id: u64, created_at: BlockNumberFor<T>) {
            ProposalMeta::<T>::insert(
                proposal_id,
                ProposalMetadata {
                    created_at,
                    passed_at: None,
                },
            );
        }

        /// 标记提案通过时间。
        pub fn set_proposal_passed(proposal_id: u64, block: BlockNumberFor<T>) {
            ProposalMeta::<T>::mutate(proposal_id, |meta| {
                if let Some(m) = meta {
                    m.passed_at = Some(block);
                }
            });
        }

        /// 读取提案辅助元数据。
        pub fn get_proposal_meta(proposal_id: u64) -> Option<ProposalMetadata<BlockNumberFor<T>>> {
            ProposalMeta::<T>::get(proposal_id)
        }
    }
}

impl<T: pallet::Config> JointVoteEngine<T::AccountId> for pallet::Pallet<T> {
    fn create_joint_proposal(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
    ) -> Result<u64, DispatchError> {
        let snapshot_nonce: pallet::VoteNonceOf<T> = snapshot_nonce
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        let signature: pallet::VoteSignatureOf<T> = signature
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        pallet::Pallet::<T>::do_create_joint_proposal(
            who,
            eligible_total,
            snapshot_nonce,
            signature,
        )
    }

    fn cleanup_joint_proposal(_proposal_id: u64) {
        // 已弃用：清理现在由 set_status_and_emit 在终态转换时自动注册。
        // 保留空实现以兼容 trait 定义。
    }
}

impl<T: pallet::Config> InternalVoteEngine<T::AccountId> for pallet::Pallet<T> {
    fn create_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        pallet::Pallet::<T>::do_create_internal_proposal(who, org, institution)
    }

    fn cast_internal_vote(
        who: T::AccountId,
        proposal_id: u64,
        approve: bool,
    ) -> Result<(), DispatchError> {
        pallet::Pallet::<T>::do_internal_vote(who, proposal_id, approve)
    }

    fn cleanup_internal_proposal(_proposal_id: u64) {
        // 已弃用：清理现在由 set_status_and_emit 在终态转换时自动注册。
        // 保留空实现以兼容 trait 定义。
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;
    use std::collections::BTreeSet;

    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32, traits::Hooks};
    use frame_system as system;
    use primitives::china::china_cb::{
        shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
    };
    use primitives::china::china_ch::{
        shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
    };
    use sp_runtime::{
        traits::Hash, traits::IdentityLookup, AccountId32, BuildStorage, DispatchError,
    };

    type Block = frame_system::mocking::MockBlock<Test>;

    #[frame_support::runtime]
    mod runtime {
        #[runtime::runtime]
        #[runtime::derive(
            RuntimeCall,
            RuntimeEvent,
            RuntimeError,
            RuntimeOrigin,
            RuntimeFreezeReason,
            RuntimeHoldReason,
            RuntimeSlashReason,
            RuntimeLockId,
            RuntimeTask,
            RuntimeViewFunction
        )]
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;

        #[runtime::pallet_index(1)]
        pub type VotingEngineSystem = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type MaxProposalsPerExpiry = ConstU32<128>;
        type MaxCleanupStepsPerBlock = ConstU32<3>;
        type CleanupKeysPerStep = ConstU32<2>;
        type MaxProposalDataLen = ConstU32<4096>;
        type MaxProposalObjectLen = ConstU32<10_240>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = TestJointVoteResultCallback;
        type InternalAdminProvider = ();
        type InternalAdminCountProvider = ();
        type InternalThresholdProvider = ();
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    thread_local! {
        static USED_VOTE_NONCES: RefCell<BTreeSet<(u64, Vec<u8>, Vec<u8>)>> = RefCell::new(BTreeSet::new());
    }
    thread_local! {
        static JOINT_CALLBACK_SHOULD_FAIL: RefCell<bool> = const { RefCell::new(false) };
    }

    pub struct TestSfidEligibility;
    pub struct TestPopulationSnapshotVerifier;
    pub struct TestJointVoteResultCallback;

    /// 测试用时间提供器：返回 2026 年中（2026-07-01 00:00:00 UTC）。
    pub struct TestTimeProvider;
    impl frame_support::traits::UnixTime for TestTimeProvider {
        fn now() -> core::time::Duration {
            // 2026-07-01 00:00:00 UTC ≈ 1782864000 秒
            core::time::Duration::from_secs(1_782_864_000)
        }
    }
    impl
        PopulationSnapshotVerifier<
            AccountId32,
            pallet::VoteNonceOf<Test>,
            pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            eligible_total: u64,
            nonce: &pallet::VoteNonceOf<Test>,
            signature: &pallet::VoteSignatureOf<Test>,
        ) -> bool {
            eligible_total > 0 && !nonce.is_empty() && signature.as_slice() == b"snapshot-ok"
        }
    }

    impl SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash> for TestSfidEligibility {
        fn is_eligible(
            binding_id: &<Test as frame_system::Config>::Hash,
            who: &AccountId32,
        ) -> bool {
            *binding_id == binding_id_ok() && who == &nrc_admin(0)
        }

        fn verify_and_consume_vote_credential(
            binding_id: &<Test as frame_system::Config>::Hash,
            who: &AccountId32,
            proposal_id: u64,
            nonce: &[u8],
            signature: &[u8],
        ) -> bool {
            if !Self::is_eligible(binding_id, who) || signature != b"vote-ok" || nonce.is_empty() {
                return false;
            }
            let key = (proposal_id, binding_id.encode(), nonce.to_vec());
            USED_VOTE_NONCES.with(|set| {
                let mut set = set.borrow_mut();
                if set.contains(&key) {
                    false
                } else {
                    set.insert(key);
                    true
                }
            })
        }

        fn cleanup_vote_credentials(proposal_id: u64) {
            USED_VOTE_NONCES.with(|set| {
                set.borrow_mut().retain(|(pid, _, _)| *pid != proposal_id);
            });
        }

        fn cleanup_vote_credentials_chunk(proposal_id: u64, limit: u32) -> VoteCredentialCleanup {
            let mut to_remove = Vec::new();
            USED_VOTE_NONCES.with(|set| {
                for key in set.borrow().iter() {
                    if key.0 == proposal_id {
                        to_remove.push(key.clone());
                        if to_remove.len() >= limit as usize {
                            break;
                        }
                    }
                }
            });

            let has_remaining = USED_VOTE_NONCES.with(|set| {
                let mut set = set.borrow_mut();
                for key in &to_remove {
                    set.remove(key);
                }
                set.iter().any(|(pid, _, _)| *pid == proposal_id)
            });

            VoteCredentialCleanup {
                removed: to_remove.len() as u32,
                loops: to_remove.len() as u32,
                has_remaining,
            }
        }
    }

    impl JointVoteResultCallback for TestJointVoteResultCallback {
        fn on_joint_vote_finalized(_vote_proposal_id: u64, _approved: bool) -> DispatchResult {
            if JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow()) {
                Err(DispatchError::Other("joint callback failed"))
            } else {
                Ok(())
            }
        }
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            USED_VOTE_NONCES.with(|set| set.borrow_mut().clear());
            JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
            System::set_block_number(1);
        });
        ext
    }

    fn nrc_pid() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
            .expect("nrc id should be shenfen_id bytes")
    }

    fn prc_pid() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id)
            .expect("prc id should be shenfen_id bytes")
    }

    fn prb_pid() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(CHINA_CH[0].shenfen_id)
            .expect("prb id should be shenfen_id bytes")
    }

    fn nrc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[0].duoqian_admins[index])
    }

    fn all_prc_institutions() -> Vec<(InstitutionPalletId, AccountId32)> {
        CHINA_CB
            .iter()
            .skip(1)
            .map(|n| {
                (
                    reserve_pallet_id_to_bytes(n.shenfen_id)
                        .expect("prc id should be shenfen_id bytes"),
                    AccountId32::new(n.duoqian_admins[0]),
                )
            })
            .collect()
    }

    fn all_prb_institutions() -> Vec<(InstitutionPalletId, AccountId32)> {
        CHINA_CH
            .iter()
            .map(|n| {
                (
                    shengbank_pallet_id_to_bytes(n.shenfen_id)
                        .expect("prb id should be shenfen_id bytes"),
                    AccountId32::new(n.duoqian_admins[0]),
                )
            })
            .collect()
    }

    fn prc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[1].duoqian_admins[index])
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CH[0].duoqian_admins[index])
    }

    fn institution_admins(institution: InstitutionPalletId) -> Vec<AccountId32> {
        CHINA_CB
            .iter()
            .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
            .map(|n| {
                n.duoqian_admins
                    .iter()
                    .copied()
                    .map(AccountId32::new)
                    .collect()
            })
            .or_else(|| {
                CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| {
                        n.duoqian_admins
                            .iter()
                            .copied()
                            .map(AccountId32::new)
                            .collect()
                    })
            })
            .expect("institution should have admins")
    }

    fn institution_threshold(institution: InstitutionPalletId) -> usize {
        if institution == nrc_pid() {
            return primitives::count_const::NRC_INTERNAL_THRESHOLD as usize;
        }
        if CHINA_CB
            .iter()
            .skip(1)
            .any(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        {
            return primitives::count_const::PRC_INTERNAL_THRESHOLD as usize;
        }
        if CHINA_CH
            .iter()
            .any(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        {
            return primitives::count_const::PRB_INTERNAL_THRESHOLD as usize;
        }
        panic!("unknown institution");
    }

    fn cast_joint_votes_until_finalized(
        proposal_id: u64,
        institution: InstitutionPalletId,
        approve: bool,
    ) {
        let admins = institution_admins(institution);
        let threshold = institution_threshold(institution);
        let required_votes = if approve {
            threshold
        } else {
            admins.len().saturating_sub(threshold).saturating_add(1)
        };
        for admin in admins.into_iter().take(required_votes) {
            assert_ok!(submit_joint_vote(admin, proposal_id, institution, approve));
        }
    }

    fn submit_joint_vote(
        who: AccountId32,
        proposal_id: u64,
        institution: InstitutionPalletId,
        approve: bool,
    ) -> DispatchResult {
        VotingEngineSystem::joint_vote(
            RuntimeOrigin::signed(who),
            proposal_id,
            institution,
            approve,
        )
    }

    fn binding_id_ok() -> <Test as frame_system::Config>::Hash {
        <Test as frame_system::Config>::Hashing::hash(b"sfid-ok")
    }

    fn vote_nonce(input: &str) -> pallet::VoteNonceOf<Test> {
        input
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("nonce should fit")
    }

    fn vote_sig_ok() -> pallet::VoteSignatureOf<Test> {
        b"vote-ok"
            .to_vec()
            .try_into()
            .expect("signature should fit")
    }

    fn vote_sig_bad() -> pallet::VoteSignatureOf<Test> {
        b"bad".to_vec().try_into().expect("signature should fit")
    }

    fn snapshot_nonce_ok() -> pallet::VoteNonceOf<Test> {
        b"snap-nonce"
            .to_vec()
            .try_into()
            .expect("snapshot nonce should fit")
    }

    fn snapshot_sig_ok() -> pallet::VoteSignatureOf<Test> {
        b"snapshot-ok"
            .to_vec()
            .try_into()
            .expect("snapshot signature should fit")
    }

    fn set_joint_callback_should_fail(should_fail: bool) {
        JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = should_fail);
    }

    fn mark_vote_nonce_used(
        proposal_id: u64,
        binding_id: <Test as frame_system::Config>::Hash,
        nonce: &str,
    ) {
        USED_VOTE_NONCES.with(|set| {
            set.borrow_mut()
                .insert((proposal_id, binding_id.encode(), nonce.as_bytes().to_vec()));
        });
    }

    fn has_used_vote_nonce(
        proposal_id: u64,
        binding_id: <Test as frame_system::Config>::Hash,
        nonce: &str,
    ) -> bool {
        USED_VOTE_NONCES.with(|set| {
            set.borrow()
                .contains(&(proposal_id, binding_id.encode(), nonce.as_bytes().to_vec()))
        })
    }

    fn create_internal_proposal_via_engine(
        who: AccountId32,
        org: u8,
        institution: InstitutionPalletId,
    ) -> u64 {
        <VotingEngineSystem as InternalVoteEngine<AccountId32>>::create_internal_proposal(
            who,
            org,
            institution,
        )
        .expect("internal proposal should be created")
    }

    fn insert_citizen_proposal(proposal_id: u64, eligible_total: u64, end: u64) {
        Proposals::<Test>::insert(
            proposal_id,
            Proposal {
                kind: PROPOSAL_KIND_JOINT,
                stage: STAGE_CITIZEN,
                status: STATUS_VOTING,
                internal_org: None,
                internal_institution: None,
                start: System::block_number(),
                end,
                citizen_eligible_total: eligible_total,
            },
        );
    }

    #[test]
    fn internal_proposal_must_be_created_by_same_institution_admin() {
        new_test_ext().execute_with(|| {
            let outsider = AccountId32::new([7u8; 32]);

            assert_noop!(
                VotingEngineSystem::create_internal_proposal(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::NoPermission
            );

            assert_noop!(
                <VotingEngineSystem as InternalVoteEngine<AccountId32>>::create_internal_proposal(
                    outsider,
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::NoPermission
            );

            assert_noop!(
                <VotingEngineSystem as InternalVoteEngine<AccountId32>>::create_internal_proposal(
                    prc_admin(0),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::NoPermission
            );

            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            assert_eq!(proposal_id, 2026_000_000);
            assert_eq!(
                VotingEngineSystem::proposals(proposal_id)
                    .expect("proposal exists")
                    .stage,
                STAGE_INTERNAL
            );
        });
    }

    #[test]
    fn internal_vote_must_be_by_same_institution_admin() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                prb_admin(0),
                internal_vote::ORG_PRB,
                prb_pid(),
            );

            assert_noop!(
                <VotingEngineSystem as InternalVoteEngine<AccountId32>>::cast_internal_vote(
                    nrc_admin(0),
                    proposal_id,
                    true,
                ),
                pallet::Error::<Test>::NoPermission
            );

            assert_ok!(
                <VotingEngineSystem as InternalVoteEngine<AccountId32>>::cast_internal_vote(
                    prb_admin(1),
                    proposal_id,
                    true,
                )
            );
        });
    }

    #[test]
    fn nrc_internal_vote_passes_at_13_yes_votes() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            for i in 0..12 {
                assert_ok!(
                    <VotingEngineSystem as InternalVoteEngine<AccountId32>>::cast_internal_vote(
                        nrc_admin(i),
                        proposal_id,
                        true,
                    )
                );
            }
            assert_eq!(
                VotingEngineSystem::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_VOTING
            );

            assert_ok!(
                <VotingEngineSystem as InternalVoteEngine<AccountId32>>::cast_internal_vote(
                    nrc_admin(12),
                    proposal_id,
                    true,
                )
            );
            assert_eq!(
                VotingEngineSystem::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_PASSED
            );
        });
    }

    #[test]
    fn internal_vote_is_rejected_after_timeout() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                prc_admin(0),
                internal_vote::ORG_PRC,
                prc_pid(),
            );

            let proposal = VotingEngineSystem::proposals(proposal_id).expect("proposal exists");
            System::set_block_number(proposal.end + 1);

            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                proposal_id,
            ));
            assert_eq!(
                VotingEngineSystem::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn internal_vote_timeout_is_auto_rejected_on_initialize() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                prc_admin(0),
                internal_vote::ORG_PRC,
                prc_pid(),
            );

            let proposal = VotingEngineSystem::proposals(proposal_id).expect("proposal exists");
            System::set_block_number(proposal.end);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(proposal.end);
            assert_eq!(
                VotingEngineSystem::proposals(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );

            let next = proposal.end + 1;
            System::set_block_number(next);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(next);
            assert_eq!(
                VotingEngineSystem::proposals(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn joint_proposal_must_be_created_by_nrc_admin() {
        new_test_ext().execute_with(|| {
            // 中文注释：外部 extrinsic 入口已禁用，统一要求事项模块通过 trait 创建联合投票提案。
            assert_noop!(
                VotingEngineSystem::create_joint_proposal(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    10,
                    snapshot_nonce_ok(),
                    snapshot_sig_ok()
                ),
                pallet::Error::<Test>::NoPermission
            );

            let outsider = AccountId32::new([9u8; 32]);
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    outsider,
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
                .is_err()
            );

            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    prc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
                .is_err()
            );

            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );
        });
    }

    #[test]
    fn joint_vote_requires_current_institution_admin() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_ok!(submit_joint_vote(
                nrc_admin(0),
                proposal_id,
                nrc_pid(),
                true
            ));

            assert_ok!(submit_joint_vote(
                prc_admin(0),
                proposal_id,
                prc_pid(),
                true
            ));

            assert_noop!(
                submit_joint_vote(prc_admin(0), proposal_id, nrc_pid(), true),
                pallet::Error::<Test>::NoPermission
            );
        });
    }

    #[test]
    fn joint_vote_rejects_duplicate_admin_vote() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_ok!(submit_joint_vote(
                nrc_admin(0),
                proposal_id,
                nrc_pid(),
                true
            ));

            assert_noop!(
                submit_joint_vote(nrc_admin(0), proposal_id, nrc_pid(), true),
                pallet::Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn joint_vote_auto_rejects_institution_when_yes_is_no_longer_reachable() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            cast_joint_votes_until_finalized(proposal_id, nrc_pid(), false);

            assert_eq!(
                JointVotesByInstitution::<Test>::get(proposal_id, nrc_pid()),
                Some(false)
            );
            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.stage, STAGE_CITIZEN);
            assert_eq!(
                JointTallies::<Test>::get(proposal_id).no,
                primitives::count_const::NRC_JOINT_VOTE_WEIGHT
            );
        });
    }

    #[test]
    fn population_snapshot_nonce_cannot_be_reused_across_proposals() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    11,
                    nonce.as_slice(),
                    sig.as_slice()
                )
                .is_err()
            );
        });
    }

    #[test]
    fn citizen_vote_rejects_invalid_signature_and_allows_valid_vote() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    binding_id_ok(),
                    vote_nonce("n-1"),
                    vote_sig_bad(),
                    true
                ),
                pallet::Error::<Test>::InvalidSfidVoteCredential
            );

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("n-2"),
                vote_sig_ok(),
                true
            ));
            assert_eq!(CitizenTallies::<Test>::get(0).yes, 1);
        });
    }

    #[test]
    fn citizen_vote_same_sfid_can_only_vote_once_per_proposal() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("n-1"),
                vote_sig_ok(),
                true
            ));

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    binding_id_ok(),
                    vote_nonce("n-2"),
                    vote_sig_ok(),
                    false
                ),
                pallet::Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn citizen_vote_credential_nonce_is_replay_protected_per_proposal_and_sfid() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            insert_citizen_proposal(1, 10, 100);

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("same"),
                vote_sig_ok(),
                true
            ));

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                1,
                binding_id_ok(),
                vote_nonce("same"),
                vote_sig_ok(),
                true
            ));
        });
    }

    #[test]
    fn citizen_vote_rejects_when_eligible_total_not_set_in_proposal() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 0, 100);

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    binding_id_ok(),
                    vote_nonce("x-1"),
                    vote_sig_ok(),
                    true
                ),
                pallet::Error::<Test>::CitizenEligibleTotalNotSet
            );
        });
    }

    #[test]
    fn citizen_timeout_with_half_or_less_is_rejected() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 5);
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });
            System::set_block_number(6);

            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));
            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn citizen_timeout_is_auto_rejected_on_initialize() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 5);
            assert_ok!(VotingEngineSystem::schedule_proposal_expiry(0, 5));
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

            System::set_block_number(6);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(6);
            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn citizen_timeout_auto_registers_cleanup_and_clears_vote_nonces() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 5);
            assert_ok!(VotingEngineSystem::schedule_proposal_expiry(0, 5));

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("timeout-cleanup"),
                vote_sig_ok(),
                true
            ));
            assert!(has_used_vote_nonce(0, binding_id_ok(), "timeout-cleanup"));

            System::set_block_number(6);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(6);

            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
            assert!(has_used_vote_nonce(0, binding_id_ok(), "timeout-cleanup"));

            // set_status_and_emit(STATUS_REJECTED) 在 on_initialize(6) 中被调用时
            // 已自动注册 90 天后清理，无需手动调用 cleanup_joint_proposal。
            // cleanup_at = 6 + retention
            let retention = 90u64 * primitives::pow_const::BLOCKS_PER_DAY;
            let cleanup_block = 6 + retention;
            for i in 0..20u64 {
                System::set_block_number(cleanup_block + i);
                <VotingEngineSystem as Hooks<u64>>::on_initialize(cleanup_block + i);
            }
            assert!(!has_used_vote_nonce(0, binding_id_ok(), "timeout-cleanup"));
        });
    }

    #[test]
    fn citizen_vote_rejects_ineligible_hash_and_ineligible_account() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    <Test as frame_system::Config>::Hashing::hash(b"sfid-other"),
                    vote_nonce("n-ineligible-hash"),
                    vote_sig_ok(),
                    true
                ),
                pallet::Error::<Test>::SfidNotEligible
            );

            let outsider = AccountId32::new([7u8; 32]);
            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(outsider),
                    0,
                    binding_id_ok(),
                    vote_nonce("n-ineligible"),
                    vote_sig_ok(),
                    true
                ),
                pallet::Error::<Test>::SfidNotEligible
            );
        });
    }

    #[test]
    fn citizen_vote_rejects_when_not_in_citizen_stage() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    proposal_id,
                    binding_id_ok(),
                    vote_nonce("joint-stage"),
                    vote_sig_ok(),
                    true
                ),
                pallet::Error::<Test>::InvalidProposalStage
            );
        });
    }

    #[test]
    fn citizen_vote_passes_immediately_when_yes_exceeds_half() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("immediate-pass"),
                vote_sig_ok(),
                true
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_PASSED);
        });
    }

    #[test]
    fn cleanup_joint_proposal_cleans_used_vote_nonce_after_retention() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("immediate-cleanup"),
                vote_sig_ok(),
                true
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_PASSED);
            assert!(has_used_vote_nonce(0, binding_id_ok(), "immediate-cleanup"));

            // set_status_and_emit(STATUS_PASSED) 在 citizen_vote 通过时已自动注册 90 天后清理。
            // 推进到清理到期区块并运行多轮 on_initialize 直到清理完成。
            let retention = 90u64 * primitives::pow_const::BLOCKS_PER_DAY;
            // schedule_cleanup 在 block 0 调用，cleanup_at = 0 + retention = retention
            let cleanup_block = retention;
            for i in 0..20u64 {
                System::set_block_number(cleanup_block + i);
                <VotingEngineSystem as Hooks<u64>>::on_initialize(cleanup_block + i);
            }
            assert!(!has_used_vote_nonce(
                0,
                binding_id_ok(),
                "immediate-cleanup"
            ));
        });
    }

    #[test]
    fn citizen_finalize_before_timeout_is_rejected() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            System::set_block_number(100);

            assert_noop!(
                VotingEngineSystem::finalize_proposal(RuntimeOrigin::signed(nrc_admin(0)), 0),
                pallet::Error::<Test>::VoteNotExpired
            );
        });
    }

    #[test]
    fn citizen_pass_threshold_function_boundaries_are_correct() {
        assert!(!citizen_vote::is_citizen_vote_passed(0, 0));
        assert!(!citizen_vote::is_citizen_vote_passed(5, 10));
        assert!(citizen_vote::is_citizen_vote_passed(6, 10));
    }

    #[test]
    fn joint_vote_all_yes_passes_immediately() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    100,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            cast_joint_votes_until_finalized(proposal_id, nrc_pid(), true);

            for (institution, _) in all_prc_institutions() {
                cast_joint_votes_until_finalized(proposal_id, institution, true);
            }
            for (institution, _) in all_prb_institutions() {
                cast_joint_votes_until_finalized(proposal_id, institution, true);
            }

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_PASSED);
            assert_eq!(proposal.stage, STAGE_JOINT);
            assert_eq!(
                JointTallies::<Test>::get(proposal_id).yes,
                primitives::count_const::JOINT_VOTE_TOTAL
            );
        });
    }

    #[test]
    fn joint_vote_non_unanimous_moves_to_citizen_immediately_after_one_institution_rejects() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    77,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");
            cast_joint_votes_until_finalized(proposal_id, nrc_pid(), true);
            let first_prc = all_prc_institutions()
                .first()
                .cloned()
                .expect("there should be at least one prc institution");
            cast_joint_votes_until_finalized(proposal_id, first_prc.0, false);

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.stage, STAGE_CITIZEN);
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(proposal.start, System::block_number());
            assert_eq!(
                proposal.end,
                proposal.start + primitives::count_const::VOTING_DURATION_BLOCKS as u64
            );
            assert_eq!(proposal.citizen_eligible_total, 77);
            assert_eq!(JointTallies::<Test>::get(proposal_id).no, 1);
        });
    }

    #[test]
    fn joint_vote_timeout_moves_to_citizen_when_not_unanimous() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    88,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_ok!(submit_joint_vote(
                nrc_admin(0),
                proposal_id,
                nrc_pid(),
                true
            ));

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            System::set_block_number(proposal.end + 1);
            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                proposal_id
            ));

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.stage, STAGE_CITIZEN);
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(
                proposal.end,
                (proposal.start + primitives::count_const::VOTING_DURATION_BLOCKS as u64)
            );
        });
    }

    #[test]
    fn joint_vote_timeout_auto_moves_to_citizen_on_initialize() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    88,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_ok!(submit_joint_vote(
                nrc_admin(0),
                proposal_id,
                nrc_pid(),
                true
            ));

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            let expired_at = proposal.end + 1;
            System::set_block_number(expired_at);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(expired_at);

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.stage, STAGE_CITIZEN);
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(proposal.start, expired_at);
            assert_eq!(
                proposal.end,
                expired_at + primitives::count_const::VOTING_DURATION_BLOCKS as u64
            );
        });
    }

    #[test]
    fn joint_vote_timeout_with_unanimous_tally_passes() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    66,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");
            JointTallies::<Test>::insert(
                proposal_id,
                VoteCountU32 {
                    yes: primitives::count_const::JOINT_VOTE_TOTAL,
                    no: 0,
                },
            );

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            System::set_block_number(proposal.end + 1);
            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                proposal_id
            ));

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_PASSED);
            assert_eq!(proposal.stage, STAGE_JOINT);
        });
    }

    #[test]
    fn joint_vote_callback_failure_rolls_back_final_status() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    100,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            set_joint_callback_should_fail(true);
            assert!(VotingEngineSystem::set_status_and_emit(proposal_id, STATUS_PASSED).is_err());

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(proposal.stage, STAGE_JOINT);
        });
    }

    #[test]
    fn joint_vote_callback_failure_does_not_cleanup_vote_credentials() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            mark_vote_nonce_used(0, binding_id_ok(), "keep-on-fail");
            set_joint_callback_should_fail(true);

            assert!(VotingEngineSystem::set_status_and_emit(0, STATUS_PASSED).is_err());
            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );
            assert!(has_used_vote_nonce(0, binding_id_ok(), "keep-on-fail"));
        });
    }

    #[test]
    fn auto_finalize_requeues_failed_joint_callback() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    66,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            JointTallies::<Test>::insert(
                proposal_id,
                VoteCountU32 {
                    yes: primitives::count_const::JOINT_VOTE_TOTAL,
                    no: 0,
                },
            );

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            let expired_at = proposal.end + 1;

            set_joint_callback_should_fail(true);
            System::set_block_number(expired_at);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(expired_at);

            assert_eq!(
                Proposals::<Test>::get(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );
            assert_eq!(PendingExpiryBucket::<Test>::get(), Some(expired_at));
            assert_eq!(
                ProposalsByExpiry::<Test>::get(expired_at),
                vec![proposal_id]
            );

            set_joint_callback_should_fail(false);
            let next_block = expired_at + 1;
            System::set_block_number(next_block);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(next_block);

            assert_eq!(
                Proposals::<Test>::get(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert!(PendingExpiryBucket::<Test>::get().is_none());
            assert!(ProposalsByExpiry::<Test>::get(expired_at).is_empty());
        });
    }

    #[test]
    fn auto_finalize_uses_pending_cursor_when_expiry_bucket_exceeds_per_block_limit() {
        new_test_ext().execute_with(|| {
            let end = 5u64;
            let expiry = end + 1;
            let total = 70u64;
            for proposal_id in 0..total {
                insert_citizen_proposal(proposal_id, 10, end);
                assert_ok!(VotingEngineSystem::schedule_proposal_expiry(
                    proposal_id,
                    end
                ));
            }

            System::set_block_number(6);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(6);
            assert_eq!(ProposalsByExpiry::<Test>::get(expiry).len(), 6);
            assert_eq!(PendingExpiryBucket::<Test>::get(), Some(expiry));

            System::set_block_number(7);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(7);
            assert!(ProposalsByExpiry::<Test>::get(expiry).is_empty());
            assert!(PendingExpiryBucket::<Test>::get().is_none());
            for proposal_id in 0..total {
                assert_eq!(
                    Proposals::<Test>::get(proposal_id)
                        .expect("proposal should exist")
                        .status,
                    STATUS_REJECTED
                );
            }
        });
    }

    #[test]
    fn schedule_proposal_expiry_rejects_bucket_overflow() {
        new_test_ext().execute_with(|| {
            let end = 5u64;
            for proposal_id in 0..128u64 {
                assert_ok!(VotingEngineSystem::schedule_proposal_expiry(
                    proposal_id,
                    end
                ));
            }

            assert_noop!(
                VotingEngineSystem::schedule_proposal_expiry(999, end),
                pallet::Error::<Test>::TooManyProposalsAtExpiry
            );
        });
    }

    #[test]
    fn cleanup_joint_proposal_chunks_cleanup_across_blocks() {
        new_test_ext().execute_with(|| {
            let proposal_id = 42u64;
            let citizen_hashes = [
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-1"),
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-2"),
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-3"),
            ];

            insert_citizen_proposal(proposal_id, 10, 100);
            JointVotesByInstitution::<Test>::insert(proposal_id, nrc_pid(), true);
            JointVotesByInstitution::<Test>::insert(proposal_id, prc_pid(), true);
            JointVotesByInstitution::<Test>::insert(proposal_id, prb_pid(), true);
            for (index, binding_id) in citizen_hashes.iter().enumerate() {
                CitizenVotesByBindingId::<Test>::insert(proposal_id, *binding_id, true);
                let nonce = match index {
                    0 => "cleanup-nonce-1",
                    1 => "cleanup-nonce-2",
                    _ => "cleanup-nonce-3",
                };
                mark_vote_nonce_used(proposal_id, *binding_id, nonce);
            }

            // set_status_and_emit 终态转换时自动注册 90 天后清理
            assert_ok!(VotingEngineSystem::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));
            // 此时 PendingProposalCleanups 尚未设置（要等 90 天后 process_cleanup_queue 触发）
            assert!(PendingProposalCleanups::<Test>::get(proposal_id).is_none());

            // set_status_and_emit 在 block 0 调用，cleanup_at = 0 + retention
            let retention = 90u64 * primitives::pow_const::BLOCKS_PER_DAY;
            let cleanup_block = retention;
            // 运行多轮 on_initialize 直到清理完成
            for i in 0..20u64 {
                System::set_block_number(cleanup_block + i);
                <VotingEngineSystem as Hooks<u64>>::on_initialize(cleanup_block + i);
                if PendingProposalCleanups::<Test>::get(proposal_id).is_none()
                    && Proposals::<Test>::get(proposal_id).is_none()
                {
                    break;
                }
            }

            assert!(PendingProposalCleanups::<Test>::get(proposal_id).is_none());
            assert!(!has_used_vote_nonce(
                proposal_id,
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-1"),
                "cleanup-nonce-1"
            ));
            assert!(!has_used_vote_nonce(
                proposal_id,
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-2"),
                "cleanup-nonce-2"
            ));
            assert!(!has_used_vote_nonce(
                proposal_id,
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-3"),
                "cleanup-nonce-3"
            ));
        });
    }

    #[test]
    fn store_and_get_proposal_data_works() {
        new_test_ext().execute_with(|| {
            assert!(VotingEngineSystem::get_proposal_data(0).is_none());

            let data = b"test proposal data".to_vec();
            assert_ok!(VotingEngineSystem::store_proposal_data(0, data.clone()));

            let stored = VotingEngineSystem::get_proposal_data(0).expect("data should exist");
            assert_eq!(&stored[..], &data[..]);

            // 覆盖
            let data2 = b"updated data".to_vec();
            assert_ok!(VotingEngineSystem::store_proposal_data(0, data2.clone()));
            let stored2 = VotingEngineSystem::get_proposal_data(0).expect("data should exist");
            assert_eq!(&stored2[..], &data2[..]);
        });
    }

    #[test]
    fn store_and_get_proposal_object_works() {
        new_test_ext().execute_with(|| {
            assert!(VotingEngineSystem::get_proposal_object(7).is_none());
            assert!(VotingEngineSystem::get_proposal_object_meta(7).is_none());

            let object = vec![1u8, 2, 3, 4, 5, 6];
            assert_ok!(VotingEngineSystem::store_proposal_object(
                7,
                1,
                object.clone()
            ));

            let stored = VotingEngineSystem::get_proposal_object(7).expect("object should exist");
            assert_eq!(stored, object);

            let meta = VotingEngineSystem::get_proposal_object_meta(7).expect("meta should exist");
            assert_eq!(meta.kind, 1);
            assert_eq!(meta.object_len, 6);
            assert_eq!(
                meta.object_hash,
                <Test as frame_system::Config>::Hashing::hash(&object)
            );

            VotingEngineSystem::remove_proposal_object(7);
            assert!(VotingEngineSystem::get_proposal_object(7).is_none());
            assert!(VotingEngineSystem::get_proposal_object_meta(7).is_none());
        });
    }

    #[test]
    fn store_proposal_meta_works() {
        new_test_ext().execute_with(|| {
            VotingEngineSystem::store_proposal_meta(42, 100);
            let meta = ProposalMeta::<Test>::get(42).expect("meta should exist");
            assert_eq!(meta.created_at, 100);
            assert!(meta.passed_at.is_none());

            VotingEngineSystem::set_proposal_passed(42, 200);
            let meta2 = ProposalMeta::<Test>::get(42).expect("meta should exist");
            assert_eq!(meta2.passed_at, Some(200));
        });
    }
}
