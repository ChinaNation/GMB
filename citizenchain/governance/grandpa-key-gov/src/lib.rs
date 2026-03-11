#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use curve25519_dalek::edwards::CompressedEdwardsY;
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{GetStorageVersion, StorageVersion},
    weights::Weight,
    Blake2_128Concat, Twox64Concat,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_consensus_grandpa::AuthorityId as GrandpaAuthorityId;
use sp_core::ed25519;
use sp_runtime::traits::{One, Saturating, Zero};

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use voting_engine_system::{
    internal_vote::{ORG_NRC, ORG_PRC},
    InstitutionPalletId, STATUS_PASSED, STATUS_REJECTED,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
pub struct GrandpaKeyReplacementAction {
    pub institution: InstitutionPalletId,
    pub old_key: [u8; 32],
    pub new_key: [u8; 32],
}

fn nrc_pallet_id_bytes() -> Option<InstitutionPalletId> {
    CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
}

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
    use sp_runtime::DispatchError;
    use sp_std::vec::Vec;
    use voting_engine_system::{InternalAdminProvider, InternalVoteEngine};

    #[pallet::config]
    pub trait Config:
        frame_system::Config + voting_engine_system::Config + pallet_grandpa::Config
    {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type StaleProposalLifetime: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type GrandpaChangeDelay: Get<BlockNumberFor<Self>>;

        /// 中文注释：内部投票引擎（返回真实 proposal_id，避免猜测 next_proposal_id）。
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn proposal_action)]
    pub type ProposalActions<T: Config> =
        StorageMap<_, Twox64Concat, u64, GrandpaKeyReplacementAction, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_grandpa_key)]
    pub type CurrentGrandpaKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, [u8; 32], OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn key_owner)]
    pub type GrandpaKeyOwnerByKey<T: Config> =
        StorageMap<_, Blake2_128Concat, [u8; 32], InstitutionPalletId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_created_at)]
    pub type ProposalCreatedAt<T: Config> =
        StorageMap<_, Twox64Concat, u64, BlockNumberFor<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn active_proposal_by_institution)]
    pub type ActiveProposalByInstitution<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, u64, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_proposal_by_new_key)]
    pub type PendingProposalByNewKey<T: Config> =
        StorageMap<_, Blake2_128Concat, [u8; 32], u64, OptionQuery>;

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
        #[cfg(feature = "std")]
        fn integrity_test() {
            assert!(
                !T::StaleProposalLifetime::get().is_zero(),
                "StaleProposalLifetime must be > 0"
            );
        }

        fn on_runtime_upgrade() -> Weight {
            let onchain = Pallet::<T>::on_chain_storage_version();
            if onchain < 2 {
                let mut reads: u64 = 1;
                let mut writes: u64 = 1;
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
        fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
            let state = (
                Pallet::<T>::on_chain_storage_version(),
                CurrentGrandpaKeys::<T>::iter().count() as u64,
                GrandpaKeyOwnerByKey::<T>::iter().count() as u64,
                PendingProposalByNewKey::<T>::iter().count() as u64,
            );
            Ok(state.encode())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            let (pre_version, pre_current_count, _pre_owner_count, _pre_pending_count): (
                StorageVersion,
                u64,
                u64,
                u64,
            ) = Decode::decode(&mut &state[..]).map_err(|_| "decode pre_upgrade state failed")?;

            if pre_version < 2 {
                if (GrandpaKeyOwnerByKey::<T>::iter().count() as u64) < pre_current_count {
                    return Err("owner map should be backfilled on migration".into());
                }
            }

            for (key, proposal_id) in PendingProposalByNewKey::<T>::iter() {
                let action = ProposalActions::<T>::get(proposal_id)
                    .ok_or("pending key index points to missing proposal")?;
                if action.new_key != key {
                    return Err("pending key index mismatches proposal new_key".into());
                }
            }

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
        /// 过期且未执行的提案被清理
        StaleProposalCancelled {
            proposal_id: u64,
            institution: InstitutionPalletId,
        },
        /// 已通过但不可执行的提案被取消
        FailedProposalCancelled {
            proposal_id: u64,
            institution: InstitutionPalletId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidInstitution,
        InstitutionOrgMismatch,
        UnsupportedOrg,
        UnauthorizedAdmin,
        ProposalActionNotFound,
        ProposalNotPassed,
        ActiveProposalExists,
        ProposalNotStale,
        PassedProposalCannotBeCancelled,
        CurrentGrandpaKeyNotFound,
        NewKeyIsZero,
        InvalidEd25519Key,
        NewKeyUnchanged,
        NewKeyAlreadyUsed,
        NewKeyPendingInOtherProposal,
        OldAuthorityNotFound,
        GrandpaChangePending,
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
            // 中文注释：仅“能解压”为曲线点还不够，small-order 弱公钥可能导致 GRANDPA 签名安全性失真。
            ensure!(!point.is_small_order(), Error::<T>::InvalidEd25519Key);

            let actual_org = institution_org(institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                matches!(actual_org, ORG_NRC | ORG_PRC),
                Error::<T>::UnsupportedOrg
            );
            ensure!(
                Self::is_internal_admin(actual_org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            let stale_proposal = Self::ensure_no_active_proposal(institution)?;

            if let Some(pending_id) = PendingProposalByNewKey::<T>::get(new_key) {
                if let Some(pending_action) = ProposalActions::<T>::get(pending_id) {
                    // 中文注释：同一 new_key 只能被一个活跃提案占用，防止并发冲突。
                    if pending_action.institution != institution {
                        return Err(Error::<T>::NewKeyPendingInOtherProposal.into());
                    }
                } else {
                    PendingProposalByNewKey::<T>::remove(new_key);
                }
            }

            let old_key = CurrentGrandpaKeys::<T>::get(institution)
                .ok_or(Error::<T>::CurrentGrandpaKeyNotFound)?;
            ensure!(new_key != old_key, Error::<T>::NewKeyUnchanged);
            ensure!(
                !Self::is_key_used_by_other_institution(institution, &new_key),
                Error::<T>::NewKeyAlreadyUsed
            );

            let proposal_id = T::InternalVoteEngine::create_internal_proposal(
                who.clone(),
                actual_org,
                institution,
            )?;

            if let Some(stale_id) = stale_proposal {
                // 中文注释：先成功拿到新 proposal_id，再清理旧脏提案，避免未授权调用借机删除历史状态。
                if stale_id != proposal_id {
                    Self::cleanup_inactive_proposal(institution, stale_id);
                }
            }

            ProposalActions::<T>::insert(
                proposal_id,
                GrandpaKeyReplacementAction {
                    institution,
                    old_key,
                    new_key,
                },
            );
            ProposalCreatedAt::<T>::insert(proposal_id, frame_system::Pallet::<T>::block_number());
            ActiveProposalByInstitution::<T>::insert(institution, proposal_id);

            Self::deposit_event(Event::<T>::GrandpaKeyReplacementProposed {
                proposal_id,
                org: actual_org,
                institution,
                proposer: who,
                old_key,
                new_key,
            });
            PendingProposalByNewKey::<T>::insert(new_key, proposal_id);
            Ok(())
        }

        /// 对“GRANDPA 密钥替换”提案投票，达到阈值通过后自动执行替换。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::vote_replace_grandpa_key())]
        pub fn vote_replace_grandpa_key(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            T::InternalVoteEngine::cast_internal_vote(who.clone(), proposal_id, approve)?;

            Self::deposit_event(Event::<T>::GrandpaKeyVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                if proposal.status == STATUS_PASSED {
                    if Self::try_execute_from_action(proposal_id, action).is_err() {
                        Self::deposit_event(Event::<T>::GrandpaKeyExecutionFailed { proposal_id });
                    }
                } else if proposal.status == STATUS_REJECTED {
                    Self::cleanup_inactive_proposal(action.institution, proposal_id);
                }
            }

            Ok(())
        }

        /// 手动执行已通过的密钥替换提案。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::execute_replace_grandpa_key())]
        pub fn execute_replace_grandpa_key(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            Self::try_execute(proposal_id)
        }

        /// 清理已过期且未执行的提案。
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_stale_replace_grandpa_key())]
        pub fn cancel_stale_replace_grandpa_key(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            let created_at = ProposalCreatedAt::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;

            let is_passed = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|proposal| proposal.status == STATUS_PASSED)
                .unwrap_or(false);
            ensure!(!is_passed, Error::<T>::PassedProposalCannotBeCancelled);

            let now = frame_system::Pallet::<T>::block_number();
            let stale_at = created_at.saturating_add(Self::effective_stale_lifetime());
            ensure!(now >= stale_at, Error::<T>::ProposalNotStale);

            Self::cleanup_inactive_proposal(action.institution, proposal_id);

            Self::deposit_event(Event::<T>::StaleProposalCancelled {
                proposal_id,
                institution: action.institution,
            });
            Ok(())
        }

        /// 清理“已通过但确定无法执行”的提案，避免机构长期被 ActiveProposal 锁死。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_failed_replace_grandpa_key())]
        pub fn cancel_failed_replace_grandpa_key(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );
            // 中文注释：这里只允许清理“确定已经执行不了”的通过提案；
            // 若只是 GRANDPA 仍有 pending change，则属于暂时不可执行，应该等待后重试。
            match Self::validate_action(&action) {
                Ok(_) => return Err(Error::<T>::ProposalStillExecutable.into()),
                Err(Error::<T>::GrandpaChangePending) => {
                    return Err(Error::<T>::GrandpaChangePending.into())
                }
                Err(_) => {}
            }

            Self::cleanup_inactive_proposal(action.institution, proposal_id);
            Self::deposit_event(Event::<T>::FailedProposalCancelled {
                proposal_id,
                institution: action.institution,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                org,
                institution,
                who,
            )
        }

        fn effective_stale_lifetime() -> BlockNumberFor<T> {
            let configured = T::StaleProposalLifetime::get();
            if configured.is_zero() {
                One::one()
            } else {
                configured
            }
        }

        fn ensure_no_active_proposal(
            institution: InstitutionPalletId,
        ) -> Result<Option<u64>, DispatchError> {
            let Some(existing_id) = ActiveProposalByInstitution::<T>::get(institution) else {
                return Ok(None);
            };

            let Some(_action) = ProposalActions::<T>::get(existing_id) else {
                // 中文注释：机构活跃索引指向缺失动作，视为悬挂脏数据，可在新提案创建后顺手清掉。
                return Ok(Some(existing_id));
            };

            let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(existing_id) else {
                return Ok(Some(existing_id));
            };

            if proposal.status == STATUS_REJECTED {
                return Ok(Some(existing_id));
            }

            if proposal.status != STATUS_PASSED {
                let now = frame_system::Pallet::<T>::block_number();
                let is_stale = ProposalCreatedAt::<T>::get(existing_id)
                    .map(|created_at| {
                        now >= created_at.saturating_add(Self::effective_stale_lifetime())
                    })
                    .unwrap_or(false);
                // 中文注释：未通过且已经超过 stale 窗口的提案，下次 propose 时自动清理，避免机构长期被锁死。
                if is_stale {
                    return Ok(Some(existing_id));
                }
            }

            Err(Error::<T>::ActiveProposalExists.into())
        }

        fn is_key_used_by_other_institution(
            institution: InstitutionPalletId,
            key: &[u8; 32],
        ) -> bool {
            GrandpaKeyOwnerByKey::<T>::get(*key)
                .map(|owner| owner != institution)
                .unwrap_or(false)
        }

        fn remove_active_proposal_if_matches(institution: InstitutionPalletId, proposal_id: u64) {
            if ActiveProposalByInstitution::<T>::get(institution) == Some(proposal_id) {
                ActiveProposalByInstitution::<T>::remove(institution);
            }
        }

        fn cleanup_inactive_proposal(institution: InstitutionPalletId, proposal_id: u64) {
            // 中文注释：统一清理提案动作、new_key 并发索引、创建时间和投票引擎内部提案，避免残留脏状态。
            if let Some(action) = ProposalActions::<T>::take(proposal_id) {
                PendingProposalByNewKey::<T>::remove(action.new_key);
            }
            ProposalCreatedAt::<T>::remove(proposal_id);
            Self::remove_active_proposal_if_matches(institution, proposal_id);
            T::InternalVoteEngine::cleanup_internal_proposal(proposal_id);
        }

        fn try_execute(proposal_id: u64) -> DispatchResult {
            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            Self::try_execute_from_action(proposal_id, action)
        }

        fn try_execute_from_action(
            proposal_id: u64,
            action: GrandpaKeyReplacementAction,
        ) -> DispatchResult {
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
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
            Self::cleanup_inactive_proposal(action.institution, proposal_id);

            Self::deposit_event(Event::<T>::GrandpaKeyReplaced {
                proposal_id,
                institution: action.institution,
                old_key: action.old_key,
                new_key: action.new_key,
            });
            Ok(())
        }

        fn validate_action(
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

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok, derive_impl, parameter_types, traits::ConstU32};
    use frame_system as system;
    use primitives::china::china_cb::CHINA_CB;
    use sp_core::{Pair, Void};
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};

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
        pub type VotingEngineSystem = voting_engine_system;

        #[runtime::pallet_index(3)]
        pub type GrandpaKeyGov = super;
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
        pub const StaleProposalLifetime: u64 = 100;
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

    impl voting_engine_system::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
        for TestSfidEligibility
    {
        fn is_eligible(
            _sfid_hash: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
        ) -> bool {
            false
        }

        fn verify_and_consume_vote_credential(
            _sfid_hash: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
            _proposal_id: u64,
            _nonce: &[u8],
            _signature: &[u8],
        ) -> bool {
            false
        }

        fn cleanup_vote_credentials(_proposal_id: u64) {}
    }

    impl
        voting_engine_system::PopulationSnapshotVerifier<
            AccountId32,
            voting_engine_system::pallet::VoteNonceOf<Test>,
            voting_engine_system::pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            _eligible_total: u64,
            _nonce: &voting_engine_system::pallet::VoteNonceOf<Test>,
            _signature: &voting_engine_system::pallet::VoteSignatureOf<Test>,
        ) -> bool {
            true
        }
    }

    impl voting_engine_system::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            let mut who_raw = [0u8; 32];
            who_raw.copy_from_slice(who.as_ref());
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|node| reserve_pallet_id_to_bytes(node.shenfen_id) == Some(institution))
                    .map(|node| node.admins.iter().any(|admin| *admin == who_raw))
                    .unwrap_or(false),
                _ => false,
            }
        }
    }

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
        type WeightInfo = ();
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type StaleProposalLifetime = StaleProposalLifetime;
        type GrandpaChangeDelay = GrandpaChangeDelay;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
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
            .expect("grandpa-key-gov genesis should assimilate");

        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
        });
        ext
    }

    fn prc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[1].admins[index])
    }

    fn prc_pallet_id() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id)
            .expect("PRC institution should map to pallet id")
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

    #[test]
    fn weak_small_order_new_key_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                GrandpaKeyGov::propose_replace_grandpa_key(
                    RuntimeOrigin::signed(prc_admin(0)),
                    prc_pallet_id(),
                    identity_public_key()
                ),
                Error::<Test>::InvalidEd25519Key
            );
        });
    }

    #[test]
    fn rejected_proposal_is_cleaned_on_next_propose() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_new_key = valid_public_key(11);
            let replacement_key = valid_public_key(12);

            assert_ok!(GrandpaKeyGov::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                old_new_key,
            ));
            assert_eq!(PendingProposalByNewKey::<Test>::get(old_new_key), Some(0));

            let proposal = voting_engine_system::Pallet::<Test>::proposals(0)
                .expect("internal proposal should exist");
            System::set_block_number(proposal.end + 1);
            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                0,
            ));
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(0)
                    .expect("proposal should still exist")
                    .status,
                STATUS_REJECTED
            );

            assert_ok!(GrandpaKeyGov::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                replacement_key,
            ));

            assert!(ProposalActions::<Test>::get(0).is_none());
            assert!(ProposalCreatedAt::<Test>::get(0).is_none());
            assert!(PendingProposalByNewKey::<Test>::get(old_new_key).is_none());
            assert_eq!(
                ActiveProposalByInstitution::<Test>::get(institution),
                Some(1)
            );
        });
    }

    #[test]
    fn stale_unfinalized_proposal_is_cleaned_on_next_propose() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_new_key = valid_public_key(21);
            let replacement_key = valid_public_key(22);

            assert_ok!(GrandpaKeyGov::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                old_new_key,
            ));

            System::set_block_number(StaleProposalLifetime::get() + 2);
            assert_ok!(GrandpaKeyGov::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                replacement_key,
            ));

            assert!(ProposalActions::<Test>::get(0).is_none());
            assert!(ProposalCreatedAt::<Test>::get(0).is_none());
            assert!(PendingProposalByNewKey::<Test>::get(old_new_key).is_none());
            assert_eq!(
                ActiveProposalByInstitution::<Test>::get(institution),
                Some(1)
            );
        });
    }
}
