//! 普选入口。
//!
//! 普选的职位、任期、候选来源、选民范围由业务模块解释后传入。
//! 本文件只把这些数据固化成快照并创建 election-vote 提案。

use frame_support::pallet_prelude::DispatchResult;

use crate::pallet::{Config, MaxElectionOfficeCodeOf, Pallet};

impl<T: Config> Pallet<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn do_create_popular_election(
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
        population_scope: votingengine::PopulationScope,
        candidates: sp_std::vec::Vec<T::AccountId>,
        voters: sp_std::vec::Vec<T::AccountId>,
    ) -> Result<u64, sp_runtime::DispatchError> {
        Self::do_create_election(
            who,
            crate::types::ElectionMode::Popular,
            organizer_code,
            organizer,
            target_code,
            target,
            office_code,
            rule_id,
            seat_count,
            term_start,
            term_end,
            Some(population_scope),
            candidates,
            voters,
        )
    }

    pub fn do_cast_popular_vote(
        who: T::AccountId,
        proposal_id: u64,
        candidate: T::AccountId,
    ) -> DispatchResult {
        Self::do_cast_election_vote(
            who,
            proposal_id,
            votingengine::STAGE_ELECTION_POPULAR,
            candidate,
        )
    }
}
