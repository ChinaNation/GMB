#![cfg(test)]

use super::*;
use crate::pallet::{CreatorPlans, PlatformPrice, RenewalIndex, Subscriptions};
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU128, ConstU32, Hooks, UnixTime},
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
    <SquarePost as Hooks<u64>>::on_finalize(System::block_number());
}

pub struct TestCitizenIdentity;
impl SquarePostCitizenIdentityProvider<AccountId32> for TestCitizenIdentity {
    fn cid_number(owner_account: &AccountId32) -> Option<Vec<u8>> {
        (*owner_account == verified_account()).then(|| b"GD001-CTZN1-000000001-2026".to_vec())
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
    type MaxSquarePostIdLen = ConstU32<64>;
    type MaxSquareCidNumberLen = ConstU32<32>;
    type MaxSquareStorageReceiptIdLen = ConstU32<96>;
    type MaxSubscriptionRenewalsPerBlock = ConstU32<64>;
    type WeightInfo = ();
}

// 平台机构永久固定为创世技术公司，测试与 pallet 共用同一创世常量 CID。
const PLATFORM_CID: &[u8] =
    primitives::cid::china::citizenchain::CITIZENCHAIN_TECHNOLOGY
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
            pending_plan: None,
            started_at: 1_699_000_000_000,
            last_charged_at: 1_699_000_000_000,
            last_charged_price_fen: 1,
            paid_until: 1_800_000_000_000,
            subscription_status: SubscriptionStatus::Active,
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
fn payment_failure_terminates_and_removes_schedule() {
    new_test_ext().execute_with(|| {
        setup_platform();
        let key = (poor_account(), IssuerKey::Platform);
        Subscriptions::<Test>::insert(
            &key,
            SubscriptionState {
                plan: platform_plan(),
                pending_plan: None,
                started_at: 1,
                last_charged_at: 1,
                last_charged_price_fen: PLATFORM_PRICE,
                paid_until: 2,
                subscription_status: SubscriptionStatus::Active,
            },
        );
        SquarePost::schedule_renewal(&key, 2);
        finalize_at(2);
        assert_eq!(
            Subscriptions::<Test>::get(&key)
                .expect("state exists")
                .subscription_status,
            SubscriptionStatus::Terminated
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
fn pending_plan_is_applied_by_automatic_renewal() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan(),
            PLATFORM_PRICE,
        ));
        let new_price = PlatformPrice::<Test>::get(MembershipLevel::Freedom).expect("price");
        assert_ok!(SquarePost::change_subscription_plan(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            platform_plan_for(MembershipLevel::Freedom),
            new_price,
        ));
        let due = RenewalIndex::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("scheduled");
        let before = Balances::free_balance(subscriber_account());
        finalize_at(due);
        let state = Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("state exists");
        assert_eq!(state.plan, platform_plan_for(MembershipLevel::Freedom));
        assert_eq!(state.pending_plan, None);
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            before - new_price
        );
    });
}

#[test]
fn creator_subscription_renews_to_creator_with_current_chain_price() {
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
        CreatorPlans::<Test>::insert(
            creator_account(),
            CreatorTiers::try_from(vec![creator_tier(75)]).expect("tiers fit"),
        );
        let due = RenewalIndex::<Test>::get((
            subscriber_account(),
            IssuerKey::Creator(creator_account()),
        ))
        .expect("scheduled");
        let creator_before = Balances::free_balance(creator_account());
        finalize_at(due);
        assert_eq!(
            Balances::free_balance(creator_account()),
            creator_before + 75
        );
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
        pending_plan: None,
        started_at: 1_700_000_000_000,
        last_charged_at: 1_700_000_000_000,
        last_charged_price_fen: PLATFORM_PRICE,
        paid_until: 1_702_000_000_000,
        subscription_status: SubscriptionStatus::Active,
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
