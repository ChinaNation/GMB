#![cfg(test)]

use super::*;
use crate::pallet::{CreatorPlans, PlatformPrice, RenewalIndex, Subscriptions};
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU128, ConstU32, Hooks, UnixTime},
    weights::Weight,
};
use frame_system as system;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use std::cell::RefCell;

type Balance = u128;
type Block = frame_system::mocking::MockBlock<Test>;

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
    pub type SquarePost = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type AccountData = pallet_balances::AccountData<Balance>;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type MaxLocks = ConstU32<0>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = ConstU32<0>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
    type WeightInfo = ();
}

thread_local! {
    static NOW_MS: RefCell<u64> = const { RefCell::new(1_700_000_000_000) };
}

pub struct MockTime;
impl UnixTime for MockTime {
    fn now() -> core::time::Duration {
        core::time::Duration::from_millis(NOW_MS.with(|value| *value.borrow()))
    }
}

fn set_now(value: u64) {
    NOW_MS.with(|now| *now.borrow_mut() = value);
}

fn finalize_at(value: u64) {
    set_now(value);
    // 续费在 on_idle 按剩余块权重排空；测试给足权重使全部到期在 backstop 内处理。
    <SquarePost as Hooks<u64>>::on_idle(System::block_number(), Weight::MAX);
}

pub struct TestCitizenIdentity;
impl SquarePostCitizenIdentityProvider<AccountId32> for TestCitizenIdentity {
    fn cid_number(owner_account_id: &AccountId32) -> Option<Vec<u8>> {
        (*owner_account_id == verified_account()).then(|| b"GD001-CTZN1-000000001-2026".to_vec())
    }
}

pub struct MockInstitutionQuery;
impl entity_primitives::InstitutionMultisigQuery<AccountId32> for MockInstitutionQuery {
    fn lookup_institution_account(cid_number: &[u8], account_name: &[u8]) -> Option<AccountId32> {
        (cid_number == PLATFORM_CID
            && account_name == primitives::account_derive::RESERVED_NAME_FEE)
            .then(platform_fee_account)
    }

    fn lookup_cid(_addr: &AccountId32) -> Option<Vec<u8>> {
        None
    }
    fn lookup_org(_addr: &AccountId32) -> Option<primitives::cid::code::InstitutionCode> {
        None
    }
    fn lookup_admin_config(
        _addr: &AccountId32,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId32>> {
        None
    }
    fn account_exists(_addr: &AccountId32) -> bool {
        false
    }
}

impl crate::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CitizenIdentity = TestCitizenIdentity;
    type Currency = Balances;
    type TimeProvider = MockTime;
    type InstitutionAccountQuery = MockInstitutionQuery;
    type InternalVoteEngine = ();
    type InstitutionRoleAuthorization = ();
    type MaxSquarePostIdLen = ConstU32<64>;
    type MaxSquareCidNumberLen = ConstU32<32>;
    type MaxSquareStorageReceiptIdLen = ConstU32<96>;
    type MaxSubscriptionRenewalsPerBlock = ConstU32<64>;
    type WeightInfo = ();
}

// 平台机构永久固定为公民链基金会，测试与 pallet 共用同一创世常量 CID。
const PLATFORM_CID: &[u8] = primitives::cid::china::citizenchain::CITIZENCHAIN_FOUNDATION
    .cid_number
    .as_bytes();
const PLATFORM_PRICE: u128 = 5_999_900;

fn account(byte: u8) -> AccountId32 {
    AccountId32::new([byte; 32])
}
fn verified_account() -> AccountId32 {
    account(1)
}
fn visitor_account() -> AccountId32 {
    account(2)
}
fn subscriber_account() -> AccountId32 {
    account(3)
}
fn poor_account() -> AccountId32 {
    account(5)
}
fn creator_account() -> AccountId32 {
    account(6)
}
fn platform_fee_account() -> AccountId32 {
    account(9)
}

fn platform_plan() -> SubscriptionPlan {
    platform_plan_for(MembershipLevel::Spark)
}

fn platform_plan_for(membership_level: MembershipLevel) -> SubscriptionPlan {
    SubscriptionPlan::Platform { membership_level }
}

