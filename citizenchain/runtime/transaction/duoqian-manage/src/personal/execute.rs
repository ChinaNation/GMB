//! 个人多签投票回调清理路径。
//!
//! 个人多签的"创建/关闭执行"主路径在 `crate::execute`(对个人/机构 ACTION_CREATE_PERSONAL
//! 和 ACTION_CLOSE 共用)。这里专放个人多签创建提案被否决/终态失败时的清理:
//! unreserve 资金 + 删 Pending storage + remove admin subject。
//!
//! 机构整体创建 ACTION_CREATE_INSTITUTION 的对应清理在 `crate::institution::execute`。

extern crate alloc;

use frame_support::traits::ReservableCurrency;
use sp_runtime::traits::Saturating;

use crate::common::account_to_institution_id;
use crate::pallet::{
    Config, DuoqianAccounts, Event, Pallet, PendingPersonalCreate, PersonalDuoqianInfo,
};
use crate::personal::types::CreateDuoqianAction;
use crate::BalanceOf;

/// 个人多签创建提案被否决/超时/终态失败时清理:
/// unreserve(amount + fee) + 删 DuoqianAccounts/PersonalDuoqianInfo/PendingPersonalCreate +
/// 移除 admin subject Pending。
///
/// `emit_event = true` 时(否决路径)发 `DuoqianCreateRejected`,终态失败路径不发。
pub(crate) fn cleanup_pending_personal_create<T: Config>(
    proposal_id: u64,
    action: &CreateDuoqianAction<T::AccountId, BalanceOf<T>>,
    emit_event: bool,
) {
    let amount_u128: u128 = sp_runtime::SaturatedConversion::saturated_into(action.amount);
    let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
    let fee: BalanceOf<T> = sp_runtime::SaturatedConversion::saturated_into(fee_u128);
    let reserve_total = action.amount.saturating_add(fee);
    let _ = T::Currency::unreserve(&action.proposer, reserve_total);

    DuoqianAccounts::<T>::remove(&action.duoqian_address);
    PersonalDuoqianInfo::<T>::remove(&action.duoqian_address);
    PendingPersonalCreate::<T>::remove(proposal_id);
    Pallet::<T>::remove_pending_admin_subject(
        proposal_id,
        account_to_institution_id(&action.duoqian_address),
    );

    if emit_event {
        Pallet::<T>::deposit_event(Event::<T>::DuoqianCreateRejected {
            proposal_id,
            duoqian_address: action.duoqian_address.clone(),
        });
    }
}
