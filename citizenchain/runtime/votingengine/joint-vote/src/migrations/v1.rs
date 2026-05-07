//! v0 → v1: sub-pallet 拆分前的 storage 从 `VotingEngine` 前缀搬到 `JointVote` 前缀,
//! 同时把跟独立 citizen-vote pallet 字面冲突的 storage 名改正为联合公投阶段语义。
//!
//! 搬运表(链上旧名 → 新名):
//! - `JointVotesByAdmin` → 同名(双层:proposal_id × (institution ++ account))
//! - `JointInstitutionTallies` → 同名(双层:proposal_id × institution)
//! - `JointVotesByInstitution` → 同名(双层:proposal_id × institution)
//! - `JointTallies` → 同名(单层:proposal_id)
//! - `CitizenVotesByBindingId` → `ReferendumVotesByBindingId`(双层:proposal_id × binding_hash)
//! - `CitizenTallies` → `ReferendumTallies`(单层:proposal_id)
//! - `UsedPopulationSnapshotNonce` → 同名(单层:hash)
//!
//! 幂等门控:pallet StorageVersion ≥ 1 直接 noop。

use core::marker::PhantomData;

use frame_support::pallet_prelude::Weight;
use frame_support::storage::migration::move_prefix;
use frame_support::traits::{Get, GetStorageVersion, OnRuntimeUpgrade, StorageVersion};
use sp_io::hashing::twox_128;
use sp_std::vec::Vec;

const OLD_PALLET: &[u8] = b"VotingEngine";
const NEW_PALLET: &[u8] = b"JointVote";

/// (链上旧 storage 名 in `VotingEngine`, 新 storage 名 in `JointVote`)。
/// 大多数同名,Citizen* 一对改名为 Referendum*。
const STORAGES: &[(&[u8], &[u8])] = &[
    (b"JointVotesByAdmin", b"JointVotesByAdmin"),
    (b"JointInstitutionTallies", b"JointInstitutionTallies"),
    (b"JointVotesByInstitution", b"JointVotesByInstitution"),
    (b"JointTallies", b"JointTallies"),
    (b"CitizenVotesByBindingId", b"ReferendumVotesByBindingId"),
    (b"CitizenTallies", b"ReferendumTallies"),
    (
        b"UsedPopulationSnapshotNonce",
        b"UsedPopulationSnapshotNonce",
    ),
];

fn build_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
    let mut p = Vec::with_capacity(32);
    p.extend_from_slice(&twox_128(pallet));
    p.extend_from_slice(&twox_128(storage));
    p
}

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
        for (old_name, new_name) in STORAGES {
            let old_prefix = build_prefix(OLD_PALLET, old_name);
            let new_prefix = build_prefix(NEW_PALLET, new_name);
            let n = count_keys(&old_prefix);
            move_prefix(&old_prefix, &new_prefix);
            total_keys = total_keys.saturating_add(n);
        }

        StorageVersion::new(1).put::<crate::Pallet<T>>();

        T::DbWeight::get().reads_writes(
            (total_keys.saturating_mul(2)).saturating_add(1) as u64,
            (total_keys.saturating_add(1)) as u64,
        )
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        use codec::Encode;
        // 记录每条搬运链路 (old_name, new_name, pre_count_at_old)。
        let mut counts: Vec<(Vec<u8>, Vec<u8>, u32)> = Vec::new();
        for (old_name, new_name) in STORAGES {
            let n = count_keys(&build_prefix(OLD_PALLET, old_name));
            counts.push((old_name.to_vec(), new_name.to_vec(), n));
        }
        Ok(counts.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        use codec::Decode;
        use frame_support::ensure;
        let pre: Vec<(Vec<u8>, Vec<u8>, u32)> = Decode::decode(&mut state.as_slice())
            .map_err(|_| sp_runtime::TryRuntimeError::Other("pre_upgrade decode failed"))?;
        for (old_name, new_name, expected) in pre {
            let old_n = count_keys(&build_prefix(OLD_PALLET, &old_name));
            let new_n = count_keys(&build_prefix(NEW_PALLET, &new_name));
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
    fn migrate_moves_joint_storage_with_load() {
        new_test_ext().execute_with(|| {
            // 7 storage 各写 1000 entries 到旧前缀(共 7000 keys 真实负载),
            // 模拟链上现存数据用旧 storage 名(Citizen* 而非 Referendum*)。
            for (old_name, _) in STORAGES {
                for i in 0u32..1000 {
                    write_legacy(old_name, &i.to_le_bytes(), &i.to_le_bytes());
                }
            }

            let weight = MigrateV0ToV1::<Test>::on_runtime_upgrade();

            // 旧前缀全空
            for (old_name, _) in STORAGES {
                let n = count_keys(&build_prefix(OLD_PALLET, old_name));
                assert_eq!(
                    n,
                    0,
                    "old prefix {:?} not empty",
                    core::str::from_utf8(old_name).unwrap()
                );
            }

            // 新前缀有正确数据(Citizen* 进了 Referendum* 的位置)
            for (_, new_name) in STORAGES {
                for i in 0u32..1000 {
                    assert_eq!(
                        read_new(new_name, &i.to_le_bytes()),
                        Some(i.to_le_bytes().to_vec()),
                        "{} entry {} mismatch after migrate",
                        core::str::from_utf8(new_name).unwrap(),
                        i,
                    );
                }
            }

            assert_eq!(
                crate::Pallet::<Test>::on_chain_storage_version(),
                StorageVersion::new(1)
            );

            // weight 不超过单 block 上限(2s ref_time)
            assert!(
                weight.ref_time() < 2_000_000_000_000,
                "migration weight {} exceeds 2s ref_time",
                weight.ref_time()
            );

            // 幂等:第二次跑无副作用,旧前缀仍空,storage version 不再被改写。
            let _ = MigrateV0ToV1::<Test>::on_runtime_upgrade();
            for (old_name, _) in STORAGES {
                assert_eq!(count_keys(&build_prefix(OLD_PALLET, old_name)), 0);
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
            assert!(weight.ref_time() < 100_000_000);
        });
    }

    #[test]
    fn migrate_does_not_touch_unrelated_prefixes() {
        new_test_ext().execute_with(|| {
            // 写一个 votingengine 主 crate 的 storage(应不动)
            write_legacy(b"Proposals", &42u32.to_le_bytes(), b"unrelated");

            MigrateV0ToV1::<Test>::on_runtime_upgrade();

            assert_eq!(
                read_legacy(b"Proposals", &42u32.to_le_bytes()),
                Some(b"unrelated".to_vec())
            );
        });
    }
}
