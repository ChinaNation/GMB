//! 管理员治理模块 Benchmark 定义。
//!
//! 投票统一走 `votingengine::internal_vote`,本模块只覆盖提案创建和
//! 执行重试两条路径。

#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as AdminsChange;
use crate::{BlockNumberFor, Call, Config, Pallet, CHINA_CB};
use codec::Decode;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::{SaturatedConversion, Saturating};
use votingengine::types::PRC;

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn prc_institution<T: Config>() -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].main_account)
}

fn prc_admin<T: Config>(index: usize) -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].admins[index])
}

fn last_proposal_id<T: Config>() -> u64 {
    votingengine::Pallet::<T>::next_proposal_id().saturating_sub(1)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_admin_set_change() {
        let institution = prc_institution::<T>();
        let proposer = prc_admin::<T>(0);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 0, 0);
        let stale_new_admin: T::AccountId = frame_benchmarking::account("stale_new_admin", 0, 0);
        let account = crate::AdminAccounts::<T>::get(institution.clone())
            .expect("benchmark genesis account should exist");
        let threshold = votingengine::types::fixed_governance_pass_threshold(&PRC).unwrap_or(2);
        let mut stale_admins = account.admins.clone();
        stale_admins[1] = stale_new_admin;
        let mut admins = account.admins;
        admins[1] = new_admin;

        // 先发一个"陈旧"提案,让它自然超时被终结,验证新提案不会冲突。
        assert!(AdminsChange::<T>::propose_admin_set_change(
            RawOrigin::Signed(proposer.clone()).into(),
            PRC,
            institution.clone(),
            stale_admins,
            threshold,
        )
        .is_ok());

        let stale_proposal_id = last_proposal_id::<T>();
        let end = votingengine::Pallet::<T>::proposals(stale_proposal_id)
            .expect("stale benchmark proposal should exist")
            .end;
        let one: BlockNumberFor<T> = 1u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(end.saturating_add(one));
        assert!(votingengine::Pallet::<T>::finalize_proposal(
            RawOrigin::Signed(proposer.clone()).into(),
            stale_proposal_id,
        )
        .is_ok());

        #[extrinsic_call]
        propose_admin_set_change(
            RawOrigin::Signed(proposer),
            PRC,
            institution,
            admins,
            threshold,
        );

        let proposal_id = last_proposal_id::<T>();
        assert!(votingengine::Pallet::<T>::get_proposal_data(proposal_id).is_some());
    }
}
