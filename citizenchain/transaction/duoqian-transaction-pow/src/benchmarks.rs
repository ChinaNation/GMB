//! 多签交易模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::{Decode, Encode};
use frame_benchmarking::v2::*;
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use rand_core::{CryptoRng, Error as RandError, RngCore};
use schnorrkel::{ExpansionMode, Keypair, MiniSecretKey};
use sfid_code_auth::SfidMainAccount;
use sp_core::sr25519;
use sp_runtime::{
    traits::{IdentifyAccount, SaturatedConversion, Saturating, Zero},
    MultiSigner,
};
use sp_std::vec::Vec;

use crate::{
    pallet::{AddressRegisteredSfid, DuoqianAccounts, SfidRegisteredAddress},
    AdminApproval, AdminApprovalsOf, BalanceOf, Call, Config,
    DuoqianAddressValidator, DuoqianAdminAuth, DuoqianAdminsOf, DuoqianReservedAddressChecker,
    Pallet, ProtectedSourceChecker, SfidIdOf,
};

type AdminPublicKeyOf<T> = <<T as Config>::AdminAuth as DuoqianAdminAuth<
    <T as frame_system::Config>::AccountId,
>>::PublicKey;
type AdminSignatureOf<T> = <<T as Config>::AdminAuth as DuoqianAdminAuth<
    <T as frame_system::Config>::AccountId,
>>::Signature;

struct GeneratedAdmin<T: Config> {
    keypair: Keypair,
    public_key: AdminPublicKeyOf<T>,
    account: T::AccountId,
}

fn chain_domain_prefix<T: Config>() -> [u8; 2] {
    T::SS58Prefix::get().to_le_bytes()
}

fn account_from_public<T: Config>(public: sr25519::Public) -> T::AccountId
where
    T::AccountId: Decode,
{
    let encoded = MultiSigner::from(public).into_account().encode();
    T::AccountId::decode(&mut &encoded[..]).expect("benchmark account must decode")
}

fn effective_threshold(admin_count: u32) -> u32 {
    core::cmp::max(2, admin_count.saturating_add(1) / 2)
}

fn required_duoqian_amount<T: Config>() -> BalanceOf<T>
where
    BalanceOf<T>: Ord + Copy,
{
    core::cmp::max(T::MinCreateAmount::get(), T::MinCloseBalance::get())
}

struct ZeroRng;

impl RngCore for ZeroRng {
    fn next_u32(&mut self) -> u32 {
        0
    }

