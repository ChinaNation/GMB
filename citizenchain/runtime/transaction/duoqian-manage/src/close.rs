//! 多签关闭流程实现(call_index=1)。
//!
//! 关闭流程对个人多签和机构多签**走相同的代码路径**:
//! 1. `resolve_admin_subject_for_account` 按 storage 命中决定主体归属
//!    - 个人多签: 账户自身就是主体地址
//!    - SFID 机构任意账户: 统一归属到该机构主账户主体
//! 2. 通过统一的 `create_internal_proposal_with_data` 创建关闭提案
//! 3. 写入 `PendingCloseProposal[address] = proposal_id` 防并发
//!
//! 所以本入口不需要按 kind 分支拆分,而是放在顶层共用模块。
//! `personal/close.rs` 与 `institution/close.rs` 保留为子目录占位指针。

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

use crate::pallet::{
    Config, DuoqianAccounts, Error, Event, Pallet, PendingCloseProposal, ACTION_CLOSE,
};
use crate::personal::types::{CloseDuoqianAction, DuoqianStatus};
use crate::traits::{
    DuoqianAddressValidator, DuoqianReservedAddressChecker, ProtectedSourceChecker,
};
use crate::BalanceOf;
use voting_engine::InternalVoteEngine;

pub(crate) fn do_propose_close<T: Config>(
    who: T::AccountId,
    duoqian_address: T::AccountId,
    beneficiary: T::AccountId,
) -> DispatchResult {
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

    let account =
        DuoqianAccounts::<T>::get(&duoqian_address).ok_or(Error::<T>::DuoqianNotFound)?;
    ensure!(
        account.status == DuoqianStatus::Active,
        Error::<T>::DuoqianNotActive
    );

    // 发起人必须是该多签主体的管理员。管理员真源统一在 admins-change。
    let subject_id = Pallet::<T>::resolve_admin_subject_for_account(&duoqian_address)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    ensure!(
        admins_change::Pallet::<T>::is_active_subject_admin(
            voting_engine::internal_vote::ORG_DUOQIAN,
            subject_id,
            &who,
        ),
        Error::<T>::PermissionDenied
    );

    // 拒绝对同一多签账户发起并发注销提案
    ensure!(
        !PendingCloseProposal::<T>::contains_key(&duoqian_address),
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

    let org = voting_engine::internal_vote::ORG_DUOQIAN;
    // 中文注释:用 subject_id 作为 voting-engine 治理索引(已由 resolve_admin_subject_for_account
    // 解出),保证个人/机构走同一管理员主体。
    let institution = subject_id;
    // 中文注释:关闭提案需全员管理员通过(2026-05-03 整改)。从 active 主体读 admins.len()。
    let close_threshold = admins_change::Pallet::<T>::active_subject_admin_count(org, institution)
        .ok_or(Error::<T>::DuoqianNotFound)?;
    let action = CloseDuoqianAction {
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
            institution,
            close_threshold,
            crate::MODULE_TAG,
            data,
        )?;
    PendingCloseProposal::<T>::insert(&duoqian_address, proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::CloseDuoqianProposed {
        proposal_id,
        duoqian_address,
        proposer: who,
        beneficiary,
    });

    Ok(())
}
