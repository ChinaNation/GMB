#![cfg_attr(not(feature = "std"), no_std)]

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
    RuntimeDebug,
};
use sp_std::{marker::PhantomData, prelude::*};

/// 最小 pallet：仅承载手续费事件，无 storage、无 call、无 hooks。
#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_runtime::RuntimeDebug;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {}

    /// 中文注释：手续费份额销毁原因，供链上事件审计和运维聚合。
    #[derive(
        Clone,
        Copy,
        Encode,
        Decode,
        DecodeWithMemTracking,
        Eq,
        PartialEq,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub enum BurnReason {
        /// 中文注释：当前区块作者无法从共识 digest 中识别。
        AuthorMissing,
        /// 中文注释：区块作者尚未绑定全节点手续费奖励钱包。
        WalletUnbound,
        /// 中文注释：全节点奖励钱包入账失败，剩余 credit 被销毁。
        FullnodeResolveFailed,
        /// 中文注释：国储会手续费账户未配置。
        NrcMissing,
        /// 中文注释：国储会手续费账户入账失败，剩余 credit 被销毁。
        NrcResolveFailed,
        /// 中文注释：安全基金账户入账失败，剩余 credit 被销毁。
        SafetyFundResolveFailed,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 交易手续费已收取。
        /// NOTE: `fee` 只包含基础手续费，不包含 tip；调整 tip 机制时必须同步
        /// `ONCHAIN_TECHNICAL.md` 第 11 节以及下游 RPC / dashboard 统计口径。
        FeePaid { who: T::AccountId, fee: u128 },
        /// 手续费分账份额因无法安全入账而被销毁。
        FeeShareBurnt { reason: BurnReason, amount: u128 },
    }
}

const EXPECTED_FEE_PERCENT_TOTAL: u32 = 100;

const _: () = {
    let total_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT
        .saturating_add(primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT)
        .saturating_add(primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT);
    assert!(
        total_percent == EXPECTED_FEE_PERCENT_TOTAL,
        "fee distribution percents must sum to 100"
    );
    assert!(
        primitives::fee_policy::ONCHAIN_MIN_FEE > 0,
        "ONCHAIN_MIN_FEE must be positive"
    );
    assert!(
        primitives::fee_policy::ONCHAIN_FEE_RATE.deconstruct() > 0,
        "ONCHAIN_FEE_RATE must be non-zero"
    );
};

/// 链上交易手续费分配器（统一入口）：
/// - 全节点（绑定钱包）分成：`ONCHAIN_FEE_FULLNODE_PERCENT`（80%）
/// - 国储会手续费账户分成：`ONCHAIN_FEE_NRC_PERCENT`（10%）
/// - 安全基金账户分成：`ONCHAIN_FEE_SAFETY_FUND_PERCENT`（10%）
pub struct OnchainFeeRouter<T, Currency, AuthorFinder, NrcProvider, SafetyFundProvider>(
    PhantomData<(T, Currency, AuthorFinder, NrcProvider, SafetyFundProvider)>,
);

/// 交易收费分类：
/// - VoteFlat: 投票 / 治理操作固定 1 元
/// - OnchainAmount: 链上资金交易，按交易金额 × 0.1%，最低 0.1 元
/// - OffchainFee: 链下清算手续费，手续费金额由清算模块执行，不进入链上分账
/// - Free: 系统调用 / 自动化调用免费
/// - Unknown: 未归入上述四类，直接拒绝，避免隐性漏收费
#[derive(Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum FeeChargeKind<Balance> {
    VoteFlat,
    OnchainAmount(Balance),
    OffchainFee(Balance),
    Free,
    Unknown,
}

/// 统一抽象：由 Runtime 提供"交易费用分类器"。
pub trait CallFeeKind<AccountId, Call, Balance> {
    /// 将具体交易显式归入五类费用模型。
    /// 这里故意不和 weight fee/length fee 绑定，避免 runtime 规则被默认手续费模型覆盖。
    fn fee_kind(who: &AccountId, call: &Call) -> FeeChargeKind<Balance>;
}

/// 可选扣费来源：
/// - None：沿用默认交易提交者扣费
/// - Some(account)：从指定账户扣费
pub trait CallFeePayer<AccountId, Call> {
    /// 返回代付账户；若为 None，则仍由交易提交者本人扣费。
    /// 该扩展点只负责"选择谁付款"，不改变手续费金额计算规则。
    fn fee_payer(who: &AccountId, call: &Call) -> Option<AccountId>;
}

