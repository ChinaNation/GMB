//! 机构账户表相关 helper。
//!
//! 此模块负责:
//! - `account_names_payload_from_initial_accounts`: 把机构创建账户列表的
//!   account_name 抽成 CID 验签 payload `Vec<Vec<u8>>`(顺序与 CID
//!   `/registration-info.account_names` 严格一致)。
//! - `account_names_payload_from_names`: 同样的 payload, 但接收
//!   `register_cid_public_institution` 入口传来的 BoundedVec<AccountName>。
//! - `validate_initial_accounts`: 校验机构初始账户列表合法性,派生每个账户
//!   的链上地址,返回固化的 `CreateInstitutionAccountsOf<T>` + 主账户/
//!   费用账户/初始余额合计,供 `do_propose_create_public_institution` 用。

extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use frame_support::ensure;
use frame_support::traits::Currency;
use sp_runtime::{
    traits::{CheckedAdd, Zero},
    DispatchError,
};

use crate::institution::types::CreateInstitutionAccount;
use crate::pallet::{
    AccountRegisteredCid, CidNumberOf, Config, CreateInstitutionAccountsOf, Error,
    InstitutionAccounts, InstitutionInitialAccountsOf, Pallet,
};
use crate::traits::{AccountValidator, ProtectedSourceChecker, ReservedAccountGuard};
use crate::BalanceOf;

/// 把机构创建账户列表里的 account_name 抽成 CID 签名 payload `Vec<Vec<u8>>`。
/// 顺序不重排,必须和 CID `/registration-info.account_names` 一致。
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

// 批量 register 入口用的 account_names_payload_from_names 实现保留在
// lib.rs 内 `Pallet::<T>::account_names_payload_from_names`(register.rs 直接调),
// 此处不重复。

/// 校验机构初始账户列表合法性,派生地址,返回:
/// - 固化的 `CreateInstitutionAccountsOf<T>`(已派生完地址)
/// - 主账户 AccountId
/// - 费用账户 AccountId
/// - 初始余额合计
pub(crate) fn validate_initial_accounts<T: Config>(
    cid_number: &CidNumberOf<T>,
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
    let required_protocol_kinds = primitives::institution_constraints::required_protocol_account_kinds(
        primitives::cid::code::institution_code_from_cid_number(
            core::str::from_utf8(cid_number.as_slice()).map_err(|_| Error::<T>::InvalidCidNumber)?,
        )
        .ok_or(Error::<T>::InvalidCidNumber)?,
        cid_number.as_slice(),
    )
    .ok_or(Error::<T>::InvalidCidNumber)?;
    let mut protocol_kinds = BTreeSet::new();
    let mut main_account: Option<T::AccountId> = None;
    let mut fee_account: Option<T::AccountId> = None;
    let mut initial_total = BalanceOf::<T>::zero();
    let mut built: Vec<CreateInstitutionAccount<_, T::AccountId, BalanceOf<T>>> =
        Vec::with_capacity(accounts.len());

    for item in accounts.iter() {
        ensure!(!item.account_name.is_empty(), Error::<T>::EmptyAccountName);
        ensure!(
            item.amount.is_zero() || item.amount >= T::Currency::minimum_balance(),
            Error::<T>::AccountInitialAmountBelowMinimum
        );
        ensure!(
            seen.insert(item.account_name.to_vec()),
            Error::<T>::DuplicateAccountName
        );

        let (address, kind) = Pallet::<T>::derive_registered_account(
            cid_number.as_slice(),
            item.account_name.as_slice(),
        )?;
        ensure!(
            !InstitutionAccounts::<T>::contains_key(cid_number, &item.account_name),
            Error::<T>::CidAlreadyRegistered
        );
        ensure!(
            !AccountRegisteredCid::<T>::contains_key(&address),
            Error::<T>::AccountAlreadyExists
        );
        ensure!(
            !T::ReservedAccountChecker::is_reserved(&address),
            Error::<T>::AccountReserved
        );
        ensure!(
            T::AccountValidator::is_valid(&address),
            Error::<T>::InvalidAccount
        );
        ensure!(
            !T::ProtectedSourceChecker::is_protected(&address),
            Error::<T>::ProtectedSource
        );

        if let Some(protocol_kind) = kind.institution_protocol_kind() {
            ensure!(
                required_protocol_kinds.contains(&protocol_kind),
                Error::<T>::ReservedAccountName
            );
            protocol_kinds.insert(protocol_kind);
            if protocol_kind == primitives::account_derive::InstitutionProtocolAccountKind::Main {
                main_account = Some(address.clone());
            }
            if protocol_kind == primitives::account_derive::InstitutionProtocolAccountKind::Fee {
                fee_account = Some(address.clone());
            }
        }

        initial_total = initial_total
            .checked_add(&item.amount)
            .ok_or(Error::<T>::InitialAmountOverflow)?;
        built.push(CreateInstitutionAccount {
            account_name: item.account_name.clone(),
            address,
            amount: item.amount,
        });
    }

    ensure!(
        required_protocol_kinds
            .iter()
            .all(|required| protocol_kinds.contains(required)),
        Error::<T>::MissingMainAccount
    );
    let bounded: CreateInstitutionAccountsOf<T> = built
        .try_into()
        .map_err(|_| Error::<T>::TooManyInstitutionAccounts)?;
    Ok((
        bounded,
        main_account.ok_or(Error::<T>::MissingMainAccount)?,
        fee_account.ok_or(Error::<T>::MissingFeeAccount)?,
        initial_total,
    ))
}
