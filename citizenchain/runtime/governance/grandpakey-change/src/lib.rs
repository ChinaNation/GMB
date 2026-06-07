//! # GRANDPA 密钥治理模块 (grandpakey-change)
//!
//! 本模块将"机构 GRANDPA 公钥替换"包装成受治理约束的链上流程：
//! - 仅国储会（NRC）与省储会（PRC）可发起密钥替换提案。
//! - 仅目标机构内部管理员可参与提案/投票/执行/清理。
//! - 借助 `votingengine` 内部投票达成通过后，调用 `pallet-grandpa::schedule_change` 变更 authority set。
//! - 新公钥必须通过 ed25519 有效性校验和 small-order 弱公钥拒绝。
//!
//! 投票通过后自动尝试执行；若因 GRANDPA pending change 暂时失败，可手动重试或取消。

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use curve25519_dalek::edwards::CompressedEdwardsY;
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{GetStorageVersion, StorageVersion},
    weights::Weight,
    Blake2_128Concat,
};
use frame_system::pallet_prelude::*;
use primitives::china::china_cb::CHINA_CB;
use scale_info::TypeInfo;
use sp_consensus_grandpa::AuthorityId as GrandpaAuthorityId;
use sp_core::ed25519;
use votingengine::{
    types::{ORG_NRC, ORG_PRC},
    InternalVoteResultCallback, ProposalCancelDecision, ProposalExecutionOutcome, STATUS_PASSED,
};

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"gra-key";

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
/// 中文注释：密钥替换提案动作，封装机构、旧公钥和新公钥。
pub struct GrandpaKeyReplacementAction<AccountId> {
    pub institution: AccountId,
    pub old_key: [u8; 32],
    pub new_key: [u8; 32],
}

fn decode_account<T: frame_system::Config>(raw: &[u8; 32]) -> Option<T::AccountId> {
    T::AccountId::decode(&mut &raw[..]).ok()
}

fn nrc_account<T: frame_system::Config>() -> Option<T::AccountId> {
    CHINA_CB
        .first()
        .and_then(|n| decode_account::<T>(&n.main_address))
}

