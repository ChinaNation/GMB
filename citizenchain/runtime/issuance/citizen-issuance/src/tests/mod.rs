#![cfg(test)]

use super::*;
use frame_support::{
    derive_impl,
    traits::{ConstU128, ConstU32, VariantCountOf},
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
fn on_cid_bound_issues_reward() {
    new_test_ext().execute_with(|| {
        let binding_id = <Test as frame_system::Config>::Hashing::hash(b"cid-a");
        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id);

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(RewardedCount::<Test>::get(), 1);
        assert!(RewardClaimed::<Test>::contains_key(binding_id));
        assert!(AccountRewarded::<Test>::contains_key(1));
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardIssued {
                who: 1,
                binding_id,
                reward: CITIZEN_ISSUANCE_HIGH_REWARD,
            },
        ));
    });
}

#[test]
fn max_cap_stops_reward() {
    new_test_ext().execute_with(|| {
        RewardedCount::<Test>::put(CITIZEN_ISSUANCE_MAX_COUNT);
        let binding_id = <Test as frame_system::Config>::Hashing::hash(b"cid-cap");

        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id);

        assert_eq!(Balances::free_balance(1), 0);
        assert_eq!(RewardedCount::<Test>::get(), CITIZEN_ISSUANCE_MAX_COUNT);
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 1,
                binding_id,
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
        let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"cid-last-slot");
        let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"cid-over-cap");

        RewardedCount::<Test>::put(CITIZEN_ISSUANCE_MAX_COUNT.saturating_sub(1));

        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id_a);
        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&2, binding_id_b);

        assert_eq!(Balances::free_balance(1), last_reward_amount);
        assert_eq!(Balances::free_balance(2), 0);
        assert_eq!(RewardedCount::<Test>::get(), CITIZEN_ISSUANCE_MAX_COUNT);
        assert!(RewardClaimed::<Test>::contains_key(binding_id_a));
        assert!(!RewardClaimed::<Test>::contains_key(binding_id_b));
        assert!(AccountRewarded::<Test>::contains_key(1));
        assert!(!AccountRewarded::<Test>::contains_key(2));
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 2,
                binding_id: binding_id_b,
                reason: SkipReason::MaxCountReached,
            },
        ));
    });
}

#[test]
fn same_cid_only_rewards_once() {
    new_test_ext().execute_with(|| {
        let binding_id = <Test as frame_system::Config>::Hashing::hash(b"cid-repeat");

        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id);
        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id);

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(RewardedCount::<Test>::get(), 1);
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 1,
                binding_id,
                reason: SkipReason::DuplicateBindingId,
            },
        ));
    });
}

#[test]
fn consecutive_rewards_switch_from_high_to_normal_in_same_block() {
    new_test_ext().execute_with(|| {
        let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"cid-tier-a");
        let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"cid-tier-b");

        RewardedCount::<Test>::put(CITIZEN_ISSUANCE_HIGH_REWARD_COUNT.saturating_sub(1));

        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id_a);
        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&2, binding_id_b);

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(Balances::free_balance(2), CITIZEN_ISSUANCE_NORMAL_REWARD);
        assert_eq!(
            RewardedCount::<Test>::get(),
            CITIZEN_ISSUANCE_HIGH_REWARD_COUNT.saturating_add(1)
        );

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
                    binding_id: binding_id_a,
                    reward: CITIZEN_ISSUANCE_HIGH_REWARD,
                },
                Event::<Test>::CertificationRewardIssued {
                    who: 2,
                    binding_id: binding_id_b,
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
        let binding_id = <Test as frame_system::Config>::Hashing::hash(b"cid-boundary");

        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id);

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
        let binding_id = <Test as frame_system::Config>::Hashing::hash(b"cid-high-minus-1");

        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id);

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardIssued {
                who: 1,
                binding_id,
                reward: CITIZEN_ISSUANCE_HIGH_REWARD,
            },
        ));
    });
}

#[test]
fn same_account_second_cid_is_not_marked_reward_claimed() {
    new_test_ext().execute_with(|| {
        let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"cid-claim-a");
        let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"cid-claim-b");

        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id_a);
        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id_b);

        assert!(RewardClaimed::<Test>::contains_key(binding_id_a));
        assert!(!RewardClaimed::<Test>::contains_key(binding_id_b));
        assert!(AccountRewarded::<Test>::contains_key(1));
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 1,
                binding_id: binding_id_b,
                reason: SkipReason::AccountAlreadyRewarded,
            },
        ));
    });
}

#[test]
fn different_accounts_and_cids_reward_independently() {
    new_test_ext().execute_with(|| {
        let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"cid-a-2");
        let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"cid-b-2");

        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id_a);
        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&2, binding_id_b);

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(Balances::free_balance(2), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(RewardedCount::<Test>::get(), 2);
    });
}

#[test]
fn same_account_different_cids_only_rewards_once() {
    new_test_ext().execute_with(|| {
        let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"cid-acc-a");
        let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"cid-acc-b");

        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id_a);
        <CitizenIssuance as cid_system::OnCidBound<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::on_cid_bound(&1, binding_id_b);

        assert_eq!(Balances::free_balance(1), CITIZEN_ISSUANCE_HIGH_REWARD);
        assert_eq!(RewardedCount::<Test>::get(), 1);
        System::assert_last_event(RuntimeEvent::CitizenIssuance(
            Event::<Test>::CertificationRewardSkipped {
                who: 1,
                binding_id: binding_id_b,
                reason: SkipReason::AccountAlreadyRewarded,
            },
        ));
    });
}
