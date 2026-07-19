//! SquarePost 订阅签名操作的 FRAME benchmark。

#![cfg(feature = "runtime-benchmarks")]

use crate::{
    pallet::{Call, Config, Pallet, RenewalIndex, RenewalSchedule, Subscriptions},
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
            pending_plan: None,
            started_at: 1,
            last_charged_at: 1,
            last_charged_price_fen: 1,
            paid_until: 2,
            subscription_status: SubscriptionStatus::Active,
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

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test,);
}
