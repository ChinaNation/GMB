//! 机构多签名地址转账模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use voting_engine_system::InternalVoteEngine;

use crate::Pallet as DuoqianTransferPow;
use crate::{
    institution_pallet_address, reserve_pallet_id_to_bytes, ActiveProposalByInstitution, BalanceOf,
    Call, Config, InstitutionPalletId, Pallet, ProposalActions, CHINA_CB, ORG_PRC,
};

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn prc_institution() -> InstitutionPalletId {
    reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id).expect("PRC institution should be valid")
}

fn prc_admin<T: Config>(index: usize) -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].admins[index])
}

fn institution_account<T: Config>(institution: InstitutionPalletId) -> T::AccountId {
    let raw = institution_pallet_address(institution).expect("institution account should exist");
    decode_account::<T>(raw)
}

fn beneficiary_account<T: Config>() -> T::AccountId {
    decode_account::<T>([99u8; 32])
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

        assert_eq!(ActiveProposalByInstitution::<T>::get(institution), Some(0));
        assert!(ProposalActions::<T>::contains_key(0));
    }

    #[benchmark]
    fn vote_transfer() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let final_voter = prc_admin::<T>(5);
        let beneficiary = beneficiary_account::<T>();
        let amount: BalanceOf<T> = 100u128.saturated_into();
        let top_up: BalanceOf<T> = 1_000_000u128.saturated_into();

        let institution_account = institution_account::<T>(institution);
        let _ = T::Currency::deposit_creating(&institution_account, top_up);

        assert!(DuoqianTransferPow::<T>::propose_transfer(
            RawOrigin::Signed(proposer).into(),
            ORG_PRC,
            institution,
            beneficiary,
            amount,
            BoundedVec::default(),
        )
        .is_ok());

        for i in 0..5 {
            let voter = prc_admin::<T>(i);
            assert!(T::InternalVoteEngine::cast_internal_vote(voter, 0, true).is_ok());
        }

        #[extrinsic_call]
        vote_transfer(RawOrigin::Signed(final_voter), 0, true);

        // 第 6 票达到阈值，转账自动执行，提案已清理
        assert!(!ProposalActions::<T>::contains_key(0));
    }
}
