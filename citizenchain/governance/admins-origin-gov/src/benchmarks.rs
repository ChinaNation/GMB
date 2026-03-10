//! 管理员治理模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_runtime::traits::{SaturatedConversion, Saturating};
use voting_engine_system::InternalVoteEngine;

use crate::Pallet as AdminsOriginGov;
use crate::{
    reserve_pallet_id_to_bytes, ActiveProposalByInstitution, BlockNumberFor, Call, Config,
    InstitutionPalletId, Pallet, ProposalActions, CHINA_CB, ORG_PRC,
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

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_admin_replacement() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let old_admin = prc_admin::<T>(1);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 0, 0);

        #[extrinsic_call]
        propose_admin_replacement(
            RawOrigin::Signed(proposer),
            ORG_PRC,
            institution,
            old_admin,
            new_admin,
        );

        assert_eq!(ActiveProposalByInstitution::<T>::get(institution), Some(0));
        assert!(ProposalActions::<T>::contains_key(0));
    }

    #[benchmark]
    fn vote_admin_replacement() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let old_admin = prc_admin::<T>(1);
        let final_voter = prc_admin::<T>(5);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 1, 0);

        assert!(AdminsOriginGov::<T>::propose_admin_replacement(
            RawOrigin::Signed(proposer).into(),
            ORG_PRC,
            institution,
            old_admin,
            new_admin,
        )
        .is_ok());

        for i in 0..5 {
            let voter = prc_admin::<T>(i);
            assert!(T::InternalVoteEngine::cast_internal_vote(voter, 0, true).is_ok());
        }

        #[extrinsic_call]
        vote_admin_replacement(RawOrigin::Signed(final_voter), 0, true);

        assert!(!ProposalActions::<T>::contains_key(0));
    }

    #[benchmark]
    fn execute_admin_replacement() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let old_admin = prc_admin::<T>(1);
        let caller = prc_admin::<T>(6);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 2, 0);

        assert!(AdminsOriginGov::<T>::propose_admin_replacement(
            RawOrigin::Signed(proposer).into(),
            ORG_PRC,
            institution,
            old_admin,
            new_admin,
        )
        .is_ok());

        for i in 0..6 {
            let voter = prc_admin::<T>(i);
            assert!(T::InternalVoteEngine::cast_internal_vote(voter, 0, true).is_ok());
        }

        #[extrinsic_call]
        execute_admin_replacement(RawOrigin::Signed(caller), 0);

        assert!(!ProposalActions::<T>::contains_key(0));
    }

    #[benchmark]
    fn cancel_stale_proposal() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let old_admin = prc_admin::<T>(1);
        let caller = prc_admin::<T>(2);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 3, 0);

        assert!(AdminsOriginGov::<T>::propose_admin_replacement(
            RawOrigin::Signed(proposer).into(),
            ORG_PRC,
            institution,
            old_admin,
            new_admin,
        )
        .is_ok());

        let one: BlockNumberFor<T> = 1u32.saturated_into();
        let stale_block = T::StaleProposalLifetime::get().saturating_add(one);
        frame_system::Pallet::<T>::set_block_number(stale_block);

        #[extrinsic_call]
        cancel_stale_proposal(RawOrigin::Signed(caller), 0);

        assert!(!ProposalActions::<T>::contains_key(0));
    }
}