impl<AccountId, Call> CallFeePayer<AccountId, Call> for () {
    fn fee_payer(_who: &AccountId, _call: &Call) -> Option<AccountId> {
        None
    }
}

/// 统一抽象：由 Runtime 注入国储会收款账户来源。
pub trait NrcAccountProvider<AccountId> {
    /// 提供国储会收款账户。
    /// 返回 None 时，NRC 份额按安全退化策略直接销毁。
    fn nrc_account() -> Option<AccountId>;
}

impl<AccountId> NrcAccountProvider<AccountId> for () {
    fn nrc_account() -> Option<AccountId> {
        None
    }
}

/// 统一抽象：由 Runtime 注入安全基金收款账户来源。
pub trait SafetyFundAccountProvider<AccountId> {
    /// 提供安全基金账户。
    /// 安全基金账户属于制度常量，使用 provider 可避免在分账热路径反复 decode。
    fn safety_fund_account() -> AccountId;
}

/// 统一手续费收取适配器：
/// - 投票 / 治理操作固定 `VOTE_FLAT_FEE`
/// - 链上资金交易按 `ONCHAIN_FEE_RATE` 和 `ONCHAIN_MIN_FEE` 计算
/// - 链下清算手续费由清算模块执行，不重复进入链上手续费分账
/// - 免费调用不扣基础费
pub struct OnchainChargeAdapter<Currency, Router, FeeKindExtractor, FeePayerExtractor>(
    PhantomData<(Currency, Router, FeeKindExtractor, FeePayerExtractor)>,
);

impl<T, Currency, Router, FeeKindExtractor, FeePayerExtractor> OnChargeTransaction<T>
    for OnchainChargeAdapter<Currency, Router, FeeKindExtractor, FeePayerExtractor>