fn creator_plan() -> SubscriptionPlan {
    SubscriptionPlan::Creator {
        tier_id: TierId::try_from(b"supporter".to_vec()).expect("tier id fits"),
        billing_period: BillingPeriod::Monthly,
    }
}

fn creator_tier(price_fen: u128) -> CreatorTier {
    CreatorTier {
        tier_id: TierId::try_from(b"supporter".to_vec()).expect("tier id fits"),
        prices_fen: PeriodPrices::try_from(vec![PeriodPrice {
            billing_period: BillingPeriod::Monthly,
            price_fen,
        }])
        .expect("period fits"),
    }
}

fn setup_platform() {
    // 平台 CID 已是创世常量，无需绑定；仅播种测试所需三档价。
    PlatformPrice::<Test>::insert(MembershipLevel::Spark, PLATFORM_PRICE);
    PlatformPrice::<Test>::insert(MembershipLevel::Freedom, 199_900);
}

fn set_active_platform_member(account: AccountId32) {
    Subscriptions::<Test>::insert(
        (account, IssuerKey::Platform),
        SubscriptionState {
            plan: platform_plan_for(MembershipLevel::Freedom),
            started_at: 1_699_000_000_000,
            last_charged_at: 1_699_000_000_000,
            last_charged_price_fen: 1,
            paid_until: 1_800_000_000_000,
            subscription_status: SubscriptionStatus::Active,
            authorized_price_fen: 1,
            suspend_reason: None,
        },
    );
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (subscriber_account(), 1_000_000_000),
            (poor_account(), 100),
            (creator_account(), 1_000_000_000),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut storage)
    .expect("balances genesis should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
        set_now(1_700_000_000_000);
    });
    ext
}

#[test]
fn real_calendar_month_keeps_calendar_date_and_time() {
    let source = 1_608_467_696_789u64;
    let expected = 1_611_146_096_789u64;
    assert_eq!(
        crate::subscription::add_calendar_period(source, BillingPeriod::Monthly),
        Some(expected)
    );
}

#[test]
fn real_calendar_handles_leap_year_and_missing_target_date() {
    assert_eq!(
        crate::subscription::add_calendar_period(1_582_938_123_456, BillingPeriod::Yearly),
        Some(1_614_474_123_456)
    );
    assert_eq!(
        crate::subscription::add_calendar_period(1_612_065_906_321, BillingPeriod::Monthly),
        Some(1_614_485_106_321)
    );
}

#[test]
fn post_regression_keeps_existing_content_boundary() {
    new_test_ext().execute_with(|| {
        assert_ok!(SquarePost::publish_post(
            RuntimeOrigin::signed(visitor_account()),
            b"sqp_normal_001".to_vec(),
            SquarePostCategory::Normal,
            [9u8; 32],
            b"receipt".to_vec(),
            1_893_456_000_000,
        ));
        assert_noop!(
            SquarePost::publish_post(
                RuntimeOrigin::signed(visitor_account()),
                b"sqp_campaign_001".to_vec(),
                SquarePostCategory::Campaign,
                [9u8; 32],
                b"receipt".to_vec(),
                1_893_456_000_000,
            ),
            Error::<Test>::CampaignRequiresCitizen
        );
    });
}

#[test]
fn subscribe_charges_immediately_and_schedules_real_calendar_due_time() {
    new_test_ext().execute_with(|| {
        setup_platform();
        let before = Balances::free_balance(subscriber_account());
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan(),
            PLATFORM_PRICE,
        ));
        let state = Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("state exists");
        let expected_due =
            crate::subscription::add_calendar_period(1_700_000_000_000, BillingPeriod::Monthly)
                .expect("calendar fits");
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            before - PLATFORM_PRICE
        );
        assert_eq!(state.paid_until, expected_due);
        assert_eq!(state.subscription_status, SubscriptionStatus::Active);
        // 5.6a：首扣后已授权价=扣款价，无挂起原因。
        assert_eq!(state.authorized_price_fen, PLATFORM_PRICE);
        assert_eq!(state.suspend_reason, None);
        assert_eq!(
            RenewalIndex::<Test>::get((subscriber_account(), IssuerKey::Platform)),
            Some(expected_due)
        );
    });
}

