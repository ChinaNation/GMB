//! 机构账户表相关 helper。
//!
//! 此模块负责:
//! - `account_names_payload_from_initial_accounts`: 把机构创建账户列表的
//!   account_name 抽成 SFID 验签 payload `Vec<Vec<u8>>`(顺序与 SFID
//!   `/registration-info.account_names` 严格一致)。
//! - `account_names_payload_from_names`: 同样的 payload, 但接收
//!   `register_sfid_institution` 入口传来的 BoundedVec<AccountName>。
//! - `validate_initial_accounts`: 校验机构初始账户列表合法性,派生每个账户
//!   的链上地址,返回固化的 `CreateInstitutionAccountsOf<T>` + 主账户/
//!   费用账户/初始余额合计,供 `do_propose_create_institution` 用。

extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use frame_support::ensure;
use frame_support::traits::Get;
use sp_runtime::{
    traits::{CheckedAdd, Zero},
    DispatchError,
};

use crate::address::InstitutionAccountRole;
use crate::institution::types::CreateInstitutionAccount;
use crate::pallet::{
    AddressRegisteredSfid, Config, CreateInstitutionAccountsOf, DuoqianAccounts, Error,
    InstitutionInitialAccountsOf, Pallet, SfidIdOf, SfidRegisteredAddress,
};
use crate::traits::{
    DuoqianAddressValidator, DuoqianReservedAddressChecker, ProtectedSourceChecker,
};
use crate::BalanceOf;

/// 把机构创建账户列表里的 account_name 抽成 SFID 签名 payload `Vec<Vec<u8>>`。
/// 顺序不重排,必须和 SFID `/registration-info.account_names` 一致。
pub(crate) fn account_names_payload_from_initial_accounts<T: Config>(
    accounts: &InstitutionInitialAccountsOf<T>,
) -> Result<Vec<Vec<u8>>, DispatchError> {
    let mut names: Vec<Vec<u8>> = Vec::with_capacity(accounts.len());
    for item in accounts.iter() {
        ensure!(!item.account_name.is_empty(), Error::<T>::EmptyAccountName);
        names.push(item.account_name.as_slice().to_vec());
    }
    Ok(names)
}

// 中文注释:批量 register 入口用的 account_names_payload_from_names 实现保留在
// lib.rs 内 `Pallet::<T>::account_names_payload_from_names`(register.rs 直接调),
// 此处不重复。

/// 校验机构初始账户列表合法性,派生地址,返回:
/// - 固化的 `CreateInstitutionAccountsOf<T>`(已派生完地址)
/// - 主账户地址
/// - 费用账户地址
/// - 初始余额合计
pub(crate) fn validate_initial_accounts<T: Config>(
    sfid_id: &SfidIdOf<T>,
    accounts: &InstitutionInitialAccountsOf<T>,
) -> Result<
    (
        CreateInstitutionAccountsOf<T>,
        T::AccountId,
        T::AccountId,
        BalanceOf<T>,
    ),
    DispatchError,
> {
    ensure!(!accounts.is_empty(), Error::<T>::EmptyInstitutionAccounts);

    let mut seen = BTreeSet::new();
    let mut has_main = false;
    let mut has_fee = false;
    let mut main_address: Option<T::AccountId> = None;
    let mut fee_address: Option<T::AccountId> = None;
    let mut initial_total = BalanceOf::<T>::zero();
    let mut built: Vec<CreateInstitutionAccount<_, T::AccountId, BalanceOf<T>>> =
        Vec::with_capacity(accounts.len());

    for item in accounts.iter() {
        ensure!(!item.account_name.is_empty(), Error::<T>::EmptyAccountName);
        ensure!(
            item.amount >= <T as Config>::MinCreateAmount::get(),
            Error::<T>::AccountInitialAmountBelowMinimum
        );
        ensure!(
            seen.insert(item.account_name.to_vec()),
            Error::<T>::DuplicateAccountName
        );

        let role = Pallet::<T>::role_from_account_name(item.account_name.as_slice())?;
        let is_default = matches!(
            role,
            InstitutionAccountRole::Main | InstitutionAccountRole::Fee
        );
        let address = Pallet::<T>::derive_institution_address(sfid_id.as_slice(), role)?;

        ensure!(
            !SfidRegisteredAddress::<T>::contains_key(sfid_id, &item.account_name),
            Error::<T>::SfidAlreadyRegistered
        );
        ensure!(
            !AddressRegisteredSfid::<T>::contains_key(&address),
            Error::<T>::AddressAlreadyExists
        );
        ensure!(
            !DuoqianAccounts::<T>::contains_key(&address),
            Error::<T>::AddressAlreadyExists
        );
        ensure!(
            !T::ReservedAddressChecker::is_reserved(&address),
            Error::<T>::AddressReserved
        );
        ensure!(
            T::AddressValidator::is_valid(&address),
            Error::<T>::InvalidAddress
        );
        ensure!(
            !T::ProtectedSourceChecker::is_protected(&address),
            Error::<T>::ProtectedSource
        );

        match role {
            InstitutionAccountRole::Main => {
                has_main = true;
                main_address = Some(address.clone());
            }
            InstitutionAccountRole::Fee => {
                has_fee = true;
                fee_address = Some(address.clone());
            }
            InstitutionAccountRole::Named(_) => {}
        }

        initial_total = initial_total
            .checked_add(&item.amount)
            .ok_or(Error::<T>::InitialAmountOverflow)?;
        built.push(CreateInstitutionAccount {
            account_name: item.account_name.clone(),
            address,
            amount: item.amount,
            is_default,
        });
    }

    ensure!(has_main, Error::<T>::MissingMainAccount);
    ensure!(has_fee, Error::<T>::MissingFeeAccount);
    let bounded: CreateInstitutionAccountsOf<T> = built
        .try_into()
        .map_err(|_| Error::<T>::TooManyInstitutionAccounts)?;
    Ok((
        bounded,
        main_address.ok_or(Error::<T>::MissingMainAccount)?,
        fee_address.ok_or(Error::<T>::MissingFeeAccount)?,
        initial_total,
    ))
}
