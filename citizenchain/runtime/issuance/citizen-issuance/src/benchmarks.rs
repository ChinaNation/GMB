//! 公民发行模块 benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use citizen_identity::OnVotingIdentityRegistered;
use codec::Decode;
use frame_benchmarking::v2::*;
use sp_runtime::traits::Hash;

use crate::pallet::{AccountRewarded, Config, IdentityRewardClaimed, Pallet, RewardedCount};

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn on_voting_identity_registered() {
        let who = decode_account::<T>([7u8; 32]);
        let cid_number = primitives::cid::generator::generate_cid_number(
            primitives::cid::generator::GenerateCidNumberInput {
                account_pubkey: "bench-0001",
                p1: "1",
                province_code: "GD",
                province_name: "广东省",
                city_code: "001",
                city_name: "荔湾市",
                year: "2026",
                institution: "CTZN",
            },
        )
        .expect("citizen cid should generate");
        let cid_number = cid_number.as_bytes();
        let cid_number_hash = T::Hashing::hash(cid_number);

        #[block]
        {
            <Pallet<T> as OnVotingIdentityRegistered<T::AccountId>>::on_voting_identity_registered(
                &who, cid_number,
            );
        }

        assert_eq!(RewardedCount::<T>::get(), 1u64);
        assert!(IdentityRewardClaimed::<T>::contains_key(cid_number_hash));
        assert!(AccountRewarded::<T>::contains_key(&who));
    }
}
