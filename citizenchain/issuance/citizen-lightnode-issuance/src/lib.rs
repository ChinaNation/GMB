#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
    use frame_support::{pallet_prelude::*, traits::Currency, Blake2_128Concat};
    use scale_info::TypeInfo;
    use sp_runtime::traits::{SaturatedConversion, Zero};
    use sp_runtime::RuntimeDebug;
    use sfid_code_auth::{OnSfidBound, OnSfidBoundWeight};

    use primitives::citizen_const::{
        CITIZEN_LIGHTNODE_HIGH_REWARD, CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT,
        CITIZEN_LIGHTNODE_MAX_COUNT, CITIZEN_LIGHTNODE_NORMAL_REWARD,
        CITIZEN_LIGHTNODE_ONE_TIME_ONLY,
    };

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rewarded_count)]
    pub type RewardedCount<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn reward_claimed)]
    pub type RewardClaimed<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (), ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_rewarded)]
    pub type AccountRewarded<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (), ValueQuery>;

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
    pub enum SkipReason {
        DuplicateSfid,
        MaxCountReached,
        AccountAlreadyRewarded,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 中文注释：SFID 绑定成功后，认证发行模块执行一次奖励发放。
        CertificationRewardIssued {
            who: T::AccountId,
            sfid_hash: T::Hash,
            reward: BalanceOf<T>,
        },
        CertificationRewardSkipped {
            who: T::AccountId,
            sfid_hash: T::Hash,
            reason: SkipReason,
        },
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    impl<T: Config> Pallet<T> {
        /// 调用方在 weight 宏中引用此值。
        pub fn on_sfid_bound_weight() -> Weight {
            // 最重成功路径: RewardClaimed + AccountRewarded + RewardedCount + Balances::deposit_creating。
            T::DbWeight::get().reads_writes(5, 5)
        }

        fn try_issue_certification_reward(
            who: &T::AccountId,
            sfid_hash: T::Hash,
        ) -> Result<BalanceOf<T>, SkipReason> {
            if CITIZEN_LIGHTNODE_ONE_TIME_ONLY && RewardClaimed::<T>::contains_key(sfid_hash) {
                return Err(SkipReason::DuplicateSfid);
            }

            if CITIZEN_LIGHTNODE_ONE_TIME_ONLY && AccountRewarded::<T>::contains_key(who) {
                return Err(SkipReason::AccountAlreadyRewarded);
            }

            let rewarded_count = RewardedCount::<T>::get();
            if rewarded_count >= CITIZEN_LIGHTNODE_MAX_COUNT {
                return Err(SkipReason::MaxCountReached);
            }

            let reward_amount = if rewarded_count < CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT {
                CITIZEN_LIGHTNODE_HIGH_REWARD
            } else {
                CITIZEN_LIGHTNODE_NORMAL_REWARD
            };

            let reward: BalanceOf<T> = reward_amount.saturated_into();
            let _imbalance = T::Currency::deposit_creating(who, reward);

            RewardedCount::<T>::put(rewarded_count.saturating_add(1));
            if CITIZEN_LIGHTNODE_ONE_TIME_ONLY {
                RewardClaimed::<T>::insert(sfid_hash, ());
                AccountRewarded::<T>::insert(who, ());
            }

            Ok(reward)
        }
    }

    impl<T: Config> OnSfidBound<T::AccountId, T::Hash> for Pallet<T> {
        fn on_sfid_bound(who: &T::AccountId, sfid_hash: T::Hash) {
            match Self::try_issue_certification_reward(who, sfid_hash) {
                // 中文注释：仅在实际发放奖励时发事件，避免 reward=0 造成“已发奖”误解。
                Ok(reward) if !reward.is_zero() => {
                    Self::deposit_event(Event::<T>::CertificationRewardIssued {
                        who: who.clone(),
                        sfid_hash,
                        reward,
                    });
                }
                Ok(_) => {}
                Err(reason) => {
                    Self::deposit_event(Event::<T>::CertificationRewardSkipped {
                        who: who.clone(),
                        sfid_hash,
                        reason,
                    });
                }
            }
        }
    }

    impl<T: Config> OnSfidBoundWeight for Pallet<T> {
        fn on_sfid_bound_weight() -> Weight {
            Pallet::<T>::on_sfid_bound_weight()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        derive_impl,
        traits::{ConstU128, ConstU32, VariantCountOf},
    };
    use frame_system as system;
    use primitives::citizen_const::{
        CITIZEN_LIGHTNODE_HIGH_REWARD, CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT,
        CITIZEN_LIGHTNODE_MAX_COUNT, CITIZEN_LIGHTNODE_NORMAL_REWARD,
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
        pub type CitizenLightnodeIssuance = super;
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
    fn on_sfid_bound_issues_reward() {
        new_test_ext().execute_with(|| {
            let sfid_hash = <Test as frame_system::Config>::Hashing::hash(b"sfid-a");
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, sfid_hash);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(RewardedCount::<Test>::get(), 1);
            assert!(RewardClaimed::<Test>::contains_key(sfid_hash));
            assert!(AccountRewarded::<Test>::contains_key(1));
            System::assert_last_event(
                RuntimeEvent::CitizenLightnodeIssuance(Event::<Test>::CertificationRewardIssued {
                    who: 1,
                    sfid_hash,
                    reward: CITIZEN_LIGHTNODE_HIGH_REWARD,
                }),
            );
        });
    }

    #[test]
    fn max_cap_stops_reward() {
        new_test_ext().execute_with(|| {
            RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_MAX_COUNT);
            let sfid_hash = <Test as frame_system::Config>::Hashing::hash(b"sfid-cap");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, sfid_hash);

            assert_eq!(Balances::free_balance(1), 0);
            assert_eq!(RewardedCount::<Test>::get(), CITIZEN_LIGHTNODE_MAX_COUNT);
            System::assert_last_event(
                RuntimeEvent::CitizenLightnodeIssuance(Event::<Test>::CertificationRewardSkipped {
                    who: 1,
                    sfid_hash,
                    reason: SkipReason::MaxCountReached,
                }),
            );
        });
    }

    #[test]
    fn same_sfid_only_rewards_once() {
        new_test_ext().execute_with(|| {
            let sfid_hash = <Test as frame_system::Config>::Hashing::hash(b"sfid-repeat");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, sfid_hash);
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, sfid_hash);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(RewardedCount::<Test>::get(), 1);
            System::assert_last_event(
                RuntimeEvent::CitizenLightnodeIssuance(Event::<Test>::CertificationRewardSkipped {
                    who: 1,
                    sfid_hash,
                    reason: SkipReason::DuplicateSfid,
                }),
            );
        });
    }

    #[test]
    fn boundary_switches_to_normal_reward_at_high_reward_count() {
        new_test_ext().execute_with(|| {
            RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT);
            let sfid_hash = <Test as frame_system::Config>::Hashing::hash(b"sfid-boundary");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, sfid_hash);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_NORMAL_REWARD);
            assert_eq!(
                RewardedCount::<Test>::get(),
                CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT.saturating_add(1)
            );
        });
    }

    #[test]
    fn high_reward_count_minus_one_still_gets_high_reward() {
        new_test_ext().execute_with(|| {
            RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT.saturating_sub(1));
            let sfid_hash = <Test as frame_system::Config>::Hashing::hash(b"sfid-high-minus-1");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, sfid_hash);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            System::assert_last_event(
                RuntimeEvent::CitizenLightnodeIssuance(Event::<Test>::CertificationRewardIssued {
                    who: 1,
                    sfid_hash,
                    reward: CITIZEN_LIGHTNODE_HIGH_REWARD,
                }),
            );
        });
    }

    #[test]
    fn different_accounts_and_sfids_reward_independently() {
        new_test_ext().execute_with(|| {
            let sfid_hash_a = <Test as frame_system::Config>::Hashing::hash(b"sfid-a-2");
            let sfid_hash_b = <Test as frame_system::Config>::Hashing::hash(b"sfid-b-2");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, sfid_hash_a);
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&2, sfid_hash_b);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(Balances::free_balance(2), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(RewardedCount::<Test>::get(), 2);
        });
    }

    #[test]
    fn same_account_different_sfids_only_rewards_once() {
        new_test_ext().execute_with(|| {
            let sfid_hash_a = <Test as frame_system::Config>::Hashing::hash(b"sfid-acc-a");
            let sfid_hash_b = <Test as frame_system::Config>::Hashing::hash(b"sfid-acc-b");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, sfid_hash_a);
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, sfid_hash_b);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(RewardedCount::<Test>::get(), 1);
            System::assert_last_event(
                RuntimeEvent::CitizenLightnodeIssuance(Event::<Test>::CertificationRewardSkipped {
                    who: 1,
                    sfid_hash: sfid_hash_b,
                    reason: SkipReason::AccountAlreadyRewarded,
                }),
            );
        });
    }
}
