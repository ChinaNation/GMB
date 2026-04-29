// SFID 绑定与资格校验模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use crate::{
    AccountToBindingId, BindCredential, BindingIdToAccount, BoundCount, Call, Config, NonceOf,
    Pallet, SfidBackupAccount1, SfidBackupAccount2, SfidMainAccount, SignatureOf,
};
use codec::{Decode, Encode};
use frame_benchmarking::v2::*;
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_core::{crypto::KeyTypeId, sr25519};
use sp_io::crypto::{sr25519_generate, sr25519_sign};
use sp_runtime::{
    traits::{BlakeTwo256, Hash, IdentifyAccount, Zero},
    MultiSigner,
};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn bind_sfid() {
        let caller: T::AccountId = frame_benchmarking::account("caller", 0, 0);
        let key_type = KeyTypeId(*b"sfid");
        let public: sr25519::Public = sr25519_generate(key_type, None);
        let sfid_main: T::AccountId = T::AccountId::decode(
            &mut &MultiSigner::from(public.clone()).into_account().encode()[..],
        )
        .expect("benchmark sfid main account must decode");
        SfidMainAccount::<T>::put(sfid_main);

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
        // 管理员（SFID 主账户）代为解绑 target
        let admin: T::AccountId = frame_benchmarking::account("admin", 0, 0);
        let target: T::AccountId = frame_benchmarking::account("target", 1, 0);
        let binding_id = T::Hashing::hash(b"bench-binding");

        SfidMainAccount::<T>::put(&admin);
        BindingIdToAccount::<T>::insert(binding_id, &target);
        AccountToBindingId::<T>::insert(&target, binding_id);
        BoundCount::<T>::put(1u64);

        #[extrinsic_call]
        unbind_sfid(RawOrigin::Signed(admin), target);
    }

    #[benchmark]
    fn rotate_sfid_keys() {
        let backup1: T::AccountId = frame_benchmarking::account("backup1", 0, 0);
        let backup2: T::AccountId = frame_benchmarking::account("backup2", 1, 0);
        let main_key: T::AccountId = frame_benchmarking::account("main", 2, 0);
        let new_backup: T::AccountId = frame_benchmarking::account("new_backup", 3, 0);

        SfidMainAccount::<T>::put(&main_key);
        SfidBackupAccount1::<T>::put(&backup1);
        SfidBackupAccount2::<T>::put(&backup2);

        #[extrinsic_call]
        rotate_sfid_keys(RawOrigin::Signed(backup1), new_backup);
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
