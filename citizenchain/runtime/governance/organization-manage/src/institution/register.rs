//! SFID 机构链上登记流程实现。
//!
//! `do_register_sfid_institution` 由 lib.rs 内 call_index=2 入口 delegate 调用。
//! 业务流程：
//! 1. 校验参数非空（sfid_number / sfid_full_name / account_names / province_name）
//! 2. 校验 register_nonce 未被复用
//! 3. 调 `SfidInstitutionVerifier` 双层验签（ADR-008 step2b: province_name + signer_admin_pubkey）
//! 4. 遍历 account_names 派生机构账户地址 + 校验保留名/重复/已注册
//! 5. 写入 `UsedRegisterNonce` / `SfidRegisteredAccount` / `AccountRegisteredSfid`
//! 6. 发射 `SfidInstitutionRegistered` 事件
//!
//! 不写入 `Institutions` / `InstitutionAccounts` —— 那是 `propose_create_institution`
//! 的职责（投票通过后 reserve→划转→激活）。

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use frame_support::ensure;
use sp_runtime::{traits::Hash, DispatchResult};

use crate::pallet::{
    self, AccountNameOf, AccountRegisteredSfid, Error, Event, InstitutionAccountNamesOf, Pallet,
    RegisterNonceOf, RegisterSignatureOf, SfidNumberOf, SfidRegisteredAccount, UsedRegisterNonce,
};
use crate::traits::{
    DuoqianAccountValidator, DuoqianReservedAccountChecker, ProtectedSourceChecker,
    SfidInstitutionVerifier,
};
use crate::RegisteredInstitution;

/// 处理 SFID 机构登记业务逻辑。
pub(crate) fn do_register_sfid_institution<T: pallet::Config>(
    submitter: T::AccountId,
    sfid_number: SfidNumberOf<T>,
    sfid_full_name: AccountNameOf<T>,
    account_names: InstitutionAccountNamesOf<T>,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    province_name: Vec<u8>,
    signer_admin_pubkey: [u8; 32],
) -> DispatchResult {
    ensure!(!sfid_number.is_empty(), Error::<T>::EmptySfidNumber);
    ensure!(!sfid_full_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!account_names.is_empty(), Error::<T>::MissingMainAccount);
    ensure!(!province_name.is_empty(), Error::<T>::EmptyProvince);

    let register_nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(register_nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );

    let account_name_payload = Pallet::<T>::account_names_payload_from_names(&account_names)?;
    ensure!(
        T::SfidInstitutionVerifier::verify_institution_registration(
            sfid_number.as_slice(),
            &sfid_full_name,
            &account_name_payload,
            &register_nonce,
            &signature,
            province_name.as_slice(),
            &signer_admin_pubkey,
        ),
        Error::<T>::InvalidSfidInstitutionSignature
    );

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
            !SfidRegisteredAccount::<T>::contains_key(&sfid_number, account_name),
            Error::<T>::SfidAlreadyRegistered
        );
        let role = Pallet::<T>::role_from_account_name(account_name.as_slice())?;
        let duoqian_account =
            Pallet::<T>::derive_institution_account(sfid_number.as_slice(), role)?;
        ensure!(
            !AccountRegisteredSfid::<T>::contains_key(&duoqian_account),
            Error::<T>::AccountAlreadyExists
        );
        ensure!(
            !T::ReservedAccountChecker::is_reserved(&duoqian_account),
            Error::<T>::AccountReserved
        );
        ensure!(
            T::AccountValidator::is_valid(&duoqian_account),
            Error::<T>::InvalidAccount
        );
        ensure!(
            !T::ProtectedSourceChecker::is_protected(&duoqian_account),
            Error::<T>::ProtectedSource
        );
        derived.push((account_name.clone(), duoqian_account));
    }

    UsedRegisterNonce::<T>::insert(register_nonce_hash, true);
    for (account_name, duoqian_account) in derived {
        SfidRegisteredAccount::<T>::insert(&sfid_number, &account_name, &duoqian_account);
        AccountRegisteredSfid::<T>::insert(
            &duoqian_account,
            RegisteredInstitution {
                sfid_number: sfid_number.clone(),
                account_name: account_name.clone(),
            },
        );
        Pallet::<T>::deposit_event(Event::<T>::SfidInstitutionRegistered {
            sfid_number: sfid_number.clone(),
            account_name,
            duoqian_account,
            submitter: submitter.clone(),
        });
    }
    Ok(())
}
