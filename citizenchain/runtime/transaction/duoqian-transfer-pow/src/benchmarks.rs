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

use crate::Pallet as DuoqianTransferPow;
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

    /// execute_transfer benchmark:触发自动执行失败,然后补足余额手动重试成功。
    /// 投票阶段直接调用 `VotingEngine::internal_vote` 统一入口累计赞成票。
    #[benchmark]
    fn execute_transfer() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let executor = prc_admin::<T>(0);
        let beneficiary = beneficiary_account::<T>();
        // 使用较大金额，使自动执行因余额不足失败，然后通过手动执行重试
        let amount: BalanceOf<T> = 500u128.saturated_into();
        let initial_balance: BalanceOf<T> = 600u128.saturated_into();

        let inst_account = institution_account::<T>(institution);
        let _ = T::Currency::deposit_creating(&inst_account, initial_balance);

        assert!(DuoqianTransferPow::<T>::propose_transfer(
            RawOrigin::Signed(proposer).into(),
            ORG_PRC,
            institution,
            beneficiary,
            amount,
            BoundedVec::default(),
        )
        .is_ok());
        let pid = last_proposal_id::<T>();

        // 投票通过（自动执行可能因余额不足失败，这不影响提案状态）。
        // Phase 3: 走 VotingEngine 统一入口,benchmark 不测验签路径。
        for i in 0..6 {
            let voter = prc_admin::<T>(i);
            assert!(voting_engine::Pallet::<T>::internal_vote(
                RawOrigin::Signed(voter).into(),
                pid,
                true,
            )
            .is_ok());
        }

        // 补充余额后手动执行
        let top_up: BalanceOf<T> = 1_000_000u128.saturated_into();
        let _ = T::Currency::deposit_creating(&inst_account, top_up);

        #[extrinsic_call]
        execute_transfer(RawOrigin::Signed(executor), pid);
    }
}
