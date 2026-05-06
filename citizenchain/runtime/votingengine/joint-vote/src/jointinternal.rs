//! 联合投票 — 管理员阶段。
//!
//! 国储会 / 省储会 / 省储行管理员按机构投票,任一机构反对或超时进入
//! 全民兜底阶段(jointreferendum)。
//!
//! 业务函数挂在 `super::Pallet<T>` 上,在 super(lib.rs)的 #[pallet::call]
//! `cast_admin` extrinsic 与 `JointVoteEngine` / `JointProposalFinalizer`
//! trait 实现中被调用。

#[cfg(test)]
use codec::Encode;
use frame_support::{
    ensure,
    pallet_prelude::DispatchResult,
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::traits::{Hash, SaturatedConversion, Saturating};
use sp_runtime::DispatchError;

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use primitives::count_const::{
    JOINT_VOTE_TOTAL, NRC_JOINT_VOTE_WEIGHT, PRB_JOINT_VOTE_WEIGHT, PRC_JOINT_VOTE_WEIGHT,
    VOTING_DURATION_BLOCKS,
};

use votingengine::{
    nrc_pallet_id_bytes,
    pallet::{Proposals, ProposalsByExpiry},
    types::{fixed_governance_pass_threshold, ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId, InternalAdminProvider, InternalProposalMutexKind,
    PopulationSnapshotVerifier, Proposal, PROPOSAL_KIND_JOINT, STAGE_JOINT, STATUS_PASSED,
};

use super::pallet::{
    Config, Error, Event, JointInstitutionTallies, JointTallies, JointVotesByAdmin,
    JointVotesByInstitution, Pallet, UsedPopulationSnapshotNonce,
};
use super::{institution_info, is_joint_unanimous};

// ──────────────────────────────────────────────────────────────────
// 私有 helper:发起人机构解析 + (org, weight) profile
// ──────────────────────────────────────────────────────────────────

pub(super) fn institution_profile(id: InstitutionPalletId) -> Option<(u8, u32)> {
    if let Some(nrc) = nrc_pallet_id_bytes() {
        if id == nrc {
            return Some((ORG_NRC, NRC_JOINT_VOTE_WEIGHT));
        }
    }
    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
        .any(|pid| pid == id)
    {
        return Some((ORG_PRC, PRC_JOINT_VOTE_WEIGHT));
    }
    if CHINA_CH
        .iter()
        .filter_map(|n| shengbank_pallet_id_to_bytes(n.shenfen_id))
        .any(|pid| pid == id)
    {
        return Some((ORG_PRB, PRB_JOINT_VOTE_WEIGHT));
    }
    None
}

fn resolve_proposer_institution<T: Config>(who: &T::AccountId) -> Option<InstitutionPalletId> {
    #[cfg(not(test))]
    {
        if let Some(nrc) = nrc_pallet_id_bytes() {
            if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                ORG_NRC, nrc, who,
            ) {
                return Some(nrc);
            }
        }
        for entry in CHINA_CB.iter().skip(1) {
            if let Some(prc) = reserve_pallet_id_to_bytes(entry.shenfen_id) {
                if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    ORG_PRC, prc, who,
                ) {
                    return Some(prc);
                }
            }
        }
        None
    }
    #[cfg(test)]
    {
        if let Some(nrc) = nrc_pallet_id_bytes() {
            if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                ORG_NRC, nrc, who,
            ) {
                return Some(nrc);
            }
        }
        for entry in CHINA_CB.iter().skip(1) {
            if let Some(prc) = reserve_pallet_id_to_bytes(entry.shenfen_id) {
                if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    ORG_PRC, prc, who,
                ) {
                    return Some(prc);
                }
            }
        }
        let who_bytes = who.encode();
        if who_bytes.len() != 32 {
            return None;
        }
        let mut who_arr = [0u8; 32];
        who_arr.copy_from_slice(&who_bytes);
        for entry in CHINA_CB.iter() {
            if let Some(institution) = reserve_pallet_id_to_bytes(entry.shenfen_id) {
                if entry.duoqian_admins.iter().any(|admin| *admin == who_arr) {
                    return Some(institution);
                }
            }
        }
        None
    }
}

// ──────────────────────────────────────────────────────────────────
// 业务方法 — 挂在 super::Pallet<T> 上
// ──────────────────────────────────────────────────────────────────

