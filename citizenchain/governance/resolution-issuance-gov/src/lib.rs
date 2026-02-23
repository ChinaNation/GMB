#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
use voting_engine_system::JointVoteResultCallback;

#[frame_support::pallet]
pub mod pallet {
    use codec::Encode;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives::china::china_cb::CHINA_CB;
    use resolution_issuance_iss::ResolutionIssuanceExecutor;
    use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
    use voting_engine_system::JointVoteEngine;

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

        /// 投票通过后，调用发行执行模块执行铸币。
        type IssuanceExecutor: ResolutionIssuanceExecutor<Self::AccountId>;
        type JointVoteEngine: JointVoteEngine<Self::AccountId>;

        #[pallet::constant]
        type MaxReasonLen: Get<u32>;

        #[pallet::constant]
        type MaxAllocations: Get<u32>;

        #[pallet::constant]
        type MaxSnapshotNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxSnapshotSignatureLength: Get<u32>;
    }

    #[pallet::pallet]
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
        IssuanceExecutionFailed {
            proposal_id: u64,
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
            eligible_total: u64,
            snapshot_nonce: SnapshotNonceOf<T>,
            snapshot_signature: SnapshotSignatureOf<T>,
        ) -> DispatchResult {
            let proposer = T::NrcProposeOrigin::ensure_origin(origin)?;

            ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
            Self::validate_allocations(total_amount, allocations.as_slice())?;

            let proposal_id = NextProposalId::<T>::get();
            NextProposalId::<T>::put(proposal_id.saturating_add(1));
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
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 4))]
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
        pub(crate) fn apply_joint_vote_result(proposal_id: u64, approved: bool) -> DispatchResult {
            let mut proposal =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                matches!(proposal.status, ProposalStatus::Voting),
                Error::<T>::ProposalNotVoting
            );

            if approved {
                // 中文注释：联合投票通过即治理流程结束，状态固定为 Passed。
                proposal.status = ProposalStatus::Passed;
                Proposals::<T>::insert(proposal_id, &proposal);
                if let Some(joint_vote_id) = GovToJointVote::<T>::take(proposal_id) {
                    JointVoteToGov::<T>::remove(joint_vote_id);
                }

                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    approved: true,
                });

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
                    Self::deposit_event(Event::<T>::IssuanceExecutionTriggered {
                        proposal_id,
                        total_amount: proposal.total_amount,
                    });
                } else {
                    Self::deposit_event(Event::<T>::IssuanceExecutionFailed { proposal_id });
                }
                return Ok(());
            } else {
                proposal.status = ProposalStatus::Rejected;
                Proposals::<T>::insert(proposal_id, &proposal);
                if let Some(joint_vote_id) = GovToJointVote::<T>::take(proposal_id) {
                    JointVoteToGov::<T>::remove(joint_vote_id);
                }
                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    approved: false,
                });
                return Ok(());
            }
        }

        fn validate_allocations(
            total_amount: u128,
            allocations: &[RecipientAmount<T::AccountId>],
        ) -> DispatchResult {
            ensure!(!allocations.is_empty(), Error::<T>::EmptyAllocations);
            let expected: BTreeSet<Vec<u8>> = CHINA_CB
                .iter()
                .skip(1)
                .map(|node| node.duoqian_address.to_vec())
                .collect();
            ensure!(
                allocations.len() == expected.len(),
                Error::<T>::InvalidAllocationCount
            );
            let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();

            let mut sum = 0u128;
            for item in allocations {
                ensure!(item.amount > 0, Error::<T>::ZeroAmount);
                let who = item.recipient.encode();
                ensure!(seen.insert(who), Error::<T>::DuplicateRecipient);
                sum = sum
                    .checked_add(item.amount)
                    .ok_or(Error::<T>::AllocationOverflow)?;
            }

            ensure!(seen == expected, Error::<T>::InvalidRecipientSet);
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;
    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32};
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
            Err(())
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
    impl resolution_issuance_iss::ResolutionIssuanceExecutor<AccountId32> for TestIssuanceExecutor {
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
                calls.borrow_mut()
                    .push((proposal_id, total_amount, allocations.len()));
            });
            Ok(())
        }
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type NrcProposeOrigin = EnsureNrcAdminForTest;
        type IssuanceExecutor = TestIssuanceExecutor;
        type JointVoteEngine = TestJointVoteEngine;
        type MaxReasonLen = ConstU32<128>;
        type MaxAllocations = ConstU32<64>;
        type MaxSnapshotNonceLength = ConstU32<64>;
        type MaxSnapshotSignatureLength = ConstU32<64>;
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
        });
        ext
    }

    fn reason_ok() -> pallet::ReasonOf<Test> {
        b"issuance"
            .to_vec()
            .try_into()
            .expect("reason should fit")
    }

    fn nonce_ok() -> pallet::SnapshotNonceOf<Test> {
        b"snap-nonce"
            .to_vec()
            .try_into()
            .expect("nonce should fit")
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
            v.push(pallet::RecipientAmount {
                recipient,
                amount,
            });
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
    fn approved_callback_execution_failure_keeps_passed() {
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
            assert!(matches!(p.status, pallet::ProposalStatus::Passed));

            let calls = EXEC_CALLS.with(|c| c.borrow().clone());
            assert_eq!(calls.len(), 0);
        });
    }
}
