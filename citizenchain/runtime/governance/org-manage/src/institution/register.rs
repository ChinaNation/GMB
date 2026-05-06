//! SFID 机构链上登记流程实现。
//!
//! `do_register_sfid_institution` 由 lib.rs 内 call_index=2 入口 delegate 调用。
//! 业务流程：
//! 1. 校验参数非空（sfid_id / institution_name / account_names / province）
//! 2. 校验 register_nonce 未被复用
//! 3. 调 `SfidInstitutionVerifier` 双层验签（ADR-008 step2b: province + signer_admin_pubkey）
//! 4. 遍历 account_names 派生机构账户地址 + 校验保留名/重复/已注册
//! 5. 写入 `UsedRegisterNonce` / `SfidRegisteredAddress` / `AddressRegisteredSfid`
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
    self, AccountNameOf, AddressRegisteredSfid, Error, Event, InstitutionAccountNamesOf, Pallet,
    RegisterNonceOf, RegisterSignatureOf, SfidIdOf, SfidRegisteredAddress, UsedRegisterNonce,
};
use crate::traits::{
    DuoqianAddressValidator, DuoqianReservedAddressChecker, ProtectedSourceChecker,
    SfidInstitutionVerifier,
};
use crate::RegisteredInstitution;

/// 处理 SFID 机构登记业务逻辑。
pub(crate) fn do_register_sfid_institution<T: pallet::Config>(
    submitter: T::AccountId,
    sfid_id: SfidIdOf<T>,
    institution_name: AccountNameOf<T>,
    account_names: InstitutionAccountNamesOf<T>,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    province: Vec<u8>,
    signer_admin_pubkey: [u8; 32],
) -> DispatchResult {
    ensure!(!sfid_id.is_empty(), Error::<T>::EmptySfidId);
    ensure!(!institution_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!account_names.is_empty(), Error::<T>::MissingMainAccount);
    ensure!(!province.is_empty(), Error::<T>::EmptyProvince);

    let register_nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(register_nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );

    let account_name_payload = Pallet::<T>::account_names_payload_from_names(&account_names)?;
    ensure!(
        T::SfidInstitutionVerifier::verify_institution_registration(
            sfid_id.as_slice(),
            &institution_name,
            &account_name_payload,
            &register_nonce,
            &signature,
            province.as_slice(),
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
            !SfidRegisteredAddress::<T>::contains_key(&sfid_id, account_name),
            Error::<T>::SfidAlreadyRegistered
        );
        let role = Pallet::<T>::role_from_account_name(account_name.as_slice())?;
        let duoqian_address = Pallet::<T>::derive_institution_address(sfid_id.as_slice(), role)?;
        ensure!(
            !AddressRegisteredSfid::<T>::contains_key(&duoqian_address),
            Error::<T>::AddressAlreadyExists
        );
        ensure!(
            !T::ReservedAddressChecker::is_reserved(&duoqian_address),
            Error::<T>::AddressReserved
        );
        ensure!(
            T::AddressValidator::is_valid(&duoqian_address),
            Error::<T>::InvalidAddress
        );
        ensure!(
            !T::ProtectedSourceChecker::is_protected(&duoqian_address),
            Error::<T>::ProtectedSource
        );
        derived.push((account_name.clone(), duoqian_address));
    }

    UsedRegisterNonce::<T>::insert(register_nonce_hash, true);
    for (account_name, duoqian_address) in derived {
        SfidRegisteredAddress::<T>::insert(&sfid_id, &account_name, &duoqian_address);
        AddressRegisteredSfid::<T>::insert(
            &duoqian_address,
            RegisteredInstitution {
                sfid_id: sfid_id.clone(),
                account_name: account_name.clone(),
            },
        );
        Pallet::<T>::deposit_event(Event::<T>::SfidInstitutionRegistered {
            sfid_id: sfid_id.clone(),
            account_name,
            duoqian_address,
            submitter: submitter.clone(),
        });
    }
    Ok(())
}