impl<T: Config> Pallet<T> {
    pub(super) fn joint_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    pub(super) fn citizen_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    /// 创建联合投票提案。锁定全部参与机构(NRC + 43 PRC + PRBs)管理员快照,
    /// 并在创建时锁定公民投票总人口分母(eligible_total),后续阶段切换不再改写。
    /// ADR-008 step3:`(province, signer_admin_pubkey)` 双层匹配字段透传至 verifier。
    pub fn do_create_joint_proposal(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: votingengine::pallet::VoteNonceOf<T>,
        signature: votingengine::pallet::VoteSignatureOf<T>,
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> Result<u64, DispatchError> {
        let proposer_institution =
            resolve_proposer_institution::<T>(&who).ok_or(votingengine::Error::<T>::NoPermission)?;
        ensure!(eligible_total > 0, Error::<T>::CitizenEligibleTotalNotSet);
        ensure!(
            !snapshot_nonce.is_empty(),
            Error::<T>::InvalidPopulationSnapshot
        );
        ensure!(!signature.is_empty(), Error::<T>::InvalidPopulationSnapshot);
        ensure!(!province.is_empty(), Error::<T>::InvalidPopulationSnapshot);

        let snapshot_nonce_hash = T::Hashing::hash(snapshot_nonce.as_slice());
        ensure!(
            !UsedPopulationSnapshotNonce::<T>::get(snapshot_nonce_hash),
            Error::<T>::InvalidPopulationSnapshot
        );
        ensure!(
            <T as votingengine::Config>::PopulationSnapshotVerifier::verify_population_snapshot(
                &who,
                eligible_total,
                &snapshot_nonce,
                &signature,
                province,
                signer_admin_pubkey,
            ),
            Error::<T>::InvalidPopulationSnapshot
        );

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::joint_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_JOINT,
            status: votingengine::STATUS_VOTING,
            internal_org: None,
            internal_institution: Some(proposer_institution),
            start: now,
            end,
            citizen_eligible_total: eligible_total,
        };

        with_transaction(|| {
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            if let Err(err) =
                votingengine::limit::try_add_active_proposal::<T>(proposer_institution, id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }

            // 锁定所有参与机构(NRC + 43 PRC + PRBs)的管理员快照。
            if let Some(nrc) = nrc_pallet_id_bytes() {
                if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                    id,
                    ORG_NRC,
                    nrc,
                    InternalProposalMutexKind::Regular,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
                if let Err(err) =
                    <votingengine::Pallet<T>>::snapshot_institution_admins(id, ORG_NRC, nrc, false)
                {
                    return TransactionOutcome::Rollback(Err(err));
                }
            }
            for entry in CHINA_CB.iter().skip(1) {
                if let Some(prc) = reserve_pallet_id_to_bytes(entry.shenfen_id) {
                    if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                        id,
                        ORG_PRC,
                        prc,
                        InternalProposalMutexKind::Regular,
                    ) {
                        return TransactionOutcome::Rollback(Err(err));
                    }
                    if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                        id, ORG_PRC, prc, false,
                    ) {
                        return TransactionOutcome::Rollback(Err(err));
                    }
                }
            }
            for entry in CHINA_CH.iter() {
                if let Some(prb) = shengbank_pallet_id_to_bytes(entry.shenfen_id) {
                    if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                        id,
                        ORG_PRB,
                        prb,
                        InternalProposalMutexKind::Regular,
                    ) {
                        return TransactionOutcome::Rollback(Err(err));
                    }
                    if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                        id, ORG_PRB, prb, false,
                    ) {
                        return TransactionOutcome::Rollback(Err(err));
                    }
                }
            }
            if !<votingengine::Pallet<T>>::is_admin_in_snapshot(id, proposer_institution, &who) {
                frame_support::defensive!(
                    "do_create_joint_proposal: proposer is missing from admin snapshot"
                );
                return TransactionOutcome::Rollback(Err(votingengine::Error::<T>::NoPermission
                    .into()));
            }

            UsedPopulationSnapshotNonce::<T>::insert(snapshot_nonce_hash, true);
            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_JOINT,
                STAGE_JOINT,
                end,
            );
            TransactionOutcome::Commit(Ok(id))
        })
    }

    /// 联合投票:管理员按机构投票。机构内达阈值后写入 `JointVotesByInstitution`,
    /// 全部机构票权累加判断是否全票通过(105 票)或推进至公民投票兜底。
    pub fn do_joint_vote(
        who: T::AccountId,
        proposal_id: u64,
        institution: InstitutionPalletId,
        approve: bool,
    ) -> DispatchResult {
        let proposal = <votingengine::Pallet<T>>::ensure_open_proposal(proposal_id)?;

        ensure!(
            proposal.kind == PROPOSAL_KIND_JOINT,
            votingengine::Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == STAGE_JOINT,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            !JointVotesByInstitution::<T>::contains_key(proposal_id, institution),
            votingengine::Error::<T>::AlreadyVoted
        );
        let (org, _) = institution_profile(institution)
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        ensure!(
            <votingengine::Pallet<T>>::is_admin_in_snapshot(proposal_id, institution, &who),
            votingengine::Error::<T>::NoPermission
        );
        ensure!(
            !JointVotesByAdmin::<T>::contains_key(proposal_id, (institution, who.clone())),
            votingengine::Error::<T>::AlreadyVoted
        );

        JointVotesByAdmin::<T>::insert(proposal_id, (institution, who.clone()), approve);
        let tally = JointInstitutionTallies::<T>::mutate(proposal_id, institution, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
            *tally
        });

        Self::deposit_event(Event::<T>::JointAdminVoteCast {
            proposal_id,
            institution,
            who,
            approve,
        });

        let threshold = fixed_governance_pass_threshold(org)
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        let admin_count = <votingengine::Pallet<T>>::snapshot_admin_count(proposal_id, institution)
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;

        if tally.yes >= threshold {
            return Self::finalize_joint_institution_vote(proposal_id, institution, true);
        }
        let casted_votes = tally.yes.saturating_add(tally.no);
        let remaining_admins = admin_count.saturating_sub(casted_votes);
        if tally.yes.saturating_add(remaining_admins) < threshold {
            return Self::finalize_joint_institution_vote(proposal_id, institution, false);
        }

        Ok(())
    }

    fn finalize_joint_institution_vote(
        proposal_id: u64,
        institution: InstitutionPalletId,
        approved: bool,
    ) -> DispatchResult {
        ensure!(
            !JointVotesByInstitution::<T>::contains_key(proposal_id, institution),
            votingengine::Error::<T>::AlreadyVoted
        );
        let weight =
            institution_info(institution).ok_or(votingengine::Error::<T>::InvalidInstitution)?;

        JointVotesByInstitution::<T>::insert(proposal_id, institution, approved);

        let tally = JointTallies::<T>::mutate(proposal_id, |tally| {
            if approved {
                tally.yes = tally.yes.saturating_add(weight);
            } else {
                tally.no = tally.no.saturating_add(weight);
            }
            *tally
        });

        Self::deposit_event(Event::<T>::JointInstitutionVoteFinalized {
            proposal_id,
            institution,
            approved,
        });

        if approved {
            if is_joint_unanimous(tally.yes) {
                <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)?;
                return Ok(());
            }
            if tally.yes.saturating_add(tally.no) >= JOINT_VOTE_TOTAL {
                return Self::advance_joint_to_citizen(proposal_id);
            }
            return Ok(());
        }
        Self::advance_joint_to_citizen(proposal_id)
    }

    /// 联合管理员阶段超时结算:全票通过 → PASSED,否则进入公民投票阶段。
    pub fn do_finalize_joint_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_JOINT,
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

        let tally = JointTallies::<T>::get(proposal_id);
        if is_joint_unanimous(tally.yes) {
            return <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED);
        }
        Self::advance_joint_to_citizen(proposal_id)
    }

    fn advance_joint_to_citizen(proposal_id: u64) -> DispatchResult {
        let now = <frame_system::Pallet<T>>::block_number();
        let citizen_end = now.saturating_add(Self::citizen_stage_duration());
        with_transaction(|| {
            let (eligible_total, old_end) = match Proposals::<T>::try_mutate(
                proposal_id,
                |maybe| -> Result<
                    (u64, frame_system::pallet_prelude::BlockNumberFor<T>),
                    DispatchError,
                > {
                    let proposal = maybe
                        .as_mut()
                        .ok_or(votingengine::Error::<T>::ProposalNotFound)?;
                    let eligible_total = proposal.citizen_eligible_total;
                    let old_end = proposal.end;
                    proposal.stage = votingengine::STAGE_CITIZEN;
                    proposal.start = now;
                    proposal.end = citizen_end;
                    Ok((eligible_total, old_end))
                },
            ) {
                Ok(v) => v,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            let old_expiry = old_end.saturating_add(sp_runtime::traits::One::one());
            ProposalsByExpiry::<T>::mutate(old_expiry, |ids| {
                ids.retain(|&id| id != proposal_id);
            });

            if let Err(err) =
                <votingengine::Pallet<T>>::schedule_proposal_expiry(proposal_id, citizen_end)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::release_internal_proposal_mutexes(proposal_id);

            <votingengine::Pallet<T>>::emit_proposal_advanced_to_citizen(
                proposal_id,
                citizen_end,
                eligible_total,
            );
            TransactionOutcome::Commit(Ok(()))
        })
    }
}
