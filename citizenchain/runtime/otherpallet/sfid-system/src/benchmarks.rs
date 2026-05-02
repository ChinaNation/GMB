// SFID 绑定与资格校验模块 Benchmark 定义。
//
// ADR-008 Step 2a 重写后:
// - 删除老的 `set_sheng_signing_pubkey` / `rotate_sfid_keys` benchmark。
// - 新增 4 个 Pays::No unsigned extrinsic 的 benchmark stub:
//   add_sheng_admin_backup / remove_sheng_admin_backup /
//   activate_sheng_signing_pubkey / rotate_sheng_signing_pubkey
// - 数值仍由 `cargo build --features runtime-benchmarks` 重新生成,
//   本文件确保 benchmark 编译路径不断。

#![cfg(feature = "runtime-benchmarks")]

use crate::pallet::{
    AccountToBindingId, BindingIdToAccount, BoundCount, Call, Config, NonceOf, Pallet, ShengAdmins,
    ShengNonce, ShengSigningPubkey, SignatureOf,
};
use crate::{BindCredential, Slot};
use codec::Encode;
use frame_benchmarking::v2::*;
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_core::{crypto::KeyTypeId, sr25519};
use sp_io::{
    crypto::{sr25519_generate, sr25519_sign},
    hashing::blake2_256,
};
use sp_runtime::traits::{BlakeTwo256, Hash, Zero};

