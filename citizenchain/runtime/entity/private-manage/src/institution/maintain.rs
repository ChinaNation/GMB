//! 机构信息维护:改名(`do_update_institution_info`)+ 新增账户
//! (`do_add_institution_account`)。
//!
//! 链是机构信息唯一真源(ADR-031);创世只铸定初始集,今后改名/加账户走交易。
//! 授权唯一真源 = 注册局机构 CID + 岗位码 + 任职管理员钱包；管理员身份本身不授权。
//! 省/市作用域由目标 CID 直接派生,不再嵌独立凭证/签名/nonce。
//! 机构码/CID/省市码物理编码在 CID 里,改不了也不给参数。防重放由 extrinsic 账户 nonce 承担。

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use frame_support::ensure;
use sp_runtime::{traits::Zero, DispatchResult};

use crate::institution::types::InstitutionAccountInfo;
use crate::pallet::{
    self, AccountNameOf, AccountRegisteredCid, CidNumberOf, Error, Event,
    InstitutionAccountNamesOf, InstitutionAccounts, Institutions, Pallet,
};
use crate::traits::{
    AccountValidator, ProtectedSourceChecker, RegistryAuthority, ReservedAccountGuard,
};
use crate::{BalanceOf, RegisteredInstitution};

/// 注册局改机构全称/简称:链是唯一真源;机构码/CID/省市码不可改故不给参数。
pub(crate) fn do_update_institution_info<T: pallet::Config>(
    submitter: T::AccountId,
    cid_number: CidNumberOf<T>,
    cid_full_name: AccountNameOf<T>,
    cid_short_name: AccountNameOf<T>,
    actor_cid_number: Vec<u8>,
    actor_role_code: Vec<u8>,
) -> DispatchResult {
    ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
    ensure!(!cid_full_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!cid_short_name.is_empty(), Error::<T>::EmptyAccountName);

    let info = Institutions::<T>::get(&cid_number).ok_or(Error::<T>::InstitutionNotFound)?;
    // 授权唯一真源:extrinsic 签名者是注册局在册管理员,且对目标机构有登记权。
    ensure!(
        T::RegistryAuthority::can_register_institution_origin(
            &submitter,
            actor_cid_number.as_slice(),
            actor_role_code.as_slice(),
            cid_number.as_slice(),
            info.institution_code,
        ),
        Error::<T>::RegistryAuthorityDenied
    );

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
pub(crate) fn do_add_institution_account<T: pallet::Config>(
    submitter: T::AccountId,
    cid_number: CidNumberOf<T>,
    account_names: InstitutionAccountNamesOf<T>,
    actor_cid_number: Vec<u8>,
    actor_role_code: Vec<u8>,
) -> DispatchResult {
    ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
    ensure!(!account_names.is_empty(), Error::<T>::MissingMainAccount);

    let info = Institutions::<T>::get(&cid_number).ok_or(Error::<T>::InstitutionNotFound)?;
    // 授权唯一真源:extrinsic 签名者是注册局在册管理员,且对目标机构有登记权。
    ensure!(
        T::RegistryAuthority::can_register_institution_origin(
            &submitter,
            actor_cid_number.as_slice(),
            actor_role_code.as_slice(),
            cid_number.as_slice(),
            info.institution_code,
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
            primitives::account_derive::is_registrable_custom_name(account_name.as_slice()),
            Error::<T>::ReservedAccountName
        );
        ensure!(
            seen.insert(account_name.as_slice().to_vec()),
            Error::<T>::DuplicateAccountName
        );
        ensure!(
            !InstitutionAccounts::<T>::contains_key(&cid_number, account_name),
            Error::<T>::CidAlreadyRegistered
        );
        let (account, _kind) = Pallet::<T>::derive_institution_account(
            cid_number.as_slice(),
            account_name.as_slice(),
        )?;
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

    let now = <frame_system::Pallet<T>>::block_number();
    for (account_name, account) in derived {
        InstitutionAccounts::<T>::insert(
            &cid_number,
            &account_name,
            InstitutionAccountInfo {
                address: account.clone(),
                initial_balance: BalanceOf::<T>::zero(),
                created_at: now,
            },
        );
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
