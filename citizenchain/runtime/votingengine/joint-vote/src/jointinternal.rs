//! 联合投票 — 内部投票阶段。
//!
//! 国储会 / 省储会 / 省储行管理员按机构投票,任一机构反对或超时进入
//! 联合公投阶段(jointreferendum)。
//!
//! 业务函数挂在 `super::Pallet<T>` 上,在 super(lib.rs)的 #[pallet::call]
//! `cast_admin` extrinsic 与 `JointVoteEngine` / `JointProposalFinalizer`
//! trait 实现中被调用。

use frame_support::{
    ensure,
    pallet_prelude::DispatchResult,
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::traits::{Hash, SaturatedConversion, Saturating};
use sp_runtime::DispatchError;

use primitives::china::china_cb::CHINA_CB;
use primitives::china::china_ch::CHINA_CH;
use primitives::count_const::{
    JOINT_VOTE_TOTAL, NRC_JOINT_VOTE_WEIGHT, PRB_JOINT_VOTE_WEIGHT, PRC_JOINT_VOTE_WEIGHT,
    VOTING_DURATION_BLOCKS,
};

use votingengine::{
    pallet::{Proposals, ProposalsByExpiry},
    types::{fixed_governance_pass_threshold, ORG_NRC, ORG_PRB, ORG_PRC},
    InternalAdminProvider, InternalProposalMutexKind, PopulationSnapshotVerifier, Proposal,
    PROPOSAL_KIND_JOINT, STAGE_JOINT, STATUS_PASSED,
};

use super::pallet::{
    Config, Error, Event, JointInstitutionTallies, JointTallies, JointVotesByAdmin,
    JointVotesByInstitution, Pallet, PendingPopulationSnapshots, PreparedPopulationSnapshot,
    UsedPopulationSnapshotNonce,
};
use super::{decode_account, institution_info, is_joint_unanimous, nrc_account};

#[cfg(test)]
use codec::Encode;

// ──────────────────────────────────────────────────────────────────
// 私有 helper:发起人机构解析 + (org, weight) profile
// ──────────────────────────────────────────────────────────────────

pub(super) fn institution_profile<T: Config>(id: &T::AccountId) -> Option<(u8, u32)> {
    if CHINA_CB
        .first()
        .and_then(|n| decode_account::<T>(&n.main_account))
        .as_ref()
        == Some(id)
    {
        return Some((ORG_NRC, NRC_JOINT_VOTE_WEIGHT));
    }
    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| decode_account::<T>(&n.main_account))
        .any(|account| &account == id)
    {
        return Some((ORG_PRC, PRC_JOINT_VOTE_WEIGHT));
    }
    if CHINA_CH
        .iter()
        .filter_map(|n| decode_account::<T>(&n.main_account))
        .any(|account| &account == id)
    {
        return Some((ORG_PRB, PRB_JOINT_VOTE_WEIGHT));
    }
    None
}

