//! 联合投票 — 内部投票阶段。
//!
//! 国家储委会 / 省储委会 / 省储行管理员按机构投票,任一机构反对或超时都进入联合公投阶段(jointreferendum)。
//!
//! 业务函数挂在 `super::Pallet<T>` 上,在 super(lib.rs)的 #[pallet::call]
//! `cast_admin` extrinsic 与 `JointVoteEngine` / `JointProposalFinalizer`
//! trait 实现中被调用。

use frame_support::{
    ensure,
    pallet_prelude::DispatchResult,
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::traits::{SaturatedConversion, Saturating};
use sp_runtime::DispatchError;

use primitives::cid::china::china_cb::CHINA_CB;
use primitives::cid::china::china_ch::CHINA_CH;
use primitives::count_const::{
    JOINT_VOTE_TOTAL, NRC_JOINT_VOTE_WEIGHT, PRB_JOINT_VOTE_WEIGHT, PRC_JOINT_VOTE_WEIGHT,
    VOTING_DURATION_BLOCKS,
};

use votingengine::{
    pallet::{Proposals, ProposalsByExpiry},
    types::{
        fixed_governance_pass_threshold, InstitutionCode, ProposalSubjectCidNumbers, NRC, PRB, PRC,
    },
    InternalAdminProvider, InternalProposalMutexKind, PopulationScope, Proposal,
    PROPOSAL_KIND_JOINT, STAGE_JOINT, STATUS_PASSED,
};

use super::pallet::{
    Config, Error, Event, JointInstitutionTallies, JointTallies, JointVotesByAdmin,
    JointVotesByInstitution, Pallet, PendingPopulationSnapshots, PreparedPopulationSnapshot,
};
use super::{decode_account, institution_info, is_joint_unanimous, nrc_account};

#[cfg(test)]
use codec::Encode;
// 私有 helper:发起人机构解析 + (institution_code, weight) profile
pub(super) fn institution_profile<T: Config>(id: &T::AccountId) -> Option<(InstitutionCode, u32)> {
    if CHINA_CB
        .first()
        .and_then(|n| decode_account::<T>(&n.main_account))
        .as_ref()
        == Some(id)
    {
        return Some((NRC, NRC_JOINT_VOTE_WEIGHT));
    }
    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| decode_account::<T>(&n.main_account))
        .any(|account| &account == id)
    {
        return Some((PRC, PRC_JOINT_VOTE_WEIGHT));
    }
    if CHINA_CH
        .iter()
        .filter_map(|n| decode_account::<T>(&n.main_account))
        .any(|account| &account == id)
    {
        return Some((PRB, PRB_JOINT_VOTE_WEIGHT));
    }
    None
}

fn resolve_proposer_institution<T: Config>(who: &T::AccountId) -> Option<T::AccountId> {
    #[cfg(not(test))]
    {
        if let Some(nrc) = nrc_account::<T>() {
            if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                NRC,
                nrc.clone(),
                who,
            ) {
                return Some(nrc);
            }
        }
        for entry in CHINA_CB.iter().skip(1) {
            if let Some(prc) = decode_account::<T>(&entry.main_account) {
                if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    PRC,
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
                NRC,
                nrc.clone(),
                who,
            ) {
                return Some(nrc);
            }
        }
        for entry in CHINA_CB.iter().skip(1) {
            if let Some(prc) = decode_account::<T>(&entry.main_account) {
                if <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    PRC,
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
                if entry.admins.iter().any(|admin| *admin == who_arr) {
                    return Some(institution);
                }
            }
        }
        None
    }
}

fn joint_subject_cid_numbers<T: Config>() -> Result<ProposalSubjectCidNumbers, DispatchError> {
    let mut raw = sp_runtime::sp_std::vec::Vec::new();
    for entry in CHINA_CB.iter() {
        raw.push(entry.cid_number.as_bytes().to_vec());
    }
    for entry in CHINA_CH.iter() {
        raw.push(entry.cid_number.as_bytes().to_vec());
    }
    <votingengine::Pallet<T>>::bound_subject_cid_numbers(raw)
}
// 业务方法 — 挂在 super::Pallet<T> 上
impl<T: Config> Pallet<T> {
    pub(super) fn joint_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    pub(super) fn referendum_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    /// 准备联合投票人口快照。
    ///
    /// 这是投票引擎内部能力。业务模块不传快照材料，只能在发起联合提案前由管理员调用本入口，让 joint-vote 从链上身份模块读取总人数。
    pub fn do_prepare_joint_population_snapshot(
        who: T::AccountId,
        scope: PopulationScope,
    ) -> DispatchResult {
        let _proposer_institution = resolve_proposer_institution::<T>(&who)
            .ok_or(votingengine::Error::<T>::NoPermission)?;
        let (snapshot_id, eligible_total) =
            <votingengine::Pallet<T>>::create_population_snapshot(&scope)?;
        if eligible_total == 0 {
            <votingengine::Pallet<T>>::release_population_snapshot(snapshot_id);
            return Err(Error::<T>::CitizenEligibleTotalNotSet.into());
        }

        let now = <frame_system::Pallet<T>>::block_number();
        if let Some(previous) = PendingPopulationSnapshots::<T>::take(&who) {
            <votingengine::Pallet<T>>::release_population_snapshot(previous.snapshot_id);
        }
        PendingPopulationSnapshots::<T>::insert(
            &who,
            PreparedPopulationSnapshot {
                snapshot_id,
                eligible_total,
                prepared_at: now,
            },
        );
        Self::deposit_event(Event::<T>::PopulationSnapshotPrepared {
            who,
            eligible_total,
            scope,
        });
        Ok(())
    }

