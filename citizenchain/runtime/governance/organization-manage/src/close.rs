//! 机构多签关闭流程实现(call_index=1)。
//!
//! 仅服务于已注册的 SFID 机构账户(`AddressRegisteredSfid.contains_key` 命中);
//! 个人多签关闭走 personal-manage::propose_close 入口。
//!
//! 业务流程:
//! 1. 校验地址是机构地址(否则返回 `NotInstitutionDuoqian`)
//! 2. 校验机构账户已 Active(从 InstitutionAccounts 读)
//! 3. 校验发起人是该机构的活跃管理员(admins-change::Subjects[sfid_id])
//! 4. 校验余额≥关闭门槛 + 转出金额≥ED + 无 reserved 余额
//! 5. 全员投票阈值 = active_subject_admin_count
//! 6. 写入 InstitutionPendingClose[address] = proposal_id 防并发
//! 7. 发射 InstitutionCloseProposed 事件

extern crate alloc;

use codec::Encode;
use frame_support::{
    ensure,
    traits::{Currency, Get, ReservableCurrency},
};
use institution_asset::{InstitutionAsset, InstitutionAssetAction};
use sp_runtime::{
    traits::{CheckedSub, Zero},
    DispatchResult, SaturatedConversion,
};

use crate::institution::types::{CloseInstitutionAction, InstitutionLifecycleStatus};
use crate::pallet::{
    AddressRegisteredSfid, Config, Error, Event, InstitutionAccounts, InstitutionPendingClose,
    Pallet, ACTION_CLOSE,
};
use crate::traits::{
    DuoqianAddressValidator, DuoqianReservedAddressChecker, ProtectedSourceChecker,
};
use crate::BalanceOf;
use votingengine::InternalVoteEngine;

pub(crate) fn do_propose_institution_close<T: Config>(
    who: T::AccountId,
    duoqian_address: T::AccountId,
    beneficiary: T::AccountId,
) -> DispatchResult {
    // 仅机构地址走本入口
    let registered = AddressRegisteredSfid::<T>::get(&duoqian_address)
        .ok_or(Error::<T>::NotInstitutionDuoqian)?;

    ensure!(
        !T::ProtectedSourceChecker::is_protected(&duoqian_address),
        Error::<T>::ProtectedSource
    );
    ensure!(
        T::InstitutionAsset::can_spend(
            &duoqian_address,
            InstitutionAssetAction::DuoqianCloseExecute,
        ),
        Error::<T>::ProtectedSource
    );
    ensure!(
        beneficiary != duoqian_address,
        Error::<T>::InvalidBeneficiary
    );
    ensure!(
        !T::ReservedAddressChecker::is_reserved(&beneficiary),
        Error::<T>::InvalidBeneficiary
    );
    ensure!(
        T::AddressValidator::is_valid(&beneficiary),
        Error::<T>::InvalidAddress
    );
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&beneficiary),
        Error::<T>::InvalidBeneficiary
    );

    // 校验机构账户已 Active(InstitutionAccounts 状态)
    let account_info =
        InstitutionAccounts::<T>::get(&registered.sfid_id, &registered.account_name)
            .ok_or(Error::<T>::DuoqianNotFound)?;
    ensure!(
        matches!(account_info.status, InstitutionLifecycleStatus::Active),
        Error::<T>::DuoqianNotActive
    );

    // 校验发起人是机构主体的活跃管理员
    let subject_id = Pallet::<T>::resolve_admin_subject_for_account(&duoqian_address)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    let org = votingengine::types::ORG_REN;
    ensure!(
        admins_change::Pallet::<T>::is_active_subject_admin(org, subject_id, &who),
        Error::<T>::PermissionDenied
    );

    // 拒绝并发关闭提案
    ensure!(
        !InstitutionPendingClose::<T>::contains_key(&duoqian_address),
        Error::<T>::CloseAlreadyPending
    );

    let all_balance = T::Currency::free_balance(&duoqian_address);
    ensure!(
        all_balance >= T::MinCloseBalance::get(),
        Error::<T>::CloseBalanceBelowMinimum
    );
    {
        let balance_u128: u128 = all_balance.saturated_into();
        let fee_u128 = onchain_transaction::calculate_onchain_fee(balance_u128);
        let fee: BalanceOf<T> = fee_u128.saturated_into();
        let transfer_amount = all_balance
            .checked_sub(&fee)
            .ok_or(Error::<T>::FeeWithdrawFailed)?;
        let ed = T::Currency::minimum_balance();
        ensure!(transfer_amount >= ed, Error::<T>::CloseTransferBelowED);
    }
    ensure!(
        T::Currency::reserved_balance(&duoqian_address).is_zero(),
        Error::<T>::ReservedBalanceRemaining
    );

    // 关闭提案需全员管理员通过(2026-05-03 整改)。
    let close_threshold = admins_change::Pallet::<T>::active_subject_admin_count(org, subject_id)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    let action = CloseInstitutionAction {
        duoqian_address: duoqian_address.clone(),
        beneficiary: beneficiary.clone(),
        proposer: who.clone(),
    };
    let mut data = alloc::vec::Vec::from(crate::MODULE_TAG);
    data.push(ACTION_CLOSE);
    data.extend_from_slice(&action.encode());
    let proposal_id =
        <T as Config>::InternalVoteEngine::create_internal_proposal_with_threshold_and_data(
            who.clone(),
            org,
            subject_id,
            close_threshold,
            crate::MODULE_TAG,
            data,
        )?;
    InstitutionPendingClose::<T>::insert(&duoqian_address, proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::InstitutionCloseProposed {
        proposal_id,
        duoqian_address,
        proposer: who,
        beneficiary,
    });

    Ok(())
}

