//! 扫码支付清算体系 Step 1 新增:L3 绑定清算行 + 充值 + 提现 + 切换。
//!
//! 中文注释:
//! - 绑定 = 开户,**无预存、无业务开户费**,链上仅产生付费调用 1 元/次(由
//!   `configs/mod.rs` 的 `OnchainTxAmountExtractor` 统一归类扣费)。
//! - 充值 / 提现走链上资金交易路径,由 L3 自持账户 ↔ 清算行主账户,链上费
//!   按金额 0.1% 最低 0.1 元(沿用既有规则)。
//! - 切换清算行无次数 / 时间间隔限制,**前置:旧清算行余额必须清零**。
//! - 本模块所有扣款/入账都必须过 `institution-asset` 的 `can_spend`,
//!   把"清算行主账户可被扣"这条规则统一落到资金白名单层。

use frame_support::{
    ensure,
    traits::{Currency, ExistenceRequirement::KeepAlive},
};
use institution_asset::{InstitutionAsset, InstitutionAssetAction};
use sp_runtime::{traits::SaturatedConversion, DispatchError, DispatchResult};

use crate::{
    bank_check, BankTotalDeposits, Config, DepositBalance, Error, Event, Pallet, UserBank,
};

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// `bind_clearing_bank`:L3 绑定清算行主账户,绑定即开户。
///
/// 约束:
/// - L3 未绑定其他清算行
/// - `bank_main_address` 必须满足 `bank_check::ensure_can_be_bound`(SFR/FFR + Active + 主账户)
/// - 无预存,`DepositBalance` 初始化为 0
pub fn do_bind_clearing_bank<T: Config>(
    user: T::AccountId,
    bank_main_address: T::AccountId,
) -> DispatchResult {
    // 1. 不允许重复绑定
    ensure!(
        !UserBank::<T>::contains_key(&user),
        Error::<T>::AlreadyHasBank
    );

    // 2. 清算行合法性(A3 + Active + 主账户三重校验)
    bank_check::ensure_can_be_bound::<T>(&bank_main_address).map_err(DispatchError::from)?;

    // 3. 落存储
    UserBank::<T>::insert(&user, &bank_main_address);
    DepositBalance::<T>::insert(&bank_main_address, &user, 0u128);

    Pallet::<T>::deposit_event(Event::<T>::BankBound {
        user,
        bank: bank_main_address,
    });
    Ok(())
}

/// `deposit`:L3 自持账户 → 清算行主账户充值。
pub fn do_deposit<T: Config>(user: T::AccountId, amount: u128) -> DispatchResult {
    ensure!(amount > 0, Error::<T>::DepositAmountTooSmall);

    let bank = UserBank::<T>::get(&user).ok_or(Error::<T>::NoOpenedBank)?;

    // 资金白名单:允许 L3 向清算行主账户转入(DepositIn 动作)
    ensure!(
        T::InstitutionAsset::can_spend(&user, InstitutionAssetAction::L3DepositIn),
        Error::<T>::DepositForbidden
    );

    let balance: BalanceOf<T> = amount.saturated_into();
    T::Currency::transfer(&user, &bank, balance, KeepAlive)?;

    DepositBalance::<T>::mutate(&bank, &user, |b| *b = b.saturating_add(amount));
    BankTotalDeposits::<T>::mutate(&bank, |t| *t = t.saturating_add(amount));

    Pallet::<T>::deposit_event(Event::<T>::Deposited { user, bank, amount });
    Ok(())
}

/// `withdraw`:清算行主账户 → L3 自持账户提现。
pub fn do_withdraw<T: Config>(user: T::AccountId, amount: u128) -> DispatchResult {
    ensure!(amount > 0, Error::<T>::WithdrawAmountTooSmall);

    let bank = UserBank::<T>::get(&user).ok_or(Error::<T>::NoOpenedBank)?;
    let current = DepositBalance::<T>::get(&bank, &user);
    ensure!(current >= amount, Error::<T>::InsufficientDepositBalance);

    // 资金白名单:清算行主账户可向外转出(L3 提现)
    ensure!(
        T::InstitutionAsset::can_spend(&bank, InstitutionAssetAction::L3WithdrawOut),
        Error::<T>::WithdrawForbidden
    );

    let balance: BalanceOf<T> = amount.saturated_into();
    T::Currency::transfer(&bank, &user, balance, KeepAlive)
        .map_err(|_| Error::<T>::InsufficientBankLiquidity)?;

    DepositBalance::<T>::mutate(&bank, &user, |b| *b = b.saturating_sub(amount));
    BankTotalDeposits::<T>::mutate(&bank, |t| *t = t.saturating_sub(amount));

    Pallet::<T>::deposit_event(Event::<T>::Withdrawn { user, bank, amount });
    Ok(())
}

/// `switch_bank`:切换清算行。
///
/// 前置条件:
/// - L3 当前已绑定清算行
/// - 旧清算行的 `DepositBalance` 必须为 0(否则要求先 withdraw 清零)
/// - 新清算行 != 旧清算行
/// - 新清算行满足 `bank_check::ensure_can_be_bound`
pub fn do_switch_bank<T: Config>(user: T::AccountId, new_bank: T::AccountId) -> DispatchResult {
    let old_bank = UserBank::<T>::get(&user).ok_or(Error::<T>::NoOpenedBank)?;
    ensure!(old_bank != new_bank, Error::<T>::NewBankSameAsCurrent);
    ensure!(
        DepositBalance::<T>::get(&old_bank, &user) == 0,
        Error::<T>::MustClearBalanceFirst
    );

    bank_check::ensure_can_be_bound::<T>(&new_bank).map_err(DispatchError::from)?;

    // 移除旧绑定下的零余额条目(保持 Storage 干净),换到新清算行
    DepositBalance::<T>::remove(&old_bank, &user);
    UserBank::<T>::insert(&user, &new_bank);
    DepositBalance::<T>::insert(&new_bank, &user, 0u128);

    Pallet::<T>::deposit_event(Event::<T>::BankSwitched {
        user,
        old_bank,
        new_bank,
    });
    Ok(())
}
