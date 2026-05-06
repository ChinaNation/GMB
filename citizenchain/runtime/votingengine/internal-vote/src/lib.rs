//! # 内部投票 pallet (internal-vote)
//!
//! 治理机构 / 注册多签的"管理员一人一票"投票模式。
//!
//! 共用基础设施(Proposals 主 storage / 双层 ID / 反向索引 / 状态机骨架 / 快照 / 锁 / 清理)
//! 仍归 [`votingengine`] 引擎核心,本 pallet 通过 `Config: votingengine::Config` 直接访问。
//!
//! 本 pallet 自有:
//! - storage:`InternalVotesByAccount` / `InternalTallies` / `InternalThresholdSnapshot`
//! - event:`InternalVoteCast`
//! - error:`InvalidInternalOrg` / `MissingThresholdSnapshot`
//! - extrinsic:`cast(proposal_id, approve)`
//! - 业务函数:`do_create_internal_proposal*` / `do_internal_vote` / `do_finalize_internal_timeout`
//! - trait impl:`InternalVoteEngine`(供业务 pallet 创建提案)
//! - trait impl:`InternalProposalFinalizer`(votingengine 主 pallet finalize 路径反向调用)
//! - trait impl:`InternalCleanupHandler`(votingengine 主 pallet cleanup 状态机反向调用)

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    ensure,
    pallet_prelude::{BoundedVec, DispatchResult},
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::traits::{SaturatedConversion, Saturating};
use sp_runtime::DispatchError;

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use primitives::count_const::VOTING_DURATION_BLOCKS;

use votingengine::{
    nrc_pallet_id_bytes,
    pallet::{AdminSnapshot, Proposals},
    types::{
        fixed_governance_pass_threshold, is_valid_org, ORG_NRC, ORG_PRB, ORG_PRC,
        ORG_REN,
    },
    InstitutionPalletId, InternalAdminProvider, InternalProposalMutexKind,
    InternalThresholdProvider, Proposal, PROPOSAL_KIND_INTERNAL, STAGE_INTERNAL, STATUS_PASSED,
    STATUS_REJECTED,
};

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

pub use pallet::*;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 内部投票记录:(proposal_id, 管理员公钥) → 赞成/反对。防止同一管理员重复投票。
    #[pallet::storage]
    pub type InternalVotesByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn internal_tally)]
    pub type InternalTallies<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, votingengine::VoteCountU32, ValueQuery>;

    /// 内部投票阈值快照:提案创建时锁定阈值,投票期间不受主体状态变化影响。
    #[pallet::storage]
    pub type InternalThresholdSnapshot<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, u32, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 内部投票已投出一票。
        InternalVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 内部投票的机构类型不合法。
        InvalidInternalOrg,
        /// 内部投票阈值快照缺失。
        MissingThresholdSnapshot,
    }

    use crate::weights::WeightInfo;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 内部投票:管理员一人一票。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::cast())]
        pub fn cast(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_internal_vote(who, proposal_id, approve)
        }
    }
}

// ──────────────────────────────────────────────────────────────────
// 内部判定 helper
// ──────────────────────────────────────────────────────────────────

