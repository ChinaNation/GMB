//! 个人多签关闭流程实现(call_index=1)。
//!
//! 仅接受个人多签账户(`PersonalAccounts.contains_key` 命中),
//! 否则返回 `Error::NotPersonalAccount`;机构多签关闭走 organization-manage 入口。
//!
//! 业务流程：
//! 1. 校验地址、受益人、地址非保留
//! 2. 校验地址 PersonalAccounts 已 Active
//! 3. 校验发起人是该个人多签账户的活跃管理员
//! 4. 校验余额≥关闭门槛 + 转出金额≥ED + 无 reserved 余额
//! 5. 注销生命周期投票的全员阈值由投票引擎按管理员快照生成
//! 6. 写入 PendingCloseProposal[address] = proposal_id 防并发
//! 7. 发射 PersonalCloseProposed 事件

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

use crate::pallet::{Config, Error, Event, Pallet, PendingCloseProposal, PersonalAccounts};
use crate::types::{PersonalCloseAction, PersonalStatus};
use crate::BalanceOf;
use crate::ACTION_CLOSE;
use primitives::multisig::{AccountValidator, ProtectedSourceChecker, ReservedAccountGuard};
use votingengine::InternalVoteEngine;

pub(crate) fn do_propose_close<T: Config>(
    who: T::AccountId,
    account: T::AccountId,
    beneficiary: T::AccountId,
) -> DispatchResult {
    // 仅个人多签可走本入口
    ensure!(
        PersonalAccounts::<T>::contains_key(&account),
        Error::<T>::NotPersonalAccount
    );

    ensure!(
        !T::ProtectedSourceChecker::is_protected(&account),
        Error::<T>::ProtectedSource
    );
    ensure!(
        T::InstitutionAsset::can_spend(&account, InstitutionAssetAction::MultisigCloseExecute,),
        Error::<T>::ProtectedSource
    );
    ensure!(beneficiary != account, Error::<T>::InvalidBeneficiary);
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

    let account_info = PersonalAccounts::<T>::get(&account).ok_or(Error::<T>::PersonalNotFound)?;
    ensure!(
        account_info.status == PersonalStatus::Active,
        Error::<T>::PersonalNotActive
    );

    // 个人多签治理账户直接使用个人多签账户地址。
    let institution = account.clone();
    let institution_code = votingengine::types::PMUL;
    ensure!(
        Pallet::<T>::is_active_account_admin(institution_code, institution.clone(), &who),
        Error::<T>::PermissionDenied
    );

    ensure!(
        !PendingCloseProposal::<T>::contains_key(&account),
        Error::<T>::CloseAlreadyPending
    );

    let all_balance = T::Currency::free_balance(&account);
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
        T::Currency::reserved_balance(&account).is_zero(),
        Error::<T>::ReservedBalanceRemaining
    );

    let action = PersonalCloseAction {
        account: account.clone(),
        beneficiary: beneficiary.clone(),
        proposer: who.clone(),
    };
    let mut data = alloc::vec::Vec::from(crate::MODULE_TAG);
    data.push(ACTION_CLOSE);
    data.extend_from_slice(&action.encode());
    let proposal_id =
        <T as Config>::InternalVoteEngine::create_lifecycle_internal_proposal_with_data(
            who.clone(),
            institution_code,
            institution,
            crate::MODULE_TAG,
            data,
        )?;
    PendingCloseProposal::<T>::insert(&account, proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::PersonalCloseProposed {
        proposal_id,
        account,
        proposer: who,
        beneficiary,
    });

    Ok(())
}