#[test]
fn runtime_renews_at_due_time_without_external_call_and_uses_latest_price() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan(),
            PLATFORM_PRICE,
        ));
        let due = RenewalIndex::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("scheduled");
        let next_price = PLATFORM_PRICE + 10;
        PlatformPrice::<Test>::insert(MembershipLevel::Spark, next_price);
        let before = Balances::free_balance(subscriber_account());
        finalize_at(due - 1);
        assert_eq!(Balances::free_balance(subscriber_account()), before);
        finalize_at(due);
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            before - next_price
        );
        let state = Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("state exists");
        assert_eq!(state.last_charged_at, due);
        assert_eq!(state.last_charged_price_fen, next_price);
        assert!(state.paid_until > due);
    });
}

#[test]
fn runtime_catches_up_every_due_period_after_blocks_resume() {
    new_test_ext().execute_with(|| {
        setup_platform();
        set_now(1_608_467_696_789);
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan(),
            PLATFORM_PRICE,
        ));
        let before = Balances::free_balance(subscriber_account());
        finalize_at(1_616_243_696_789);
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            before - PLATFORM_PRICE * 3
        );
        let state = Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("state exists");
        assert!(state.paid_until > 1_616_243_696_789);
    });
}

#[test]
fn payment_failure_suspends_and_removes_schedule() {
    new_test_ext().execute_with(|| {
        setup_platform();
        let key = (poor_account(), IssuerKey::Platform);
        Subscriptions::<Test>::insert(
            &key,
            SubscriptionState {
                plan: platform_plan(),
                started_at: 1,
                last_charged_at: 1,
                last_charged_price_fen: PLATFORM_PRICE,
                paid_until: 2,
                subscription_status: SubscriptionStatus::Active,
                authorized_price_fen: PLATFORM_PRICE,
                suspend_reason: None,
            },
        );
        SquarePost::schedule_renewal(&key, 2);
        finalize_at(2);
        // 余额不足 → 挂起（不再终止），退出续费调度，保留订阅。
        let state = Subscriptions::<Test>::get(&key).expect("state exists");
        assert_eq!(state.subscription_status, SubscriptionStatus::Suspended);
        assert_eq!(
            state.suspend_reason,
            Some(crate::SuspendReason::InsufficientBalance)
        );
        assert!(!RenewalIndex::<Test>::contains_key(&key));
    });
}

#[test]
fn cancellation_preserves_paid_rights_and_revokes_future_charges() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan(),
            PLATFORM_PRICE,
        ));
        let key = (subscriber_account(), IssuerKey::Platform);
        let paid_until = Subscriptions::<Test>::get(&key)
            .expect("state exists")
            .paid_until;
        assert_ok!(SquarePost::cancel(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
        ));
        let state = Subscriptions::<Test>::get(&key).expect("state exists");
        assert_eq!(state.paid_until, paid_until);
        assert_eq!(state.subscription_status, SubscriptionStatus::Cancelled);
        assert!(!RenewalIndex::<Test>::contains_key(&key));
        let before = Balances::free_balance(subscriber_account());
        finalize_at(paid_until);
        assert_eq!(Balances::free_balance(subscriber_account()), before);
    });
}

#[test]
fn change_plan_upgrade_charges_difference_immediately() {
    new_test_ext().execute_with(|| {
        setup_platform();
        let now = 1_700_000_000_000u64;
        // 先订自由档（199_900）。
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan_for(MembershipLevel::Freedom),
            199_900,
        ));
        let balance_after_subscribe = Balances::free_balance(subscriber_account());
        let base_end = crate::subscription::add_calendar_period(now, BillingPeriod::Monthly)
            .expect("calendar fits");
        // 立即升档到薪火（5_999_900）；刚扣款故剩余权益=全额 199_900。
        assert_ok!(SquarePost::change_subscription_plan(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan(),
            PLATFORM_PRICE,
        ));
        // 升档补扣「新价 − 剩余权益」= 5_999_900 − 199_900。
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            balance_after_subscribe - (PLATFORM_PRICE - 199_900)
        );
        let state = Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("state exists");
        assert_eq!(state.plan, platform_plan());
        assert_eq!(state.authorized_price_fen, PLATFORM_PRICE);
        assert_eq!(state.subscription_status, SubscriptionStatus::Active);
        // 新周期从现在起算。
        assert_eq!(state.paid_until, base_end);
    });
}

