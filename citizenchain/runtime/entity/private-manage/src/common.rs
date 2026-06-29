//! 私权机构多签共用的基础工具函数。
//!
//! 与 Pallet 内部状态无关的纯函数放在这里。
//! 涉及 Pallet storage 的 helper 仍留在 lib.rs 的 `impl<T: Config> Pallet<T>` 块。
//!
//! 账户地址派生只允许调用 `primitives::account_derive`(唯一真源)
//! 及各业务模块对它的薄封装；本文件不再保存任何派生协议常量。

use frame_support::{ensure, traits::Currency};
use sp_runtime::{traits::CheckedAdd, DispatchResult, SaturatedConversion};

use crate::pallet::{Config, Error};
use crate::BalanceOf;

/// 校验发起人 free 余额覆盖 amount + fee + ED,返回 (reserve_total = amount + fee, fee)。
///
/// 私权机构多签的资金模型:提案创建时 reserve(amount + fee),
/// 投票通过后 unreserve→划转→withdraw fee。本 helper 集中"金额合法性 + 余额够付"
/// 的预检查;personal-manage 自持平行实现。
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
