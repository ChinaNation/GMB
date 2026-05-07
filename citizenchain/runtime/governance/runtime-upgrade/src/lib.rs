#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
use votingengine::JointVoteResultCallback;

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
    use votingengine::JointVoteEngine;

    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type CodeOf<T> = BoundedVec<u8, <T as Config>::MaxRuntimeCodeSize>;
    pub type SnapshotNonceOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotNonceLength>;
    pub type SnapshotSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotSignatureLength>;
    pub const PROPOSAL_OBJECT_KIND_RUNTIME_WASM: u8 = 1;

    /// 提案摘要数据：序列化后存入 votingengine 的 ProposalData。
    /// 大对象 wasm code 单独写入 votingengine 的 ProposalObject。
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
        /// 创建时摘要状态；真实投票/执行终态由 votingengine 维护。
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
        /// 创建时默认状态；生产回调路径不再回写该字段。
        Voting,
        /// 历史兼容枚举，真实成功终态读取 votingengine STATUS_EXECUTED。
        Passed,
        /// 历史兼容枚举，真实否决终态读取 votingengine STATUS_REJECTED。
        Rejected,
        /// 历史兼容枚举，真实失败终态读取 votingengine STATUS_EXECUTION_FAILED。
        ExecutionFailed,
    }

    use crate::weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
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

    // 提案数据、元数据均已移至 votingengine 统一管控，本模块不再持有任何 Storage。

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
        /// ADR-008 step3:`(province, signer_admin_pubkey)` 双层匹配字段必填,
        /// 由 votingengine PopulationSnapshotVerifier 走 `ShengSigningPubkey` 派生公钥验签。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_runtime_upgrade())]
        pub fn propose_runtime_upgrade(
            origin: OriginFor<T>,
            reason: ReasonOf<T>,
            code: CodeOf<T>,
            eligible_total: u64,
            snapshot_nonce: SnapshotNonceOf<T>,
            signature: SnapshotSignatureOf<T>,
            province: BoundedVec<u8, ConstU32<64>>,
            signer_admin_pubkey: [u8; 32],
        ) -> DispatchResult {
            let proposer = T::ProposeOrigin::ensure_origin(origin)?;

            ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
            ensure!(!code.is_empty(), Error::<T>::EmptyRuntimeCode);

            let code_vec = code.into_inner();
            let code_hash = T::Hashing::hash(code_vec.as_slice());
            let proposal = Proposal::<T> {
                proposer: proposer.clone(),
                reason,
                code_hash,
                status: ProposalStatus::Voting,
            };
            let mut encoded = sp_runtime::sp_std::vec::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&proposal.encode());
            let proposal_id = T::JointVoteEngine::create_joint_proposal_with_data_and_object(
                proposer.clone(),
                eligible_total,
                snapshot_nonce.as_slice(),
                signature.as_slice(),
                province.as_slice(),
                &signer_admin_pubkey,
                crate::MODULE_TAG,
                encoded,
                PROPOSAL_OBJECT_KIND_RUNTIME_WASM,
                code_vec,
            )
            .map_err(|_| Error::<T>::JointVoteCreateFailed)?;

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

        // call_index = 1 保持空缺：联合投票终结只能由 votingengine 回调进入，
        // 不再暴露 Root 手工回放 extrinsic，避免形成第二条执行入口。
    }

    impl<T: Config> Pallet<T> {
        /// 快速判断 proposal_id 是否属于本模块（通过 ProposalOwner 匹配）。
        pub fn owns_proposal(proposal_id: u64) -> bool {
            votingengine::Pallet::<T>::is_proposal_owner(proposal_id, crate::MODULE_TAG)
        }

        /// 从投票引擎 ProposalData 中读取并解码本模块的提案摘要。
        /// 先校验 MODULE_TAG 前缀，防止跨模块误解码。
        pub(crate) fn load_proposal(proposal_id: u64) -> Result<Proposal<T>, DispatchError> {
            let raw = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            let tag = crate::MODULE_TAG;
            if raw.len() < tag.len() || &raw[..tag.len()] != tag {
                return Err(Error::<T>::ProposalNotFound.into());
            }
            Proposal::<T>::decode(&mut &raw[tag.len()..])
                .map_err(|_| Error::<T>::ProposalNotFound.into())
        }

        fn load_runtime_code(proposal_id: u64) -> Result<CodeOf<T>, DispatchError> {
            let meta = votingengine::Pallet::<T>::get_proposal_object_meta(proposal_id)
                .ok_or(Error::<T>::RuntimeCodeMissing)?;
            ensure!(
                meta.kind == PROPOSAL_OBJECT_KIND_RUNTIME_WASM,
                Error::<T>::RuntimeCodeMissing
            );
            let raw = votingengine::Pallet::<T>::get_proposal_object(proposal_id)
                .ok_or(Error::<T>::RuntimeCodeMissing)?;
            raw.try_into()
                .map_err(|_| Error::<T>::RuntimeCodeMissing.into())
        }

        /// 联合投票结果回调（由 votingengine 的 set_status_and_emit 在事务内调用）。
        ///
        /// 状态处理模式与 votingengine 对齐：
        /// - approved + 执行成功 → 返回 `Executed`，由投票引擎写 STATUS_EXECUTED。
        /// - approved + 执行失败 → 返回 `FatalFailed`，由投票引擎写 STATUS_EXECUTION_FAILED。
        /// - rejected → 返回 `Executed`，投票引擎保留 STATUS_REJECTED。
        pub(crate) fn apply_joint_vote_result(
            proposal_id: u64,
            approved: bool,
        ) -> Result<votingengine::ProposalExecutionOutcome, DispatchError> {
            let proposal = Self::load_proposal(proposal_id)?;
            if let Some(engine_proposal) = votingengine::Pallet::<T>::proposals(proposal_id) {
                let expected_status = if approved {
                    votingengine::STATUS_PASSED
                } else {
                    votingengine::STATUS_REJECTED
                };
                ensure!(
                    engine_proposal.status == expected_status,
                    Error::<T>::ProposalNotVoting
                );
            } else {
                ensure!(
                    matches!(proposal.status, ProposalStatus::Voting),
                    Error::<T>::ProposalNotVoting
                );
            }

            if approved {
                let code_to_execute = Self::load_runtime_code(proposal_id)?;
                let exec_ok =
                    T::RuntimeCodeExecutor::execute_runtime_code(code_to_execute.as_slice())
                        .is_ok();

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
                Ok(if exec_ok {
                    votingengine::ProposalExecutionOutcome::Executed
                } else {
                    votingengine::ProposalExecutionOutcome::FatalFailed
                })
            } else {
                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    approved: false,
                });
                Ok(votingengine::ProposalExecutionOutcome::Executed)
            }
        }
    }
}

impl<T: pallet::Config> JointVoteResultCallback for pallet::Pallet<T> {
    fn on_joint_vote_finalized(
        vote_proposal_id: u64,
        approved: bool,
    ) -> Result<votingengine::ProposalExecutionOutcome, sp_runtime::DispatchError> {
        // 中文注释：统一使用 voting engine 的 proposal_id，不再需要反查映射。
        pallet::Pallet::<T>::apply_joint_vote_result(vote_proposal_id, approved)
    }
}

#[cfg(test)]
mod tests;
