//! 投票通过/否决终态回调时的业务执行体。
//!
//! 涵盖:
//! - `execute_create_with_finalizer`: ACTION_CREATE 通过后入金 + 激活 PersonalDuoqians
//! - `execute_close_with_finalizer`: ACTION_CLOSE 通过后转出余额 + 删除 PersonalDuoqians
//!   + 关闭 admin subject + 清 PendingCloseProposal
//! - `cleanup_pending_create`: 创建提案被否决/超时/终态失败时清理 reserve

extern crate alloc;

use frame_support::{
    ensure,
    traits::{Currency, ExistenceRequirement, OnUnbalanced, ReservableCurrency},
};
use institution_asset::{InstitutionAsset, InstitutionAssetAction};
use sp_runtime::{
    traits::{CheckedSub, Saturating, Zero},
    DispatchResult, SaturatedConversion,
};

use crate::pallet::{
    Config, Error, Event, Pallet, PendingCloseProposal, PendingPersonalCreate, PersonalDuoqians,
};
use crate::types::{CloseDuoqianAction, CreateDuoqianAction, DuoqianStatus};
use crate::BalanceOf;
use primitives::derive::subject_id_from_account;

/// 执行创建：unreserve + 划转 + 扣手续费 + 激活 PersonalDuoqians。
///
/// 资金模型:提案创建时已 reserve(amount + fee),此处先 unreserve 再划转入金 + 扣手续费。
pub(crate) fn execute_create_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &CreateDuoqianAction<T::AccountId, BalanceOf<T>>,
) -> DispatchResult {
    let fee = action.fee;
    let reserve_total = action.amount.saturating_add(fee);

    let leftover = T::Currency::unreserve(&action.proposer, reserve_total);
    ensure!(leftover.is_zero(), Error::<T>::ReserveReleaseFailed);

    if !fee.is_zero() {
        let fee_imbalance = T::Currency::withdraw(
            &action.proposer,
            fee,
            frame_support::traits::WithdrawReasons::FEE,
            ExistenceRequirement::KeepAlive,
        )
        .map_err(|_| Error::<T>::FeeWithdrawFailed)?;
        T::FeeRouter::on_unbalanced(fee_imbalance);
    }

    T::Currency::transfer(
        &action.proposer,
        &action.duoqian_address,
        action.amount,
        ExistenceRequirement::KeepAlive,
    )
    .map_err(|_| Error::<T>::TransferFailed)?;

    let subject = subject_id_from_account(&action.duoqian_address);
    Pallet::<T>::activate_admin_subject(proposal_id, subject)?;
    PersonalDuoqians::<T>::mutate(&action.duoqian_address, |maybe_account| {
        if let Some(account) = maybe_account {
            account.status = DuoqianStatus::Active;
        }
    });
    let org = votingengine::types::ORG_REN;
    let admin_count = admins_change::Pallet::<T>::active_subject_admin_count(org, subject)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    let threshold = admins_change::Pallet::<T>::active_subject_threshold(org, subject)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    PendingPersonalCreate::<T>::remove(proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::DuoqianCreated {
        proposal_id,
        duoqian_address: action.duoqian_address.clone(),
        creator: action.proposer.clone(),
        admin_count,
        threshold,
        amount: action.amount,
        fee,
    });

    Ok(())
}

/// 执行关闭：转出余额 + 删除 PersonalDuoqians + 关闭 admin subject。
pub(crate) fn execute_close_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &CloseDuoqianAction<T::AccountId>,
) -> DispatchResult {
    ensure!(
        T::InstitutionAsset::can_spend(
            &action.duoqian_address,
            InstitutionAssetAction::DuoqianCloseExecute,
        ),
        Error::<T>::ProtectedSource
    );
    let subject_id = subject_id_from_account(&action.duoqian_address);
    let org = votingengine::types::ORG_REN;
    let admin_count = admins_change::Pallet::<T>::active_subject_admin_count(org, subject_id)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    let threshold = admins_change::Pallet::<T>::active_subject_threshold(org, subject_id)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    let all_balance = T::Currency::free_balance(&action.duoqian_address);

    let balance_u128: u128 = all_balance.saturated_into();
    let fee_u128 = onchain_transaction::calculate_onchain_fee(balance_u128);
    let fee: BalanceOf<T> = fee_u128.saturated_into();
    let transfer_amount = all_balance
        .checked_sub(&fee)
        .ok_or(Error::<T>::FeeWithdrawFailed)?;

    let ed = T::Currency::minimum_balance();
    ensure!(transfer_amount >= ed, Error::<T>::CloseTransferBelowED);

    if !fee.is_zero() {
        let fee_imbalance = T::Currency::withdraw(
            &action.duoqian_address,
            fee,
            frame_support::traits::WithdrawReasons::FEE,
            ExistenceRequirement::AllowDeath,
        )
        .map_err(|_| Error::<T>::FeeWithdrawFailed)?;
        T::FeeRouter::on_unbalanced(fee_imbalance);
    }

    T::Currency::transfer(
        &action.duoqian_address,
        &action.beneficiary,
        transfer_amount,
        ExistenceRequirement::AllowDeath,
    )
    .map_err(|_| Error::<T>::TransferFailed)?;

    PersonalDuoqians::<T>::remove(&action.duoqian_address);
    Pallet::<T>::close_admin_subject(proposal_id, subject_id)?;
    PendingCloseProposal::<T>::remove(&action.duoqian_address);

    Pallet::<T>::deposit_event(Event::<T>::DuoqianClosed {
        proposal_id,
        duoqian_address: action.duoqian_address.clone(),
        beneficiary: action.beneficiary.clone(),
        admin_count,
        threshold,
        amount: transfer_amount,
        fee,
    });

    Ok(())
}

/// 创建提案被否决/超时/终态失败时清理:
/// unreserve(amount + fee) + 删 PersonalDuoqians/PendingPersonalCreate +
/// 移除 admin subject Pending。
///
/// `emit_event = true` 时(否决路径)发 `DuoqianCreateRejected`,终态失败路径不发。
pub(crate) fn cleanup_pending_create<T: Config>(
    proposal_id: u64,
    action: &CreateDuoqianAction<T::AccountId, BalanceOf<T>>,
    emit_event: bool,
) -> Result<bool, sp_runtime::DispatchError> {
    if !PendingPersonalCreate::<T>::contains_key(proposal_id) {
        return Ok(false);
    }

    Pallet::<T>::remove_pending_admin_subject(
        proposal_id,
        subject_id_from_account(&action.duoqian_address),
    )?;

    let reserve_total = action.amount.saturating_add(action.fee);
    let _ = T::Currency::unreserve(&action.proposer, reserve_total);

    PersonalDuoqians::<T>::remove(&action.duoqian_address);
    PendingPersonalCreate::<T>::remove(proposal_id);

    if emit_event {
        Pallet::<T>::deposit_event(Event::<T>::DuoqianCreateRejected {
            proposal_id,
            duoqian_address: action.duoqian_address.clone(),
        });
    }
    Ok(true)
}
