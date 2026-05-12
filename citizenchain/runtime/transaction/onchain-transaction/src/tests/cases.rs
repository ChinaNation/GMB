#![cfg(test)]

use super::*;

#[test]
fn onchain_fee_round_and_min_work() {
    let rate = Perbill::from_parts(1_000_000); // 0.1%
                                               // 1分*0.1%=0.001分 => round=0分，应用最低10分
    let fee_small = mul_perbill_round(1, rate).max(primitives::fee_policy::ONCHAIN_MIN_FEE);
    assert_eq!(fee_small, 10);

    // 10000分(100元)*0.1%=10分，刚好最低线
    let fee_boundary = mul_perbill_round(10_000, rate).max(primitives::fee_policy::ONCHAIN_MIN_FEE);
    assert_eq!(fee_boundary, 10);

    // 50000分(500元)*0.1%=50分，大于最低线按实际收取
    let fee_large = mul_perbill_round(50_000, rate).max(primitives::fee_policy::ONCHAIN_MIN_FEE);
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
fn custom_fee_with_tip_handles_all_fee_kinds() {
    new_test_ext().execute_with(|| {
        let who = account(1);
        let call = sample_call();
        let info = call.get_dispatch_info();

        // OnchainAmount：50_000 * 0.1% = 50 分，+ tip(3) => 53 分
        let fee_amount = custom_fee_with_tip::<Test, Balances, FeeKindExtractorOnchainAmount>(
            &who, &call, &info, 3,
        )
        .expect("onchain amount fee must be computable");
        assert_eq!(fee_amount, 53);

        // VoteFlat：投票 / 治理固定 1 元，+ tip(4) => 104 分
        let fee_vote =
            custom_fee_with_tip::<Test, Balances, FeeKindExtractorVoteFlat>(&who, &call, &info, 4)
                .expect("vote flat fee must be computable");
        assert_eq!(fee_vote, 104);

        // OffchainFee：清算手续费在清算模块执行，本层仅保留 tip
        let fee_offchain = custom_fee_with_tip::<Test, Balances, FeeKindExtractorOffchainFee>(
            &who, &call, &info, 5,
        )
        .expect("offchain fee kind must only charge tip here");
        assert_eq!(fee_offchain, 5);

        // Free：不收基础费，仅返回 tip
        let fee_free =
            custom_fee_with_tip::<Test, Balances, FeeKindExtractorFree>(&who, &call, &info, 7)
                .expect("free call must only charge tip");
        assert_eq!(fee_free, 7);

        // Unknown：拒绝交易，避免新增调用漏归类
        let unknown_err =
            custom_fee_with_tip::<Test, Balances, FeeKindExtractorUnknown>(&who, &call, &info, 0)
                .expect_err("unknown extract result should be rejected");
        assert_eq!(unknown_err, InvalidTransaction::Call.into());
    });
}

#[test]
fn withdraw_and_can_withdraw_use_default_payer_and_min_fee() {
    type Adapter = OnchainChargeAdapter<Balances, (), FeeKindExtractorTinyOnchainAmount, ()>;

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
        OnchainChargeAdapter<Balances, (), FeeKindExtractorTinyOnchainAmount, FeePayerAsAccount2>;

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
    type Adapter = OnchainChargeAdapter<Balances, (), FeeKindExtractorFree, ()>;

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
    type Adapter = OnchainChargeAdapter<Balances, (), FeeKindExtractorTinyOnchainAmount, ()>;

    new_test_ext().execute_with(|| {
        let poor = account(3);
        let call = sample_call();
        let info = call.get_dispatch_info();

        assert!(<Adapter as OnChargeTransaction<Test>>::can_withdraw_fee(
            &poor, &call, &info, 0, 0
        )
        .is_err());

        assert!(
            <Adapter as OnChargeTransaction<Test>>::withdraw_fee(&poor, &call, &info, 0, 0)
                .is_err()
        );
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
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT as u128;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
        let remainder = total_fee.saturating_sub(expected_fullnode);
        let expected_nrc = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
            0
        } else {
            remainder.saturating_mul(nrc_percent) / nrc_percent.saturating_add(safety_fund_percent)
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
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT as u128;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
        let remainder = total_fee.saturating_sub(expected_fullnode);
        let expected_nrc = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
            0
        } else {
            remainder.saturating_mul(nrc_percent) / nrc_percent.saturating_add(safety_fund_percent)
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
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT as u128;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
        let remainder = total_fee.saturating_sub(expected_fullnode);
        let expected_nrc = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
            0
        } else {
            remainder.saturating_mul(nrc_percent) / nrc_percent.saturating_add(safety_fund_percent)
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
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT as u128;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
        let remainder = total_fee.saturating_sub(expected_fullnode);
        // NRC 账户缺失时：NRC 份额的 nrc_credit 被 drop（销毁），安全基金正常分配。
        let expected_nrc_for_split = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
            0
        } else {
            remainder.saturating_mul(nrc_percent) / nrc_percent.saturating_add(safety_fund_percent)
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
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT as u128;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
        let remainder = total_fee.saturating_sub(expected_fullnode);
        let expected_nrc =
            remainder.saturating_mul(nrc_percent) / nrc_percent.saturating_add(safety_fund_percent);
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
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT as u128;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
        let remainder = total_fee.saturating_sub(expected_fullnode);
        let expected_nrc =
            remainder.saturating_mul(nrc_percent) / nrc_percent.saturating_add(safety_fund_percent);
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
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT as u128;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
        let remainder = total_fee.saturating_sub(expected_fullnode);
        let expected_nrc =
            remainder.saturating_mul(nrc_percent) / nrc_percent.saturating_add(safety_fund_percent);
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
        FeeKindExtractorOnchainAmount,
        (),
    >;

    new_test_ext().execute_with(|| {
        let who = account(1);
        let call = sample_call();
        let info = call.get_dispatch_info();
        let safety_fund = AccountId32::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS);
        let issuance_before = Balances::total_issuance();
        let total_fee = 55u128; // base 50 + tip 5
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT as u128;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
        let remainder = total_fee.saturating_sub(expected_fullnode);
        let expected_nrc_split = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
            0
        } else {
            remainder.saturating_mul(nrc_percent) / nrc_percent.saturating_add(safety_fund_percent)
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
        assert_eq!(
            fee_share_burn_event_count(pallet::BurnReason::AuthorMissing, expected_fullnode),
            1
        );
        assert_eq!(
            fee_share_burn_event_count(pallet::BurnReason::NrcMissing, expected_nrc_split),
            1
        );
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
        FeeKindExtractorOnchainAmount,
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
        FeeKindExtractorTinyOnchainAmount,
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
        let fullnode_percent = primitives::fee_policy::ONCHAIN_FEE_FULLNODE_PERCENT as u128;
        let nrc_percent = primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT as u128;
        let safety_fund_percent = primitives::fee_policy::ONCHAIN_FEE_SAFETY_FUND_PERCENT as u128;
        let total_percent = fullnode_percent
            .saturating_add(nrc_percent)
            .saturating_add(safety_fund_percent);
        let expected_fullnode = total_fee.saturating_mul(fullnode_percent) / total_percent;
        let remainder = total_fee.saturating_sub(expected_fullnode);
        let expected_nrc = if nrc_percent.saturating_add(safety_fund_percent) == 0 {
            0
        } else {
            remainder.saturating_mul(nrc_percent) / nrc_percent.saturating_add(safety_fund_percent)
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
        assert_eq!(fee_share_burn_event_total(), 0);
    });
}
