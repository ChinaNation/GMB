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

const _: () = {
    assert!(
        primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT
            .saturating_add(primitives::core_const::ONCHAIN_FEE_NRC_PERCENT)
            .saturating_add(primitives::core_const::ONCHAIN_FEE_BLACKHOLE_PERCENT)
            > 0,
        "fee distribution percents must sum to positive"
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

/// 链上 PoW 交易手续费分配器（统一入口）：
/// - 全节点（绑定钱包）分成：`ONCHAIN_FEE_FULLNODE_PERCENT`
/// - 国储会分成：`ONCHAIN_FEE_NRC_PERCENT`
/// - 黑洞销毁：`ONCHAIN_FEE_BLACKHOLE_PERCENT`
pub struct PowOnchainFeeRouter<T, Currency, AuthorFinder, NrcAccountProvider>(
    PhantomData<(T, Currency, AuthorFinder, NrcAccountProvider)>,
);

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

/// 统一抽象：由 Runtime 注入国储会收款账户来源。
pub trait NrcAccountProvider<AccountId> {
    fn nrc_account() -> Option<AccountId>;
}

impl<AccountId> NrcAccountProvider<AccountId> for () {
    fn nrc_account() -> Option<AccountId> {
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
        // 中文注释：本制度按交易金额固定收费，协议上明确“不做执行后退款”；
        // 因此 corrected_fee_with_tip 在此实现中被有意忽略。
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

impl<T, Currency, AuthorFinder, NrcProvider> OnUnbalanced<Credit<T::AccountId, Currency>>
    for PowOnchainFeeRouter<T, Currency, AuthorFinder, NrcProvider>
where
    T: frame_system::Config + fullnode_pow_reward::Config,
    Currency: Balanced<T::AccountId>,
    AuthorFinder: FindAuthor<T::AccountId>,
    NrcProvider: NrcAccountProvider<T::AccountId>,
{
    fn on_nonzero_unbalanced(amount: Credit<T::AccountId, Currency>) {
        let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT;
        let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT;
        let blackhole_percent = primitives::core_const::ONCHAIN_FEE_BLACKHOLE_PERCENT;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(blackhole_percent);
        debug_assert_eq!(
            nrc_percent.saturating_add(blackhole_percent),
            total_percent.saturating_sub(fullnode_percent),
            "fee distribution constants must sum correctly"
        );

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
        match AuthorFinder::find_author(pre_runtime_digests) {
            Some(miner) => {
                if let Some(wallet) = fullnode_pow_reward::RewardWalletByMiner::<T>::get(&miner) {
                    if let Err(remaining) = Currency::resolve(&wallet, fullnode_credit) {
                        log::warn!(
                            target: "runtime::onchain_transaction_pow",
                            "burn fullnode fee share: failed to resolve reward wallet credit: {:?}",
                            remaining.peek()
                        );
                    }
                } else {
                    log::warn!(
                        target: "runtime::onchain_transaction_pow",
                        "burn fullnode fee share: author found but reward wallet not bound"
                    );
                    drop(fullnode_credit);
                }
            }
            None => {
                log::warn!(
                    target: "runtime::onchain_transaction_pow",
                    "burn fullnode fee share: block author not found"
                );
                drop(fullnode_credit);
            }
        }

        // 中文注释：国储会分成发到 CHINA_CB[0] 对应交易地址；解析失败则自动销毁。
        if let Some(nrc_account) = NrcProvider::nrc_account() {
            if let Err(remaining) = Currency::resolve(&nrc_account, nrc_credit) {
                log::warn!(
                    target: "runtime::onchain_transaction_pow",
                    "burn nrc fee share: failed to resolve nrc account credit: {:?}",
                    remaining.peek()
                );
            }
        } else {
            log::warn!(
                target: "runtime::onchain_transaction_pow",
                "burn nrc fee share: nrc account decode failed"
            );
        }
        // 中文注释：黑洞分成改为“直接销毁”（不入任何地址），总发行量同步减少。
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
        AmountExtractResult::Unknown => return Err(InvalidTransaction::Call.into()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        assert_ok,
        derive_impl,
        dispatch::GetDispatchInfo,
        traits::VariantCountOf,
        weights::ConstantMultiplier,
    };
    use frame_system as system;
    use pallet_transaction_payment::OnChargeTransaction;
    use sp_runtime::{
        traits::IdentityLookup,
        AccountId32, BuildStorage, Perbill,
    };
    use std::{cell::RefCell, thread_local};

    type Block = frame_system::mocking::MockBlock<Test>;
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
        pub type FullnodePowReward = fullnode_pow_reward;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Nonce = u64;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = frame_support::traits::ConstU32<0>;
        type MaxReserves = frame_support::traits::ConstU32<0>;
        type ReserveIdentifier = [u8; 8];
        type Balance = Balance;
        type RuntimeEvent = RuntimeEvent;
        type DustRemoval = ();
        type ExistentialDeposit = frame_support::traits::ConstU128<1>;
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

    impl fullnode_pow_reward::Config for Test {
        type Currency = Balances;
        type FindAuthor = MockFindAuthor;
    }

    struct MockNrcAccountProvider;
    impl NrcAccountProvider<AccountId32> for MockNrcAccountProvider {
        fn nrc_account() -> Option<AccountId32> {
            Some(AccountId32::new(
                primitives::china::china_cb::CHINA_CB[0].duoqian_address,
            ))
        }
    }

    struct MockNrcAccountProviderNone;
    impl NrcAccountProvider<AccountId32> for MockNrcAccountProviderNone {
        fn nrc_account() -> Option<AccountId32> {
            None
        }
    }

    fn account(n: u8) -> AccountId32 {
        AccountId32::new([n; 32])
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("system genesis build should succeed");
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(account(1), 1_000), (account(2), 1_000), (account(3), 3)],
            dev_accounts: None,
        }
        .assimilate_storage(&mut storage)
        .expect("balances genesis build should succeed");
        sp_io::TestExternalities::new(storage)
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
        RuntimeCall::System(frame_system::Call::remark { remark: vec![1, 2, 3] })
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
    fn custom_fee_with_tip_handles_all_extract_results() {
        new_test_ext().execute_with(|| {
            let who = account(1);
            let call = sample_call();
            let info = call.get_dispatch_info();

            // Amount：50_000 * 0.1% = 50 分，+ tip(3) => 53 分
            let fee_amount = custom_fee_with_tip::<Test, Balances, AmountExtractorAmount>(
                &who, &call, &info, 3,
            )
            .expect("amount fee must be computable");
            assert_eq!(fee_amount, 53);

            // NoAmount：不收基础费，仅返回 tip
            let fee_no_amount = custom_fee_with_tip::<Test, Balances, AmountExtractorNoAmount>(
                &who, &call, &info, 7,
            )
            .expect("no-amount call must only charge tip");
            assert_eq!(fee_no_amount, 7);

            // Unknown：拒绝交易，避免漏提取手续费
            let unknown_err =
                custom_fee_with_tip::<Test, Balances, AmountExtractorUnknown>(&who, &call, &info, 0)
                    .expect_err("unknown extract result should be rejected");
            assert_eq!(
                unknown_err,
                InvalidTransaction::Call.into()
            );
        });
    }

    #[test]
    fn withdraw_and_can_withdraw_use_default_payer_and_min_fee() {
        type Adapter = PowOnchainChargeAdapter<Balances, (), AmountExtractorTiny, ()>;

        new_test_ext().execute_with(|| {
            let who = account(1);
            let call = sample_call();
            let info = call.get_dispatch_info();

            assert_ok!(<Adapter as OnChargeTransaction<Test>>::can_withdraw_fee(
                &who, &call, &info, 0, 2
            ));

            let liq = <Adapter as OnChargeTransaction<Test>>::withdraw_fee(&who, &call, &info, 0, 2)
                .expect("withdraw should succeed")
                .expect("non-zero fee must return liquidity info");

            assert_eq!(Balances::free_balance(who), 988);
            assert_eq!(liq.0.peek(), 10);
            assert_eq!(liq.1.peek(), 2);
        });
    }

    #[test]
    fn withdraw_uses_custom_fee_payer() {
        type Adapter =
            PowOnchainChargeAdapter<Balances, (), AmountExtractorTiny, FeePayerAsAccount2>;

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
    fn can_withdraw_and_withdraw_fail_when_insufficient_balance() {
        type Adapter = PowOnchainChargeAdapter<Balances, (), AmountExtractorTiny, ()>;

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
    fn fee_router_distributes_to_bound_author_wallet_and_nrc_then_burns_remainder() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let miner = account(9);
            let reward_wallet = account(8);
            let nrc = MockNrcAccountProvider::nrc_account().expect("nrc account must exist");
            let issuance_before = Balances::total_issuance();
            let total_fee = 100u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let blackhole_percent = primitives::core_const::ONCHAIN_FEE_BLACKHOLE_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(blackhole_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = if nrc_percent.saturating_add(blackhole_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(blackhole_percent)
            };
            let expected_burn = total_fee
                .saturating_sub(expected_fullnode)
                .saturating_sub(expected_nrc);

            fullnode_pow_reward::RewardWalletByMiner::<Test>::insert(&miner, &reward_wallet);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner.clone()));

            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                100,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            PowOnchainFeeRouter::<Test, Balances, MockFindAuthor, MockNrcAccountProvider>::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 900);
            assert_eq!(Balances::free_balance(reward_wallet), expected_fullnode);
            assert_eq!(Balances::free_balance(nrc), expected_nrc);
            assert_eq!(Balances::total_issuance(), issuance_before - expected_burn);
        });
    }

    #[test]
    fn fee_router_burns_fullnode_share_when_author_not_bound() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let miner = account(7);
            let missing_wallet = account(6);
            let nrc = MockNrcAccountProvider::nrc_account().expect("nrc account must exist");
            let issuance_before = Balances::total_issuance();
            let total_fee = 100u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let blackhole_percent = primitives::core_const::ONCHAIN_FEE_BLACKHOLE_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(blackhole_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = if nrc_percent.saturating_add(blackhole_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(blackhole_percent)
            };
            let expected_burn = total_fee
                .saturating_sub(expected_nrc);

            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner.clone()));
            assert_eq!(fullnode_pow_reward::RewardWalletByMiner::<Test>::get(&miner), None);

            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                100,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            PowOnchainFeeRouter::<Test, Balances, MockFindAuthor, MockNrcAccountProvider>::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 900);
            assert_eq!(Balances::free_balance(missing_wallet), 0);
            assert_eq!(Balances::free_balance(nrc), expected_nrc);
            // 无作者钱包时：全节点分成+黑洞分成均销毁。
            assert_eq!(Balances::total_issuance(), issuance_before - expected_burn);
        });
    }

    #[test]
    fn fee_router_burns_fullnode_share_when_author_not_found() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let nrc = MockNrcAccountProvider::nrc_account().expect("nrc account must exist");
            let issuance_before = Balances::total_issuance();
            let total_fee = 100u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let blackhole_percent = primitives::core_const::ONCHAIN_FEE_BLACKHOLE_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(blackhole_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = if nrc_percent.saturating_add(blackhole_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(blackhole_percent)
            };
            let expected_burn = total_fee.saturating_sub(expected_nrc);

            MOCK_AUTHOR.with(|v| *v.borrow_mut() = None);
            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                total_fee,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            PowOnchainFeeRouter::<Test, Balances, MockFindAuthor, MockNrcAccountProvider>::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 900);
            assert_eq!(Balances::free_balance(nrc), expected_nrc);
            assert_eq!(Balances::total_issuance(), issuance_before - expected_burn);
        });
    }

    #[test]
    fn fee_router_burns_nrc_share_when_nrc_account_missing() {
        new_test_ext().execute_with(|| {
            let payer = account(1);
            let miner = account(9);
            let reward_wallet = account(8);
            let issuance_before = Balances::total_issuance();
            let total_fee = 100u128;
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let blackhole_percent = primitives::core_const::ONCHAIN_FEE_BLACKHOLE_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(blackhole_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let expected_burn = total_fee.saturating_sub(expected_fullnode);

            fullnode_pow_reward::RewardWalletByMiner::<Test>::insert(&miner, &reward_wallet);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner.clone()));
            let credit = <Balances as Balanced<AccountId32>>::withdraw(
                &payer,
                total_fee,
                Precision::Exact,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .expect("payer should have enough balance");

            PowOnchainFeeRouter::<Test, Balances, MockFindAuthor, MockNrcAccountProviderNone>::on_nonzero_unbalanced(credit);

            assert_eq!(Balances::free_balance(payer), 900);
            assert_eq!(Balances::free_balance(reward_wallet), expected_fullnode);
            assert_eq!(Balances::total_issuance(), issuance_before - expected_burn);
        });
    }

    #[test]
    fn correct_and_deposit_does_not_refund_overpayment() {
        type Adapter = PowOnchainChargeAdapter<
            Balances,
            PowOnchainFeeRouter<Test, Balances, MockFindAuthor, MockNrcAccountProviderNone>,
            AmountExtractorAmount,
            (),
        >;

        new_test_ext().execute_with(|| {
            let who = account(1);
            let call = sample_call();
            let info = call.get_dispatch_info();
            let issuance_before = Balances::total_issuance();

            MOCK_AUTHOR.with(|v| *v.borrow_mut() = None);
            let liquidity =
                <Adapter as OnChargeTransaction<Test>>::withdraw_fee(&who, &call, &info, 0, 5)
                    .expect("withdraw should succeed");
            assert_eq!(Balances::free_balance(&who), 945);

            assert_ok!(<Adapter as OnChargeTransaction<Test>>::correct_and_deposit_fee(
                &who,
                &info,
                &Default::default(),
                1, // pretend corrected fee is tiny; adapter intentionally ignores it
                5,
                liquidity,
            ));

            assert_eq!(Balances::free_balance(&who), 945);
            assert_eq!(Balances::total_issuance(), issuance_before - 55);
        });
    }

    #[test]
    fn tip_is_routed_with_fee_using_same_distribution() {
        type Adapter = PowOnchainChargeAdapter<
            Balances,
            PowOnchainFeeRouter<Test, Balances, MockFindAuthor, MockNrcAccountProvider>,
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

            fullnode_pow_reward::RewardWalletByMiner::<Test>::insert(&miner, &reward_wallet);
            MOCK_AUTHOR.with(|v| *v.borrow_mut() = Some(miner));

            let total_fee = 15u128; // base 10 + tip 5
            let fullnode_percent = primitives::core_const::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
            let nrc_percent = primitives::core_const::ONCHAIN_FEE_NRC_PERCENT as u128;
            let blackhole_percent = primitives::core_const::ONCHAIN_FEE_BLACKHOLE_PERCENT as u128;
            let total_percent = fullnode_percent
                .saturating_add(nrc_percent)
                .saturating_add(blackhole_percent);
            let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
            let remainder = total_fee.saturating_sub(expected_fullnode);
            let expected_nrc = if nrc_percent.saturating_add(blackhole_percent) == 0 {
                0
            } else {
                remainder.saturating_mul(nrc_percent)
                    / nrc_percent.saturating_add(blackhole_percent)
            };

            let liquidity =
                <Adapter as OnChargeTransaction<Test>>::withdraw_fee(&who, &call, &info, 0, 5)
                    .expect("withdraw should succeed");
            assert_ok!(<Adapter as OnChargeTransaction<Test>>::correct_and_deposit_fee(
                &who,
                &info,
                &Default::default(),
                total_fee,
                5,
                liquidity,
            ));

            assert_eq!(Balances::free_balance(who), 985);
            assert_eq!(Balances::free_balance(reward_wallet), expected_fullnode);
            assert_eq!(Balances::free_balance(nrc), expected_nrc);
        });
    }
}