where
    T: TxPaymentConfig + fullnode_issuance::Config + pallet::Config,
    Currency: Balanced<T::AccountId> + 'static,
    Router: OnUnbalanced<Credit<T::AccountId, Currency>>,
    FeeKindExtractor:
        CallFeeKind<T::AccountId, T::RuntimeCall, <Currency as Inspect<T::AccountId>>::Balance>,
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
        // 中文注释：这里完全忽略 pallet-transaction-payment 传入的 _fee_with_tip，
        // 改为执行本模块自定义的五类费用模型。
        let fee_with_tip =
            custom_fee_with_tip::<T, Currency, FeeKindExtractor>(who, call, dispatch_info, tip)?;
        if fee_with_tip.is_zero() {
            return Ok(None);
        }
        let payer = FeePayerExtractor::fee_payer(who, call).unwrap_or_else(|| who.clone());

        // 中文注释：扣费使用 Exact，避免"只扣到一部分也继续执行"。
        let credit = Currency::withdraw(
            &payer,
            fee_with_tip,
            Precision::Exact,
            Preservation::Preserve,
            Fortitude::Polite,
        )
        .map_err(|_| InvalidTransaction::Payment)?;

        // 中文注释：tip 会单独拆出来，但后续仍和基础费一起交给 Router，
        // 这样可以保留 tip 语义，同时复用统一分账路径。
        let (tip_credit, inclusion_fee) = credit.split(tip);

        // 发出链上手续费事件，供手机端 / 浏览器 / node 读取真实手续费。
        let base_fee: u128 = fee_with_tip.saturating_sub(tip).saturated_into();
        pallet::Pallet::<T>::deposit_event(pallet::Event::FeePaid {
            who: payer.clone(),
            fee: base_fee,
        });

        Ok(Some((inclusion_fee, tip_credit)))
    }

    fn can_withdraw_fee(
        who: &T::AccountId,
        call: &T::RuntimeCall,
        dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _fee_with_tip: Self::Balance,
        tip: Self::Balance,
    ) -> Result<(), TransactionValidityError> {
        // 中文注释：预检查与正式扣费保持同一套金额计算逻辑，
        // 否则容易出现"预检查能过、正式扣费失败"的行为偏差。
        let fee_with_tip =
            custom_fee_with_tip::<T, Currency, FeeKindExtractor>(who, call, dispatch_info, tip)?;
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
        // PROTOCOL: no post-dispatch refund.
        // 中文注释：本制度按五类费用模型固定收费，协议上明确"不做执行后退款"。
        // `_corrected_fee_with_tip` 和 `_tip` 仅来自 transaction-payment 标准接口，
        // 本实现只对 `withdraw_fee` 已扣出的 credit 做最终分账。
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

impl<T, Currency, Router, FeeKindExtractor, FeePayerExtractor> TxCreditHold<T>
    for OnchainChargeAdapter<Currency, Router, FeeKindExtractor, FeePayerExtractor>
where
    T: TxPaymentConfig,
    Currency: Balanced<T::AccountId> + 'static,
{
    type Credit = ();
}

impl<T, Currency, AuthorFinder, NrcProvider, SafetyFundProvider>
    OnUnbalanced<Credit<T::AccountId, Currency>>
    for OnchainFeeRouter<T, Currency, AuthorFinder, NrcProvider, SafetyFundProvider>
where
    T: frame_system::Config + fullnode_issuance::Config + pallet::Config,
    Currency: Balanced<T::AccountId>,
    AuthorFinder: FindAuthor<T::AccountId>,
    NrcProvider: NrcAccountProvider<T::AccountId>,
    SafetyFundProvider: SafetyFundAccountProvider<T::AccountId>,
{
    fn on_nonzero_unbalanced(amount: Credit<T::AccountId, Currency>) {
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        debug_assert_eq!(
            total_percent, EXPECTED_FEE_PERCENT_TOTAL,
            "fee distribution constants must sum to 100"
        );

        // 中文注释：制度常量异常时，直接全部销毁，避免错误分配。
        if total_percent != EXPECTED_FEE_PERCENT_TOTAL {
            log::error!(
                target: "runtime::onchain_transaction",
                "fee distribution percents must sum to {}; got fullnode={}, nrc={}, safety_fund={}, total={}",
                EXPECTED_FEE_PERCENT_TOTAL,
                fullnode_percent,
                nrc_percent,
                safety_fund_percent,
                total_percent
            );
            return;
        }

        // 中文注释：先切出全节点份额，再把剩余部分在 NRC 和安全基金之间二次切分，
        // 可以避免三项分账时因为整数除法带来更复杂的舍入误差。
        let (fullnode_credit, remainder) = amount.ration(
            fullnode_percent,
            total_percent.saturating_sub(fullnode_percent),
        );
        let (nrc_credit, safety_fund_credit) = remainder.ration(nrc_percent, safety_fund_percent);

        // 中文注释：手续费全节点分成只发给"当前区块作者对应绑定钱包"；未绑定则不分配（自动销毁）。
        let digest = <frame_system::Pallet<T>>::digest();
        let pre_runtime_digests = digest.logs().iter().filter_map(|d| d.as_pre_runtime());
        match AuthorFinder::find_author(pre_runtime_digests) {
            Some(miner) => {
                if let Some(wallet) = fullnode_issuance::RewardWalletByMiner::<T>::get(&miner) {
                    if let Err(remaining) = Currency::resolve(&wallet, fullnode_credit) {
                        let burnt_amount = remaining.peek().saturated_into::<u128>();
                        log::warn!(
                            target: "runtime::onchain_transaction",
                            "burn fullnode fee share: failed to resolve reward wallet credit: {:?}",
                            remaining.peek()
                        );
                        emit_fee_share_burn::<T>(
                            pallet::BurnReason::FullnodeResolveFailed,
                            burnt_amount,
                        );
                    }
                } else {
                    let burnt_amount = fullnode_credit.peek().saturated_into::<u128>();
                    log::warn!(
                        target: "runtime::onchain_transaction",
                        "burn fullnode fee share: author found but reward wallet not bound"
                    );
                    emit_fee_share_burn::<T>(pallet::BurnReason::WalletUnbound, burnt_amount);
                    drop(fullnode_credit);
                }
            }
            None => {
                let burnt_amount = fullnode_credit.peek().saturated_into::<u128>();
                log::warn!(
                    target: "runtime::onchain_transaction",
                    "burn fullnode fee share: block author not found"
                );
                emit_fee_share_burn::<T>(pallet::BurnReason::AuthorMissing, burnt_amount);
                drop(fullnode_credit);
            }
        }

        // 中文注释：国储会手续费账户分成。
        if let Some(nrc_fee_account) = NrcProvider::nrc_account() {
            if let Err(remaining) = Currency::resolve(&nrc_fee_account, nrc_credit) {
                let burnt_amount = remaining.peek().saturated_into::<u128>();
                log::warn!(
                    target: "runtime::onchain_transaction",
                    "nrc fee share: failed to resolve nrc fee account credit: {:?}",
                    remaining.peek()
                );
                emit_fee_share_burn::<T>(pallet::BurnReason::NrcResolveFailed, burnt_amount);
            }
        } else {
            let burnt_amount = nrc_credit.peek().saturated_into::<u128>();
            log::warn!(
                target: "runtime::onchain_transaction",
                "nrc fee share: nrc fee account not configured"
            );
            emit_fee_share_burn::<T>(pallet::BurnReason::NrcMissing, burnt_amount);
            drop(nrc_credit);
        }

        // 中文注释：安全基金账户由 runtime provider 注入，避免每笔手续费分账重复 decode 常量地址。
        let safety_fund_account = SafetyFundProvider::safety_fund_account();
        if let Err(remaining) = Currency::resolve(&safety_fund_account, safety_fund_credit) {
            let burnt_amount = remaining.peek().saturated_into::<u128>();
            log::warn!(
                target: "runtime::onchain_transaction",
                "safety fund fee share: failed to resolve credit: {:?}",
                remaining.peek()
            );
            emit_fee_share_burn::<T>(pallet::BurnReason::SafetyFundResolveFailed, burnt_amount);
        }
    }
}

fn emit_fee_share_burn<T: pallet::Config>(reason: pallet::BurnReason, amount: u128) {
    if amount == 0 {
        return;
    }
    pallet::Pallet::<T>::deposit_event(pallet::Event::FeeShareBurnt { reason, amount });
}

fn custom_fee_with_tip<T, Currency, FeeKindExtractor>(
    who: &T::AccountId,
    call: &T::RuntimeCall,
    _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
    tip: <Currency as Inspect<T::AccountId>>::Balance,
) -> Result<<Currency as Inspect<T::AccountId>>::Balance, TransactionValidityError>
where
    T: TxPaymentConfig,
    Currency: Balanced<T::AccountId>,
    FeeKindExtractor:
        CallFeeKind<T::AccountId, T::RuntimeCall, <Currency as Inspect<T::AccountId>>::Balance>,
{
    // 中文注释：Runtime 必须把每个调用显式归入五类费用模型；
    // Unknown 直接拒绝，防止新增交易绕过收费制度。
    let base_fee_u128 = match FeeKindExtractor::fee_kind(who, call) {
        FeeChargeKind::VoteFlat => primitives::fee_policy::VOTE_FLAT_FEE,
        FeeChargeKind::OnchainAmount(amount) => {
            // 中文注释：链上资金交易才按金额套 0.1% 费率。
            let amount_u128: u128 = amount.saturated_into();
            calculate_onchain_fee(amount_u128)
        }
        FeeChargeKind::OffchainFee(_fee) => {
            // 中文注释：链下清算手续费已经在 offchain-transaction 结算执行时转账，
            // 这里不再进入链上手续费 80/10/10 分账，避免重复扣费和错分账。
            0
        }
        FeeChargeKind::Free => 0,
        FeeChargeKind::Unknown => return Err(InvalidTransaction::Call.into()),
    };
    let base_fee: <Currency as Inspect<T::AccountId>>::Balance = base_fee_u128.saturated_into();
    Ok(base_fee.saturating_add(tip))
}

/// 按交易金额计算链上手续费（对外公开，供其他 pallet 复用）。
///
/// 公式：`max(amount × ONCHAIN_FEE_RATE, ONCHAIN_MIN_FEE)`
/// 返回值单位为"分"。
pub fn calculate_onchain_fee(amount: u128) -> u128 {
    let by_rate = mul_perbill_round(amount, primitives::fee_policy::ONCHAIN_FEE_RATE);
    by_rate.max(primitives::fee_policy::ONCHAIN_MIN_FEE)
}

fn mul_perbill_round(amount: u128, rate: sp_runtime::Perbill) -> u128 {
    // 中文注释：链上精度为"分"，这里做四舍五入到分。
    const PERBILL_DENOMINATOR: u128 = 1_000_000_000;
    let parts: u128 = rate.deconstruct() as u128;
    let whole = amount / PERBILL_DENOMINATOR;
    let remainder = amount % PERBILL_DENOMINATOR;

    // 中文注释：先拆成"整分量"和"小数尾量"分别计算，避免 `amount * parts`
    // 在极大金额下先溢出再饱和，导致费率结果被错误压扁。
    // 中文注释：按 Perbill 约束 parts 不超过分母，因此 whole * parts <= amount；
    // 这里仍用饱和乘法防御未来改动破坏该约束。
    let whole_component = whole.saturating_mul(parts);
    let remainder_component =
        (remainder * parts).saturating_add(PERBILL_DENOMINATOR / 2) / PERBILL_DENOMINATOR;
    whole_component.saturating_add(remainder_component)
}

#[cfg(test)]
mod tests;
