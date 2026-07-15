//! 互选入口。
//!
//! 互选的选民列表由业务调用方提交，但创建时必须与 admins provider 返回的
//! 目标机构完整管理员快照等长且逐成员一致，调用方不能删减或夹带账户。

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
            None,
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
