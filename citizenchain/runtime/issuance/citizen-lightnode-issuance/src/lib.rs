//! # 公民轻节点认证奖励发行模块 (citizen-lightnode-issuance)
//!
//! 本模块在 SFID 绑定成功时，通过 `OnSfidBound` 回调自动发放一次性认证奖励。
//!
//! ## 核心规则
//! - 双重防重：按 `binding_id` + 按账户，防止同一身份或同一账户重复领奖。
//! - 阶梯奖励：前 `CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT` 人获高额奖励，之后降为常规奖励。
//! - 总量硬顶：累计发放人数达到 `CITIZEN_LIGHTNODE_MAX_COUNT` 后停止发放。
//! - 本模块不暴露任何 extrinsic，所有触发均来自上游回调。

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, Imbalance},
        Blake2_128Concat,
    };
    use scale_info::TypeInfo;
    use sfid_code_auth::{OnSfidBound, OnSfidBoundWeight};
    use sp_runtime::traits::{SaturatedConversion, Zero};
    use sp_runtime::RuntimeDebug;

    use crate::weights::WeightInfo;
    use primitives::citizen_const::{
        CITIZEN_LIGHTNODE_HIGH_REWARD, CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT,
        CITIZEN_LIGHTNODE_MAX_COUNT, CITIZEN_LIGHTNODE_NORMAL_REWARD,
        CITIZEN_LIGHTNODE_ONE_TIME_ONLY,
    };

    // 中文注释：链上规则强制“一次性奖励”，禁止通过配置关闭该约束。
    const _: () = assert!(
        CITIZEN_LIGHTNODE_ONE_TIME_ONLY,
        "CITIZEN_LIGHTNODE_ONE_TIME_ONLY must be true"
    );

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rewarded_count)]
    /// 中文注释：全局累计已领奖人数，用于控制总发放上限与奖励档位切换。
    pub type RewardedCount<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn reward_claimed)]
    /// 中文注释：按 binding_id 维度防重，确保同一身份标识不会重复领取奖励。
    pub type RewardClaimed<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (), ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_rewarded)]
    /// 中文注释：按账户维度再做一次防重，避免同一账户换绑 SFID 后再次领奖。
    pub type AccountRewarded<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (), ValueQuery>;

    /// 中文注释：描述奖励被跳过的具体原因，用于链上事件记录和前端展示。
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
        /// 中文注释：同一 binding_id 已经领取过奖励。
        DuplicateBindingId,
        /// 中文注释：全局累计发放人数已达上限。
        MaxCountReached,
        /// 中文注释：该账户已通过其他 SFID 领取过奖励，换绑不可再领。
        AccountAlreadyRewarded,
        /// 中文注释：奖励常量配置为零（不应出现，属防御性检查）。
        ZeroRewardConfigured,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 中文注释：SFID 绑定成功后，认证发行模块执行一次奖励发放。
        CertificationRewardIssued {
            who: T::AccountId,
            binding_id: T::Hash,
            reward: BalanceOf<T>,
        },
        /// 中文注释：奖励因重复、超限等原因被跳过时触发，reason 字段说明具体原因。
        CertificationRewardSkipped {
            who: T::AccountId,
            binding_id: T::Hash,
            reason: SkipReason,
        },
    }

    /// 中文注释：本模块不暴露 extrinsic，所有逻辑通过回调触发，因此无需定义错误类型。
    #[pallet::error]
    pub enum Error<T> {}

    /// 中文注释：本模块不暴露 extrinsic，奖励发放由 OnSfidBound 回调驱动，无需用户直接调用。
    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    impl<T: Config> Pallet<T> {
        /// 调用方在 weight 宏中引用此值。
        pub fn on_sfid_bound_weight() -> Weight {
            // 中文注释：上游 bind_sfid 在申报 weight 时会叠加这里的回调预算。
            T::WeightInfo::on_sfid_bound()
        }

        fn try_issue_certification_reward(
            who: &T::AccountId,
            binding_id: T::Hash,
        ) -> Result<BalanceOf<T>, SkipReason> {
            // 中文注释：先查 binding_id，再查账户，优先返回更贴近业务语义的跳过原因。
            if RewardClaimed::<T>::contains_key(binding_id) {
                return Err(SkipReason::DuplicateBindingId);
            }

            if AccountRewarded::<T>::contains_key(who) {
                return Err(SkipReason::AccountAlreadyRewarded);
            }

            let rewarded_count = RewardedCount::<T>::get();
            // 中文注释：总人数达到上限后直接跳过，不再尝试铸币或写入任何领奖标记。
            if rewarded_count >= CITIZEN_LIGHTNODE_MAX_COUNT {
                return Err(SkipReason::MaxCountReached);
            }

            // 中文注释：奖励档位完全由全局累计人数决定，避免链下参与者各自推导口径不一致。
            let reward_amount = if rewarded_count < CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT {
                CITIZEN_LIGHTNODE_HIGH_REWARD
            } else {
                CITIZEN_LIGHTNODE_NORMAL_REWARD
            };

            let reward: BalanceOf<T> = reward_amount.saturated_into();
            debug_assert!(
                !reward.is_zero(),
                "citizen lightnode reward constants must stay greater than zero"
            );
            if reward.is_zero() {
                return Err(SkipReason::ZeroRewardConfigured);
            }

            // 中文注释：这里有意通过 deposit_creating 主动增发，并丢弃返回的 PositiveImbalance；
            // 奖励发行本身就是本模块的职责，不需要再将该发行凭证向外传递。
            let imbalance = T::Currency::deposit_creating(who, reward);
            debug_assert_eq!(
                imbalance.peek(),
                reward,
                "deposit_creating must return full reward"
            );

            // 中文注释：只有铸币成功进入账本后，才推进累计人数并写入双重防重标记。
            RewardedCount::<T>::put(rewarded_count.saturating_add(1));
            RewardClaimed::<T>::insert(binding_id, ());
            AccountRewarded::<T>::insert(who, ());

            Ok(reward)
        }
    }

    /// 中文注释：实现 sfid-code-auth 的绑定回调，在 SFID 绑定成功后自动尝试发放认证奖励。
    impl<T: Config> OnSfidBound<T::AccountId, T::Hash> for Pallet<T> {
        fn on_sfid_bound(who: &T::AccountId, binding_id: T::Hash) {
            match Self::try_issue_certification_reward(who, binding_id) {
                Ok(reward) => {
                    Self::deposit_event(Event::<T>::CertificationRewardIssued {
                        who: who.clone(),
                        binding_id,
                        reward,
                    });
                }
                Err(reason) => {
                    Self::deposit_event(Event::<T>::CertificationRewardSkipped {
                        who: who.clone(),
                        binding_id,
                        reason,
                    });
                }
            }
        }
    }

    /// 中文注释：向上游提供回调的 weight 预算，供 bind_sfid 在申报交易权重时叠加。
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
    fn on_sfid_bound_issues_reward() {
        new_test_ext().execute_with(|| {
            let binding_id = <Test as frame_system::Config>::Hashing::hash(b"sfid-a");
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(RewardedCount::<Test>::get(), 1);
            assert!(RewardClaimed::<Test>::contains_key(binding_id));
            assert!(AccountRewarded::<Test>::contains_key(1));
            System::assert_last_event(RuntimeEvent::CitizenLightnodeIssuance(
                Event::<Test>::CertificationRewardIssued {
                    who: 1,
                    binding_id,
                    reward: CITIZEN_LIGHTNODE_HIGH_REWARD,
                },
            ));
        });
    }

    #[test]
    fn max_cap_stops_reward() {
        new_test_ext().execute_with(|| {
            RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_MAX_COUNT);
            let binding_id = <Test as frame_system::Config>::Hashing::hash(b"sfid-cap");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id);

            assert_eq!(Balances::free_balance(1), 0);
            assert_eq!(RewardedCount::<Test>::get(), CITIZEN_LIGHTNODE_MAX_COUNT);
            System::assert_last_event(RuntimeEvent::CitizenLightnodeIssuance(
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
            let last_reward_amount = if CITIZEN_LIGHTNODE_MAX_COUNT.saturating_sub(1)
                < CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT
            {
                CITIZEN_LIGHTNODE_HIGH_REWARD
            } else {
                CITIZEN_LIGHTNODE_NORMAL_REWARD
            };
            let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"sfid-last-slot");
            let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"sfid-over-cap");

            RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_MAX_COUNT.saturating_sub(1));

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id_a);
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&2, binding_id_b);

            assert_eq!(Balances::free_balance(1), last_reward_amount);
            assert_eq!(Balances::free_balance(2), 0);
            assert_eq!(RewardedCount::<Test>::get(), CITIZEN_LIGHTNODE_MAX_COUNT);
            assert!(RewardClaimed::<Test>::contains_key(binding_id_a));
            assert!(!RewardClaimed::<Test>::contains_key(binding_id_b));
            assert!(AccountRewarded::<Test>::contains_key(1));
            assert!(!AccountRewarded::<Test>::contains_key(2));
            System::assert_last_event(RuntimeEvent::CitizenLightnodeIssuance(
                Event::<Test>::CertificationRewardSkipped {
                    who: 2,
                    binding_id: binding_id_b,
                    reason: SkipReason::MaxCountReached,
                },
            ));
        });
    }

    #[test]
    fn same_sfid_only_rewards_once() {
        new_test_ext().execute_with(|| {
            let binding_id = <Test as frame_system::Config>::Hashing::hash(b"sfid-repeat");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id);
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(RewardedCount::<Test>::get(), 1);
            System::assert_last_event(RuntimeEvent::CitizenLightnodeIssuance(
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
            let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"sfid-tier-a");
            let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"sfid-tier-b");

            RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT.saturating_sub(1));

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id_a);
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&2, binding_id_b);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(Balances::free_balance(2), CITIZEN_LIGHTNODE_NORMAL_REWARD);
            assert_eq!(
                RewardedCount::<Test>::get(),
                CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT.saturating_add(1)
            );

            let issuance_events: Vec<_> = System::events()
                .into_iter()
                .filter_map(|record| match record.event {
                    RuntimeEvent::CitizenLightnodeIssuance(event) => Some(event),
                    _ => None,
                })
                .collect();

            assert_eq!(
                issuance_events,
                vec![
                    Event::<Test>::CertificationRewardIssued {
                        who: 1,
                        binding_id: binding_id_a,
                        reward: CITIZEN_LIGHTNODE_HIGH_REWARD,
                    },
                    Event::<Test>::CertificationRewardIssued {
                        who: 2,
                        binding_id: binding_id_b,
                        reward: CITIZEN_LIGHTNODE_NORMAL_REWARD,
                    },
                ]
            );
        });
    }

    #[test]
    fn boundary_switches_to_normal_reward_at_high_reward_count() {
        new_test_ext().execute_with(|| {
            RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT);
            let binding_id = <Test as frame_system::Config>::Hashing::hash(b"sfid-boundary");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id);

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
            let binding_id = <Test as frame_system::Config>::Hashing::hash(b"sfid-high-minus-1");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            System::assert_last_event(RuntimeEvent::CitizenLightnodeIssuance(
                Event::<Test>::CertificationRewardIssued {
                    who: 1,
                    binding_id,
                    reward: CITIZEN_LIGHTNODE_HIGH_REWARD,
                },
            ));
        });
    }

    #[test]
    fn same_account_second_sfid_is_not_marked_reward_claimed() {
        new_test_ext().execute_with(|| {
            let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"sfid-claim-a");
            let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"sfid-claim-b");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id_a);
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id_b);

            assert!(RewardClaimed::<Test>::contains_key(binding_id_a));
            assert!(!RewardClaimed::<Test>::contains_key(binding_id_b));
            assert!(AccountRewarded::<Test>::contains_key(1));
            System::assert_last_event(RuntimeEvent::CitizenLightnodeIssuance(
                Event::<Test>::CertificationRewardSkipped {
                    who: 1,
                    binding_id: binding_id_b,
                    reason: SkipReason::AccountAlreadyRewarded,
                },
            ));
        });
    }

    #[test]
    fn different_accounts_and_sfids_reward_independently() {
        new_test_ext().execute_with(|| {
            let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"sfid-a-2");
            let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"sfid-b-2");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id_a);
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&2, binding_id_b);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(Balances::free_balance(2), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(RewardedCount::<Test>::get(), 2);
        });
    }

    #[test]
    fn same_account_different_sfids_only_rewards_once() {
        new_test_ext().execute_with(|| {
            let binding_id_a = <Test as frame_system::Config>::Hashing::hash(b"sfid-acc-a");
            let binding_id_b = <Test as frame_system::Config>::Hashing::hash(b"sfid-acc-b");

            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id_a);
            <CitizenLightnodeIssuance as sfid_code_auth::OnSfidBound<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::on_sfid_bound(&1, binding_id_b);

            assert_eq!(Balances::free_balance(1), CITIZEN_LIGHTNODE_HIGH_REWARD);
            assert_eq!(RewardedCount::<Test>::get(), 1);
            System::assert_last_event(RuntimeEvent::CitizenLightnodeIssuance(
                Event::<Test>::CertificationRewardSkipped {
                    who: 1,
                    binding_id: binding_id_b,
                    reason: SkipReason::AccountAlreadyRewarded,
                },
            ));
        });
    }
}
