//! CID 机构链上登记流程实现。
//!
//! `do_register_cid_private_institution` 由 lib.rs 内 call_index=2 入口 delegate 调用。
//! 业务流程：
//! 1. 校验参数非空（cid_number / cid_full_name / account_names / 签发机构 / 作用域）
//! 2. 校验 register_nonce 未被复用
//! 3. 调 `CidInstitutionVerifier` 校验签发机构 admins 与 sr25519 签名
//! 4. 遍历 account_names 派生机构账户地址 + 校验保留名/重复/已注册
//! 5. 写入 `UsedRegisterNonce` / `CidRegisteredAccount` / `AccountRegisteredCid`
//! 6. 发射 `CidInstitutionRegistered` 事件
//!
//! 不写入 `Institutions` / `InstitutionAccounts` —— 那是 `propose_create_private_institution`
//! 的职责（注册交易成功即划转初始余额并激活）。

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use frame_support::ensure;
use sp_runtime::{traits::Hash, DispatchResult};

use crate::pallet::{
    self, AccountNameOf, AccountRegisteredCid, CidNumberOf, CidRegisteredAccount, Error, Event,
    InstitutionAccountNamesOf, Pallet, RegisterNonceOf, RegisterSignatureOf, UsedRegisterNonce,
};
use crate::traits::{
    AccountValidator, CidInstitutionVerifier, InstitutionCidQuery, ProtectedSourceChecker,
    ReservedAccountGuard,
};
use crate::RegisteredInstitution;

/// 处理 CID 机构登记业务逻辑。
pub(crate) fn do_register_cid_private_institution<T: pallet::Config>(
    submitter: T::AccountId,
    cid_number: CidNumberOf<T>,
    cid_full_name: AccountNameOf<T>,
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
    // CID 号全量校验单源 primitives::cid,且机构码必须是私权法人/非法人家族。
    let parts = primitives::cid::number::parse_cid_number_parts_bytes(cid_number.as_slice())
        .map_err(|_| Error::<T>::InvalidCidNumber)?;
    ensure!(
        primitives::cid::code::is_private_legal_code(&parts.institution)
            || primitives::cid::code::is_unincorporated_code(&parts.institution),
        Error::<T>::InvalidCidNumber
    );
    // 机构级墓碑拦截:整机构已关闭的 CID 永不复用,拒绝重建账户索引。
    if let Some(info) = crate::pallet::Institutions::<T>::get(&cid_number) {
        ensure!(
            info.status != crate::institution::types::InstitutionLifecycleStatus::Closed,
            Error::<T>::InstitutionAlreadyClosed
        );
    }
    ensure!(
        !T::SiblingInstitutionQuery::cid_exists(&cid_number),
        Error::<T>::CidAlreadyRegistered
    );
    ensure!(!cid_full_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!account_names.is_empty(), Error::<T>::MissingMainAccount);
    ensure!(
        !issuer_cid_number.is_empty(),
        Error::<T>::EmptyIssuerCidNumber
    );
    ensure!(
        !scope_province_name.is_empty(),
        Error::<T>::EmptyScopeProvinceName
    );

    let register_nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(register_nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );

    let account_name_payload = Pallet::<T>::account_names_payload_from_names(&account_names)?;
    ensure!(
        T::CidInstitutionVerifier::verify_institution_registration(
            cid_number.as_slice(),
            &cid_full_name,
            &[],
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

    UsedRegisterNonce::<T>::insert(register_nonce_hash, true);
    for (account_name, account) in derived {
        CidRegisteredAccount::<T>::insert(&cid_number, &account_name, &account);
        AccountRegisteredCid::<T>::insert(
            &account,
            RegisteredInstitution {
                cid_number: cid_number.clone(),
                account_name: account_name.clone(),
            },
        );
        Pallet::<T>::deposit_event(Event::<T>::CidInstitutionRegistered {
            cid_number: cid_number.clone(),
            account_name,
            account,
            submitter: submitter.clone(),
        });
    }
    Ok(())
}
