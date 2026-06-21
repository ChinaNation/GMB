// SFID 绑定与资格校验模块 Benchmark 定义。
//
// 中文注释:本 pallet 不再维护省级签发管理员。benchmark 只覆盖仍存在的
// bind_sfid / unbind_sfid 两个 extrinsic;签发管理员有效性由 runtime verifier 注入。

#![cfg(feature = "runtime-benchmarks")]

use crate::pallet::{
    AccountToBindingId, BindingIdToAccount, BoundCount, Call, Config, NonceOf, Pallet, SignatureOf,
};
use crate::BindCredential;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::Hash;

const BENCH_ISSUER_SFID: &[u8] = b"SFID-BENCH-ISSUER";
const BENCH_SCOPE_PROVINCE: &[u8] = b"bench-province";
const BENCH_SCOPE_CITY: &[u8] = b"bench-city";

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn bind_sfid() {
        let caller: T::AccountId = frame_benchmarking::account("caller", 0, 0);
        let issuer_main_account: T::AccountId = frame_benchmarking::account("issuer", 0, 0);
        let binding_id = T::Hashing::hash(b"benchmark-binding-id");
        let nonce: NonceOf<T> = b"benchmark-nonce"
            .to_vec()
            .try_into()
            .expect("nonce should fit");
        let signature: SignatureOf<T> = vec![1u8; 64].try_into().expect("sig should fit");

        let credential = BindCredential {
            binding_id,
            bind_nonce: nonce,
            issuer_sfid_number: BENCH_ISSUER_SFID
                .to_vec()
                .try_into()
                .expect("issuer sfid should fit"),
            issuer_main_account,
            signer_pubkey: [7u8; 32],
            scope_province_name: BENCH_SCOPE_PROVINCE
                .to_vec()
                .try_into()
                .expect("scope province should fit"),
            scope_city_name: BENCH_SCOPE_CITY
                .to_vec()
                .try_into()
                .expect("scope city should fit"),
            signature,
        };

        #[extrinsic_call]
        bind_sfid(RawOrigin::Signed(caller), credential);
    }

    #[benchmark]
    fn unbind_sfid() {
        let target: T::AccountId = frame_benchmarking::account("target", 1, 0);
        let binding_id = T::Hashing::hash(b"bench-binding");

        BindingIdToAccount::<T>::insert(binding_id, &target);
        AccountToBindingId::<T>::insert(&target, binding_id);
        BoundCount::<T>::put(1u64);

        #[extrinsic_call]
        unbind_sfid(RawOrigin::Root, target);
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
