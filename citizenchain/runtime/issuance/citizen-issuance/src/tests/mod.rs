#![cfg(test)]

use super::*;
use frame_support::{
    derive_impl,
    traits::{ConstU128, ConstU32, Hooks, VariantCountOf},
};
use frame_system as system;
use primitives::citizen_const::{
    CITIZEN_ISSUANCE_HIGH_REWARD, CITIZEN_ISSUANCE_HIGH_REWARD_COUNT, CITIZEN_ISSUANCE_MAX_COUNT,
    CITIZEN_ISSUANCE_NORMAL_REWARD,
};
use sp_runtime::{
    traits::{Hash, IdentityLookup},
    BuildStorage,
};

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
    pub type CitizenIssuance = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = u64;
    type AccountData = pallet_balances::AccountData<u128>;
    type Lookup = IdentityLookup<Self::AccountId>;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<0>;
    type MaxReserves = ConstU32<0>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("frame system genesis storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(10);
    });
    ext
}

#[test]
fn voting_identity_registered_issues_reward() {
    new_test_ext().execute_with(|| {
        let cid_number_hash = notify_voting_identity_registered(1, &citizen_cid_number("0001"));

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(RewardedCount::<Test>::get(), 1);
        assert!(IdentityRewardClaimed::<Test>::contains_key(cid_number_hash));
        assert!(AccountRewarded::<Test>::contains_key(1));
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardIssued {
                who: 1,
                cid_number_hash,
                reward: CITIZEN_ISSUANCE_HIGH_REWARD,
            },
        ));
    });
}

#[test]
fn max_cap_stops_reward() {
    new_test_ext().execute_with(|| {
        RewardedCount::<Test>::put(CITIZEN_ISSUANCE_MAX_COUNT);
        let cid_number_hash = notify_voting_identity_registered(1, &citizen_cid_number("CAP"));

        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(RewardedCount::<Test>::get(), CITIZEN_ISSUANCE_MAX_COUNT);
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 1,
                cid_number_hash,
                reason: SkipReason::MaxCountReached,
            },
        ));
    });
}

#[test]
fn max_count_minus_one_allows_last_reward_then_rejects_next() {
    new_test_ext().execute_with(|| {
        let last_reward_amount =
            if CITIZEN_ISSUANCE_MAX_COUNT.saturating_sub(1) < CITIZEN_ISSUANCE_HIGH_REWARD_COUNT {
                CITIZEN_ISSUANCE_HIGH_REWARD
            } else {
                CITIZEN_ISSUANCE_NORMAL_REWARD
            };

        RewardedCount::<Test>::put(CITIZEN_ISSUANCE_MAX_COUNT.saturating_sub(1));
        let cid_number_hash_a = notify_voting_identity_registered(1, &citizen_cid_number("LAST"));
        let cid_number_hash_b = notify_voting_identity_registered(2, &citizen_cid_number("OVER"));

        assert_eq!(Balances::free_balance(1), last_reward_amount);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(RewardedCount::<Test>::get(), CITIZEN_ISSUANCE_MAX_COUNT);
        assert!(IdentityRewardClaimed::<Test>::contains_key(
            cid_number_hash_a
        ));
        assert!(!IdentityRewardClaimed::<Test>::contains_key(
            cid_number_hash_b
        ));
        assert!(AccountRewarded::<Test>::contains_key(1));
        assert!(!AccountRewarded::<Test>::contains_key(2));
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 2,
                cid_number_hash: cid_number_hash_b,
                reason: SkipReason::MaxCountReached,
            },
        ));
    });
}

#[test]
fn same_citizen_identity_only_rewards_once() {
    new_test_ext().execute_with(|| {
        let cid_number_hash = notify_voting_identity_registered(1, &citizen_cid_number("REPEAT"));
        notify_voting_identity_registered(1, &citizen_cid_number("REPEAT"));

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(RewardedCount::<Test>::get(), 1);
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 1,
                cid_number_hash,
                reason: SkipReason::DuplicateCitizenIdentity,
            },
        ));
    });
}

#[test]
fn consecutive_rewards_switch_from_high_to_normal_in_same_block() {
    new_test_ext().execute_with(|| {
        RewardedCount::<Test>::put(CITIZEN_ISSUANCE_HIGH_REWARD_COUNT.saturating_sub(1));

        let cid_number_hash_a = queue_voting_identity_registered(1, &citizen_cid_number("TIER-A"));
        let cid_number_hash_b = queue_voting_identity_registered(2, &citizen_cid_number("TIER-B"));
        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(PendingRewardCount::<Test>::get(), 2);

        CitizenIssuance::on_finalize(System::block_number());

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(Balances::free_balance(2), CITIZEN_ISSUANCE_NORMAL_REWARD);
        assert_eq!(
            RewardedCount::<Test>::get(),
            CITIZEN_ISSUANCE_HIGH_REWARD_COUNT.saturating_add(1)
        );
        assert_eq!(PendingRewardCount::<Test>::get(), 0);
        assert!(!PendingRewards::<Test>::contains_key(0));
        assert!(!PendingRewards::<Test>::contains_key(1));
        assert!(!PendingIdentityRewardClaimed::<Test>::contains_key(
            cid_number_hash_a
        ));
        assert!(!PendingAccountRewarded::<Test>::contains_key(1));

        let issuance_events: Vec<_> = System::events()
            .into_iter()
            .filter_map(|record| match record.event {
                RuntimeEvent::CitizenIssuance(event) => Some(event),
                _ => None,
            })
            .collect();

        assert_eq!(
            issuance_events,
            vec![
                Event::<Test>::CertificationRewardIssued {
                    who: 1,
                    cid_number_hash: cid_number_hash_a,
                    reward: CITIZEN_ISSUANCE_HIGH_REWARD,
                },
                Event::<Test>::CertificationRewardIssued {
                    who: 2,
                    cid_number_hash: cid_number_hash_b,
                    reward: CITIZEN_ISSUANCE_NORMAL_REWARD,
                },
            ]
        );
    });
}

