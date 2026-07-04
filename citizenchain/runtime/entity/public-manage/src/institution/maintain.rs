//! 机构信息维护:改名(`do_update_institution_info`)+ 新增账户
//! (`do_add_institution_account`)。
//!
//! 链是机构信息唯一真源(ADR-031);创世只铸定初始集,今后改名/加账户走交易。
//! 注册局授权凭证与登记同一套(复用 `verify_institution_registration` +
//! `RegistryAuthority` + `UsedRegisterNonce` 防重放),不引入新签名域:
//! - 改名 payload = cid + 新全称 + 新简称 + 空账户名 + nonce;
//! - 加账户 payload = cid + 机构现全称 + 机构现简称 + 新账户名列表 + nonce。
//! 机构码/CID/省市码物理编码在 CID 里,改不了也不给参数。

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use frame_support::ensure;
use sp_runtime::{
    traits::{Hash, Zero},
    DispatchResult,
};

use crate::institution::types::{InstitutionAccountInfo, InstitutionLifecycleStatus};
use crate::pallet::{
    self, AccountNameOf, AccountRegisteredCid, CidNumberOf, CidRegisteredAccount, Error, Event,
    InstitutionAccountNamesOf, InstitutionAccounts, Institutions, Pallet, RegisterNonceOf,
    RegisterSignatureOf, UsedRegisterNonce,
};
use crate::traits::{
    AccountValidator, CidInstitutionVerifier, ProtectedSourceChecker, RegistryAuthority,
    ReservedAccountGuard,
};
use crate::{BalanceOf, RegisteredInstitution};

/// 注册局改机构全称/简称:链是唯一真源;机构码/CID/省市码不可改故不给参数。
#[allow(clippy::too_many_arguments)]
pub(crate) fn do_update_institution_info<T: pallet::Config>(
    submitter: T::AccountId,
    cid_number: CidNumberOf<T>,
    cid_full_name: AccountNameOf<T>,
    cid_short_name: AccountNameOf<T>,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    issuer_cid_number: Vec<u8>,
    issuer_main_account: T::AccountId,
    signer_pubkey: [u8; 32],
    scope_province_name: Vec<u8>,
    scope_city_name: Vec<u8>,
) -> DispatchResult {
    ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
    ensure!(!cid_full_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!cid_short_name.is_empty(), Error::<T>::EmptyAccountName);

    let info = Institutions::<T>::get(&cid_number).ok_or(Error::<T>::InstitutionNotFound)?;
    ensure!(
        info.status != InstitutionLifecycleStatus::Closed,
        Error::<T>::InstitutionAlreadyClosed
    );

    let nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );
    // 复用登记验签:payload = cid + 新全称 + 新简称 + 空账户名 + nonce。
    ensure!(
        T::CidInstitutionVerifier::verify_institution_registration(
            cid_number.as_slice(),
            &cid_full_name,
            cid_short_name.as_slice(),
            &[],
            &register_nonce,
            &signature,
            issuer_cid_number.as_slice(),
            &issuer_main_account,
            &signer_pubkey,
            scope_province_name.as_slice(),
            scope_city_name.as_slice(),
            &[],
        ),
        Error::<T>::InvalidCidInstitutionSignature
    );
    ensure!(
        T::RegistryAuthority::can_register_institution(
            &submitter,
            issuer_cid_number.as_slice(),
            &issuer_main_account,
            &signer_pubkey,
            cid_number.as_slice(),
            info.institution_code,
            scope_province_name.as_slice(),
            scope_city_name.as_slice(),
        ),
        Error::<T>::RegistryAuthorityDenied
    );

    UsedRegisterNonce::<T>::insert(nonce_hash, true);
    Institutions::<T>::mutate(&cid_number, |maybe| {
        if let Some(info) = maybe {
            info.cid_full_name = cid_full_name.clone();
            info.cid_short_name = cid_short_name.clone();
        }
    });
    Pallet::<T>::deposit_event(Event::<T>::InstitutionInfoUpdated {
        cid_number,
        cid_full_name,
        cid_short_name,
        submitter,
    });
    Ok(())
}

