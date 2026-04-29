//! 决议发行模块 storage 版本。
//!
//! 中文注释：本次合并发生在开发期 fresh genesis 口径下，历史拆分时期的
//! 两套 storage 前缀不做线上迁移。
//! 如果未来已有运行链数据，必须单独增加显式 storage migration。

use crate::pallet::{Config, Pallet};
use frame_support::{pallet_prelude::StorageVersion, traits::Get, weights::Weight};

pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

pub fn on_runtime_upgrade<T: Config>() -> Weight {
    let db = T::DbWeight::get();
    let on_chain = StorageVersion::get::<Pallet<T>>();
    if on_chain >= STORAGE_VERSION {
        return db.reads(1);
    }

    let mut reads = 1u64;
    let mut writes = 0u64;
    if crate::pallet::AllowedRecipients::<T>::get().is_empty() {
        reads = reads.saturating_add(1);
        if let Some(defaults) = Pallet::<T>::decode_default_allowed_recipients() {
            crate::pallet::AllowedRecipients::<T>::put(defaults);
            writes = writes.saturating_add(1);
        }
    }

    STORAGE_VERSION.put::<Pallet<T>>();
    writes = writes.saturating_add(1);
    db.reads_writes(reads, writes)
}
