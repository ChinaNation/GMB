#![cfg(test)]

use super::*;
use crate::pallet::{BillingKeeper, CidNumberOf, PlatformCidNumber, PlatformPrice, Subscriptions};
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU128, ConstU32, Currency, UnixTime},
};
use frame_system as system;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};

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

/// 固定时间源。`charge_due` 已无到期门，测试不需要推进时间，返回常量即可。
pub struct MockTime;
impl UnixTime for MockTime {
    fn now() -> core::time::Duration {
        core::time::Duration::from_millis(1_700_000_000_000)
    }
}

pub struct TestCitizenIdentity;
impl SquarePostCitizenIdentityProvider<AccountId32> for TestCitizenIdentity {
    fn cid_number(owner_account: &AccountId32) -> Option<Vec<u8>> {
        if *owner_account == verified_account() {
            Some(b"GD001-CTZN1-000000001-2026".to_vec())
        } else {
            None
        }
    }
}

/// 机构账户查询 mock：仅平台 CID 的「费用账户」有解，其余无。
pub struct MockInstitutionQuery;
impl entity_primitives::InstitutionMultisigQuery<AccountId32> for MockInstitutionQuery {
    fn lookup_institution_account(cid_number: &[u8], account_name: &[u8]) -> Option<AccountId32> {
        if cid_number == PLATFORM_CID
            && account_name == primitives::account_derive::RESERVED_NAME_FEE
        {
            Some(platform_fee_account())
        } else {
            None
        }
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
    type MaxSquarePostIdLen = ConstU32<64>;
    type MaxSquareCidNumberLen = ConstU32<32>;
    type MaxSquareStorageReceiptIdLen = ConstU32<96>;
    type WeightInfo = ();
}

const PLATFORM_CID: &[u8] = b"GD001-TECH1-000000001-2026";
const SPARK_PRICE_FEN: u128 = 5_999_900;

fn verified_account() -> AccountId32 {
    AccountId32::new([1u8; 32])
}
fn visitor_account() -> AccountId32 {
    AccountId32::new([2u8; 32])
}
fn subscriber_account() -> AccountId32 {
    AccountId32::new([3u8; 32])
}
fn keeper_account() -> AccountId32 {
    AccountId32::new([4u8; 32])
}
fn poor_account() -> AccountId32 {
    AccountId32::new([5u8; 32])
}
fn platform_fee_account() -> AccountId32 {
    AccountId32::new([9u8; 32])
}
fn creator_account() -> AccountId32 {
    AccountId32::new([6u8; 32])
}

/// 直接塞一条 Active 平台订阅记录（创作者门禁前置：只有平台会员才能被订阅为创作者）。
fn set_active_platform_member(account: AccountId32) {
    Subscriptions::<Test>::insert(
        (account, IssuerKey::Platform),
        SubscriptionState {
            plan: SubscriptionPlan::Level(MembershipLevel::Freedom),
            price_fen: 199_900,
            last_charged_at: 1_700_000_000_000,
            status: SubscriptionStatus::Active,
        },
    );
}

fn content_hash() -> [u8; 32] {
    [9u8; 32]
}
fn post_id(value: &[u8]) -> Vec<u8> {
    value.to_vec()
}
fn receipt() -> Vec<u8> {
    b"sqr_local_receipt".to_vec()
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (subscriber_account(), 1_000_000_000),
            (keeper_account(), 1_000_000),
            (poor_account(), 100),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut storage)
    .expect("balances genesis should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

/// 装配平台轨：三档价 + 技术公司 CID 绑定 + 续订触发方。
fn setup_platform() {
    PlatformPrice::<Test>::insert(MembershipLevel::Spark, SPARK_PRICE_FEN);
    PlatformPrice::<Test>::insert(MembershipLevel::Freedom, 199_900u128);
    let cid = CidNumberOf::<Test>::try_from(PLATFORM_CID.to_vec()).expect("cid fits");
    PlatformCidNumber::<Test>::put(cid);
    BillingKeeper::<Test>::put(keeper_account());
}

fn spark() -> SubscriptionPlan {
    SubscriptionPlan::Level(MembershipLevel::Spark)
}

// ---------------------------------------------------------------------------
// 发帖回归（不受订阅扩展影响）
// ---------------------------------------------------------------------------

#[test]
fn normal_post_can_be_published_by_visitor_wallet() {
    new_test_ext().execute_with(|| {
        assert_ok!(SquarePost::publish_post(
            RuntimeOrigin::signed(visitor_account()),
            post_id(b"sqp_normal_001"),
            SquarePostCategory::Normal,
            content_hash(),
            receipt(),
            1_893_456_000_000,
        ));
        let stored = SquarePosts::<Test>::get(
            crate::pallet::PostIdOf::<Test>::try_from(b"sqp_normal_001".to_vec())
                .expect("post id fits"),
        )
        .expect("post should be indexed");
        assert_eq!(stored.owner_account, visitor_account());
        assert_eq!(stored.cid_number, None);
    });
}

#[test]
fn campaign_post_requires_verified_citizen_wallet() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            SquarePost::publish_post(
                RuntimeOrigin::signed(visitor_account()),
                post_id(b"sqp_campaign_denied"),
                SquarePostCategory::Campaign,
                content_hash(),
                receipt(),
                1_893_456_000_000,
            ),
            Error::<Test>::CampaignRequiresCitizen
        );
    });
}

