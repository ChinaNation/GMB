//! 管理员治理模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as AdminsOriginGov;
use crate::{
    reserve_pallet_id_to_bytes, BlockNumberFor, Call, Config, CurrentAdmins, InstitutionPalletId,
    Pallet, CHINA_CB, ORG_PRC,
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
        let stale_new_admin: T::AccountId = frame_benchmarking::account("stale_new_admin", 0, 0);

        assert!(AdminsOriginGov::<T>::propose_admin_replacement(
            RawOrigin::Signed(proposer.clone()).into(),
            ORG_PRC,
            institution,
            old_admin.clone(),
            stale_new_admin,
        )
        .is_ok());

        let end = voting_engine_system::Pallet::<T>::proposals(0)
            .expect("stale benchmark proposal should exist")
            .end;
        let one: BlockNumberFor<T> = 1u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(end.saturating_add(one));
        assert!(voting_engine_system::Pallet::<T>::finalize_proposal(
            RawOrigin::Signed(proposer.clone()).into(),
            0,
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

        assert!(voting_engine_system::Pallet::<T>::get_proposal_data(1).is_some());
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
            old_admin.clone(),
            new_admin,
        )
        .is_ok());

        for i in 0..5 {
            let voter = prc_admin::<T>(i);
            assert!(AdminsOriginGov::<T>::vote_admin_replacement(
                RawOrigin::Signed(voter).into(),
                0,
                true,
            )
            .is_ok());
        }

        #[extrinsic_call]
        vote_admin_replacement(RawOrigin::Signed(final_voter), 0, true);
    }

    #[benchmark]
    fn execute_admin_replacement() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let old_admin = prc_admin::<T>(1);
        let caller = prc_admin::<T>(6);
        let final_voter = prc_admin::<T>(5);
        let new_admin: T::AccountId = frame_benchmarking::account("new_admin", 2, 0);
        let temp_admin: T::AccountId = frame_benchmarking::account("temp_admin", 0, 0);

        assert!(AdminsOriginGov::<T>::propose_admin_replacement(
            RawOrigin::Signed(proposer).into(),
            ORG_PRC,
            institution,
            old_admin.clone(),
            new_admin,
        )
        .is_ok());

        for i in 0..5 {
            let voter = prc_admin::<T>(i);
            assert!(AdminsOriginGov::<T>::vote_admin_replacement(
                RawOrigin::Signed(voter).into(),
                0,
                true,
            )
            .is_ok());
        }

        CurrentAdmins::<T>::mutate(institution, |maybe_admins| {
            let admins = maybe_admins
                .as_mut()
                .expect("benchmark institution should exist");
            let old_pos = admins
                .iter()
                .position(|admin| admin == &old_admin)
                .expect("benchmark old_admin should exist");
            admins[old_pos] = temp_admin.clone();
        });

        assert!(AdminsOriginGov::<T>::vote_admin_replacement(
            RawOrigin::Signed(final_voter).into(),
            0,
            true,
        )
        .is_ok());

        CurrentAdmins::<T>::mutate(institution, |maybe_admins| {
            let admins = maybe_admins
                .as_mut()
                .expect("benchmark institution should exist");
            let temp_pos = admins
                .iter()
                .position(|admin| admin == &temp_admin)
                .expect("temporary benchmark admin marker should exist");
            admins[temp_pos] = old_admin.clone();
        });

        assert!(voting_engine_system::Pallet::<T>::get_proposal_data(0).is_some());

        #[extrinsic_call]
        execute_admin_replacement(RawOrigin::Signed(caller), 0);
    }
}
