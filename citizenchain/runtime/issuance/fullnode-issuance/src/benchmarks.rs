//! 全节点发行模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use crate::{pallet, Call, Config, Pallet};
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn bind_reward_wallet() {
        let miner: T::AccountId = frame_benchmarking::account("miner", 0, 0);
        let wallet: T::AccountId = frame_benchmarking::account("wallet", 0, 0);

        // 中文注释：绑定资格来自真实出块记录，benchmark 需预置已出块矿工。
        pallet::LastAuthoredBlockByMiner::<T>::insert(&miner, 1u32);

        #[extrinsic_call]
        bind_reward_wallet(RawOrigin::Signed(miner), wallet);
    }

    #[benchmark]
    fn rebind_reward_wallet() {
        let miner: T::AccountId = frame_benchmarking::account("miner", 0, 0);
        let wallet: T::AccountId = frame_benchmarking::account("wallet", 0, 0);
        let new_wallet: T::AccountId = frame_benchmarking::account("new_wallet", 0, 0);

        // 前置：先绑定一次
        pallet::RewardWalletByMiner::<T>::insert(&miner, &wallet);

        #[extrinsic_call]
        rebind_reward_wallet(RawOrigin::Signed(miner), new_wallet);
    }
}
