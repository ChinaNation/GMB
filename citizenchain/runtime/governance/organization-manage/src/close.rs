//! 机构多签关闭流程实现(call_index=1)。
//!
//! 仅服务于已注册的 SFID 机构账户(`AccountRegisteredSfid.contains_key` 命中);
//! 个人多签关闭走 personal-manage::propose_close 入口。
//!
//! 业务流程:
//! 1. 校验地址是机构地址(否则返回 `NotInstitutionDuoqian`)
//! 2. 校验机构账户已 Active(从 InstitutionAccounts 读)
//! 3. 校验发起人是该机构账户的活跃管理员(admins-change::AdminAccounts[account account])
//! 4. 校验余额≥关闭门槛 + 转出金额≥ED + 无 reserved 余额
//! 5. 注销生命周期投票的全员阈值由投票引擎按管理员快照生成
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
    AccountRegisteredSfid, Config, Error, Event, InstitutionAccounts, InstitutionPendingClose,
    Pallet, SfidRegisteredAccount, ACTION_CLOSE,
};
use crate::traits::{
    DuoqianAccountValidator, DuoqianReservedAccountChecker, ProtectedSourceChecker,
};
use crate::BalanceOf;
use votingengine::InternalVoteEngine;

pub(crate) fn do_propose_institution_close<T: Config>(
    who: T::AccountId,
    duoqian_account: T::AccountId,
    beneficiary: T::AccountId,
) -> DispatchResult {
    // 仅机构地址走本入口
    let registered = AccountRegisteredSfid::<T>::get(&duoqian_account)
        .ok_or(Error::<T>::NotInstitutionDuoqian)?;

    ensure!(
        !T::ProtectedSourceChecker::is_protected(&duoqian_account),
        Error::<T>::ProtectedSource
    );
    ensure!(
        T::InstitutionAsset::can_spend(
            &duoqian_account,
            InstitutionAssetAction::DuoqianCloseExecute,
        ),
        Error::<T>::ProtectedSource
    );
    ensure!(
        beneficiary != duoqian_account,
        Error::<T>::InvalidBeneficiary
    );
    ensure!(
        !T::ReservedAccountChecker::is_reserved(&beneficiary),
        Error::<T>::InvalidBeneficiary
    );
    ensure!(
        T::AccountValidator::is_valid(&beneficiary),
        Error::<T>::InvalidAccount
    );
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&beneficiary),
        Error::<T>::InvalidBeneficiary
    );

    // 校验机构账户已 Active(InstitutionAccounts 状态)
    let account_info =
        InstitutionAccounts::<T>::get(&registered.sfid_number, &registered.account_name)
            .ok_or(Error::<T>::DuoqianNotFound)?;
    ensure!(
        matches!(account_info.status, InstitutionLifecycleStatus::Active),
        Error::<T>::DuoqianNotActive
    );

    // 校验发起人是机构账户的活跃管理员
    let account = Pallet::<T>::resolve_admin_account_for_account(&duoqian_account)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    let org = Pallet::<T>::resolve_admin_org_for_account(&duoqian_account)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    ensure!(
        admins_change::Pallet::<T>::is_active_account_admin(org, account.clone(), &who),
        Error::<T>::PermissionDenied
    );

    // 拒绝并发关闭提案
    ensure!(
        !InstitutionPendingClose::<T>::contains_key(&duoqian_account),
        Error::<T>::CloseAlreadyPending
    );

    let all_balance = T::Currency::free_balance(&duoqian_account);
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
        T::Currency::reserved_balance(&duoqian_account).is_zero(),
        Error::<T>::ReservedBalanceRemaining
    );

    let action = CloseInstitutionAction {
        duoqian_account: duoqian_account.clone(),
        beneficiary: beneficiary.clone(),
        proposer: who.clone(),
    };
    let mut data = alloc::vec::Vec::from(crate::MODULE_TAG);
    data.push(ACTION_CLOSE);
    data.extend_from_slice(&action.encode());
    let proposal_id =
        <T as Config>::InternalVoteEngine::create_lifecycle_internal_proposal_with_data(
            who.clone(),
            org,
            account,
            crate::MODULE_TAG,
            data,
        )?;
    InstitutionPendingClose::<T>::insert(&duoqian_account, proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::InstitutionCloseProposed {
        proposal_id,
        duoqian_account,
        proposer: who,
        beneficiary,
    });

    Ok(())
}

/// 执行关闭：转出余额 + 删除 InstitutionAccounts entry 状态置 Closed + 关闭 admin account。
pub(crate) fn execute_institution_close_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &CloseInstitutionAction<T::AccountId>,
    _callback_context: bool,
) -> DispatchResult {
    ensure!(
        T::InstitutionAsset::can_spend(
            &action.duoqian_account,
            InstitutionAssetAction::DuoqianCloseExecute,
        ),
        Error::<T>::ProtectedSource
    );
    let account = Pallet::<T>::resolve_admin_account_for_account(&action.duoqian_account)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    let registered = AccountRegisteredSfid::<T>::get(&action.duoqian_account)
        .ok_or(Error::<T>::DuoqianNotFound)?;

    let all_balance = T::Currency::free_balance(&action.duoqian_account);
    // 中文注释：执行阶段也复核 reserved，保证注销完成后账户能被彻底清空和复用。
    ensure!(
        T::Currency::reserved_balance(&action.duoqian_account).is_zero(),
        Error::<T>::ReservedBalanceRemaining
    );
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
            &action.duoqian_account,
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
            &action.duoqian_account,
            &action.beneficiary,
            transfer_amount,
            ExistenceRequirement::AllowDeath,
        )
        .map_err(|_| Error::<T>::TransferFailed)?;
    }

    // 中文注释：机构账户注销成功后必须删除账户当前索引；历史事件/提案仍保留在链历史中。
    InstitutionAccounts::<T>::remove(&registered.sfid_number, &registered.account_name);
    SfidRegisteredAccount::<T>::remove(&registered.sfid_number, &registered.account_name);
    AccountRegisteredSfid::<T>::remove(&action.duoqian_account);
    Pallet::<T>::close_admin_account(proposal_id, account)?;
    InstitutionPendingClose::<T>::remove(&action.duoqian_account);

    Pallet::<T>::deposit_event(Event::<T>::InstitutionClosed {
        proposal_id,
        duoqian_account: action.duoqian_account.clone(),
        beneficiary: action.beneficiary.clone(),
        amount: transfer_amount,
        fee,
    });

    Ok(())
}

// pallet::Call::propose_close 入口仍在 lib.rs 内,delegate 到 do_propose_institution_close。
