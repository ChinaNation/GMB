#![cfg(test)]

use super::*;

#[test]
fn first_year_should_mint_and_settle() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(LastSettledYear::<Test>::get(), 1);

        let first_bank = &primitives::china::china_ch::CHINA_CH[0];
        let account = shengbank_account(0);
        let expected = first_bank.stake_amount * 100u128 / 10_000u128;
        assert_eq!(Balances::free_balance(account), expected);

        let has_settled_event = System::events().iter().any(|r| {
            matches!(
                r.event,
                RuntimeEvent::ShengBankInterest(Event::ShengBankYearSettled { year: 1 })
            )
        });
        assert!(has_settled_event);
    });
}

#[test]
fn later_boundary_auto_settles_only_next_unsettled_year() {
    new_test_ext().execute_with(|| {
        // 中文注释：直接跳到第 2 年边界时，自动路径也只补下一个未结算年度。
        System::set_block_number(20);
        ShengBankInterest::on_initialize(20);

        assert_eq!(LastSettledYear::<Test>::get(), 1);

        let first_bank = &primitives::china::china_ch::CHINA_CH[0];
        let account = shengbank_account(0);
        let year1 = first_bank.stake_amount * 100u128 / 10_000u128;
        assert_eq!(Balances::free_balance(account), year1);
    });
}

#[test]
fn second_year_should_use_decayed_rate() {
    new_test_ext().execute_with(|| {
        run_to_block(20);
        assert_eq!(LastSettledYear::<Test>::get(), 2);

        let first_bank = &primitives::china::china_ch::CHINA_CH[0];
        let account = shengbank_account(0);
        let year1 = first_bank.stake_amount * 100u128 / 10_000u128;
        let year2 = first_bank.stake_amount * 99u128 / 10_000u128;
        assert_eq!(Balances::free_balance(account), year1 + year2);
    });
}

#[test]
fn should_stop_settling_after_duration_years() {
    new_test_ext().execute_with(|| {
        LastSettledYear::<Test>::put(primitives::core_const::SHENGBANK_INTEREST_DURATION_YEARS);
        let account = shengbank_account(0);
        assert_eq!(Balances::free_balance(account.clone()), 0);

        // current_year = 101（边界块），但因已到年限上限，不应继续发放。
        System::set_block_number(1010);
        ShengBankInterest::on_initialize(1010);

        assert_eq!(
            LastSettledYear::<Test>::get(),
            primitives::core_const::SHENGBANK_INTEREST_DURATION_YEARS
        );
        assert_eq!(Balances::free_balance(account), 0);
    });
}

#[test]
fn root_can_force_advance_year_for_recovery() {
    new_test_ext().execute_with(|| {
        System::set_block_number(50); // current_year = 5
        assert_ok!(ShengBankInterest::force_advance_year(
            RuntimeOrigin::root(),
            5
        ));
        assert_eq!(LastSettledYear::<Test>::get(), 5);
    });
}

#[test]
fn force_advance_year_rejects_noop_and_invalid() {
    new_test_ext().execute_with(|| {
        System::set_block_number(50); // current_year = 5
        LastSettledYear::<Test>::put(5);
        assert_noop!(
            ShengBankInterest::force_advance_year(RuntimeOrigin::root(), 5),
            Error::<Test>::InvalidYear
        );
        assert_noop!(
            ShengBankInterest::force_advance_year(RuntimeOrigin::root(), 101),
            Error::<Test>::InvalidYear
        );
    });
}

#[test]
fn force_advance_year_rejects_future_years() {
    new_test_ext().execute_with(|| {
        System::set_block_number(20); // current_year = 2
        assert_noop!(
            ShengBankInterest::force_advance_year(RuntimeOrigin::root(), 3),
            Error::<Test>::InvalidYear
        );
    });
}

#[test]
fn interest_always_goes_to_hardcoded_multisig_account() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        // 利息只能发到 CHINA_CH 中硬编码的省储行多签账户
        let first_bank = &primitives::china::china_ch::CHINA_CH[0];
        let account = shengbank_account(0);
        let expected = first_bank.stake_amount * 100u128 / 10_000u128;
        assert_eq!(Balances::free_balance(account), expected);
    });
}

#[test]
fn force_settle_years_can_backfill_multiple_years() {
    new_test_ext().execute_with(|| {
        System::set_block_number(50); // current_year = 5
        assert_ok!(ShengBankInterest::force_settle_years(
            RuntimeOrigin::root(),
            3
        ));
        assert_eq!(LastSettledYear::<Test>::get(), 3);
    });
}

