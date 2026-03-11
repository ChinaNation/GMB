//! GRANDPA 密钥治理模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_core::Pair;
use sp_runtime::traits::{SaturatedConversion, Saturating};
use voting_engine_system::InternalVoteEngine;

use crate::{
    pallet, reserve_pallet_id_to_bytes, ActiveProposalByInstitution, BlockNumberFor, Call, Config,
    InstitutionPalletId, Pallet, ProposalActions, CHINA_CB,
};

use crate::Pallet as GrandpaKeyGov;

fn decode_account<T: pallet::Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

fn prc_institution() -> InstitutionPalletId {
    reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id).expect("PRC institution should be valid")
}

fn prc_admin<T: pallet::Config>(index: usize) -> T::AccountId {
    decode_account::<T>(CHINA_CB[1].admins[index])
}

fn seeded_public_key(seed: u8) -> [u8; 32] {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = seed;
    sp_core::ed25519::Pair::from_seed(&seed_bytes).public().0
}

fn propose<T: pallet::Config>(
    institution: InstitutionPalletId,
    proposer: T::AccountId,
    new_key: [u8; 32],
) {
    assert!(GrandpaKeyGov::<T>::propose_replace_grandpa_key(
        RawOrigin::Signed(proposer).into(),
        institution,
        new_key,
    )
    .is_ok());
}

fn pass_proposal<T: pallet::Config>(proposal_id: u64) {
    for i in 0..6 {
        let voter = prc_admin::<T>(i);
        assert!(T::InternalVoteEngine::cast_internal_vote(voter, proposal_id, true).is_ok());
    }
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn propose_replace_grandpa_key() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let new_key = seeded_public_key(11);

        #[extrinsic_call]
        propose_replace_grandpa_key(RawOrigin::Signed(proposer), institution, new_key);

        assert_eq!(ActiveProposalByInstitution::<T>::get(institution), Some(0));
        assert!(ProposalActions::<T>::contains_key(0));
    }

    #[benchmark]
    fn vote_replace_grandpa_key() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let final_voter = prc_admin::<T>(5);
        let new_key = seeded_public_key(12);

        propose::<T>(institution, proposer, new_key);

        for i in 0..5 {
            let voter = prc_admin::<T>(i);
            assert!(T::InternalVoteEngine::cast_internal_vote(voter, 0, true).is_ok());
        }

        #[extrinsic_call]
        vote_replace_grandpa_key(RawOrigin::Signed(final_voter), 0, true);
    }

    #[benchmark]
    fn execute_replace_grandpa_key() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let caller = prc_admin::<T>(6);
        let new_key = seeded_public_key(13);

        propose::<T>(institution, proposer, new_key);
        pass_proposal::<T>(0);

        #[extrinsic_call]
        execute_replace_grandpa_key(RawOrigin::Signed(caller), 0);

        assert!(!ProposalActions::<T>::contains_key(0));
    }

    #[benchmark]
    fn cancel_stale_replace_grandpa_key() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let caller = prc_admin::<T>(1);
        let new_key = seeded_public_key(14);

        propose::<T>(institution, proposer, new_key);

        let one: BlockNumberFor<T> = 1u32.saturated_into();
        let stale_block = T::StaleProposalLifetime::get().saturating_add(one);
        frame_system::Pallet::<T>::set_block_number(stale_block);

        #[extrinsic_call]
        cancel_stale_replace_grandpa_key(RawOrigin::Signed(caller), 0);

        assert!(!ProposalActions::<T>::contains_key(0));
    }

    #[benchmark]
    fn cancel_failed_replace_grandpa_key() {
        let institution = prc_institution();
        let proposer = prc_admin::<T>(0);
        let caller = prc_admin::<T>(1);
        let new_key = seeded_public_key(15);

        propose::<T>(institution, proposer, new_key);
        pass_proposal::<T>(0);

        // 将 old_key 篡改为不存在的 authority，制造“已通过但不可执行”场景。
        ProposalActions::<T>::mutate(0, |action| {
            if let Some(action) = action {
                action.old_key = seeded_public_key(250);
            }
        });

        #[extrinsic_call]
        cancel_failed_replace_grandpa_key(RawOrigin::Signed(caller), 0);

        assert!(!ProposalActions::<T>::contains_key(0));
    }
}
