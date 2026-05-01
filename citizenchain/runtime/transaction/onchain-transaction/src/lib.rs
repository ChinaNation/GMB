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
    let total_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT
        .saturating_add(primitives::core_const::ONCHAIN_FEE_NRC_PERCENT)
        .saturating_add(primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT);
    assert!(
        total_percent == EXPECTED_FEE_PERCENT_TOTAL,
        "fee distribution percents must sum to 100"
    );
    assert!(
        primitives::core_const::ONCHAIN_MIN_FEE > 0,
        "ONCHAIN_MIN_FEE must be positive"
    );
    assert!(
        primitives::core_const::ONCHAIN_FEE_RATE.deconstruct() > 0,
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

/// 金额提取分类结果：
/// - Amount: 确认是"有金额交易"，并返回金额
/// - NoAmount: 确认是"无金额交易"
/// - Unknown: 无法确认（按制度应拒绝，避免漏提取）
pub enum AmountExtractResult<Balance> {
    Amount(Balance),
    NoAmount,
    Unknown,
}

/// 统一抽象：由 Runtime 提供"交易金额提取器"。
pub trait CallAmount<AccountId, Call, Balance> {
    /// 从具体交易中抽取"制度定义的交易金额"。
    /// 这里故意不和 weight fee/length fee 绑定，避免 runtime 规则被默认手续费模型覆盖。
    fn amount(who: &AccountId, call: &Call) -> AmountExtractResult<Balance>;
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

/// 链上手续费收取适配器：
/// - 手续费按交易金额 `ONCHAIN_FEE_RATE` 计算
/// - 单笔最低 `ONCHAIN_MIN_FEE`
/// - 具体分配交给 `OnchainFeeRouter`
pub struct OnchainChargeAdapter<Currency, Router, AmountExtractor, FeePayerExtractor>(
    PhantomData<(Currency, Router, AmountExtractor, FeePayerExtractor)>,
);

impl<T, Currency, Router, AmountExtractor, FeePayerExtractor> OnChargeTransaction<T>
    for OnchainChargeAdapter<Currency, Router, AmountExtractor, FeePayerExtractor>
where
    T: TxPaymentConfig + fullnode_issuance::Config + pallet::Config,
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
        // 中文注释：这里完全忽略 pallet-transaction-payment 传入的 _fee_with_tip，
        // 改为执行本模块自定义的"按业务金额收费"规则。
        let fee_with_tip =
            custom_fee_with_tip::<T, Currency, AmountExtractor>(who, call, dispatch_info, tip)?;
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
        // PROTOCOL: no post-dispatch refund.
        // 中文注释：本制度按交易金额固定收费，协议上明确"不做执行后退款"。
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

impl<T, Currency, Router, AmountExtractor, FeePayerExtractor> TxCreditHold<T>
    for OnchainChargeAdapter<Currency, Router, AmountExtractor, FeePayerExtractor>
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
        let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT;
        let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT;
        let safety_fund_percent = primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT;
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
        AmountExtractResult::Unknown => return Err(InvalidTransaction::Call.into()),
    };
    // 中文注释：统一先转成 u128 做费率计算，避免不同 Balance 类型下重复实现乘法与舍入逻辑。
    let amount_u128: u128 = amount.saturated_into();
    let base_fee: <Currency as Inspect<T::AccountId>>::Balance =
        calculate_onchain_fee(amount_u128).saturated_into();
    Ok(base_fee.saturating_add(tip))
}

/// 按交易金额计算链上手续费（对外公开，供其他 pallet 复用）。
///
/// 公式：`max(amount × ONCHAIN_FEE_RATE, ONCHAIN_MIN_FEE)`
/// 返回值单位为"分"。
pub fn calculate_onchain_fee(amount: u128) -> u128 {
    let by_rate = mul_perbill_round(amount, primitives::core_const::ONCHAIN_FEE_RATE);
    by_rate.max(primitives::core_const::ONCHAIN_MIN_FEE)
}

