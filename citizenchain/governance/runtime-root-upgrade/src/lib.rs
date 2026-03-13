#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarks;
pub mod weights;

use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
use voting_engine_system::JointVoteResultCallback;

pub trait RuntimeCodeExecutor {
    /// 中文注释：由 Runtime 注入真正的 set_code 执行器，pallet 本身只负责编排治理状态机。
    fn execute_runtime_code(code: &[u8]) -> DispatchResult;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::Hash, DispatchError};
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
        /// 联合投票进行中，尚未得到最终治理结果。
        Voting,
        /// 联合投票已通过，且 runtime code 已执行成功。
        Passed,
        /// 联合投票被拒绝，治理流程结束。
        Rejected,
        /// 联合投票已通过，但 runtime code 执行失败，允许后续人工重试。
        ExecutionFailed,
        /// 执行失败且重试额度已经耗尽，人工确认放弃后清空 code。
        Cancelled,
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
        /// 提案发起人（仅允许 NRC 管理员）
        pub proposer: T::AccountId,
        /// 升级理由
        pub reason: ReasonOf<T>,
        /// 代码哈希，便于事件与链下审计对齐
        pub code_hash: T::Hash,
        /// 待执行 wasm code；执行成功或投票拒绝后会清空
        pub code: CodeOf<T>,
        /// 当前治理状态
        pub status: ProposalStatus,
    }

    use crate::weights::WeightInfo;

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

        #[pallet::constant]
        type MaxExecutionRetries: Get<u32>;

        type WeightInfo: crate::weights::WeightInfo;
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

    /// 执行失败后的重试次数；只统计手动 retry_failed_execution。
    #[pallet::storage]
    #[pallet::getter(fn retry_count)]
    pub type RetryCount<T> = StorageMap<_, Blake2_128Concat, u64, u32, ValueQuery>;

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
        RuntimeUpgradeCancelled {
            proposal_id: u64,
            code_hash: T::Hash,
            retry_count: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyReason,
        EmptyRuntimeCode,
        ProposalNotFound,
        ProposalNotVoting,
        ProposalNotExecutionFailed,
        ProposalNotRetryExhausted,
        JointVoteCreateFailed,
        JointVoteMappingNotFound,
        ProposalIdOverflow,
        MaxRetriesExceeded,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 国储会管理员发起 runtime 升级提案，升级流程走联合投票。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::propose_runtime_upgrade())]
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

            let proposal_id = Self::allocate_proposal_id()?;

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
        #[pallet::weight(if *approved {
            T::WeightInfo::finalize_joint_vote_approved().saturating_add(
                <<T as frame_system::Config>::SystemWeightInfo as frame_system::weights::WeightInfo>::set_code()
            )
        } else {
            T::WeightInfo::finalize_joint_vote_rejected()
        })]
        pub fn finalize_joint_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approved: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;
            Self::apply_joint_vote_result(proposal_id, approved)
        }

        /// 中文注释：联合投票已经通过但执行 runtime code 失败时，
        /// 允许 NRC 管理员在保留原始 code 的前提下重试执行。
        #[pallet::call_index(2)]
        #[pallet::weight(
            T::WeightInfo::retry_failed_execution().saturating_add(
                <<T as frame_system::Config>::SystemWeightInfo as frame_system::weights::WeightInfo>::set_code()
            )
        )]
        pub fn retry_failed_execution(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = T::NrcProposeOrigin::ensure_origin(origin)?;
            Self::retry_execution(proposal_id)
        }

        /// 中文注释：当执行失败提案已经用尽全部人工重试额度后，
        /// 允许 NRC 管理员确认放弃并清空残留 wasm code，释放长期占用的存储。
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::cancel_failed_proposal())]
        pub fn cancel_failed_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = T::NrcProposeOrigin::ensure_origin(origin)?;

            let mut proposal =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                matches!(proposal.status, ProposalStatus::ExecutionFailed),
                Error::<T>::ProposalNotExecutionFailed
            );

            let retry_count = RetryCount::<T>::get(proposal_id);
            ensure!(
                retry_count >= T::MaxExecutionRetries::get(),
                Error::<T>::ProposalNotRetryExhausted
            );

            proposal.status = ProposalStatus::Cancelled;
            proposal.code = Default::default();
            Proposals::<T>::insert(proposal_id, &proposal);
            RetryCount::<T>::remove(proposal_id);
            Self::deposit_event(Event::<T>::RuntimeUpgradeCancelled {
                proposal_id,
                code_hash: proposal.code_hash,
                retry_count,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn allocate_proposal_id() -> Result<u64, DispatchError> {
            // 中文注释：必须使用 checked_add，避免 u64::MAX 时回绕覆盖旧提案。
            let proposal_id = NextProposalId::<T>::get();
            let next = proposal_id
                .checked_add(1)
                .ok_or(Error::<T>::ProposalIdOverflow)?;
            NextProposalId::<T>::put(next);
            Ok(proposal_id)
        }

        fn cleanup_joint_vote_mapping(proposal_id: u64) {
            // 中文注释：联合投票一旦结束，就释放双向映射并清理投票引擎侧计票状态。
            if let Some(joint_vote_id) = GovToJointVote::<T>::take(proposal_id) {
                JointVoteToGov::<T>::remove(joint_vote_id);
                T::JointVoteEngine::cleanup_joint_proposal(joint_vote_id);
            }
        }

        pub(crate) fn apply_joint_vote_result(proposal_id: u64, approved: bool) -> DispatchResult {
            let mut proposal =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                matches!(proposal.status, ProposalStatus::Voting),
                Error::<T>::ProposalNotVoting
            );

            if approved {
                // 中文注释：联合投票一旦给出“通过”，联合阶段就结束，因此映射与投票快照可以先清理。
                Self::cleanup_joint_vote_mapping(proposal_id);

                let code_to_execute = proposal.code.clone();
                // 中文注释：只有真正 set_code 成功，提案才允许进入 Passed；
                // 否则必须保留 code 进入 ExecutionFailed，便于后续重试。
                if T::RuntimeCodeExecutor::execute_runtime_code(code_to_execute.as_slice()).is_ok()
                {
                    proposal.status = ProposalStatus::Passed;
                    proposal.code = Default::default();
                    RetryCount::<T>::remove(proposal_id);
                    Proposals::<T>::insert(proposal_id, &proposal);
                    Self::deposit_event(Event::<T>::JointVoteFinalized {
                        proposal_id,
                        approved: true,
                    });
                    Self::deposit_event(Event::<T>::RuntimeUpgradeExecuted {
                        proposal_id,
                        code_hash: proposal.code_hash,
                    });
                } else {
                    proposal.status = ProposalStatus::ExecutionFailed;
                    RetryCount::<T>::insert(proposal_id, 0);
                    Proposals::<T>::insert(proposal_id, &proposal);
                    Self::deposit_event(Event::<T>::JointVoteFinalized {
                        proposal_id,
                        approved: true,
                    });
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
                RetryCount::<T>::remove(proposal_id);
                Self::cleanup_joint_vote_mapping(proposal_id);

                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    approved: false,
                });
                return Ok(());
            }
        }

        fn retry_execution(proposal_id: u64) -> DispatchResult {
            let mut proposal =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                matches!(proposal.status, ProposalStatus::ExecutionFailed),
                Error::<T>::ProposalNotExecutionFailed
            );

            let retries = RetryCount::<T>::get(proposal_id);
            ensure!(
                retries < T::MaxExecutionRetries::get(),
                Error::<T>::MaxRetriesExceeded
            );

            // 中文注释：重试不会重新走联合投票，只对已经批准的同一份 code 再执行一次。
            if T::RuntimeCodeExecutor::execute_runtime_code(proposal.code.as_slice()).is_ok() {
                proposal.status = ProposalStatus::Passed;
                proposal.code = Default::default();
                Proposals::<T>::insert(proposal_id, &proposal);
                RetryCount::<T>::remove(proposal_id);
                Self::deposit_event(Event::<T>::RuntimeUpgradeExecuted {
                    proposal_id,
                    code_hash: proposal.code_hash,
                });
            } else {
                RetryCount::<T>::insert(proposal_id, retries.saturating_add(1));
                Self::deposit_event(Event::<T>::RuntimeUpgradeExecutionFailed {
                    proposal_id,
                    code_hash: proposal.code_hash,
                });
            }

            Ok(())
        }
    }
}

