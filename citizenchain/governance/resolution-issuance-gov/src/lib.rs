#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
use voting_engine_system::JointVoteResultCallback;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};
    use frame_support::{pallet_prelude::*, weights::constants::RocksDbWeight, weights::Weight};
    use frame_system::pallet_prelude::*;
    use primitives::china::china_cb::CHINA_CB;
    use resolution_issuance_iss::ResolutionIssuanceExecutor;
    use resolution_issuance_iss::WeightInfo as IssuanceWeightInfoT;
    use sp_std::{collections::btree_set::BTreeSet, marker::PhantomData, vec::Vec};
    use voting_engine_system::JointVoteEngine;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type AllocationOf<T> = BoundedVec<
        RecipientAmount<<T as frame_system::Config>::AccountId>,
        <T as Config>::MaxAllocations,
    >;
    pub type SnapshotNonceOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotNonceLength>;
    pub type SnapshotSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotSignatureLength>;

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    pub struct RecipientAmount<AccountId> {
        pub recipient: AccountId,
        pub amount: u128,
    }

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    pub enum VoteKind {
        Joint,
    }

    pub(crate) enum FinalizeOutcome {
        ApprovedExecutionSucceeded,
        ApprovedExecutionFailed,
        Rejected,
    }

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    pub enum ProposalStatus {
        Voting,
        Passed,
        Rejected,
        ExecutionFailed,
    }

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct Proposal<T: Config> {
        pub proposer: T::AccountId,
        pub reason: ReasonOf<T>,
        pub total_amount: u128,
        pub allocations: AllocationOf<T>,
        pub vote_kind: VoteKind,
        pub status: ProposalStatus,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 仅允许国储会管理员发起提案。
        type NrcProposeOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
        /// 更新合法收款账户集合。
        type RecipientSetOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// 回放联合投票结果的受限来源（生产可配置为拒绝所有外部来源）。
        type JointVoteFinalizeOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 投票通过后，调用发行执行模块执行铸币。
        type IssuanceExecutor: ResolutionIssuanceExecutor<Self::AccountId, u128>;
        /// 用于估算发行执行路径的 weight。
        type IssuanceWeightInfo: IssuanceWeightInfoT;
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
        type MaxExecutionRetries: Get<u32>;

        /// 本 pallet 的 weight 配置。
        type WeightInfo: WeightInfo;
    }

    pub trait WeightInfo {
        fn propose_resolution_issuance(allocation_count: u32, reason_len: u32) -> Weight;
        fn finalize_joint_vote_approved() -> Weight;
        fn finalize_joint_vote_rejected() -> Weight;
        fn set_allowed_recipients(recipient_count: u32) -> Weight;
        fn retry_failed_execution() -> Weight;
    }

    pub struct SubstrateWeight<T>(PhantomData<T>);

    impl<T: Config> WeightInfo for SubstrateWeight<T> {
        fn propose_resolution_issuance(allocation_count: u32, reason_len: u32) -> Weight {
            T::DbWeight::get()
                .reads_writes(4, 7)
                .saturating_add(
                    Weight::from_parts(80_000, 128).saturating_mul(allocation_count as u64),
                )
                .saturating_add(Weight::from_parts(500, 1).saturating_mul(reason_len as u64))
        }

        fn finalize_joint_vote_approved() -> Weight {
            T::DbWeight::get()
                .reads_writes(2, 4)
                .saturating_add(T::IssuanceWeightInfo::execute_resolution_issuance(
                    T::MaxReasonLen::get(),
                    T::MaxAllocations::get(),
                ))
        }

        fn finalize_joint_vote_rejected() -> Weight {
            T::DbWeight::get().reads_writes(3, 4)
        }

        fn set_allowed_recipients(recipient_count: u32) -> Weight {
            T::DbWeight::get().reads_writes(1, 1).saturating_add(
                Weight::from_parts(80_000, 128).saturating_mul(recipient_count as u64),
            )
        }

        fn retry_failed_execution() -> Weight {
            T::DbWeight::get()
                .reads_writes(2, 2)
                .saturating_add(T::IssuanceWeightInfo::execute_resolution_issuance(
                    T::MaxReasonLen::get(),
                    T::MaxAllocations::get(),
                ))
        }
    }

    impl WeightInfo for () {
        fn propose_resolution_issuance(allocation_count: u32, reason_len: u32) -> Weight {
            RocksDbWeight::get()
                .reads_writes(4, 7)
                .saturating_add(
                    Weight::from_parts(80_000, 128).saturating_mul(allocation_count as u64),
                )
                .saturating_add(Weight::from_parts(500, 1).saturating_mul(reason_len as u64))
        }

        fn finalize_joint_vote_approved() -> Weight {
            RocksDbWeight::get().reads_writes(2, 4).saturating_add(
                <() as IssuanceWeightInfoT>::execute_resolution_issuance(1024, 128),
            )
        }

        fn finalize_joint_vote_rejected() -> Weight {
            RocksDbWeight::get().reads_writes(3, 4)
        }

        fn set_allowed_recipients(recipient_count: u32) -> Weight {
            RocksDbWeight::get().reads_writes(1, 1).saturating_add(
                Weight::from_parts(80_000, 128).saturating_mul(recipient_count as u64),
            )
        }

        fn retry_failed_execution() -> Weight {
            RocksDbWeight::get().reads_writes(2, 2).saturating_add(
                <() as IssuanceWeightInfoT>::execute_resolution_issuance(1024, 128),
            )
        }
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, u64, Proposal<T>, OptionQuery>;

    /// 决议发行提案ID -> 联合投票提案ID
    #[pallet::storage]
    #[pallet::getter(fn gov_to_joint_vote)]
    pub type GovToJointVote<T> = StorageMap<_, Blake2_128Concat, u64, u64, OptionQuery>;

    /// 联合投票提案ID -> 决议发行提案ID
    #[pallet::storage]
    #[pallet::getter(fn joint_vote_to_gov)]
    pub type JointVoteToGov<T> = StorageMap<_, Blake2_128Concat, u64, u64, OptionQuery>;

    /// 合法收款账户集合（链上可更新）。
    #[pallet::storage]
    #[pallet::getter(fn allowed_recipients)]
    pub type AllowedRecipients<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, T::MaxAllocations>, ValueQuery>;

    /// 当前处于 Voting 状态的提案数量，用于阻止治理中途切换收款集合。
    #[pallet::storage]
    #[pallet::getter(fn voting_proposal_count)]
    pub type VotingProposalCount<T> = StorageValue<_, u32, ValueQuery>;

    /// 执行失败提案的重试次数（仅统计 retry_failed_execution）。
    #[pallet::storage]
    #[pallet::getter(fn retry_count)]
    pub type RetryCount<T> = StorageMap<_, Blake2_128Concat, u64, u32, ValueQuery>;

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
                    T::AccountId::decode(&mut &node.duoqian_address[..])
                        .expect("CHINA_CB duoqian_address must decode to AccountId")
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
            AllowedRecipients::<T>::put(bounded);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let db = T::DbWeight::get();
            let on_chain = StorageVersion::get::<Pallet<T>>();
            if on_chain >= STORAGE_VERSION {
                return db.reads(1);
            }

            let mut reads = 1u64;
            let mut writes = 0u64;
            let mut iter_weight = Weight::from_parts(0, 0);

            if on_chain < StorageVersion::new(1) {
                reads = reads.saturating_add(1);
                if AllowedRecipients::<T>::get().is_empty() {
                    if let Some(defaults) = Self::decode_default_allowed_recipients() {
                        AllowedRecipients::<T>::put(defaults);
                        writes = writes.saturating_add(1);
                    }
                }
            }

            if on_chain < StorageVersion::new(2) {
                reads = reads.saturating_add(1);
                let current_allowed = AllowedRecipients::<T>::get();
                if Self::ensure_unique_recipients(current_allowed.as_slice()).is_err() {
                    if let Some(defaults) = Self::decode_default_allowed_recipients() {
                        AllowedRecipients::<T>::put(defaults);
                        writes = writes.saturating_add(1);
                    }
                }

                let mut scanned = 0u64;
                let mut voting = 0u32;
                for proposal in Proposals::<T>::iter_values() {
                    scanned = scanned.saturating_add(1);
                    if matches!(proposal.status, ProposalStatus::Voting) {
                        voting = voting.saturating_add(1);
                    }
                }
                reads = reads.saturating_add(scanned);
                iter_weight = iter_weight
                    .saturating_add(Weight::from_parts(50_000, 64).saturating_mul(scanned));
                VotingProposalCount::<T>::put(voting);
                writes = writes.saturating_add(1);
            }

            STORAGE_VERSION.put::<Pallet<T>>();
            writes = writes.saturating_add(1);
            db.reads_writes(reads, writes).saturating_add(iter_weight)
        }

        #[cfg(feature = "std")]
        fn integrity_test() {
            assert!(
                (CHINA_CB.len() as u32).saturating_sub(1) <= T::MaxAllocations::get(),
                "MaxAllocations must cover CHINA_CB recipients"
            );
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ResolutionIssuanceProposed {
            proposal_id: u64,
            joint_vote_id: u64,
            proposer: T::AccountId,
            total_amount: u128,
            allocation_count: u32,
        },
        JointVoteFinalized {
            proposal_id: u64,
            joint_vote_id: Option<u64>,
            approved: bool,
        },
        IssuanceExecutionTriggered {
            proposal_id: u64,
            total_amount: u128,
        },
        IssuanceExecutionFailed {
            proposal_id: u64,
        },
        AllowedRecipientsUpdated {
            count: u32,
        },
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
        ProposalNotVoting,
        JointVoteCreateFailed,
        JointVoteMappingNotFound,
        ProposalIdOverflow,
        RecipientsNotConfigured,
        DuplicateAllowedRecipient,
        ProposalNotExecutionFailed,
        MaxRetriesExceeded,
        ActiveVotingProposalsExist,
        VotingProposalCountOverflow,
        VotingProposalCountUnderflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 国储会提案：创建“决议发行”联合投票提案。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::propose_resolution_issuance(
            allocations.len() as u32,
            reason.len() as u32
        ))]
        pub fn propose_resolution_issuance(
            origin: OriginFor<T>,
            reason: ReasonOf<T>,
            total_amount: u128,
            allocations: AllocationOf<T>,
            eligible_total: u64,
            snapshot_nonce: SnapshotNonceOf<T>,
            snapshot_signature: SnapshotSignatureOf<T>,
        ) -> DispatchResult {
            let proposer = T::NrcProposeOrigin::ensure_origin(origin)?;

            ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
            Self::validate_allocations(total_amount, allocations.as_slice())?;

            let proposal_id = NextProposalId::<T>::get();
            let next = proposal_id
                .checked_add(1)
                .ok_or(Error::<T>::ProposalIdOverflow)?;
            NextProposalId::<T>::put(next);
            let joint_vote_id = T::JointVoteEngine::create_joint_proposal(
                proposer.clone(),
                eligible_total,
                snapshot_nonce.as_slice(),
                snapshot_signature.as_slice(),
            )
            .map_err(|_| Error::<T>::JointVoteCreateFailed)?;

            let proposal = Proposal::<T> {
                proposer: proposer.clone(),
                reason,
                total_amount,
                allocations: allocations.clone(),
                vote_kind: VoteKind::Joint,
                status: ProposalStatus::Voting,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            GovToJointVote::<T>::insert(proposal_id, joint_vote_id);
            JointVoteToGov::<T>::insert(joint_vote_id, proposal_id);
            Self::increment_voting_proposal_count()?;

            Self::deposit_event(Event::<T>::ResolutionIssuanceProposed {
                proposal_id,
                joint_vote_id,
                proposer,
                total_amount,
                allocation_count: allocations.len() as u32,
            });
            Ok(())
        }

        /// 联合投票回调：仅接受联合投票引擎/治理权限来源。
        /// approved=true 时，触发 execution pallet 执行发行。
        #[pallet::call_index(1)]
        #[pallet::weight(if *approved {
            T::WeightInfo::finalize_joint_vote_approved()
        } else {
            T::WeightInfo::finalize_joint_vote_rejected()
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
                    Some(T::DbWeight::get().reads_writes(3, 5))
                }
                FinalizeOutcome::Rejected => Some(T::DbWeight::get().reads_writes(3, 4)),
            };
            Ok(actual.into())
        }

        /// 更新链上合法收款账户集合。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::set_allowed_recipients(recipients.len() as u32))]
        pub fn set_allowed_recipients(
            origin: OriginFor<T>,
            recipients: BoundedVec<T::AccountId, T::MaxAllocations>,
        ) -> DispatchResult {
            T::RecipientSetOrigin::ensure_origin(origin)?;
            ensure!(!recipients.is_empty(), Error::<T>::RecipientsNotConfigured);
            ensure!(
                VotingProposalCount::<T>::get() == 0,
                Error::<T>::ActiveVotingProposalsExist
            );
            Self::ensure_unique_recipients(recipients.as_slice())?;
            AllowedRecipients::<T>::put(recipients.clone());
            Self::deposit_event(Event::<T>::AllowedRecipientsUpdated {
                count: recipients.len() as u32,
            });
            Ok(())
        }

        /// 对执行失败的提案进行重试。
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::retry_failed_execution())]
        pub fn retry_failed_execution(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResultWithPostInfo {
            let _ = T::NrcProposeOrigin::ensure_origin(origin)?;

            let mut proposal =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                matches!(proposal.status, ProposalStatus::ExecutionFailed),
                Error::<T>::ProposalNotExecutionFailed
            );
            let retry_count = RetryCount::<T>::get(proposal_id);
            ensure!(
                retry_count < T::MaxExecutionRetries::get(),
                Error::<T>::MaxRetriesExceeded
            );
            let next_retry_count = retry_count.saturating_add(1);

            let execute_allocations = proposal
                .allocations
                .iter()
                .map(|x| (x.recipient.clone(), x.amount))
                .collect();

            if T::IssuanceExecutor::execute_resolution_issuance(
                proposal_id,
                proposal.reason.to_vec(),
                proposal.total_amount,
                execute_allocations,
            )
            .is_ok()
            {
                proposal.status = ProposalStatus::Passed;
                Proposals::<T>::insert(proposal_id, &proposal);
                RetryCount::<T>::remove(proposal_id);
                Self::deposit_event(Event::<T>::IssuanceExecutionTriggered {
                    proposal_id,
                    total_amount: proposal.total_amount,
                });
                Ok(().into())
            } else {
                RetryCount::<T>::insert(proposal_id, next_retry_count);
                Self::deposit_event(Event::<T>::IssuanceExecutionFailed { proposal_id });
                Ok(Some(T::DbWeight::get().reads_writes(2, 1)).into())
            }
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn apply_joint_vote_result(
            proposal_id: u64,
            approved: bool,
        ) -> Result<FinalizeOutcome, DispatchError> {
            let mut proposal =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                matches!(proposal.status, ProposalStatus::Voting),
                Error::<T>::ProposalNotVoting
            );

            if approved {
                let execute_allocations = proposal
                    .allocations
                    .iter()
                    .map(|x| (x.recipient.clone(), x.amount))
                    .collect();

                if T::IssuanceExecutor::execute_resolution_issuance(
                    proposal_id,
                    proposal.reason.to_vec(),
                    proposal.total_amount,
                    execute_allocations,
                )
                .is_ok()
                {
                    // 中文注释：执行成功后再标记 Passed，避免“通过但未执行”的伪成功状态。
                    proposal.status = ProposalStatus::Passed;
                    Proposals::<T>::insert(proposal_id, &proposal);
                    let joint_vote_id = Self::cleanup_vote_mapping(proposal_id);
                    RetryCount::<T>::remove(proposal_id);
                    Self::decrement_voting_proposal_count()?;
                    Self::deposit_event(Event::<T>::JointVoteFinalized {
                        proposal_id,
                        joint_vote_id,
                        approved: true,
                    });
                    Self::deposit_event(Event::<T>::IssuanceExecutionTriggered {
                        proposal_id,
                        total_amount: proposal.total_amount,
                    });
                    return Ok(FinalizeOutcome::ApprovedExecutionSucceeded);
                } else {
                    proposal.status = ProposalStatus::ExecutionFailed;
                    Proposals::<T>::insert(proposal_id, &proposal);
                    let joint_vote_id = Self::cleanup_vote_mapping(proposal_id);
                    Self::decrement_voting_proposal_count()?;
                    Self::deposit_event(Event::<T>::JointVoteFinalized {
                        proposal_id,
                        joint_vote_id,
                        approved: true,
                    });
                    Self::deposit_event(Event::<T>::IssuanceExecutionFailed { proposal_id });
                    return Ok(FinalizeOutcome::ApprovedExecutionFailed);
                }
            } else {
                proposal.status = ProposalStatus::Rejected;
                Proposals::<T>::insert(proposal_id, &proposal);
                let joint_vote_id = Self::cleanup_vote_mapping(proposal_id);
                Self::decrement_voting_proposal_count()?;
                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    joint_vote_id,
                    approved: false,
                });
                return Ok(FinalizeOutcome::Rejected);
            }
        }

        fn validate_allocations(
            total_amount: u128,
            allocations: &[RecipientAmount<T::AccountId>],
        ) -> DispatchResult {
            ensure!(!allocations.is_empty(), Error::<T>::EmptyAllocations);
            let expected = AllowedRecipients::<T>::get();
            ensure!(!expected.is_empty(), Error::<T>::RecipientsNotConfigured);
            let expected_set: BTreeSet<Vec<u8>> = expected.iter().map(|who| who.encode()).collect();
            ensure!(
                expected_set.len() == expected.len(),
                Error::<T>::DuplicateAllowedRecipient
            );
            ensure!(
                allocations.len() == expected_set.len(),
                Error::<T>::InvalidAllocationCount
            );
            let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();

            let mut sum = 0u128;
            for item in allocations {
                ensure!(item.amount > 0, Error::<T>::ZeroAmount);
                let who = item.recipient.encode();
                ensure!(seen.insert(who.clone()), Error::<T>::DuplicateRecipient);
                ensure!(expected_set.contains(&who), Error::<T>::InvalidRecipientSet);
                sum = sum
                    .checked_add(item.amount)
                    .ok_or(Error::<T>::AllocationOverflow)?;
            }

            // 防御性校验：正常流程在上面的长度/成员约束下已可推出相等，这里保留用于防回归。
            ensure!(seen == expected_set, Error::<T>::InvalidRecipientSet);
            ensure!(sum == total_amount, Error::<T>::TotalMismatch);
            Ok(())
        }

        fn ensure_unique_recipients(recipients: &[T::AccountId]) -> DispatchResult {
            let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
            for recipient in recipients {
                let encoded = recipient.encode();
                ensure!(seen.insert(encoded), Error::<T>::DuplicateAllowedRecipient);
            }
            Ok(())
        }

        fn decode_default_allowed_recipients() -> Option<BoundedVec<T::AccountId, T::MaxAllocations>>
        {
            let recipients: Vec<T::AccountId> = CHINA_CB
                .iter()
                .skip(1)
                .filter_map(|node| T::AccountId::decode(&mut &node.duoqian_address[..]).ok())
                .collect();
            if recipients.is_empty() {
                return None;
            }
            let bounded: BoundedVec<T::AccountId, T::MaxAllocations> =
                recipients.try_into().ok()?;
            if Self::ensure_unique_recipients(bounded.as_slice()).is_err() {
                return None;
            }
            Some(bounded)
        }

        fn cleanup_vote_mapping(proposal_id: u64) -> Option<u64> {
            let joint_vote_id = GovToJointVote::<T>::take(proposal_id);
            if let Some(joint_vote_id) = joint_vote_id {
                JointVoteToGov::<T>::remove(joint_vote_id);
            }
            joint_vote_id
        }

        fn increment_voting_proposal_count() -> DispatchResult {
            VotingProposalCount::<T>::try_mutate(|count| -> DispatchResult {
                *count = count
                    .checked_add(1)
                    .ok_or(Error::<T>::VotingProposalCountOverflow)?;
                Ok(())
            })
        }

        fn decrement_voting_proposal_count() -> DispatchResult {
            VotingProposalCount::<T>::try_mutate(|count| -> DispatchResult {
                *count = count
                    .checked_sub(1)
                    .ok_or(Error::<T>::VotingProposalCountUnderflow)?;
                Ok(())
            })
        }
    }
}