fn mul_perbill_round(amount: u128, rate: sp_runtime::Perbill) -> u128 {
    // 中文注释：链上精度为"分"，这里做四舍五入到分。
    const PERBILL_DENOMINATOR: u128 = 1_000_000_000;
    let parts: u128 = rate.deconstruct() as u128;
    let whole = amount / PERBILL_DENOMINATOR;
    let remainder = amount % PERBILL_DENOMINATOR;

    // 中文注释：先拆成"整分量"和"小数尾量"分别计算，避免 `amount * parts`
    // 在极大金额下先溢出再饱和，导致费率结果被错误压扁。
    let whole_component = whole * parts;
    let remainder_component =
        (remainder * parts).saturating_add(PERBILL_DENOMINATOR / 2) / PERBILL_DENOMINATOR;
    whole_component.saturating_add(remainder_component)
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        assert_ok, derive_impl,
        dispatch::GetDispatchInfo,
        parameter_types,
        traits::{Currency as _, VariantCountOf},
        weights::ConstantMultiplier,
    };
    use frame_system as system;
    use pallet_transaction_payment::OnChargeTransaction;
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage, Perbill};
    use std::{cell::RefCell, thread_local};

    type Block = frame_system::mocking::MockBlockU32<Test>;
    type Balance = u128;

    thread_local! {
        static MOCK_AUTHOR: RefCell<Option<AccountId32>> = const { RefCell::new(None) };
    }

    #[frame_support::runtime]
    mod runtime {
        #[runtime::runtime]
        #[runtime::derive(
            RuntimeCall,
            RuntimeEvent,
            RuntimeError,
            RuntimeOrigin,
            RuntimeFreezeReason,
            RuntimeHoldReason,
            RuntimeSlashReason,
            RuntimeLockId,
            RuntimeTask,
            RuntimeViewFunction
        )]
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;
        #[runtime::pallet_index(1)]
        pub type Balances = pallet_balances;
        #[runtime::pallet_index(2)]
        pub type TransactionPayment = pallet_transaction_payment;
        #[runtime::pallet_index(3)]
        pub type FullnodeIssuance = fullnode_issuance;
        #[runtime::pallet_index(4)]
        pub type OnchainTransaction = crate::pallet;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Nonce = u64;
    }

    parameter_types! {
        pub static TestExistentialDeposit: Balance = 1;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = frame_support::traits::ConstU32<0>;
        type MaxReserves = frame_support::traits::ConstU32<0>;
        type ReserveIdentifier = [u8; 8];
        type Balance = Balance;
        type RuntimeEvent = RuntimeEvent;
        type DustRemoval = ();
        type ExistentialDeposit = TestExistentialDeposit;
        type AccountStore = System;
        type WeightInfo = ();
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
    }

    impl pallet_transaction_payment::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
        type WeightToFee = ConstantMultiplier<Balance, frame_support::traits::ConstU128<1>>;
        type LengthToFee = ConstantMultiplier<Balance, frame_support::traits::ConstU128<1>>;
        type FeeMultiplierUpdate = ();
        type OperationalFeeMultiplier = frame_support::traits::ConstU8<1>;
        type WeightInfo = ();
    }

    pub struct MockFindAuthor;
    impl FindAuthor<AccountId32> for MockFindAuthor {
        fn find_author<'a, I>(_digests: I) -> Option<AccountId32>
        where
            I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
        {
            MOCK_AUTHOR.with(|v| v.borrow().clone())
        }
    }

    impl fullnode_issuance::Config for Test {
        type Currency = Balances;
        type FindAuthor = MockFindAuthor;
        type WeightInfo = ();
    }

    impl crate::pallet::Config for Test {}

    struct MockNrcAccountProvider;
    impl NrcAccountProvider<AccountId32> for MockNrcAccountProvider {
        fn nrc_account() -> Option<AccountId32> {
            Some(AccountId32::new(
                primitives::china::china_cb::CHINA_CB[0].main_address,
            ))
        }
    }

    struct MockNrcAccountProviderNone;
    impl NrcAccountProvider<AccountId32> for MockNrcAccountProviderNone {
        fn nrc_account() -> Option<AccountId32> {
            None
        }
    }

    struct MockSafetyFundAccountProvider;
    impl SafetyFundAccountProvider<AccountId32> for MockSafetyFundAccountProvider {
        fn safety_fund_account() -> AccountId32 {
            AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS)
        }
    }

    fn account(n: u8) -> AccountId32 {
        AccountId32::new([n; 32])
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        TestExistentialDeposit::set(1);
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("system genesis build should succeed");
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(account(1), 1_000), (account(2), 1_000), (account(3), 3)],
            dev_accounts: None,
        }
        .assimilate_storage(&mut storage)
        .expect("balances genesis build should succeed");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    struct AmountExtractorAmount;
    impl CallAmount<AccountId32, RuntimeCall, Balance> for AmountExtractorAmount {
        fn amount(_who: &AccountId32, _call: &RuntimeCall) -> AmountExtractResult<Balance> {
            AmountExtractResult::Amount(50_000)
        }
    }

    struct AmountExtractorNoAmount;
    impl CallAmount<AccountId32, RuntimeCall, Balance> for AmountExtractorNoAmount {
        fn amount(_who: &AccountId32, _call: &RuntimeCall) -> AmountExtractResult<Balance> {
            AmountExtractResult::NoAmount
        }
    }

    struct AmountExtractorUnknown;
    impl CallAmount<AccountId32, RuntimeCall, Balance> for AmountExtractorUnknown {
        fn amount(_who: &AccountId32, _call: &RuntimeCall) -> AmountExtractResult<Balance> {
            AmountExtractResult::Unknown
        }
    }

    struct AmountExtractorTiny;
    impl CallAmount<AccountId32, RuntimeCall, Balance> for AmountExtractorTiny {
        fn amount(_who: &AccountId32, _call: &RuntimeCall) -> AmountExtractResult<Balance> {
            AmountExtractResult::Amount(1)
        }
    }

    struct FeePayerAsAccount2;
    impl CallFeePayer<AccountId32, RuntimeCall> for FeePayerAsAccount2 {
        fn fee_payer(_who: &AccountId32, _call: &RuntimeCall) -> Option<AccountId32> {
            Some(account(2))
        }
    }

    fn sample_call() -> RuntimeCall {
        RuntimeCall::System(frame_system::Call::remark {
            remark: vec![1, 2, 3],
        })
    }

    fn has_fee_share_burn_event(reason: pallet::BurnReason, amount: u128) -> bool {
        System::events().iter().any(|r| {
            matches!(
                &r.event,
                RuntimeEvent::OnchainTransaction(pallet::Event::FeeShareBurnt {
                    reason: event_reason,
                    amount: event_amount,
                }) if *event_reason == reason && *event_amount == amount
            )
        })
    }

    fn has_fee_paid_event() -> bool {
        System::events().iter().any(|r| {
            matches!(
                r.event,
                RuntimeEvent::OnchainTransaction(pallet::Event::FeePaid { .. })
            )
        })
    }

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

    #[test]
    fn mul_perbill_round_half_up_works() {
        // 500 * 0.1% = 0.5 分，按四舍五入应为 1 分
        assert_eq!(mul_perbill_round(500, Perbill::from_parts(1_000_000)), 1);
        // 499 * 0.1% = 0.499 分，按四舍五入应为 0 分
        assert_eq!(mul_perbill_round(499, Perbill::from_parts(1_000_000)), 0);
    }

    #[test]
    fn mul_perbill_round_handles_u128_max_without_saturating_distortion() {
        assert_eq!(
            mul_perbill_round(u128::MAX, Perbill::from_percent(100)),
            u128::MAX
        );
    }

    #[test]
    fn custom_fee_with_tip_handles_all_extract_results() {
        new_test_ext().execute_with(|| {
            let who = account(1);
            let call = sample_call();
            let info = call.get_dispatch_info();

            // Amount：50_000 * 0.1% = 50 分，+ tip(3) => 53 分
            let fee_amount =
                custom_fee_with_tip::<Test, Balances, AmountExtractorAmount>(&who, &call, &info, 3)
                    .expect("amount fee must be computable");
            assert_eq!(fee_amount, 53);

            // NoAmount：不收基础费，仅返回 tip
            let fee_no_amount = custom_fee_with_tip::<Test, Balances, AmountExtractorNoAmount>(
                &who, &call, &info, 7,
            )
            .expect("no-amount call must only charge tip");
            assert_eq!(fee_no_amount, 7);

            // Unknown：拒绝交易，避免漏提取手续费
            let unknown_err = custom_fee_with_tip::<Test, Balances, AmountExtractorUnknown>(
                &who, &call, &info, 0,
            )
            .expect_err("unknown extract result should be rejected");
            assert_eq!(unknown_err, InvalidTransaction::Call.into());
        });
    }

    #[test]
    fn withdraw_and_can_withdraw_use_default_payer_and_min_fee() {
        type Adapter = OnchainChargeAdapter<Balances, (), AmountExtractorTiny, ()>;

        new_test_ext().execute_with(|| {
            let who = account(1);
            let call = sample_call();
            let info = call.get_dispatch_info();

            assert_ok!(<Adapter as OnChargeTransaction<Test>>::can_withdraw_fee(
                &who, &call, &info, 0, 2
            ));

            let liq =
                <Adapter as OnChargeTransaction<Test>>::withdraw_fee(&who, &call, &info, 0, 2)
                    .expect("withdraw should succeed")
                    .expect("non-zero fee must return liquidity info");

            assert_eq!(Balances::free_balance(who), 988);
            assert_eq!(liq.0.peek(), 10);
            assert_eq!(liq.1.peek(), 2);
        });
    }

    #[test]
    fn withdraw_uses_custom_fee_payer() {
        type Adapter = OnchainChargeAdapter<Balances, (), AmountExtractorTiny, FeePayerAsAccount2>;

        new_test_ext().execute_with(|| {
            let who = account(1);
            let payer = account(2);
            let call = sample_call();
            let info = call.get_dispatch_info();

            let _ = <Adapter as OnChargeTransaction<Test>>::withdraw_fee(&who, &call, &info, 0, 0)
                .expect("withdraw should succeed")
                .expect("non-zero fee must return liquidity info");

            assert_eq!(Balances::free_balance(who), 1_000);
            assert_eq!(Balances::free_balance(payer), 990);
        });
    }

    #[test]
    fn withdraw_no_amount_without_tip_returns_none_and_no_fee_paid_event() {
        type Adapter = OnchainChargeAdapter<Balances, (), AmountExtractorNoAmount, ()>;

        new_test_ext().execute_with(|| {
            let who = account(1);
            let call = sample_call();
            let info = call.get_dispatch_info();
            let issuance_before = Balances::total_issuance();

            let liquidity =
                <Adapter as OnChargeTransaction<Test>>::withdraw_fee(&who, &call, &info, 0, 0)
                    .expect("zero-fee withdraw should succeed");

            assert!(liquidity.is_none());
            assert_eq!(Balances::free_balance(who), 1_000);
            assert_eq!(Balances::total_issuance(), issuance_before);
            assert!(!has_fee_paid_event());
        });
    }

    #[test]
    fn can_withdraw_and_withdraw_fail_when_insufficient_balance() {
        type Adapter = OnchainChargeAdapter<Balances, (), AmountExtractorTiny, ()>;

        new_test_ext().execute_with(|| {
            let poor = account(3);
            let call = sample_call();
            let info = call.get_dispatch_info();

            assert!(<Adapter as OnChargeTransaction<Test>>::can_withdraw_fee(
                &poor, &call, &info, 0, 0
            )
            .is_err());

            assert!(<Adapter as OnChargeTransaction<Test>>::withdraw_fee(
                &poor, &call, &info, 0, 0
            )
            .is_err());
        });
    }

    #[test]
    fn fee_router_distributes_to_bound_author_wallet_and_nrc_and_safety_fund() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let miner = account(9);
            let reward_wallet = account(8);
            let nrc = MockNrcAccountProvider::nrc_account().expect("nrc account must exist");
            let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);
            let issuance_before = Balances::total_issuance();
            let total_fee = 100u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let safety_fund_percent =
                primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(safety_fund_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(safety_fund_percent)
            };
            let expected_safety_fund = remainder.saturating_sub(expected_nrc);

            fullnode_issuance::RewardWalletByMiner::<Test>::insert(&miner, &reward_wallet);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner.clone()));

            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                100,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            OnchainFeeRouter::<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProvider,
                MockSafetyFundAccountProvider,
            >::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 900);
            assert_eq!(Balances::free_balance(&reward_wallet), expected_fullnode);
            assert_eq!(Balances::free_balance(&nrc), expected_nrc);
            assert_eq!(Balances::free_balance(&safety_fund), expected_safety_fund);
            // 所有手续费都已分配到各账户，无销毁。
            assert_eq!(Balances::total_issuance(), issuance_before);
        });
    }

    #[test]
    fn fee_router_burns_fullnode_share_when_author_not_bound() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let miner = account(7);
            let missing_wallet = account(6);
            let nrc = MockNrcAccountProvider::nrc_account().expect("nrc account must exist");
            let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);
            let issuance_before = Balances::total_issuance();
            let total_fee = 100u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let safety_fund_percent =
                primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(safety_fund_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(safety_fund_percent)
            };
            let expected_safety_fund = remainder.saturating_sub(expected_nrc);
            // 无作者钱包时：全节点分成销毁，NRC 和安全基金正常分配。
            let expected_burn = expected_fullnode;

            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner.clone()));
            assert_eq!(
                fullnode_issuance::RewardWalletByMiner::<Test>::get(&miner),
                None
            );

            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                100,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            OnchainFeeRouter::<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProvider,
                MockSafetyFundAccountProvider,
            >::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 900);
            assert_eq!(Balances::free_balance(missing_wallet), 0);
            assert_eq!(Balances::free_balance(&nrc), expected_nrc);
            assert_eq!(Balances::free_balance(&safety_fund), expected_safety_fund);
            assert_eq!(Balances::total_issuance(), issuance_before - expected_burn);
            assert!(has_fee_share_burn_event(
                pallet::BurnReason::WalletUnbound,
                expected_burn
            ));
        });
    }

    #[test]
    fn fee_router_burns_fullnode_share_when_author_not_found() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let nrc = MockNrcAccountProvider::nrc_account().expect("nrc account must exist");
            let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);
            let issuance_before = Balances::total_issuance();
            let total_fee = 100u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let safety_fund_percent =
                primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(safety_fund_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(safety_fund_percent)
            };
            let expected_safety_fund = remainder.saturating_sub(expected_nrc);
            // 无作者时：全节点分成销毁，NRC 和安全基金正常分配。
            let expected_burn = expected_fullnode;

            MOCK_AUTHOR.with(|v| *v.borrow_mut() = None);
            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                total_fee,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            OnchainFeeRouter::<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProvider,
                MockSafetyFundAccountProvider,
            >::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 900);
            assert_eq!(Balances::free_balance(&nrc), expected_nrc);
            assert_eq!(Balances::free_balance(&safety_fund), expected_safety_fund);
            assert_eq!(Balances::total_issuance(), issuance_before - expected_burn);
            assert!(has_fee_share_burn_event(
                pallet::BurnReason::AuthorMissing,
                expected_burn
            ));
        });
    }

    #[test]
    fn fee_router_burns_nrc_share_when_nrc_account_missing() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let miner = account(9);
            let reward_wallet = account(8);
            let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);
            let issuance_before = Balances::total_issuance();
            let total_fee = 100u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let safety_fund_percent =
                primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(safety_fund_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            // NRC 账户缺失时：NRC 份额的 nrc_credit 被 drop（销毁），安全基金正常分配。
            let expected_nrc_for_split = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(safety_fund_percent)
            };
            let expected_safety_fund = remainder.saturating_sub(expected_nrc_for_split);
            let expected_burn = expected_nrc_for_split;

            fullnode_issuance::RewardWalletByMiner::<Test>::insert(&miner, &reward_wallet);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner.clone()));
            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                total_fee,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            OnchainFeeRouter::<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProviderNone,
                MockSafetyFundAccountProvider,
            >::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 900);
            assert_eq!(Balances::free_balance(&reward_wallet), expected_fullnode);
            assert_eq!(Balances::free_balance(&safety_fund), expected_safety_fund);
            assert_eq!(Balances::total_issuance(), issuance_before - expected_burn);
            assert!(has_fee_share_burn_event(
                pallet::BurnReason::NrcMissing,
                expected_burn
            ));
        });
    }

    #[test]
    fn fee_router_burns_fullnode_share_when_reward_wallet_resolve_fails() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let miner = account(9);
            let reward_wallet = account(8);
            let nrc = MockNrcAccountProvider::nrc_account().expect("nrc account must exist");
            let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);
            let total_fee = 50u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let safety_fund_percent =
                primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(safety_fund_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = remainder.saturating_mul(nrc_percent)
                / nrc_percent.saturating_add(safety_fund_percent);
            let expected_safety_fund = remainder.saturating_sub(expected_nrc);

            TestExistentialDeposit::set(100);
            // 中文注释：只让全节点奖励钱包保持未创建状态，确保本用例命中 fullnode resolve 失败。
            let _ = Balances::deposit_creating(&nrc, 100);
            let _ = Balances::deposit_creating(&safety_fund, 100);
            let issuance_before = Balances::total_issuance();
            fullnode_issuance::RewardWalletByMiner::<Test>::insert(&miner, &reward_wallet);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner));
            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                total_fee,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            OnchainFeeRouter::<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProvider,
                MockSafetyFundAccountProvider,
            >::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 950);
            assert_eq!(Balances::free_balance(&reward_wallet), 0);
            assert_eq!(Balances::free_balance(&nrc), 100 + expected_nrc);
            assert_eq!(
                Balances::free_balance(&safety_fund),
                100 + expected_safety_fund
            );
            assert_eq!(
                Balances::total_issuance(),
                issuance_before - expected_fullnode
            );
            assert!(has_fee_share_burn_event(
                pallet::BurnReason::FullnodeResolveFailed,
                expected_fullnode
            ));
            TestExistentialDeposit::set(1);
        });
    }

    struct MockNrcAccountProviderResolveFail;
    impl NrcAccountProvider<AccountId32> for MockNrcAccountProviderResolveFail {
        fn nrc_account() -> Option<AccountId32> {
            Some(account(42))
        }
    }

    #[test]
    fn fee_router_burns_nrc_share_when_resolve_fails() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let miner = account(9);
            let reward_wallet = account(8);
            let nrc = MockNrcAccountProviderResolveFail::nrc_account()
                .expect("nrc account must exist for resolve failure test");
            let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);
            let total_fee = 500u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let safety_fund_percent =
                primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(safety_fund_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = remainder.saturating_mul(nrc_percent)
                / nrc_percent.saturating_add(safety_fund_percent);
            let expected_safety_fund = remainder.saturating_sub(expected_nrc);
            // NRC resolve 失败（ED 过高），NRC 份额销毁；安全基金正常分配。
            let expected_burn = expected_nrc;

            TestExistentialDeposit::set(100);
            // 中文注释：本用例只验证 NRC 新账户低于 ED 时被销毁；
            // 安全基金账户先置为已存在账户，避免同样低于 ED 的份额也被销毁。
            let safety_fund_initial = 100;
            let _ = Balances::deposit_creating(&safety_fund, safety_fund_initial);
            let issuance_before = Balances::total_issuance();
            fullnode_issuance::RewardWalletByMiner::<Test>::insert(&miner, &reward_wallet);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner));
            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                total_fee,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            OnchainFeeRouter::<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProviderResolveFail,
                MockSafetyFundAccountProvider,
            >::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 500);
            assert_eq!(Balances::free_balance(&reward_wallet), expected_fullnode);
            assert_eq!(Balances::free_balance(&nrc), 0);
            assert_eq!(
                Balances::free_balance(&safety_fund),
                safety_fund_initial + expected_safety_fund
            );
            assert_eq!(Balances::total_issuance(), issuance_before - expected_burn);
            assert!(has_fee_share_burn_event(
                pallet::BurnReason::NrcResolveFailed,
                expected_burn
            ));
            assert!(expected_nrc < 100, "nrc share should stay below high ED");
            TestExistentialDeposit::set(1);
        });
    }

    #[test]
    fn fee_router_burns_safety_fund_share_when_resolve_fails() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let miner = account(9);
            let reward_wallet = account(8);
            let nrc = MockNrcAccountProvider::nrc_account().expect("nrc account must exist");
            let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);
            let total_fee = 500u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let safety_fund_percent =
                primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(safety_fund_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = remainder.saturating_mul(nrc_percent)
                / nrc_percent.saturating_add(safety_fund_percent);
            let expected_safety_fund = remainder.saturating_sub(expected_nrc);

            TestExistentialDeposit::set(100);
            // 中文注释：全节点钱包与 NRC 账户先置为已存在账户，只让安全基金新账户低于 ED。
            let _ = Balances::deposit_creating(&reward_wallet, 100);
            let _ = Balances::deposit_creating(&nrc, 100);
            let issuance_before = Balances::total_issuance();
            fullnode_issuance::RewardWalletByMiner::<Test>::insert(&miner, &reward_wallet);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner));
            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                total_fee,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            OnchainFeeRouter::<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProvider,
                MockSafetyFundAccountProvider,
            >::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 500);
            assert_eq!(
                Balances::free_balance(&reward_wallet),
                100 + expected_fullnode
            );
            assert_eq!(Balances::free_balance(&nrc), 100 + expected_nrc);
            assert_eq!(Balances::free_balance(&safety_fund), 0);
            assert_eq!(
                Balances::total_issuance(),
                issuance_before - expected_safety_fund
            );
            assert!(has_fee_share_burn_event(
                pallet::BurnReason::SafetyFundResolveFailed,
                expected_safety_fund
            ));
            TestExistentialDeposit::set(1);
        });
    }

    #[test]
    fn correct_and_deposit_does_not_refund_overpayment() {
        type Adapter = OnchainChargeAdapter<
            Balances,
            OnchainFeeRouter<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProviderNone,
                MockSafetyFundAccountProvider,
            >,
            AmountExtractorAmount,
            (),
        >;

        new_test_ext().execute_with(|| {
            let who = account(1);
            let call = sample_call();
            let info = call.get_dispatch_info();
            let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);
            let issuance_before = Balances::total_issuance();
            let total_fee = 55u128; // base 50 + tip 5
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let safety_fund_percent =
                primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(safety_fund_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc_split = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(safety_fund_percent)
            };
            let expected_safety_fund = remainder.saturating_sub(expected_nrc_split);
            // 无作者 + 无 NRC 账户：全节点分成和 NRC 分成销毁，安全基金正常分配。
            let expected_burn = total_fee.saturating_sub(expected_safety_fund);

            MOCK_AUTHOR.with(|v| *v.borrow_mut() = None);
            let liquidity =
                <Adapter as OnChargeTransaction<Test>>::withdraw_fee(&who, &call, &info, 0, 5)
                    .expect("withdraw should succeed");
            assert_eq!(Balances::free_balance(&who), 945);

            assert_ok!(
                <Adapter as OnChargeTransaction<Test>>::correct_and_deposit_fee(
                    &who,
                    &info,
                    &Default::default(),
                    1, // pretend corrected fee is tiny; adapter intentionally ignores it
                    5,
                    liquidity,
                )
            );

            assert_eq!(Balances::free_balance(&who), 945);
            assert_eq!(Balances::free_balance(&safety_fund), expected_safety_fund);
            assert_eq!(Balances::total_issuance(), issuance_before - expected_burn);
        });
    }

    #[test]
    fn correct_and_deposit_fee_none_is_noop() {
        type Adapter = OnchainChargeAdapter<
            Balances,
            OnchainFeeRouter<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProvider,
                MockSafetyFundAccountProvider,
            >,
            AmountExtractorAmount,
            (),
        >;

        new_test_ext().execute_with(|| {
            let who = account(1);
            let call = sample_call();
            let info = call.get_dispatch_info();
            let issuance_before = Balances::total_issuance();
            let balance_before = Balances::free_balance(&who);

            assert_ok!(
                <Adapter as OnChargeTransaction<Test>>::correct_and_deposit_fee(
                    &who,
                    &info,
                    &Default::default(),
                    0,
                    0,
                    None,
                )
            );

            assert_eq!(Balances::free_balance(&who), balance_before);
            assert_eq!(Balances::total_issuance(), issuance_before);
            assert!(!has_fee_paid_event());
        });
    }

    #[test]
    fn tip_is_routed_with_fee_using_same_distribution() {
        type Adapter = OnchainChargeAdapter<
            Balances,
            OnchainFeeRouter<
                Test,
                Balances,
                MockFindAuthor,
                MockNrcAccountProvider,
                MockSafetyFundAccountProvider,
            >,
            AmountExtractorTiny,
            (),
        >;

        new_test_ext().execute_with(|| {
            let who = account(1);
            let call = sample_call();
            let info = call.get_dispatch_info();
            let miner = account(9);
            let reward_wallet = account(8);
            let nrc = MockNrcAccountProvider::nrc_account().expect("nrc account must exist");
            let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);

            fullnode_issuance::RewardWalletByMiner::<Test>::insert(&miner, &reward_wallet);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner));

            let total_fee = 15u128; // base 10 + tip 5
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let safety_fund_percent =
                primitives::core_const::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(safety_fund_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(safety_fund_percent)
            };
            let expected_safety_fund = remainder.saturating_sub(expected_nrc);

            let liquidity =
                <Adapter as OnChargeTransaction<Test>>::withdraw_fee(&who, &call, &info, 0, 5)
                    .expect("withdraw should succeed");
            assert_ok!(
                <Adapter as OnChargeTransaction<Test>>::correct_and_deposit_fee(
                    &who,
                    &info,
                    &Default::default(),
                    total_fee,
                    5,
                    liquidity,
                )
            );

            assert_eq!(Balances::free_balance(who), 985);
            assert_eq!(Balances::free_balance(&reward_wallet), expected_fullnode);
            assert_eq!(Balances::free_balance(&nrc), expected_nrc);
            assert_eq!(Balances::free_balance(&safety_fund), expected_safety_fund);
        });
    }
}
