//! 投票通过/否决终态回调时的业务执行体(共用部分)。
//!
//! 涵盖:
//! - `execute_create_with_finalizer`: ACTION_CREATE_PERSONAL 通过后入金 + 激活
//!   `DuoqianAccounts`(单账户机构 / 个人多签共用)。
//! - `execute_close_with_finalizer`: ACTION_CLOSE 通过后转出余额 + 删除
//!   `DuoqianAccounts` + `PersonalDuoqianInfo`(对机构 no-op) + 关闭 admin
//!   subject + 清 PendingCloseProposal。
//!
//! 机构整体创建(ACTION_CREATE_INSTITUTION)的执行/清理另在 `institution::execute`。

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

use crate::common::account_to_institution_id;
use crate::pallet::{
    Config, DuoqianAccounts, Error, Event, Pallet, PendingCloseProposal, PendingPersonalCreate,
    PersonalDuoqianInfo,
};
use crate::personal::types::{CloseDuoqianAction, CreateDuoqianAction, DuoqianStatus};
use crate::BalanceOf;

/// 执行创建：unreserve + 划转 + 扣手续费 + 激活 DuoqianAccounts。
///
/// 资金模型(2026-05-03 整改后与机构多签一致):
/// 提案创建时已 reserve(amount + fee),此处先 unreserve 再划转入金 + 扣手续费。
pub(crate) fn execute_create_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &CreateDuoqianAction<T::AccountId, BalanceOf<T>>,
    _callback_context: bool,
) -> DispatchResult {
    let amount_u128: u128 = action.amount.saturated_into();
    let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
    let fee: BalanceOf<T> = fee_u128.saturated_into();
    let reserve_total = action.amount.saturating_add(fee);

    // 先 unreserve 再划转(避开 KeepAlive 与已 reserve 资金的语义冲突)。
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

    DuoqianAccounts::<T>::mutate(&action.duoqian_address, |maybe_account| {
        if let Some(account) = maybe_account {
            account.status = DuoqianStatus::Active;
        }
    });
    // 中文注释:ACTION_CREATE_PERSONAL 仅用于个人多签创建,主体 institution_id
    // 由 personal address 直接派生(个人多签锚在地址自身)。
    Pallet::<T>::activate_admin_subject(
        proposal_id,
        account_to_institution_id(&action.duoqian_address),
    )?;
    // 备份 action 已在 propose 阶段写入 PendingPersonalCreate,激活后清理。
    PendingPersonalCreate::<T>::remove(proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::DuoqianCreated {
        proposal_id,
        duoqian_address: action.duoqian_address.clone(),
        creator: action.proposer.clone(),
        admin_count: action.admin_count,
        threshold: action.threshold,
        amount: action.amount,
        fee,
    });

    Ok(())
}

/// 执行关闭：转出余额 + 删除 DuoqianAccounts + 关闭 admin subject。
pub(crate) fn execute_close_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &CloseDuoqianAction<T::AccountId>,
    _callback_context: bool,
) -> DispatchResult {
    ensure!(
        T::InstitutionAsset::can_spend(
            &action.duoqian_address,
            InstitutionAssetAction::DuoqianCloseExecute,
        ),
        Error::<T>::ProtectedSource
    );
    // 中文注释:必须在删 PersonalDuoqianInfo 之前 resolve subject_id,因为
    // resolve_admin_subject_for_account 依赖 PersonalDuoqianInfo / AddressRegisteredSfid。
    let subject_id = Pallet::<T>::resolve_admin_subject_for_account(&action.duoqian_address)
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

    DuoqianAccounts::<T>::remove(&action.duoqian_address);
    PersonalDuoqianInfo::<T>::remove(&action.duoqian_address);
    Pallet::<T>::close_admin_subject(proposal_id, subject_id)?;
    PendingCloseProposal::<T>::remove(&action.duoqian_address);

    Pallet::<T>::deposit_event(Event::<T>::DuoqianClosed {
        proposal_id,
        duoqian_address: action.duoqian_address.clone(),
        beneficiary: action.beneficiary.clone(),
        amount: transfer_amount,
        fee,
    });

    Ok(())
}
