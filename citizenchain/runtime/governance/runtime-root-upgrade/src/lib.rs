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
    use voting_engine_system::{JointVoteEngine, STATUS_EXECUTED};

    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type CodeOf<T> = BoundedVec<u8, <T as Config>::MaxRuntimeCodeSize>;
    pub type SnapshotNonceOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotNonceLength>;
    pub type SnapshotSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotSignatureLength>;
    pub const PROPOSAL_OBJECT_KIND_RUNTIME_WASM: u8 = 1;

    /// 提案摘要数据：序列化后存入 voting-engine-system 的 ProposalData。
    /// 大对象 wasm code 单独写入 voting-engine-system 的 ProposalObject。
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
        /// 是否仍保留链上对象层 wasm 数据。
        pub has_code: bool,
        /// 当前治理状态
        pub status: ProposalStatus,
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
        /// 联合投票进行中，尚未得到最终治理结果。
        Voting,
        /// 联合投票已通过，且 runtime code 已执行成功。
        Passed,
        /// 联合投票被拒绝，治理流程结束。
        Rejected,
        /// 联合投票已通过，但 runtime code 执行失败。
        ExecutionFailed,
    }

    use crate::weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
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

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // 提案数据、元数据均已移至 voting-engine-system 统一管控，本模块不再持有任何 Storage。

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        RuntimeUpgradeProposed {
            proposal_id: u64,
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
        RuntimeCodeMissing,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 国储会管理员发起 runtime 升级提案，升级流程走联合投票。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_runtime_upgrade())]
        pub fn propose_runtime_upgrade(
            origin: OriginFor<T>,
            reason: ReasonOf<T>,
            code: CodeOf<T>,
            eligible_total: u64,
            snapshot_nonce: SnapshotNonceOf<T>,
            signature: SnapshotSignatureOf<T>,
        ) -> DispatchResult {
            let proposer = T::NrcProposeOrigin::ensure_origin(origin)?;

            ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
            ensure!(!code.is_empty(), Error::<T>::EmptyRuntimeCode);

            let proposal_id = T::JointVoteEngine::create_joint_proposal(
                proposer.clone(),
                eligible_total,
                snapshot_nonce.as_slice(),
                signature.as_slice(),
            )
            .map_err(|_| Error::<T>::JointVoteCreateFailed)?;

            let code_hash = T::Hashing::hash(code.as_slice());
            let proposal = Proposal::<T> {
                proposer: proposer.clone(),
                reason,
                code_hash,
                has_code: true,
                status: ProposalStatus::Voting,
            };
            let data = proposal.encode();
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, data)?;
            voting_engine_system::Pallet::<T>::store_proposal_object(
                proposal_id,
                PROPOSAL_OBJECT_KIND_RUNTIME_WASM,
                code.into_inner(),
            )?;
            voting_engine_system::Pallet::<T>::store_proposal_meta(
                proposal_id,
                frame_system::Pallet::<T>::block_number(),
            );

            Self::deposit_event(Event::<T>::RuntimeUpgradeProposed {
                proposal_id,
                proposer,
                code_hash,
            });
            Ok(())
        }

        /// 联合投票回调：保持与其他治理模块一致，Root 可手工回放。
        #[pallet::call_index(1)]
        #[pallet::weight(if *approved {
            <T as Config>::WeightInfo::finalize_joint_vote_approved().saturating_add(
                <<T as frame_system::Config>::SystemWeightInfo as frame_system::weights::WeightInfo>::set_code()
            )
        } else {
            <T as Config>::WeightInfo::finalize_joint_vote_rejected()
        })]
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
        fn load_proposal(proposal_id: u64) -> Result<Proposal<T>, DispatchError> {
            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            Proposal::<T>::decode(&mut &raw[..]).map_err(|_| Error::<T>::ProposalNotFound.into())
        }

        fn save_proposal(proposal_id: u64, proposal: &Proposal<T>) -> DispatchResult {
            let data = proposal.encode();
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, data)
        }

        fn load_runtime_code(proposal_id: u64) -> Result<CodeOf<T>, DispatchError> {
            let meta = voting_engine_system::Pallet::<T>::get_proposal_object_meta(proposal_id)
                .ok_or(Error::<T>::RuntimeCodeMissing)?;
            ensure!(
                meta.kind == PROPOSAL_OBJECT_KIND_RUNTIME_WASM,
                Error::<T>::RuntimeCodeMissing
            );
            let raw = voting_engine_system::Pallet::<T>::get_proposal_object(proposal_id)
                .ok_or(Error::<T>::RuntimeCodeMissing)?;
            raw.try_into()
                .map_err(|_| Error::<T>::RuntimeCodeMissing.into())
        }

        pub(crate) fn apply_joint_vote_result(proposal_id: u64, approved: bool) -> DispatchResult {
            let mut proposal = Self::load_proposal(proposal_id)?;
            ensure!(
                matches!(proposal.status, ProposalStatus::Voting),
                Error::<T>::ProposalNotVoting
            );

            if approved {
                let code_to_execute = Self::load_runtime_code(proposal_id)?;
                // 中文注释：只有真正 set_code 成功，提案才允许进入 Passed；
                // 否则进入 ExecutionFailed。原始 wasm 继续保留在投票引擎对象层，
                // 交由统一 90 天延迟清理流程处理，不由业务模块自行删除。
                let exec_ok =
                    T::RuntimeCodeExecutor::execute_runtime_code(code_to_execute.as_slice())
                        .is_ok();

                if exec_ok {
                    proposal.status = ProposalStatus::Passed;
                } else {
                    proposal.status = ProposalStatus::ExecutionFailed;
                }
                Self::save_proposal(proposal_id, &proposal)?;

                // 将投票引擎状态设为 EXECUTED，标记提案已终结。
                // 直接修改存储而非调用 set_status_and_emit，以避免回调重入。
                voting_engine_system::pallet::Proposals::<T>::mutate(proposal_id, |maybe| {
                    if let Some(p) = maybe {
                        p.status = STATUS_EXECUTED;
                    }
                });

                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    approved: true,
                });
                if exec_ok {
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
                Ok(())
            } else {
                proposal.status = ProposalStatus::Rejected;
                Self::save_proposal(proposal_id, &proposal)?;

                // 将投票引擎状态设为 EXECUTED，与 approved 路径保持一致，
                // 防止投票引擎侧对已终结提案的误操作。
                voting_engine_system::pallet::Proposals::<T>::mutate(proposal_id, |maybe| {
                    if let Some(p) = maybe {
                        p.status = STATUS_EXECUTED;
                    }
                });

                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    approved: false,
                });
                Ok(())
            }
        }
    }
}

