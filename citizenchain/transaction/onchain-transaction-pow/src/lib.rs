#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode;
use frame_support::traits::{
    fungible::Inspect,
    tokens::{
        fungible::{Balanced, Credit},
        Fortitude, Imbalance, Precision, Preservation,
    },
    FindAuthor, OnUnbalanced,
};
use frame_support::unsigned::TransactionValidityError;
use pallet_transaction_payment::{Config as TxPaymentConfig, OnChargeTransaction, TxCreditHold};
use sp_runtime::{
    traits::{DispatchInfoOf, PostDispatchInfoOf, SaturatedConversion, Saturating, Zero},
    transaction_validity::InvalidTransaction,
};
use sp_std::{marker::PhantomData, prelude::*};

/// 链上 PoW 交易手续费分配器（统一入口）：
/// - 全节点（绑定钱包）分成：`ONCHAIN_FEE_FULLNODE_PERCENT`
/// - 国储会分成：`ONCHAIN_FEE_NRC_PERCENT`
/// - 黑洞销毁：`ONCHAIN_FEE_BLACKHOLE_PERCENT`
pub struct PowOnchainFeeRouter<T, Currency, AuthorFinder>(PhantomData<(T, Currency, AuthorFinder)>);

/// 金额提取分类结果：
/// - Amount: 确认是“有金额交易”，并返回金额
/// - NoAmount: 确认是“无金额交易”
/// - Unknown: 无法确认（按制度应拒绝，避免漏提取）
pub enum AmountExtractResult<Balance> {
    Amount(Balance),
    NoAmount,
    Unknown,
}

/// 统一抽象：由 Runtime 提供“交易金额提取器”。
pub trait CallAmount<AccountId, Call, Balance> {
    fn amount(who: &AccountId, call: &Call) -> AmountExtractResult<Balance>;
}

/// 链上 PoW 手续费收取适配器：
/// - 手续费按交易金额 `ONCHAIN_FEE_RATE` 计算
/// - 单笔最低 `ONCHAIN_MIN_FEE`
/// - 具体分配交给 `PowOnchainFeeRouter`
pub struct PowOnchainChargeAdapter<Currency, Router, AmountExtractor>(
    PhantomData<(Currency, Router, AmountExtractor)>,
);

impl<T, Currency, Router, AmountExtractor> OnChargeTransaction<T>
    for PowOnchainChargeAdapter<Currency, Router, AmountExtractor>