    fn next_u64(&mut self) -> u64 {
        0
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest.iter_mut() {
            *byte = 0;
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RandError> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for ZeroRng {}

fn sign_payload(keypair: &Keypair, payload: &[u8]) -> [u8; 64] {
    let transcript = schnorrkel::signing_context(b"substrate").bytes(payload);
    keypair
        .sign(schnorrkel::context::attach_rng(transcript, ZeroRng))
        .to_bytes()
}

fn find_safe_sfid<T: Config>() -> Result<(SfidIdOf<T>, T::AccountId), BenchmarkError> {
    let _domain = chain_domain_prefix::<T>();

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

fn generate_admins<T: Config>(admin_count: u32) -> Result<Vec<GeneratedAdmin<T>>, BenchmarkError>
where
    T::AccountId: Decode,
    AdminPublicKeyOf<T>: From<[u8; 32]>,
{
    let mut admins = Vec::with_capacity(admin_count as usize);
    let mut attempt = 0u32;

    while admins.len() < admin_count as usize {
        let mut seed = [0u8; 32];
        seed[..4].copy_from_slice(&attempt.to_le_bytes());
        let keypair = MiniSecretKey::from_bytes(&seed)
            .map_err(|_| BenchmarkError::Stop("benchmark seed should be valid"))?
            .expand_to_keypair(ExpansionMode::Ed25519);
        let public = sr25519::Public::from(keypair.public.to_bytes());
        let public_key = AdminPublicKeyOf::<T>::from(public.0);
        attempt = attempt.saturating_add(1);

        if !T::AdminAuth::is_valid_public_key(&public_key) {
            continue;
        }

        admins.push(GeneratedAdmin {
            account: account_from_public::<T>(public),
            keypair,
            public_key,
        });
    }

    Ok(admins)
}

fn build_admin_fixture<T: Config>(
    admin_count: u32,
    approval_count: u32,
    payload: &[u8],
) -> Result<(T::AccountId, DuoqianAdminsOf<T>, AdminApprovalsOf<T>), BenchmarkError>
where
    T::AccountId: Decode,
    AdminPublicKeyOf<T>: From<[u8; 32]>,
    AdminSignatureOf<T>: From<[u8; 64]>,
{
    let generated = generate_admins::<T>(admin_count)?;
    let caller = generated
        .first()
        .ok_or(BenchmarkError::Stop(
            "benchmark requires at least one admin",
        ))?
        .account
        .clone();

    let admins: DuoqianAdminsOf<T> = generated
        .iter()
        .map(|item| item.public_key.clone())
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark admin list should fit"))?;

    let mut approvals_vec = Vec::with_capacity(approval_count as usize);
    for item in generated.iter().take(approval_count as usize) {
        approvals_vec.push(AdminApproval {
            public_key: item.public_key.clone(),
            signature: AdminSignatureOf::<T>::from(sign_payload(&item.keypair, payload)),
        });
    }

    let approvals: AdminApprovalsOf<T> = approvals_vec
        .try_into()
        .map_err(|_| BenchmarkError::Stop("benchmark approvals should fit"))?;

    Ok((caller, admins, approvals))
}

fn register_institution<T: Config>(
    operator: &T::AccountId,
    sfid_id: &SfidIdOf<T>,
) -> Result<T::AccountId, BenchmarkError> {
    Pallet::<T>::register_sfid_institution(
        RawOrigin::Signed(operator.clone()).into(),
        sfid_id.clone(),
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
    T: Config + sfid_code_auth::Config,
    <T as frame_system::Config>::AccountId: Decode,
    AdminPublicKeyOf<T>: From<[u8; 32]>,
    AdminSignatureOf<T>: From<[u8; 64]>,
    BalanceOf<T>: Ord + Saturating + Copy,
)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_sfid_institution() -> Result<(), BenchmarkError> {
        let _domain = chain_domain_prefix::<T>();
        let operator: T::AccountId = frame_benchmarking::account("operator", 0, 0);
        SfidMainAccount::<T>::put(&operator);

        let (sfid_id, duoqian_address) = find_safe_sfid::<T>()?;

        #[extrinsic_call]
        register_sfid_institution(RawOrigin::Signed(operator.clone()), sfid_id.clone());

        assert_eq!(
            SfidRegisteredAddress::<T>::get(&sfid_id),
            Some(duoqian_address.clone())
        );
        assert!(AddressRegisteredSfid::<T>::contains_key(&duoqian_address));
        Ok(())
    }

    #[benchmark]
    fn create_duoqian(
        a: Linear<2, { T::MaxAdmins::get() }>,
        s: Linear<2, { T::MaxAdmins::get() }>,
    ) -> Result<(), BenchmarkError> {
        let _domain = chain_domain_prefix::<T>();
        let operator: T::AccountId = frame_benchmarking::account("operator", 1, 0);
        SfidMainAccount::<T>::put(&operator);

        let (sfid_id, _) = find_safe_sfid::<T>()?;
        let duoqian_address = register_institution::<T>(&operator, &sfid_id)?;
        let now = frame_system::Pallet::<T>::block_number();
        let expires_at = now.saturating_add(10u32.saturated_into());

        let admin_count = a;
        let approval_count = s;
        let effective_approval_count =
            core::cmp::max(approval_count, effective_threshold(admin_count));
        let effective_admin_count = core::cmp::max(admin_count, effective_approval_count);
        let threshold = effective_threshold(effective_admin_count);
        let amount = required_duoqian_amount::<T>();

        let admin_shell = generate_admins::<T>(effective_admin_count)?;
        let caller = admin_shell
            .first()
            .ok_or(BenchmarkError::Stop("benchmark requires caller admin"))?
            .account
            .clone();
        let payload_admins: DuoqianAdminsOf<T> = admin_shell
            .iter()
            .map(|item| item.public_key.clone())
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark admins should fit"))?;

        let funding = amount
            .saturating_add(amount)
            .saturating_add(T::Currency::minimum_balance());
        let _ = T::Currency::deposit_creating(&caller, funding);

        let payload = (
            b"DUOQIAN_CREATE_V3".to_vec(),
            chain_domain_prefix::<T>(),
            0u64,
            expires_at,
            &sfid_id,
            &duoqian_address,
            &caller,
            effective_admin_count,
            &payload_admins,
            threshold,
            amount,
        )
            .encode();

        let (caller, duoqian_admins, approvals) =
            build_admin_fixture::<T>(effective_admin_count, effective_approval_count, &payload)?;

        #[extrinsic_call]
        create_duoqian(
            RawOrigin::Signed(caller.clone()),
            sfid_id.clone(),
            effective_admin_count,
            duoqian_admins.clone(),
            threshold,
            amount,
            expires_at,
            approvals.clone(),
        );

        assert!(DuoqianAccounts::<T>::contains_key(&duoqian_address));
        assert_eq!(
            AddressRegisteredSfid::<T>::get(&duoqian_address)
                .ok_or(BenchmarkError::Stop(
                    "benchmark registered institution should exist"
                ))?
                .nonce,
            1u64
        );
        Ok(())
    }

    #[benchmark]
    fn close_duoqian(
        a: Linear<2, { T::MaxAdmins::get() }>,
        s: Linear<2, { T::MaxAdmins::get() }>,
    ) -> Result<(), BenchmarkError> {
        let _domain = chain_domain_prefix::<T>();
        let operator: T::AccountId = frame_benchmarking::account("operator", 2, 0);
        SfidMainAccount::<T>::put(&operator);

        let (sfid_id, _) = find_safe_sfid::<T>()?;
        let duoqian_address = register_institution::<T>(&operator, &sfid_id)?;
        let now = frame_system::Pallet::<T>::block_number();
        let expires_at = now.saturating_add(10u32.saturated_into());

        let admin_count = a;
        let approval_count = s;
        let effective_approval_count =
            core::cmp::max(approval_count, effective_threshold(admin_count));
        let effective_admin_count = core::cmp::max(admin_count, effective_approval_count);
        let threshold = effective_threshold(effective_admin_count);
        let amount = required_duoqian_amount::<T>();

        let admin_shell = generate_admins::<T>(effective_admin_count)?;
        let caller = admin_shell
            .first()
            .ok_or(BenchmarkError::Stop("benchmark requires caller admin"))?
            .account
            .clone();
        let payload_admins: DuoqianAdminsOf<T> = admin_shell
            .iter()
            .map(|item| item.public_key.clone())
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("benchmark admins should fit"))?;
        let funding = amount
            .saturating_add(amount)
            .saturating_add(T::Currency::minimum_balance());
        let _ = T::Currency::deposit_creating(&caller, funding);

        let create_payload = (
            b"DUOQIAN_CREATE_V3".to_vec(),
            chain_domain_prefix::<T>(),
            0u64,
            expires_at,
            &sfid_id,
            &duoqian_address,
            &caller,
            effective_admin_count,
            &payload_admins,
            threshold,
            amount,
        )
            .encode();
        let (caller, duoqian_admins, create_approvals) = build_admin_fixture::<T>(
            effective_admin_count,
            effective_approval_count,
            &create_payload,
        )?;

        Pallet::<T>::create_duoqian(
            RawOrigin::Signed(caller.clone()).into(),
            sfid_id,
            effective_admin_count,
            duoqian_admins.clone(),
            threshold,
            amount,
            expires_at,
            create_approvals,
        )?;

        let beneficiary = find_safe_beneficiary::<T>(&duoqian_address)?;
        let min_balance: BalanceOf<T> = Zero::zero();
        let close_payload = (
            b"DUOQIAN_CLOSE_V3".to_vec(),
            chain_domain_prefix::<T>(),
            1u64,
            expires_at,
            &duoqian_address,
            &beneficiary,
            &caller,
            &duoqian_admins,
            effective_admin_count,
            threshold,
            min_balance,
        )
            .encode();
        let (_, _, approvals) = build_admin_fixture::<T>(
            effective_admin_count,
            effective_approval_count,
            &close_payload,
        )?;

        #[extrinsic_call]
        close_duoqian(
            RawOrigin::Signed(caller.clone()),
            duoqian_address.clone(),
            beneficiary.clone(),
            min_balance,
            expires_at,
            approvals.clone(),
        );

        assert!(!DuoqianAccounts::<T>::contains_key(&duoqian_address));
        assert_eq!(
            AddressRegisteredSfid::<T>::get(&duoqian_address)
                .ok_or(BenchmarkError::Stop(
                    "benchmark registered institution should exist"
                ))?
                .nonce,
            2u64
        );
        Ok(())
    }
}