const BENCH_PROVINCE: &[u8] = b"liaoning";

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn bind_sfid() {
        let caller: T::AccountId = frame_benchmarking::account("caller", 0, 0);
        let key_type = KeyTypeId(*b"sfid");
        let public: sr25519::Public = sr25519_generate(key_type, None);
        // 中文注释:占用 ShengAdmins[Main] 充当 bind 凭证签发方。
        let bounded: frame_support::BoundedVec<u8, frame_support::pallet_prelude::ConstU32<64>> =
            BENCH_PROVINCE.to_vec().try_into().expect("province fits");
        ShengAdmins::<T>::insert(&bounded, Slot::Main, public.0);

        let binding_id = T::Hashing::hash(b"benchmark-binding-id");

        let nonce_bytes = b"benchmark-nonce".to_vec();
        let nonce: NonceOf<T> = nonce_bytes.try_into().expect("nonce should fit");

        let genesis_block = BlockNumberFor::<T>::zero();
        let payload = (
            primitives::core_const::DUOQIAN_DOMAIN,
            primitives::core_const::OP_SIGN_BIND,
            frame_system::Pallet::<T>::block_hash(genesis_block),
            &caller,
            binding_id,
            nonce.as_slice(),
        );
        let msg = BlakeTwo256::hash_of(&payload);
        let signature: SignatureOf<T> = sr25519_sign(key_type, &public, msg.as_fixed_bytes())
            .expect("benchmark sr25519 signature should be generated")
            .0
            .to_vec()
            .try_into()
            .expect("sig should fit");

        let credential = BindCredential {
            binding_id,
            bind_nonce: nonce,
            signature,
        };

        #[extrinsic_call]
        bind_sfid(RawOrigin::Signed(caller), credential);
    }

    #[benchmark]
    fn unbind_sfid() {
        // 中文注释:UnbindOrigin 由 runtime 决定;benchmark 端给 Root 占位。
        let target: T::AccountId = frame_benchmarking::account("target", 1, 0);
        let binding_id = T::Hashing::hash(b"bench-binding");

        BindingIdToAccount::<T>::insert(binding_id, &target);
        AccountToBindingId::<T>::insert(&target, binding_id);
        BoundCount::<T>::put(1u64);

        #[extrinsic_call]
        unbind_sfid(RawOrigin::Root, target);
    }

    #[benchmark]
    fn add_sheng_admin_backup() {
        // 1. 占用 ShengAdmins[Main] = bench 公钥
        let key_type = KeyTypeId(*b"sfid");
        let main_pub: sr25519::Public = sr25519_generate(key_type, None);
        let bounded: frame_support::BoundedVec<u8, frame_support::pallet_prelude::ConstU32<64>> =
            BENCH_PROVINCE.to_vec().try_into().expect("province fits");
        ShengAdmins::<T>::insert(&bounded, Slot::Main, main_pub.0);

        // 2. 构造 add_backup payload 并由 main 私钥签名
        let new_pubkey: [u8; 32] = [42u8; 32];
        let nonce: ShengNonce = [7u8; 32];
        let slot = Slot::Backup1;
        let payload = (
            crate::ADD_BACKUP_DOMAIN,
            BENCH_PROVINCE,
            slot,
            &new_pubkey,
            &nonce,
        );
        let msg = blake2_256(&payload.encode());
        let sig_bytes = sr25519_sign(key_type, &main_pub, &msg)
            .expect("benchmark sign should succeed")
            .0;

        #[extrinsic_call]
        add_sheng_admin_backup(
            RawOrigin::None,
            BENCH_PROVINCE.to_vec(),
            slot,
            new_pubkey,
            nonce,
            sig_bytes,
        );
    }

    #[benchmark]
    fn remove_sheng_admin_backup() {
        let key_type = KeyTypeId(*b"sfid");
        let main_pub: sr25519::Public = sr25519_generate(key_type, None);
        let bounded: frame_support::BoundedVec<u8, frame_support::pallet_prelude::ConstU32<64>> =
            BENCH_PROVINCE.to_vec().try_into().expect("province fits");
        ShengAdmins::<T>::insert(&bounded, Slot::Main, main_pub.0);
        let backup_pubkey: [u8; 32] = [43u8; 32];
        ShengAdmins::<T>::insert(&bounded, Slot::Backup1, backup_pubkey);
        ShengSigningPubkey::<T>::insert(&bounded, &backup_pubkey, [44u8; 32]);

        let nonce: ShengNonce = [8u8; 32];
        let slot = Slot::Backup1;
        let payload = (crate::REMOVE_BACKUP_DOMAIN, BENCH_PROVINCE, slot, &nonce);
        let msg = blake2_256(&payload.encode());
        let sig_bytes = sr25519_sign(key_type, &main_pub, &msg)
            .expect("benchmark sign should succeed")
            .0;

        #[extrinsic_call]
        remove_sheng_admin_backup(
            RawOrigin::None,
            BENCH_PROVINCE.to_vec(),
            slot,
            nonce,
            sig_bytes,
        );
    }

    #[benchmark]
    fn activate_sheng_signing_pubkey() {
        // 中文注释:Main 槽空 → first-come-first-serve。
        let key_type = KeyTypeId(*b"sfid");
        let admin_pub: sr25519::Public = sr25519_generate(key_type, None);
        let signing: [u8; 32] = [99u8; 32];
        let nonce: ShengNonce = [9u8; 32];
        let payload = (
            crate::ACTIVATE_DOMAIN,
            BENCH_PROVINCE,
            &admin_pub.0,
            &signing,
            &nonce,
        );
        let msg = blake2_256(&payload.encode());
        let sig_bytes = sr25519_sign(key_type, &admin_pub, &msg)
            .expect("benchmark sign should succeed")
            .0;

        #[extrinsic_call]
        activate_sheng_signing_pubkey(
            RawOrigin::None,
            BENCH_PROVINCE.to_vec(),
            admin_pub.0,
            signing,
            nonce,
            sig_bytes,
        );
    }

    #[benchmark]
    fn rotate_sheng_signing_pubkey() {
        let key_type = KeyTypeId(*b"sfid");
        let admin_pub: sr25519::Public = sr25519_generate(key_type, None);
        let bounded: frame_support::BoundedVec<u8, frame_support::pallet_prelude::ConstU32<64>> =
            BENCH_PROVINCE.to_vec().try_into().expect("province fits");
        ShengAdmins::<T>::insert(&bounded, Slot::Main, admin_pub.0);
        ShengSigningPubkey::<T>::insert(&bounded, &admin_pub.0, [9u8; 32]);

        let new_signing: [u8; 32] = [100u8; 32];
        let nonce: ShengNonce = [10u8; 32];
        let payload = (
            crate::ROTATE_DOMAIN,
            BENCH_PROVINCE,
            &admin_pub.0,
            &new_signing,
            &nonce,
        );
        let msg = blake2_256(&payload.encode());
        let sig_bytes = sr25519_sign(key_type, &admin_pub, &msg)
            .expect("benchmark sign should succeed")
            .0;

        #[extrinsic_call]
        rotate_sheng_signing_pubkey(
            RawOrigin::None,
            BENCH_PROVINCE.to_vec(),
            admin_pub.0,
            new_signing,
            nonce,
            sig_bytes,
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