#[test]
fn duplicate_post_id_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_ok!(SquarePost::publish_post(
            RuntimeOrigin::signed(visitor_account()),
            post_id(b"sqp_duplicate"),
            SquarePostCategory::Normal,
            content_hash(),
            receipt(),
            1_893_456_000_000,
        ));
        assert_noop!(
            SquarePost::publish_post(
                RuntimeOrigin::signed(visitor_account()),
                post_id(b"sqp_duplicate"),
                SquarePostCategory::Normal,
                content_hash(),
                receipt(),
                1_893_456_000_000,
            ),
            Error::<Test>::DuplicatePostId
        );
    });
}

// ---------------------------------------------------------------------------
// 平台会员：订阅 / 取消 / 续扣 / 欠费即停
// ---------------------------------------------------------------------------

#[test]
fn subscribe_platform_charges_first_month_to_fee_account() {
    new_test_ext().execute_with(|| {
        setup_platform();
        let before = Balances::free_balance(subscriber_account());
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        // 扣款转入技术公司费用账户。
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            before - SPARK_PRICE_FEN
        );
        assert_eq!(
            Balances::free_balance(platform_fee_account()),
            SPARK_PRICE_FEN
        );
        // 订阅态记录 Active + 扣款时间戳。
        let state = Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("subscription recorded");
        assert_eq!(state.status, SubscriptionStatus::Active);
        assert_eq!(state.price_fen, SPARK_PRICE_FEN);
        assert_eq!(state.last_charged_at, 1_700_000_000_000);
    });
}

#[test]
fn subscribe_twice_same_tier_is_idempotent_no_double_charge() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        let after_first = Balances::free_balance(subscriber_account());
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        assert_eq!(Balances::free_balance(subscriber_account()), after_first);
    });
}

#[test]
fn subscribe_fails_when_platform_price_not_set() {
    new_test_ext().execute_with(|| {
        // 只绑 CID + keeper，不设价。
        let cid = CidNumberOf::<Test>::try_from(PLATFORM_CID.to_vec()).unwrap();
        PlatformCidNumber::<Test>::put(cid);
        assert_noop!(
            SquarePost::subscribe(
                RuntimeOrigin::signed(subscriber_account()),
                IssuerKey::Platform,
                spark(),
            ),
            Error::<Test>::PlatformPriceNotSet
        );
    });
}

#[test]
fn subscribe_fails_when_platform_cid_not_bound() {
    new_test_ext().execute_with(|| {
        PlatformPrice::<Test>::insert(MembershipLevel::Spark, SPARK_PRICE_FEN);
        assert_noop!(
            SquarePost::subscribe(
                RuntimeOrigin::signed(subscriber_account()),
                IssuerKey::Platform,
                spark(),
            ),
            Error::<Test>::PlatformNotBound
        );
    });
}

