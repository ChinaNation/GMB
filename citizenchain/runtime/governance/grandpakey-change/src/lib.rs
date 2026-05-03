//! # GRANDPA 密钥治理模块 (grandpakey-change)
//!
//! 本模块将"机构 GRANDPA 公钥替换"包装成受治理约束的链上流程：
//! - 仅国储会（NRC）与省储会（PRC）可发起密钥替换提案。
//! - 仅目标机构内部管理员可参与提案/投票/执行/清理。
//! - 借助 `voting-engine` 内部投票达成通过后，调用 `pallet-grandpa::schedule_change` 变更 authority set。
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
use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use scale_info::TypeInfo;
use sp_consensus_grandpa::AuthorityId as GrandpaAuthorityId;
use sp_core::ed25519;
use voting_engine::{
    internal_vote::{ORG_NRC, ORG_PRC},
    InstitutionPalletId, InternalVoteResultCallback, ProposalCancelDecision,
    ProposalExecutionOutcome, STATUS_PASSED,
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
pub struct GrandpaKeyReplacementAction {
    pub institution: InstitutionPalletId,
    pub old_key: [u8; 32],
    pub new_key: [u8; 32],
}

/// 中文注释：获取国储会（NRC）的机构 pallet ID。
fn nrc_pallet_id_bytes() -> Option<InstitutionPalletId> {
    CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
}

/// 中文注释：判断机构属于 NRC 还是 PRC，不属于任何一类则返回 None。
/// PRB（省储行）不参与 GRANDPA 共识出块，故不纳入密钥治理范围。
fn institution_org(institution: InstitutionPalletId) -> Option<u8> {
    if Some(institution) == nrc_pallet_id_bytes() {
        return Some(ORG_NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
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
    use voting_engine::{InternalAdminProvider, InternalVoteEngine};

    #[pallet::config]
    pub trait Config:
        frame_system::Config + voting_engine::Config + pallet_grandpa::Config
    {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type GrandpaChangeDelay: Get<BlockNumberFor<Self>>;

        /// 中文注释：内部投票引擎（返回真实 proposal_id，避免猜测 next_proposal_id）。
        type InternalVoteEngine: voting_engine::InternalVoteEngine<Self::AccountId>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 中文注释：机构当前 GRANDPA 公钥，治理认可的目标 key（真正生效由 pallet-grandpa delay 控制）。
    #[pallet::storage]
    #[pallet::getter(fn current_grandpa_key)]
    pub type CurrentGrandpaKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, [u8; 32], OptionQuery>;

    /// 中文注释：公钥到机构的反向索引，O(1) 判断 new_key 是否已被其他机构占用。
    #[pallet::storage]
    #[pallet::getter(fn key_owner)]
    pub type GrandpaKeyOwnerByKey<T: Config> =
        StorageMap<_, Blake2_128Concat, [u8; 32], InstitutionPalletId, OptionQuery>;

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
                let Some(institution) = reserve_pallet_id_to_bytes(node.shenfen_id) else {
                    continue;
                };
                assert!(
                    !GrandpaKeyOwnerByKey::<T>::contains_key(node.grandpa_key),
                    "duplicated initial grandpa key in CHINA_CB"
                );
                CurrentGrandpaKeys::<T>::insert(institution, node.grandpa_key);
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
            institution: InstitutionPalletId,
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
            institution: InstitutionPalletId,
            old_key: [u8; 32],
            new_key: [u8; 32],
        },
        /// 已通过但不可执行的提案被取消
        FailedProposalCancelled {
            proposal_id: u64,
            institution: InstitutionPalletId,
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
            institution: InstitutionPalletId,
            new_key: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(new_key != [0u8; 32], Error::<T>::NewKeyIsZero);
            let point = CompressedEdwardsY(new_key)
                .decompress()
                .ok_or(Error::<T>::InvalidEd25519Key)?;
            // 中文注释：仅”能解压”为曲线点还不够，small-order 弱公钥可能导致 GRANDPA 签名安全性失真。
            ensure!(!point.is_small_order(), Error::<T>::InvalidEd25519Key);

            let actual_org = institution_org(institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(actual_org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let old_key = CurrentGrandpaKeys::<T>::get(institution)
                .ok_or(Error::<T>::CurrentGrandpaKeyNotFound)?;
            ensure!(new_key != old_key, Error::<T>::NewKeyUnchanged);
            ensure!(
                !Self::is_key_used_by_other_institution(institution, &new_key),
                Error::<T>::NewKeyAlreadyUsed
            );

            let action = GrandpaKeyReplacementAction {
                institution,
                old_key,
                new_key,
            };

            let mut encoded = sp_std::vec::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&action.encode());
            let proposal_id = T::InternalVoteEngine::create_internal_proposal_with_data(
                who.clone(),
                actual_org,
                institution,
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
        fn is_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            <T as voting_engine::Config>::InternalAdminProvider::is_internal_admin(
                org,
                institution,
                who,
            )
        }

        /// 中文注释：检查 new_key 是否已被其他机构占用（通过反向索引 O(1) 判断）。
        fn is_key_used_by_other_institution(
            institution: InstitutionPalletId,
            key: &[u8; 32],
        ) -> bool {
            GrandpaKeyOwnerByKey::<T>::get(*key)
                .map(|owner| owner != institution)
                .unwrap_or(false)
        }

        /// 中文注释：尝试执行已通过的密钥替换提案，成功后调度 GRANDPA authority set 变更。
        pub(crate) fn try_execute_from_action(
            proposal_id: u64,
            action: GrandpaKeyReplacementAction,
        ) -> DispatchResult {
            let proposal = voting_engine::Pallet::<T>::proposals(proposal_id)
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
            CurrentGrandpaKeys::<T>::insert(action.institution, action.new_key);
            GrandpaKeyOwnerByKey::<T>::remove(action.old_key);
            GrandpaKeyOwnerByKey::<T>::insert(action.new_key, action.institution);

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
            action: &GrandpaKeyReplacementAction,
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
// Phase 2 整改后业务模块不再自行处理投票,提案通过(或否决)由投票引擎
// 通过 [`voting_engine::InternalVoteResultCallback`] 广播回来。
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
        let raw = match voting_engine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(ProposalExecutionOutcome::Ignored),
        };
        if !approved {
            return Ok(ProposalExecutionOutcome::Executed);
        }
        let action = GrandpaKeyReplacementAction::decode(&mut &raw[crate::MODULE_TAG.len()..])
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
        let raw = match voting_engine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(ProposalCancelDecision::Ignored),
        };
        let action = GrandpaKeyReplacementAction::decode(&mut &raw[crate::MODULE_TAG.len()..])
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
mod tests {
    use super::*;
    use frame_support::{
        assert_noop, assert_ok, derive_impl, parameter_types,
        traits::{ConstU32, Hooks},
    };
    use frame_system as system;
    use primitives::china::china_cb::CHINA_CB;
    use sp_core::{Pair, Void};
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
    use voting_engine::STATUS_EXECUTION_FAILED;

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
        pub type Grandpa = pallet_grandpa;

        #[runtime::pallet_index(2)]
        pub type VotingEngine = voting_engine;

        #[runtime::pallet_index(3)]
        pub type GrandpaKeyChange = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    parameter_types! {
        pub const MaxGrandpaAuthorities: u32 = 64;
        pub const MaxGrandpaNominators: u32 = 0;
        pub const MaxSetIdSessionEntries: u64 = 16;
        pub const GrandpaChangeDelay: u64 = 30;
    }

    impl pallet_grandpa::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type WeightInfo = ();
        type MaxAuthorities = MaxGrandpaAuthorities;
        type MaxNominators = MaxGrandpaNominators;
        type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
        type KeyOwnerProof = Void;
        type EquivocationReportSystem = ();
    }

    pub struct TestSfidEligibility;
    pub struct TestPopulationSnapshotVerifier;
    pub struct TestInternalAdminProvider;

    impl voting_engine::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
        for TestSfidEligibility
    {
        fn is_eligible(
            _binding_id: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
        ) -> bool {
            false
        }

        fn verify_and_consume_vote_credential(
            _binding_id: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
            _proposal_id: u64,
            _nonce: &[u8],
            _signature: &[u8],
            _province: &[u8],
            _signer_admin_pubkey: &[u8; 32],
        ) -> bool {
            false
        }

        fn cleanup_vote_credentials(_proposal_id: u64) {}
    }

    impl
        voting_engine::PopulationSnapshotVerifier<
            AccountId32,
            voting_engine::pallet::VoteNonceOf<Test>,
            voting_engine::pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            _eligible_total: u64,
            _nonce: &voting_engine::pallet::VoteNonceOf<Test>,
            _signature: &voting_engine::pallet::VoteSignatureOf<Test>,
            _province: &[u8],
            _signer_admin_pubkey: &[u8; 32],
        ) -> bool {
            true
        }
    }

    impl voting_engine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            let mut who_raw = [0u8; 32];
            who_raw.copy_from_slice(who.as_ref());
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|node| reserve_pallet_id_to_bytes(node.shenfen_id) == Some(institution))
                    .map(|node| node.duoqian_admins.iter().any(|admin| *admin == who_raw))
                    .unwrap_or(false),
                _ => false,
            }
        }

        fn get_admin_list(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<sp_std::vec::Vec<AccountId32>> {
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|node| reserve_pallet_id_to_bytes(node.shenfen_id) == Some(institution))
                    .map(|node| {
                        node.duoqian_admins
                            .iter()
                            .map(|raw| AccountId32::new(*raw))
                            .collect()
                    }),
                _ => None,
            }
        }
    }

    pub struct TestTimeProvider;
    pub struct TestInternalThresholdProvider;
    impl voting_engine::InternalThresholdProvider for TestInternalThresholdProvider {
        fn pass_threshold(org: u8, _institution: InstitutionPalletId) -> Option<u32> {
            voting_engine::internal_vote::fixed_governance_pass_threshold(org)
        }
    }

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
        type MaxInternalProposalMutexBindings = ConstU32<256>;
        type MaxActiveProposals = ConstU32<10>;
        type MaxCleanupStepsPerBlock = ConstU32<8>;
        type CleanupKeysPerStep = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<256>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type MaxModuleTagLen = ConstU32<32>;
        type MaxManualExecutionAttempts = ConstU32<3>;
        type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
        type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
        type MaxCleanupQueueBucketLimit = ConstU32<50>;
        type MaxCleanupScheduleOffset = ConstU32<100>;
        type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalThresholdProvider = TestInternalThresholdProvider;
        type InternalAdminCountProvider = ();
        type MaxAdminsPerInstitution = ConstU32<32>;
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type GrandpaChangeDelay = GrandpaChangeDelay;
        type InternalVoteEngine = voting_engine::Pallet<Test>;
        type WeightInfo = ();
    }

    fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
        vec![
            (
                GrandpaAuthorityId::from(ed25519::Public::from_raw(CHINA_CB[0].grandpa_key)),
                1,
            ),
            (
                GrandpaAuthorityId::from(ed25519::Public::from_raw(CHINA_CB[1].grandpa_key)),
                1,
            ),
            (
                GrandpaAuthorityId::from(ed25519::Public::from_raw(CHINA_CB[2].grandpa_key)),
                1,
            ),
        ]
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");
        pallet_grandpa::GenesisConfig::<Test> {
            authorities: grandpa_authorities(),
            _config: Default::default(),
        }
        .assimilate_storage(&mut storage)
        .expect("grandpa genesis should assimilate");
        GenesisConfig::<Test>::default()
            .assimilate_storage(&mut storage)
            .expect("grandpakey-change genesis should assimilate");

        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
        });
        ext
    }

    fn cb_admin(node_index: usize, admin_index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[node_index].duoqian_admins[admin_index])
    }

    fn cb_pallet_id(node_index: usize) -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[node_index].shenfen_id)
            .expect("institution should map to pallet id")
    }

    fn prc_admin(index: usize) -> AccountId32 {
        cb_admin(1, index)
    }

    fn prc_pallet_id() -> InstitutionPalletId {
        cb_pallet_id(1)
    }

    fn valid_public_key(seed: u8) -> [u8; 32] {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[0] = seed;
        ed25519::Pair::from_seed(&seed_bytes).public().0
    }

    fn identity_public_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        key[0] = 1;
        key
    }

    fn authority_id_from_key(key: [u8; 32]) -> GrandpaAuthorityId {
        GrandpaAuthorityId::from(ed25519::Public::from_raw(key))
    }

    fn pass_prc_proposal(node_index: usize, proposal_id: u64) {
        for admin_index in 0..6 {
            assert_ok!(cast_vote(
                cb_admin(node_index, admin_index),
                proposal_id,
                true
            ));
        }
    }

    fn finalize_grandpa_at(block: u64) {
        System::set_block_number(block);
        <Grandpa as Hooks<u64>>::on_finalize(block);
    }

    /// 获取最近一次 create_internal_proposal 分配的 proposal_id。
    fn last_proposal_id() -> u64 {
        voting_engine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
    }

    /// 测试辅助:走投票引擎公开 `internal_vote` extrinsic 投票(Phase 2 统一入口)。
    fn cast_vote(who: AccountId32, proposal_id: u64, approve: bool) -> DispatchResult {
        voting_engine::Pallet::<Test>::internal_vote(
            RuntimeOrigin::signed(who),
            proposal_id,
            approve,
        )
    }

    #[test]
    fn weak_small_order_new_key_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                GrandpaKeyChange::propose_replace_grandpa_key(
                    RuntimeOrigin::signed(prc_admin(0)),
                    prc_pallet_id(),
                    identity_public_key()
                ),
                Error::<Test>::InvalidEd25519Key
            );
        });
    }

    #[test]
    fn passed_proposal_executes_and_cleans_up_state() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_key = CurrentGrandpaKeys::<Test>::get(institution)
                .expect("institution should have an initial key");
            let new_key = valid_public_key(31);

            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();

            pass_prc_proposal(1, pid);

            let pending_change = Grandpa::pending_change().expect("change should be scheduled");
            assert_eq!(pending_change.scheduled_at, 1);
            assert_eq!(pending_change.delay, GrandpaChangeDelay::get());
            assert!(pending_change
                .next_authorities
                .iter()
                .any(|(authority, _)| *authority == authority_id_from_key(new_key)));

            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(new_key));
            assert!(GrandpaKeyOwnerByKey::<Test>::get(old_key).is_none());
            assert_eq!(
                GrandpaKeyOwnerByKey::<Test>::get(new_key),
                Some(institution)
            );
            assert!(System::events().iter().any(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyReplaced {
                        proposal_id,
                        institution: inst,
                        old_key: replaced_old_key,
                        new_key: replaced_new_key,
                    }) if *proposal_id == pid
                        && *inst == institution
                        && *replaced_old_key == old_key
                        && *replaced_new_key == new_key
                )
            }));
        });
    }

    #[test]
    fn passed_proposal_can_be_manually_executed_after_pending_change_clears() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_key = CurrentGrandpaKeys::<Test>::get(institution)
                .expect("institution should have an initial key");
            let new_key = valid_public_key(41);

            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();
            assert_ok!(Grandpa::schedule_change(
                grandpa_authorities(),
                GrandpaChangeDelay::get(),
                None,
            ));

            pass_prc_proposal(1, pid);

            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("passed proposal should remain for retries")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(old_key));
            assert!(voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some());
            assert!(System::events().iter().any(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyExecutionFailed {
                        proposal_id
                    }) if *proposal_id == pid
                )
            }));

            finalize_grandpa_at(1 + GrandpaChangeDelay::get());
            assert!(Grandpa::pending_change().is_none());

            assert_ok!(VotingEngine::retry_passed_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                pid,
            ));

            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(new_key));
            assert!(GrandpaKeyOwnerByKey::<Test>::get(old_key).is_none());
            assert_eq!(
                GrandpaKeyOwnerByKey::<Test>::get(new_key),
                Some(institution)
            );
            assert!(Grandpa::pending_change().is_some());
        });
    }

    #[test]
    fn cancel_failed_replace_grandpa_key_cleans_up_passed_but_invalid_proposal() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_key = CurrentGrandpaKeys::<Test>::get(institution)
                .expect("institution should have an initial key");
            let new_key = valid_public_key(51);
            let replacement_authority = valid_public_key(52);

            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();
            assert_ok!(Grandpa::schedule_change(
                vec![
                    (authority_id_from_key(CHINA_CB[0].grandpa_key), 1),
                    (authority_id_from_key(replacement_authority), 1),
                ],
                GrandpaChangeDelay::get(),
                None,
            ));

            pass_prc_proposal(1, pid);

            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("passed proposal should remain for cleanup")
                    .status,
                STATUS_PASSED
            );
            finalize_grandpa_at(1 + GrandpaChangeDelay::get());

            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(old_key));
            assert_eq!(
                Grandpa::grandpa_authorities(),
                vec![
                    (authority_id_from_key(CHINA_CB[0].grandpa_key), 1),
                    (authority_id_from_key(replacement_authority), 1),
                ]
            );

            assert_ok!(VotingEngine::cancel_passed_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                pid,
                Default::default(),
            ));
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("cancelled proposal should remain until cleanup")
                    .status,
                STATUS_EXECUTION_FAILED
            );

            assert!(System::events().iter().any(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::GrandpaKeyChange(Event::<Test>::FailedProposalCancelled {
                        proposal_id,
                        institution: inst,
                    }) if *proposal_id == pid && *inst == institution
                )
            }));
        });
    }

    #[test]
    fn cancel_failed_replace_grandpa_key_rejects_temporarily_blocked_proposal() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_key = CurrentGrandpaKeys::<Test>::get(institution)
                .expect("institution should have an initial key");
            let new_key = valid_public_key(71);

            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();
            assert_ok!(Grandpa::schedule_change(
                grandpa_authorities(),
                GrandpaChangeDelay::get(),
                None,
            ));

            pass_prc_proposal(1, pid);

            assert_noop!(
                VotingEngine::cancel_passed_proposal(
                    RuntimeOrigin::signed(prc_admin(0)),
                    pid,
                    Default::default(),
                ),
                Error::<Test>::GrandpaChangePending
            );

            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(old_key));
            assert!(voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some());
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("passed proposal should remain active")
                    .status,
                STATUS_PASSED
            );
        });
    }

    #[test]
    fn finalized_vote_fatal_fails_when_old_authority_disappeared() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_key = CurrentGrandpaKeys::<Test>::get(institution)
                .expect("institution should have an initial key");
            let new_key = valid_public_key(72);
            let replacement_authority = valid_public_key(73);

            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();

            // 中文注释：模拟其他治理动作已经把提案绑定的旧 authority 替换掉。
            assert_ok!(Grandpa::schedule_change(
                vec![
                    (authority_id_from_key(CHINA_CB[0].grandpa_key), 1),
                    (authority_id_from_key(replacement_authority), 1),
                    (authority_id_from_key(CHINA_CB[2].grandpa_key), 1),
                ],
                GrandpaChangeDelay::get(),
                None,
            ));
            finalize_grandpa_at(1 + GrandpaChangeDelay::get());
            assert!(Grandpa::pending_change().is_none());

            pass_prc_proposal(1, pid);

            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("fatal failed proposal should remain until cleanup")
                    .status,
                STATUS_EXECUTION_FAILED
            );
            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(old_key));
            assert!(GrandpaKeyOwnerByKey::<Test>::get(new_key).is_none());
            assert!(System::events().iter().any(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyExecutionFailed {
                        proposal_id
                    }) if *proposal_id == pid
                )
            }));
        });
    }

    #[test]
    fn finalized_vote_fatal_fails_when_new_key_collides_after_first_execution() {
        new_test_ext().execute_with(|| {
            let first_institution = cb_pallet_id(1);
            let second_institution = cb_pallet_id(2);
            let first_old_key = CurrentGrandpaKeys::<Test>::get(first_institution)
                .expect("first institution should have an initial key");
            let second_old_key = CurrentGrandpaKeys::<Test>::get(second_institution)
                .expect("second institution should have an initial key");
            let shared_new_key = valid_public_key(74);

            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(cb_admin(1, 0)),
                first_institution,
                shared_new_key,
            ));
            let first_pid = last_proposal_id();
            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(cb_admin(2, 0)),
                second_institution,
                shared_new_key,
            ));
            let second_pid = last_proposal_id();

            pass_prc_proposal(1, first_pid);
            assert_eq!(
                CurrentGrandpaKeys::<Test>::get(first_institution),
                Some(shared_new_key)
            );
            assert_eq!(
                GrandpaKeyOwnerByKey::<Test>::get(shared_new_key),
                Some(first_institution)
            );
            finalize_grandpa_at(1 + GrandpaChangeDelay::get());
            assert!(Grandpa::pending_change().is_none());

            pass_prc_proposal(2, second_pid);

            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(second_pid)
                    .expect("colliding proposal should remain until cleanup")
                    .status,
                STATUS_EXECUTION_FAILED
            );
            assert_eq!(
                CurrentGrandpaKeys::<Test>::get(second_institution),
                Some(second_old_key)
            );
            assert_eq!(
                GrandpaKeyOwnerByKey::<Test>::get(shared_new_key),
                Some(first_institution)
            );
            assert!(GrandpaKeyOwnerByKey::<Test>::get(first_old_key).is_none());
            assert!(System::events().iter().any(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyExecutionFailed {
                        proposal_id
                    }) if *proposal_id == second_pid
                )
            }));
        });
    }

    // ========================================================================
    // 补充的错误路径和边界测试
    // ========================================================================

    #[test]
    fn propose_rejects_zero_key() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                GrandpaKeyChange::propose_replace_grandpa_key(
                    RuntimeOrigin::signed(prc_admin(0)),
                    prc_pallet_id(),
                    [0u8; 32],
                ),
                Error::<Test>::NewKeyIsZero
            );
        });
    }

    #[test]
    fn propose_rejects_unchanged_key() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let current_key =
                CurrentGrandpaKeys::<Test>::get(institution).expect("institution should have key");
            assert_noop!(
                GrandpaKeyChange::propose_replace_grandpa_key(
                    RuntimeOrigin::signed(prc_admin(0)),
                    institution,
                    current_key,
                ),
                Error::<Test>::NewKeyUnchanged
            );
        });
    }

    #[test]
    fn propose_rejects_key_owned_by_other_institution() {
        new_test_ext().execute_with(|| {
            // CHINA_CB[0] 是国储会的 key，用它作为省储会的 new_key 应失败
            let nrc_key = CHINA_CB[0].grandpa_key;
            assert_noop!(
                GrandpaKeyChange::propose_replace_grandpa_key(
                    RuntimeOrigin::signed(prc_admin(0)),
                    prc_pallet_id(),
                    nrc_key,
                ),
                Error::<Test>::NewKeyAlreadyUsed
            );
        });
    }

    #[test]
    fn propose_rejects_unauthorized_admin() {
        new_test_ext().execute_with(|| {
            // 使用一个不在 duoqian_admins 中的随机账户
            let outsider = AccountId32::new([99u8; 32]);
            assert_noop!(
                GrandpaKeyChange::propose_replace_grandpa_key(
                    RuntimeOrigin::signed(outsider),
                    prc_pallet_id(),
                    valid_public_key(80),
                ),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn propose_rejects_invalid_institution() {
        new_test_ext().execute_with(|| {
            let fake_institution: InstitutionPalletId = [99u8; 48];
            assert_noop!(
                GrandpaKeyChange::propose_replace_grandpa_key(
                    RuntimeOrigin::signed(prc_admin(0)),
                    fake_institution,
                    valid_public_key(81),
                ),
                Error::<Test>::InvalidInstitution
            );
        });
    }

    #[test]
    fn execute_rejects_non_passed_proposal() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let new_key = valid_public_key(82);
            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();
            // 不投票，直接尝试执行
            assert_noop!(
                VotingEngine::retry_passed_proposal(
                    RuntimeOrigin::signed(prc_admin(0)),
                    pid,
                ),
                voting_engine::pallet::Error::<Test>::ProposalNotRetryable
            );
        });
    }

    #[test]
    fn cancel_rejects_still_executable_proposal() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let new_key = valid_public_key(83);

            // 先制造 pending change 阻塞
            assert_ok!(Grandpa::schedule_change(
                grandpa_authorities(),
                GrandpaChangeDelay::get(),
                None,
            ));

            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();

            // 投票通过，自动执行因 pending change 失败
            pass_prc_proposal(1, pid);
            assert!(System::events().iter().any(|r| matches!(
                &r.event,
                RuntimeEvent::GrandpaKeyChange(Event::<Test>::GrandpaKeyExecutionFailed { .. })
            )));

            // 清除 pending change
            finalize_grandpa_at(1 + GrandpaChangeDelay::get());
            assert!(Grandpa::pending_change().is_none());

            // 提案仍可执行，不允许取消
            assert_noop!(
                VotingEngine::cancel_passed_proposal(
                    RuntimeOrigin::signed(prc_admin(0)),
                    pid,
                    Default::default(),
                ),
                Error::<Test>::ProposalStillExecutable
            );
        });
    }

    #[test]
    fn vote_rejects_unauthorized_admin() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let new_key = valid_public_key(85);
            assert_ok!(GrandpaKeyChange::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();
            let outsider = AccountId32::new([98u8; 32]);
            assert_noop!(
                cast_vote(outsider, pid, true),
                voting_engine::pallet::Error::<Test>::NoPermission
            );
        });
    }
}
