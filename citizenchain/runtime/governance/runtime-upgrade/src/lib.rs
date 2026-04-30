#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
use voting_engine::JointVoteResultCallback;

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"rt-upg";

pub trait RuntimeCodeExecutor {
    /// 中文注释：由 Runtime 注入真正的 set_code 执行器，pallet 本身只负责编排治理状态机。
    fn execute_runtime_code(code: &[u8]) -> DispatchResult;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use genesis_pallet::DeveloperUpgradeCheck;
    use sp_runtime::{traits::Hash, DispatchError};
    use voting_engine::JointVoteEngine;

    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type CodeOf<T> = BoundedVec<u8, <T as Config>::MaxRuntimeCodeSize>;
    pub type SnapshotNonceOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotNonceLength>;
    pub type SnapshotSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotSignatureLength>;
    pub const PROPOSAL_OBJECT_KIND_RUNTIME_WASM: u8 = 1;

    /// 提案摘要数据：序列化后存入 voting-engine 的 ProposalData。
    /// 大对象 wasm code 单独写入 voting-engine 的 ProposalObject。
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
        /// 提案发起人（国储会或省储会管理员）
        pub proposer: T::AccountId,
        /// 升级理由
        pub reason: ReasonOf<T>,
        /// 代码哈希，便于事件与链下审计对齐
        pub code_hash: T::Hash,
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
    pub trait Config: frame_system::Config + voting_engine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 允许国储会或省储会管理员发起 runtime 升级提案。
        type ProposeOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

        type JointVoteEngine: JointVoteEngine<Self::AccountId>;
        type RuntimeCodeExecutor: RuntimeCodeExecutor;

        /// 开发者直升 runtime 开关检查（由 genesis-pallet 注入）。
        type DeveloperUpgradeCheck: genesis_pallet::DeveloperUpgradeCheck;

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

