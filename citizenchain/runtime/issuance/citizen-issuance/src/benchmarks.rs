//! 公民发行模块 benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use sfid_system::OnSfidBound;
use sp_runtime::traits::Hash;

use crate::pallet::{AccountRewarded, Config, Pallet, RewardClaimed, RewardedCount};

fn decode_account<T: Config>(raw: [u8; 32]) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn on_sfid_bound() {
        let who = decode_account::<T>([7u8; 32]);
        let binding_id = T::Hashing::hash(b"citizen-issuance-bench");

        #[block]
        {
            <Pallet<T> as OnSfidBound<T::AccountId, T::Hash>>::on_sfid_bound(&who, binding_id);
        }

        assert_eq!(RewardedCount::<T>::get(), 1u64);
        assert!(RewardClaimed::<T>::contains_key(binding_id));
        assert!(AccountRewarded::<T>::contains_key(&who));
    }
}
