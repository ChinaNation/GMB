//! 公民轻节点奖励模块 benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use sfid_code_auth::OnSfidBound;
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
        let sfid_hash = T::Hashing::hash(b"citizen-lightnode-bench");

        #[block]
        {
            <Pallet<T> as OnSfidBound<T::AccountId, T::Hash>>::on_sfid_bound(&who, sfid_hash);
        }

        assert_eq!(RewardedCount::<T>::get(), 1u64);
        assert!(RewardClaimed::<T>::contains_key(sfid_hash));
        assert!(AccountRewarded::<T>::contains_key(&who));
    }
}
