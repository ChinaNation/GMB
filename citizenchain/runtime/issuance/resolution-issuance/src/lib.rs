//! # 决议发行模块 (resolution-issuance)
//!
//! 本模块把决议发行治理与执行合并为一个完整业务 pallet：
//! 在同一模块内完成决议发行提案、联合投票回调、发行执行、
//! 防重放、暂停和审计维护。

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod execution;
pub mod migration;
pub mod proposal;
#[cfg(test)]
mod tests;
pub mod validation;
pub mod weights;

use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
use voting_engine::JointVoteResultCallback;

/// 模块标识前缀，用于在投票引擎 ProposalData 中识别决议发行提案。
///
/// 中文注释：保留旧值 `res-iss`，避免 nodeui / wuminapp 的提案展示识别逻辑无谓变化。
pub const MODULE_TAG: &[u8] = b"res-iss";

#[frame_support::pallet]
pub mod pallet {
    use crate::{migration, proposal::RecipientAmount, weights::WeightInfo};
    use codec::Decode;
    use frame_support::{pallet_prelude::*, traits::Currency, weights::Weight};
    use frame_system::pallet_prelude::*;
    use primitives::china::china_cb::CHINA_CB;
    #[cfg(feature = "std")]
    use sp_runtime::traits::Zero;
    use sp_std::vec::Vec;
    use voting_engine::JointVoteEngine;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type AllocationOf<T> = BoundedVec<
        RecipientAmount<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        <T as Config>::MaxAllocations,
    >;
    pub type SnapshotNonceOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotNonceLength>;
    pub type SnapshotSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotSignatureLength>;

    /// 中文注释：联合投票终结后的业务执行结果，用于决定 post-dispatch 退费和状态覆盖。
    pub(crate) enum FinalizeOutcome {
        ApprovedExecutionSucceeded,
        ApprovedExecutionFailed,
        Rejected,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        /// 允许国储会或省储会管理员发起决议发行提案。
        type ProposeOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
        /// 更新合法收款账户集合。
        type RecipientSetOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// 回放联合投票结果的受限来源（生产配置为拒绝所有外部来源）。
        type JointVoteFinalizeOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// 维护入口：仅用于清理短期执行记录和暂停开关。
        type MaintenanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 统一投票引擎：本模块只创建联合提案，投票动作由投票引擎公开入口承载。
        type JointVoteEngine: JointVoteEngine<Self::AccountId>;

        #[pallet::constant]
        type MaxReasonLen: Get<u32>;
        #[pallet::constant]
        type MaxAllocations: Get<u32>;
        #[pallet::constant]
        type MaxSnapshotNonceLength: Get<u32>;
        #[pallet::constant]
        type MaxSnapshotSignatureLength: Get<u32>;
        #[pallet::constant]
        type MaxTotalIssuance: Get<BalanceOf<Self>>;
        #[pallet::constant]
        type MaxSingleIssuance: Get<BalanceOf<Self>>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(migration::STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 合法收款账户集合。决议发行只允许向该集合精确分配。
    #[pallet::storage]
    #[pallet::getter(fn allowed_recipients)]
    pub type AllowedRecipients<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, T::MaxAllocations>, ValueQuery>;

    /// 当前处于 Voting 状态的决议发行提案数量，用于阻止治理中途切换收款集合。
    #[pallet::storage]
    #[pallet::getter(fn voting_proposal_count)]
    pub type VotingProposalCount<T> = StorageValue<_, u32, ValueQuery>;

    /// proposal_id 是否已有短期执行记录，用于审计展示和维护排障。
    #[pallet::storage]
    pub type Executed<T: Config> = StorageMap<_, Twox64Concat, u64, BlockNumberFor<T>, OptionQuery>;

    /// proposal_id 是否历史上执行过。该标记永久防重放，维护清理不得删除。
    #[pallet::storage]
    pub type EverExecuted<T: Config> = StorageMap<_, Twox64Concat, u64, (), OptionQuery>;

    /// 决议发行累计执行量。
    #[pallet::storage]
    pub type TotalIssued<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// 紧急暂停开关。开启后拒绝新的发行执行，但不影响只读查询和记录清理。
    #[pallet::storage]
    pub type Paused<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub allowed_recipients: Vec<T::AccountId>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            let allowed_recipients = CHINA_CB
                .iter()
                .skip(1)
                .map(|node| {
                    T::AccountId::decode(&mut &node.main_address[..])
                        .expect("CHINA_CB main_address must decode to AccountId")
                })
                .collect();
            Self { allowed_recipients }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let bounded: BoundedVec<T::AccountId, T::MaxAllocations> = self
                .allowed_recipients
                .clone()
                .try_into()
                .expect("allowed_recipients must fit MaxAllocations");
            Pallet::<T>::ensure_unique_recipients(bounded.as_slice())
                .expect("allowed_recipients must not contain duplicates");
            Pallet::<T>::ensure_recipients_in_china_cb(&bounded)
                .expect("allowed_recipients must be CHINA_CB PRC addresses");
            AllowedRecipients::<T>::put(bounded);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            migration::on_runtime_upgrade::<T>()
        }