impl<T: pallet::Config> JointVoteResultCallback for pallet::Pallet<T> {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult {
        let gov_id = pallet::JointVoteToGov::<T>::get(vote_proposal_id)
            .ok_or(pallet::Error::<T>::JointVoteMappingNotFound)?;
        pallet::Pallet::<T>::apply_joint_vote_result(gov_id, approved).map(|_| ())
    }
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking {
    use super::*;

    use codec::Decode;
    use frame_benchmarking::v2::*;
    use frame_support::traits::Get;
    use frame_support::BoundedVec;
    use frame_system::RawOrigin;
    use primitives::china::china_cb::CHINA_CB;
    use sp_std::{vec, vec::Vec};

    fn decode_account<T: pallet::Config>(raw: [u8; 32]) -> T::AccountId {
        T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
    }

    fn nrc_admin<T: pallet::Config>() -> T::AccountId {
        decode_account::<T>(CHINA_CB[0].admins[0])
    }

    fn prc_recipients<T: pallet::Config>() -> BoundedVec<T::AccountId, T::MaxAllocations> {
        let recipients: Vec<T::AccountId> = CHINA_CB
            .iter()
            .skip(1)
            .map(|node| decode_account::<T>(node.duoqian_address))
            .collect();
        recipients
            .try_into()
            .expect("benchmark recipients should fit MaxAllocations")
    }

    fn reason_ok<T: pallet::Config>() -> pallet::ReasonOf<T> {
        b"bench-reason"
            .to_vec()
            .try_into()
            .expect("benchmark reason should fit")
    }

    fn reason_max<T: pallet::Config>() -> pallet::ReasonOf<T> {
        let len = core::cmp::max(1usize, T::MaxReasonLen::get() as usize);
        vec![b'r'; len]
            .try_into()
            .expect("max benchmark reason should fit")
    }

    fn one_allocation<T: pallet::Config>() -> pallet::AllocationOf<T> {
        let recipient = decode_account::<T>(CHINA_CB[1].duoqian_address);
        let alloc = vec![pallet::RecipientAmount {
            recipient,
            amount: 1_000_000u128,
        }];
        alloc
            .try_into()
            .expect("benchmark allocations should fit MaxAllocations")
    }

    fn full_allocations<T: pallet::Config>() -> (pallet::AllocationOf<T>, u128) {
        let recipients = prc_recipients::<T>();
        let mut allocations: Vec<pallet::RecipientAmount<T::AccountId>> =
            Vec::with_capacity(recipients.len());
        let mut total = 0u128;
        for recipient in recipients {
            let amount = 1_000_000u128;
            total = total.saturating_add(amount);
            allocations.push(pallet::RecipientAmount { recipient, amount });
        }
        (
            allocations
                .try_into()
                .expect("benchmark allocations should fit MaxAllocations"),
            total,
        )
    }

    fn snapshot_nonce_ok<T: pallet::Config>() -> pallet::SnapshotNonceOf<T> {
        let len = core::cmp::max(1usize, T::MaxSnapshotNonceLength::get() as usize).min(16);
        vec![b'n'; len]
            .try_into()
            .expect("benchmark nonce should fit")
    }

    fn snapshot_sig_ok<T: pallet::Config>() -> pallet::SnapshotSignatureOf<T> {
        let len = core::cmp::max(1usize, T::MaxSnapshotSignatureLength::get() as usize).min(64);
        vec![b's'; len]
            .try_into()
            .expect("benchmark signature should fit")
    }

    #[benchmarks]
    mod benchmarks {
        use super::*;

        #[benchmark]
        fn set_allowed_recipients() {
            VotingProposalCount::<T>::put(0u32);
            let recipients = prc_recipients::<T>();

            #[extrinsic_call]
            set_allowed_recipients(RawOrigin::Root, recipients.clone());

            assert_eq!(AllowedRecipients::<T>::get(), recipients);
        }

        #[benchmark]
        fn propose_resolution_issuance() {
            let proposer = nrc_admin::<T>();
            let recipients = prc_recipients::<T>();
            AllowedRecipients::<T>::put(recipients.clone());
            VotingProposalCount::<T>::put(0u32);

            let reason = reason_max::<T>();
            let (allocations, total_amount) = full_allocations::<T>();
            let nonce = snapshot_nonce_ok::<T>();
            let signature = snapshot_sig_ok::<T>();

            #[extrinsic_call]
            propose_resolution_issuance(
                RawOrigin::Signed(proposer),
                reason,
                total_amount,
                allocations,
                10u64,
                nonce,
                signature,
            );

            assert!(Proposals::<T>::contains_key(0u64));
            assert_eq!(VotingProposalCount::<T>::get(), 1u32);
        }

        #[benchmark]
        fn finalize_joint_vote_approved() {
            let proposal_id = 11u64;
            let proposer = nrc_admin::<T>();
            let reason = reason_max::<T>();
            let (allocations, total_amount) = full_allocations::<T>();
            let proposal = pallet::Proposal::<T> {
                proposer,
                reason,
                total_amount,
                allocations,
                vote_kind: pallet::VoteKind::Joint,
                status: pallet::ProposalStatus::Voting,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            GovToJointVote::<T>::insert(proposal_id, 111u64);
            JointVoteToGov::<T>::insert(111u64, proposal_id);
            VotingProposalCount::<T>::put(1u32);

            #[extrinsic_call]
            finalize_joint_vote(RawOrigin::Root, proposal_id, true);

            assert!(matches!(
                Proposals::<T>::get(proposal_id).map(|p| p.status),
                Some(pallet::ProposalStatus::Passed | pallet::ProposalStatus::ExecutionFailed)
            ));
            assert_eq!(VotingProposalCount::<T>::get(), 0u32);
        }

        #[benchmark]
        fn finalize_joint_vote_rejected() {
            let proposal_id = 12u64;
            let proposer = nrc_admin::<T>();
            let reason = reason_ok::<T>();
            let allocations = one_allocation::<T>();
            let proposal = pallet::Proposal::<T> {
                proposer,
                reason,
                total_amount: 1_000_000u128,
                allocations,
                vote_kind: pallet::VoteKind::Joint,
                status: pallet::ProposalStatus::Voting,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            GovToJointVote::<T>::insert(proposal_id, 112u64);
            JointVoteToGov::<T>::insert(112u64, proposal_id);
            VotingProposalCount::<T>::put(1u32);

            #[extrinsic_call]
            finalize_joint_vote(RawOrigin::Root, proposal_id, false);

            assert!(matches!(
                Proposals::<T>::get(proposal_id).map(|p| p.status),
                Some(pallet::ProposalStatus::Rejected)
            ));
            assert_eq!(VotingProposalCount::<T>::get(), 0u32);
        }

        #[benchmark]
        fn retry_failed_execution() {
            let proposal_id = 7u64;
            let proposer = nrc_admin::<T>();
            let reason = reason_max::<T>();
            let (allocations, total_amount) = full_allocations::<T>();
            let proposal = pallet::Proposal::<T> {
                proposer: proposer.clone(),
                reason,
                total_amount,
                allocations,
                vote_kind: pallet::VoteKind::Joint,
                status: pallet::ProposalStatus::ExecutionFailed,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            RetryCount::<T>::insert(proposal_id, 0u32);

            #[extrinsic_call]
            retry_failed_execution(RawOrigin::Signed(proposer), proposal_id);

            assert!(matches!(
                Proposals::<T>::get(proposal_id).map(|p| p.status),
                Some(pallet::ProposalStatus::Passed)
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;
    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32, BoundedVec};
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage, DispatchError};

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
        pub type ResolutionIssuanceGov = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    pub struct EnsureNrcAdminForTest;
    impl frame_support::traits::EnsureOrigin<RuntimeOrigin> for EnsureNrcAdminForTest {
        type Success = AccountId32;

        fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
            let who = frame_system::EnsureSigned::<AccountId32>::try_origin(o)?;
            if who == AccountId32::new([1u8; 32]) {
                Ok(who)
            } else {
                Err(RuntimeOrigin::from(frame_system::RawOrigin::Signed(who)))
            }
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
            Ok(RuntimeOrigin::signed(AccountId32::new([1u8; 32])))
        }
    }

    thread_local! {
        static NEXT_JOINT_ID: RefCell<u64> = const { RefCell::new(100) };
        static EXEC_CALLS: RefCell<Vec<(u64, u128, usize)>> = const { RefCell::new(Vec::new()) };
        static EXEC_SHOULD_FAIL: RefCell<bool> = const { RefCell::new(false) };
    }

    pub struct TestJointVoteEngine;
    impl voting_engine_system::JointVoteEngine<AccountId32> for TestJointVoteEngine {
        fn create_joint_proposal(
            _who: AccountId32,
            eligible_total: u64,
            snapshot_nonce: &[u8],
            snapshot_signature: &[u8],
        ) -> Result<u64, DispatchError> {
            if eligible_total == 0 || snapshot_nonce.is_empty() || snapshot_signature.is_empty() {
                return Err(DispatchError::Other("bad snapshot"));
            }
            NEXT_JOINT_ID.with(|id| {
                let mut id = id.borrow_mut();
                let v = *id;
                *id = id.saturating_add(1);
                Ok(v)
            })
        }
    }

    pub struct TestIssuanceExecutor;
    impl resolution_issuance_iss::ResolutionIssuanceExecutor<AccountId32, u128>
        for TestIssuanceExecutor
    {
        fn execute_resolution_issuance(
            proposal_id: u64,
            _reason: Vec<u8>,
            total_amount: u128,
            allocations: Vec<(AccountId32, u128)>,
        ) -> DispatchResult {
            let should_fail = EXEC_SHOULD_FAIL.with(|v| *v.borrow());
            if should_fail {
                return Err(DispatchError::Other("exec failed"));
            }
            EXEC_CALLS.with(|calls| {
                calls
                    .borrow_mut()
                    .push((proposal_id, total_amount, allocations.len()));
            });
            Ok(())
        }
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type NrcProposeOrigin = EnsureNrcAdminForTest;
        type RecipientSetOrigin = frame_system::EnsureRoot<AccountId32>;
        type JointVoteFinalizeOrigin = frame_system::EnsureRoot<AccountId32>;
        type IssuanceExecutor = TestIssuanceExecutor;
        type IssuanceWeightInfo = ();
        type WeightInfo = pallet::SubstrateWeight<Test>;
        type JointVoteEngine = TestJointVoteEngine;
        type MaxReasonLen = ConstU32<128>;
        type MaxAllocations = ConstU32<64>;
        type MaxSnapshotNonceLength = ConstU32<64>;
        type MaxSnapshotSignatureLength = ConstU32<64>;
        type MaxExecutionRetries = ConstU32<2>;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");
        let mut ext: sp_io::TestExternalities = storage.into();
        ext.execute_with(|| {
            EXEC_CALLS.with(|c| c.borrow_mut().clear());
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = false);
            NEXT_JOINT_ID.with(|id| *id.borrow_mut() = 100);
            let recipients = reserve_council_accounts();
            let bounded: BoundedVec<AccountId32, ConstU32<64>> =
                recipients.try_into().expect("recipients should fit");
            pallet::AllowedRecipients::<Test>::put(bounded);
        });
        ext
    }

    fn reason_ok() -> pallet::ReasonOf<Test> {
        b"issuance".to_vec().try_into().expect("reason should fit")
    }

    fn nonce_ok() -> pallet::SnapshotNonceOf<Test> {
        b"snap-nonce".to_vec().try_into().expect("nonce should fit")
    }

    fn sig_ok() -> pallet::SnapshotSignatureOf<Test> {
        b"snap-signature"
            .to_vec()
            .try_into()
            .expect("signature should fit")
    }

    fn reserve_council_accounts() -> Vec<AccountId32> {
        primitives::china::china_cb::CHINA_CB
            .iter()
            .skip(1)
            .map(|n| AccountId32::new(n.duoqian_address))
            .collect()
    }

    fn allocations_ok(total: u128) -> pallet::AllocationOf<Test> {
        let recipients = reserve_council_accounts();
        let count = recipients.len() as u128;
        let per = total / count;
        let mut left = total;
        let mut v = Vec::new();
        for (i, recipient) in recipients.into_iter().enumerate() {
            let amount = if i + 1 == count as usize { left } else { per };
            left = left.saturating_sub(amount);
            v.push(pallet::RecipientAmount { recipient, amount });
        }
        v.try_into().expect("allocations should fit")
    }

    #[test]
    fn only_nrc_admin_can_propose() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([2u8; 32])),
                    reason_ok(),
                    1000,
                    allocations_ok(1000),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn reject_invalid_allocation_count() {
        new_test_ext().execute_with(|| {
            let one = vec![pallet::RecipientAmount {
                recipient: reserve_council_accounts()[0].clone(),
                amount: 1000,
            }];
            let alloc: pallet::AllocationOf<Test> = one.try_into().expect("should fit");
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    1000,
                    alloc,
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::InvalidAllocationCount
            );
        });
    }

    #[test]
    fn reject_empty_reason() {
        new_test_ext().execute_with(|| {
            let reason: pallet::ReasonOf<Test> = Vec::<u8>::new().try_into().expect("should fit");
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason,
                    1000,
                    allocations_ok(1000),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::EmptyReason
            );
        });
    }

    #[test]
    fn reject_zero_amount_allocation() {
        new_test_ext().execute_with(|| {
            let mut raw = allocations_ok(1000).into_inner();
            raw[0].amount = 0;
            let alloc: pallet::AllocationOf<Test> = raw.try_into().expect("should fit");
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    1000,
                    alloc,
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::ZeroAmount
            );
        });
    }