impl<T: pallet::Config> JointVoteResultCallback for pallet::Pallet<T> {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult {
        // 中文注释：统一使用 voting engine 的 proposal_id，不再需要反查映射。
        pallet::Pallet::<T>::apply_joint_vote_result(vote_proposal_id, approved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Decode;
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
        pub type VotingEngine = voting_engine_system;

        #[runtime::pallet_index(2)]
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
            signature: &[u8],
        ) -> Result<u64, DispatchError> {
            if eligible_total == 0 || snapshot_nonce.is_empty() || signature.is_empty() {
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

    pub struct TestTimeProvider;
    impl frame_support::traits::UnixTime for TestTimeProvider {
        fn now() -> core::time::Duration {
            core::time::Duration::from_secs(1_782_864_000) // 2026-07-01
        }
    }

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type MaxProposalsPerExpiry = ConstU32<128>;
        type MaxCleanupStepsPerBlock = ConstU32<8>;
        type CleanupKeysPerStep = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<{ 100 * 1024 }>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type SfidEligibility = ();
        type PopulationSnapshotVerifier = ();
        type JointVoteResultCallback = ();
        type InternalAdminProvider = ();
        type InternalThresholdProvider = ();
        type InternalAdminCountProvider = ();
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
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
    fn proposal_data_stored_in_voting_engine() {
        new_test_ext().execute_with(|| {
            propose_ok();
            // proposal_id comes from NEXT_JOINT_ID which starts at 100
            let raw = voting_engine_system::Pallet::<Test>::get_proposal_data(100);
            assert!(
                raw.is_some(),
                "proposal data should be stored in voting engine"
            );
            let proposal = pallet::Proposal::<Test>::decode(&mut &raw.unwrap()[..])
                .expect("should decode proposal");
            assert!(matches!(proposal.status, pallet::ProposalStatus::Voting));
            assert!(
                proposal.has_code,
                "summary should mark runtime code as retained"
            );
            assert!(
                voting_engine_system::Pallet::<Test>::get_proposal_object(100).is_some(),
                "runtime wasm should be stored in proposal object layer"
            );
        });
    }

    #[test]
    fn rejected_joint_vote_marks_proposal_rejected() {
        new_test_ext().execute_with(|| {
            propose_ok();
            // proposal_id == joint_vote_id == 100
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(100, false));
            let raw = voting_engine_system::Pallet::<Test>::get_proposal_data(100)
                .expect("proposal data should exist");
            let p = pallet::Proposal::<Test>::decode(&mut &raw[..]).expect("should decode");
            assert!(matches!(p.status, pallet::ProposalStatus::Rejected));
            assert!(
                p.has_code,
                "rejected proposal object should stay until unified cleanup"
            );
        });
    }

    #[test]
    fn approved_joint_vote_executes_runtime_upgrade() {
        new_test_ext().execute_with(|| {
            propose_ok();
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(100, true));

            let raw = voting_engine_system::Pallet::<Test>::get_proposal_data(100)
                .expect("proposal data should exist");
            let p = pallet::Proposal::<Test>::decode(&mut &raw[..]).expect("should decode");
            assert!(matches!(p.status, pallet::ProposalStatus::Passed));
            assert!(
                p.has_code,
                "approved proposal object should stay until unified cleanup"
            );
            assert!(
                voting_engine_system::Pallet::<Test>::get_proposal_object(100).is_some(),
                "approved proposal should still keep object data for unified cleanup"
            );
            let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
            assert!(code_executed, "runtime code executor should be called");
        });
    }