#[test]
fn force_settle_years_rejects_zero_and_oversized_count() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ShengBankInterest::force_settle_years(RuntimeOrigin::root(), 0),
            Error::<Test>::InvalidOperationCount
        );
        assert_noop!(
            ShengBankInterest::force_settle_years(RuntimeOrigin::root(), 9),
            Error::<Test>::InvalidOperationCount
        );
    });
}

#[test]
fn force_settle_years_allows_max_batch() {
    new_test_ext().execute_with(|| {
        System::set_block_number(100); // current_year = 10
        assert_ok!(ShengBankInterest::force_settle_years(
            RuntimeOrigin::root(),
            8
        ));
        assert_eq!(LastSettledYear::<Test>::get(), 8);
    });
}

#[test]
fn non_root_calls_are_rejected() {
    new_test_ext().execute_with(|| {
        let caller = RuntimeOrigin::signed(AccountId32::new([1u8; 32]));

        assert_noop!(
            ShengBankInterest::force_settle_years(caller.clone(), 1),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            ShengBankInterest::force_advance_year(caller, 1),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn on_initialize_settles_only_one_year_per_boundary() {
    new_test_ext().execute_with(|| {
        System::set_block_number(100); // current_year = 10
        ShengBankInterest::on_initialize(100);
        assert_eq!(LastSettledYear::<Test>::get(), 1); // AUTO_BACKFILL_MAX_YEARS_PER_BLOCK
    });
}

#[test]
fn blocks_per_year_zero_disables_settlement() {
    new_test_ext().execute_with(|| {
        set_blocks_per_year(0);
        run_to_block(50);
        assert_eq!(LastSettledYear::<Test>::get(), 0);
        assert_eq!(Balances::free_balance(shengbank_account(0)), 0);
    });
}

#[test]
fn force_advance_then_settle_resumes() {
    // 模拟故障恢复场景：前两年因故障被 force_advance 跳过，
    // 验证自动结算从第 3 年正常恢复。
    new_test_ext().execute_with(|| {
        System::set_block_number(50); // current_year = 5
                                      // 模拟 Root 已跳过前两年故障
        LastSettledYear::<Test>::put(2);
        // 自动结算应从第 3 年开始恢复，但单个边界块只结算 1 年。
        ShengBankInterest::on_initialize(50);
        assert_eq!(LastSettledYear::<Test>::get(), 3);
        let first_bank = &primitives::china::china_ch::CHINA_CH[0];
        let account = shengbank_account(0);
        // 第 3 年利率为 98 BP。
        let year3 = first_bank.stake_amount * 98u128 / 10_000u128;
        assert_eq!(Balances::free_balance(account), year3);
    });
}

#[test]
fn force_settle_years_caps_at_current_year() {
    // 在 current_year=3 时请求补结算 8 年，验证只结算 3 年。
    new_test_ext().execute_with(|| {
        System::set_block_number(30); // current_year = 3
        assert_ok!(ShengBankInterest::force_settle_years(
            RuntimeOrigin::root(),
            8
        ));
        assert_eq!(LastSettledYear::<Test>::get(), 3);
    });
}

#[test]
fn year_100_boundary_settles_with_minimum_rate() {
    // 验证第 100 年（最后一年）的利率为 1 BP (0.01%)，且发放正确。
    new_test_ext().execute_with(|| {
        LastSettledYear::<Test>::put(99);
        System::set_block_number(1000); // current_year = 100
        ShengBankInterest::on_initialize(1000);
        assert_eq!(LastSettledYear::<Test>::get(), 100);

        let first_bank = &primitives::china::china_ch::CHINA_CH[0];
        let account = shengbank_account(0);
        // 第 100 年利率 = 100 - (100-1)*1 = 1 BP
        let expected = first_bank.stake_amount * 1u128 / 10_000u128;
        assert_eq!(Balances::free_balance(account), expected);
        assert!(expected > 0, "最后一年利息不应为零");

        // 推进到第 101 年边界，验证不再发放
        let balance_after_100 = Balances::free_balance(shengbank_account(0));
        System::set_block_number(1010); // current_year = 101
        ShengBankInterest::on_initialize(1010);
        // LastSettledYear 不应前进，余额不应变化
        assert_eq!(LastSettledYear::<Test>::get(), 100);
        assert_eq!(
            Balances::free_balance(shengbank_account(0)),
            balance_after_100
        );
    });
}
