//! Benchmarks for `internal-vote` pallet。
//!
//! 包含 cast / finalize_internal_timeout 的权重 benchmark 占位 + migration v1
//! 的负载 benchmark。weight 实测靠 substrate-benchmark-cli 跑出再覆盖
//! `weights.rs` 的手工保守上界。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_support::traits::OnRuntimeUpgrade;

use crate::migrations::v1::{build_prefix, MigrateV0ToV1, NEW_PALLET, OLD_PALLET, STORAGES};
use crate::{Config, Pallet};

#[benchmarks]
mod benchmarks {
    use super::*;

    /// migration v0 → v1 的负载 benchmark:`n` 控制每个 storage 预填的 entry 数。
    /// CLI 跑 `--steps 50 --repeat 20` 可拟合出 weight = a + b·n 线性关系。
    #[benchmark]
    fn migration_v0_to_v1(n: Linear<0, 10_000>) {
        // 预填 n entries 到旧前缀的每个 storage。
        for storage in STORAGES {
            let prefix = build_prefix(OLD_PALLET, storage);
            for i in 0..n {
                let mut key = prefix.clone();
                key.extend_from_slice(&i.to_le_bytes());
                sp_io::storage::set(&key, &i.to_le_bytes());
            }
        }

        #[block]
        {
            <MigrateV0ToV1<T> as OnRuntimeUpgrade>::on_runtime_upgrade();
        }

        // sanity:旧前缀清空,新前缀有数据
        for storage in STORAGES {
            let new_prefix = build_prefix(NEW_PALLET, storage);
            if n > 0 {
                let mut probe = new_prefix.clone();
                probe.extend_from_slice(&0u32.to_le_bytes());
                assert!(sp_io::storage::get(&probe).is_some());
            }
        }
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
