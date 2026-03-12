#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarks;
pub mod citizen_vote;
pub mod internal_vote;
pub mod joint_vote;
pub mod weights;

pub use citizen_vote::{SfidEligibility, VoteCredentialCleanup};
pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

pub type InstitutionPalletId = [u8; 48];

pub const PROPOSAL_KIND_INTERNAL: u8 = 0;
pub const PROPOSAL_KIND_JOINT: u8 = 1;

pub const STAGE_INTERNAL: u8 = 0;
pub const STAGE_JOINT: u8 = 1;
pub const STAGE_CITIZEN: u8 = 2;

pub const STATUS_VOTING: u8 = 0;
pub const STATUS_PASSED: u8 = 1;
pub const STATUS_REJECTED: u8 = 2;

/// 中文注释：事项模块接入联合投票时，统一由投票引擎创建提案并写入人口快照。
pub trait JointVoteEngine<AccountId> {
    fn create_joint_proposal(
        who: AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        snapshot_signature: &[u8],
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
    JointVotes,
    CitizenVotes,
    VoteCredentials,
}

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

        /// 每个区块最多执行多少个清理步骤，避免历史提案清理拖垮 on_initialize。
        #[pallet::constant]
        type MaxCleanupStepsPerBlock: Get<u32>;

        /// 每个清理步骤最多删除多少条前缀项。
        #[pallet::constant]
        type CleanupKeysPerStep: Get<u32>;

        type SfidEligibility: SfidEligibility<Self::AccountId, Self::Hash>;
        type PopulationSnapshotVerifier: PopulationSnapshotVerifier<
            Self::AccountId,
            VoteNonceOf<Self>,
            VoteSignatureOf<Self>,
        >;

        type JointVoteResultCallback: JointVoteResultCallback;
        type InternalAdminProvider: InternalAdminProvider<Self::AccountId>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    use crate::weights::WeightInfo;

    pub type VoteNonceOf<T> = BoundedVec<u8, <T as Config>::MaxVoteNonceLength>;
    pub type VoteSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxVoteSignatureLength>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, u64, ValueQuery>;

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

