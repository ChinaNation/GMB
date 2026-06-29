//! 多签交易模块 Benchmark 定义。
//!
//! 投票统一走 `votingengine::internal_vote`,本模块不承担投票/聚合 extrinsic。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_std::{vec, vec::Vec};

use crate::{
    pallet::{
        AccountNameOf, AccountRegisteredCid, CidNumberOf, CidRegisteredAccount,
        InstitutionAccountNamesOf, RegisterNonceOf, RegisterSignatureOf,
    },
    AccountValidator, Call, Config, Pallet, ProtectedSourceChecker, ReservedAccountGuard,
};

fn find_safe_cid<T: Config>() -> Result<(CidNumberOf<T>, T::AccountId), BenchmarkError> {
    for candidate in 0..2_048u32 {
        let mut raw = b"multisig-benchmark-cid-".to_vec();
        raw.extend_from_slice(&candidate.to_le_bytes());
        let cid_number: CidNumberOf<T> = raw
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark cid id should fit"))?;

        // benchmark 场景用主账户名派生，走机构主账户 OP_MAIN 路径。
        let Ok((account, _kind)) = Pallet::<T>::derive_registered_account(
            cid_number.as_slice(),
            crate::RESERVED_NAME_MAIN,
        ) else {
            continue;
        };

        if T::ReservedAccountChecker::is_reserved(&account) {
            continue;
        }
        if T::ProtectedSourceChecker::is_protected(&account) {
            continue;
        }
        if !T::AccountValidator::is_valid(&account) {
            continue;
        }

        return Ok((cid_number, account));
    }

    Err(BenchmarkError::Stop(
        "failed to find a benchmark-safe cid id",
    ))
}

fn bench_account_name<T: Config>() -> Result<AccountNameOf<T>, BenchmarkError> {
    b"Benchmark Institution"
        .to_vec()
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark account_name should fit"))
}

fn issuer_cid_number() -> Vec<u8> {
    b"BENCH-ISSUER".to_vec()
}

fn scope_province_name() -> Vec<u8> {
    b"BENCH-PROVINCE".to_vec()
}

fn scope_city_name() -> Vec<u8> {
    b"BENCH-CITY".to_vec()
}

#[benchmarks(where T: Config)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_cid_private_institution() -> Result<(), BenchmarkError> {
        let relayer: T::AccountId = frame_benchmarking::account("relayer", 0, 0);

        let (cid_number, account) = find_safe_cid::<T>()?;
        let account_name = bench_account_name::<T>()?;
        let register_nonce: RegisterNonceOf<T> = b"bench-register-nonce"
            .to_vec()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark register nonce should fit"))?;
        let signature: RegisterSignatureOf<T> = vec![1u8; 64]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark register signature should fit"))?;
        let account_names: InstitutionAccountNamesOf<T> = vec![account_name.clone()]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark account_names should fit"))?;

        #[extrinsic_call]
        register_cid_private_institution(
            RawOrigin::Signed(relayer.clone()),
            cid_number.clone(),
            account_name.clone(),
            account_names,
            register_nonce,
            signature,
            issuer_cid_number(),
            relayer.clone(),
            [1u8; 32],
            scope_province_name(),
            scope_city_name(),
        );

        assert_eq!(
            CidRegisteredAccount::<T>::get(&cid_number, &account_name),
            Some(account.clone())
        );
        assert!(AccountRegisteredCid::<T>::contains_key(&account));
        Ok(())
    }

    // 当前 private-manage 仅保留 register_cid_private_institution benchmark。
    // propose_create_private_institution / propose_close / cleanup_rejected_private_proposal
    // 需补齐真实投票流水线 fixture 后再生成正式权重。
}