/// 执行关闭：转出余额 + 删除 InstitutionAccounts entry 状态置 Closed + 关闭 admin subject。
pub(crate) fn execute_institution_close_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &CloseInstitutionAction<T::AccountId>,
    _callback_context: bool,
) -> DispatchResult {
    ensure!(
        T::InstitutionAsset::can_spend(
            &action.duoqian_address,
            InstitutionAssetAction::DuoqianCloseExecute,
        ),
        Error::<T>::ProtectedSource
    );
    let subject_id = Pallet::<T>::resolve_admin_subject_for_account(&action.duoqian_address)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    let registered = AddressRegisteredSfid::<T>::get(&action.duoqian_address)
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
        use frame_support::traits::{ExistenceRequirement, OnUnbalanced};
        let fee_imbalance = T::Currency::withdraw(
            &action.duoqian_address,
            fee,
            frame_support::traits::WithdrawReasons::FEE,
            ExistenceRequirement::AllowDeath,
        )
        .map_err(|_| Error::<T>::FeeWithdrawFailed)?;
        T::FeeRouter::on_unbalanced(fee_imbalance);
    }

    {
        use frame_support::traits::ExistenceRequirement;
        T::Currency::transfer(
            &action.duoqian_address,
            &action.beneficiary,
            transfer_amount,
            ExistenceRequirement::AllowDeath,
        )
        .map_err(|_| Error::<T>::TransferFailed)?;
    }

    // 删 InstitutionAccounts entry(标记 Closed 状态后整体删除该 entry)。
    InstitutionAccounts::<T>::remove(&registered.sfid_id, &registered.account_name);
    Pallet::<T>::close_admin_subject(proposal_id, subject_id)?;
    InstitutionPendingClose::<T>::remove(&action.duoqian_address);

    Pallet::<T>::deposit_event(Event::<T>::InstitutionClosed {
        proposal_id,
        duoqian_address: action.duoqian_address.clone(),
        beneficiary: action.beneficiary.clone(),
        amount: transfer_amount,
        fee,
    });

    Ok(())
}

// pallet::Call::propose_close 入口仍在 lib.rs 内,delegate 到 do_propose_institution_close。