    #[pallet::storage]
    pub type CitizenVotesBySfid<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, T::Hash, bool, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn citizen_tally)]
    pub type CitizenTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU64, ValueQuery>;

    /// 中文注释：总人口快照 nonce 防重放（全局维度，防止跨提案重放）。
    #[pallet::storage]
    #[pallet::getter(fn used_population_snapshot_nonce)]
    pub type UsedPopulationSnapshotNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProposalCreated {
            proposal_id: u64,
            kind: u8,
            stage: u8,
            end: BlockNumberFor<T>,
        },
        ProposalAdvancedToCitizen {
            proposal_id: u64,
            citizen_end: BlockNumberFor<T>,
            eligible_total: u64,
        },
        ProposalFinalized {
            proposal_id: u64,
            status: u8,
        },
        InternalVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        JointInstitutionVoteCast {
            proposal_id: u64,
            institution: InstitutionPalletId,
            internal_passed: bool,
        },
        CitizenVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            sfid_hash: T::Hash,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ProposalNotFound,
        InvalidProposalKind,
        InvalidProposalStage,
        InvalidProposalStatus,
        InvalidInternalOrg,
        InvalidInstitution,
        NoPermission,
        VoteClosed,
        VoteNotExpired,
        AlreadyVoted,
        SfidNotEligible,
        InvalidSfidVoteCredential,
        CitizenEligibleTotalNotSet,
        InvalidPopulationSnapshot,
        ProposalAlreadyFinalized,
        ProposalIdOverflow,
        AccountIdEncodingMismatch,
        TooManyProposalsAtExpiry,
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

            weight.saturating_add(Self::process_pending_cleanup_steps())
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 0))]
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
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 0))]
        pub fn create_joint_proposal(
            origin: OriginFor<T>,
            _eligible_total: u64,
            _snapshot_nonce: VoteNonceOf<T>,
            _snapshot_signature: VoteSignatureOf<T>,
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
        #[pallet::weight(T::WeightInfo::submit_joint_institution_vote())]
        pub fn submit_joint_institution_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            institution: InstitutionPalletId,
            internal_passed: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_submit_joint_institution_vote(who, proposal_id, institution, internal_passed)
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::citizen_vote())]
        pub fn citizen_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            sfid_hash: T::Hash,
            nonce: VoteNonceOf<T>,
            signature: VoteSignatureOf<T>,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_citizen_vote(who, proposal_id, sfid_hash, nonce, signature, approve)
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

        pub(crate) fn allocate_proposal_id() -> Result<u64, DispatchError> {
            let id = NextProposalId::<T>::get();
            let next = id.checked_add(1).ok_or(Error::<T>::ProposalIdOverflow)?;
            NextProposalId::<T>::put(next);
            Ok(id)
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

        pub(crate) fn set_status_and_emit(proposal_id: u64, status: u8) -> DispatchResult {
            with_transaction(|| {
                let kind = match Proposals::<T>::try_mutate(
                    proposal_id,
                    |maybe| -> Result<u8, DispatchError> {
                        let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                        let kind = proposal.kind;
                        proposal.status = status;
                        Ok(kind)
                    },
                ) {
                    Ok(kind) => kind,
                    Err(err) => return TransactionOutcome::Rollback(Err(err)),
                };

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

                TransactionOutcome::Commit(Ok(()))
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

            for _ in 0..max_steps {
                let Some((proposal_id, stage)) = PendingProposalCleanups::<T>::iter().next() else {
                    break;
                };
                weight = weight.saturating_add(db_weight.reads(1));

                let (next_stage, step_weight) =
                    Self::process_pending_cleanup_step(proposal_id, stage, cleanup_limit);
                weight = weight.saturating_add(step_weight);

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
                        None
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointVotes => {
                    let result = JointVotesByInstitution::<T>::clear_prefix(
                        proposal_id,
                        cleanup_limit,
                        None,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::JointVotes)
                    } else {
                        Some(PendingCleanupStage::CitizenVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::CitizenVotes => {
                    let result =
                        CitizenVotesBySfid::<T>::clear_prefix(proposal_id, cleanup_limit, None);
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
                        None
                    };
                    (next, weight)
                }
            }
        }
    }
}

impl<T: pallet::Config> JointVoteEngine<T::AccountId> for pallet::Pallet<T> {
    fn create_joint_proposal(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        snapshot_signature: &[u8],
    ) -> Result<u64, DispatchError> {
        let snapshot_nonce: pallet::VoteNonceOf<T> = snapshot_nonce
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        let snapshot_signature: pallet::VoteSignatureOf<T> = snapshot_signature
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        pallet::Pallet::<T>::do_create_joint_proposal(
            who,
            eligible_total,
            snapshot_nonce,
            snapshot_signature,
        )
    }

    fn cleanup_joint_proposal(proposal_id: u64) {
        pallet::Proposals::<T>::remove(proposal_id);
        pallet::JointTallies::<T>::remove(proposal_id);
        pallet::CitizenTallies::<T>::remove(proposal_id);
        pallet::PendingProposalCleanups::<T>::insert(proposal_id, PendingCleanupStage::JointVotes);
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

    fn cleanup_internal_proposal(proposal_id: u64) {
        pallet::Proposals::<T>::remove(proposal_id);
        pallet::InternalTallies::<T>::remove(proposal_id);
        pallet::PendingProposalCleanups::<T>::insert(
            proposal_id,
            PendingCleanupStage::InternalVotes,
        );
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
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = TestJointVoteResultCallback;
        type InternalAdminProvider = ();
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
            sfid_hash: &<Test as frame_system::Config>::Hash,
            who: &AccountId32,
        ) -> bool {
            *sfid_hash == sfid_hash_ok() && who == &nrc_admin(0)
        }

        fn verify_and_consume_vote_credential(
            sfid_hash: &<Test as frame_system::Config>::Hash,
            who: &AccountId32,
            proposal_id: u64,
            nonce: &[u8],
            signature: &[u8],
        ) -> bool {
            if !Self::is_eligible(sfid_hash, who) || signature != b"vote-ok" || nonce.is_empty() {
                return false;
            }
            let key = (proposal_id, sfid_hash.encode(), nonce.to_vec());
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
        AccountId32::new(CHINA_CB[0].admins[index])
    }

    fn nrc_multisig() -> AccountId32 {
        AccountId32::new(CHINA_CB[0].duoqian_address)
    }

    fn prc_multisig() -> AccountId32 {
        AccountId32::new(CHINA_CB[1].duoqian_address)
    }

    fn all_prc_institutions() -> Vec<(InstitutionPalletId, AccountId32)> {
        CHINA_CB
            .iter()
            .skip(1)
            .map(|n| {
                (
                    reserve_pallet_id_to_bytes(n.shenfen_id)
                        .expect("prc id should be shenfen_id bytes"),
                    AccountId32::new(n.duoqian_address),
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
                    AccountId32::new(n.duoqian_address),
                )
            })
            .collect()
    }

    fn prc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[1].admins[index])
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CH[0].admins[index])
    }

    fn sfid_hash_ok() -> <Test as frame_system::Config>::Hash {
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
        sfid_hash: <Test as frame_system::Config>::Hash,
        nonce: &str,
    ) {
        USED_VOTE_NONCES.with(|set| {
            set.borrow_mut()
                .insert((proposal_id, sfid_hash.encode(), nonce.as_bytes().to_vec()));
        });
    }

    fn has_used_vote_nonce(
        proposal_id: u64,
        sfid_hash: <Test as frame_system::Config>::Hash,
        nonce: &str,
    ) -> bool {
        USED_VOTE_NONCES.with(|set| {
            set.borrow()
                .contains(&(proposal_id, sfid_hash.encode(), nonce.as_bytes().to_vec()))
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

            assert_eq!(
                create_internal_proposal_via_engine(
                    nrc_admin(0),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                0
            );
            assert_eq!(
                VotingEngineSystem::proposals(0)
                    .expect("proposal exists")
                    .stage,
                STAGE_INTERNAL
            );
        });
    }

    #[test]
    fn internal_vote_must_be_by_same_institution_admin() {
        new_test_ext().execute_with(|| {
            create_internal_proposal_via_engine(prb_admin(0), internal_vote::ORG_PRB, prb_pid());

            assert_noop!(
                <VotingEngineSystem as InternalVoteEngine<AccountId32>>::cast_internal_vote(
                    nrc_admin(0),
                    0,
                    true,
                ),
                pallet::Error::<Test>::NoPermission
            );

            assert_ok!(
                <VotingEngineSystem as InternalVoteEngine<AccountId32>>::cast_internal_vote(
                    prb_admin(1),
                    0,
                    true,
                )
            );
        });
    }

    #[test]
    fn nrc_internal_vote_passes_at_13_yes_votes() {
        new_test_ext().execute_with(|| {
            create_internal_proposal_via_engine(nrc_admin(0), internal_vote::ORG_NRC, nrc_pid());

            for i in 0..12 {
                assert_ok!(
                    <VotingEngineSystem as InternalVoteEngine<AccountId32>>::cast_internal_vote(
                        nrc_admin(i),
                        0,
                        true,
                    )
                );
            }
            assert_eq!(
                VotingEngineSystem::proposals(0)
                    .expect("proposal exists")
                    .status,
                STATUS_VOTING
            );

            assert_ok!(
                <VotingEngineSystem as InternalVoteEngine<AccountId32>>::cast_internal_vote(
                    nrc_admin(12),
                    0,
                    true,
                )
            );
            assert_eq!(
                VotingEngineSystem::proposals(0)
                    .expect("proposal exists")
                    .status,
                STATUS_PASSED
            );
        });
    }

    #[test]
    fn internal_vote_is_rejected_after_timeout() {
        new_test_ext().execute_with(|| {
            create_internal_proposal_via_engine(prc_admin(0), internal_vote::ORG_PRC, prc_pid());

            let proposal = VotingEngineSystem::proposals(0).expect("proposal exists");
            System::set_block_number(proposal.end + 1);

            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                0,
            ));
            assert_eq!(
                VotingEngineSystem::proposals(0)
                    .expect("proposal exists")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn internal_vote_timeout_is_auto_rejected_on_initialize() {
        new_test_ext().execute_with(|| {
            create_internal_proposal_via_engine(prc_admin(0), internal_vote::ORG_PRC, prc_pid());

            let proposal = VotingEngineSystem::proposals(0).expect("proposal exists");
            System::set_block_number(proposal.end);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(proposal.end);
            assert_eq!(
                VotingEngineSystem::proposals(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );

            let next = proposal.end + 1;
            System::set_block_number(next);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(next);
            assert_eq!(
                VotingEngineSystem::proposals(0)
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
    fn joint_vote_submission_must_be_by_institution_multisig() {
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

            assert_noop!(
                VotingEngineSystem::submit_joint_institution_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    nrc_pid(),
                    true
                ),
                pallet::Error::<Test>::NoPermission
            );

            assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                RuntimeOrigin::signed(nrc_multisig()),
                0,
                nrc_pid(),
                true
            ));

            // 中文注释：国储会多签不能代省储会提交省储会内部投票结果。
            assert_noop!(
                VotingEngineSystem::submit_joint_institution_vote(
                    RuntimeOrigin::signed(nrc_multisig()),
                    0,
                    prc_pid(),
                    true
                ),
                pallet::Error::<Test>::NoPermission
            );

            assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                RuntimeOrigin::signed(prc_multisig()),
                0,
                prc_pid(),
                true
            ));
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
                    sfid_hash_ok(),
                    vote_nonce("n-1"),
                    vote_sig_bad(),
                    true
                ),
                pallet::Error::<Test>::InvalidSfidVoteCredential
            );

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                sfid_hash_ok(),
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
                sfid_hash_ok(),
                vote_nonce("n-1"),
                vote_sig_ok(),
                true
            ));

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    sfid_hash_ok(),
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
                sfid_hash_ok(),
                vote_nonce("same"),
                vote_sig_ok(),
                true
            ));

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                1,
                sfid_hash_ok(),
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
                    sfid_hash_ok(),
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
    fn citizen_timeout_cleanup_requires_explicit_joint_cleanup_request() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 5);
            assert_ok!(VotingEngineSystem::schedule_proposal_expiry(0, 5));

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                sfid_hash_ok(),
                vote_nonce("timeout-cleanup"),
                vote_sig_ok(),
                true
            ));
            assert!(has_used_vote_nonce(0, sfid_hash_ok(), "timeout-cleanup"));

            System::set_block_number(6);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(6);

            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
            assert!(has_used_vote_nonce(0, sfid_hash_ok(), "timeout-cleanup"));

            <VotingEngineSystem as JointVoteEngine<AccountId32>>::cleanup_joint_proposal(0);
            System::set_block_number(7);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(7);
            assert!(!has_used_vote_nonce(0, sfid_hash_ok(), "timeout-cleanup"));
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
                    sfid_hash_ok(),
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
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    sfid_hash_ok(),
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
                sfid_hash_ok(),
                vote_nonce("immediate-pass"),
                vote_sig_ok(),
                true
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_PASSED);
        });
    }

    #[test]
    fn cleanup_joint_proposal_cleans_used_vote_nonce_on_next_initialize() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                sfid_hash_ok(),
                vote_nonce("immediate-cleanup"),
                vote_sig_ok(),
                true
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_PASSED);
            assert!(has_used_vote_nonce(0, sfid_hash_ok(), "immediate-cleanup"));

            <VotingEngineSystem as JointVoteEngine<AccountId32>>::cleanup_joint_proposal(0);
            System::set_block_number(101);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(101);
            assert!(!has_used_vote_nonce(0, sfid_hash_ok(), "immediate-cleanup"));
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
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    100,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                RuntimeOrigin::signed(nrc_multisig()),
                0,
                nrc_pid(),
                true
            ));

            for (institution, multisig) in all_prc_institutions() {
                assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                    RuntimeOrigin::signed(multisig),
                    0,
                    institution,
                    true
                ));
            }
            for (institution, multisig) in all_prb_institutions() {
                assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                    RuntimeOrigin::signed(multisig),
                    0,
                    institution,
                    true
                ));
            }

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_PASSED);
            assert_eq!(proposal.stage, STAGE_JOINT);
            assert_eq!(
                JointTallies::<Test>::get(0).yes,
                primitives::count_const::JOINT_VOTE_TOTAL
            );
        });
    }

    #[test]
    fn joint_vote_non_unanimous_moves_to_citizen_after_all_institutions_submit() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    77,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );
            let joint_end = Proposals::<Test>::get(0)
                .expect("proposal should exist")
                .end;

            assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                RuntimeOrigin::signed(nrc_multisig()),
                0,
                nrc_pid(),
                true
            ));

            let mut all_others = all_prc_institutions();
            all_others.extend(all_prb_institutions());
            let (last_institution, last_multisig) = all_others
                .pop()
                .expect("there should be at least one non-nrc institution");

            let first_prc = all_prc_institutions()
                .first()
                .cloned()
                .expect("there should be at least one prc institution");
            assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                RuntimeOrigin::signed(first_prc.1),
                0,
                first_prc.0,
                false
            ));

            for (institution, multisig) in all_others {
                if institution == first_prc.0 {
                    continue;
                }
                assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                    RuntimeOrigin::signed(multisig),
                    0,
                    institution,
                    true
                ));
            }

            System::set_block_number(50);
            assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                RuntimeOrigin::signed(last_multisig),
                0,
                last_institution,
                true
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.stage, STAGE_CITIZEN);
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(
                proposal.end,
                50 + primitives::count_const::VOTING_DURATION_BLOCKS as u64
            );
            assert_eq!(proposal.citizen_eligible_total, 77);
            assert!(proposal.end > joint_end);
            assert_eq!(JointTallies::<Test>::get(0).no, 1);
        });
    }

    #[test]
    fn joint_vote_timeout_moves_to_citizen_when_not_unanimous() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    88,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                RuntimeOrigin::signed(nrc_multisig()),
                0,
                nrc_pid(),
                true
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            System::set_block_number(proposal.end + 1);
            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
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
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    88,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                RuntimeOrigin::signed(nrc_multisig()),
                0,
                nrc_pid(),
                true
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            let expired_at = proposal.end + 1;
            System::set_block_number(expired_at);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(expired_at);

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
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
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    66,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );
            JointTallies::<Test>::insert(
                0,
                VoteCountU32 {
                    yes: primitives::count_const::JOINT_VOTE_TOTAL,
                    no: 0,
                },
            );

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            System::set_block_number(proposal.end + 1);
            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_PASSED);
            assert_eq!(proposal.stage, STAGE_JOINT);
        });
    }

    #[test]
    fn joint_vote_callback_failure_rolls_back_final_status() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    100,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            set_joint_callback_should_fail(true);
            assert!(VotingEngineSystem::set_status_and_emit(0, STATUS_PASSED).is_err());

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(proposal.stage, STAGE_JOINT);
        });
    }

    #[test]
    fn joint_vote_callback_failure_does_not_cleanup_vote_credentials() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            mark_vote_nonce_used(0, sfid_hash_ok(), "keep-on-fail");
            set_joint_callback_should_fail(true);

            assert!(VotingEngineSystem::set_status_and_emit(0, STATUS_PASSED).is_err());
            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );
            assert!(has_used_vote_nonce(0, sfid_hash_ok(), "keep-on-fail"));
        });
    }

    #[test]
    fn auto_finalize_requeues_failed_joint_callback() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    66,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            JointTallies::<Test>::insert(
                0,
                VoteCountU32 {
                    yes: primitives::count_const::JOINT_VOTE_TOTAL,
                    no: 0,
                },
            );

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            let expired_at = proposal.end + 1;

            set_joint_callback_should_fail(true);
            System::set_block_number(expired_at);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(expired_at);

            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );
            assert_eq!(PendingExpiryBucket::<Test>::get(), Some(expired_at));
            assert_eq!(ProposalsByExpiry::<Test>::get(expired_at), vec![0]);

            set_joint_callback_should_fail(false);
            let next_block = expired_at + 1;
            System::set_block_number(next_block);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(next_block);

            assert_eq!(
                Proposals::<Test>::get(0)
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
            for (index, sfid_hash) in citizen_hashes.iter().enumerate() {
                CitizenVotesBySfid::<Test>::insert(proposal_id, *sfid_hash, true);
                let nonce = match index {
                    0 => "cleanup-nonce-1",
                    1 => "cleanup-nonce-2",
                    _ => "cleanup-nonce-3",
                };
                mark_vote_nonce_used(proposal_id, *sfid_hash, nonce);
            }

            <VotingEngineSystem as JointVoteEngine<AccountId32>>::cleanup_joint_proposal(
                proposal_id,
            );
            assert_eq!(
                PendingProposalCleanups::<Test>::get(proposal_id),
                Some(PendingCleanupStage::JointVotes)
            );

            System::set_block_number(1);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(1);
            assert_eq!(
                PendingProposalCleanups::<Test>::get(proposal_id),
                Some(PendingCleanupStage::VoteCredentials)
            );
            assert!(
                has_used_vote_nonce(proposal_id, citizen_hashes[0], "cleanup-nonce-1")
                    || has_used_vote_nonce(proposal_id, citizen_hashes[1], "cleanup-nonce-2")
                    || has_used_vote_nonce(proposal_id, citizen_hashes[2], "cleanup-nonce-3")
            );

            System::set_block_number(2);
            <VotingEngineSystem as Hooks<u64>>::on_initialize(2);
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
}
