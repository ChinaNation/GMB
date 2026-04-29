//! 多签交易模块 Benchmark 定义。
//!
//! Phase 2 整改后投票统一走 `voting-engine-system::internal_vote`,本模块不再有
//! `finalize_create` / `vote_close` extrinsic。对应的 benchmark 已删除。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec;
use voting_engine_system::STATUS_PASSED;

use crate::{
    pallet::{
        AccountNameOf, AddressRegisteredSfid, DuoqianAccounts, DuoqianAdminsOf, RegisterNonceOf,
        RegisterSignatureOf, SfidIdOf, SfidRegisteredAddress,
    },
    BalanceOf, Call, Config, DuoqianAddressValidator, DuoqianReservedAddressChecker, Pallet,
    ProtectedSourceChecker,
};

fn find_safe_sfid<T: Config>() -> Result<(SfidIdOf<T>, T::AccountId), BenchmarkError> {
    for candidate in 0..2_048u32 {
        let mut raw = b"duoqian-benchmark-sfid-".to_vec();
        raw.extend_from_slice(&candidate.to_le_bytes());
        let sfid_id: SfidIdOf<T> = raw
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark sfid id should fit"))?;

        // benchmark 场景用 Role::Main 派生，哈希公式等价于历史空 account_name 路径。
        let Ok(duoqian_address) = Pallet::<T>::derive_institution_address(
            sfid_id.as_slice(),
            crate::InstitutionAccountRole::Main,
        ) else {
            continue;
        };

        if T::ReservedAddressChecker::is_reserved(&duoqian_address) {
            continue;
        }
        if T::ProtectedSourceChecker::is_protected(&duoqian_address) {
            continue;
        }
        if !T::AddressValidator::is_valid(&duoqian_address) {
            continue;
        }

        return Ok((sfid_id, duoqian_address));
    }

    Err(BenchmarkError::Stop(
        "failed to find a benchmark-safe sfid id",
    ))
}

fn bench_account_name<T: Config>() -> Result<AccountNameOf<T>, BenchmarkError> {
    b"Benchmark Institution"
        .to_vec()
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark account_name should fit"))
}

fn register_institution<T: Config>(
    relayer: &T::AccountId,
    sfid_id: &SfidIdOf<T>,
) -> Result<T::AccountId, BenchmarkError> {
    let account_name = bench_account_name::<T>()?;
    let register_nonce: RegisterNonceOf<T> = b"bench-register-nonce"
        .to_vec()
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark register nonce should fit"))?;
    let signature: RegisterSignatureOf<T> = vec![1u8; 64]
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark register signature should fit"))?;
    Pallet::<T>::register_sfid_institution(
        RawOrigin::Signed(relayer.clone()).into(),
        sfid_id.clone(),
        account_name.clone(),
        register_nonce,
        signature,
        None,
    )?;
    SfidRegisteredAddress::<T>::get(sfid_id, &account_name)
        .ok_or(BenchmarkError::Stop("benchmark sfid should be registered"))
}

fn find_safe_beneficiary<T: Config>(
    duoqian_address: &T::AccountId,
) -> Result<T::AccountId, BenchmarkError> {
    for index in 0..64u32 {
        let beneficiary: T::AccountId = frame_benchmarking::account("beneficiary", index, 0);
        if &beneficiary == duoqian_address {
            continue;
        }
        if T::ReservedAddressChecker::is_reserved(&beneficiary) {
            continue;
        }
        if T::ProtectedSourceChecker::is_protected(&beneficiary) {
            continue;
        }
        if !T::AddressValidator::is_valid(&beneficiary) {
            continue;
        }
        return Ok(beneficiary);
    }

    Err(BenchmarkError::Stop(
        "failed to find a benchmark-safe beneficiary",
    ))
}

/// Benchmark 辅助:让指定提案通过(绕开投票路径,benchmark 只关心后续业务执行开销)。
fn pass_proposal<T: Config>(proposal_id: u64) -> Result<(), BenchmarkError> {
    voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_PASSED)
        .map_err(|_| BenchmarkError::Stop("benchmark: set_status_and_emit PASSED failed"))?;
    Ok(())
}

