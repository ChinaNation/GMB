//! 联合投票 — 联合公投阶段。
//!
//! 联合内部投票阶段非全票通过或超时进入此阶段,SFID 持有者按 >50% 严格多数投票。
//!
//! 业务函数挂在 `super::Pallet<T>` 上,在 super(lib.rs)的 #[pallet::call]
//! `cast_referendum` extrinsic 与 `JointProposalFinalizer::finalize_jointreferendum_timeout`
//! trait 实现中被调用。
//!
//! `SfidEligibility` trait + `VoteCredentialCleanup` struct 在
//! `votingengine::traits`(用作 `votingengine::Config` 的 type bound)。

use frame_support::{ensure, pallet_prelude::DispatchResult};

use votingengine::{Proposal, SfidEligibility, PROPOSAL_KIND_JOINT, STATUS_PASSED};

use super::pallet::{Config, Error, Event, Pallet, ReferendumTallies, ReferendumVotesByBindingId};
use super::{is_jointreferendum_vote_passed, is_jointreferendum_vote_rejected};

impl<T: Config> Pallet<T> {
    /// 联合公投:由外部 SFID 系统判定资格,链上去重计票。
    /// 中文注释:签发身份按机构 admins 验签,作用域名称只参与 payload 防串用。
    pub fn do_jointreferendum_vote(
        who: T::AccountId,
        proposal_id: u64,
        binding_id: T::Hash,
        nonce: votingengine::pallet::VoteNonceOf<T>,
        signature: votingengine::pallet::VoteSignatureOf<T>,
        issuer_sfid_number: frame_support::BoundedVec<
            u8,
            frame_support::pallet_prelude::ConstU32<128>,
        >,
        issuer_main_account: T::AccountId,
        signer_pubkey: [u8; 32],
        scope_province_name: frame_support::BoundedVec<
            u8,
            frame_support::pallet_prelude::ConstU32<64>,
        >,
        scope_city_name: frame_support::BoundedVec<u8, frame_support::pallet_prelude::ConstU32<64>>,
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
        ensure!(
            proposal.citizen_eligible_total > 0,
            Error::<T>::CitizenEligibleTotalNotSet
        );
        ensure!(
            <T as votingengine::Config>::SfidEligibility::is_eligible(&binding_id, &who),
            Error::<T>::SfidNotEligible
        );

        ensure!(
            !ReferendumVotesByBindingId::<T>::contains_key(proposal_id, binding_id),
            votingengine::Error::<T>::AlreadyVoted
        );
        ensure!(
            <T as votingengine::Config>::SfidEligibility::verify_and_consume_vote_credential(
                &binding_id,
                &who,
                proposal_id,
                nonce.as_slice(),
                signature.as_slice(),
                issuer_sfid_number.as_slice(),
                &issuer_main_account,
                &signer_pubkey,
                scope_province_name.as_slice(),
                scope_city_name.as_slice(),
            ),
            Error::<T>::InvalidSfidVoteCredential
        );

        ReferendumVotesByBindingId::<T>::insert(proposal_id, binding_id, approve);
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
            binding_id,
            approve,
        });

        if is_jointreferendum_vote_passed(tally.yes, proposal.citizen_eligible_total) {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)?;
        } else if is_jointreferendum_vote_rejected(tally.no, proposal.citizen_eligible_total) {
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
        let status = if is_jointreferendum_vote_passed(tally.yes, proposal.citizen_eligible_total) {
            STATUS_PASSED
        } else {
            votingengine::STATUS_REJECTED
        };
        <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, status)
    }
}
