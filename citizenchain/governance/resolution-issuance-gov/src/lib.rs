#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::pallet_prelude::DispatchResult;
use voting_engine_system::JointVoteResultCallback;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, PalletId};
    use frame_system::pallet_prelude::*;
    use resolution_issuance_iss::ResolutionIssuanceExecutor;
    use voting_engine_system::JointVoteEngine;

    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type AllocationOf<T> = BoundedVec<
        RecipientAmount<<T as frame_system::Config>::AccountId>,
        <T as Config>::MaxAllocations,
    >;

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
        Executed,
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
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 仅允许国储会管理员发起提案。
        type NrcProposeOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

        /// 国储会模块ID（固定为 `nrcgch01`）。
        type NrcPalletId: Get<PalletId>;

        /// 投票通过后，调用发行执行模块执行铸币。
        type IssuanceExecutor: ResolutionIssuanceExecutor<Self::AccountId>;
        type JointVoteEngine: JointVoteEngine;

        #[pallet::constant]
        type MaxReasonLen: Get<u32>;

        #[pallet::constant]
        type MaxAllocations: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, Proposal<T>, OptionQuery>;

    /// 决议发行提案ID -> 联合投票提案ID
    #[pallet::storage]
    #[pallet::getter(fn gov_to_joint_vote)]
    pub type GovToJointVote<T> = StorageMap<_, Blake2_128Concat, u64, u64, OptionQuery>;

    /// 联合投票提案ID -> 决议发行提案ID
    #[pallet::storage]
    #[pallet::getter(fn joint_vote_to_gov)]
    pub type JointVoteToGov<T> = StorageMap<_, Blake2_128Concat, u64, u64, OptionQuery>;

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
            approved: bool,
        },
        IssuanceExecutionTriggered {
            proposal_id: u64,
            total_amount: u128,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyReason,
        EmptyAllocations,
        ZeroAmount,
        AllocationOverflow,
        TotalMismatch,
        ProposalNotFound,
        ProposalNotVoting,
        JointVoteCreateFailed,
        JointVoteMappingNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 国储会提案：创建“决议发行”联合投票提案。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 2))]
        pub fn propose_resolution_issuance(
            origin: OriginFor<T>,
            reason: ReasonOf<T>,
            total_amount: u128,
            allocations: AllocationOf<T>,
        ) -> DispatchResult {
            let proposer = T::NrcProposeOrigin::ensure_origin(origin)?;

            ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
            Self::validate_allocations(total_amount, allocations.as_slice())?;

            let proposal_id = NextProposalId::<T>::get();
            NextProposalId::<T>::put(proposal_id.saturating_add(1));
            let joint_vote_id = T::JointVoteEngine::create_joint_proposal()
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
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn finalize_joint_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approved: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;
            Self::apply_joint_vote_result(proposal_id, approved)
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn apply_joint_vote_result(
            proposal_id: u64,
            approved: bool,
        ) -> DispatchResult {
            let mut proposal =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                matches!(proposal.status, ProposalStatus::Voting),
                Error::<T>::ProposalNotVoting
            );

            if approved {
                proposal.status = ProposalStatus::Passed;
                Proposals::<T>::insert(proposal_id, &proposal);

                let execute_allocations = proposal
                    .allocations
                    .iter()
                    .map(|x| (x.recipient.clone(), x.amount))
                    .collect();

                T::IssuanceExecutor::execute_resolution_issuance(
                    proposal_id,
                    proposal.reason.to_vec(),
                    proposal.total_amount,
                    execute_allocations,
                )?;

                proposal.status = ProposalStatus::Executed;
                Proposals::<T>::insert(proposal_id, &proposal);

                Self::deposit_event(Event::<T>::IssuanceExecutionTriggered {
                    proposal_id,
                    total_amount: proposal.total_amount,
                });
            } else {
                proposal.status = ProposalStatus::Rejected;
                Proposals::<T>::insert(proposal_id, &proposal);
            }

            Self::deposit_event(Event::<T>::JointVoteFinalized {
                proposal_id,
                approved,
            });
            Ok(())
        }

        fn validate_allocations(
            total_amount: u128,
            allocations: &[RecipientAmount<T::AccountId>],
        ) -> DispatchResult {
            ensure!(!allocations.is_empty(), Error::<T>::EmptyAllocations);

            let mut sum = 0u128;
            for item in allocations {
                ensure!(item.amount > 0, Error::<T>::ZeroAmount);
                sum = sum
                    .checked_add(item.amount)
                    .ok_or(Error::<T>::AllocationOverflow)?;
            }
            ensure!(sum == total_amount, Error::<T>::TotalMismatch);
            Ok(())
        }
    }
}

impl<T: pallet::Config> JointVoteResultCallback for pallet::Pallet<T> {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult {
        let gov_id = pallet::JointVoteToGov::<T>::get(vote_proposal_id)
            .ok_or(pallet::Error::<T>::JointVoteMappingNotFound)?;
        pallet::Pallet::<T>::apply_joint_vote_result(gov_id, approved)
    }
}
