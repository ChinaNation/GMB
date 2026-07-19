//! SquarePost StorageVersion v1 -> v2 原地升级。
//!
//! 正式链已确认没有订阅数据，但升级仍做双重检查。发现任何旧订阅或到期索引时不 panic、
//! 不删除数据、不提升 StorageVersion，并用 `MigrationBlocked` 让新订阅入口 fail-closed。

use crate::{
    pallet::{
        Config, MigrationBlocked, Pallet, PlatformPrice, RenewalIndex, RenewalSchedule,
        Subscriptions, DEMOCRACY_PRICE_FEN, FREEDOM_PRICE_FEN, SPARK_PRICE_FEN,
    },
    MembershipLevel,
};
#[cfg(feature = "try-runtime")]
use crate::{
    pallet::{PlatformCidNumber, PublishedPostCountByAccount, SquarePosts},
    SubscriptionStatus,
};
#[cfg(feature = "try-runtime")]
use codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use frame_support::traits::Currency;
use frame_support::{
    traits::{Get, StorageVersion},
    weights::Weight,
    StorageHasher,
};
#[cfg(feature = "try-runtime")]
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec::Vec;

pub const TARGET_STORAGE_VERSION: StorageVersion = StorageVersion::new(2);
const SOURCE_STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

fn storage_prefix(storage_name: &[u8]) -> Vec<u8> {
    let mut prefix = frame_support::Twox128::hash(b"SquarePost").to_vec();
    prefix.extend_from_slice(&frame_support::Twox128::hash(storage_name));
    prefix
}

fn prefix_has_keys(storage_name: &[u8]) -> bool {
    let prefix = storage_prefix(storage_name);
    sp_io::storage::next_key(&prefix)
        .map(|key| key.starts_with(&prefix))
        .unwrap_or(false)
}

fn subscription_state_exists<T: Config>() -> bool {
    Subscriptions::<T>::iter_keys().next().is_some()
        || RenewalSchedule::<T>::iter_keys().next().is_some()
        || RenewalIndex::<T>::iter_keys().next().is_some()
        || prefix_has_keys(b"DueQueue")
        || prefix_has_keys(b"DueIndex")
}

pub fn migrate<T: Config>() -> Weight {
    let db = T::DbWeight::get();
    let on_chain = StorageVersion::get::<Pallet<T>>();
    if on_chain >= TARGET_STORAGE_VERSION {
        return db.reads(1);
    }
    if on_chain != SOURCE_STORAGE_VERSION || subscription_state_exists::<T>() {
        MigrationBlocked::<T>::put(true);
        return db.reads_writes(4, 1);
    }

    let mut price_writes = 0u64;
    for (level, default_price) in [
        (MembershipLevel::Freedom, FREEDOM_PRICE_FEN),
        (MembershipLevel::Democracy, DEMOCRACY_PRICE_FEN),
        (MembershipLevel::Spark, SPARK_PRICE_FEN),
    ] {
        if !PlatformPrice::<T>::contains_key(level) {
            PlatformPrice::<T>::insert(level, default_price);
            price_writes = price_writes.saturating_add(1);
        }
    }

    // 只有所有订阅相关前缀均为空时，才精确清理已经退役的 keeper 单值。
    let keeper_key = storage_prefix(b"BillingKeeper");
    frame_support::storage::unhashed::kill(&keeper_key);
    MigrationBlocked::<T>::kill();
    TARGET_STORAGE_VERSION.put::<Pallet<T>>();
    db.reads_writes(8, price_writes.saturating_add(3))
}

#[cfg(feature = "try-runtime")]
#[derive(Encode, Decode)]
struct UpgradeSnapshot {
    square_posts_hash: [u8; 32],
    published_counts_hash: [u8; 32],
    total_issuance: u128,
    account_state_hash: [u8; 32],
    platform_prices: [Option<u128>; 3],
    platform_cid_number: Option<Vec<u8>>,
}