/// 中文注释：判断机构属于 NRC 还是 PRC，不属于任何一类则返回 None。
/// PRB（省储行）不参与 GRANDPA 共识出块，故不纳入密钥治理范围。
fn account_org<T: frame_system::Config>(institution: T::AccountId) -> Option<u8> {
    if Some(institution.clone()) == nrc_account::<T>() {
        return Some(ORG_NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| decode_account::<T>(&n.main_address))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRC);
    }

    None
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use sp_std::vec::Vec;
    use votingengine::{InternalAdminProvider, InternalVoteEngine};

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config + pallet_grandpa::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type GrandpaChangeDelay: Get<BlockNumberFor<Self>>;

        /// 中文注释：内部投票引擎（返回真实 proposal_id，避免猜测 next_proposal_id）。
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 中文注释：机构当前 GRANDPA 公钥，治理认可的目标 key（真正生效由 pallet-grandpa delay 控制）。
    #[pallet::storage]
    #[pallet::getter(fn current_grandpa_key)]
    pub type CurrentGrandpaKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, [u8; 32], OptionQuery>;

    /// 中文注释：公钥到机构的反向索引，O(1) 判断 new_key 是否已被其他机构占用。
    #[pallet::storage]
    #[pallet::getter(fn key_owner)]
    pub type GrandpaKeyOwnerByKey<T: Config> =
        StorageMap<_, Blake2_128Concat, [u8; 32], T::AccountId, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub _phantom: core::marker::PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // 中文注释：初始 GRANDPA 公钥与 CHINA_CB 的机构地址一一对应（1 国储会 + 43 省储会）。
            for node in CHINA_CB.iter() {
                let Some(institution) = decode_account::<T>(&node.main_address) else {
                    panic!(
                        "genesis: sfid_number {} 主账户 decode 失败",
                        node.sfid_number
                    );
                };
                assert!(
                    !GrandpaKeyOwnerByKey::<T>::contains_key(node.grandpa_key),
                    "duplicated initial grandpa key in CHINA_CB"
                );
                CurrentGrandpaKeys::<T>::insert(institution.clone(), node.grandpa_key);
                GrandpaKeyOwnerByKey::<T>::insert(node.grandpa_key, institution);
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let onchain = Pallet::<T>::on_chain_storage_version();
            if onchain < 2 {
                let mut reads: u64 = 1;
                let clear_result = GrandpaKeyOwnerByKey::<T>::clear(u32::MAX, None);
                let mut writes: u64 = 1u64.saturating_add(clear_result.unique as u64);
                for (inst, key) in CurrentGrandpaKeys::<T>::iter() {
                    reads = reads.saturating_add(1);
                    GrandpaKeyOwnerByKey::<T>::insert(key, inst);
                    writes = writes.saturating_add(1);
                }
                STORAGE_VERSION.put::<Pallet<T>>();
                return T::DbWeight::get().reads_writes(reads, writes);
            }
            Weight::zero()
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
            let mut seen = sp_std::collections::btree_set::BTreeSet::new();
            let mut count: u32 = 0;
            for (_inst, key) in CurrentGrandpaKeys::<T>::iter() {
                ensure!(
                    seen.insert(key),
                    "CurrentGrandpaKeys 中存在重复 GRANDPA 公钥"
                );
                count = count.saturating_add(1);
            }
            Ok(count.encode())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            let expected_count = u32::decode(&mut &state[..]).map_err(|_| {
                sp_runtime::TryRuntimeError::Other("grandpakey-change pre_upgrade 状态解码失败")
            })?;

            ensure!(
                Pallet::<T>::on_chain_storage_version() >= STORAGE_VERSION,
                "grandpakey-change storage version 未升级到 v2"
            );

            let mut current_count: u32 = 0;
            for (inst, key) in CurrentGrandpaKeys::<T>::iter() {
                current_count = current_count.saturating_add(1);
                ensure!(
                    GrandpaKeyOwnerByKey::<T>::get(key) == Some(inst),
                    "GrandpaKeyOwnerByKey 反向索引与 CurrentGrandpaKeys 不一致"
                );
            }

            let reverse_count = GrandpaKeyOwnerByKey::<T>::iter().count() as u32;
            ensure!(
                current_count == expected_count,
                "CurrentGrandpaKeys 数量在迁移前后不一致"
            );
            ensure!(
                reverse_count == current_count,
                "GrandpaKeyOwnerByKey 数量与 CurrentGrandpaKeys 不一致"
            );

            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 已发起 GRANDPA 密钥替换提案（并已在投票引擎创建内部提案）
        GrandpaKeyReplacementProposed {
            proposal_id: u64,
            org: u8,
            institution: T::AccountId,
            proposer: T::AccountId,
            old_key: [u8; 32],
            new_key: [u8; 32],
        },
        /// GRANDPA 密钥替换提案已提交一票
        GrandpaKeyVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 提案达到通过状态但自动执行失败（投票不回滚）
        GrandpaKeyExecutionFailed { proposal_id: u64 },
        /// GRANDPA 密钥替换已完成并已调度 GRANDPA authority set 变更
        GrandpaKeyReplaced {
            proposal_id: u64,
            institution: T::AccountId,
            old_key: [u8; 32],
            new_key: [u8; 32],
        },
        /// 已通过但不可执行的提案被取消
        FailedProposalCancelled {
            proposal_id: u64,
            institution: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 中文注释：机构不属于 NRC 或 PRC。
        InvalidInstitution,
        /// 中文注释：调用者不是该机构的内部管理员。
        UnauthorizedAdmin,
        /// 中文注释：提案动作数据未找到或解码失败。
        ProposalActionNotFound,
        /// 中文注释：提案未达到通过状态，不可执行。
        ProposalNotPassed,
        /// 中文注释：机构当前 GRANDPA 公钥未找到（创世未初始化）。
        CurrentGrandpaKeyNotFound,
        /// 中文注释：新公钥不能为全零值。
        NewKeyIsZero,
        /// 中文注释：新公钥不是有效的 ed25519 曲线点，或为 small-order 弱公钥。
        InvalidEd25519Key,
        /// 中文注释：新公钥与当前公钥相同，无需替换。
        NewKeyUnchanged,
        /// 中文注释：新公钥已被其他机构占用或替换后 authority set 中出现重复。
        NewKeyAlreadyUsed,
        /// 中文注释：提案绑定的旧公钥已不在当前 GRANDPA authority set 中。
        OldAuthorityNotFound,
        /// 中文注释：当前已有待生效的 GRANDPA authority set 变更，需等待其完成。
        GrandpaChangePending,
        /// 中文注释：提案仍可执行，不允许误取消。
        ProposalStillExecutable,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起“GRANDPA 密钥替换”内部投票提案（仅支持国储会/省储会）。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_replace_grandpa_key())]
        pub fn propose_replace_grandpa_key(
            origin: OriginFor<T>,
            institution: T::AccountId,
            new_key: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(new_key != [0u8; 32], Error::<T>::NewKeyIsZero);
            let point = CompressedEdwardsY(new_key)
                .decompress()
                .ok_or(Error::<T>::InvalidEd25519Key)?;
            // 中文注释：仅”能解压”为曲线点还不够，small-order 弱公钥可能导致 GRANDPA 签名安全性失真。
            ensure!(!point.is_small_order(), Error::<T>::InvalidEd25519Key);

            let actual_org =
                account_org::<T>(institution.clone()).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(actual_org, institution.clone(), &who),
                Error::<T>::UnauthorizedAdmin
            );

            let old_key = CurrentGrandpaKeys::<T>::get(institution.clone())
                .ok_or(Error::<T>::CurrentGrandpaKeyNotFound)?;
            ensure!(new_key != old_key, Error::<T>::NewKeyUnchanged);
            ensure!(
                !Self::is_key_used_by_other_institution(institution.clone(), &new_key),
                Error::<T>::NewKeyAlreadyUsed
            );

            let action = GrandpaKeyReplacementAction::<T::AccountId> {
                institution: institution.clone(),
                old_key,
                new_key,
            };

            let mut encoded = sp_std::vec::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&action.encode());
            let proposal_id = T::InternalVoteEngine::create_general_internal_proposal_with_data(
                who.clone(),
                actual_org,
                institution.clone(),
                crate::MODULE_TAG,
                encoded,
            )?;

            Self::deposit_event(Event::<T>::GrandpaKeyReplacementProposed {
                proposal_id,
                org: actual_org,
                institution,
                proposer: who,
                old_key,
                new_key,
            });
            Ok(())
        }

        // call_index = 1, 2 已废弃: execute_replace_grandpa_key /
        // cancel_failed_replace_grandpa_key 已统一到 VotingEngine 的
        // retry_passed_proposal / cancel_passed_proposal —— 前端必须直接调用
        // 投票引擎入口,业务 pallet 不再保留 wrapper extrinsic。
    }

    impl<T: Config> Pallet<T> {
        /// 中文注释：检查调用者是否为指定机构的内部管理员。
        fn is_internal_admin(org: u8, institution: T::AccountId, who: &T::AccountId) -> bool {
            <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                org,
                institution,
                who,
            )
        }

        /// 中文注释：检查 new_key 是否已被其他机构占用（通过反向索引 O(1) 判断）。
        fn is_key_used_by_other_institution(institution: T::AccountId, key: &[u8; 32]) -> bool {
            GrandpaKeyOwnerByKey::<T>::get(*key)
                .map(|owner| owner != institution)
                .unwrap_or(false)
        }

        /// 中文注释：尝试执行已通过的密钥替换提案，成功后调度 GRANDPA authority set 变更。
        pub(crate) fn try_execute_from_action(
            proposal_id: u64,
            action: GrandpaKeyReplacementAction<T::AccountId>,
        ) -> DispatchResult {
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let next_authorities = Self::validate_action(&action)?;

            pallet_grandpa::Pallet::<T>::schedule_change(
                next_authorities,
                T::GrandpaChangeDelay::get(),
                None,
            )?;

            // 中文注释：GRANDPA 接受调度后，链上“当前治理认可的目标 key”立即切到新值；
            // 真正 authority set 生效仍由 pallet-grandpa 在 delay 结束时完成。
            CurrentGrandpaKeys::<T>::insert(action.institution.clone(), action.new_key);
            GrandpaKeyOwnerByKey::<T>::remove(action.old_key);
            GrandpaKeyOwnerByKey::<T>::insert(action.new_key, action.institution.clone());

            Self::deposit_event(Event::<T>::GrandpaKeyReplaced {
                proposal_id,
                institution: action.institution,
                old_key: action.old_key,
                new_key: action.new_key,
            });
            Ok(())
        }

        /// 中文注释：校验提案可执行性——无 pending change、旧 key 存在、替换后无重复。
        pub(crate) fn validate_action(
            action: &GrandpaKeyReplacementAction<T::AccountId>,
        ) -> Result<Vec<(GrandpaAuthorityId, u64)>, Error<T>> {
            ensure!(
                pallet_grandpa::Pallet::<T>::pending_change().is_none(),
                Error::<T>::GrandpaChangePending
            );

            let old_authority = GrandpaAuthorityId::from(ed25519::Public::from_raw(action.old_key));
            let new_authority = GrandpaAuthorityId::from(ed25519::Public::from_raw(action.new_key));

            let mut found = false;
            // 中文注释：仅替换目标机构对应的一把 key，其余 authority 与权重原样保留。
            let next_authorities: Vec<(GrandpaAuthorityId, u64)> =
                pallet_grandpa::Pallet::<T>::grandpa_authorities()
                    .into_iter()
                    .map(|(authority, weight)| {
                        if authority == old_authority {
                            found = true;
                            (new_authority.clone(), weight)
                        } else {
                            (authority, weight)
                        }
                    })
                    .collect();

            ensure!(found, Error::<T>::OldAuthorityNotFound);
            let mut uniq = sp_std::collections::btree_set::BTreeSet::new();
            ensure!(
                next_authorities
                    .iter()
                    .all(|(authority, _)| uniq.insert(authority.encode())),
                Error::<T>::NewKeyAlreadyUsed
            );

            Ok(next_authorities)
        }
    }
}

