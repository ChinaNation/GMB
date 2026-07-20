//! 内部投票写票、提前判定和超时终结。

use super::*;

impl<T: Config> Pallet<T> {
    pub fn do_internal_vote(who: T::AccountId, proposal_id: u64, approve: bool) -> DispatchResult {
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
        let (eligible, eligible_total) = if let Some(actor_cid_number) = proposal.actor_cid_number {
            let subject = ProposalSubject::InstitutionCid(actor_cid_number);
            (
                <votingengine::Pallet<T>>::is_effective_voter_in_snapshot(
                    proposal_id,
                    subject.clone(),
                    &who,
                ),
                <votingengine::Pallet<T>>::effective_voters_len(proposal_id, subject)
                    .ok_or(votingengine::Error::<T>::MissingVoterSnapshot)?,
            )
        } else {
            let personal_account = proposal
                .execution_account
                .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
            let subject = ProposalSubject::PersonalAccount(personal_account);
            (
                <votingengine::Pallet<T>>::is_admin_in_snapshot(proposal_id, subject.clone(), &who),
                <votingengine::Pallet<T>>::snapshot_admins_len(proposal_id, subject)
                    .ok_or(votingengine::Error::<T>::MissingAdminSnapshot)?,
            )
        };
        ensure!(eligible, votingengine::Error::<T>::NoPermission);

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
            let casted = tally.yes.saturating_add(tally.no);
            let remaining = eligible_total.saturating_sub(casted);
            if tally.yes.saturating_add(remaining) < threshold {
                <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)?;
            }
        }

        Ok(())
    }

    pub fn do_finalize_internal_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
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
