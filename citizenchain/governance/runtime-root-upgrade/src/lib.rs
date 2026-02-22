#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
use voting_engine_system::JointVoteResultCallback;

pub trait RuntimeCodeExecutor {
    fn execute_runtime_code(code: &[u8]) -> DispatchResult;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Hash;
    use voting_engine_system::JointVoteEngine;

    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type CodeOf<T> = BoundedVec<u8, <T as Config>::MaxRuntimeCodeSize>;
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
        pub code_hash: T::Hash,
        pub code: CodeOf<T>,
        pub status: ProposalStatus,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 仅允许国储会管理员发起 runtime 升级提案。
        type NrcProposeOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

        type JointVoteEngine: JointVoteEngine<Self::AccountId>;
        type RuntimeCodeExecutor: RuntimeCodeExecutor;

        #[pallet::constant]
        type MaxReasonLen: Get<u32>;

        #[pallet::constant]
        type MaxRuntimeCodeSize: Get<u32>;

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

    /// 本模块提案ID -> 联合投票提案ID
    #[pallet::storage]
    #[pallet::getter(fn gov_to_joint_vote)]
    pub type GovToJointVote<T> = StorageMap<_, Blake2_128Concat, u64, u64, OptionQuery>;

    /// 联合投票提案ID -> 本模块提案ID
    #[pallet::storage]
    #[pallet::getter(fn joint_vote_to_gov)]
    pub type JointVoteToGov<T> = StorageMap<_, Blake2_128Concat, u64, u64, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        RuntimeUpgradeProposed {
            proposal_id: u64,
            joint_vote_id: u64,
            proposer: T::AccountId,
            code_hash: T::Hash,
        },
        JointVoteFinalized {
            proposal_id: u64,
            approved: bool,
        },
        RuntimeUpgradeExecuted {
            proposal_id: u64,
            code_hash: T::Hash,
        },
        RuntimeUpgradeExecutionFailed {
            proposal_id: u64,
            code_hash: T::Hash,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyReason,
        EmptyRuntimeCode,
        ProposalNotFound,
        ProposalNotVoting,
        JointVoteCreateFailed,
        JointVoteMappingNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 国储会管理员发起 runtime 升级提案，升级流程走联合投票。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 3))]
        pub fn propose_runtime_upgrade(
            origin: OriginFor<T>,
            reason: ReasonOf<T>,
            code: CodeOf<T>,
            eligible_total: u64,
            snapshot_nonce: SnapshotNonceOf<T>,
            snapshot_signature: SnapshotSignatureOf<T>,
        ) -> DispatchResult {
            let proposer = T::NrcProposeOrigin::ensure_origin(origin)?;

            ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
            ensure!(!code.is_empty(), Error::<T>::EmptyRuntimeCode);

            let proposal_id = NextProposalId::<T>::get();
            NextProposalId::<T>::put(proposal_id.saturating_add(1));

            let joint_vote_id = T::JointVoteEngine::create_joint_proposal(
                proposer.clone(),
                eligible_total,
                snapshot_nonce.as_slice(),
                snapshot_signature.as_slice(),
            )
            .map_err(|_| Error::<T>::JointVoteCreateFailed)?;

            let code_hash = T::Hashing::hash(code.as_slice());
            let proposal = Proposal::<T> {
                proposer: proposer.clone(),
                reason,
                code_hash,
                code: code.clone(),
                status: ProposalStatus::Voting,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            GovToJointVote::<T>::insert(proposal_id, joint_vote_id);
            JointVoteToGov::<T>::insert(joint_vote_id, proposal_id);

            Self::deposit_event(Event::<T>::RuntimeUpgradeProposed {
                proposal_id,
                joint_vote_id,
                proposer,
                code_hash,
            });
            Ok(())
        }

        /// 联合投票回调：保持与其他治理模块一致，Root 可手工回放。
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
                let code_to_execute = proposal.code.clone();
                proposal.status = ProposalStatus::Passed;
                proposal.code = Default::default();
                Proposals::<T>::insert(proposal_id, &proposal);
                if let Some(joint_vote_id) = GovToJointVote::<T>::take(proposal_id) {
                    JointVoteToGov::<T>::remove(joint_vote_id);
                }

                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    approved: true,
                });

                // 中文注释：联合投票通过即治理流程结束；执行结果单独记录，不回滚治理状态。
                if T::RuntimeCodeExecutor::execute_runtime_code(code_to_execute.as_slice()).is_ok()
                {
                    Self::deposit_event(Event::<T>::RuntimeUpgradeExecuted {
                        proposal_id,
                        code_hash: proposal.code_hash,
                    });
                } else {
                    Self::deposit_event(Event::<T>::RuntimeUpgradeExecutionFailed {
                        proposal_id,
                        code_hash: proposal.code_hash,
                    });
                }
                return Ok(());
            } else {
                proposal.status = ProposalStatus::Rejected;
                proposal.code = Default::default();
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
        pub type RuntimeRootUpgrade = super;
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
    }
    thread_local! {
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

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type NrcProposeOrigin = EnsureNrcAdminForTest;
        type JointVoteEngine = TestJointVoteEngine;
        type RuntimeCodeExecutor = TestRuntimeCodeExecutor;
        type MaxReasonLen = ConstU32<64>;
        type MaxRuntimeCodeSize = ConstU32<1024>;
        type MaxSnapshotNonceLength = ConstU32<64>;
        type MaxSnapshotSignatureLength = ConstU32<64>;
    }

    thread_local! {
        static RUNTIME_CODE_EXECUTED: RefCell<bool> = const { RefCell::new(false) };
    }

    pub struct TestRuntimeCodeExecutor;
    impl RuntimeCodeExecutor for TestRuntimeCodeExecutor {
        fn execute_runtime_code(_code: &[u8]) -> DispatchResult {
            let should_fail = EXEC_SHOULD_FAIL.with(|v| *v.borrow());
            if should_fail {
                return Err(DispatchError::Other("set_code failed"));
            }
            RUNTIME_CODE_EXECUTED.with(|v| *v.borrow_mut() = true);
            Ok(())
        }
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");
        let mut ext: sp_io::TestExternalities = storage.into();
        ext.execute_with(|| {
            RUNTIME_CODE_EXECUTED.with(|v| *v.borrow_mut() = false);
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = false);
            NEXT_JOINT_ID.with(|id| *id.borrow_mut() = 100);
        });
        ext
    }

    fn reason_ok() -> pallet::ReasonOf<Test> {
        b"upgrade reason"
            .to_vec()
            .try_into()
            .expect("reason should fit")
    }

    fn code_ok() -> pallet::CodeOf<Test> {
        vec![1, 2, 3, 4, 5]
            .try_into()
            .expect("runtime code should fit")
    }

    fn nonce_ok() -> pallet::SnapshotNonceOf<Test> {
        b"snap-nonce"
            .to_vec()
            .try_into()
            .expect("snapshot nonce should fit")
    }

    fn sig_ok() -> pallet::SnapshotSignatureOf<Test> {
        b"snap-signature"
            .to_vec()
            .try_into()
            .expect("snapshot signature should fit")
    }

    #[test]
    fn only_nrc_admin_can_propose_runtime_upgrade() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                RuntimeRootUpgrade::propose_runtime_upgrade(
                    RuntimeOrigin::signed(AccountId32::new([2u8; 32])),
                    reason_ok(),
                    code_ok(),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                sp_runtime::DispatchError::BadOrigin
            );

            assert_ok!(RuntimeRootUpgrade::propose_runtime_upgrade(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                code_ok(),
                10,
                nonce_ok(),
                sig_ok()
            ));
        });
    }

    #[test]
    fn joint_vote_callback_rejects_when_mapping_missing() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                RuntimeRootUpgrade::on_joint_vote_finalized(999, true),
                pallet::Error::<Test>::JointVoteMappingNotFound
            );
        });
    }

    #[test]
    fn rejected_joint_vote_marks_proposal_rejected() {
        new_test_ext().execute_with(|| {
            assert_ok!(RuntimeRootUpgrade::propose_runtime_upgrade(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                code_ok(),
                10,
                nonce_ok(),
                sig_ok()
            ));

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                false
            ));
            let p = RuntimeRootUpgrade::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::Rejected));
            assert!(RuntimeRootUpgrade::gov_to_joint_vote(0).is_none());
            assert!(RuntimeRootUpgrade::joint_vote_to_gov(joint_vote_id).is_none());
        });
    }

    #[test]
    fn approved_joint_vote_executes_runtime_upgrade() {
        new_test_ext().execute_with(|| {
            assert_ok!(RuntimeRootUpgrade::propose_runtime_upgrade(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                code_ok(),
                10,
                nonce_ok(),
                sig_ok()
            ));

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                true
            ));

            let p = RuntimeRootUpgrade::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::Passed));
            assert!(
                p.code.is_empty(),
                "proposal code should be cleared after finalize"
            );
            let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
            assert!(code_executed, "runtime code executor should be called");
            assert!(RuntimeRootUpgrade::gov_to_joint_vote(0).is_none());
            assert!(RuntimeRootUpgrade::joint_vote_to_gov(joint_vote_id).is_none());
        });
    }

    #[test]
    fn approved_joint_vote_execution_failure_keeps_passed() {
        new_test_ext().execute_with(|| {
            assert_ok!(RuntimeRootUpgrade::propose_runtime_upgrade(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                code_ok(),
                10,
                nonce_ok(),
                sig_ok()
            ));
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                true
            ));

            let p = RuntimeRootUpgrade::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::Passed));
            assert!(
                p.code.is_empty(),
                "proposal code should be cleared after finalize"
            );
            let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
            assert!(
                !code_executed,
                "runtime code executor should fail in this test"
            );
            assert!(RuntimeRootUpgrade::gov_to_joint_vote(0).is_none());
            assert!(RuntimeRootUpgrade::joint_vote_to_gov(joint_vote_id).is_none());
        });
    }

    #[test]
    fn rejected_joint_vote_clears_runtime_code() {
        new_test_ext().execute_with(|| {
            assert_ok!(RuntimeRootUpgrade::propose_runtime_upgrade(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                code_ok(),
                10,
                nonce_ok(),
                sig_ok()
            ));

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                false
            ));
            let p = RuntimeRootUpgrade::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::Rejected));
            assert!(
                p.code.is_empty(),
                "proposal code should be cleared after finalize"
            );
        });
    }
}