#[test]
fn subscribe_first_charge_failure_writes_no_state() {
    new_test_ext().execute_with(|| {
        setup_platform();
        // 余额不足账户首扣失败。
        assert!(SquarePost::subscribe(
            RuntimeOrigin::signed(poor_account()),
            IssuerKey::Platform,
            spark(),
        )
        .is_err());
        assert!(Subscriptions::<Test>::get((poor_account(), IssuerKey::Platform)).is_none());
    });
}

#[test]
fn cancel_writes_cancelled_and_keeps_record() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        assert_ok!(SquarePost::cancel(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
        ));
        let state = Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform))
            .expect("record kept after cancel");
        assert_eq!(state.status, SubscriptionStatus::Cancelled);
    });
}

#[test]
fn resubscribe_after_cancel_flips_active_without_recharge() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        assert_ok!(SquarePost::cancel(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
        ));
        let after_cancel = Balances::free_balance(subscriber_account());
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        // 续订不二次收费。
        assert_eq!(Balances::free_balance(subscriber_account()), after_cancel);
        let state =
            Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform)).unwrap();
        assert_eq!(state.status, SubscriptionStatus::Active);
    });
}

#[test]
fn charge_due_only_by_keeper() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        // 非 keeper 调用被拒。
        assert_noop!(
            SquarePost::charge_due(
                RuntimeOrigin::signed(subscriber_account()),
                subscriber_account(),
                IssuerKey::Platform,
                None,
            ),
            Error::<Test>::NotBillingKeeper
        );
    });
}

#[test]
fn charge_due_by_keeper_renews_and_transfers() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        let before = Balances::free_balance(subscriber_account());
        assert_ok!(SquarePost::charge_due(
            RuntimeOrigin::signed(keeper_account()),
            subscriber_account(),
            IssuerKey::Platform,
            None,
        ));
        // 续扣再转一个月费。
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            before - SPARK_PRICE_FEN
        );
        assert_eq!(
            Balances::free_balance(platform_fee_account()),
            SPARK_PRICE_FEN * 2
        );
    });
}

#[test]
fn charge_due_insufficient_balance_marks_past_due() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        // 把订阅者余额抽干到不足以再扣一档。
        let bal = Balances::free_balance(subscriber_account());
        let _ = Balances::withdraw(
            &subscriber_account(),
            bal - 1,
            frame_support::traits::WithdrawReasons::TRANSFER,
            frame_support::traits::ExistenceRequirement::AllowDeath,
        );
        let fee_before = Balances::free_balance(platform_fee_account());
        assert_ok!(SquarePost::charge_due(
            RuntimeOrigin::signed(keeper_account()),
            subscriber_account(),
            IssuerKey::Platform,
            None,
        ));
        // 欠费即停：无转账、翻 PastDue。
        assert_eq!(Balances::free_balance(platform_fee_account()), fee_before);
        let state =
            Subscriptions::<Test>::get((subscriber_account(), IssuerKey::Platform)).unwrap();
        assert_eq!(state.status, SubscriptionStatus::PastDue);
    });
}

// ---------------------------------------------------------------------------
// 创作者会员：平台会员门禁 / 全额转创作者 / 新价续扣
// ---------------------------------------------------------------------------

fn creator_issuer() -> IssuerKey<AccountId32> {
    IssuerKey::Creator(creator_account())
}

#[test]
fn creator_subscribe_rejected_when_creator_not_platform_member() {
    new_test_ext().execute_with(|| {
        // 创作者不是平台会员 → 谁都订不了他。
        assert_noop!(
            SquarePost::subscribe(
                RuntimeOrigin::signed(subscriber_account()),
                creator_issuer(),
                SubscriptionPlan::CreatorPrice(1_000),
            ),
            Error::<Test>::CreatorNotPlatformMember
        );
    });
}