#[cfg(feature = "try-runtime")]
pub fn pre_upgrade<T: Config>() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
    frame_support::ensure!(
        StorageVersion::get::<Pallet<T>>() == SOURCE_STORAGE_VERSION,
        "square-post pre_upgrade: StorageVersion must be v1"
    );
    frame_support::ensure!(
        !subscription_state_exists::<T>(),
        "square-post pre_upgrade: subscription state must be empty"
    );
    let posts = SquarePosts::<T>::iter().collect::<Vec<_>>().encode();
    let counts = PublishedPostCountByAccount::<T>::iter()
        .collect::<Vec<_>>()
        .encode();
    let accounts = frame_system::Account::<T>::iter()
        .collect::<Vec<_>>()
        .encode();
    Ok(UpgradeSnapshot {
        square_posts_hash: sp_io::hashing::blake2_256(&posts),
        published_counts_hash: sp_io::hashing::blake2_256(&counts),
        total_issuance: T::Currency::total_issuance().saturated_into::<u128>(),
        account_state_hash: sp_io::hashing::blake2_256(&accounts),
        platform_prices: [
            PlatformPrice::<T>::get(MembershipLevel::Freedom),
            PlatformPrice::<T>::get(MembershipLevel::Democracy),
            PlatformPrice::<T>::get(MembershipLevel::Spark),
        ],
        platform_cid_number: PlatformCidNumber::<T>::get().map(|cid| cid.to_vec()),
    }
    .encode())
}

#[cfg(feature = "try-runtime")]
pub fn post_upgrade<T: Config>(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
    let snapshot = UpgradeSnapshot::decode(&mut &state[..])
        .map_err(|_| "square-post post_upgrade: invalid snapshot")?;
    frame_support::ensure!(
        StorageVersion::get::<Pallet<T>>() == TARGET_STORAGE_VERSION,
        "square-post post_upgrade: StorageVersion must be v2"
    );
    frame_support::ensure!(
        !MigrationBlocked::<T>::get() && !subscription_state_exists::<T>(),
        "square-post post_upgrade: migration was blocked or subscription state appeared"
    );
    let posts = SquarePosts::<T>::iter().collect::<Vec<_>>().encode();
    let counts = PublishedPostCountByAccount::<T>::iter()
        .collect::<Vec<_>>()
        .encode();
    let accounts = frame_system::Account::<T>::iter()
        .collect::<Vec<_>>()
        .encode();
    frame_support::ensure!(
        sp_io::hashing::blake2_256(&posts) == snapshot.square_posts_hash
            && sp_io::hashing::blake2_256(&counts) == snapshot.published_counts_hash,
        "square-post post_upgrade: post state changed"
    );
    frame_support::ensure!(
        T::Currency::total_issuance().saturated_into::<u128>() == snapshot.total_issuance
            && sp_io::hashing::blake2_256(&accounts) == snapshot.account_state_hash,
        "square-post post_upgrade: unrelated account state changed"
    );
    frame_support::ensure!(
        PlatformCidNumber::<T>::get().map(|cid| cid.to_vec()) == snapshot.platform_cid_number,
        "square-post post_upgrade: PlatformCidNumber changed"
    );
    let after = [
        PlatformPrice::<T>::get(MembershipLevel::Freedom),
        PlatformPrice::<T>::get(MembershipLevel::Democracy),
        PlatformPrice::<T>::get(MembershipLevel::Spark),
    ];
    let defaults = [FREEDOM_PRICE_FEN, DEMOCRACY_PRICE_FEN, SPARK_PRICE_FEN];
    for index in 0..3 {
        frame_support::ensure!(
            after[index] == Some(snapshot.platform_prices[index].unwrap_or(defaults[index])),
            "square-post post_upgrade: platform price invariant failed"
        );
    }
    Ok(())
}

#[cfg(feature = "try-runtime")]
pub fn try_state<T: Config>() -> Result<(), sp_runtime::TryRuntimeError> {
    if StorageVersion::get::<Pallet<T>>() != TARGET_STORAGE_VERSION {
        return Ok(());
    }
    frame_support::ensure!(
        !MigrationBlocked::<T>::get(),
        "square-post try_state: migration is blocked at target version"
    );
    for (key, state) in Subscriptions::<T>::iter() {
        match state.subscription_status {
            SubscriptionStatus::Active => {
                frame_support::ensure!(
                    RenewalIndex::<T>::get(&key) == Some(state.paid_until)
                        && RenewalSchedule::<T>::contains_key(state.paid_until.to_be_bytes(), &key,),
                    "square-post try_state: active subscription schedule mismatch"
                );
            }
            SubscriptionStatus::Cancelled | SubscriptionStatus::Terminated => {
                frame_support::ensure!(
                    !RenewalIndex::<T>::contains_key(&key),
                    "square-post try_state: inactive subscription remains scheduled"
                );
            }
        }
    }
    for (due_key, key, ()) in RenewalSchedule::<T>::iter() {
        frame_support::ensure!(
            RenewalIndex::<T>::get(&key) == Some(u64::from_be_bytes(due_key)),
            "square-post try_state: renewal reverse index mismatch"
        );
    }
    Ok(())
}
