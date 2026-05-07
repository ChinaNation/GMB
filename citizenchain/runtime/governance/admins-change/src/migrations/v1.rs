//! v1 → v2: storage 改名物理迁移 `Institutions` → `Subjects`(C 阶段命名修正,2026-05-06)。
//!
//! 链上 storage prefix = twox128(pallet_name) ++ twox128(storage_name);
//! storage 名变 → 后半截 16B 改了,旧前缀下的数据成孤儿,新代码读空。
//!
//! 不迁移则:
//! - admins-change::Subjects::iter() 空
//! - 客户端按 `AdminsChange::Subjects[subject_id]` 查任何机构都返回 None
//! - 体现:wuminapp 治理 tab 任意机构详情页"管理员 0 人",
//!   节点端机构页同样空表
//!
//! 涉及 storage(本 pallet 当前唯一一张表):
//! - `Subjects`(前身 `Institutions`,StorageMap<SubjectId, AdminSubject>)
//!
//! 幂等门控:pallet StorageVersion ≥ 2 直接 noop,二次 set_code 安全。

use core::marker::PhantomData;

use frame_support::pallet_prelude::Weight;
use frame_support::storage::migration::move_prefix;
use frame_support::traits::{Get, GetStorageVersion, OnRuntimeUpgrade, StorageVersion};
use sp_io::hashing::twox_128;
use sp_std::vec::Vec;

const PALLET: &[u8] = b"AdminsChange";
const OLD_STORAGE: &[u8] = b"Institutions";
const NEW_STORAGE: &[u8] = b"Subjects";

fn build_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
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

pub struct MigrateV1ToV2<T>(PhantomData<T>);

impl<T: crate::Config> OnRuntimeUpgrade for MigrateV1ToV2<T> {
    fn on_runtime_upgrade() -> Weight {
        let on_chain = crate::Pallet::<T>::on_chain_storage_version();
        if on_chain >= 2 {
            return T::DbWeight::get().reads(1);
        }

        let old_prefix = build_prefix(PALLET, OLD_STORAGE);
        let new_prefix = build_prefix(PALLET, NEW_STORAGE);
        let total_keys = count_keys(&old_prefix);
        move_prefix(&old_prefix, &new_prefix);

        StorageVersion::new(2).put::<crate::Pallet<T>>();

        // 动态 weight:count 阶段每个 key 1 read;move_prefix 内部每个 key 1 read + 1 write;
        // 加 storage_version 1 write + 入口 storage_version 1 read。
        T::DbWeight::get().reads_writes(
            (total_keys.saturating_mul(2)).saturating_add(1) as u64,
            (total_keys.saturating_add(1)) as u64,
        )
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
        use codec::Encode;
        let old_prefix = build_prefix(PALLET, OLD_STORAGE);
        let count = count_keys(&old_prefix);
        Ok(count.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        use codec::Decode;
        let pre_count = u32::decode(&mut &state[..])
            .map_err(|_| sp_runtime::TryRuntimeError::Other("decode pre count failed"))?;
        let new_prefix = build_prefix(PALLET, NEW_STORAGE);
        let post_count = count_keys(&new_prefix);
        if post_count < pre_count {
            return Err(sp_runtime::TryRuntimeError::Other(
                "AdminsChange Subjects key count below pre-upgrade Institutions count",
            ));
        }
        let old_prefix = build_prefix(PALLET, OLD_STORAGE);
        let leftover = count_keys(&old_prefix);
        if leftover != 0 {
            return Err(sp_runtime::TryRuntimeError::Other(
                "AdminsChange::Institutions still has keys after migration",
            ));
        }
        Ok(())
    }
}
