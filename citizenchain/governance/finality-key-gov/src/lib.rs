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
pub mod weights;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
pub struct FinalityKeyReplacementAction {
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
    use sp_std::vec::Vec;
    use voting_engine_system::{InternalAdminProvider, InternalVoteEngine};

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config + pallet_grandpa::Config {
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
        StorageMap<_, Twox64Concat, u64, FinalityKeyReplacementAction, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_finality_key)]
    pub type CurrentFinalityKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, [u8; 32], OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn key_owner)]
    pub type FinalityKeyOwnerByKey<T: Config> =
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
            // 中文注释：初始最终性公钥与 CHINA_CB 的机构地址一一对应（1 国储会 + 43 省储会）。
            for node in CHINA_CB.iter() {
                let Some(institution) = reserve_pallet_id_to_bytes(node.shenfen_id) else {
                    continue;
                };
                assert!(
                    !FinalityKeyOwnerByKey::<T>::contains_key(node.finality_key),
                    "duplicated initial finality key in CHINA_CB"
                );
                CurrentFinalityKeys::<T>::insert(institution, node.finality_key);
                FinalityKeyOwnerByKey::<T>::insert(node.finality_key, institution);
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
                for (inst, key) in CurrentFinalityKeys::<T>::iter() {
                    reads = reads.saturating_add(1);
                    FinalityKeyOwnerByKey::<T>::insert(key, inst);
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
                CurrentFinalityKeys::<T>::iter().count() as u64,
                FinalityKeyOwnerByKey::<T>::iter().count() as u64,
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
                if (FinalityKeyOwnerByKey::<T>::iter().count() as u64) < pre_current_count {
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
        /// 已发起最终性密钥替换提案（并已在投票引擎创建内部提案）
        FinalityKeyReplacementProposed {
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            old_key: [u8; 32],
            new_key: [u8; 32],
        },
        /// 最终性密钥替换提案已提交一票
        FinalityKeyVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 提案达到通过状态但自动执行失败（投票不回滚）
        FinalityKeyExecutionFailed { proposal_id: u64 },
        /// 最终性密钥替换已完成并已调度 GRANDPA authority set 变更
        FinalityKeyReplaced {
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
        CurrentFinalityKeyNotFound,
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
        /// 发起“最终性密钥替换”内部投票提案（仅支持国储会/省储会）。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_replace_finality_key())]
        pub fn propose_replace_finality_key(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            new_key: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(new_key != [0u8; 32], Error::<T>::NewKeyIsZero);
            ensure!(
                CompressedEdwardsY(new_key).decompress().is_some(),
                Error::<T>::InvalidEd25519Key
            );

            let actual_org = institution_org(institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(matches!(actual_org, ORG_NRC | ORG_PRC), Error::<T>::UnsupportedOrg);
            ensure!(
                Self::is_internal_admin(actual_org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

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

            if let Some(existing_id) = ActiveProposalByInstitution::<T>::get(institution) {
                if ProposalActions::<T>::contains_key(existing_id) {
                    return Err(Error::<T>::ActiveProposalExists.into());
                }
                ActiveProposalByInstitution::<T>::remove(institution);
            }

            let old_key =
                CurrentFinalityKeys::<T>::get(institution).ok_or(Error::<T>::CurrentFinalityKeyNotFound)?;
            ensure!(new_key != old_key, Error::<T>::NewKeyUnchanged);
            ensure!(!Self::is_key_used_by_other_institution(institution, &new_key), Error::<T>::NewKeyAlreadyUsed);

            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), actual_org, institution)?;

            ProposalActions::<T>::insert(
                proposal_id,
                FinalityKeyReplacementAction {
                    institution,
                    old_key,
                    new_key,
                },
            );
            ProposalCreatedAt::<T>::insert(proposal_id, frame_system::Pallet::<T>::block_number());
            ActiveProposalByInstitution::<T>::insert(institution, proposal_id);

            Self::deposit_event(Event::<T>::FinalityKeyReplacementProposed {
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

        /// 对“最终性密钥替换”提案投票，达到阈值通过后自动执行替换。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::vote_replace_finality_key())]
        pub fn vote_replace_finality_key(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(Self::is_internal_admin(org, action.institution, &who), Error::<T>::UnauthorizedAdmin);

            T::InternalVoteEngine::cast_internal_vote(who.clone(), proposal_id, approve)?;

            Self::deposit_event(Event::<T>::FinalityKeyVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                if proposal.status == STATUS_PASSED {
                    if Self::try_execute_from_action(proposal_id, action).is_err() {
                        Self::deposit_event(Event::<T>::FinalityKeyExecutionFailed { proposal_id });
                    }
                } else if proposal.status == STATUS_REJECTED {
                    Self::cleanup_inactive_proposal(action.institution, proposal_id);
                }
            }

            Ok(())
        }

        /// 手动执行已通过的密钥替换提案。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::execute_replace_finality_key())]
        pub fn execute_replace_finality_key(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
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
        #[pallet::weight(<T as Config>::WeightInfo::cancel_stale_replace_finality_key())]
        pub fn cancel_stale_replace_finality_key(
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
        #[pallet::weight(<T as Config>::WeightInfo::cancel_failed_replace_finality_key())]
        pub fn cancel_failed_replace_finality_key(
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
            ensure!(proposal.status == STATUS_PASSED, Error::<T>::ProposalNotPassed);
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

        fn is_key_used_by_other_institution(
            institution: InstitutionPalletId,
            key: &[u8; 32],
        ) -> bool {
            FinalityKeyOwnerByKey::<T>::get(*key)
                .map(|owner| owner != institution)
                .unwrap_or(false)
        }

        fn remove_active_proposal_if_matches(institution: InstitutionPalletId, proposal_id: u64) {
            if ActiveProposalByInstitution::<T>::get(institution) == Some(proposal_id) {
                ActiveProposalByInstitution::<T>::remove(institution);
            }
        }

        fn cleanup_inactive_proposal(institution: InstitutionPalletId, proposal_id: u64) {
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
            action: FinalityKeyReplacementAction,
        ) -> DispatchResult {
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(proposal.status == STATUS_PASSED, Error::<T>::ProposalNotPassed);

            let next_authorities = Self::validate_action(&action)?;

            pallet_grandpa::Pallet::<T>::schedule_change(
                next_authorities,
                T::GrandpaChangeDelay::get(),
                None,
            )?;

            CurrentFinalityKeys::<T>::insert(action.institution, action.new_key);
            FinalityKeyOwnerByKey::<T>::remove(action.old_key);
            FinalityKeyOwnerByKey::<T>::insert(action.new_key, action.institution);
            Self::cleanup_inactive_proposal(action.institution, proposal_id);

            Self::deposit_event(Event::<T>::FinalityKeyReplaced {
                proposal_id,
                institution: action.institution,
                old_key: action.old_key,
                new_key: action.new_key,
            });
            Ok(())
        }

        fn validate_action(
            action: &FinalityKeyReplacementAction,
        ) -> Result<Vec<(GrandpaAuthorityId, u64)>, Error<T>> {
            ensure!(
                pallet_grandpa::Pallet::<T>::pending_change().is_none(),
                Error::<T>::GrandpaChangePending
            );

            let old_authority = GrandpaAuthorityId::from(ed25519::Public::from_raw(action.old_key));
            let new_authority = GrandpaAuthorityId::from(ed25519::Public::from_raw(action.new_key));

            let mut found = false;
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
