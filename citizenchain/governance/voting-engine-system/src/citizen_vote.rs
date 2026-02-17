#![allow(dead_code)]

use frame_support::{ensure, pallet_prelude::DispatchResult};
use sp_runtime::traits::Hash;

use crate::{
    pallet::{CiicOf, CitizenTallies, CitizenVotesByCiic, Config, Error, Event, Pallet, Proposals},
    PROPOSAL_KIND_JOINT, STAGE_CITIZEN, STATUS_PASSED,
};

pub trait CiicEligibility<AccountId> {
    fn is_eligible(ciic: &[u8], who: &AccountId) -> bool;
    fn verify_and_consume_vote_credential(
        ciic: &[u8],
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
    ) -> bool;
}

impl<AccountId> CiicEligibility<AccountId> for () {
    fn is_eligible(_ciic: &[u8], _who: &AccountId) -> bool {
        false
    }

    fn verify_and_consume_vote_credential(
        _ciic: &[u8],
        _who: &AccountId,
        _proposal_id: u64,
        _nonce: &[u8],
        _signature: &[u8],
    ) -> bool {
        false
    }
}

pub fn is_citizen_vote_passed(yes_votes: u64, eligible_total: u64) -> bool {
    if eligible_total == 0 {
        return false;
    }
    yes_votes.saturating_mul(100) > eligible_total.saturating_mul(50)
}

impl<T: Config> Pallet<T> {
    /// 公民投票执行：由外部 CIIC 系统判定资格，链上负责去重计票。
    pub(crate) fn do_citizen_vote(
        who: T::AccountId,
        proposal_id: u64,
        ciic: CiicOf<T>,
        nonce: crate::pallet::VoteNonceOf<T>,
        signature: crate::pallet::VoteSignatureOf<T>,
        approve: bool,
    ) -> DispatchResult {
        let proposal = Self::ensure_open_proposal(proposal_id)?;

        ensure!(
            proposal.kind == PROPOSAL_KIND_JOINT,
            Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == STAGE_CITIZEN,
            Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.citizen_eligible_total > 0,
            Error::<T>::CitizenEligibleTotalNotSet
        );
        ensure!(!ciic.is_empty(), Error::<T>::EmptyCiic);
        ensure!(
            T::CiicEligibility::is_eligible(ciic.as_slice(), &who),
            Error::<T>::CiicNotEligible
        );

        let ciic_hash = T::Hashing::hash(ciic.as_slice());
        ensure!(
            !CitizenVotesByCiic::<T>::contains_key(proposal_id, ciic_hash),
            Error::<T>::AlreadyVoted
        );
        ensure!(
            T::CiicEligibility::verify_and_consume_vote_credential(
                ciic.as_slice(),
                &who,
                proposal_id,
                nonce.as_slice(),
                signature.as_slice()
            ),
            Error::<T>::InvalidCiicVoteCredential
        );

        CitizenVotesByCiic::<T>::insert(proposal_id, ciic_hash, approve);
        CitizenTallies::<T>::mutate(proposal_id, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
        });

        Self::deposit_event(Event::<T>::CitizenVoteCast {
            proposal_id,
            who,
            ciic_hash,
            approve,
        });

        let tally = CitizenTallies::<T>::get(proposal_id);
        let current = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
        if is_citizen_vote_passed(tally.yes, current.citizen_eligible_total) {
            Self::set_status_and_emit(proposal_id, STATUS_PASSED)?;
        }

        Ok(())
    }

    /// 公民投票超时处理：
    /// - 按 >50% 规则计算是否通过；
    /// - 未达到阈值则否决。
    pub(crate) fn do_finalize_citizen_timeout(proposal_id: u64) -> DispatchResult {
        let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
        ensure!(
            proposal.stage == STAGE_CITIZEN,
            Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.status == crate::STATUS_VOTING,
            Error::<T>::ProposalAlreadyFinalized
        );
        ensure!(
            <frame_system::Pallet<T>>::block_number() > proposal.end,
            Error::<T>::VoteNotExpired
        );
        let tally = CitizenTallies::<T>::get(proposal_id);
        let status = if is_citizen_vote_passed(tally.yes, proposal.citizen_eligible_total) {
            STATUS_PASSED
        } else {
            crate::STATUS_REJECTED
        };
        Self::set_status_and_emit(proposal_id, status)
    }
}
