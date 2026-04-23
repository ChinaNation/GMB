//! 决议销毁模块 Benchmark 定义。
//!
//! Phase 2 整改后投票统一走 `voting-engine-system::internal_vote`,本模块不再有
//! `vote_destroy` extrinsic。benchmark 只覆盖"发起提案"和"任意人重试执行"两条路径。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use voting_engine_system::STATUS_PASSED;

use crate::Pallet as ResolutionDestroGov;
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

fn last_proposal_id<T: Config>() -> u64 {
    voting_engine_system::Pallet::<T>::next_proposal_id().saturating_sub(1)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_destroy() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let amount: BalanceOf<T> = 100u128.saturated_into();

        #[extrinsic_call]
        propose_destroy(
            RawOrigin::Signed(proposer.clone()),
            ORG_PRC,
            institution,
            amount,
        );

        let proposal_id = last_proposal_id::<T>();
        assert!(voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id).is_some());
    }

    /// `execute_destroy` benchmark:
    /// 1. 发起提案
    /// 2. 给机构账户充值以通过 ED 检查
    /// 3. 手动把提案推到 PASSED(绕开投票路径,benchmark 只测 execute)
    /// 4. 调 `execute_destroy` 完成补救执行
    #[benchmark]
    fn execute_destroy() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let caller = prc_admin::<T>(6);
        let amount: BalanceOf<T> = 100u128.saturated_into();
        let top_up: BalanceOf<T> = 1_000_000u128.saturated_into();

        assert!(ResolutionDestroGov::<T>::propose_destroy(
            RawOrigin::Signed(proposer).into(),
            ORG_PRC,
            institution,
            amount,
        )
        .is_ok());
        let proposal_id = last_proposal_id::<T>();

        let institution_account = institution_account::<T>(institution);
        let _ = T::Currency::deposit_creating(&institution_account, top_up);

        // 用引擎低级接口直接把提案推到 PASSED。
        assert!(
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_PASSED)
                .is_ok()
        );

        #[extrinsic_call]
        execute_destroy(RawOrigin::Signed(caller), proposal_id);
    }
}