    #[test]
    fn reject_duplicate_recipient_allocation() {
        new_test_ext().execute_with(|| {
            let recipients = reserve_council_accounts();
            let mut raw: Vec<pallet::RecipientAmount<AccountId32>> = recipients
                .iter()
                .cloned()
                .map(|recipient| pallet::RecipientAmount {
                    recipient,
                    amount: 1u128,
                })
                .collect();
            let last = raw.len().saturating_sub(1);
            raw[last].recipient = raw[0].recipient.clone();
            let alloc: pallet::AllocationOf<Test> = raw.try_into().expect("should fit");
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    recipients.len() as u128,
                    alloc,
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::DuplicateRecipient
            );
        });
    }

    #[test]
    fn reject_total_mismatch() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    999,
                    allocations_ok(1000),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::TotalMismatch
            );
        });
    }

    #[test]
    fn approved_callback_executes_issuance() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));

            assert_ok!(ResolutionIssuanceGov::on_joint_vote_finalized(100, true));
            let p = ResolutionIssuanceGov::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::Passed));

            let calls = EXEC_CALLS.with(|c| c.borrow().clone());
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].0, 0);
            assert_eq!(calls[0].1, 1000);
            assert_eq!(calls[0].2, reserve_council_accounts().len());
        });
    }

    #[test]
    fn finalize_rejects_missing_proposal() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                ResolutionIssuanceGov::finalize_joint_vote(RuntimeOrigin::root(), 99, true),
                pallet::Error::<Test>::ProposalNotFound
            );
        });
    }

    #[test]
    fn finalize_rejects_non_voting_passed_proposal() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));
            assert_ok!(ResolutionIssuanceGov::finalize_joint_vote(
                RuntimeOrigin::root(),
                0,
                true
            ));
            assert_noop!(
                ResolutionIssuanceGov::finalize_joint_vote(RuntimeOrigin::root(), 0, false),
                pallet::Error::<Test>::ProposalNotVoting
            );
        });
    }

    #[test]
    fn finalize_rejects_non_voting_rejected_proposal() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));
            assert_ok!(ResolutionIssuanceGov::finalize_joint_vote(
                RuntimeOrigin::root(),
                0,
                false
            ));
            assert_noop!(
                ResolutionIssuanceGov::finalize_joint_vote(RuntimeOrigin::root(), 0, true),
                pallet::Error::<Test>::ProposalNotVoting
            );
        });
    }

    #[test]
    fn rejected_callback_marks_rejected() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));

            assert_ok!(ResolutionIssuanceGov::on_joint_vote_finalized(100, false));
            let p = ResolutionIssuanceGov::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::Rejected));
        });
    }

    #[test]
    fn approved_callback_execution_failure_marks_execution_failed() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));

            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);
            assert_ok!(ResolutionIssuanceGov::on_joint_vote_finalized(100, true));

            let p = ResolutionIssuanceGov::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::ExecutionFailed));

            let calls = EXEC_CALLS.with(|c| c.borrow().clone());
            assert_eq!(calls.len(), 0);
        });
    }

    #[test]
    fn retry_failed_execution_can_recover_to_passed() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));

            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);
            assert_ok!(ResolutionIssuanceGov::on_joint_vote_finalized(100, true));

            let p = ResolutionIssuanceGov::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::ExecutionFailed));

            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = false);
            assert_ok!(ResolutionIssuanceGov::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));

            let p = ResolutionIssuanceGov::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::Passed));
            assert!(!pallet::RetryCount::<Test>::contains_key(0));
        });
    }

    #[test]
    fn retry_failed_execution_respects_retry_limit() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));

            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);
            assert_ok!(ResolutionIssuanceGov::on_joint_vote_finalized(100, true));

            assert_ok!(ResolutionIssuanceGov::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));
            assert_ok!(ResolutionIssuanceGov::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));
            assert_noop!(
                ResolutionIssuanceGov::retry_failed_execution(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    0
                ),
                pallet::Error::<Test>::MaxRetriesExceeded
            );
        });
    }

    #[test]
    fn retry_rejects_non_execution_failed_proposal() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));
            assert_noop!(
                ResolutionIssuanceGov::retry_failed_execution(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    0
                ),
                pallet::Error::<Test>::ProposalNotExecutionFailed
            );
        });
    }

    #[test]
    fn set_allowed_recipients_rejected_when_voting_exists() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));
            let recipients: BoundedVec<AccountId32, ConstU32<64>> = reserve_council_accounts()
                .try_into()
                .expect("recipients should fit");
            assert_noop!(
                ResolutionIssuanceGov::set_allowed_recipients(RuntimeOrigin::root(), recipients),
                pallet::Error::<Test>::ActiveVotingProposalsExist
            );
        });
    }

    #[test]
    fn set_allowed_recipients_rejects_empty_list() {
        new_test_ext().execute_with(|| {
            let recipients: BoundedVec<AccountId32, ConstU32<64>> =
                Vec::new().try_into().expect("empty should fit");
            assert_noop!(
                ResolutionIssuanceGov::set_allowed_recipients(RuntimeOrigin::root(), recipients),
                pallet::Error::<Test>::RecipientsNotConfigured
            );
        });
    }

    #[test]
    fn set_allowed_recipients_rejects_duplicates() {
        new_test_ext().execute_with(|| {
            let first = reserve_council_accounts()[0].clone();
            let recipients: BoundedVec<AccountId32, ConstU32<64>> = vec![first.clone(), first]
                .try_into()
                .expect("recipients should fit");
            assert_noop!(
                ResolutionIssuanceGov::set_allowed_recipients(RuntimeOrigin::root(), recipients),
                pallet::Error::<Test>::DuplicateAllowedRecipient
            );
        });
    }
}