#[test]
fn creator_subscribe_transfers_full_to_creator() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        let sub_before = Balances::free_balance(subscriber_account());
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            creator_issuer(),
            SubscriptionPlan::CreatorPrice(1_000),
        ));
        // 全额转创作者本人钱包。
        assert_eq!(
            Balances::free_balance(subscriber_account()),
            sub_before - 1_000
        );
        assert_eq!(Balances::free_balance(creator_account()), 1_000);
        let state = Subscriptions::<Test>::get((subscriber_account(), creator_issuer())).unwrap();
        assert_eq!(state.status, SubscriptionStatus::Active);
        assert_eq!(state.price_fen, 1_000);
    });
}

#[test]
fn creator_subscribe_zero_price_rejected() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        assert_noop!(
            SquarePost::subscribe(
                RuntimeOrigin::signed(subscriber_account()),
                creator_issuer(),
                SubscriptionPlan::CreatorPrice(0),
            ),
            Error::<Test>::ZeroPrice
        );
    });
}

#[test]
fn cannot_subscribe_self_as_creator() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(subscriber_account());
        assert_noop!(
            SquarePost::subscribe(
                RuntimeOrigin::signed(subscriber_account()),
                IssuerKey::Creator(subscriber_account()),
                SubscriptionPlan::CreatorPrice(1_000),
            ),
            Error::<Test>::CannotSubscribeSelf
        );
    });
}

#[test]
fn creator_charge_due_uses_keeper_supplied_new_price() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        BillingKeeper::<Test>::put(keeper_account());
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            creator_issuer(),
            SubscriptionPlan::CreatorPrice(1_000),
        ));
        // 创作者改价后 keeper 带入新价续扣。
        assert_ok!(SquarePost::charge_due(
            RuntimeOrigin::signed(keeper_account()),
            subscriber_account(),
            creator_issuer(),
            Some(1_500),
        ));
        assert_eq!(Balances::free_balance(creator_account()), 1_000 + 1_500);
        let state = Subscriptions::<Test>::get((subscriber_account(), creator_issuer())).unwrap();
        assert_eq!(state.price_fen, 1_500);
    });
}

#[test]
fn creator_charge_due_requires_price() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        BillingKeeper::<Test>::put(keeper_account());
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            creator_issuer(),
            SubscriptionPlan::CreatorPrice(1_000),
        ));
        assert_noop!(
            SquarePost::charge_due(
                RuntimeOrigin::signed(keeper_account()),
                subscriber_account(),
                creator_issuer(),
                None,
            ),
            Error::<Test>::CreatorPriceRequired
        );
    });
}

#[test]
fn platform_charge_due_rejects_keeper_supplied_price() {
    new_test_ext().execute_with(|| {
        setup_platform();
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            IssuerKey::Platform,
            spark(),
        ));
        // 平台价链上真源，不接受 keeper 带价。
        assert_noop!(
            SquarePost::charge_due(
                RuntimeOrigin::signed(keeper_account()),
                subscriber_account(),
                IssuerKey::Platform,
                Some(1),
            ),
            Error::<Test>::PlanIssuerMismatch
        );
    });
}

#[test]
fn creator_charge_due_insufficient_marks_past_due() {
    new_test_ext().execute_with(|| {
        set_active_platform_member(creator_account());
        BillingKeeper::<Test>::put(keeper_account());
        assert_ok!(SquarePost::subscribe(
            RuntimeOrigin::signed(subscriber_account()),
            creator_issuer(),
            SubscriptionPlan::CreatorPrice(1_000),
        ));
        // 抽干订阅者余额。
        let bal = Balances::free_balance(subscriber_account());
        let _ = Balances::withdraw(
            &subscriber_account(),
            bal - 1,
            frame_support::traits::WithdrawReasons::TRANSFER,
            frame_support::traits::ExistenceRequirement::AllowDeath,
        );
        let creator_before = Balances::free_balance(creator_account());
        assert_ok!(SquarePost::charge_due(
            RuntimeOrigin::signed(keeper_account()),
            subscriber_account(),
            creator_issuer(),
            Some(1_000),
        ));
        assert_eq!(Balances::free_balance(creator_account()), creator_before);
        let state = Subscriptions::<Test>::get((subscriber_account(), creator_issuer())).unwrap();
        assert_eq!(state.status, SubscriptionStatus::PastDue);
    });
}