// ──── 投票终态回调:把已通过的 GRANDPA 密钥替换提案落地到链上 ────
//
// 投票统一由投票引擎承担,提案通过(或否决)经
// [`votingengine::InternalVoteResultCallback`] 广播回来。
// 本 Executor 按 `MODULE_TAG` 前缀认领本模块的提案。
//
// 失败语义:自动执行失败(如 GRANDPA pending change 未清理)时发
// `GrandpaKeyExecutionFailed` 事件,提案状态保留 PASSED,任何签名管理员可以通过
// `execute_replace_grandpa_key` 手动重试,或用 `cancel_failed_replace_grandpa_key`
// 清理确定无法执行的提案。
pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, sp_runtime::DispatchError> {
        let raw = match votingengine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(ProposalExecutionOutcome::Ignored),
        };
        if !approved {
            return Ok(ProposalExecutionOutcome::Executed);
        }
        let action = GrandpaKeyReplacementAction::<T::AccountId>::decode(
            &mut &raw[crate::MODULE_TAG.len()..],
        )
        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;

        match pallet::Pallet::<T>::validate_action(&action) {
            Err(pallet::Error::<T>::GrandpaChangePending) => {
                pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::GrandpaKeyExecutionFailed {
                    proposal_id,
                });
                return Ok(ProposalExecutionOutcome::RetryableFailed);
            }
            Err(_) => {
                pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::GrandpaKeyExecutionFailed {
                    proposal_id,
                });
                return Ok(ProposalExecutionOutcome::FatalFailed);
            }
            Ok(_) => {}
        }

        match pallet::Pallet::<T>::try_execute_from_action(proposal_id, action) {
            Ok(()) => Ok(ProposalExecutionOutcome::Executed),
            Err(_) => {
                pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::GrandpaKeyExecutionFailed {
                    proposal_id,
                });
                Ok(ProposalExecutionOutcome::RetryableFailed)
            }
        }
    }

    fn can_cancel_passed_proposal(
        proposal_id: u64,
    ) -> Result<ProposalCancelDecision, sp_runtime::DispatchError> {
        let raw = match votingengine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(ProposalCancelDecision::Ignored),
        };
        let action = GrandpaKeyReplacementAction::<T::AccountId>::decode(
            &mut &raw[crate::MODULE_TAG.len()..],
        )
        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
        // 中文注释：只允许取消确定不可执行的 GRANDPA 替换；pending change 属于可恢复失败。
        match pallet::Pallet::<T>::validate_action(&action) {
            Ok(_) => Err(pallet::Error::<T>::ProposalStillExecutable.into()),
            Err(pallet::Error::<T>::GrandpaChangePending) => {
                Err(pallet::Error::<T>::GrandpaChangePending.into())
            }
            Err(_) => {
                pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::FailedProposalCancelled {
                    proposal_id,
                    institution: action.institution,
                });
                Ok(ProposalCancelDecision::Allow)
            }
        }
    }
}

#[cfg(test)]
mod tests;
