//! Weights for `sfid_system`
//!
//! ADR-008 Step 2a 重写后的占位权重(2026-05-01):
//! - 老的 `set_sheng_signing_pubkey` / `rotate_sfid_keys` benchmark 已删除。
//! - 新增 4 个 Pays::No unsigned extrinsic(`add/remove_sheng_admin_backup` /
//!   `activate/rotate_sheng_signing_pubkey`)的权重为占位值,实际数值
//!   等链端基线就绪后通过 `cargo build --features runtime-benchmarks` 重新生成。
//!
//! 数值估算(读 + 写 + 一次 sr25519_verify ≈ 25_000_000 weight):
//! - reads: 2-3(ShengAdmins / ShengSigningPubkey / UsedShengNonce)
//! - writes: 1-2(目标 storage + UsedShengNonce)
//! - sr25519_verify: ~25_000_000
//!
//! 综合给一个 35_000_000 量级的 stub。

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for `sfid_system`.
pub trait WeightInfo {
    fn bind_sfid() -> Weight;
    fn unbind_sfid() -> Weight;
    fn add_sheng_admin_backup() -> Weight;
    fn remove_sheng_admin_backup() -> Weight;
    fn activate_sheng_signing_pubkey() -> Weight;
    fn rotate_sheng_signing_pubkey() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn bind_sfid() -> Weight {
        // 中文注释:bind_sfid 触达 BindingId/AccountTo/BoundCount/UsedBindNonce + 回调,reads 6 / writes 5。
        Weight::from_parts(150_000_000, 6_403_489)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    fn unbind_sfid() -> Weight {
        // 中文注释:Root origin → 不读 SFID admin storage;reads = AccountTo + 写 BindingId/AccountTo/BoundCount。
        Weight::from_parts(28_000_000, 3_562)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn add_sheng_admin_backup() -> Weight {
        // 中文注释:reads = ShengAdmins[Main] + ShengAdmins[slot] + UsedShengNonce;writes = ShengAdmins[slot] + UsedShengNonce;含 sr25519_verify。
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn remove_sheng_admin_backup() -> Weight {
        // 中文注释:reads = ShengAdmins[Main] + ShengAdmins[slot] + UsedShengNonce;writes = ShengAdmins[slot] + ShengSigningPubkey + UsedShengNonce。
        Weight::from_parts(36_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn activate_sheng_signing_pubkey() -> Weight {
        // 中文注释:reads = ShengAdmins[Main/Backup1/Backup2] + UsedShengNonce;writes = ShengSigningPubkey + 可能 ShengAdmins[Main] + UsedShengNonce。
        Weight::from_parts(38_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn rotate_sheng_signing_pubkey() -> Weight {
        // 中文注释:reads = ShengAdmins[Main/Backup1/Backup2] + ShengSigningPubkey + UsedShengNonce;writes = ShengSigningPubkey + UsedShengNonce。
        Weight::from_parts(36_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

impl WeightInfo for () {
    fn bind_sfid() -> Weight {
        Weight::from_parts(150_000_000, 6_403_489)
            .saturating_add(RocksDbWeight::get().reads(6))
            .saturating_add(RocksDbWeight::get().writes(5))
    }

    fn unbind_sfid() -> Weight {
        Weight::from_parts(28_000_000, 3_562)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn add_sheng_admin_backup() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn remove_sheng_admin_backup() -> Weight {
        Weight::from_parts(36_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn activate_sheng_signing_pubkey() -> Weight {
        Weight::from_parts(38_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn rotate_sheng_signing_pubkey() -> Weight {
        Weight::from_parts(36_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}