where
    T: TxPaymentConfig + fullnode_pow_reward::Config,
    Currency: Balanced<T::AccountId> + 'static,
    Router: OnUnbalanced<Credit<T::AccountId, Currency>>,
    AmountExtractor:
        CallAmount<T::AccountId, T::RuntimeCall, <Currency as Inspect<T::AccountId>>::Balance>,
{
    type LiquidityInfo = Option<(
        Credit<T::AccountId, Currency>,
        Credit<T::AccountId, Currency>,
    )>;
    type Balance = <Currency as Inspect<T::AccountId>>::Balance;

    fn withdraw_fee(
        who: &T::AccountId,
        call: &T::RuntimeCall,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _fee_with_tip: Self::Balance,
        tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        let fee_with_tip =
            custom_fee_with_tip::<T, Currency, AmountExtractor>(who, call, dispatch_info, tip)?;
        if fee_with_tip.is_zero() {
            return Ok(None);
        }

        let credit = Currency::withdraw(
            who,
            fee_with_tip,
            Precision::Exact,
            Preservation::Preserve,
            Fortitude::Polite,
        )
        .map_err(|_| InvalidTransaction::Payment)?;

        let (tip_credit, inclusion_fee) = credit.split(tip);
        Ok(Some((inclusion_fee, tip_credit)))
    }

    fn can_withdraw_fee(
        who: &T::AccountId,
        call: &T::RuntimeCall,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _fee_with_tip: Self::Balance,
        tip: Self::Balance,
    ) -> Result<(), TransactionValidityError> {
        let fee_with_tip =
            custom_fee_with_tip::<T, Currency, AmountExtractor>(who, call, dispatch_info, tip)?;
        if fee_with_tip.is_zero() {
            return Ok(());
        }
        match Currency::can_withdraw(who, fee_with_tip) {
            frame_support::traits::tokens::WithdrawConsequence::Success => Ok(()),
            _ => Err(InvalidTransaction::Payment.into()),
        }
    }

    fn correct_and_deposit_fee(
        _who: &T::AccountId,
        _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        _corrected_fee_with_tip: Self::Balance,
        _tip: Self::Balance,
        liquidity_info: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        // 中文注释：本制度按交易金额固定收费，不做基于执行后权重的退款。
        if let Some((fee_credit, tip_credit)) = liquidity_info {
            Router::on_unbalanceds(Some(fee_credit).into_iter().chain(Some(tip_credit)));
        }
        Ok(())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn endow_account(who: &T::AccountId, amount: Self::Balance) {
        let _ = Currency::deposit(who, amount, Precision::BestEffort);
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn minimum_balance() -> Self::Balance {
        Currency::minimum_balance()
    }
}

impl<T, Currency, Router, AmountExtractor> TxCreditHold<T>
    for PowOnchainChargeAdapter<Currency, Router, AmountExtractor>
where
    T: TxPaymentConfig,
    Currency: Balanced<T::AccountId> + 'static,
{
    type Credit = ();
}

impl<T, Currency, AuthorFinder> OnUnbalanced<Credit<T::AccountId, Currency>>
    for PowOnchainFeeRouter<T, Currency, AuthorFinder>
where
    T: frame_system::Config + fullnode_pow_reward::Config,
    Currency: Balanced<T::AccountId>,
    AuthorFinder: FindAuthor<T::AccountId>,
{
    fn on_nonzero_unbalanced(amount: Credit<T::AccountId, Currency>) {
        let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT;
        let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT;
        let blackhole_percent = primitives::core_const::ONCHAIN_FEE_BLACKHOLE_PERCENT;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(blackhole_percent);

        // 中文注释：制度常量异常时，直接全部销毁，避免错误分配。
        if total_percent == 0 {
            return;
        }

        let (fullnode_credit, remainder) = amount.ration(
            fullnode_percent,
            total_percent.saturating_sub(fullnode_percent),
        );
        let (nrc_credit, blackhole_credit) = remainder.ration(nrc_percent, blackhole_percent);

        // 中文注释：手续费全节点分成只发给“当前区块作者对应绑定钱包”；未绑定则不分配（自动销毁）。
        let digest = <frame_system::Pallet<T>>::digest();
        let pre_runtime_digests = digest.logs().iter().filter_map(|d| d.as_pre_runtime());
        if let Some(miner) = AuthorFinder::find_author(pre_runtime_digests) {
            if let Some(wallet) = fullnode_pow_reward::RewardWalletByMiner::<T>::get(&miner) {
                let _ = Currency::resolve(&wallet, fullnode_credit);
            }
        }

        // 中文注释：国储会分成发到常量数组中的 nrcgch01 交易地址；解析失败则自动销毁。
        if let Some(nrc_account) = nrc_account::<T>() {
            let _ = Currency::resolve(&nrc_account, nrc_credit);
        }
        // 中文注释：黑洞分成统一转入常量黑洞地址；若地址解析失败则自动销毁。
        if let Some(blackhole_account) = blackhole_account::<T>() {
            let _ = Currency::resolve(&blackhole_account, blackhole_credit);
        }
        // 未被 resolve 的余额离开作用域后自动销毁。
    }
}

fn custom_fee_with_tip<T, Currency, AmountExtractor>(
    who: &T::AccountId,
    call: &T::RuntimeCall,
    _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
    tip: <Currency as Inspect<T::AccountId>>::Balance,
) -> Result<<Currency as Inspect<T::AccountId>>::Balance, TransactionValidityError>
where
    T: TxPaymentConfig,
    Currency: Balanced<T::AccountId>,
    AmountExtractor:
        CallAmount<T::AccountId, T::RuntimeCall, <Currency as Inspect<T::AccountId>>::Balance>,
{
    // 中文注释：有金额交易必须提取金额收费；无金额交易放行不收费；无法判断则拒绝，防止漏提取。
    let amount = match AmountExtractor::amount(who, call) {
        AmountExtractResult::Amount(v) => v,
        AmountExtractResult::NoAmount => return Ok(tip),
        AmountExtractResult::Unknown => return Err(InvalidTransaction::Payment.into()),
    };
    let amount_u128: u128 = amount.saturated_into();
    let by_rate: u128 = primitives::core_const::ONCHAIN_FEE_RATE.mul_floor(amount_u128);
    let min_fee: u128 = primitives::core_const::ONCHAIN_MIN_FEE;
    let base_fee: <Currency as Inspect<T::AccountId>>::Balance =
        by_rate.max(min_fee).saturated_into();
    Ok(base_fee.saturating_add(tip))
}

fn nrc_account<T: frame_system::Config>() -> Option<T::AccountId> {
    primitives::reserve_nodes_const::RESERVE_NODES
        .iter()
        .find(|n| n.pallet_id == "nrcgch01")
        .and_then(|n| T::AccountId::decode(&mut &n.pallet_address[..]).ok())
}

fn blackhole_account<T: frame_system::Config>() -> Option<T::AccountId> {
    T::AccountId::decode(&mut &primitives::core_const::BLACKHOLE_ADDRESS[..]).ok()
}
