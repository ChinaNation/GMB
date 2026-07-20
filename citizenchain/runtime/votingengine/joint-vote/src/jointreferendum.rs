//! 联合投票 — 联合公投阶段。
//!
//! 联合内部投票阶段非全票通过或超时进入此阶段,链上公民身份持有者按 >50% 严格多数投票。
//!
//! 业务函数挂在 `super::Pallet<T>` 上,在 super(lib.rs)的 #[pallet::call]
//! `cast_referendum` extrinsic 与 `JointProposalFinalizer::finalize_jointreferendum_timeout`
//! trait 实现中被调用。
//!
use frame_support::{ensure, pallet_prelude::DispatchResult};

use votingengine::{Proposal, PROPOSAL_KIND_JOINT, STATUS_PASSED};

use super::pallet::{Config, Error, Event, Pallet, ReferendumTallies, ReferendumVotesByAccount};
use super::{is_jointreferendum_vote_passed, is_jointreferendum_vote_rejected};

impl<T: Config> Pallet<T> {
    /// 联合公投:直接读取链上公民身份资格,链上按账户去重计票。
    pub fn do_jointreferendum_vote(
        who: T::AccountId,
        proposal_id: u64,
        approve: bool,
    ) -> DispatchResult {
        let proposal = <votingengine::Pallet<T>>::ensure_open_proposal(proposal_id)?;

        ensure!(
            proposal.kind == PROPOSAL_KIND_JOINT,
            votingengine::Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == votingengine::STAGE_REFERENDUM,
            votingengine::Error::<T>::InvalidProposalStage
        );
        let eligible_total = <votingengine::Pallet<T>>::population_eligible_total_of(proposal_id)
            .ok_or(Error::<T>::CitizenEligibleTotalNotSet)?;
        ensure!(eligible_total > 0, Error::<T>::CitizenEligibleTotalNotSet);
        ensure!(
            <votingengine::Pallet<T>>::can_vote_at_population_snapshot(proposal_id, &who),
            Error::<T>::CitizenNotEligible
        );

        ensure!(
            !ReferendumVotesByAccount::<T>::contains_key(proposal_id, &who),
            votingengine::Error::<T>::AlreadyVoted
        );
        let current_tally = ReferendumTallies::<T>::get(proposal_id);
        ensure!(
            current_tally.yes.saturating_add(current_tally.no) < eligible_total,
            Error::<T>::ReferendumSnapshotExhausted
        );

        ReferendumVotesByAccount::<T>::insert(proposal_id, &who, approve);
        let tally = ReferendumTallies::<T>::mutate(proposal_id, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
            *tally
        });

        Self::deposit_event(Event::<T>::ReferendumVoteCast {
            proposal_id,
            who,
            approve,
        });

        if is_jointreferendum_vote_passed(tally.yes, eligible_total) {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)?;
        } else if is_jointreferendum_vote_rejected(tally.no, eligible_total) {
            <votingengine::Pallet<T>>::set_status_and_emit(
                proposal_id,
                votingengine::STATUS_REJECTED,
            )?;
        }

        Ok(())
    }

    /// 联合公投超时结算:按 >50% 规则,未达阈值否决。
    pub fn do_finalize_jointreferendum_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == votingengine::STAGE_REFERENDUM,
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
        let tally = ReferendumTallies::<T>::get(proposal_id);
        let eligible_total = <votingengine::Pallet<T>>::population_eligible_total_of(proposal_id)
            .ok_or(Error::<T>::CitizenEligibleTotalNotSet)?;
        let status = if is_jointreferendum_vote_passed(tally.yes, eligible_total) {
            STATUS_PASSED
        } else {
            votingengine::STATUS_REJECTED
        };
        <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, status)
    }
}