// ---------------------------------------------------------------------------
// 金标 SCALE 向量：五端（链/App/BFF/CitizenWallet）逐字节对齐锚点。
// `SUBSCRIPTION_VECTORS_UPDATE=1 cargo test -p square-post subscription_scale_vectors` 回填 fixture；
// 默认只断言链端编码与 fixture 逐字节一致（类型/call 布局漂移即红）。
// ---------------------------------------------------------------------------

const VECTORS_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/subscription_scale_vectors.json"
);
const VECTORS_UPDATE_ENV: &str = "SUBSCRIPTION_VECTORS_UPDATE";

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// 全部跨端类型 / call 的 `(name, note, 链端计算 hex)`。
fn compute_scale_vectors() -> Vec<(&'static str, &'static str, String)> {
    use codec::Encode;
    let creator = AccountId32::new([2u8; 32]);
    let subscriber = AccountId32::new([1u8; 32]);
    let price: u128 = 599_900;
    let state_price: u128 = 5_999_900;
    let last: u64 = 1_700_000_000_000;

    vec![
        (
            "membership_level_freedom",
            "MembershipLevel::Freedom",
            hex_encode(&MembershipLevel::Freedom.encode()),
        ),
        (
            "membership_level_democracy",
            "MembershipLevel::Democracy",
            hex_encode(&MembershipLevel::Democracy.encode()),
        ),
        (
            "membership_level_spark",
            "MembershipLevel::Spark",
            hex_encode(&MembershipLevel::Spark.encode()),
        ),
        (
            "issuer_platform",
            "IssuerKey::Platform",
            hex_encode(&IssuerKey::<AccountId32>::Platform.encode()),
        ),
        (
            "issuer_creator",
            "IssuerKey::Creator(0x02*32)",
            hex_encode(&IssuerKey::Creator(creator.clone()).encode()),
        ),
        (
            "plan_level_spark",
            "SubscriptionPlan::Level(Spark)",
            hex_encode(&SubscriptionPlan::Level(MembershipLevel::Spark).encode()),
        ),
        (
            "plan_creator_price_599900",
            "SubscriptionPlan::CreatorPrice(599900)",
            hex_encode(&SubscriptionPlan::CreatorPrice(price).encode()),
        ),
        (
            "status_active",
            "SubscriptionStatus::Active",
            hex_encode(&SubscriptionStatus::Active.encode()),
        ),
        (
            "status_past_due",
            "SubscriptionStatus::PastDue",
            hex_encode(&SubscriptionStatus::PastDue.encode()),
        ),
        (
            "status_cancelled",
            "SubscriptionStatus::Cancelled",
            hex_encode(&SubscriptionStatus::Cancelled.encode()),
        ),
        (
            "state_platform_spark",
            "SubscriptionState{Level(Spark),5999900,1700000000000,Active}",
            hex_encode(
                &SubscriptionState {
                    plan: SubscriptionPlan::Level(MembershipLevel::Spark),
                    price_fen: state_price,
                    last_charged_at: last,
                    status: SubscriptionStatus::Active,
                }
                .encode(),
            ),
        ),
        (
            "state_creator",
            "SubscriptionState{CreatorPrice(599900),599900,1700000000000,Active}",
            hex_encode(
                &SubscriptionState {
                    plan: SubscriptionPlan::CreatorPrice(price),
                    price_fen: price,
                    last_charged_at: last,
                    status: SubscriptionStatus::Active,
                }
                .encode(),
            ),
        ),
        (
            "call_subscribe_platform_spark",
            "Call::subscribe{Platform, Level(Spark)}",
            hex_encode(
                &crate::pallet::Call::<Test>::subscribe {
                    issuer: IssuerKey::Platform,
                    plan: SubscriptionPlan::Level(MembershipLevel::Spark),
                }
                .encode(),
            ),
        ),
        (
            "call_subscribe_creator",
            "Call::subscribe{Creator(0x02*32), CreatorPrice(599900)}",
            hex_encode(
                &crate::pallet::Call::<Test>::subscribe {
                    issuer: IssuerKey::Creator(creator.clone()),
                    plan: SubscriptionPlan::CreatorPrice(price),
                }
                .encode(),
            ),
        ),
        (
            "call_cancel_platform",
            "Call::cancel{Platform}",
            hex_encode(
                &crate::pallet::Call::<Test>::cancel {
                    issuer: IssuerKey::Platform,
                }
                .encode(),
            ),
        ),
        (
            "call_charge_due_platform_none",
            "Call::charge_due{sub=0x01*32, Platform, None}",
            hex_encode(
                &crate::pallet::Call::<Test>::charge_due {
                    subscriber: subscriber.clone(),
                    issuer: IssuerKey::Platform,
                    amount: None,
                }
                .encode(),
            ),
        ),
        (
            "call_charge_due_creator_some",
            "Call::charge_due{sub=0x01*32, Creator(0x02*32), Some(599900)}",
            hex_encode(
                &crate::pallet::Call::<Test>::charge_due {
                    subscriber: subscriber.clone(),
                    issuer: IssuerKey::Creator(creator.clone()),
                    amount: Some(price),
                }
                .encode(),
            ),
        ),
        (
            "calldata_subscribe_platform_spark",
            "0x22(pallet=34) ++ Call::subscribe{Platform,Level(Spark)}",
            {
                let mut data: Vec<u8> = Vec::new();
                data.push(34u8);
                data.extend(
                    crate::pallet::Call::<Test>::subscribe {
                        issuer: IssuerKey::Platform,
                        plan: SubscriptionPlan::Level(MembershipLevel::Spark),
                    }
                    .encode(),
                );
                hex_encode(&data)
            },
        ),
    ]
}

