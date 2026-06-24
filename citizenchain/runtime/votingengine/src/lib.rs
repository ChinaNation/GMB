//! # 投票引擎 (votingengine)
//!
//! 投票基础设施模块，统一承载三类投票流程：
//! - **内部投票**（INTERNAL）：机构内部管理员按阈值投票，赞成 ≥ 阈值提前通过，
//!   剩余票不足达到阈值提前否决，30 天超时兜底否决。

//! - **联合投票**（JOINT）：国储会/省储会/省储行管理员按票权加权投票，
//!   105 票全票通过直接执行，任一机构反对立即进入联合公投，30 天超时进入联合公投。

//! - **公民投票**（CITIZEN）：CID 持有者按 >50% 严格多数投票，
//!   赞成 > 50% 提前通过，反对 ≥ 50% 提前否决，30 天超时按最终票数判定。
//!
//! 关键机制：
//! - **管理员快照锁定**：提案创建时锁定管理员名单，投票期间不受链上管理员更换影响。
//! - **联合提案发起权**：国储会和省储会管理员均可发起联合投票提案。
//!
//! 通过 trait 为上层治理模块提供标准化能力：
//! - `InternalVoteEngine` / `JointVoteEngine`：业务模块发起提案的内部入口;
//!   投票走对应 sub-pallet(`internal-vote::cast` / `joint-vote::cast_admin` /
//!   `joint-vote::cast_referendum`)的公开 extrinsic。
//! - `InternalVoteResultCallback` / `JointVoteResultCallback`:内部/联合提案
//!   完成投票判定时,投票引擎按统一状态机调用业务 executor。
//!   业务模块只返回统一执行结果，不再直接推进投票引擎状态；PASSED 表示执行授权/可重试态。
//! - 自动超时结算、原子终结 + 回调一致性(回调返回 Err 整体回滚)、
//!   90 天延迟分块清理。

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod cleanup;
pub mod data;
pub mod id;
pub mod index;
pub mod limit;
pub mod mutex;
pub mod snapshot;
pub mod traits;
pub mod types;
pub mod weights;

pub use pallet::*;
pub use traits::*;
pub use traits::{CidEligibility, VoteCredentialCleanup};
pub use types::*;

