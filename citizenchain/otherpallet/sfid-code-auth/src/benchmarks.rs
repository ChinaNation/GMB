// SFID 绑定与资格校验模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use crate::{
    AccountToSfid, BindCredential, BoundCount, Call, Config, NonceOf, Pallet, SfidBackupAccount1,
    SfidBackupAccount2, SfidMainAccount, SfidOf, SfidToAccount, SignatureOf,
};
use codec::{Decode, Encode};
use frame_benchmarking::v2::*;
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_core::{crypto::KeyTypeId, sr25519};
use sp_io::crypto::{sr25519_generate, sr25519_sign};
use sp_runtime::{traits::{BlakeTwo256, Hash, IdentifyAccount, Saturating, Zero}, MultiSigner};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn bind_sfid() {
        let caller: T::AccountId = frame_benchmarking::account("caller", 0, 0);
        let key_type = KeyTypeId(*b"sfid");
        let public: sr25519::Public = sr25519_generate(key_type, None);
        let sfid_main: T::AccountId =
            T::AccountId::decode(&mut &MultiSigner::from(public.clone()).into_account().encode()[..])
                .expect("benchmark sfid main account must decode");
        SfidMainAccount::<T>::put(sfid_main);

        let sfid_bytes = b"benchmark-sfid-code".to_vec();
        let sfid_code: SfidOf<T> = sfid_bytes.try_into().expect("sfid should fit");
        let sfid_hash = T::Hashing::hash(sfid_code.as_slice());

        let nonce_bytes = b"benchmark-nonce".to_vec();
        let nonce: NonceOf<T> = nonce_bytes.try_into().expect("nonce should fit");

        let now = frame_system::Pallet::<T>::block_number();
        let expires_at = now.saturating_add(10u32.into());
        let genesis_block = BlockNumberFor::<T>::zero();
        let payload = (
            b"GMB_SFID_BIND_V2",
            frame_system::Pallet::<T>::block_hash(genesis_block),
            &caller,
            sfid_hash,
            nonce.as_slice(),
            expires_at,
        );
        let msg = BlakeTwo256::hash_of(&payload);
        let signature: SignatureOf<T> = sr25519_sign(key_type, &public, msg.as_fixed_bytes())
            .expect("benchmark sr25519 signature should be generated")
            .0
            .to_vec()
            .try_into()
            .expect("sig should fit");

        let credential = BindCredential {
            sfid_code_hash: sfid_hash,
            nonce,
            expires_at,
            signature,
        };

        #[extrinsic_call]
        bind_sfid(RawOrigin::Signed(caller), sfid_code, credential);
    }

    #[benchmark]
    fn unbind_sfid() {
        let caller: T::AccountId = frame_benchmarking::account("caller", 0, 0);
        let sfid_hash = T::Hashing::hash(b"bench-sfid");

        SfidToAccount::<T>::insert(sfid_hash, &caller);
        AccountToSfid::<T>::insert(&caller, sfid_hash);
        BoundCount::<T>::put(1u64);

        #[extrinsic_call]
        unbind_sfid(RawOrigin::Signed(caller));
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