fn is_valid_internal_institution<T: Config>(
    org: u8,
    institution: InstitutionPalletId,
    pending_subject: bool,
) -> bool {
    match org {
        ORG_NRC => {
            !pending_subject
                && nrc_pallet_id_bytes()
                    .map(|nrc| institution == nrc)
                    .unwrap_or(false)
        }
        ORG_PRC => {
            !pending_subject
                && CHINA_CB
                    .iter()
                    .skip(1)
                    .filter_map(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
                    .any(|pid| pid == institution)
        }
        ORG_PRB => {
            !pending_subject
                && CHINA_CH
                    .iter()
                    .filter_map(|n| shengbank_pallet_id_to_bytes(n.shenfen_id))
                    .any(|pid| pid == institution)
        }
        ORG_REN if pending_subject => {
            <T as votingengine::Config>::InternalThresholdProvider::is_known_pending_subject(
                org,
                institution,
            )
        }
        ORG_REN => {
            <T as votingengine::Config>::InternalThresholdProvider::is_known_subject(
                org,
                institution,
            )
        }
        _ => false,
    }
}

fn is_internal_admin<T: Config>(
    org: u8,
    institution: InstitutionPalletId,
    who: &T::AccountId,
    pending_subject: bool,
) -> bool {
    if pending_subject {
        return <T as votingengine::Config>::InternalAdminProvider::is_pending_internal_admin(
            org,
            institution,
            who,
        );
    }
    <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(org, institution, who)
}

fn internal_threshold<T: Config>(
    org: u8,
    institution: InstitutionPalletId,
    pending_subject: bool,
) -> Option<u32> {
    match org {
        ORG_NRC | ORG_PRC | ORG_PRB => fixed_governance_pass_threshold(org),
        ORG_REN if pending_subject => {
            <T as votingengine::Config>::InternalThresholdProvider::pending_pass_threshold(
                org,
                institution,
            )
        }
        ORG_REN => {
            <T as votingengine::Config>::InternalThresholdProvider::pass_threshold(org, institution)
        }
        _ => None,
    }
}

// ──────────────────────────────────────────────────────────────────
// 业务方法
// ──────────────────────────────────────────────────────────────────

impl<T: Config> Pallet<T> {
    fn internal_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    pub fn do_create_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Self::do_create_internal_proposal_with_subject_status(
            who,
            org,
            institution,
            false,
            InternalProposalMutexKind::Regular,
        )
    }

    pub fn do_create_pending_subject_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Self::do_create_internal_proposal_with_subject_status(
            who,
            org,
            institution,
            true,
            InternalProposalMutexKind::Regular,
        )
    }

    pub fn do_create_pending_subject_internal_proposal_with_snapshot(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        admins: sp_std::vec::Vec<T::AccountId>,
        threshold: u32,
    ) -> Result<u64, DispatchError> {
        ensure!(org == ORG_REN, Error::<T>::InvalidInternalOrg);
        ensure!(
            !admins.is_empty(),
            votingengine::Error::<T>::MissingAdminSnapshot
        );
        ensure!(
            admins.iter().any(|admin| admin == &who),
            votingengine::Error::<T>::NoPermission
        );
        for i in 0..admins.len() {
            for j in i.saturating_add(1)..admins.len() {
                ensure!(
                    admins[i] != admins[j],
                    votingengine::Error::<T>::InvalidInstitution
                );
            }
        }
        let admin_count = admins.len() as u32;
        ensure!(
            threshold > 0 && threshold <= admin_count,
            Error::<T>::InvalidInternalOrg
        );
        let bounded_admins: BoundedVec<
            T::AccountId,
            <T as votingengine::Config>::MaxAdminsPerInstitution,
        > = admins
            .try_into()
            .map_err(|_| votingengine::Error::<T>::InvalidInstitution)?;

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());
        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_org: Some(org),
            internal_institution: Some(institution),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            if let Err(err) =
                votingengine::limit::try_add_active_proposal::<T>(institution, id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                id,
                org,
                institution,
                InternalProposalMutexKind::Regular,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }

            AdminSnapshot::<T>::insert(id, institution, bounded_admins);
            InternalThresholdSnapshot::<T>::insert(id, threshold);
            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_INTERNAL,
                STAGE_INTERNAL,
                end,
            );
            TransactionOutcome::Commit(Ok(id))
        })
    }

    pub fn do_create_admin_set_mutation_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Self::do_create_internal_proposal_with_subject_status(
            who,
            org,
            institution,
            false,
            InternalProposalMutexKind::AdminSetMutationExclusive,
        )
    }

    /// 创建普通内部提案,但**显式传 threshold**(不走 internal_threshold 反查)。
    pub fn do_create_internal_proposal_with_explicit_threshold(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        threshold: u32,
    ) -> Result<u64, DispatchError> {
        ensure!(is_valid_org(org), Error::<T>::InvalidInternalOrg);
        ensure!(
            is_valid_internal_institution::<T>(org, institution, false),
            votingengine::Error::<T>::InvalidInstitution
        );
        ensure!(
            is_internal_admin::<T>(org, institution, &who, false),
            votingengine::Error::<T>::NoPermission
        );
        ensure!(threshold > 0, Error::<T>::InvalidInternalOrg);

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_org: Some(org),
            internal_institution: Some(institution),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            if let Err(err) =
                votingengine::limit::try_add_active_proposal::<T>(institution, id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                id,
                org,
                institution,
                InternalProposalMutexKind::Regular,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }

            if let Err(err) =
                <votingengine::Pallet<T>>::snapshot_institution_admins(id, org, institution, false)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if !<votingengine::Pallet<T>>::is_admin_in_snapshot(id, institution, &who) {
                frame_support::defensive!(
                    "do_create_internal_proposal_with_explicit_threshold: proposer is missing from admin snapshot"
                );
                return TransactionOutcome::Rollback(Err(votingengine::Error::<T>::NoPermission.into()));
            }
            let snapshot_size =
                AdminSnapshot::<T>::get(id, institution).map(|admins| admins.len() as u32);
            if let Some(size) = snapshot_size {
                if threshold > size {
                    return TransactionOutcome::Rollback(Err(Error::<T>::InvalidInternalOrg.into()));
                }
            }
            InternalThresholdSnapshot::<T>::insert(id, threshold);

            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_INTERNAL,
                STAGE_INTERNAL,
                end,
            );
            TransactionOutcome::Commit(Ok(id))
        })
    }

    fn do_create_internal_proposal_with_subject_status(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        pending_subject: bool,
        mutex_kind: InternalProposalMutexKind,
    ) -> Result<u64, DispatchError> {
        ensure!(is_valid_org(org), Error::<T>::InvalidInternalOrg);
        ensure!(
            is_valid_internal_institution::<T>(org, institution, pending_subject),
            votingengine::Error::<T>::InvalidInstitution
        );
        ensure!(
            is_internal_admin::<T>(org, institution, &who, pending_subject),
            votingengine::Error::<T>::NoPermission
        );
        let threshold = internal_threshold::<T>(org, institution, pending_subject)
            .ok_or(Error::<T>::InvalidInternalOrg)?;

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_org: Some(org),
            internal_institution: Some(institution),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            if let Err(err) =
                votingengine::limit::try_add_active_proposal::<T>(institution, id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                id, org, institution, mutex_kind,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }

            if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                id,
                org,
                institution,
                pending_subject,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            if !<votingengine::Pallet<T>>::is_admin_in_snapshot(id, institution, &who) {
                frame_support::defensive!(
                    "do_create_internal_proposal: proposer is missing from admin snapshot"
                );
                return TransactionOutcome::Rollback(Err(votingengine::Error::<T>::NoPermission.into()));
            }
            InternalThresholdSnapshot::<T>::insert(id, threshold);

            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_INTERNAL,
                STAGE_INTERNAL,
                end,
            );
            TransactionOutcome::Commit(Ok(id))
        })
    }

    pub fn do_internal_vote(
        who: T::AccountId,
        proposal_id: u64,
        approve: bool,
    ) -> DispatchResult {
        let proposal = <votingengine::Pallet<T>>::ensure_open_proposal(proposal_id)?;

        ensure!(
            proposal.kind == PROPOSAL_KIND_INTERNAL,
            votingengine::Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == STAGE_INTERNAL,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            !InternalVotesByAccount::<T>::contains_key(proposal_id, &who),
            votingengine::Error::<T>::AlreadyVoted
        );
        let _org = proposal
            .internal_org
            .ok_or(Error::<T>::InvalidInternalOrg)?;
        let institution = proposal
            .internal_institution
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        ensure!(
            <votingengine::Pallet<T>>::is_admin_in_snapshot(proposal_id, institution, &who),
            votingengine::Error::<T>::NoPermission
        );

        InternalVotesByAccount::<T>::insert(proposal_id, &who, approve);
        let tally = InternalTallies::<T>::mutate(proposal_id, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
            *tally
        });

        Self::deposit_event(Event::<T>::InternalVoteCast {
            proposal_id,
            who,
            approve,
        });

        let threshold = InternalThresholdSnapshot::<T>::get(proposal_id)
            .ok_or(Error::<T>::MissingThresholdSnapshot)?;
        if tally.yes >= threshold {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)?;
        } else {
            let admin_count = <votingengine::Pallet<T>>::snapshot_admin_count(
                proposal_id,
                institution,
            )
            .ok_or(votingengine::Error::<T>::MissingAdminSnapshot)?;
            let casted = tally.yes.saturating_add(tally.no);
            let remaining = admin_count.saturating_sub(casted);
            if tally.yes.saturating_add(remaining) < threshold {
                <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)?;
            }
        }

        Ok(())
    }

    pub fn do_finalize_internal_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_INTERNAL,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.status == votingengine::STATUS_VOTING,
            votingengine::Error::<T>::ProposalAlreadyFinalized
        );
        ensure!(
            <frame_system::Pallet<T>>::block_number() > proposal.end,
            votingengine::Error::<T>::VoteNotExpired
        );
        <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, votingengine::STATUS_REJECTED)
    }
}