#[test]
fn boundary_switches_to_normal_reward_at_high_reward_count() {
    new_test_ext().execute_with(|| {
        RewardedCount::<Test>::put(CITIZEN_ISSUANCE_HIGH_REWARD_COUNT);

        notify_voting_identity_registered(1, &citizen_cid_number("BOUNDARY"));

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_NORMAL_REWARD);
        assert_eq!(
            RewardedCount::<Test>::get(),
            CITIZEN_ISSUANCE_HIGH_REWARD_COUNT.saturating_add(1)
        );
    });
}

#[test]
fn high_reward_count_minus_one_still_gets_high_reward() {
    new_test_ext().execute_with(|| {
        RewardedCount::<Test>::put(CITIZEN_ISSUANCE_HIGH_REWARD_COUNT.saturating_sub(1));
        let cid_number_hash =
            notify_voting_identity_registered(1, &citizen_cid_number("HIGH-MINUS-1"));

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardIssued {
                who: 1,
                cid_number_hash,
                reward: CITIZEN_ISSUANCE_HIGH_REWARD,
            },
        ));
    });
}

#[test]
fn same_account_second_citizen_identity_is_not_marked_reward_claimed() {
    new_test_ext().execute_with(|| {
        let cid_number_hash_a =
            notify_voting_identity_registered(1, &citizen_cid_number("CLAIM-A"));
        let cid_number_hash_b =
            notify_voting_identity_registered(1, &citizen_cid_number("CLAIM-B"));

        assert!(IdentityRewardClaimed::<Test>::contains_key(
            cid_number_hash_a
        ));
        assert!(!IdentityRewardClaimed::<Test>::contains_key(
            cid_number_hash_b
        ));
        assert!(AccountRewarded::<Test>::contains_key(1));
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 1,
                cid_number_hash: cid_number_hash_b,
                reason: SkipReason::AccountAlreadyRewarded,
            },
        ));
    });
}

#[test]
fn different_accounts_and_citizen_identities_reward_independently() {
    new_test_ext().execute_with(|| {
        notify_voting_identity_registered(1, &citizen_cid_number("A-2"));
        notify_voting_identity_registered(2, &citizen_cid_number("B-2"));

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(Balances::free_balance(2), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(RewardedCount::<Test>::get(), 2);
    });
}

#[test]
fn same_account_different_citizen_identities_only_rewards_once() {
    new_test_ext().execute_with(|| {
        notify_voting_identity_registered(1, &citizen_cid_number("ACC-A"));
        let cid_number_hash_b = notify_voting_identity_registered(1, &citizen_cid_number("ACC-B"));

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(RewardedCount::<Test>::get(), 1);
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 1,
                cid_number_hash: cid_number_hash_b,
                reason: SkipReason::AccountAlreadyRewarded,
            },
        ));
    });
}

#[test]
fn same_block_pending_tables_prevent_duplicate_account_reward() {
    new_test_ext().execute_with(|| {
        let first_hash = queue_voting_identity_registered(1, &citizen_cid_number("PENDING-A"));
        let second_hash = queue_voting_identity_registered(1, &citizen_cid_number("PENDING-B"));

        assert_eq!(PendingRewardCount::<Test>::get(), 1);
        assert!(PendingIdentityRewardClaimed::<Test>::contains_key(
            first_hash
        ));
        assert!(!PendingIdentityRewardClaimed::<Test>::contains_key(
            second_hash
        ));
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 1,
                cid_number_hash: second_hash,
                reason: SkipReason::AccountAlreadyRewarded,
            },
        ));

        CitizenIssuance::on_finalize(System::block_number());
        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(RewardedCount::<Test>::get(), 1);
        assert_eq!(PendingRewardCount::<Test>::get(), 0);
        assert!(!PendingIdentityRewardClaimed::<Test>::contains_key(
            first_hash
        ));
        assert!(!PendingAccountRewarded::<Test>::contains_key(1));
    });
}

/// 按 tag 生成真实规则公民 CID 号(格式/校验和/机构码全合规)。
fn citizen_cid_number(tag: &str) -> Vec<u8> {
    primitives::cid::generator::generate_cid_number(
        primitives::cid::generator::GenerateCidNumberInput {
            account_pubkey: tag,
            p1: "1",
            province_code: "GD",
            province_name: "广东省",
            city_code: "001",
            city_name: "荔湾市",
            year: "2026",
            institution: "CTZN",
        },
    )
    .expect("citizen cid should generate")
    .into_bytes()
}

fn notify_voting_identity_registered(
    who: u64,
    cid_number: &[u8],
) -> <Test as frame_system::Config>::Hash {
    let cid_number_hash = queue_voting_identity_registered(who, cid_number);
    CitizenIssuance::on_finalize(System::block_number());
    cid_number_hash
}

fn queue_voting_identity_registered(
    who: u64,
    cid_number: &[u8],
) -> <Test as frame_system::Config>::Hash {
    let cid_number_hash = <Test as frame_system::Config>::Hashing::hash(cid_number);
    <CitizenIssuance as citizen_identity::OnVotingIdentityRegistered<u64>>::on_voting_identity_registered(
        &who,
        cid_number,
    );
    cid_number_hash
}