#[test]
fn change_plan_downgrade_extends_duration() {
    new_test_ext().execute_with(|| {
        setup_platform();
        let now = 1_700_000_000_000u64;
        // 先订薪火（5_999_900）。
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan(),
            PLATFORM_PRICE,
        ));
        let balance_after_subscribe = Balances::free_balance(subscriber_account());
        let base_end = crate::subscription::add_calendar_period(now, BillingPeriod::Monthly)
            .expect("calendar fits");
        let new_price = PlatformPrice::<Test>::get(MembershipLevel::Freedom).expect("price");
        // 立即降档到自由档；剩余权益=全额 5_999_900 ＞ 新价 199_900。
        assert_ok!(SquarePost::change_subscription_plan(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan_for(MembershipLevel::Freedom),
            new_price,
        ));
        // 降档不扣款。
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            balance_after_subscribe
        );
        let state = Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("state exists");
        assert_eq!(state.plan, platform_plan_for(MembershipLevel::Freedom));
        assert_eq!(state.authorized_price_fen, new_price);
        // 剩余信用按新档单价折算成额外时长叠加到 base_end。
        let period_ms = u128::from(base_end - now);
        let extra_ms = (PLATFORM_PRICE - new_price) * period_ms / new_price;
        assert_eq!(state.paid_until, base_end + extra_ms as u64);
    });
}

#[test]
fn change_plan_prorates_partial_remaining_credit() {
    new_test_ext().execute_with(|| {
        setup_platform();
        let now = 1_700_000_000_000u64;
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan_for(MembershipLevel::Freedom),
            199_900,
        ));
        let te = crate::subscription::add_calendar_period(now, BillingPeriod::Monthly)
            .expect("calendar");
        // 推进到本期中段，剩余权益按时间比例折算。
        let mid = now + (te - now) / 3;
        set_now(mid);
        let balance_before_change = Balances::free_balance(subscriber_account());
        let credit = 199_900u128 * u128::from(te - mid) / u128::from(te - now);
        assert_ok!(SquarePost::change_subscription_plan(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan(),
            PLATFORM_PRICE,
        ));
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            balance_before_change - (PLATFORM_PRICE - credit)
        );
        let base_end = crate::subscription::add_calendar_period(mid, BillingPeriod::Monthly)
            .expect("calendar");
        let state = Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("state exists");
        assert_eq!(state.paid_until, base_end);
    });
}

#[test]
fn creator_subscription_renews_at_authorized_price_when_unchanged() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        CreatorPlans::<Test>::insert(
            creator_account(),
            CreatorTiers::try_from(vec![creator_tier(50)]).expect("tiers fit"),
        );
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Creator(creator_account()),
            creator_plan(),
            50,
        ));
        let due = RenewalIndex::<Test>::get((
            subscriber_account(),
            IssuerKey::Creator(creator_account()),
        ))
        .expect("scheduled");
        let creator_before = Balances::free_balance(creator_account());
        // 价格未变 → 自动续扣当前授权价。
        finalize_at(due);
        assert_eq!(
            Balances::free_balance(creator_account()),
            creator_before + 50
        );
    });
}

#[test]
fn creator_price_change_suspends_renewal_until_reconsent() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        let ck = (subscriber_account(), IssuerKey::Creator(creator_account()));
        CreatorPlans::<Test>::insert(
            creator_account(),
            CreatorTiers::try_from(vec![creator_tier(50)]).expect("tiers fit"),
        );
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Creator(creator_account()),
            creator_plan(),
            50,
        ));
        // 创作者升价 50 → 75。
        CreatorPlans::<Test>::insert(
            creator_account(),
            CreatorTiers::try_from(vec![creator_tier(75)]).expect("tiers fit"),
        );
        let due = RenewalIndex::<Test>::get(&ck).expect("scheduled");
        let creator_before = Balances::free_balance(creator_account());
        finalize_at(due);
        // 续费挂起、创作者未收款、离调度。
        let state = Subscriptions::<Test>::get(&ck).expect("state exists");
        assert_eq!(state.subscription_status, SubscriptionStatus::Suspended);
        assert_eq!(
            state.suspend_reason,
            Some(crate::SuspendReason::NeedReconsent)
        );
        assert_eq!(Balances::free_balance(creator_account()), creator_before);
        assert!(!RenewalIndex::<Test>::contains_key(&ck));
        // 订阅者按新价再签名 → 恢复 Active 并扣新价。
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Creator(creator_account()),
            creator_plan(),
            75,
        ));
        let resumed = Subscriptions::<Test>::get(&ck).expect("state exists");
        assert_eq!(resumed.subscription_status, SubscriptionStatus::Active);
        assert_eq!(resumed.authorized_price_fen, 75);
        assert_eq!(
            Balances::free_balance(creator_account()),
            creator_before + 75
        );
        assert!(RenewalIndex::<Test>::contains_key(&ck));
    });
}