        #[cfg(feature = "std")]
        fn integrity_test() {
            assert!(
                (CHINA_CB.len() as u32).saturating_sub(1) <= T::MaxAllocations::get(),
                "MaxAllocations must cover CHINA_CB recipients"
            );
            assert!(
                !T::MaxTotalIssuance::get().is_zero(),
                "MaxTotalIssuance must be greater than 0"
            );
            assert!(
                !T::MaxSingleIssuance::get().is_zero(),
                "MaxSingleIssuance must be greater than 0"
            );
            assert!(
                T::MaxSingleIssuance::get() <= T::MaxTotalIssuance::get(),
                "MaxSingleIssuance must not exceed MaxTotalIssuance"
            );
            assert!(T::MaxReasonLen::get() > 0, "MaxReasonLen must be > 0");
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 决议发行提案已创建，联合投票已发起。
        ResolutionIssuanceProposed {
            proposal_id: u64,
            proposer: T::AccountId,
            total_amount: BalanceOf<T>,
            allocation_count: u32,
        },
        /// 联合投票已终结，approved 表示是否通过。
        JointVoteFinalized { proposal_id: u64, approved: bool },
        /// 投票通过且发行执行成功，铸币已落账。
        IssuanceExecutionTriggered {
            proposal_id: u64,
            total_amount: BalanceOf<T>,
        },
        /// 投票通过但发行执行失败，提案状态会覆盖为 STATUS_EXECUTION_FAILED。
        IssuanceExecutionFailed { proposal_id: u64 },
        /// 合法收款账户集合已更新。
        AllowedRecipientsUpdated { count: u32 },
        /// 决议发行已经执行。
        ResolutionIssuanceExecuted {
            proposal_id: u64,
            total_amount: BalanceOf<T>,
            recipient_count: u32,
            reason_hash: T::Hash,
            allocations_hash: T::Hash,
        },
        /// 短期执行记录已清理。
        ExecutedCleared { proposal_id: u64 },
        /// 暂停状态已变更。
        PausedSet { paused: bool },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyReason,
        EmptyAllocations,
        InvalidAllocationCount,
        DuplicateRecipient,
        InvalidRecipientSet,
        ZeroAmount,
        AllocationOverflow,
        TotalMismatch,
        ProposalNotFound,
        JointVoteCreateFailed,
        RecipientsNotConfigured,
        DuplicateAllowedRecipient,
        ActiveVotingProposalsExist,
        VotingProposalCountOverflow,
        VotingProposalCountUnderflow,
        ProposalDataStoreFailed,
        RecipientRemoved,
        RecipientNotInChinaCb,
        AlreadyExecuted,
        AlreadyInState,
        TotalIssuedOverflow,
        ReasonTooLong,
        BelowExistentialDeposit,
        DepositFailed,
        ExceedsTotalIssuanceCap,
        ExceedsSingleIssuanceCap,
        NotExecuted,
        PalletPaused,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建“决议发行”联合投票提案。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_resolution_issuance())]
        pub fn propose_resolution_issuance(
            origin: OriginFor<T>,
            reason: ReasonOf<T>,
            total_amount: BalanceOf<T>,
            allocations: AllocationOf<T>,
            eligible_total: u64,
            snapshot_nonce: SnapshotNonceOf<T>,
            signature: SnapshotSignatureOf<T>,
        ) -> DispatchResult {
            let proposer = T::ProposeOrigin::ensure_origin(origin)?;
            Self::create_resolution_issuance_proposal(
                proposer,
                reason,
                total_amount,
                allocations,
                eligible_total,
                snapshot_nonce,
                signature,
            )
        }

        /// 联合投票回调入口。生产环境通过 Runtime 配置禁止外部直接调用。
        #[pallet::call_index(1)]
        #[pallet::weight(if *approved {
            <T as Config>::WeightInfo::finalize_joint_vote_approved()
        } else {
            <T as Config>::WeightInfo::finalize_joint_vote_rejected()
        })]
        pub fn finalize_joint_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approved: bool,
        ) -> DispatchResultWithPostInfo {
            T::JointVoteFinalizeOrigin::ensure_origin(origin)?;
            let outcome = Self::apply_joint_vote_result(proposal_id, approved)?;
            let actual = match outcome {
                FinalizeOutcome::ApprovedExecutionSucceeded => None,
                FinalizeOutcome::ApprovedExecutionFailed => {
                    Some(T::DbWeight::get().reads_writes(5, 7))
                }
                FinalizeOutcome::Rejected => Some(T::DbWeight::get().reads_writes(3, 4)),
            };
            Ok(actual.into())
        }

        /// 更新链上合法收款账户集合（只允许新增，不允许删除）。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::set_allowed_recipients())]
        pub fn set_allowed_recipients(
            origin: OriginFor<T>,
            recipients: BoundedVec<T::AccountId, T::MaxAllocations>,
        ) -> DispatchResult {
            T::RecipientSetOrigin::ensure_origin(origin)?;
            Self::set_allowed_recipients_inner(recipients)
        }

        /// 清理短期执行记录。永久防重放标记 `EverExecuted` 不允许清理。
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::clear_executed())]
        pub fn clear_executed(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            T::MaintenanceOrigin::ensure_origin(origin)?;
            Self::clear_executed_marker(proposal_id)
        }

        /// 设置发行暂停开关。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::set_paused())]
        pub fn set_paused(origin: OriginFor<T>, paused: bool) -> DispatchResult {
            T::MaintenanceOrigin::ensure_origin(origin)?;
            Self::set_pause_state(paused)
        }
    }
}

impl<T: pallet::Config> JointVoteResultCallback for pallet::Pallet<T> {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult {
        let outcome = pallet::Pallet::<T>::apply_joint_vote_result(vote_proposal_id, approved)?;
        if matches!(outcome, pallet::FinalizeOutcome::ApprovedExecutionFailed) {
            // 中文注释：本回调运行在投票引擎 set_status_and_emit 的事务中；
            // 执行失败时覆盖 PASSED，保证“投票通过但业务失败”有独立终态。
            voting_engine::Pallet::<T>::override_proposal_status(
                vote_proposal_id,
                voting_engine::STATUS_EXECUTION_FAILED,
            )?;
        }
        Ok(())
    }
}