#[test]
fn subscription_scale_vectors() {
    let computed = compute_scale_vectors();
    let update = std::env::var(VECTORS_UPDATE_ENV)
        .map(|v| v == "1")
        .unwrap_or(false);

    if update {
        let vectors: Vec<serde_json::Value> = computed
            .iter()
            .map(|(name, note, hex)| serde_json::json!({ "name": name, "note": note, "hex": hex }))
            .collect();
        let doc = serde_json::json!({ "pallet_index": 34, "vectors": vectors });
        let pretty = serde_json::to_string_pretty(&doc).expect("序列化 fixture 失败");
        std::fs::write(VECTORS_FIXTURE, format!("{pretty}\n"))
            .unwrap_or_else(|e| panic!("写回 fixture 失败 {VECTORS_FIXTURE}: {e}"));
        eprintln!("[subscription vectors] 已重算并写回 {VECTORS_FIXTURE}");
        return;
    }

    let raw = std::fs::read_to_string(VECTORS_FIXTURE).unwrap_or_else(|e| {
        panic!("读取金标 fixture 失败 {VECTORS_FIXTURE}: {e}（先跑 {VECTORS_UPDATE_ENV}=1 回填）")
    });
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("金标 fixture 非法 JSON");
    assert_eq!(
        doc["pallet_index"].as_u64(),
        Some(34),
        "fixture pallet_index 必须=34"
    );
    let arr = doc["vectors"].as_array().expect("vectors 必须是数组");
    assert_eq!(
        arr.len(),
        computed.len(),
        "fixture 向量条数与链端不一致（先跑 {VECTORS_UPDATE_ENV}=1 回填）"
    );
    for (name, _note, hex) in &computed {
        let entry = arr
            .iter()
            .find(|v| v["name"].as_str() == Some(name))
            .unwrap_or_else(|| panic!("金标 fixture 缺向量 {name}"));
        assert_eq!(
            entry["hex"].as_str(),
            Some(hex.as_str()),
            "向量 {name} 逐字节漂移（链端与 fixture 不一致）"
        );
    }
}