impl<T: pallet::Config> JointVoteResultCallback for pallet::Pallet<T> {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult {
        // 中文注释：投票引擎只认识 joint_vote_id，本模块需先反查自己的 proposal_id。
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
        type MaxExecutionRetries = ConstU32<3>;
        type WeightInfo = ();
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

    fn propose_ok() {
        assert_ok!(RuntimeRootUpgrade::propose_runtime_upgrade(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            code_ok(),
            10,
            nonce_ok(),
            sig_ok()
        ));
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
    fn proposal_id_overflow_is_rejected() {
        new_test_ext().execute_with(|| {
            NextProposalId::<Test>::put(u64::MAX);

            assert_noop!(
                RuntimeRootUpgrade::propose_runtime_upgrade(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    code_ok(),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::ProposalIdOverflow
            );
        });
    }

    #[test]
    fn rejected_joint_vote_marks_proposal_rejected() {
        new_test_ext().execute_with(|| {
            propose_ok();

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
            propose_ok();

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
    fn approved_joint_vote_execution_failure_enters_retryable_state() {
        new_test_ext().execute_with(|| {
            propose_ok();
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                true
            ));

            let p = RuntimeRootUpgrade::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::ExecutionFailed));
            assert!(
                !p.code.is_empty(),
                "proposal code should be retained for retry"
            );
            assert_eq!(RuntimeRootUpgrade::retry_count(0), 0);
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
            propose_ok();

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

    #[test]
    fn retry_failed_execution_succeeds_and_clears_code() {
        new_test_ext().execute_with(|| {
            propose_ok();
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                true
            ));

            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = false);
            assert_ok!(RuntimeRootUpgrade::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));

            let p = RuntimeRootUpgrade::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::Passed));
            assert!(p.code.is_empty());
            assert_eq!(RuntimeRootUpgrade::retry_count(0), 0);
        });
    }

    #[test]
    fn retry_failed_execution_failure_increments_retry_count() {
        new_test_ext().execute_with(|| {
            propose_ok();
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                true
            ));

            assert_ok!(RuntimeRootUpgrade::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));
            assert_eq!(RuntimeRootUpgrade::retry_count(0), 1);
            let p = RuntimeRootUpgrade::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::ExecutionFailed));
            assert!(!p.code.is_empty());
        });
    }

    #[test]
    fn retry_failed_execution_respects_retry_limit() {
        new_test_ext().execute_with(|| {
            propose_ok();
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                true
            ));

            assert_ok!(RuntimeRootUpgrade::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));
            assert_ok!(RuntimeRootUpgrade::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));
            assert_ok!(RuntimeRootUpgrade::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));
            assert_eq!(RuntimeRootUpgrade::retry_count(0), 3);

            assert_noop!(
                RuntimeRootUpgrade::retry_failed_execution(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    0
                ),
                pallet::Error::<Test>::MaxRetriesExceeded
            );
        });
    }

    #[test]
    fn cancel_failed_proposal_clears_code_after_retry_limit() {
        new_test_ext().execute_with(|| {
            propose_ok();
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                true
            ));

            assert_ok!(RuntimeRootUpgrade::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));
            assert_ok!(RuntimeRootUpgrade::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));
            assert_ok!(RuntimeRootUpgrade::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));
            assert_eq!(RuntimeRootUpgrade::retry_count(0), 3);

            assert_ok!(RuntimeRootUpgrade::cancel_failed_proposal(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));

            let p = RuntimeRootUpgrade::proposals(0).expect("proposal should exist");
            assert!(matches!(p.status, pallet::ProposalStatus::Cancelled));
            assert!(
                p.code.is_empty(),
                "cancel should clear retained runtime code"
            );
            assert_eq!(RuntimeRootUpgrade::retry_count(0), 0);

            assert_noop!(
                RuntimeRootUpgrade::retry_failed_execution(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    0
                ),
                pallet::Error::<Test>::ProposalNotExecutionFailed
            );
        });
    }

    #[test]
    fn cancel_failed_proposal_requires_exhausted_retries() {
        new_test_ext().execute_with(|| {
            propose_ok();
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

            let joint_vote_id =
                RuntimeRootUpgrade::gov_to_joint_vote(0).expect("joint vote id should exist");
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(
                joint_vote_id,
                true
            ));

            assert_ok!(RuntimeRootUpgrade::retry_failed_execution(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                0
            ));

            assert_noop!(
                RuntimeRootUpgrade::cancel_failed_proposal(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    0
                ),
                pallet::Error::<Test>::ProposalNotRetryExhausted
            );
        });
    }

    #[test]
    fn retry_failed_execution_requires_failed_state() {
        new_test_ext().execute_with(|| {
            propose_ok();

            assert_noop!(
                RuntimeRootUpgrade::retry_failed_execution(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    0
                ),
                pallet::Error::<Test>::ProposalNotExecutionFailed
            );
        });
    }
}