use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use frame_support::{
        pallet_prelude::*,
        storage::{with_transaction, TransactionOutcome},
        Blake2_128Concat,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{One, Saturating};
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

        /// 单个提案最多持有的内部互斥锁 binding 数量。
        #[pallet::constant]
        type MaxInternalProposalMutexBindings: Get<u32>;

        /// 每个机构最多允许同时存在的活跃提案数量。
        #[pallet::constant]
        type MaxActiveProposals: Get<u32>;

        /// 每个区块最多执行多少个清理步骤，避免历史提案清理拖垮 on_initialize。
        #[pallet::constant]
        type MaxCleanupStepsPerBlock: Get<u32>;

        /// 单个延迟清理到期桶最多挂载多少个 proposal_id。
        #[pallet::constant]
        type MaxCleanupQueueBucketLimit: Get<u32>;

        /// 延迟清理登记时最多向后顺延多少个区块桶。
        #[pallet::constant]
        type MaxCleanupScheduleOffset: Get<u32>;

        /// 每个清理步骤最多删除多少条前缀项。
        #[pallet::constant]
        type CleanupKeysPerStep: Get<u32>;

        /// 提案业务数据最大长度（字节），各业务模块序列化后的数据不超过此限制。
        #[pallet::constant]
        type MaxProposalDataLen: Get<u32>;

        /// 提案大对象数据最大长度（字节），用于 runtime wasm 等大载荷。
        #[pallet::constant]
        type MaxProposalObjectLen: Get<u32>;

        /// 业务模块标识最大长度，用于 ProposalOwner 绑定。
        #[pallet::constant]
        type MaxModuleTagLen: Get<u32>;

        /// 自动执行失败后允许的最大手动失败次数。
        #[pallet::constant]
        type MaxManualExecutionAttempts: Get<u32>;

        /// 自动执行失败后等待管理员手动执行的宽限区块数。
        #[pallet::constant]
        type ExecutionRetryGraceBlocks: Get<BlockNumberFor<Self>>;

        /// 单个区块最多处理多少个执行重试超时提案。
        #[pallet::constant]
        type MaxExecutionRetryDeadlinesPerBlock: Get<u32>;

        /// 每块最多处理多少个因 deadline 重排失败进入待处理队列的 retry 提案。
        #[pallet::constant]
        type MaxPendingRetryExpirationsPerBlock: Get<u32>;

        type CidEligibility: CidEligibility<Self::AccountId, Self::Hash>;
        type PopulationSnapshotVerifier: PopulationSnapshotVerifier<
            Self::AccountId,
            VoteNonceOf<Self>,
            VoteSignatureOf<Self>,
        >;

        type JointVoteResultCallback: JointVoteResultCallback;
        /// 内部投票终态回调(对称于 `JointVoteResultCallback`)。
        /// Runtime 用 tuple 注册多个业务模块的 Executor,投票引擎在提案进入
        /// `STATUS_PASSED` / `STATUS_REJECTED` 时广播到每个成员。
        type InternalVoteResultCallback: InternalVoteResultCallback;
        type InternalAdminProvider: InternalAdminProvider<Self::AccountId>;
        type InternalAdminsLenProvider: InternalAdminsLenProvider<Self::AccountId>;
        /// 每个机构最大管理员数量（与 admins-change 一致），用于管理员快照 BoundedVec。
        #[pallet::constant]
        type MaxAdminsPerInstitution: Get<u32>;

        /// 时间源，用于提案 ID 编码年份。
        type TimeProvider: frame_support::traits::UnixTime;

        type WeightInfo: crate::weights::WeightInfo;

        /// 内部投票 mode 超时结算回调,由 internal-vote pallet 实现。
        type InternalFinalizer: crate::traits::InternalProposalFinalizer<
            BlockNumberFor<Self>,
            Self::AccountId,
        >;

        /// 内部投票 mode chunked cleanup 派发,由 internal-vote pallet 实现。
        type InternalCleanup: crate::traits::InternalCleanupHandler;

        /// 联合投票 mode 超时结算回调,覆盖内部投票阶段(STAGE_JOINT)与联合公投阶段(STAGE_REFERENDUM),
        /// 由 joint-vote pallet 实现。
        type JointFinalizer: crate::traits::JointProposalFinalizer<
            BlockNumberFor<Self>,
            Self::AccountId,
        >;

        /// 联合投票 mode chunked cleanup 派发,由 joint-vote pallet 实现。
        type JointCleanup: crate::traits::JointCleanupHandler;

        /// 立法投票终态业务回调(ADR-027),由 legislation-yuan 业务壳实现。
        /// 核心在 PROPOSAL_KIND_LEGISLATION 提案达终态时按 kind 广播。第1步装 `()`。
        type LegislationVoteResultCallback: LegislationVoteResultCallback;

        /// 立法投票 mode 超时结算回调,覆盖内部表决(STAGE_LEG_HOUSE)与强制公投(STAGE_LEG_REFERENDUM),
        /// 由 legislation-vote pallet 实现。未实装时装 `()`。
        type LegislationFinalizer: crate::traits::LegislationProposalFinalizer<
            BlockNumberFor<Self>,
            Self::AccountId,
        >;

        /// 立法投票 mode chunked cleanup 派发,由 legislation-vote pallet 实现。未实装时装 `()`。
        type LegislationCleanup: crate::traits::LegislationCleanupHandler;
    }

    use crate::weights::WeightInfo;

    pub type VoteNonceOf<T> = BoundedVec<u8, <T as Config>::MaxVoteNonceLength>;
    pub type VoteSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxVoteSignatureLength>;

    /// VotingEngine 主 pallet on-chain storage 版本。
    ///
    /// 布局:提案主键纯单调 u64 + ProposalDisplayId 展示号 +
    /// ProposalsByCode/Institution/Owner/Year 4 张反向索引,创世直写,无历史回填。
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 当前提案年份（用于年度计数器重置）。
    #[pallet::storage]
    pub type CurrentProposalYear<T> = StorageValue<_, u16, ValueQuery>;

    /// 当前年份内的提案计数器（每年从 0 开始）。
    #[pallet::storage]
    pub type YearProposalCounter<T> = StorageValue<_, u32, ValueQuery>;

    /// 主键计数器:下一次 `allocate_proposal_id` 要返回的 u64。
    ///
    /// 双层 ID 设计:主键 `proposal_id` 全局单调累加,跨业务/跨年/跨机构唯一;
    /// 展示号 `(year, seq_in_year)` 单独存于 `ProposalDisplayId[id]` 反查表,
    /// 渲染层基于其拼成 "2026000123" 风格,**与主键解耦**。
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, u64, ValueQuery>;

    /// 全局提案表：proposal_id → 提案元数据（类型/阶段/状态/起止区块/机构等）。
    /// 由 `create_internal_proposal` 写入，`set_status_and_emit` 更新状态，超时清理自动删除。
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        Proposal<BlockNumberFor<T>, T::AccountId>,
        OptionQuery,
    >;

    /// 回调执行作用域：只在 `set_status_and_emit` 调业务回调期间临时存在。
    ///
    /// 中文注释：生产业务模块通过回调返回 `ProposalExecutionOutcome`；该作用域保护
    /// 测试和回调执行入口，避免非回调路径绕过最终事件和互斥锁释放逻辑。
    #[pallet::storage]
    pub type CallbackExecutionScopes<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, (), OptionQuery>;

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

    /// 提案管理员快照：提案创建时锁定各机构管理员名单，投票期间不随链上名单变化。
    /// 内部提案只存一条（提案所属机构），联合提案存所有参与机构（约105条）。
    /// 投票时查快照判定资格，保证管理员更换不影响已有提案的投票过程。
    #[pallet::storage]
    pub type AdminSnapshot<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<T::AccountId, T::MaxAdminsPerInstitution>,
        OptionQuery,
    >;

    /// 提案业务数据（由各业务模块序列化后写入，投票引擎统一存储和清理）。
    #[pallet::storage]
    #[pallet::getter(fn proposal_data)]
    pub type ProposalData<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BoundedVec<u8, T::MaxProposalDataLen>, OptionQuery>;

    /// 提案 owner：proposal_id → 业务模块 MODULE_TAG。
    ///
    /// 中文注释：ProposalOwner 是投票引擎分发自动执行、手动重试和取消的唯一归属来源。
    /// 业务模块不再只依赖 ProposalData 前缀自认领，避免跨模块覆写后静默跳过。
    #[pallet::storage]
    #[pallet::getter(fn proposal_owner)]
    pub type ProposalOwner<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BoundedVec<u8, T::MaxModuleTagLen>, OptionQuery>;

    /// 自动执行失败后的可重试状态。
    #[pallet::storage]
    #[pallet::getter(fn proposal_execution_retry_state)]
    pub type ProposalExecutionRetryStates<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ExecutionRetryState<BlockNumberFor<T>>, OptionQuery>;

    /// 执行重试 deadline 重排失败后的待处理队列。
    ///
    /// 中文注释：只要提案仍处于 PASSED + retry state，就必须保留一个可观测入口；
    /// 不能因为 deadline 桶连续满而丢失后续 on_initialize 处理机会。
    #[pallet::storage]
    pub type PendingExecutionRetryExpirations<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BlockNumberFor<T>, OptionQuery>;

    /// 执行失败终态通知失败后的待处理队列。
    ///
    /// 中文注释：提案已经进入 EXECUTION_FAILED 终态时，业务模块释放 pending 锁等
    /// 清理通知不能被静默吞掉；失败后保留 proposal_id，后续 on_initialize 有界重试。
    #[pallet::storage]
    pub type PendingTerminalCleanups<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, (), OptionQuery>;

    /// 执行重试超时队列：retry_deadline → proposal_id 列表。
    #[pallet::storage]
    #[pallet::getter(fn execution_retry_deadlines)]
    pub type ExecutionRetryDeadlines<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<u64, T::MaxExecutionRetryDeadlinesPerBlock>,
        ValueQuery,
    >;

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
        BoundedVec<u64, T::MaxCleanupQueueBucketLimit>,
        ValueQuery,
    >;

    /// 每个机构的活跃提案 ID 列表（全局管控，不区分提案类型，上限由 Runtime 配置）。
    #[pallet::storage]
    #[pallet::getter(fn active_proposals_by_institution)]
    pub type ActiveProposalsByInstitution<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, T::MaxActiveProposals>,
        ValueQuery,
    >;

    /// 同一治理账户的内部提案互斥状态：(institution_code, institution) → 锁状态。
    #[pallet::storage]
    #[pallet::getter(fn internal_proposal_mutex)]
    pub type InternalProposalMutexes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        InstitutionCode,
        Blake2_128Concat,
        T::AccountId,
        InternalProposalMutexState,
        OptionQuery,
    >;

    /// 提案持有的互斥锁列表，用于终态或联合投票进入联合公投阶段时释放。
    #[pallet::storage]
    #[pallet::getter(fn proposal_mutex_bindings)]
    pub type ProposalMutexBindings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<InternalProposalMutexBinding<T::AccountId>, T::MaxInternalProposalMutexBindings>,
        ValueQuery,
    >;

    // ──── 双层 ID 与反向索引(spec_version v1) ────

    /// 提案展示号:`proposal_id → (year, seq_in_year)`。
    ///
    /// 主键纯单调 u64 实质无上限;展示号在创建提案时同步写入,渲染走该表查询。
    /// 客户端"2026-#000123"格式由前端基于本表内容拼接,链端不持有展示格式。
    #[pallet::storage]
    pub type ProposalDisplayId<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ProposalDisplayMeta, OptionQuery>;

    /// 反向索引:institution_code → 该机构码下所有提案 ID。
    /// 客户端按"国储会/省储会/省储行/多签"分类查询时直接迭代该表,无需扫全表。
    #[pallet::storage]
    pub type ProposalsByCode<T: Config> =
        StorageDoubleMap<_, Twox64Concat, InstitutionCode, Twox64Concat, u64, (), OptionQuery>;

    /// 反向索引:institution(48 字节 PalletId) → 该机构所有提案 ID。
    /// 机构详情页直接迭代该表,不再走"全年扫描 + 客户端过滤"。
    #[pallet::storage]
    pub type ProposalsByInstitution<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, (), OptionQuery>;

    /// 反向索引:业务模块 MODULE_TAG → 该模块所有提案 ID。
    /// "只看 runtime 升级提案 / 只看决议销毁提案"等视图走该表。
    #[pallet::storage]
    pub type ProposalsByOwner<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        BoundedVec<u8, T::MaxModuleTagLen>,
        Twox64Concat,
        u64,
        (),
        OptionQuery,
    >;

    /// 反向索引:年份 → 该年所有提案 ID。
    /// 历史年份归档查询走该表,不再依赖主键的"年份编码"心智。
    #[pallet::storage]
    pub type ProposalsByYear<T: Config> =
        StorageDoubleMap<_, Twox64Concat, u16, Twox64Concat, u64, (), OptionQuery>;

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
        /// 中文注释：联合投票阶段非全票通过或超时，提案推进到联合公投阶段。
        ProposalAdvancedToCitizen {
            proposal_id: u64,
            citizen_end: BlockNumberFor<T>,
            eligible_total: u64,
        },
        /// 中文注释：投票阶段完成或执行状态变化；PASSED 是执行授权/可重试态，不是终态。
        ProposalFinalized { proposal_id: u64, status: u8 },
        /// 中文注释：自动执行失败，提案进入 PASSED 可重试态。
        ProposalExecutionRetryScheduled {
            proposal_id: u64,
            retry_deadline: BlockNumberFor<T>,
        },
        /// 中文注释：管理员手动执行已尝试。
        ProposalExecutionRetried {
            proposal_id: u64,
            manual_attempts: u8,
            outcome: u8,
        },
        /// 中文注释：PASSED 可重试提案超过宽限期，转入执行失败终态。
        ProposalExecutionRetryExpired { proposal_id: u64 },
        /// retry deadline 无法重新登记到 deadline 桶，已进入待处理重试队列。
        ProposalExecutionRetryExpirationQueued {
            proposal_id: u64,
            retry_deadline: BlockNumberFor<T>,
        },
        /// 中文注释：执行失败终态通知业务模块失败，已进入待处理重试队列。
        ProposalTerminalCleanupQueued { proposal_id: u64 },
        /// 中文注释：待处理的执行失败终态通知已补偿完成。
        ProposalTerminalCleanupCompleted { proposal_id: u64 },
        /// 中文注释：管理员取消 PASSED 可重试提案，转入执行失败终态。
        ProposalExecutionCancelled { proposal_id: u64 },
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
        /// 中文注释：内部投票管理员快照缺失。
        MissingAdminSnapshot,
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
        /// 中文注释：提案已终结，不可重复结算。
        ProposalAlreadyFinalized,
        /// 中文注释:提案主键 u64 单调累加溢出(实质永不发生,1.84×10¹⁹ 上限)。
        ProposalIdOverflow,
        /// 中文注释:年内累加器 u32 溢出(实质永不发生,42.9 亿/年上限)。
        YearCounterOverflow,
        /// 中文注释：单个到期区块的提案数超出上限。
        TooManyProposalsAtExpiry,
        /// 中文注释：该机构活跃提案数已达上限（10 个），需等待现有提案完成后再发起。
        ActiveProposalLimitReached,
        /// 中文注释：同一治理账户已有管理员集合变更提案活跃，普通提案需等待其结束。
        AdminSetMutationProposalActive,
        /// 中文注释：同一治理账户已有普通提案活跃，管理员更换需等待普通提案结束。
        RegularInternalProposalActive,
        /// 中文注释：内部提案互斥计数溢出。
        InternalProposalMutexOverflow,
        /// 中文注释：单个提案持有的互斥锁数量超出上限。
        TooManyInternalProposalMutexBindings,
        /// 中文注释：管理员更换提案不是当前治理账户的独占锁 owner。
        InternalProposalMutexOwnerMismatch,
        /// 中文注释：提案尚未绑定业务 owner。
        ProposalOwnerMissing,
        /// 中文注释：提案 owner 与当前业务模块不匹配。
        ProposalOwnerMismatch,
        /// 中文注释：提案业务数据已绑定，禁止跨模块覆写。
        ProposalDataAlreadyRegistered,
        /// 中文注释：提案不是可手动执行状态。
        ProposalNotRetryable,
        /// 中文注释：手动执行失败次数已达上限。
        ManualExecutionAttemptsExceeded,
        /// 中文注释：手动执行宽限期已过。
        ExecutionRetryDeadlinePassed,
        /// 中文注释：单个区块执行重试超时队列已满。
        TooManyExecutionRetryDeadlines,
        /// 中文注释：owner 模块没有明确允许取消该 PASSED 重试提案。
        ProposalCancellationNotAllowed,
        /// 中文注释：终态提案的延迟清理队列连续多个目标区块已满，必须回滚终态写入。
        CleanupQueueFull,
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

            weight = weight.saturating_add(Self::process_pending_execution_retry_expirations(n));

            weight = weight.saturating_add(Self::process_execution_retry_deadlines(n));

            weight = weight.saturating_add(Self::process_pending_terminal_cleanups());

            weight = weight.saturating_add(Self::process_pending_cleanup_steps());

            // 处理延迟清理队列：清理 90 天前完成的提案的全部数据
            weight = weight.saturating_add(cleanup::process_cleanup_queue::<T>(n));

            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // 引擎核心仅承载生命周期 extrinsic(超时结算 / 重试 / 取消)。
        // mode-specific 投票 extrinsic 由各 sub-pallet 提供:
        //   - InternalVote::cast(22.0)
        //   - JointVote::cast_admin(23.0)
        //   - JointVote::cast_referendum(23.1)
        // call_index 从 3 起,0/1/2 留空。

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::finalize_proposal())]
        pub fn finalize_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            match proposal.stage {
                STAGE_INTERNAL => {
                    <T::InternalFinalizer as crate::traits::InternalProposalFinalizer<
                        BlockNumberFor<T>,
                        T::AccountId,
                    >>::finalize_internal_timeout(&proposal, proposal_id)?;
                }
                STAGE_JOINT => {
                    <T::JointFinalizer as crate::traits::JointProposalFinalizer<
                        BlockNumberFor<T>,
                        T::AccountId,
                    >>::finalize_joint_timeout(&proposal, proposal_id)?;
                }
                STAGE_REFERENDUM => {
                    <T::JointFinalizer as crate::traits::JointProposalFinalizer<
                        BlockNumberFor<T>,
                        T::AccountId,
                    >>::finalize_jointreferendum_timeout(
                        &proposal, proposal_id
                    )?;
                }
                STAGE_LEG_HOUSE => {
                    <T::LegislationFinalizer as crate::traits::LegislationProposalFinalizer<
                        BlockNumberFor<T>,
                        T::AccountId,
                    >>::finalize_legislation_house_timeout(
                        &proposal, proposal_id
                    )?;
                }
                STAGE_LEG_REFERENDUM => {
                    <T::LegislationFinalizer as crate::traits::LegislationProposalFinalizer<
                        BlockNumberFor<T>,
                        T::AccountId,
                    >>::finalize_legislation_referendum_timeout(
                        &proposal, proposal_id
                    )?;
                }
                _ => return Err(Error::<T>::InvalidProposalStage.into()),
            }

            // weight 已在 #[pallet::weight] 中静态指定为 max(三模式),无需返回 actual_weight。
            Ok(().into())
        }

        /// 统一手动执行已通过但自动执行失败的提案。
        ///
        /// 中文注释：业务模块不得再各自暴露 execute_xxx 重试入口；所有手动执行
        /// 都必须经过投票引擎校验 PASSED 状态、管理员权限、重试次数和宽限期。
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::retry_passed_proposal())]
        pub fn retry_passed_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::retry_passed_proposal_inner(&who, proposal_id)
        }

        /// 统一取消已通过但无法继续执行的提案。
        ///
        /// 中文注释：取消只允许 `PASSED -> EXECUTION_FAILED`，进入执行失败终态后
        /// 不再允许重试或再次取消。
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::cancel_passed_proposal())]
        pub fn cancel_passed_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
            _reason: BoundedVec<u8, T::MaxProposalDataLen>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::cancel_passed_proposal_inner(&who, proposal_id)
        }
    }

    impl<T: Config> Pallet<T> {
        // ──────────────────────────────────────────────────────────
        // sub-pallet 调用的事件 emit helper(do_X 搬到 sub-pallet 后,
        // 仍需要发 votingengine 自己的 lifecycle event)
        // ──────────────────────────────────────────────────────────

        pub fn emit_proposal_created(
            proposal_id: u64,
            kind: u8,
            stage: u8,
            end: BlockNumberFor<T>,
        ) {
            Self::deposit_event(Event::<T>::ProposalCreated {
                proposal_id,
                kind,
                stage,
                end,
            });
        }

        pub fn emit_proposal_advanced_to_citizen(
            proposal_id: u64,
            citizen_end: BlockNumberFor<T>,
            eligible_total: u64,
        ) {
            Self::deposit_event(Event::<T>::ProposalAdvancedToCitizen {
                proposal_id,
                citizen_end,
                eligible_total,
            });
        }

        pub fn schedule_proposal_expiry(
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
                    STAGE_INTERNAL => {
                        <T::InternalFinalizer as crate::traits::InternalProposalFinalizer<
                            BlockNumberFor<T>,
                            T::AccountId,
                        >>::finalize_internal_timeout(&proposal, proposal_id)
                    }
                    STAGE_JOINT => {
                        <T::JointFinalizer as crate::traits::JointProposalFinalizer<
                            BlockNumberFor<T>,
                            T::AccountId,
                        >>::finalize_joint_timeout(&proposal, proposal_id)
                    }
                    STAGE_REFERENDUM => {
                        <T::JointFinalizer as crate::traits::JointProposalFinalizer<
                            BlockNumberFor<T>,
                            T::AccountId,
                        >>::finalize_jointreferendum_timeout(
                            &proposal, proposal_id
                        )
                    }
                    STAGE_LEG_HOUSE => {
                        <T::LegislationFinalizer as crate::traits::LegislationProposalFinalizer<
                            BlockNumberFor<T>,
                            T::AccountId,
                        >>::finalize_legislation_house_timeout(
                            &proposal, proposal_id
                        )
                    }
                    STAGE_LEG_REFERENDUM => {
                        <T::LegislationFinalizer as crate::traits::LegislationProposalFinalizer<
                            BlockNumberFor<T>,
                            T::AccountId,
                        >>::finalize_legislation_referendum_timeout(
                            &proposal, proposal_id
                        )
                    }
                    _ => Ok(()),
                };
                if finalize_result.is_err() {
                    // 中文注释：终结失败时必须保留自动重试索引，
                    // 避免提案状态仍是 Voting，但后续再也不会被 on_initialize 处理。
                    retry_ids.push(proposal_id);
                }
            }
            for proposal_id in retry_ids {
                if proposal_ids.try_push(proposal_id).is_err() {
                    frame_support::defensive!(
                        "auto_finalize_expiry_bucket: retry id should fit drained expiry bucket"
                    );
                }
            }

            let has_remaining = !proposal_ids.is_empty();
            if has_remaining {
                ProposalsByExpiry::<T>::insert(expiry, proposal_ids);
                weight = weight.saturating_add(db_weight.writes(1));
            }

            // mode-specific 权重住在各 sub-pallet,引擎核心用统一 finalize_proposal
            // 静态权重作保守上界。
            let finalize_weight =
                T::WeightInfo::finalize_proposal().saturating_mul(process_count as u64);
            weight = weight.saturating_add(finalize_weight);

            (process_count, has_remaining, weight)
        }

        pub fn ensure_open_proposal(
            proposal_id: u64,
        ) -> Result<Proposal<BlockNumberFor<T>, T::AccountId>, DispatchError> {
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

        fn should_release_internal_proposal_mutexes(kind: u8, stage: u8, final_status: u8) -> bool {
            matches!(
                final_status,
                STATUS_REJECTED | STATUS_EXECUTED | STATUS_EXECUTION_FAILED
            ) || (kind == PROPOSAL_KIND_JOINT
                && stage == STAGE_JOINT
                && final_status == STATUS_PASSED)
        }

        fn ensure_valid_status_transition(old_status: u8, new_status: u8) -> DispatchResult {
            ensure!(
                matches!(
                    (old_status, new_status),
                    (STATUS_VOTING, STATUS_PASSED)
                        | (STATUS_VOTING, STATUS_REJECTED)
                        | (STATUS_PASSED, STATUS_EXECUTED)
                        | (STATUS_PASSED, STATUS_EXECUTION_FAILED)
                ),
                Error::<T>::InvalidProposalStatus
            );
            Ok(())
        }

        fn is_terminal_status(status: u8) -> bool {
            matches!(
                status,
                STATUS_REJECTED | STATUS_EXECUTED | STATUS_EXECUTION_FAILED
            )
        }

        pub fn mark_proposal_passed_at(proposal_id: u64, block: BlockNumberFor<T>) {
            ProposalMeta::<T>::mutate(proposal_id, |meta| {
                if let Some(m) = meta {
                    if m.passed_at.is_none() {
                        m.passed_at = Some(block);
                    }
                }
            });
        }

        fn set_proposal_status(proposal_id: u64, status: u8) -> DispatchResult {
            Proposals::<T>::try_mutate(proposal_id, |maybe| {
                let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                Self::ensure_valid_status_transition(proposal.status, status)?;
                proposal.status = status;
                Ok(())
            })
        }

        fn apply_terminal_side_effects(proposal_id: u64, status: u8) -> DispatchResult {
            ensure!(
                Self::is_terminal_status(status),
                Error::<T>::InvalidProposalStatus
            );
            let now = frame_system::Pallet::<T>::block_number();
            cleanup::schedule_cleanup::<T>(proposal_id, now)?;
            ProposalExecutionRetryStates::<T>::remove(proposal_id);
            PendingExecutionRetryExpirations::<T>::remove(proposal_id);
            PendingTerminalCleanups::<T>::remove(proposal_id);
            if status == STATUS_EXECUTION_FAILED {
                if let Some(proposal) = Proposals::<T>::get(proposal_id) {
                    // 中文注释：清理登记成功后再通知业务模块释放 pending 锁，
                    // 避免先产生业务侧副作用、再发现链上清理无法登记。通知失败
                    // 不再吞掉，而是进入有界重试队列。
                    Self::notify_execution_failed_terminal_or_queue(proposal_id, proposal.kind);
                }
            }
            if let Some(proposal) = Proposals::<T>::get(proposal_id) {
                if proposal.kind == PROPOSAL_KIND_INTERNAL {
                    <T::InternalCleanup as crate::traits::InternalCleanupHandler>::on_internal_proposal_terminal(
                        proposal_id,
                        status,
                    )?;
                }
                if Self::should_release_internal_proposal_mutexes(
                    proposal.kind,
                    proposal.stage,
                    status,
                ) {
                    Self::release_internal_proposal_mutexes(proposal_id);
                }
            }
            Ok(())
        }

        fn queue_execution_retry_deadline(
            proposal_id: u64,
            target: BlockNumberFor<T>,
        ) -> DispatchResult {
            Ok(ExecutionRetryDeadlines::<T>::try_mutate(target, |ids| {
                ids.try_push(proposal_id)
                    .map_err(|_| Error::<T>::TooManyExecutionRetryDeadlines)
            })?)
        }

        fn reschedule_execution_retry_deadline(
            proposal_id: u64,
            from: BlockNumberFor<T>,
        ) -> DispatchResult {
            let mut target = from;
            for _ in 0..100u32 {
                if Self::queue_execution_retry_deadline(proposal_id, target).is_ok() {
                    return Ok(());
                }
                target = target.saturating_add(BlockNumberFor::<T>::one());
            }
            Err(Error::<T>::TooManyExecutionRetryDeadlines.into())
        }

        fn queue_pending_retry_expiration(proposal_id: u64, retry_deadline: BlockNumberFor<T>) {
            PendingExecutionRetryExpirations::<T>::insert(proposal_id, retry_deadline);
            Self::deposit_event(Event::<T>::ProposalExecutionRetryExpirationQueued {
                proposal_id,
                retry_deadline,
            });
        }

        fn finish_terminal_status(proposal_id: u64, status: u8) -> DispatchResult {
            Self::apply_terminal_side_effects(proposal_id, status)?;
            Self::deposit_event(Event::<T>::ProposalFinalized {
                proposal_id,
                status,
            });
            Ok(())
        }

        fn ensure_retry_admin(who: &T::AccountId, proposal_id: u64) -> DispatchResult {
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            let institution = proposal
                .internal_institution
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_admin_in_snapshot(proposal_id, institution, who),
                Error::<T>::NoPermission
            );
            Ok(())
        }

        fn invoke_execution_callback(
            proposal_id: u64,
            kind: u8,
            approved: bool,
        ) -> Result<ProposalExecutionOutcome, DispatchError> {
            match kind {
                PROPOSAL_KIND_INTERNAL => {
                    T::InternalVoteResultCallback::on_internal_vote_finalized(proposal_id, approved)
                }
                PROPOSAL_KIND_JOINT => {
                    T::JointVoteResultCallback::on_joint_vote_finalized(proposal_id, approved)
                }
                PROPOSAL_KIND_LEGISLATION => {
                    T::LegislationVoteResultCallback::on_legislation_vote_finalized(
                        proposal_id,
                        approved,
                    )
                }
                _ => Err(Error::<T>::InvalidProposalKind.into()),
            }
        }

        fn can_cancel_passed_proposal_by_owner(proposal_id: u64, kind: u8) -> DispatchResult {
            let decision = match kind {
                PROPOSAL_KIND_INTERNAL => {
                    T::InternalVoteResultCallback::can_cancel_passed_proposal(proposal_id)
                }
                PROPOSAL_KIND_JOINT => {
                    T::JointVoteResultCallback::can_cancel_passed_proposal(proposal_id)
                }
                PROPOSAL_KIND_LEGISLATION => {
                    T::LegislationVoteResultCallback::can_cancel_passed_proposal(proposal_id)
                }
                _ => Err(Error::<T>::InvalidProposalKind.into()),
            }?;
            ensure!(
                decision == ProposalCancelDecision::Allow,
                Error::<T>::ProposalCancellationNotAllowed
            );
            Ok(())
        }

        fn notify_execution_failed_terminal(proposal_id: u64, kind: u8) -> DispatchResult {
            match kind {
                PROPOSAL_KIND_INTERNAL => {
                    T::InternalVoteResultCallback::on_execution_failed_terminal(proposal_id)
                }
                PROPOSAL_KIND_JOINT => {
                    T::JointVoteResultCallback::on_execution_failed_terminal(proposal_id)
                }
                PROPOSAL_KIND_LEGISLATION => {
                    T::LegislationVoteResultCallback::on_execution_failed_terminal(proposal_id)
                }
                _ => Err(Error::<T>::InvalidProposalKind.into()),
            }
        }

        fn queue_terminal_cleanup(proposal_id: u64) {
            let already_pending = PendingTerminalCleanups::<T>::contains_key(proposal_id);
            PendingTerminalCleanups::<T>::insert(proposal_id, ());
            if !already_pending {
                Self::deposit_event(Event::<T>::ProposalTerminalCleanupQueued { proposal_id });
            }
        }

        fn notify_execution_failed_terminal_or_queue(proposal_id: u64, kind: u8) {
            let result = Self::with_callback_execution_scope(proposal_id, || {
                Self::notify_execution_failed_terminal(proposal_id, kind)
            });
            if result.is_ok() {
                PendingTerminalCleanups::<T>::remove(proposal_id);
                return;
            }
            Self::queue_terminal_cleanup(proposal_id);
        }

        fn process_pending_terminal_cleanups() -> Weight {
            let db_weight = T::DbWeight::get();
            let mut weight = db_weight.reads(1);
            let max = T::MaxPendingRetryExpirationsPerBlock::get() as usize;
            if max == 0 {
                return weight;
            }

            let pending: Vec<u64> = PendingTerminalCleanups::<T>::iter()
                .take(max)
                .map(|(proposal_id, _)| proposal_id)
                .collect();
            for proposal_id in pending {
                weight = weight.saturating_add(db_weight.reads_writes(2, 3));
                let Some(proposal) = Proposals::<T>::get(proposal_id) else {
                    PendingTerminalCleanups::<T>::remove(proposal_id);
                    continue;
                };
                if proposal.status != STATUS_EXECUTION_FAILED {
                    PendingTerminalCleanups::<T>::remove(proposal_id);
                    continue;
                }
                let result = Self::with_callback_execution_scope(proposal_id, || {
                    Self::notify_execution_failed_terminal(proposal_id, proposal.kind)
                });
                if result.is_ok() {
                    PendingTerminalCleanups::<T>::remove(proposal_id);
                    Self::deposit_event(Event::<T>::ProposalTerminalCleanupCompleted {
                        proposal_id,
                    });
                }
            }
            weight
        }

        fn schedule_execution_retry(proposal_id: u64) -> DispatchResult {
            if ProposalExecutionRetryStates::<T>::contains_key(proposal_id) {
                return Ok(());
            }
            let now = frame_system::Pallet::<T>::block_number();
            let retry_deadline = now.saturating_add(T::ExecutionRetryGraceBlocks::get());
            let state = ExecutionRetryState {
                manual_attempts: 0,
                first_auto_failed_at: now,
                retry_deadline,
                last_attempt_at: None,
            };
            if Self::reschedule_execution_retry_deadline(proposal_id, retry_deadline).is_err() {
                Self::queue_pending_retry_expiration(proposal_id, retry_deadline);
            }
            ProposalExecutionRetryStates::<T>::insert(proposal_id, state);
            Self::deposit_event(Event::<T>::ProposalExecutionRetryScheduled {
                proposal_id,
                retry_deadline,
            });
            Ok(())
        }

        fn apply_automatic_execution_outcome(
            proposal_id: u64,
            kind: u8,
            outcome: ProposalExecutionOutcome,
        ) -> DispatchResult {
            match outcome {
                ProposalExecutionOutcome::Ignored => Err(Error::<T>::ProposalOwnerMissing.into()),
                ProposalExecutionOutcome::Executed => {
                    Self::set_proposal_status(proposal_id, STATUS_EXECUTED)?;
                    if kind == PROPOSAL_KIND_INTERNAL {
                        <T::InternalCleanup as crate::traits::InternalCleanupHandler>::on_internal_proposal_executed(
                            proposal_id,
                        )?;
                    }
                    Ok(())
                }
                ProposalExecutionOutcome::RetryableFailed => {
                    if kind == PROPOSAL_KIND_INTERNAL {
                        Self::schedule_execution_retry(proposal_id)
                    } else {
                        // 中文注释：当前统一 retry/cancel 管理员权限只支持内部提案；
                        // joint callback 若误返回 RetryableFailed，立即失败终态，避免 PASSED 卡死。
                        Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)
                    }
                }
                ProposalExecutionOutcome::FatalFailed => {
                    Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)
                }
            }
        }

        fn process_execution_retry_deadlines(now: BlockNumberFor<T>) -> Weight {
            let db_weight = T::DbWeight::get();
            let mut weight = db_weight.reads_writes(1, 1);
            let queue = ExecutionRetryDeadlines::<T>::take(now);
            if queue.is_empty() {
                return weight;
            }

            for proposal_id in queue.into_iter() {
                weight = weight.saturating_add(db_weight.reads_writes(2, 3));
                let Some(state) = ProposalExecutionRetryStates::<T>::get(proposal_id) else {
                    continue;
                };
                if state.retry_deadline > now {
                    if Self::reschedule_execution_retry_deadline(proposal_id, state.retry_deadline)
                        .is_err()
                    {
                        Self::queue_pending_retry_expiration(proposal_id, state.retry_deadline);
                    }
                    continue;
                }
                let Some(proposal) = Proposals::<T>::get(proposal_id) else {
                    ProposalExecutionRetryStates::<T>::remove(proposal_id);
                    continue;
                };
                if proposal.status != STATUS_PASSED {
                    ProposalExecutionRetryStates::<T>::remove(proposal_id);
                    continue;
                }
                let result = with_transaction(|| {
                    let result = (|| -> DispatchResult {
                        Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)?;
                        Self::deposit_event(Event::<T>::ProposalExecutionRetryExpired {
                            proposal_id,
                        });
                        Self::finish_terminal_status(proposal_id, STATUS_EXECUTION_FAILED)
                    })();
                    match result {
                        Ok(()) => TransactionOutcome::Commit(Ok(())),
                        Err(err) => TransactionOutcome::Rollback(Err(err)),
                    }
                });
                if result.is_err() {
                    let next_block = now.saturating_add(BlockNumberFor::<T>::one());
                    if Self::reschedule_execution_retry_deadline(proposal_id, next_block).is_err() {
                        Self::queue_pending_retry_expiration(proposal_id, state.retry_deadline);
                    }
                }
            }
            weight
        }

        fn process_pending_execution_retry_expirations(now: BlockNumberFor<T>) -> Weight {
            let db_weight = T::DbWeight::get();
            let mut weight = db_weight.reads(1);
            let max = T::MaxPendingRetryExpirationsPerBlock::get() as usize;
            if max == 0 {
                return weight;
            }

            let pending: sp_std::vec::Vec<_> = PendingExecutionRetryExpirations::<T>::iter()
                .take(max)
                .collect();
            for (proposal_id, retry_deadline) in pending {
                weight = weight.saturating_add(db_weight.reads_writes(3, 4));
                let Some(state) = ProposalExecutionRetryStates::<T>::get(proposal_id) else {
                    PendingExecutionRetryExpirations::<T>::remove(proposal_id);
                    continue;
                };
                if state.retry_deadline > now {
                    if Self::reschedule_execution_retry_deadline(proposal_id, state.retry_deadline)
                        .is_ok()
                    {
                        PendingExecutionRetryExpirations::<T>::remove(proposal_id);
                    } else {
                        PendingExecutionRetryExpirations::<T>::insert(
                            proposal_id,
                            state.retry_deadline,
                        );
                    }
                    continue;
                }
                let Some(proposal) = Proposals::<T>::get(proposal_id) else {
                    ProposalExecutionRetryStates::<T>::remove(proposal_id);
                    PendingExecutionRetryExpirations::<T>::remove(proposal_id);
                    continue;
                };
                if proposal.status != STATUS_PASSED {
                    ProposalExecutionRetryStates::<T>::remove(proposal_id);
                    PendingExecutionRetryExpirations::<T>::remove(proposal_id);
                    continue;
                }

                let result = with_transaction(|| {
                    let result = (|| -> DispatchResult {
                        Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)?;
                        Self::deposit_event(Event::<T>::ProposalExecutionRetryExpired {
                            proposal_id,
                        });
                        Self::finish_terminal_status(proposal_id, STATUS_EXECUTION_FAILED)
                    })();
                    match result {
                        Ok(()) => TransactionOutcome::Commit(Ok(())),
                        Err(err) => TransactionOutcome::Rollback(Err(err)),
                    }
                });
                if result.is_ok() {
                    PendingExecutionRetryExpirations::<T>::remove(proposal_id);
                } else {
                    PendingExecutionRetryExpirations::<T>::insert(proposal_id, retry_deadline);
                }
            }
            weight
        }

        fn retry_passed_proposal_inner(who: &T::AccountId, proposal_id: u64) -> DispatchResult {
            with_transaction(|| {
                let result = (|| -> DispatchResult {
                    Self::ensure_retry_admin(who, proposal_id)?;
                    let proposal =
                        Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
                    ensure!(
                        proposal.status == STATUS_PASSED,
                        Error::<T>::ProposalNotRetryable
                    );
                    let mut state = ProposalExecutionRetryStates::<T>::get(proposal_id)
                        .ok_or(Error::<T>::ProposalNotRetryable)?;
                    let now = frame_system::Pallet::<T>::block_number();
                    ensure!(
                        now <= state.retry_deadline,
                        Error::<T>::ExecutionRetryDeadlinePassed
                    );
                    ensure!(
                        u32::from(state.manual_attempts) < T::MaxManualExecutionAttempts::get(),
                        Error::<T>::ManualExecutionAttemptsExceeded
                    );

                    let outcome = Self::with_callback_execution_scope(proposal_id, || {
                        Self::invoke_execution_callback(proposal_id, proposal.kind, true)
                    })?;
                    match outcome {
                        ProposalExecutionOutcome::Executed => {
                            Self::set_proposal_status(proposal_id, STATUS_EXECUTED)?;
                            if proposal.kind == PROPOSAL_KIND_INTERNAL {
                                <T::InternalCleanup as crate::traits::InternalCleanupHandler>::on_internal_proposal_executed(
                                    proposal_id,
                                )?;
                            }
                            Self::deposit_event(Event::<T>::ProposalExecutionRetried {
                                proposal_id,
                                manual_attempts: state.manual_attempts,
                                outcome: STATUS_EXECUTED,
                            });
                            Self::finish_terminal_status(proposal_id, STATUS_EXECUTED)
                        }
                        ProposalExecutionOutcome::RetryableFailed => {
                            state.manual_attempts = state.manual_attempts.saturating_add(1);
                            state.last_attempt_at = Some(now);
                            if u32::from(state.manual_attempts)
                                >= T::MaxManualExecutionAttempts::get()
                            {
                                Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)?;
                                Self::deposit_event(Event::<T>::ProposalExecutionRetried {
                                    proposal_id,
                                    manual_attempts: state.manual_attempts,
                                    outcome: STATUS_EXECUTION_FAILED,
                                });
                                Self::finish_terminal_status(proposal_id, STATUS_EXECUTION_FAILED)
                            } else {
                                Self::deposit_event(Event::<T>::ProposalExecutionRetried {
                                    proposal_id,
                                    manual_attempts: state.manual_attempts,
                                    outcome: STATUS_PASSED,
                                });
                                ProposalExecutionRetryStates::<T>::insert(proposal_id, state);
                                Ok(())
                            }
                        }
                        ProposalExecutionOutcome::FatalFailed => {
                            Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)?;
                            Self::deposit_event(Event::<T>::ProposalExecutionRetried {
                                proposal_id,
                                manual_attempts: state.manual_attempts,
                                outcome: STATUS_EXECUTION_FAILED,
                            });
                            Self::finish_terminal_status(proposal_id, STATUS_EXECUTION_FAILED)
                        }
                        ProposalExecutionOutcome::Ignored => {
                            Err(Error::<T>::ProposalOwnerMissing.into())
                        }
                    }
                })();
                match result {
                    Ok(()) => TransactionOutcome::Commit(Ok(())),
                    Err(err) => TransactionOutcome::Rollback(Err(err)),
                }
            })
        }

        fn cancel_passed_proposal_inner(who: &T::AccountId, proposal_id: u64) -> DispatchResult {
            with_transaction(|| {
                let result = (|| -> DispatchResult {
                    Self::ensure_retry_admin(who, proposal_id)?;
                    let proposal =
                        Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
                    ensure!(
                        proposal.status == STATUS_PASSED,
                        Error::<T>::ProposalNotRetryable
                    );
                    Self::can_cancel_passed_proposal_by_owner(proposal_id, proposal.kind)?;
                    Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)?;
                    Self::deposit_event(Event::<T>::ProposalExecutionCancelled { proposal_id });
                    Self::finish_terminal_status(proposal_id, STATUS_EXECUTION_FAILED)
                })();
                match result {
                    Ok(()) => TransactionOutcome::Commit(Ok(())),
                    Err(err) => TransactionOutcome::Rollback(Err(err)),
                }
            })
        }

        /// 查询当前是否处于某个提案的业务回调/终态清理作用域。
        ///
        /// 中文注释：业务 pallet 用它保护敏感生命周期写入，避免普通 runtime 调用绕过投票引擎。
        pub fn is_callback_execution_scope(proposal_id: u64) -> bool {
            CallbackExecutionScopes::<T>::contains_key(proposal_id)
        }

        fn with_callback_execution_scope<F, R>(
            proposal_id: u64,
            callback: F,
        ) -> Result<R, DispatchError>
        where
            F: FnOnce() -> Result<R, DispatchError>,
        {
            CallbackExecutionScopes::<T>::insert(proposal_id, ());
            let result = callback();
            CallbackExecutionScopes::<T>::remove(proposal_id);
            result
        }

        /// 更新提案状态，并按统一 executor 结果推进业务执行状态。
        pub fn set_status_and_emit(proposal_id: u64, status: u8) -> DispatchResult {
            with_transaction(|| {
                let (kind, stage, institution, should_run_callback) =
                    match Proposals::<T>::try_mutate(
                        proposal_id,
                        |maybe| -> Result<(u8, u8, Option<T::AccountId>, bool), DispatchError> {
                            let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                            let old_status = proposal.status;
                            Self::ensure_valid_status_transition(old_status, status)?;
                            let kind = proposal.kind;
                            let stage = proposal.stage;
                            let inst = proposal.internal_institution.clone();
                            proposal.status = status;
                            if old_status == STATUS_VOTING && status == STATUS_PASSED {
                                let now = frame_system::Pallet::<T>::block_number();
                                Self::mark_proposal_passed_at(proposal_id, now);
                            }
                            Ok((
                                kind,
                                stage,
                                inst,
                                old_status == STATUS_VOTING
                                    && matches!(status, STATUS_PASSED | STATUS_REJECTED),
                            ))
                        },
                    ) {
                        Ok(v) => v,
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    };

                // 提案结束（通过或拒绝），立即释放活跃提案名额
                if status != STATUS_VOTING {
                    if let Some(inst) = institution {
                        limit::remove_active_proposal::<T>(inst, proposal_id);
                    }
                }

                if should_run_callback {
                    let outcome = match Self::with_callback_execution_scope(proposal_id, || {
                        Self::invoke_execution_callback(proposal_id, kind, status == STATUS_PASSED)
                    }) {
                        Ok(outcome) => outcome,
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    };
                    if status == STATUS_PASSED {
                        if let Err(err) =
                            Self::apply_automatic_execution_outcome(proposal_id, kind, outcome)
                        {
                            return TransactionOutcome::Rollback(Err(err));
                        }
                    }
                }

                let final_status = match Proposals::<T>::get(proposal_id) {
                    Some(proposal) => proposal.status,
                    None => {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::ProposalNotFound.into()
                        ))
                    }
                };
                // 中文注释：PASSED 是执行授权/可重试态，不再视为终态。
                // 90 天延迟清理只登记 REJECTED / EXECUTED / EXECUTION_FAILED。
                if Self::is_terminal_status(final_status) {
                    if let Err(err) = Self::apply_terminal_side_effects(proposal_id, final_status) {
                        return TransactionOutcome::Rollback(Err(err));
                    }
                } else if Self::should_release_internal_proposal_mutexes(kind, stage, final_status)
                {
                    Self::release_internal_proposal_mutexes(proposal_id);
                }
                Self::deposit_event(Event::<T>::ProposalFinalized {
                    proposal_id,
                    status: final_status,
                });

                TransactionOutcome::Commit(Ok(()))
            })
        }

        /// 回调专用执行结果写入。
        ///
        /// 中文注释：仅供单测验证旧回调作用域保护；生产业务回调应直接返回
        /// `ProposalExecutionOutcome`，由外层 `set_status_and_emit` 统一收口状态、事件和清理。
        #[cfg(test)]
        pub fn set_callback_execution_result(proposal_id: u64, final_status: u8) -> DispatchResult {
            ensure!(
                CallbackExecutionScopes::<T>::contains_key(proposal_id),
                Error::<T>::InvalidProposalStatus
            );
            ensure!(
                matches!(final_status, STATUS_EXECUTED | STATUS_EXECUTION_FAILED),
                Error::<T>::InvalidProposalStatus
            );
            Proposals::<T>::try_mutate(proposal_id, |maybe| {
                let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                Self::ensure_valid_status_transition(proposal.status, final_status)?;
                proposal.status = final_status;
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
                PendingCleanupStage::AdminSnapshots => {
                    let result = AdminSnapshot::<T>::clear_prefix(proposal_id, cleanup_limit, None);
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::AdminSnapshots)
                    } else {
                        Some(PendingCleanupStage::InternalVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::InternalVotes => {
                    let (removed, has_remaining) =
                        <T::InternalCleanup as crate::traits::InternalCleanupHandler>::cleanup_internal_votes_chunk(
                            proposal_id, cleanup_limit,
                        );
                    let weight = db_weight.reads_writes(u64::from(removed), u64::from(removed));
                    let next = if has_remaining {
                        Some(PendingCleanupStage::InternalVotes)
                    } else {
                        Some(PendingCleanupStage::JointAdminVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointAdminVotes => {
                    let (removed, has_remaining) =
                        <T::JointCleanup as crate::traits::JointCleanupHandler>::cleanup_joint_admin_votes_chunk(
                            proposal_id, cleanup_limit,
                        );
                    let weight = db_weight.reads_writes(u64::from(removed), u64::from(removed));
                    let next = if has_remaining {
                        Some(PendingCleanupStage::JointAdminVotes)
                    } else {
                        Some(PendingCleanupStage::JointInstitutionVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointInstitutionVotes => {
                    let (removed, has_remaining) =
                        <T::JointCleanup as crate::traits::JointCleanupHandler>::cleanup_joint_institution_votes_chunk(
                            proposal_id, cleanup_limit,
                        );
                    let weight = db_weight.reads_writes(u64::from(removed), u64::from(removed));
                    let next = if has_remaining {
                        Some(PendingCleanupStage::JointInstitutionVotes)
                    } else {
                        Some(PendingCleanupStage::JointInstitutionTallies)
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointInstitutionTallies => {
                    let (removed, has_remaining) =
                        <T::JointCleanup as crate::traits::JointCleanupHandler>::cleanup_joint_institution_tallies_chunk(
                            proposal_id, cleanup_limit,
                        );
                    let weight = db_weight.reads_writes(u64::from(removed), u64::from(removed));
                    let next = if has_remaining {
                        Some(PendingCleanupStage::JointInstitutionTallies)
                    } else {
                        Some(PendingCleanupStage::CitizenVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::CitizenVotes => {
                    let (removed, has_remaining) =
                        <T::JointCleanup as crate::traits::JointCleanupHandler>::cleanup_referendum_votes_chunk(
                            proposal_id, cleanup_limit,
                        );
                    let weight = db_weight.reads_writes(u64::from(removed), u64::from(removed));
                    let next = if has_remaining {
                        Some(PendingCleanupStage::CitizenVotes)
                    } else {
                        Some(PendingCleanupStage::VoteCredentials)
                    };
                    (next, weight)
                }
                PendingCleanupStage::VoteCredentials => {
                    let result = T::CidEligibility::cleanup_vote_credentials_chunk(
                        proposal_id,
                        cleanup_limit,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.removed));
                    let next = if result.has_remaining {
                        Some(PendingCleanupStage::VoteCredentials)
                    } else {
                        Some(PendingCleanupStage::LegislationHouseVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::LegislationHouseVotes => {
                    let (removed, has_remaining) =
                        <T::LegislationCleanup as crate::traits::LegislationCleanupHandler>::cleanup_legislation_house_votes_chunk(
                            proposal_id, cleanup_limit,
                        );
                    let weight = db_weight.reads_writes(u64::from(removed), u64::from(removed));
                    let next = if has_remaining {
                        Some(PendingCleanupStage::LegislationHouseVotes)
                    } else {
                        Some(PendingCleanupStage::LegislationReferendumVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::LegislationReferendumVotes => {
                    let (removed, has_remaining) =
                        <T::LegislationCleanup as crate::traits::LegislationCleanupHandler>::cleanup_legislation_referendum_votes_chunk(
                            proposal_id, cleanup_limit,
                        );
                    let weight = db_weight.reads_writes(u64::from(removed), u64::from(removed));
                    let next = if has_remaining {
                        Some(PendingCleanupStage::LegislationReferendumVotes)
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
                    // 清理核心数据 + 业务数据（单次完成）。
                    //
                    // 双层 ID v1:必须先清反向索引(它们依赖 Proposals/ProposalOwner/
                    // ProposalDisplayId 反查分类键),再清主表。
                    Self::release_internal_proposal_mutexes(proposal_id);
                    Self::cleanup_proposal_indexes(proposal_id);
                    Proposals::<T>::remove(proposal_id);
                    // internal / joint mode storage 由 sub-pallet 删
                    <T::InternalCleanup as crate::traits::InternalCleanupHandler>::cleanup_internal_terminal(
                        proposal_id,
                    );
                    <T::JointCleanup as crate::traits::JointCleanupHandler>::cleanup_joint_terminal(
                        proposal_id,
                    );
                    <T::LegislationCleanup as crate::traits::LegislationCleanupHandler>::cleanup_legislation_terminal(
                        proposal_id,
                    );
                    ProposalData::<T>::remove(proposal_id);
                    ProposalOwner::<T>::remove(proposal_id);
                    ProposalMeta::<T>::remove(proposal_id);
                    ProposalExecutionRetryStates::<T>::remove(proposal_id);
                    // 反向索引 4 张 + ProposalDisplayId 1 张额外 5 次 write
                    let weight = db_weight.writes(16);
                    (None, weight) // 全部完成
                }
            }
        }
    }
}
