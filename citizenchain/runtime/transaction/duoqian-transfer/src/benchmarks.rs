//! 机构多签名地址转账模块 Benchmark 定义。
//!
//! Phase 3(2026-04-22)「投票引擎统一入口整改」:
//! 本 pallet 的 `vote_X` / `finalize_X` 已物理删除,所有管理员投票一律通过
//! `VotingEngine::internal_vote`(9.0)。本文件只保留 `propose_transfer`
//! 和 `execute_transfer` 两个业务动作的 benchmark;投票 weight 全部归入
//! voting-engine pallet 自身的 benchmark,业务端无需重复覆盖。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;

use crate::{
    institution_pallet_address, reserve_pallet_id_to_bytes, BalanceOf, Call, Config,
    InstitutionPalletId, Pallet, CHINA_CB, ORG_PRC,
};

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn prc_institution() -> InstitutionPalletId {
    reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id).expect("PRC institution should be valid")
}

fn prc_admin<T: Config>(index: usize) -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].duoqian_admins[index])
}

fn institution_account<T: Config>(institution: InstitutionPalletId) -> T::AccountId {
    let raw = institution_pallet_address(institution).expect("institution account should exist");
    decode_account::<T>(raw)
}

fn beneficiary_account<T: Config>() -> T::AccountId {
    decode_account::<T>([99u8; 32])
}

fn last_proposal_id<T: Config>() -> u64 {
    voting_engine::Pallet::<T>::next_proposal_id().saturating_sub(1)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_transfer() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let beneficiary = beneficiary_account::<T>();
        let amount: BalanceOf<T> = 100u128.saturated_into();
        let top_up: BalanceOf<T> = 1_000_000u128.saturated_into();

        let institution_account = institution_account::<T>(institution);
        let _ = T::Currency::deposit_creating(&institution_account, top_up);

        #[extrinsic_call]
        propose_transfer(
            RawOrigin::Signed(proposer.clone()),
            ORG_PRC,
            institution,
            beneficiary,
            amount,
            BoundedVec::default(),
        );

        let pid = last_proposal_id::<T>();
        assert!(voting_engine::Pallet::<T>::get_proposal_data(pid).is_some());
    }

    // execute_transfer / execute_safety_fund_transfer / execute_sweep_to_main
    // benchmark 已废弃: 三个 wrapper extrinsic 已统一到
    // VotingEngine::retry_passed_proposal,benchmark 由 voting-engine 自身覆盖。
}
