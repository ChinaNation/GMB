//! 决议销毁模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use voting_engine_system::InternalVoteEngine;

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

    #[benchmark]
    fn vote_destroy() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let final_voter = prc_admin::<T>(5);
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

        for i in 0..5 {
            let voter = prc_admin::<T>(i);
            assert!(
                T::InternalVoteEngine::cast_internal_vote(voter, proposal_id, true).is_ok()
            );
        }

        #[extrinsic_call]
        vote_destroy(RawOrigin::Signed(final_voter), proposal_id, true);

        // 执行完成后提案数据仍在 voting-engine-system 中（由统一清理流程处理）。
    }

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

        for i in 0..6 {
            let voter = prc_admin::<T>(i);
            assert!(
                T::InternalVoteEngine::cast_internal_vote(voter, proposal_id, true).is_ok()
            );
        }

        #[extrinsic_call]
        execute_destroy(RawOrigin::Signed(caller), proposal_id);

        // 执行完成后提案数据仍在 voting-engine-system 中（由统一清理流程处理）。
    }
}
