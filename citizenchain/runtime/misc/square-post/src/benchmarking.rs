//! SquarePost 订阅签名操作的 FRAME benchmark。

#![cfg(feature = "runtime-benchmarks")]

use crate::{
    pallet::{Call, Config, Pallet, PlatformPrice, RenewalIndex, RenewalSchedule, Subscriptions},
    IssuerKey, MembershipLevel, SubscriptionPlan, SubscriptionState, SubscriptionStatus,
};
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    fn active_platform_state<T: Config>() -> SubscriptionState {
        SubscriptionState {
            plan: SubscriptionPlan::Platform {
                membership_level: MembershipLevel::Freedom,
            },
            started_at: 1,
            last_charged_at: 1,
            last_charged_price_fen: 1,
            paid_until: 2,
            subscription_status: SubscriptionStatus::Active,
            authorized_price_fen: 1,
            suspend_reason: None,
        }
    }

    #[benchmark]
    fn cancel() {
        let caller: T::AccountId = whitelisted_caller();
        let key = (caller.clone(), IssuerKey::Platform);
        Subscriptions::<T>::insert(&key, active_platform_state::<T>());
        RenewalSchedule::<T>::insert(2u64.to_be_bytes(), &key, ());
        RenewalIndex::<T>::insert(&key, 2u64);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), IssuerKey::Platform);

        assert_eq!(
            Subscriptions::<T>::get((caller, IssuerKey::Platform))
                .expect("benchmark state exists")
                .subscription_status,
            SubscriptionStatus::Cancelled
        );
    }

    /// 单笔到期续费处理路径（on_idle 按此估算每块可排空笔数）。
    #[benchmark]
    fn process_one_due() {
        let subscriber_account_id: T::AccountId = whitelisted_caller();
        let key = (subscriber_account_id.clone(), IssuerKey::Platform);
        PlatformPrice::<T>::insert(MembershipLevel::Freedom, 199_900u128);
        Subscriptions::<T>::insert(&key, active_platform_state::<T>());
        RenewalSchedule::<T>::insert(2u64.to_be_bytes(), &key, ());
        RenewalIndex::<T>::insert(&key, 2u64);

        #[block]
        {
            Pallet::<T>::process_due_subscriptions(3u64, 1);
        }

        assert!(!RenewalSchedule::<T>::contains_key(
            2u64.to_be_bytes(),
            &key
        ));
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test,);
}
