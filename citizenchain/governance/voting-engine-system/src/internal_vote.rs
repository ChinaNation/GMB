#![allow(dead_code)]

use frame_support::{ensure, pallet_prelude::DispatchResult};
use sp_runtime::traits::{SaturatedConversion, Saturating};

use primitives::count_const::{
    NRC_INTERNAL_THRESHOLD,
    PRB_INTERNAL_THRESHOLD,
    PRC_INTERNAL_THRESHOLD,
    VOTING_DURATION_BLOCKS,
};

use crate::{
    pallet::{Config, Error, Event, InternalTallies, InternalVotesByAccount, Pallet, Proposals},
    Proposal,
    PROPOSAL_KIND_INTERNAL,
    STAGE_INTERNAL,
    STATUS_PASSED,
};

pub const ORG_NRC: u8 = 0;
pub const ORG_PRC: u8 = 1;
pub const ORG_PRB: u8 = 2;

pub fn is_valid_org(org: u8) -> bool {
    matches!(org, ORG_NRC | ORG_PRC | ORG_PRB)
}

pub fn org_pass_threshold(org: u8) -> Option<u32> {
    match org {
        ORG_NRC => Some(NRC_INTERNAL_THRESHOLD),
        ORG_PRC => Some(PRC_INTERNAL_THRESHOLD),
        ORG_PRB => Some(PRB_INTERNAL_THRESHOLD),
        _ => None,
    }
}

impl<T: Config> Pallet<T> {
    fn internal_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    pub(crate) fn do_create_internal_proposal(org: u8) -> DispatchResult {
        ensure!(is_valid_org(org), Error::<T>::InvalidInternalOrg);

        let id = Self::allocate_proposal_id();
        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: crate::STATUS_VOTING,
            internal_org: Some(org),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        Proposals::<T>::insert(id, proposal);
        Self::deposit_event(Event::<T>::ProposalCreated {
            proposal_id: id,
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            end,
        });
        Ok(())
    }

    pub(crate) fn do_internal_vote(
        who: T::AccountId,
        proposal_id: u64,
        approve: bool,
    ) -> DispatchResult {
        let mut proposal = Self::ensure_open_proposal(proposal_id)?;

        ensure!(proposal.kind == PROPOSAL_KIND_INTERNAL, Error::<T>::InvalidProposalKind);
        ensure!(proposal.stage == STAGE_INTERNAL, Error::<T>::InvalidProposalStage);
        ensure!(
            !InternalVotesByAccount::<T>::contains_key(proposal_id, &who),
            Error::<T>::AlreadyVoted
        );

        InternalVotesByAccount::<T>::insert(proposal_id, &who, approve);
        InternalTallies::<T>::mutate(proposal_id, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
        });

        Self::deposit_event(Event::<T>::InternalVoteCast {
            proposal_id,
            who,
            approve,
        });

        let org = proposal.internal_org.ok_or(Error::<T>::InvalidInternalOrg)?;
        let threshold = org_pass_threshold(org).ok_or(Error::<T>::InvalidInternalOrg)?;
        let tally = InternalTallies::<T>::get(proposal_id);
        if tally.yes >= threshold {
            proposal.status = STATUS_PASSED;
            Proposals::<T>::insert(proposal_id, proposal);
            Self::deposit_event(Event::<T>::ProposalFinalized {
                proposal_id,
                status: STATUS_PASSED,
            });
        }

        Ok(())
    }

    pub(crate) fn do_finalize_internal_timeout(proposal_id: u64) -> DispatchResult {
        let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
        ensure!(proposal.stage == STAGE_INTERNAL, Error::<T>::InvalidProposalStage);
        ensure!(proposal.status == crate::STATUS_VOTING, Error::<T>::ProposalAlreadyFinalized);
        ensure!(
            <frame_system::Pallet<T>>::block_number() > proposal.end,
            Error::<T>::VoteNotExpired
        );
        Self::set_status_and_emit(proposal_id, crate::STATUS_REJECTED)
    }
}
