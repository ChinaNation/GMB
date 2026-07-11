#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32};
use frame_system as system;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};

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
    pub type SquarePost = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
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

impl crate::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CitizenIdentity = TestCitizenIdentity;
    type MaxSquarePostIdLen = ConstU32<64>;
    type MaxSquareCidNumberLen = ConstU32<32>;
    type MaxSquareStorageReceiptIdLen = ConstU32<96>;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

fn verified_account() -> AccountId32 {
    AccountId32::new([1u8; 32])
}

fn visitor_account() -> AccountId32 {
    AccountId32::new([2u8; 32])
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
        assert_eq!(stored.post_category, SquarePostCategory::Normal);
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
fn campaign_post_records_cid_from_chain_identity() {
    new_test_ext().execute_with(|| {
        assert_ok!(SquarePost::publish_post(
            RuntimeOrigin::signed(verified_account()),
            post_id(b"sqp_campaign_ok"),
            SquarePostCategory::Campaign,
            content_hash(),
            receipt(),
            1_893_456_000_000,
        ));

        let stored = SquarePosts::<Test>::get(
            crate::pallet::PostIdOf::<Test>::try_from(b"sqp_campaign_ok".to_vec())
                .expect("post id fits"),
        )
        .expect("post should be indexed");
        assert_eq!(
            stored.cid_number.map(|value| value.to_vec()),
            Some(b"GD001-CTZN1-000000001-2026".to_vec())
        );
        assert_eq!(
            PublishedPostCountByAccount::<Test>::get(verified_account()),
            1
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
