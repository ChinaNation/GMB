//! 管理员治理模块 Benchmark 定义。
//!
//! Phase 2 整改后投票统一由 `voting-engine::internal_vote` 公开 call 承担,
//! 本模块不再保留独立投票 extrinsic。Benchmark 只覆盖提案创建和
//! 执行重试两条路径。

#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as AdminsChange;
use crate::{
    reserve_pallet_id_to_bytes, BlockNumberFor, Call, Config, InstitutionPalletId, Pallet,
    CHINA_CB, ORG_PRC,
};
use codec::Decode;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::{SaturatedConversion, Saturating};

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn prc_institution() -> InstitutionPalletId {
    reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id).expect("PRC institution should be valid")
}

fn prc_admin<T: Config>(index: usize) -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].duoqian_admins[index])
}

fn last_proposal_id<T: Config>() -> u64 {
    voting_engine::Pallet::<T>::next_proposal_id().saturating_sub(1)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_admin_replacement() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let old_admin = prc_admin::<T>(1);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 0, 0);
        let stale_new_admin: T::AccountId = frame_benchmarking::account("stale_new_admin", 0, 0);

        // 先发一个"陈旧"提案,让它自然超时被终结,验证新提案不会冲突。
        assert!(AdminsChange::<T>::propose_admin_replacement(
            RawOrigin::Signed(proposer.clone()).into(),
            ORG_PRC,
            institution,
            old_admin.clone(),
            stale_new_admin,
        )
        .is_ok());

        let stale_proposal_id = last_proposal_id::<T>();
        let end = voting_engine::Pallet::<T>::proposals(stale_proposal_id)
            .expect("stale benchmark proposal should exist")
            .end;
        let one: BlockNumberFor<T> = 1u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(end.saturating_add(one));
        assert!(voting_engine::Pallet::<T>::finalize_proposal(
            RawOrigin::Signed(proposer.clone()).into(),
            stale_proposal_id,
        )
        .is_ok());

        #[extrinsic_call]
        propose_admin_replacement(
            RawOrigin::Signed(proposer),
            ORG_PRC,
            institution,
            old_admin,
            new_admin,
        );

        let proposal_id = last_proposal_id::<T>();
        assert!(voting_engine::Pallet::<T>::get_proposal_data(proposal_id).is_some());
    }

    // execute_admin_replacement benchmark 已废弃: 该 wrapper extrinsic 已统一到
    // VotingEngine::retry_passed_proposal,benchmark 由 voting-engine 自身覆盖。
}