#[benchmarks(where
    T: Config,
    <T as frame_system::Config>::AccountId: Decode,
    BalanceOf<T>: Ord + sp_runtime::traits::Saturating + Copy,
)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_sfid_institution() -> Result<(), BenchmarkError> {
        let relayer: T::AccountId = frame_benchmarking::account("relayer", 0, 0);

        let (sfid_id, duoqian_address) = find_safe_sfid::<T>()?;
        let account_name = bench_account_name::<T>()?;
        let register_nonce: RegisterNonceOf<T> = b"bench-register-nonce"
            .to_vec()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark register nonce should fit"))?;
        let signature: RegisterSignatureOf<T> = vec![1u8; 64]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark register signature should fit"))?;

        #[extrinsic_call]
        register_sfid_institution(
            RawOrigin::Signed(relayer.clone()),
            sfid_id.clone(),
            account_name.clone(),
            register_nonce,
            signature,
            None,
        );

        assert_eq!(
            SfidRegisteredAddress::<T>::get(&sfid_id, &account_name),
            Some(duoqian_address.clone())
        );
        assert!(AddressRegisteredSfid::<T>::contains_key(&duoqian_address));
        Ok(())
    }

    #[benchmark]
    fn propose_create() -> Result<(), BenchmarkError> {
        let relayer: T::AccountId = frame_benchmarking::account("relayer", 1, 0);

        let (sfid_id, _) = find_safe_sfid::<T>()?;
        let duoqian_address = register_institution::<T>(&relayer, &sfid_id)?;

        let admin1: T::AccountId = frame_benchmarking::account("admin", 0, 0);
        let admin2: T::AccountId = frame_benchmarking::account("admin", 1, 0);
        let admin3: T::AccountId = frame_benchmarking::account("admin", 2, 0);

        let admins: DuoqianAdminsOf<T> = vec![admin1.clone(), admin2.clone(), admin3.clone()]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark admins should fit"))?;

        let amount: BalanceOf<T> = 1_000u128.saturated_into();
        let funding: BalanceOf<T> = 1_000_000u128.saturated_into();
        let _ = T::Currency::deposit_creating(&admin1, funding);

        let account_name = bench_account_name::<T>()?;

        #[extrinsic_call]
        propose_create(
            RawOrigin::Signed(admin1.clone()),
            sfid_id,
            account_name,
            3,
            admins,
            2,
            amount,
        );

        assert!(DuoqianAccounts::<T>::contains_key(&duoqian_address));
        assert!(voting_engine_system::Pallet::<T>::get_proposal_data(0).is_some());
        Ok(())
    }

    #[benchmark]
    fn propose_close() -> Result<(), BenchmarkError> {
        let relayer: T::AccountId = frame_benchmarking::account("relayer", 3, 0);

        let (sfid_id, duoqian_address) = find_safe_sfid::<T>()?;
        let _ = register_institution::<T>(&relayer, &sfid_id)?;

        let admin1: T::AccountId = frame_benchmarking::account("admin", 20, 0);
        let admin2: T::AccountId = frame_benchmarking::account("admin", 21, 0);

        let admins: DuoqianAdminsOf<T> = vec![admin1.clone(), admin2.clone()]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark admins should fit"))?;

        let amount: BalanceOf<T> = 1_000u128.saturated_into();
        let funding: BalanceOf<T> = 1_000_000u128.saturated_into();
        let _ = T::Currency::deposit_creating(&admin1, funding);

        let account_name = bench_account_name::<T>()?;

        // Create 提案 → 推到 PASSED 触发 Executor.execute_create → DuoqianAccounts 激活。
        assert!(Pallet::<T>::propose_create(
            RawOrigin::Signed(admin1.clone()).into(),
            sfid_id,
            account_name,
            2,
            admins,
            2,
            amount,
        )
        .is_ok());
        pass_proposal::<T>(0)?;

        let beneficiary = find_safe_beneficiary::<T>(&duoqian_address)?;

        #[extrinsic_call]
        propose_close(
            RawOrigin::Signed(admin1),
            duoqian_address.clone(),
            beneficiary,
        );

        assert!(voting_engine_system::Pallet::<T>::get_proposal_data(1).is_some());
        let _ = admin2; // avoid unused warning for admin2
        Ok(())
    }

    #[benchmark]
    fn propose_create_personal() -> Result<(), BenchmarkError> {
        let admin1: T::AccountId = frame_benchmarking::account("admin", 40, 0);
        let admin2: T::AccountId = frame_benchmarking::account("admin", 41, 0);

        let admins: DuoqianAdminsOf<T> = vec![admin1.clone(), admin2.clone()]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark admins should fit"))?;

        let amount: BalanceOf<T> = 1_000u128.saturated_into();
        let funding: BalanceOf<T> = 1_000_000u128.saturated_into();
        let _ = T::Currency::deposit_creating(&admin1, funding);

        let account_name: AccountNameOf<T> = b"Benchmark Personal"
            .to_vec()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark account_name should fit"))?;

        #[extrinsic_call]
        propose_create_personal(
            RawOrigin::Signed(admin1.clone()),
            account_name,
            2,
            admins,
            2,
            amount,
        );

        assert!(voting_engine_system::Pallet::<T>::get_proposal_data(0).is_some());
        Ok(())
    }
}