fn resolve_proposer_institution<T: Config>(who: &T::AccountId) -> Option<T::AccountId> {
    #[cfg(not(test))]
    {
        if let Some(nrc) = nrc_account::<T>() {
            if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                ORG_NRC,
                nrc.clone(),
                who,
            ) {
                return Some(nrc);
            }
        }
        for entry in CHINA_CB.iter().skip(1) {
            if let Some(prc) = decode_account::<T>(&entry.main_account) {
                if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    ORG_PRC,
                    prc.clone(),
                    who,
                ) {
                    return Some(prc);
                }
            }
        }
        None
    }
    #[cfg(test)]
    {
        if let Some(nrc) = nrc_account::<T>() {
            if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                ORG_NRC,
                nrc.clone(),
                who,
            ) {
                return Some(nrc);
            }
        }
        for entry in CHINA_CB.iter().skip(1) {
            if let Some(prc) = decode_account::<T>(&entry.main_account) {
                if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    ORG_PRC,
                    prc.clone(),
                    who,
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
            if let Some(institution) = decode_account::<T>(&entry.main_account) {
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

    /// 准备联合投票人口快照。
    ///
    /// 中文注释：这是投票引擎内部能力。业务模块不再传快照材料，只能在发起
    /// 联合提案前由管理员调用本入口，让 joint-vote 验签、去重并缓存总人数。
    pub fn do_prepare_population_snapshot(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: votingengine::pallet::VoteNonceOf<T>,
        signature: votingengine::pallet::VoteSignatureOf<T>,
        province_name: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> DispatchResult {
        let _proposer_institution = resolve_proposer_institution::<T>(&who)
            .ok_or(votingengine::Error::<T>::NoPermission)?;
        ensure!(eligible_total > 0, Error::<T>::CitizenEligibleTotalNotSet);
        ensure!(
            !snapshot_nonce.is_empty(),
            Error::<T>::InvalidPopulationSnapshot
        );
        ensure!(!signature.is_empty(), Error::<T>::InvalidPopulationSnapshot);
        ensure!(
            !province_name.is_empty(),
            Error::<T>::InvalidPopulationSnapshot
        );

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
                province_name,
                signer_admin_pubkey,
            ),
            Error::<T>::InvalidPopulationSnapshot
        );

        let now = <frame_system::Pallet<T>>::block_number();
        UsedPopulationSnapshotNonce::<T>::insert(snapshot_nonce_hash, true);
        PendingPopulationSnapshots::<T>::insert(
            &who,
            PreparedPopulationSnapshot {
                eligible_total,
                nonce_hash: snapshot_nonce_hash,
                prepared_at: now,
            },
        );
        Self::deposit_event(Event::<T>::PopulationSnapshotPrepared {
            who,
            eligible_total,
            nonce_hash: snapshot_nonce_hash,
        });
        Ok(())
    }

    /// 创建联合投票提案。锁定全部参与机构(NRC + 43 PRC + PRBs)管理员快照,
    /// 并消费已准备的人口快照总分母，后续阶段切换不再改写。
    pub fn do_create_joint_proposal(who: T::AccountId) -> Result<u64, DispatchError> {
        let proposer_institution = resolve_proposer_institution::<T>(&who)
            .ok_or(votingengine::Error::<T>::NoPermission)?;
        let prepared = PendingPopulationSnapshots::<T>::get(&who)
            .ok_or(Error::<T>::PopulationSnapshotNotPrepared)?;
        let now = <frame_system::Pallet<T>>::block_number();
        if prepared.prepared_at != now {
            PendingPopulationSnapshots::<T>::remove(&who);
            return Err(Error::<T>::PopulationSnapshotNotCurrent.into());
        }
        let eligible_total = prepared.eligible_total;
        let end = now.saturating_add(Self::joint_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_JOINT,
            status: votingengine::STATUS_VOTING,
            internal_org: None,
            internal_institution: Some(proposer_institution.clone()),
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
                votingengine::limit::try_add_active_proposal::<T>(proposer_institution.clone(), id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }

            // 锁定所有参与机构(NRC + 43 PRC + PRBs)的管理员快照。
            if let Some(nrc) = nrc_account::<T>() {
                if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                    id,
                    ORG_NRC,
                    nrc.clone(),
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
                if let Some(prc) = decode_account::<T>(&entry.main_account) {
                    if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                        id,
                        ORG_PRC,
                        prc.clone(),
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
                if let Some(prb) = decode_account::<T>(&entry.main_account) {
                    if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                        id,
                        ORG_PRB,
                        prb.clone(),
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
                return TransactionOutcome::Rollback(Err(
                    votingengine::Error::<T>::NoPermission.into()
                ));
            }

            PendingPopulationSnapshots::<T>::remove(&who);
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
    /// 全部机构票权累加判断是否全票通过(105 票)或推进至联合公投阶段。
    pub fn do_joint_vote(
        who: T::AccountId,
        proposal_id: u64,
        institution: T::AccountId,
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
            !JointVotesByInstitution::<T>::contains_key(proposal_id, institution.clone()),
            votingengine::Error::<T>::AlreadyVoted
        );
        let (org, _) = institution_profile::<T>(&institution)
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        ensure!(
            <votingengine::Pallet<T>>::is_admin_in_snapshot(proposal_id, institution.clone(), &who,),
            votingengine::Error::<T>::NoPermission
        );
        ensure!(
            !JointVotesByAdmin::<T>::contains_key(proposal_id, (institution.clone(), who.clone()),),
            votingengine::Error::<T>::AlreadyVoted
        );

        JointVotesByAdmin::<T>::insert(proposal_id, (institution.clone(), who.clone()), approve);
        let tally =
            JointInstitutionTallies::<T>::mutate(proposal_id, institution.clone(), |tally| {
                if approve {
                    tally.yes = tally.yes.saturating_add(1);
                } else {
                    tally.no = tally.no.saturating_add(1);
                }
                *tally
            });

        Self::deposit_event(Event::<T>::JointAdminVoteCast {
            proposal_id,
            institution: institution.clone(),
            who,
            approve,
        });

        let threshold = fixed_governance_pass_threshold(org)
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        let admin_count =
            <votingengine::Pallet<T>>::snapshot_admin_count(proposal_id, institution.clone())
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
        institution: T::AccountId,
        approved: bool,
    ) -> DispatchResult {
        ensure!(
            !JointVotesByInstitution::<T>::contains_key(proposal_id, institution.clone()),
            votingengine::Error::<T>::AlreadyVoted
        );
        let weight = institution_info::<T>(&institution)
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;

        JointVotesByInstitution::<T>::insert(proposal_id, institution.clone(), approved);

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

    /// 联合内部投票阶段超时结算:全票通过 → PASSED,否则进入联合公投阶段。
    pub fn do_finalize_joint_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
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
                    proposal.stage = votingengine::STAGE_REFERENDUM;
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
