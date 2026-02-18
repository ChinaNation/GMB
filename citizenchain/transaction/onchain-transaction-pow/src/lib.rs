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

/// 可选扣费来源：
/// - None：沿用默认交易提交者扣费
/// - Some(account)：从指定账户扣费
pub trait CallFeePayer<AccountId, Call> {
    fn fee_payer(who: &AccountId, call: &Call) -> Option<AccountId>;
}

impl<AccountId, Call> CallFeePayer<AccountId, Call> for () {
    fn fee_payer(_who: &AccountId, _call: &Call) -> Option<AccountId> {
        None
    }
}

/// 链上 PoW 手续费收取适配器：
/// - 手续费按交易金额 `ONCHAIN_FEE_RATE` 计算
/// - 单笔最低 `ONCHAIN_MIN_FEE`
/// - 具体分配交给 `PowOnchainFeeRouter`
pub struct PowOnchainChargeAdapter<Currency, Router, AmountExtractor, FeePayerExtractor>(
    PhantomData<(Currency, Router, AmountExtractor, FeePayerExtractor)>,
);

impl<T, Currency, Router, AmountExtractor, FeePayerExtractor> OnChargeTransaction<T>
    for PowOnchainChargeAdapter<Currency, Router, AmountExtractor, FeePayerExtractor>
where
    T: TxPaymentConfig + fullnode_pow_reward::Config,
    Currency: Balanced<T::AccountId> + 'static,
    Router: OnUnbalanced<Credit<T::AccountId, Currency>>,
    AmountExtractor:
        CallAmount<T::AccountId, T::RuntimeCall, <Currency as Inspect<T::AccountId>>::Balance>,
    FeePayerExtractor: CallFeePayer<T::AccountId, T::RuntimeCall>,
    T::AccountId: Clone,
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
        let payer = FeePayerExtractor::fee_payer(who, call).unwrap_or_else(|| who.clone());

        let credit = Currency::withdraw(
            &payer,
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
        let payer = FeePayerExtractor::fee_payer(who, call).unwrap_or_else(|| who.clone());
        match Currency::can_withdraw(&payer, fee_with_tip) {
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

impl<T, Currency, Router, AmountExtractor, FeePayerExtractor> TxCreditHold<T>
    for PowOnchainChargeAdapter<Currency, Router, AmountExtractor, FeePayerExtractor>
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
        // 中文注释：黑洞分成改为“直接销毁”（不入任何地址），总发行量同步减少。
        // 这里不再向 BLACKHOLE_ADDRESS 转账。
        drop(blackhole_credit);
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
    let by_rate: u128 = mul_perbill_round(amount_u128, primitives::core_const::ONCHAIN_FEE_RATE);
    let min_fee: u128 = primitives::core_const::ONCHAIN_MIN_FEE; // 0.1元=10分
    let base_fee: <Currency as Inspect<T::AccountId>>::Balance =
        by_rate.max(min_fee).saturated_into();
    Ok(base_fee.saturating_add(tip))
}

fn mul_perbill_round(amount: u128, rate: sp_runtime::Perbill) -> u128 {
    // 中文注释：链上精度为“分”，这里做四舍五入到分。
    const PERBILL_DENOMINATOR: u128 = 1_000_000_000;
    let parts: u128 = rate.deconstruct() as u128;
    amount
        .saturating_mul(parts)
        .saturating_add(PERBILL_DENOMINATOR / 2)
        .saturating_div(PERBILL_DENOMINATOR)
}

fn nrc_account<T: frame_system::Config>() -> Option<T::AccountId> {
    primitives::reserve_nodes_const::RESERVE_NODES
        .iter()
        .find(|n| n.pallet_id == "nrcgch01")
        .and_then(|n| T::AccountId::decode(&mut &n.pallet_address[..]).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_runtime::Perbill;

    #[test]
    fn onchain_fee_round_and_min_work() {
        let rate = Perbill::from_parts(1_000_000); // 0.1%
                                                   // 1分*0.1%=0.001分 => round=0分，应用最低10分
        let fee_small = mul_perbill_round(1, rate).max(primitives::core_const::ONCHAIN_MIN_FEE);
        assert_eq!(fee_small, 10);

        // 10000分(100元)*0.1%=10分，刚好最低线
        let fee_boundary =
            mul_perbill_round(10_000, rate).max(primitives::core_const::ONCHAIN_MIN_FEE);
        assert_eq!(fee_boundary, 10);

        // 50000分(500元)*0.1%=50分，大于最低线按实际收取
        let fee_large =
            mul_perbill_round(50_000, rate).max(primitives::core_const::ONCHAIN_MIN_FEE);
        assert_eq!(fee_large, 50);
    }
}
