#![allow(dead_code)]

use frame_support::{ensure, pallet_prelude::DispatchResult};
use sp_runtime::traits::{SaturatedConversion, Saturating};

use primitives::count_const::{
    JOINT_VOTE_PASS_THRESHOLD,
    NRC_JOINT_VOTE_WEIGHT,
    PRB_JOINT_VOTE_WEIGHT,
    PRC_JOINT_VOTE_WEIGHT,
    JOINT_VOTE_TOTAL,
    VOTING_DURATION_BLOCKS,
};
use primitives::reserve_nodes_const::{
    pallet_id_to_bytes as reserve_pallet_id_to_bytes,
    RESERVE_NODES,
};
use primitives::shengbank_nodes_const::{
    pallet_id_to_bytes as shengbank_pallet_id_to_bytes,
    SHENG_BANK_NODES,
};

use crate::{
    citizen_vote::CiicEligibility,
    pallet::{Config, Error, Event, JointTallies, JointVotesByInstitution, Pallet, Proposals},
    InstitutionPalletId,
    Proposal,
    PROPOSAL_KIND_JOINT,
    STAGE_JOINT,
    STATUS_PASSED,
};

fn str_to_pallet_id(s: &str) -> Option<InstitutionPalletId> {
    reserve_pallet_id_to_bytes(s)
}

fn str_to_shengbank_pallet_id(s: &str) -> Option<InstitutionPalletId> {
    shengbank_pallet_id_to_bytes(s)
}

fn nrc_pallet_id_bytes() -> InstitutionPalletId {
    reserve_pallet_id_to_bytes(RESERVE_NODES[0].pallet_id)
        .expect("NRC pallet_id must be 8 bytes")
}

pub fn is_valid_institution(id: InstitutionPalletId) -> bool {
    if id == nrc_pallet_id_bytes() {
        return true;
    }

    let in_prc = RESERVE_NODES
        .iter()
        .filter_map(|n| str_to_pallet_id(n.pallet_id))
        .any(|pid| pid == id);
    if in_prc {
        return true;
    }

    SHENG_BANK_NODES
        .iter()
        .filter_map(|n| str_to_shengbank_pallet_id(n.pallet_id))
        .any(|pid| pid == id)
}

pub fn institution_weight(id: InstitutionPalletId) -> Option<u32> {
    if id == nrc_pallet_id_bytes() {
        return Some(NRC_JOINT_VOTE_WEIGHT);
    }

    let in_prc = RESERVE_NODES
        .iter()
        .filter_map(|n| str_to_pallet_id(n.pallet_id))
        .any(|pid| pid == id);
    if in_prc {
        return Some(PRC_JOINT_VOTE_WEIGHT);
    }

    let in_prb = SHENG_BANK_NODES
        .iter()
        .filter_map(|n| str_to_shengbank_pallet_id(n.pallet_id))
        .any(|pid| pid == id);
    if in_prb {
        return Some(PRB_JOINT_VOTE_WEIGHT);
    }

    None
}

pub fn is_joint_unanimous(yes_weight: u32) -> bool {
    yes_weight >= JOINT_VOTE_PASS_THRESHOLD
}

impl<T: Config> Pallet<T> {
    /// 联合投票阶段时长（30天，按区块高度计）
    fn joint_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    /// 公民投票阶段时长（30天，按区块高度计）
    fn citizen_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    /// 创建联合投票提案：独立计算本阶段 30 天截止区块。
    pub(crate) fn do_create_joint_proposal() -> DispatchResult {
        let id = Self::allocate_proposal_id();
        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::joint_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_JOINT,
            status: crate::STATUS_VOTING,
            internal_org: None,
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        Proposals::<T>::insert(id, proposal);
        Self::deposit_event(Event::<T>::ProposalCreated {
            proposal_id: id,
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_JOINT,
            end,
        });
        Ok(())
    }

    pub(crate) fn do_submit_joint_institution_vote(
        proposal_id: u64,
        institution: InstitutionPalletId,
        internal_passed: bool,
    ) -> DispatchResult {
        let proposal = Self::ensure_open_proposal(proposal_id)?;

        ensure!(proposal.kind == PROPOSAL_KIND_JOINT, Error::<T>::InvalidProposalKind);
        ensure!(proposal.stage == STAGE_JOINT, Error::<T>::InvalidProposalStage);
        ensure!(is_valid_institution(institution), Error::<T>::InvalidInstitution);
        ensure!(
            !JointVotesByInstitution::<T>::contains_key(proposal_id, institution),
            Error::<T>::AlreadyVoted
        );

        JointVotesByInstitution::<T>::insert(proposal_id, institution, internal_passed);

        let weight = institution_weight(institution).ok_or(Error::<T>::InvalidInstitution)?;
        JointTallies::<T>::mutate(proposal_id, |tally| {
            if internal_passed {
                tally.yes = tally.yes.saturating_add(weight);
            } else {
                tally.no = tally.no.saturating_add(weight);
            }
        });

        Self::deposit_event(Event::<T>::JointInstitutionVoteCast {
            proposal_id,
            institution,
            internal_passed,
        });

        let tally = JointTallies::<T>::get(proposal_id);
        if is_joint_unanimous(tally.yes) {
            Self::set_status_and_emit(proposal_id, STATUS_PASSED)?;
            return Ok(());
        }

        if tally.yes.saturating_add(tally.no) >= JOINT_VOTE_TOTAL {
            Self::advance_joint_to_citizen(proposal_id)?;
        }

        Ok(())
    }

    /// 联合投票超时处理：
    /// - 若已全票通过，直接通过；
    /// - 否则进入公民投票阶段，并重新计算公民投票的 30 天时限。
    pub(crate) fn do_finalize_joint_timeout(proposal_id: u64) -> DispatchResult {
        let proposal = Proposals::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalNotFound)?;
        ensure!(proposal.stage == STAGE_JOINT, Error::<T>::InvalidProposalStage);
        ensure!(proposal.status == crate::STATUS_VOTING, Error::<T>::ProposalAlreadyFinalized);
        ensure!(
            <frame_system::Pallet<T>>::block_number() > proposal.end,
            Error::<T>::VoteNotExpired
        );

        let tally = JointTallies::<T>::get(proposal_id);
        if is_joint_unanimous(tally.yes) {
            return Self::set_status_and_emit(proposal_id, STATUS_PASSED);
        }
        Self::advance_joint_to_citizen(proposal_id)
    }

    /// 联合投票未全票通过时，进入公民投票并重新计算公民投票阶段的 30 天截止区块。
    fn advance_joint_to_citizen(proposal_id: u64) -> DispatchResult {
        let now = <frame_system::Pallet<T>>::block_number();
        let citizen_end = now.saturating_add(Self::citizen_stage_duration());
        let eligible_total = T::CiicEligibility::eligible_voter_count();

        Proposals::<T>::try_mutate(proposal_id, |maybe| -> DispatchResult {
            let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
            proposal.stage = crate::STAGE_CITIZEN;
            proposal.start = now;
            proposal.end = citizen_end;
            proposal.citizen_eligible_total = eligible_total;
            Ok(())
        })?;

        Self::deposit_event(Event::<T>::ProposalAdvancedToCitizen {
            proposal_id,
            citizen_end,
            eligible_total,
        });
        Ok(())
    }
}
