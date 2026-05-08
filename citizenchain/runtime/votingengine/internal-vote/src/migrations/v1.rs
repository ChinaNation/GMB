//! v0 → v1: sub-pallet 拆分前的 storage 从 `VotingEngine` 前缀搬到 `InternalVote` 前缀。
//!
//! 拆分后 storage prefix = twox128(pallet_name),pallet 名变了 → 前缀变了。
//! 不迁移则 runtime upgrade 后链上旧数据成孤儿、新代码读空。
//!
//! 涉及 storage:
//! - `InternalVotesByAccount`(双层:proposal_id × account)
//! - `InternalTallies`(单层:proposal_id)
//! - `InternalThresholdSnapshot`(单层:proposal_id)
//!
//! 幂等门控:pallet StorageVersion ≥ 1 直接 noop,二次 set_code 安全。

use core::marker::PhantomData;

use frame_support::pallet_prelude::Weight;
use frame_support::storage::migration::move_prefix;
use frame_support::traits::{Get, GetStorageVersion, OnRuntimeUpgrade, StorageVersion};
use sp_io::hashing::twox_128;
use sp_std::vec::Vec;

// 中文注释:OLD/NEW pallet 名 + STORAGES + build_prefix 同时被 benchmarks.rs 引用,
// pub(crate) 共享避免两处定义漂移。
pub(crate) const OLD_PALLET: &[u8] = b"VotingEngine";
pub(crate) const NEW_PALLET: &[u8] = b"InternalVote";

/// 受影响的 storage 前缀名(本 sub-pallet 自有的全部 storage)。
pub(crate) const STORAGES: &[&[u8]] = &[
    b"InternalVotesByAccount",
    b"InternalTallies",
    b"InternalThresholdSnapshot",
];

pub(crate) fn build_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
    let mut p = Vec::with_capacity(32);
    p.extend_from_slice(&twox_128(pallet));
    p.extend_from_slice(&twox_128(storage));
    p
}

/// 数 prefix 下 key 数量,用于动态 weight 计算与 try-runtime 校验。
fn count_keys(prefix: &[u8]) -> u32 {
    let mut count = 0u32;
    let mut cursor: Vec<u8> = prefix.to_vec();
    loop {
        match sp_io::storage::next_key(&cursor) {
            Some(k) if k.starts_with(prefix) => {
                count = count.saturating_add(1);
                cursor = k;
            }
            _ => break,
        }
    }
    count
}

pub struct MigrateV0ToV1<T>(PhantomData<T>);

