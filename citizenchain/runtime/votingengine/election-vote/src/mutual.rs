//! 互选入口。
//!
//! 互选的选民快照通常来自机构现任成员/admins。当前由调用方传入并冻结，
//! 后续可接业务 provider 自动生成。

use frame_support::pallet_prelude::DispatchResult;

use crate::pallet::{Config, MaxElectionOfficeCodeOf, Pallet};

impl<T: Config> Pallet<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn do_create_mutual_election(
        who: T::AccountId,
        organizer_code: votingengine::InstitutionCode,
        organizer: T::AccountId,
        target_code: votingengine::InstitutionCode,
        target: T::AccountId,
        office_code: frame_support::pallet_prelude::BoundedVec<u8, MaxElectionOfficeCodeOf<T>>,
        rule_id: u32,
        seat_count: u16,
        term_start: u32,
        term_end: u32,
        candidates: sp_std::vec::Vec<T::AccountId>,
        voters: sp_std::vec::Vec<T::AccountId>,
    ) -> Result<u64, sp_runtime::DispatchError> {
        Self::do_create_election(
            who,
            crate::types::ElectionMode::Mutual,
            organizer_code,
            organizer,
            target_code,
            target,
            office_code,
            rule_id,
            seat_count,
            term_start,
            term_end,
            candidates,
            voters,
        )
    }

    pub fn do_cast_mutual_vote(
        who: T::AccountId,
        proposal_id: u64,
        candidate: T::AccountId,
    ) -> DispatchResult {
        Self::do_cast_election_vote(
            who,
            proposal_id,
            votingengine::STAGE_ELECTION_MUTUAL,
            candidate,
        )
    }
}
