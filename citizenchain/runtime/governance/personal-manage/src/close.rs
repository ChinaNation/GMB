//! 个人多签关闭流程实现(call_index=1)。
//!
//! 仅接受个人多签地址(`PersonalDuoqians.contains_key` 命中),
//! 否则返回 `Error::NotPersonalDuoqian`;机构多签关闭走 organization-manage 入口。
//!
//! 业务流程：
//! 1. 校验地址、受益人、地址非保留
//! 2. 校验地址 PersonalDuoqians 已 Active
//! 3. 校验发起人是该多签 admins-change 主体的活跃管理员
//! 4. 校验余额≥关闭门槛 + 转出金额≥ED + 无 reserved 余额
//! 5. 注销生命周期投票的全员阈值由投票引擎按管理员快照生成
//! 6. 写入 PendingCloseProposal[address] = proposal_id 防并发
//! 7. 发射 CloseDuoqianProposed 事件

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

use crate::pallet::{Config, Error, Event, Pallet, PendingCloseProposal, PersonalDuoqians};
use crate::types::{CloseDuoqianAction, DuoqianStatus};
use crate::BalanceOf;
use crate::ACTION_CLOSE;
use primitives::derive::subject_id_from_account;
use primitives::traits::{
    DuoqianAddressValidator, DuoqianReservedAddressChecker, ProtectedSourceChecker,
};
use votingengine::InternalVoteEngine;

pub(crate) fn do_propose_close<T: Config>(
    who: T::AccountId,
    duoqian_address: T::AccountId,
    beneficiary: T::AccountId,
) -> DispatchResult {
    // 仅个人多签可走本入口
    ensure!(
        PersonalDuoqians::<T>::contains_key(&duoqian_address),
        Error::<T>::NotPersonalDuoqian
    );

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
        PersonalDuoqians::<T>::get(&duoqian_address).ok_or(Error::<T>::DuoqianNotFound)?;
    ensure!(
        account.status == DuoqianStatus::Active,
        Error::<T>::DuoqianNotActive
    );

    // 个人多签的治理主体 institution_id = subject_id_from_account(personal_address)。
    let institution = subject_id_from_account(&duoqian_address);
    let org = votingengine::types::ORG_REN;
    ensure!(
        admins_change::Pallet::<T>::is_active_subject_admin(org, institution, &who),
        Error::<T>::PermissionDenied
    );

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

    let action = CloseDuoqianAction {
        duoqian_address: duoqian_address.clone(),
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
            institution,
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
