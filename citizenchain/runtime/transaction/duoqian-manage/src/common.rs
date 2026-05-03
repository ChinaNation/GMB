//! 个人/机构多签共用的基础工具函数。
//!
//! 与 Pallet 内部状态无关的纯函数放在这里。
//! 涉及 Pallet storage 的 helper 仍留在 lib.rs 的 `impl<T: Config> Pallet<T>` 块。

use codec::Encode;
use frame_support::{ensure, traits::Currency};
use sp_runtime::{traits::CheckedAdd, DispatchResult, SaturatedConversion};
use voting_engine::InstitutionPalletId;

use crate::pallet::{Config, Error};
use crate::BalanceOf;

/// 将 AccountId（32 字节）转为 InstitutionPalletId（48 字节），右填充 16 个零。
///
/// **个人多签**的 admin 主体在 `admins-change::Institutions` 表里以
/// `account_to_institution_id(personal_address)` 为 key (32 字节地址 + 16 字节零)。
/// 机构多签**不再用此派生**,改走 `sfid_id_to_institution_id`。
pub fn account_to_institution_id<AccountId: Encode>(account: &AccountId) -> InstitutionPalletId {
    let encoded = account.encode();
    let mut id = [0u8; 48];
    let copy_len = core::cmp::min(encoded.len(), 32);
    id[..copy_len].copy_from_slice(&encoded[..copy_len]);
    id
}

/// 校验发起人 free 余额覆盖 amount + fee + ED,返回 (reserve_total = amount + fee, fee)。
///
/// 个人多签 / 机构整体创建走同一资金模型(2026-05-03 整改): 提案创建时
/// reserve(amount + fee), 投票通过后 unreserve→划转→withdraw fee。
/// 本 helper 集中"金额合法性 + 余额够付"的预检查。
pub(crate) fn ensure_proposer_can_afford<T: Config>(
    who: &T::AccountId,
    amount: BalanceOf<T>,
) -> Result<(BalanceOf<T>, BalanceOf<T>), sp_runtime::DispatchError> {
    let amount_u128: u128 = amount.saturated_into();
    let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
    let fee: BalanceOf<T> = fee_u128.saturated_into();
    let reserve_total = amount
        .checked_add(&fee)
        .ok_or(Error::<T>::InsufficientAmount)?;
    let ed = T::Currency::minimum_balance();
    let required = reserve_total
        .checked_add(&ed)
        .ok_or(Error::<T>::InsufficientAmount)?;
    ensure!(
        T::Currency::free_balance(who) >= required,
        Error::<T>::InsufficientAmount
    );
    Ok((reserve_total, fee))
}

#[allow(dead_code)]
fn _unused_dispatch_result_anchor() -> DispatchResult {
    Ok(())
}

/// 将 sfid_id (= shenfen_id) 字节直接填充到 48 字节 InstitutionPalletId。
///
/// 与 `primitives::china::china_cb::shenfen_id_to_fixed48` 算法一致,确保
/// 机构多签和制度内置主体(NRC/PRC/PRB)的治理索引派生公式同源。
///
/// `sfid_id` 为空或超过 48 字节时返回 None(应在调用方 `ensure!` 拦截)。
pub fn sfid_id_to_institution_id(sfid_id: &[u8]) -> Option<InstitutionPalletId> {
    if sfid_id.is_empty() || sfid_id.len() > 48 {
        return None;
    }
    let mut id = [0u8; 48];
    id[..sfid_id.len()].copy_from_slice(sfid_id);
    Some(id)
}