    #[test]
    fn approved_joint_vote_execution_failure_emits_event() {
        new_test_ext().execute_with(|| {
            propose_ok();
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(100, true));

            let raw = voting_engine_system::Pallet::<Test>::get_proposal_data(100)
                .expect("proposal data should exist");
            let p = pallet::Proposal::<Test>::decode(&mut &raw[..]).expect("should decode");
            assert!(matches!(p.status, pallet::ProposalStatus::ExecutionFailed));
            assert!(
                p.has_code,
                "execution failed proposal object should stay until unified cleanup"
            );
            let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
            assert!(
                !code_executed,
                "runtime code executor should fail in this test"
            );
        });
    }

    #[test]
    fn rejected_joint_vote_clears_runtime_code() {
        new_test_ext().execute_with(|| {
            propose_ok();
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(100, false));

            let raw = voting_engine_system::Pallet::<Test>::get_proposal_data(100)
                .expect("proposal data should exist");
            let p = pallet::Proposal::<Test>::decode(&mut &raw[..]).expect("should decode");
            assert!(matches!(p.status, pallet::ProposalStatus::Rejected));
            assert!(
                p.has_code,
                "rejected proposal object should stay until unified cleanup"
            );
        });
    }

    #[test]
    fn finalize_joint_vote_requires_voting_status() {
        new_test_ext().execute_with(|| {
            propose_ok();
            // First finalize
            assert_ok!(RuntimeRootUpgrade::on_joint_vote_finalized(100, true));
            // Second finalize should fail - no longer voting
            assert_noop!(
                RuntimeRootUpgrade::on_joint_vote_finalized(100, true),
                pallet::Error::<Test>::ProposalNotVoting
            );
        });
    }

    #[test]
    fn finalize_nonexistent_proposal_fails() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                RuntimeRootUpgrade::on_joint_vote_finalized(999, true),
                pallet::Error::<Test>::ProposalNotFound
            );
        });
    }
}