impl<T: crate::Config> OnRuntimeUpgrade for MigrateV0ToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        let on_chain = crate::Pallet::<T>::on_chain_storage_version();
        if on_chain >= 1 {
            return T::DbWeight::get().reads(1);
        }

        let mut total_keys: u32 = 0;
        for storage in STORAGES {
            let old_prefix = build_prefix(OLD_PALLET, storage);
            let new_prefix = build_prefix(NEW_PALLET, storage);
            let n = count_keys(&old_prefix);
            move_prefix(&old_prefix, &new_prefix);
            total_keys = total_keys.saturating_add(n);
        }

        StorageVersion::new(1).put::<crate::Pallet<T>>();

        // 动态 weight:count 阶段每个 key 1 read;move_prefix 内部每个 key 1 read + 1 write;
        // 加 storage_version 1 write + 入口 storage_version 1 read。
        T::DbWeight::get().reads_writes(
            (total_keys.saturating_mul(2)).saturating_add(1) as u64,
            (total_keys.saturating_add(1)) as u64,
        )
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        use codec::Encode;
        let mut counts: Vec<(Vec<u8>, u32)> = Vec::new();
        for storage in STORAGES {
            let n = count_keys(&build_prefix(OLD_PALLET, storage));
            counts.push((storage.to_vec(), n));
        }
        Ok(counts.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        use codec::Decode;
        use frame_support::ensure;
        let pre: Vec<(Vec<u8>, u32)> = Decode::decode(&mut state.as_slice())
            .map_err(|_| sp_runtime::TryRuntimeError::Other("pre_upgrade decode failed"))?;
        for (storage, expected) in pre {
            let old_n = count_keys(&build_prefix(OLD_PALLET, &storage));
            let new_n = count_keys(&build_prefix(NEW_PALLET, &storage));
            ensure!(old_n == 0, "old prefix still has keys");
            ensure!(new_n == expected, "new prefix key count mismatch");
        }
        ensure!(
            crate::Pallet::<T>::on_chain_storage_version() == 1,
            "storage version not bumped"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{new_test_ext, Test};
    use frame_support::traits::OnRuntimeUpgrade;

    fn write_legacy(storage: &[u8], key_suffix: &[u8], value: &[u8]) {
        let mut full = build_prefix(OLD_PALLET, storage);
        full.extend_from_slice(key_suffix);
        sp_io::storage::set(&full, value);
    }

    fn read_new(storage: &[u8], key_suffix: &[u8]) -> Option<Vec<u8>> {
        let mut full = build_prefix(NEW_PALLET, storage);
        full.extend_from_slice(key_suffix);
        sp_io::storage::get(&full).map(|v| v.to_vec())
    }

    fn read_legacy(storage: &[u8], key_suffix: &[u8]) -> Option<Vec<u8>> {
        let mut full = build_prefix(OLD_PALLET, storage);
        full.extend_from_slice(key_suffix);
        sp_io::storage::get(&full).map(|v| v.to_vec())
    }

    #[test]
    fn migrate_moves_internal_storage_with_load() {
        new_test_ext().execute_with(|| {
            // 1000 entries 写到旧前缀 InternalVotesByAccount
            for i in 0u32..1000 {
                write_legacy(
                    b"InternalVotesByAccount",
                    &i.to_le_bytes(),
                    &[(i % 256) as u8],
                );
            }
            // 500 entries 写到 InternalTallies
            for i in 0u32..500 {
                write_legacy(b"InternalTallies", &i.to_le_bytes(), &i.to_le_bytes());
            }
            // 50 entries 写到 InternalThresholdSnapshot
            for i in 0u32..50 {
                write_legacy(
                    b"InternalThresholdSnapshot",
                    &i.to_le_bytes(),
                    &(i as u8).to_le_bytes(),
                );
            }

            // 跑迁移
            let weight = MigrateV0ToV1::<Test>::on_runtime_upgrade();

            // 验证旧前缀全空
            for storage in STORAGES {
                let n = count_keys(&build_prefix(OLD_PALLET, storage));
                assert_eq!(
                    n,
                    0,
                    "old prefix {:?} not empty",
                    core::str::from_utf8(storage).unwrap()
                );
            }

            // 验证新前缀有正确数据
            for i in 0u32..1000 {
                assert_eq!(
                    read_new(b"InternalVotesByAccount", &i.to_le_bytes()),
                    Some(vec![(i % 256) as u8]),
                );
            }
            for i in 0u32..500 {
                assert_eq!(
                    read_new(b"InternalTallies", &i.to_le_bytes()),
                    Some(i.to_le_bytes().to_vec()),
                );
            }
            for i in 0u32..50 {
                assert_eq!(
                    read_new(b"InternalThresholdSnapshot", &i.to_le_bytes()),
                    Some((i as u8).to_le_bytes().to_vec()),
                );
            }

            // 验证 StorageVersion = 1
            assert_eq!(
                crate::Pallet::<Test>::on_chain_storage_version(),
                StorageVersion::new(1)
            );

            // weight 不超过单 block 上限
            assert!(
                weight.ref_time() < 1_000_000_000_000,
                "migration weight {} exceeds 1s ref_time",
                weight.ref_time()
            );

            // 幂等:第二次跑无副作用,旧前缀仍空,storage version 不再被改写。
            let _ = MigrateV0ToV1::<Test>::on_runtime_upgrade();
            for storage in STORAGES {
                assert_eq!(count_keys(&build_prefix(OLD_PALLET, storage)), 0);
            }
            assert_eq!(
                crate::Pallet::<Test>::on_chain_storage_version(),
                StorageVersion::new(1)
            );
        });
    }

    #[test]
    fn migrate_handles_empty_storage() {
        new_test_ext().execute_with(|| {
            let weight = MigrateV0ToV1::<Test>::on_runtime_upgrade();
            assert_eq!(
                crate::Pallet::<Test>::on_chain_storage_version(),
                StorageVersion::new(1)
            );
            // 空 storage 时 weight 仅 1 read(version) + 1 write(version) + 0 keys
            assert!(weight.ref_time() < 100_000_000);
        });
    }

    #[test]
    fn migrate_does_not_touch_unrelated_prefixes() {
        new_test_ext().execute_with(|| {
            // 在 OLD_PALLET 下写一个 NOT in STORAGES 的 storage(模拟 votingengine 主 crate 自有 storage)
            write_legacy(b"Proposals", &42u32.to_le_bytes(), b"unrelated");

            MigrateV0ToV1::<Test>::on_runtime_upgrade();

            // 验证不相关 storage 没动
            assert_eq!(
                read_legacy(b"Proposals", &42u32.to_le_bytes()),
                Some(b"unrelated".to_vec())
            );
        });
    }
}
