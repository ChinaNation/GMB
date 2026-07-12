//! 投票通过/否决终态回调时的业务执行体。
//!
//! 涵盖:
//! - `execute_create_with_finalizer`: ACTION_CREATE 通过后入金 + 激活 PersonalAccounts
//! - `execute_close_with_finalizer`: ACTION_CLOSE 通过后转出余额 + 删除 PersonalAccounts
//!   + 关闭 admin account + 清 PendingCloseProposal
//! - `cleanup_pending_create`: 创建提案被否决/超时/终态失败时清理 reserve

extern crate alloc;

use frame_support::{
    ensure,
    traits::{Currency, ExistenceRequirement, OnUnbalanced, ReservableCurrency},
};
use primitives::institution_asset::{InstitutionAsset, InstitutionAssetAction};
use sp_runtime::{
    traits::{CheckedSub, Saturating, Zero},
    DispatchResult, SaturatedConversion,
};

use crate::pallet::{
    Config, Error, Event, Pallet, PendingCloseProposal, PendingPersonalCreate, PersonalAccounts,
};
use crate::types::{PersonalCloseAction, PersonalCreateAction, PersonalStatus};
use crate::BalanceOf;
use votingengine::InternalVoteEngine;

/// 执行创建：unreserve + 划转 + 扣手续费 + 激活 PersonalAccounts。
///
/// 资金模型:提案创建时已 reserve(amount + fee),此处先 unreserve 再划转入金 + 扣手续费。
pub(crate) fn execute_create_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &PersonalCreateAction<T::AccountId, BalanceOf<T>>,
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
        &action.account,
        action.amount,
        ExistenceRequirement::KeepAlive,
    )
    .map_err(|_| Error::<T>::TransferFailed)?;

    let account = action.account.clone();
    Pallet::<T>::activate_admin_account(proposal_id, account.clone())?;
    PersonalAccounts::<T>::mutate(&action.account, |maybe_account| {
        if let Some(account) = maybe_account {
            account.status = PersonalStatus::Active;
        }
    });
    let institution_code = votingengine::types::PMUL;
    let admins_len = Pallet::<T>::active_account_admins_len(institution_code, account.clone())
        .ok_or(Error::<T>::PersonalNotFound)?;
    let threshold = <T as Config>::InternalVoteEngine::configured_dynamic_threshold(
        institution_code,
        account.clone(),
    )
    .ok_or(Error::<T>::PersonalNotFound)?;
    PendingPersonalCreate::<T>::remove(proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::PersonalCreated {
        proposal_id,
        account: action.account.clone(),
        creator: action.proposer.clone(),
        admins_len,
        threshold,
        amount: action.amount,
        fee,
    });

    Ok(())
}

/// 执行关闭：转出余额 + 删除 PersonalAccounts + 关闭 admin account。
pub(crate) fn execute_close_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &PersonalCloseAction<T::AccountId>,
) -> DispatchResult {
    ensure!(
        T::InstitutionAsset::can_spend(
            &action.account,
            InstitutionAssetAction::MultisigCloseExecute,
        ),
        Error::<T>::ProtectedSource
    );
    let account = action.account.clone();
    let institution_code = votingengine::types::PMUL;
    let admins_len = Pallet::<T>::active_account_admins_len(institution_code, account.clone())
        .ok_or(Error::<T>::PersonalNotFound)?;
    let threshold = <T as Config>::InternalVoteEngine::active_dynamic_threshold(
        institution_code,
        account.clone(),
    )
    .ok_or(Error::<T>::PersonalNotFound)?;
    let all_balance = T::Currency::free_balance(&action.account);
    // 注销执行前再次确认没有 reserved 余额，避免提案后新增锁定资金导致销户不彻底。
    ensure!(
        T::Currency::reserved_balance(&action.account).is_zero(),
        Error::<T>::ReservedBalanceRemaining
    );

    let balance_u128: u128 = all_balance.saturated_into();
    let fee_u128 = onchain::calculate_onchain_fee(balance_u128);
    let fee: BalanceOf<T> = fee_u128.saturated_into();
    let transfer_amount = all_balance
        .checked_sub(&fee)
        .ok_or(Error::<T>::FeeWithdrawFailed)?;

    let ed = T::Currency::minimum_balance();
    ensure!(transfer_amount >= ed, Error::<T>::CloseTransferBelowED);

    if !fee.is_zero() {
        let fee_imbalance = T::Currency::withdraw(
            &action.account,
            fee,
            frame_support::traits::WithdrawReasons::FEE,
            ExistenceRequirement::AllowDeath,
        )
        .map_err(|_| Error::<T>::FeeWithdrawFailed)?;
        T::FeeRouter::on_unbalanced(fee_imbalance);
    }

    T::Currency::transfer(
        &action.account,
        &action.beneficiary,
        transfer_amount,
        ExistenceRequirement::AllowDeath,
    )
    .map_err(|_| Error::<T>::TransferFailed)?;

    PersonalAccounts::<T>::remove(&action.account);
    Pallet::<T>::close_admin_account(proposal_id, account)?;
    PendingCloseProposal::<T>::remove(&action.account);

    Pallet::<T>::deposit_event(Event::<T>::PersonalClosed {
        proposal_id,
        account: action.account.clone(),
        beneficiary: action.beneficiary.clone(),
        admins_len,
        threshold,
        amount: transfer_amount,
        fee,
    });

    Ok(())
}

/// 创建提案被否决/超时/终态失败时清理:
/// unreserve(amount + fee) + 删 PersonalAccounts/PendingPersonalCreate +
/// 移除 admin account Pending。
///
/// `emit_event = true` 时(否决路径)发 `PersonalCreateRejected`,终态失败路径不发。
pub(crate) fn cleanup_pending_create<T: Config>(
    proposal_id: u64,
    action: &PersonalCreateAction<T::AccountId, BalanceOf<T>>,
    emit_event: bool,
) -> Result<bool, sp_runtime::DispatchError> {
    if !PendingPersonalCreate::<T>::contains_key(proposal_id) {
        return Ok(false);
    }

    Pallet::<T>::remove_pending_admin_account(proposal_id, action.account.clone())?;

    let reserve_total = action.amount.saturating_add(action.fee);
    let _ = T::Currency::unreserve(&action.proposer, reserve_total);

    PersonalAccounts::<T>::remove(&action.account);
    PendingPersonalCreate::<T>::remove(proposal_id);

    if emit_event {
        Pallet::<T>::deposit_event(Event::<T>::PersonalCreateRejected {
            proposal_id,
            account: action.account.clone(),
        });
    }
    Ok(true)
}
