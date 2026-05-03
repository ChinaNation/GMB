//! 机构整体创建(ACTION_CREATE_INSTITUTION)的投票回调执行体。
//!
//! 涵盖:
//! - `execute_create_institution_with_finalizer`: 投票通过后划转初始余额 +
//!   扣手续费 + 激活机构和所有账户 + 激活 admin subject + 移除 PendingInstitutionCreate
//! - `cleanup_pending_institution_create`: 投票否决/超时/执行失败终态时,
//!   unreserve 创建者资金 + 清空 Institutions / InstitutionAccounts /
//!   SfidRegisteredAddress / AddressRegisteredSfid / DuoqianAccounts +
//!   移除 admin subject Pending。

extern crate alloc;

use frame_support::{
    ensure,
    traits::{Currency, ExistenceRequirement, OnUnbalanced, ReservableCurrency},
};
use sp_runtime::{traits::Zero, DispatchResult};

use crate::common::sfid_id_to_institution_id;
use crate::institution::types::InstitutionLifecycleStatus;
use crate::pallet::{
    AddressRegisteredSfid, Config, CreateInstitutionActionOf, DuoqianAccounts, Error, Event,
    InstitutionAccounts, Institutions, Pallet, PendingInstitutionCreate, SfidRegisteredAddress,
};
use crate::personal::types::DuoqianStatus;

/// 投票否决/超时/执行失败终态时清理机构整体创建相关存储。
pub(crate) fn cleanup_pending_institution_create<T: Config>(
    proposal_id: u64,
    action: &CreateInstitutionActionOf<T>,
    emit_event: bool,
) {
    let _ = T::Currency::unreserve(&action.proposer, action.reserve_total);
    PendingInstitutionCreate::<T>::remove(proposal_id);
    Institutions::<T>::remove(&action.sfid_id);
    for account in action.accounts.iter() {
        InstitutionAccounts::<T>::remove(&action.sfid_id, &account.account_name);
        SfidRegisteredAddress::<T>::remove(&action.sfid_id, &account.account_name);
        AddressRegisteredSfid::<T>::remove(&account.address);
    }
    DuoqianAccounts::<T>::remove(&action.main_address);
    if let Some(institution_id) = sfid_id_to_institution_id(action.sfid_id.as_slice()) {
        Pallet::<T>::remove_pending_admin_subject(proposal_id, institution_id);
    }
    if emit_event {
        Pallet::<T>::deposit_event(Event::<T>::InstitutionCreateRejected {
            proposal_id,
            sfid_id: action.sfid_id.clone(),
            main_address: action.main_address.clone(),
            reserve_total: action.reserve_total,
        });
    }
}

/// 投票通过后执行机构整体创建：unreserve + 扣手续费 + 划转 + 激活。
pub(crate) fn execute_create_institution_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &CreateInstitutionActionOf<T>,
    _callback_context: bool,
) -> DispatchResult {
    ensure!(
        PendingInstitutionCreate::<T>::contains_key(proposal_id),
        Error::<T>::ProposalActionNotFound
    );

    let leftover = T::Currency::unreserve(&action.proposer, action.reserve_total);
    ensure!(leftover.is_zero(), Error::<T>::ReserveReleaseFailed);

    if !action.fee.is_zero() {
        let fee_imbalance = T::Currency::withdraw(
            &action.proposer,
            action.fee,
            frame_support::traits::WithdrawReasons::FEE,
            ExistenceRequirement::KeepAlive,
        )
        .map_err(|_| Error::<T>::FeeWithdrawFailed)?;
        T::FeeRouter::on_unbalanced(fee_imbalance);
    }

    for account in action.accounts.iter() {
        T::Currency::transfer(
            &action.proposer,
            &account.address,
            account.amount,
            ExistenceRequirement::KeepAlive,
        )
        .map_err(|_| Error::<T>::TransferFailed)?;
        InstitutionAccounts::<T>::mutate(
            &action.sfid_id,
            &account.account_name,
            |maybe_account| {
                if let Some(stored) = maybe_account {
                    stored.status = InstitutionLifecycleStatus::Active;
                }
            },
        );
    }

    Institutions::<T>::try_mutate(&action.sfid_id, |maybe_institution| -> DispatchResult {
        let institution = maybe_institution
            .as_mut()
            .ok_or(Error::<T>::InstitutionNotRegistered)?;
        institution.status = InstitutionLifecycleStatus::Active;
        Ok(())
    })?;
    DuoqianAccounts::<T>::mutate(&action.main_address, |maybe_account| {
        if let Some(account) = maybe_account {
            account.status = DuoqianStatus::Active;
        }
    });
    let institution_id = sfid_id_to_institution_id(action.sfid_id.as_slice())
        .ok_or(Error::<T>::EmptySfidId)?;
    Pallet::<T>::activate_admin_subject(proposal_id, institution_id)?;
    PendingInstitutionCreate::<T>::remove(proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::InstitutionCreated {
        proposal_id,
        sfid_id: action.sfid_id.clone(),
        main_address: action.main_address.clone(),
        account_count: action.accounts.len() as u32,
        initial_total: action.initial_total,
        fee: action.fee,
    });

    Ok(())
}
