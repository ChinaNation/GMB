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

    // 中文注释：当前链按 fresh genesis 口径启动，AllowedRecipients 由 genesis_build 写入。
    // 这里仅推进 storage version，不伪装承担历史数据迁移职责。
    STORAGE_VERSION.put::<Pallet<T>>();
    db.reads_writes(1, 1)
}