/// 给已存在机构新增账户:新账户名 → 确定性派生地址 → 写三索引 + 账户表。
/// 复用登记的账户校验链(保留名/重复/占用/保护);新账户初始余额 0。
#[allow(clippy::too_many_arguments)]
pub(crate) fn do_add_institution_account<T: pallet::Config>(
    submitter: T::AccountId,
    cid_number: CidNumberOf<T>,
    account_names: InstitutionAccountNamesOf<T>,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    issuer_cid_number: Vec<u8>,
    issuer_main_account: T::AccountId,
    signer_pubkey: [u8; 32],
    scope_province_name: Vec<u8>,
    scope_city_name: Vec<u8>,
) -> DispatchResult {
    ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
    ensure!(!account_names.is_empty(), Error::<T>::MissingMainAccount);

    let info = Institutions::<T>::get(&cid_number).ok_or(Error::<T>::InstitutionNotFound)?;
    ensure!(
        info.status == InstitutionLifecycleStatus::Active,
        Error::<T>::InstitutionAlreadyClosed
    );

    let nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );
    // 复用登记验签:payload = cid + 机构现全称 + 机构现简称 + 新账户名列表 + nonce。
    let account_name_payload = Pallet::<T>::account_names_payload_from_names(&account_names)?;
    ensure!(
        T::CidInstitutionVerifier::verify_institution_registration(
            cid_number.as_slice(),
            &info.cid_full_name,
            info.cid_short_name.as_slice(),
            &account_name_payload,
            &register_nonce,
            &signature,
            issuer_cid_number.as_slice(),
            &issuer_main_account,
            &signer_pubkey,
            scope_province_name.as_slice(),
            scope_city_name.as_slice(),
            &[],
        ),
        Error::<T>::InvalidCidInstitutionSignature
    );
    ensure!(
        T::RegistryAuthority::can_register_institution(
            &submitter,
            issuer_cid_number.as_slice(),
            &issuer_main_account,
            &signer_pubkey,
            cid_number.as_slice(),
            info.institution_code,
            scope_province_name.as_slice(),
            scope_city_name.as_slice(),
        ),
        Error::<T>::RegistryAuthorityDenied
    );

    // 派生 + 校验(与登记同链):保留名/重复/占用/非法/保护。
    let mut derived: Vec<(AccountNameOf<T>, T::AccountId)> =
        Vec::with_capacity(account_names.len());
    let mut seen = BTreeSet::<Vec<u8>>::new();
    for account_name in account_names.iter() {
        ensure!(!account_name.is_empty(), Error::<T>::EmptyAccountName);
        ensure!(
            seen.insert(account_name.as_slice().to_vec()),
            Error::<T>::DuplicateAccountName
        );
        ensure!(
            !CidRegisteredAccount::<T>::contains_key(&cid_number, account_name),
            Error::<T>::CidAlreadyRegistered
        );
        let (account, _kind) =
            Pallet::<T>::derive_registered_account(cid_number.as_slice(), account_name.as_slice())?;
        ensure!(
            !AccountRegisteredCid::<T>::contains_key(&account),
            Error::<T>::AccountAlreadyExists
        );
        ensure!(
            !T::ReservedAccountChecker::is_reserved(&account),
            Error::<T>::AccountReserved
        );
        ensure!(
            T::AccountValidator::is_valid(&account),
            Error::<T>::InvalidAccount
        );
        ensure!(
            !T::ProtectedSourceChecker::is_protected(&account),
            Error::<T>::ProtectedSource
        );
        derived.push((account_name.clone(), account));
    }

    UsedRegisterNonce::<T>::insert(nonce_hash, true);
    let now = <frame_system::Pallet<T>>::block_number();
    for (account_name, account) in derived {
        InstitutionAccounts::<T>::insert(
            &cid_number,
            &account_name,
            InstitutionAccountInfo {
                address: account.clone(),
                initial_balance: BalanceOf::<T>::zero(),
                status: InstitutionLifecycleStatus::Active,
                is_default: false,
                created_at: now,
            },
        );
        CidRegisteredAccount::<T>::insert(&cid_number, &account_name, &account);
        AccountRegisteredCid::<T>::insert(
            &account,
            RegisteredInstitution {
                cid_number: cid_number.clone(),
                account_name: account_name.clone(),
            },
        );
        Pallet::<T>::deposit_event(Event::<T>::InstitutionAccountAdded {
            cid_number: cid_number.clone(),
            account_name,
            account,
            submitter: submitter.clone(),
        });
    }
    Ok(())
}