#[test]
fn creator_price_change_reconsent_before_lapse_keeps_active_without_charge() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        let ck = (subscriber_account(), IssuerKey::Creator(creator_account()));
        CreatorPlans::<Test>::insert(
            creator_account(),
            CreatorTiers::try_from(vec![creator_tier(50)]).expect("tiers fit"),
        );
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Creator(creator_account()),
            creator_plan(),
            50,
        ));
        let due = RenewalIndex::<Test>::get(&ck).expect("scheduled");
        // 创作者升价 50 → 75；订阅者到期前再签名。
        CreatorPlans::<Test>::insert(
            creator_account(),
            CreatorTiers::try_from(vec![creator_tier(75)]).expect("tiers fit"),
        );
        let creator_before = Balances::free_balance(creator_account());
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Creator(creator_account()),
            creator_plan(),
            75,
        ));
        // 保持 Active、不即时扣款、已授权价更新。
        let state = Subscriptions::<Test>::get(&ck).expect("state exists");
        assert_eq!(state.subscription_status, SubscriptionStatus::Active);
        assert_eq!(state.authorized_price_fen, 75);
        assert_eq!(Balances::free_balance(creator_account()), creator_before);
        // 下期按新价自动扣。
        finalize_at(due);
        assert_eq!(
            Balances::free_balance(creator_account()),
            creator_before + 75
        );
    });
}

#[test]
fn creator_loses_membership_pauses_fans_and_resumes() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        let ck = (subscriber_account(), IssuerKey::Creator(creator_account()));
        CreatorPlans::<Test>::insert(
            creator_account(),
            CreatorTiers::try_from(vec![creator_tier(50)]).expect("tiers fit"),
        );
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Creator(creator_account()),
            creator_plan(),
            50,
        ));
        let due = RenewalIndex::<Test>::get(&ck).expect("scheduled");
        // 创作者掉平台会员。
        Subscriptions::<Test>::remove((creator_account(), IssuerKey::Platform));
        let creator_before = Balances::free_balance(creator_account());
        finalize_at(due);
        // 粉丝暂停：CreatorPaused、未扣、未终止、仍在调度、下周期重试。
        let state = Subscriptions::<Test>::get(&ck).expect("state exists");
        assert_eq!(state.subscription_status, SubscriptionStatus::CreatorPaused);
        assert_eq!(Balances::free_balance(creator_account()), creator_before);
        let retry = RenewalIndex::<Test>::get(&ck).expect("still scheduled");
        assert!(retry > due);
        // 创作者恢复平台会员 → 下个重试自动续扣、回 Active。
        set_active_platform_member(creator_account());
        finalize_at(retry);
        let resumed = Subscriptions::<Test>::get(&ck).expect("state exists");
        assert_eq!(resumed.subscription_status, SubscriptionStatus::Active);
        assert_eq!(
            Balances::free_balance(creator_account()),
            creator_before + 50
        );
    });
}

#[test]
fn creator_paused_and_repriced_suspends_for_reconsent_on_return() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        let ck = (subscriber_account(), IssuerKey::Creator(creator_account()));
        CreatorPlans::<Test>::insert(
            creator_account(),
            CreatorTiers::try_from(vec![creator_tier(50)]).expect("tiers fit"),
        );
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Creator(creator_account()),
            creator_plan(),
            50,
        ));
        let due = RenewalIndex::<Test>::get(&ck).expect("scheduled");
        Subscriptions::<Test>::remove((creator_account(), IssuerKey::Platform));
        finalize_at(due);
        let retry = RenewalIndex::<Test>::get(&ck).expect("still scheduled");
        // 创作者恢复会员但同时改了价 → 恢复时先挂起待再签名。
        set_active_platform_member(creator_account());
        CreatorPlans::<Test>::insert(
            creator_account(),
            CreatorTiers::try_from(vec![creator_tier(75)]).expect("tiers fit"),
        );
        finalize_at(retry);
        let state = Subscriptions::<Test>::get(&ck).expect("state exists");
        assert_eq!(state.subscription_status, SubscriptionStatus::Suspended);
        assert_eq!(
            state.suspend_reason,
            Some(crate::SuspendReason::NeedReconsent)
        );
        assert!(!RenewalIndex::<Test>::contains_key(&ck));
    });
}

