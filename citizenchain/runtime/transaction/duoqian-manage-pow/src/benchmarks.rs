//! 多签交易模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_core::{sr25519, Pair};
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec;
use voting_engine_system::InternalVoteEngine;

use crate::{
    pallet::{
        AccountNameOf, AddressRegisteredSfid, AdminSignatureOf, AdminSignaturesOf, DuoqianAccounts,
        DuoqianAdminsOf, RegisterNonceOf, RegisterSignatureOf, SfidIdOf, SfidRegisteredAddress,
    },
    BalanceOf, Call, Config, CreateVoteIntent, DuoqianAddressValidator,
    DuoqianReservedAddressChecker, Pallet, ProtectedSourceChecker,
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

/// Benchmark 辅助:用 sr25519::Pair 派生管理员 (AccountId, Pair),
/// 使 `finalize_create` 能构造真实可验证的 sr25519 签名。
fn bench_admin_with_key<T: Config>(seed: u8) -> Result<(T::AccountId, sr25519::Pair), BenchmarkError>
where
    T::AccountId: Decode,
{
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0] = seed;
    let pair = sr25519::Pair::from_seed(&seed_bytes);
    let pubkey = pair.public();
    let account = T::AccountId::decode(&mut &pubkey.0[..])
        .map_err(|_| BenchmarkError::Stop("pubkey should decode to AccountId"))?;
    Ok((account, pair))
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

    /// `finalize_create` benchmark:2/2 阈值,两个 sr25519 管理员签名聚合。
    ///
    /// 用真实 `sr25519::Pair` 派生管理员账户,签出可验证的 `CreateVoteIntent` 签名,
    /// 测量本 pallet 循环验签 + 代投 + 自动 `execute_create` 的总权重。
    #[benchmark]
    fn finalize_create() -> Result<(), BenchmarkError> {
        let relayer: T::AccountId = frame_benchmarking::account("relayer", 2, 0);
        let (sfid_id, duoqian_address) = find_safe_sfid::<T>()?;
        let _ = register_institution::<T>(&relayer, &sfid_id)?;

        let (admin1, pair1) = bench_admin_with_key::<T>(201)?;
        let (admin2, pair2) = bench_admin_with_key::<T>(202)?;
        let admins: DuoqianAdminsOf<T> = vec![admin1.clone(), admin2.clone()]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark admins should fit"))?;

        let amount: BalanceOf<T> = 1_000u128.saturated_into();
        let funding: BalanceOf<T> = 1_000_000u128.saturated_into();
        let _ = T::Currency::deposit_creating(&admin1, funding);

        let account_name = bench_account_name::<T>()?;

        assert!(Pallet::<T>::propose_create(
            RawOrigin::Signed(admin1.clone()).into(),
            sfid_id,
            account_name,
            2,
            admins.clone(),
            2,
            amount,
        )
        .is_ok());

        let proposal_id = 0u64;

        // 构造 intent 并让两管理员签名
        let admins_root = Pallet::<T>::compute_admins_root(&admins);
        let intent = CreateVoteIntent::<T::AccountId, BalanceOf<T>> {
            proposal_id,
            duoqian_address: duoqian_address.clone(),
            creator: admin1.clone(),
            admins_root,
            threshold: 2,
            amount,
            approve: true,
        };
        let msg = intent.signing_hash(T::SS58Prefix::get());
        let sig1: AdminSignatureOf<T> = pair1
            .sign(&msg)
            .0
            .to_vec()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("sig1 should fit"))?;
        let sig2: AdminSignatureOf<T> = pair2
            .sign(&msg)
            .0
            .to_vec()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("sig2 should fit"))?;
        let sigs: AdminSignaturesOf<T> = vec![(admin1.clone(), sig1), (admin2.clone(), sig2)]
            .try_into()
            .map_err(|_| BenchmarkError::Stop("sigs vec should fit"))?;

        #[extrinsic_call]
        finalize_create(RawOrigin::Signed(admin1), proposal_id, sigs);

        // 投票通过后 DuoqianAccounts 应变为 Active
        let account =
            DuoqianAccounts::<T>::get(&duoqian_address).expect("duoqian account should exist");
        assert_eq!(account.status, crate::DuoqianStatus::Active);
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

        // Create and activate via trait(benchmark 里不走 finalize_create 路径,直接让投票引擎代投)
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

        let account_name = bench_account_name::<T>()?;

        // Create and activate
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