    /// 创建联合投票提案。锁定全部参与机构(NRC + 43 PRC + 43 PRB)管理员快照,
    /// 并消费已准备的人口快照总分母，后续阶段切换不再改写。
    pub fn do_create_joint_proposal(who: T::AccountId) -> Result<u64, DispatchError> {
        let proposer_institution = resolve_proposer_institution::<T>(&who)
            .ok_or(votingengine::Error::<T>::NoPermission)?;
        let prepared = PendingPopulationSnapshots::<T>::get(&who)
            .ok_or(Error::<T>::PopulationSnapshotNotPrepared)?;
        let now = <frame_system::Pallet<T>>::block_number();
        if prepared.prepared_at != now {
            PendingPopulationSnapshots::<T>::remove(&who);
            <votingengine::Pallet<T>>::release_population_snapshot(prepared.snapshot_id);
            return Err(Error::<T>::PopulationSnapshotNotCurrent.into());
        }
        let snapshot_id = prepared.snapshot_id;
        let eligible_total = prepared.eligible_total;
        let end = now.saturating_add(Self::joint_stage_duration());
        let subject_cid_numbers = joint_subject_cid_numbers::<T>()?;

        let proposal = Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_JOINT,
            status: votingengine::STATUS_VOTING,
            internal_code: None,
            account_context: Some(proposer_institution.clone()),
            subject_cid_numbers,
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
                votingengine::limit::try_add_active_proposals::<T>(proposal.subject_keys(), id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }

            // 联合提案关联全部固定治理机构,互斥锁按机构 CID 而非账户占用。
            for subject in proposal.subject_keys() {
                if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                    id,
                    subject,
                    InternalProposalMutexKind::Regular,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
            }

            // 锁定所有参与机构(NRC + 43 PRC + 43 PRB)的管理员快照。
            if let Some(nrc) = nrc_account::<T>() {
                if let Err(err) =
                    <votingengine::Pallet<T>>::snapshot_institution_admins(id, NRC, nrc, false)
                {
                    return TransactionOutcome::Rollback(Err(err));
                }
            }
            for entry in CHINA_CB.iter().skip(1) {
                if let Some(prc) = decode_account::<T>(&entry.main_account) {
                    if let Err(err) =
                        <votingengine::Pallet<T>>::snapshot_institution_admins(id, PRC, prc, false)
                    {
                        return TransactionOutcome::Rollback(Err(err));
                    }
                }
            }
            for entry in CHINA_CH.iter() {
                if let Some(prb) = decode_account::<T>(&entry.main_account) {
                    if let Err(err) =
                        <votingengine::Pallet<T>>::snapshot_institution_admins(id, PRB, prb, false)
                    {
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
            if let Err(err) = <votingengine::Pallet<T>>::bind_population_snapshot(id, snapshot_id) {
                return TransactionOutcome::Rollback(Err(err));
            }
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
        let (institution_code, _) = institution_profile::<T>(&institution)
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

        let threshold = fixed_governance_pass_threshold(&institution_code)
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        let admins_len =
            <votingengine::Pallet<T>>::snapshot_admins_len(proposal_id, institution.clone())
                .ok_or(votingengine::Error::<T>::InvalidInstitution)?;

        if tally.yes >= threshold {
            return Self::finalize_joint_institution_vote(proposal_id, institution, true);
        }
        let casted_votes = tally.yes.saturating_add(tally.no);
        let remaining_admins = admins_len.saturating_sub(casted_votes);
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
                return Self::advance_joint_to_referendum(proposal_id);
            }
            return Ok(());
        }
        Self::advance_joint_to_referendum(proposal_id)
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
        Self::advance_joint_to_referendum(proposal_id)
    }

    fn advance_joint_to_referendum(proposal_id: u64) -> DispatchResult {
        let now = <frame_system::Pallet<T>>::block_number();
        let referendum_end = now.saturating_add(Self::referendum_stage_duration());
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
                    proposal.end = referendum_end;
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
                <votingengine::Pallet<T>>::schedule_proposal_expiry(proposal_id, referendum_end)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::release_internal_proposal_mutexes(proposal_id);

            <votingengine::Pallet<T>>::emit_proposal_advanced_to_referendum(
                proposal_id,
                referendum_end,
                eligible_total,
            );
            TransactionOutcome::Commit(Ok(()))
        })
    }
}