#[test]
fn only_effective_platform_member_can_publish_creator_plans() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            SquarePost::set_creator_plans(
                RuntimeOrigin::signed(creator_account()),
                vec![creator_tier(50)],
            ),
            Error::<Test>::CreatorNotPlatformMember
        );
        set_active_platform_member(creator_account());
        assert_ok!(SquarePost::set_creator_plans(
            RuntimeOrigin::signed(creator_account()),
            vec![creator_tier(50)],
        ));
        assert_eq!(CreatorPlans::<Test>::get(creator_account()).len(), 1);
    });
}

#[test]
fn scale_fixture_matches_target_contract() {
    use codec::Encode;
    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
    let state = SubscriptionState {
        plan: platform_plan(),
        started_at: 1_700_000_000_000,
        last_charged_at: 1_700_000_000_000,
        last_charged_price_fen: PLATFORM_PRICE,
        paid_until: 1_702_000_000_000,
        subscription_status: SubscriptionStatus::Active,
        authorized_price_fen: PLATFORM_PRICE,
        suspend_reason: None,
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/subscription_scale_vectors.json"
    ))
    .expect("fixture parses");
    assert_eq!(
        to_hex(&state.encode()),
        fixture["state_platform"].as_str().unwrap()
    );
}

#[test]
fn genesis_build_seeds_default_platform_prices() {
    use frame_support::traits::BuildGenesisConfig;
    new_test_ext().execute_with(|| {
        crate::pallet::GenesisConfig::<Test>::default().build();
        assert_eq!(
            PlatformPrice::<Test>::get(MembershipLevel::Freedom),
            Some(crate::pallet::FREEDOM_PRICE_FEN)
        );
        assert_eq!(
            PlatformPrice::<Test>::get(MembershipLevel::Democracy),
            Some(crate::pallet::DEMOCRACY_PRICE_FEN)
        );
        assert_eq!(
            PlatformPrice::<Test>::get(MembershipLevel::Spark),
            Some(crate::pallet::SPARK_PRICE_FEN)
        );
    });
}

// 平台调价治理：可测的前置校验（零价 / 非基金会）。完整「投票通过→执行→改价」
// 路径需 votingengine 集成夹具，列为跟进项。
#[test]
fn propose_platform_price_rejects_zero_price() {
    new_test_ext().execute_with(|| {
        let cid = votingengine::types::CidNumber::try_from(
            primitives::cid::china::citizenchain::CITIZENCHAIN_FOUNDATION
                .cid_number
                .as_bytes()
                .to_vec(),
        )
        .expect("cid fits");
        assert_noop!(
            SquarePost::propose_set_platform_price(
                RuntimeOrigin::signed(verified_account()),
                cid,
                primitives::cid::china::citizenchain::ROLE_CODE_GENESIS_PRODUCT_MANAGER
                    .to_vec()
                    .try_into()
                    .expect("product manager role fits"),
                MembershipLevel::Spark,
                0,
            ),
            Error::<Test>::InvalidPlatformPrice
        );
    });
}

#[test]
fn propose_platform_price_rejects_non_foundation_institution() {
    new_test_ext().execute_with(|| {
        let cid = votingengine::types::CidNumber::try_from(b"GD001-CTZN1-000000001-2026".to_vec())
            .expect("cid fits");
        assert_noop!(
            SquarePost::propose_set_platform_price(
                RuntimeOrigin::signed(verified_account()),
                cid,
                primitives::cid::china::citizenchain::ROLE_CODE_GENESIS_PRODUCT_MANAGER
                    .to_vec()
                    .try_into()
                    .expect("product manager role fits"),
                MembershipLevel::Spark,
                599_900,
            ),
            Error::<Test>::NotPlatformInstitution
        );
    });
}