    // 提案数据、元数据均已移至 voting-engine 统一管控，本模块不再持有任何 Storage。

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
        /// 开发期直接升级成功（不走投票）。
        DeveloperDirectUpgradeExecuted {
            who: T::AccountId,
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
        /// 开发者直升已关闭（链已进入运行期）。
        DeveloperUpgradeDisabled,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 国储会或省储会管理员发起 runtime 升级提案，升级流程走联合投票。
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
            let proposer = T::ProposeOrigin::ensure_origin(origin)?;

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
                status: ProposalStatus::Voting,
            };
            let mut encoded = sp_runtime::sp_std::vec::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&proposal.encode());
            voting_engine::Pallet::<T>::store_proposal_data(proposal_id, encoded)?;
            voting_engine::Pallet::<T>::store_proposal_object(
                proposal_id,
                PROPOSAL_OBJECT_KIND_RUNTIME_WASM,
                code.into_inner(),
            )?;
            voting_engine::Pallet::<T>::store_proposal_meta(
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

        /// 开发期快捷通道：联合提案发起人直接 set_code，不走投票。
        /// 仅在 genesis-pallet 的 DeveloperUpgradeEnabled 为 true 时可用。
        /// 链进入运行期后此调用永久失效，升级必须走 propose_runtime_upgrade 联合投票。
        #[pallet::call_index(2)]
        #[pallet::weight(
            <<T as frame_system::Config>::SystemWeightInfo as frame_system::weights::WeightInfo>::set_code()
        )]
        pub fn developer_direct_upgrade(origin: OriginFor<T>, code: CodeOf<T>) -> DispatchResult {
            let who = T::ProposeOrigin::ensure_origin(origin)?;
            ensure!(
                T::DeveloperUpgradeCheck::is_enabled(),
                Error::<T>::DeveloperUpgradeDisabled
            );
            ensure!(!code.is_empty(), Error::<T>::EmptyRuntimeCode);
            let code_hash = T::Hashing::hash(code.as_slice());
            T::RuntimeCodeExecutor::execute_runtime_code(code.as_slice())?;
            Self::deposit_event(Event::<T>::DeveloperDirectUpgradeExecuted { who, code_hash });
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
        /// 快速判断 proposal_id 是否属于本模块（通过 MODULE_TAG 前缀匹配）。
        pub fn owns_proposal(proposal_id: u64) -> bool {
            voting_engine::Pallet::<T>::get_proposal_data(proposal_id)
                .map(|raw| raw.starts_with(crate::MODULE_TAG))
                .unwrap_or(false)
        }

        /// 从投票引擎 ProposalData 中读取并解码本模块的提案摘要。
        /// 先校验 MODULE_TAG 前缀，防止跨模块误解码。
        pub(crate) fn load_proposal(proposal_id: u64) -> Result<Proposal<T>, DispatchError> {
            let raw = voting_engine::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            let tag = crate::MODULE_TAG;
            if raw.len() < tag.len() || &raw[..tag.len()] != tag {
                return Err(Error::<T>::ProposalNotFound.into());
            }
            Proposal::<T>::decode(&mut &raw[tag.len()..])
                .map_err(|_| Error::<T>::ProposalNotFound.into())
        }

        fn save_proposal(proposal_id: u64, proposal: &Proposal<T>) -> DispatchResult {
            let mut encoded = sp_runtime::sp_std::vec::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&proposal.encode());
            voting_engine::Pallet::<T>::store_proposal_data(proposal_id, encoded)
        }

        fn load_runtime_code(proposal_id: u64) -> Result<CodeOf<T>, DispatchError> {
            let meta = voting_engine::Pallet::<T>::get_proposal_object_meta(proposal_id)
                .ok_or(Error::<T>::RuntimeCodeMissing)?;
            ensure!(
                meta.kind == PROPOSAL_OBJECT_KIND_RUNTIME_WASM,
                Error::<T>::RuntimeCodeMissing
            );
            let raw = voting_engine::Pallet::<T>::get_proposal_object(proposal_id)
                .ok_or(Error::<T>::RuntimeCodeMissing)?;
            raw.try_into()
                .map_err(|_| Error::<T>::RuntimeCodeMissing.into())
        }

        /// 联合投票结果回调（由 voting-engine 的 set_status_and_emit 在事务内调用）。
        ///
        /// 状态处理模式与 resolution-issuance 对齐：
        /// - approved + 执行成功 → 业务状态记为 Passed，投票引擎回调外层再写 STATUS_EXECUTED
        /// - approved + 执行失败 → 业务状态记为 ExecutionFailed，投票引擎回调外层再写 STATUS_EXECUTION_FAILED
        /// - rejected → 不覆盖投票引擎状态，保持 set_status_and_emit 设置的 STATUS_REJECTED
        pub(crate) fn apply_joint_vote_result(proposal_id: u64, approved: bool) -> DispatchResult {
            let mut proposal = Self::load_proposal(proposal_id)?;
            ensure!(
                matches!(proposal.status, ProposalStatus::Voting),
                Error::<T>::ProposalNotVoting
            );

            if approved {
                let code_to_execute = Self::load_runtime_code(proposal_id)?;
                let exec_ok =
                    T::RuntimeCodeExecutor::execute_runtime_code(code_to_execute.as_slice())
                        .is_ok();

                if exec_ok {
                    proposal.status = ProposalStatus::Passed;
                } else {
                    proposal.status = ProposalStatus::ExecutionFailed;
                }
                Self::save_proposal(proposal_id, &proposal)?;

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
                // 否决：只更新业务层数据，不覆盖投票引擎状态（保持 STATUS_REJECTED）。
                proposal.status = ProposalStatus::Rejected;
                Self::save_proposal(proposal_id, &proposal)?;

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
        pallet::Pallet::<T>::apply_joint_vote_result(vote_proposal_id, approved)?;
        if approved {
            let proposal = pallet::Pallet::<T>::load_proposal(vote_proposal_id)?;
            match proposal.status {
                pallet::ProposalStatus::Passed => {
                    voting_engine::Pallet::<T>::set_callback_execution_result(
                        vote_proposal_id,
                        voting_engine::STATUS_EXECUTED,
                    )?;
                }
                pallet::ProposalStatus::ExecutionFailed => {
                    voting_engine::Pallet::<T>::set_callback_execution_result(
                        vote_proposal_id,
                        voting_engine::STATUS_EXECUTION_FAILED,
                    )?;
                }
                _ => {}
            }
        }
        Ok(())
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
        pub type VotingEngine = voting_engine;

        #[runtime::pallet_index(2)]
        pub type RuntimeUpgrade = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    pub struct EnsureJointProposerForTest;
    impl frame_support::traits::EnsureOrigin<RuntimeOrigin> for EnsureJointProposerForTest {
        type Success = AccountId32;

        fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
            let who = frame_system::EnsureSigned::<AccountId32>::try_origin(o)?;
            if who == nrc_admin() || who == prc_admin() {
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
    impl voting_engine::JointVoteEngine<AccountId32> for TestJointVoteEngine {
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

    impl voting_engine::Config for Test {
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
        type InternalVoteResultCallback = ();
        type InternalAdminProvider = ();
        type InternalThresholdProvider = ();
        type InternalAdminCountProvider = ();
        type MaxAdminsPerInstitution = ConstU32<32>;
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    // 测试用开发者直升开关：默认开启，可通过 thread_local 控制。
    thread_local! {
        static DEV_UPGRADE_ENABLED: RefCell<bool> = const { RefCell::new(true) };
    }
    pub struct TestDeveloperUpgradeCheck;
    impl genesis_pallet::DeveloperUpgradeCheck for TestDeveloperUpgradeCheck {
        fn is_enabled() -> bool {
            DEV_UPGRADE_ENABLED.with(|v| *v.borrow())
        }
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type ProposeOrigin = EnsureJointProposerForTest;
        type JointVoteEngine = TestJointVoteEngine;
        type RuntimeCodeExecutor = TestRuntimeCodeExecutor;
        type DeveloperUpgradeCheck = TestDeveloperUpgradeCheck;
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
            DEV_UPGRADE_ENABLED.with(|v| *v.borrow_mut() = true);
        });
        ext
    }

    fn nrc_admin() -> AccountId32 {
        AccountId32::new([1u8; 32])
    }

    fn outsider() -> AccountId32 {
        AccountId32::new([2u8; 32])
    }

    fn prc_admin() -> AccountId32 {
        AccountId32::new([3u8; 32])
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

    /// 从 ProposalData 读取并跳过 MODULE_TAG 后 decode 提案摘要。
    fn decode_proposal(proposal_id: u64) -> pallet::Proposal<Test> {
        let raw = voting_engine::Pallet::<Test>::get_proposal_data(proposal_id)
            .expect("proposal data should exist");
        let tag = crate::MODULE_TAG;
        assert!(
            raw.len() >= tag.len() && &raw[..tag.len()] == tag,
            "MODULE_TAG mismatch"
        );
        pallet::Proposal::<Test>::decode(&mut &raw[tag.len()..]).expect("should decode proposal")
    }

    fn propose_ok() {
        assert_ok!(RuntimeUpgrade::propose_runtime_upgrade(
            RuntimeOrigin::signed(nrc_admin()),
            reason_ok(),
            code_ok(),
            10,
            nonce_ok(),
            sig_ok()
        ));
    }

    /// 在投票引擎中插入一个 PASSED 状态的 Proposal，使回调执行结果写入可用。
    /// 测试 mock 的 TestJointVoteEngine 不创建真实 Proposals 条目，
    /// 需手工补一个以模拟真实回调上下文。
    fn insert_engine_proposal(proposal_id: u64) {
        voting_engine::pallet::Proposals::<Test>::insert(
            proposal_id,
            voting_engine::Proposal {
                kind: voting_engine::PROPOSAL_KIND_JOINT,
                stage: voting_engine::STAGE_JOINT,
                status: voting_engine::STATUS_PASSED,
                internal_org: None,
                internal_institution: None,
                start: 0u64,
                end: 100u64,
                citizen_eligible_total: 10,
            },
        );
    }

    fn call_joint_callback(proposal_id: u64, approved: bool) -> DispatchResult {
        voting_engine::pallet::CallbackExecutionScopes::<Test>::insert(proposal_id, ());
        let result = RuntimeUpgrade::on_joint_vote_finalized(proposal_id, approved);
        voting_engine::pallet::CallbackExecutionScopes::<Test>::remove(proposal_id);
        result
    }

    #[test]
    fn joint_proposers_can_propose_runtime_upgrade() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                RuntimeUpgrade::propose_runtime_upgrade(
                    RuntimeOrigin::signed(outsider()),
                    reason_ok(),
                    code_ok(),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                sp_runtime::DispatchError::BadOrigin
            );

            assert_ok!(RuntimeUpgrade::propose_runtime_upgrade(
                RuntimeOrigin::signed(nrc_admin()),
                reason_ok(),
                code_ok(),
                10,
                nonce_ok(),
                sig_ok()
            ));

            assert_ok!(RuntimeUpgrade::propose_runtime_upgrade(
                RuntimeOrigin::signed(prc_admin()),
                reason_ok(),
                code_ok(),
                10,
                nonce_ok(),
                sig_ok()
            ));

            assert!(
                voting_engine::Pallet::<Test>::get_proposal_data(100).is_some(),
                "NRC proposer should create proposal data"
            );
            assert!(
                voting_engine::Pallet::<Test>::get_proposal_data(101).is_some(),
                "PRC proposer should create proposal data"
            );
        });
    }

    #[test]
    fn proposal_data_stored_in_voting_engine() {
        new_test_ext().execute_with(|| {
            propose_ok();
            // proposal_id comes from NEXT_JOINT_ID which starts at 100
            assert!(
                voting_engine::Pallet::<Test>::get_proposal_data(100).is_some(),
                "proposal data should be stored in voting engine"
            );
            let proposal = decode_proposal(100);
            assert!(matches!(proposal.status, pallet::ProposalStatus::Voting));
            assert!(
                voting_engine::Pallet::<Test>::get_proposal_object(100).is_some(),
                "runtime wasm should be stored in proposal object layer"
            );
        });
    }

    #[test]
    fn rejected_joint_vote_marks_proposal_rejected() {
        new_test_ext().execute_with(|| {
            propose_ok();
            // proposal_id == joint_vote_id == 100
            assert_ok!(call_joint_callback(100, false));
            let p = decode_proposal(100);
            assert!(matches!(p.status, pallet::ProposalStatus::Rejected));
        });
    }

    #[test]
    fn approved_joint_vote_executes_runtime_upgrade() {
        new_test_ext().execute_with(|| {
            propose_ok();
            insert_engine_proposal(100);
            assert_ok!(call_joint_callback(100, true));

            let p = decode_proposal(100);
            assert!(matches!(p.status, pallet::ProposalStatus::Passed));
            assert!(
                voting_engine::Pallet::<Test>::get_proposal_object(100).is_some(),
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
            insert_engine_proposal(100);
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);

            assert_ok!(call_joint_callback(100, true));

            let p = decode_proposal(100);
            assert!(matches!(p.status, pallet::ProposalStatus::ExecutionFailed));
            let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
            assert!(
                !code_executed,
                "runtime code executor should fail in this test"
            );
            // 投票引擎侧应为 STATUS_EXECUTION_FAILED
            let engine_proposal = voting_engine::pallet::Proposals::<Test>::get(100).unwrap();
            assert_eq!(
                engine_proposal.status,
                voting_engine::STATUS_EXECUTION_FAILED
            );
        });
    }

    #[test]
    fn rejected_joint_vote_retains_object_for_unified_cleanup() {
        new_test_ext().execute_with(|| {
            propose_ok();
            assert_ok!(call_joint_callback(100, false));

            let p = decode_proposal(100);
            assert!(matches!(p.status, pallet::ProposalStatus::Rejected));
            assert!(
                voting_engine::Pallet::<Test>::get_proposal_object(100).is_some(),
                "rejected proposal object should stay until unified cleanup"
            );
        });
    }

    #[test]
    fn owns_proposal_returns_true_for_own_proposals() {
        new_test_ext().execute_with(|| {
            propose_ok();
            assert!(pallet::Pallet::<Test>::owns_proposal(100));
            assert!(!pallet::Pallet::<Test>::owns_proposal(999));
        });
    }

    #[test]
    fn approved_success_marks_engine_status_executed() {
        new_test_ext().execute_with(|| {
            propose_ok();
            insert_engine_proposal(100);
            assert_ok!(call_joint_callback(100, true));

            // 执行成功时在回调作用域内静默写入 EXECUTED，最终事件由投票引擎外层发出。
            let engine_proposal = voting_engine::pallet::Proposals::<Test>::get(100).unwrap();
            assert_eq!(
                engine_proposal.status,
                voting_engine::STATUS_EXECUTED,
                "success path should mark engine status executed"
            );
        });
    }

    #[test]
    fn finalize_joint_vote_requires_voting_status() {
        new_test_ext().execute_with(|| {
            propose_ok();
            insert_engine_proposal(100);
            // First finalize
            assert_ok!(call_joint_callback(100, true));
            // Second finalize should fail - no longer voting
            assert_noop!(
                call_joint_callback(100, true),
                pallet::Error::<Test>::ProposalNotVoting
            );
        });
    }

    #[test]
    fn finalize_nonexistent_proposal_fails() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                call_joint_callback(999, true),
                pallet::Error::<Test>::ProposalNotFound
            );
        });
    }

    // ─── developer_direct_upgrade 测试 ─────────────────────────────────

    #[test]
    fn developer_direct_upgrade_allows_joint_proposer_when_enabled() {
        new_test_ext().execute_with(|| {
            assert_ok!(RuntimeUpgrade::developer_direct_upgrade(
                RuntimeOrigin::signed(nrc_admin()),
                code_ok(),
            ));
            let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
            assert!(code_executed, "runtime code executor should be called");

            RUNTIME_CODE_EXECUTED.with(|v| *v.borrow_mut() = false);

            assert_ok!(RuntimeUpgrade::developer_direct_upgrade(
                RuntimeOrigin::signed(prc_admin()),
                code_ok(),
            ));
            let code_executed = RUNTIME_CODE_EXECUTED.with(|v| *v.borrow());
            assert!(
                code_executed,
                "PRC proposer should also trigger runtime code executor"
            );
        });
    }

    #[test]
    fn developer_direct_upgrade_fails_when_disabled() {
        new_test_ext().execute_with(|| {
            DEV_UPGRADE_ENABLED.with(|v| *v.borrow_mut() = false);
            assert_noop!(
                RuntimeUpgrade::developer_direct_upgrade(
                    RuntimeOrigin::signed(nrc_admin()),
                    code_ok(),
                ),
                pallet::Error::<Test>::DeveloperUpgradeDisabled
            );
        });
    }

    #[test]
    fn developer_direct_upgrade_rejects_non_joint_proposer() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                RuntimeUpgrade::developer_direct_upgrade(
                    RuntimeOrigin::signed(outsider()),
                    code_ok(),
                ),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn developer_direct_upgrade_rejects_empty_code() {
        new_test_ext().execute_with(|| {
            let empty_code: pallet::CodeOf<Test> = vec![].try_into().expect("empty code");
            assert_noop!(
                RuntimeUpgrade::developer_direct_upgrade(
                    RuntimeOrigin::signed(nrc_admin()),
                    empty_code,
                ),
                pallet::Error::<Test>::EmptyRuntimeCode
            );
        });
    }
}
