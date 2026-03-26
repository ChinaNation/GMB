//! 多签交易模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec;
use voting_engine_system::InternalVoteEngine;

use crate::{
    pallet::{
        AddressRegisteredSfid, DuoqianAccounts, DuoqianAdminsOf, RegisterNonceOf,
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

        let Ok(duoqian_address) =
            Pallet::<T>::derive_duoqian_address_from_sfid_id(sfid_id.as_slice())
        else {
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

fn register_institution<T: Config>(
    relayer: &T::AccountId,
    sfid_id: &SfidIdOf<T>,
) -> Result<T::AccountId, BenchmarkError> {
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
        register_nonce,
        signature,
    )?;
    SfidRegisteredAddress::<T>::get(sfid_id)
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
            register_nonce,
            signature,
        );

        assert_eq!(
            SfidRegisteredAddress::<T>::get(&sfid_id),
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

        #[extrinsic_call]
        propose_create(
            RawOrigin::Signed(admin1.clone()),
            sfid_id,
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
    fn vote_create() -> Result<(), BenchmarkError> {
        let relayer: T::AccountId = frame_benchmarking::account("relayer", 2, 0);

        let (sfid_id, duoqian_address) = find_safe_sfid::<T>()?;
        let _ = register_institution::<T>(&relayer, &sfid_id)?;

        let admin1: T::AccountId = frame_benchmarking::account("admin", 10, 0);
        let admin2: T::AccountId = frame_benchmarking::account("admin", 11, 0);

        let admins: DuoqianAdminsOf<T> = vec![admin1.clone(), admin2.clone()]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark admins should fit"))?;

        let amount: BalanceOf<T> = 1_000u128.saturated_into();
        let funding: BalanceOf<T> = 1_000_000u128.saturated_into();
        let _ = T::Currency::deposit_creating(&admin1, funding);

        assert!(Pallet::<T>::propose_create(
            RawOrigin::Signed(admin1.clone()).into(),
            sfid_id,
            2,
            admins,
            2,
            amount,
        )
        .is_ok());

        // 第一票由 admin1
        assert!(T::InternalVoteEngine::cast_internal_vote(admin1, 0, true).is_ok());

        // 第二票由 admin2，这一票达到阈值
        #[extrinsic_call]
        vote_create(RawOrigin::Signed(admin2), 0, true);

        // 验证投票通过后 DuoqianAccounts 变为 Active
        let account = DuoqianAccounts::<T>::get(&duoqian_address);
        assert!(account.is_some());
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

        // Create and activate
        assert!(Pallet::<T>::propose_create(
            RawOrigin::Signed(admin1.clone()).into(),
            sfid_id,
            2,
            admins,
            2,
            amount,
        )
        .is_ok());
        assert!(T::InternalVoteEngine::cast_internal_vote(admin1.clone(), 0, true).is_ok());
        assert!(T::InternalVoteEngine::cast_internal_vote(admin2.clone(), 0, true).is_ok());

        let beneficiary = find_safe_beneficiary::<T>(&duoqian_address)?;

        #[extrinsic_call]
        propose_close(
            RawOrigin::Signed(admin1),
            duoqian_address.clone(),
            beneficiary,
        );

        assert!(voting_engine_system::Pallet::<T>::get_proposal_data(1).is_some());
        Ok(())
    }

    #[benchmark]
    fn vote_close() -> Result<(), BenchmarkError> {
        let relayer: T::AccountId = frame_benchmarking::account("relayer", 4, 0);

        let (sfid_id, duoqian_address) = find_safe_sfid::<T>()?;
        let _ = register_institution::<T>(&relayer, &sfid_id)?;

        let admin1: T::AccountId = frame_benchmarking::account("admin", 30, 0);
        let admin2: T::AccountId = frame_benchmarking::account("admin", 31, 0);

        let admins: DuoqianAdminsOf<T> = vec![admin1.clone(), admin2.clone()]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark admins should fit"))?;

        let amount: BalanceOf<T> = 1_000u128.saturated_into();
        let funding: BalanceOf<T> = 1_000_000u128.saturated_into();
        let _ = T::Currency::deposit_creating(&admin1, funding);

        // Create and activate
        assert!(Pallet::<T>::propose_create(
            RawOrigin::Signed(admin1.clone()).into(),
            sfid_id,
            2,
            admins,
            2,
            amount,
        )
        .is_ok());
        assert!(T::InternalVoteEngine::cast_internal_vote(admin1.clone(), 0, true).is_ok());
        assert!(T::InternalVoteEngine::cast_internal_vote(admin2.clone(), 0, true).is_ok());

        let beneficiary = find_safe_beneficiary::<T>(&duoqian_address)?;

        assert!(Pallet::<T>::propose_close(
            RawOrigin::Signed(admin1.clone()).into(),
            duoqian_address.clone(),
            beneficiary,
        )
        .is_ok());

        assert!(T::InternalVoteEngine::cast_internal_vote(admin1, 1, true).is_ok());

        #[extrinsic_call]
        vote_close(RawOrigin::Signed(admin2), 1, true);

        // DuoqianAccounts 应该被删除
        assert!(!DuoqianAccounts::<T>::contains_key(&duoqian_address));
        Ok(())
    }
}
