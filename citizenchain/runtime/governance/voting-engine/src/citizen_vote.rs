use crate::{
    pallet::{CitizenTallies, CitizenVotesByBindingId, Config, Error, Event, Pallet},
    PROPOSAL_KIND_JOINT, STAGE_CITIZEN, STATUS_PASSED, STATUS_REJECTED,
};
use frame_support::{ensure, pallet_prelude::DispatchResult};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VoteCredentialCleanup {
    pub removed: u32,
    pub loops: u32,
    pub has_remaining: bool,
}

impl VoteCredentialCleanup {
    pub const fn done() -> Self {
        Self {
            removed: 0,
            loops: 0,
            has_remaining: false,
        }
    }
}

pub trait SfidEligibility<AccountId, Hash> {
    fn is_eligible(binding_id: &Hash, who: &AccountId) -> bool;
    fn verify_and_consume_vote_credential(
        binding_id: &Hash,
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
    ) -> bool;

    /// 清理某个联合/公民提案对应的投票凭证防重放状态。
    /// 默认给兼容实现保留一次性清理入口；生产路径优先走分块清理。
    fn cleanup_vote_credentials(_proposal_id: u64) {}

    /// 分块清理某个提案维度下的投票凭证，避免单次 clear_prefix 无界增长。
    fn cleanup_vote_credentials_chunk(proposal_id: u64, _limit: u32) -> VoteCredentialCleanup {
        Self::cleanup_vote_credentials(proposal_id);
        let _ = proposal_id;
        VoteCredentialCleanup::done()
    }
}

impl<AccountId, Hash> SfidEligibility<AccountId, Hash> for () {
    fn is_eligible(_binding_id: &Hash, _who: &AccountId) -> bool {
        false
    }

    fn verify_and_consume_vote_credential(
        _binding_id: &Hash,
        _who: &AccountId,
        _proposal_id: u64,
        _nonce: &[u8],
        _signature: &[u8],
    ) -> bool {
        false
    }
}

pub fn is_citizen_vote_passed(yes_votes: u64, eligible_total: u64) -> bool {
    // 中文注释：公民投票必须严格”大于 50%”才算通过，恰好一半不通过。
    if eligible_total == 0 {
        return false;
    }
    yes_votes.saturating_mul(100) > eligible_total.saturating_mul(50)
}

/// 公民投票是否已注定无法通过：反对票 ≥ 50% 时，赞成票不可能严格 > 50%。
pub fn is_citizen_vote_rejected(no_votes: u64, eligible_total: u64) -> bool {
    if eligible_total == 0 {
        return false;
    }
    no_votes.saturating_mul(100) >= eligible_total.saturating_mul(50)
}

impl<T: Config> Pallet<T> {
    /// 公民投票执行：由外部 SFID 系统判定资格，链上负责去重计票。
    pub(crate) fn do_citizen_vote(
        who: T::AccountId,
        proposal_id: u64,
        binding_id: T::Hash,
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
        ensure!(
            T::SfidEligibility::is_eligible(&binding_id, &who),
            Error::<T>::SfidNotEligible
        );

        ensure!(
            !CitizenVotesByBindingId::<T>::contains_key(proposal_id, binding_id),
            Error::<T>::AlreadyVoted
        );
        // 中文注释：资格校验只证明“这个人能投”，这里还要消费一次性投票凭证来阻止离线重放。
        ensure!(
            T::SfidEligibility::verify_and_consume_vote_credential(
                &binding_id,
                &who,
                proposal_id,
                nonce.as_slice(),
                signature.as_slice()
            ),
            Error::<T>::InvalidSfidVoteCredential
        );

        CitizenVotesByBindingId::<T>::insert(proposal_id, binding_id, approve);
        let tally = CitizenTallies::<T>::mutate(proposal_id, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
            *tally
        });

        Self::deposit_event(Event::<T>::CitizenVoteCast {
            proposal_id,
            who,
            binding_id,
            approve,
        });

        if is_citizen_vote_passed(tally.yes, proposal.citizen_eligible_total) {
            // 中文注释：赞成票严格 > 50%，提前通过。
            Self::set_status_and_emit(proposal_id, STATUS_PASSED)?;
        } else if is_citizen_vote_rejected(tally.no, proposal.citizen_eligible_total) {
            // 中文注释：反对票 ≥ 50%，赞成票不可能再严格过半，提前否决。
            // 30 天超时只是兜底，不应让注定失败的提案空等。
            Self::set_status_and_emit(proposal_id, STATUS_REJECTED)?;
        }

        Ok(())
    }

    /// 公民投票超时处理：
    /// - 按 >50% 规则计算是否通过；
    /// - 未达到阈值则否决。
    pub(crate) fn do_finalize_citizen_timeout(
        proposal: &crate::Proposal<frame_system::pallet_prelude::BlockNumberFor<T>>,
        proposal_id: u64,
    ) -> DispatchResult {
        // 中文注释：公民投票超时后只看最终 yes 是否严格过半，不存在“弃权自动通过”。
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