// ──────────────────────────────────────────────────────────────────
// trait 实现
// ──────────────────────────────────────────────────────────────────

impl<T: Config> votingengine::InternalVoteEngine<T::AccountId> for Pallet<T> {
    fn create_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Self::do_create_internal_proposal(who, org, institution)
    }

    fn create_internal_proposal_with_data(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id = match Self::do_create_internal_proposal(who, org, institution) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let now = <frame_system::Pallet<T>>::block_number();
            match <votingengine::Pallet<T>>::register_proposal_data(
                proposal_id,
                module_tag,
                data,
                now,
            ) {
                Ok(()) => TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_internal_proposal_with_threshold_and_data(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        threshold: u32,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id = match Self::do_create_internal_proposal_with_explicit_threshold(
                who,
                org,
                institution,
                threshold,
            ) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let now = <frame_system::Pallet<T>>::block_number();
            match <votingengine::Pallet<T>>::register_proposal_data(
                proposal_id,
                module_tag,
                data,
                now,
            ) {
                Ok(()) => TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_pending_subject_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Self::do_create_pending_subject_internal_proposal(who, org, institution)
    }

    fn create_pending_subject_internal_proposal_with_data(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id =
                match Self::do_create_pending_subject_internal_proposal(who, org, institution) {
                    Ok(id) => id,
                    Err(err) => return TransactionOutcome::Rollback(Err(err)),
                };
            let now = <frame_system::Pallet<T>>::block_number();
            match <votingengine::Pallet<T>>::register_proposal_data(
                proposal_id,
                module_tag,
                data,
                now,
            ) {
                Ok(()) => TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_pending_subject_internal_proposal_with_snapshot_data(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        admins: sp_std::vec::Vec<T::AccountId>,
        threshold: u32,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id = match Self::do_create_pending_subject_internal_proposal_with_snapshot(
                who,
                org,
                institution,
                admins,
                threshold,
            ) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let now = <frame_system::Pallet<T>>::block_number();
            match <votingengine::Pallet<T>>::register_proposal_data(
                proposal_id,
                module_tag,
                data,
                now,
            ) {
                Ok(()) => TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_admin_set_mutation_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Self::do_create_admin_set_mutation_internal_proposal(who, org, institution)
    }

    fn create_admin_set_mutation_internal_proposal_with_data(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id =
                match Self::do_create_admin_set_mutation_internal_proposal(who, org, institution) {
                    Ok(id) => id,
                    Err(err) => return TransactionOutcome::Rollback(Err(err)),
                };
            let now = <frame_system::Pallet<T>>::block_number();
            match <votingengine::Pallet<T>>::register_proposal_data(
                proposal_id,
                module_tag,
                data,
                now,
            ) {
                Ok(()) => TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => TransactionOutcome::Rollback(Err(err)),
            }
        })
    }
}

impl<T: Config> votingengine::traits::InternalProposalFinalizer<frame_system::pallet_prelude::BlockNumberFor<T>>
    for Pallet<T>
{
    fn finalize_internal_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_internal_timeout(proposal, proposal_id)
    }
}

impl<T: Config> votingengine::traits::InternalCleanupHandler for Pallet<T> {
    fn cleanup_internal_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::traits::CleanupChunkResult {
        let result = InternalVotesByAccount::<T>::clear_prefix(proposal_id, limit, None);
        let has_remaining = result.maybe_cursor.is_some();
        (result.unique, has_remaining)
    }

    fn cleanup_internal_terminal(proposal_id: u64) {
        InternalTallies::<T>::remove(proposal_id);
        InternalThresholdSnapshot::<T>::remove(proposal_id);
    }
}
