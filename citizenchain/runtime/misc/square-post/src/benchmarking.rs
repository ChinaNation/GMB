//! SquarePost 订阅签名操作的 FRAME benchmark。

#![cfg(feature = "runtime-benchmarks")]

use crate::{
    pallet::{
        Call, CidNumberOf, Config, Pallet, PlatformPrice, PostIdOf, RenewalIndex, RenewalSchedule,
        SquarePosts, Subscriptions,
    },
    IssuerKey, MembershipLevel, SquarePostCategory, SquarePostCitizenIdentityProvider,
    SubscriptionPlan, SubscriptionState, SubscriptionStatus,
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

    fn benchmark_cid<T: Config>(account_id: &T::AccountId) -> CidNumberOf<T> {
        T::CitizenIdentity::benchmark_seed_identity(account_id)
            .try_into()
            .ok()
            .expect("benchmark CID must fit SquarePost bounds")
    }

    /// 竞选动态是发布入口最重路径：除 active CID 双向绑定外，还必须读取竞选身份。
    #[benchmark]
    fn publish_post() {
        let caller: T::AccountId = whitelisted_caller();
        let _ = benchmark_cid::<T>(&caller);
        let post_id = b"benchmark-campaign-post".to_vec();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            post_id.clone(),
            SquarePostCategory::Campaign,
            [7u8; 32],
            b"benchmark-storage-receipt".to_vec(),
            1,
        );

        let post_id: PostIdOf<T> = post_id.try_into().ok().expect("benchmark post id fits");
        assert!(SquarePosts::<T>::contains_key(post_id));
    }

    #[benchmark]
    fn cancel() {
        let caller: T::AccountId = whitelisted_caller();
        let caller_cid_number = benchmark_cid::<T>(&caller);
        let key = (caller_cid_number.clone(), IssuerKey::Platform);
        Subscriptions::<T>::insert(&key, active_platform_state::<T>());
        RenewalSchedule::<T>::insert(2u64.to_be_bytes(), &key, ());
        RenewalIndex::<T>::insert(&key, 2u64);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), IssuerKey::Platform);

        assert_eq!(
            Subscriptions::<T>::get((caller_cid_number, IssuerKey::Platform))
                .expect("benchmark state exists")
                .subscription_status,
            SubscriptionStatus::Cancelled
        );
    }

    /// 单笔到期续费处理路径（on_initialize 按实际处理笔数记账）。
    #[benchmark]
    fn process_one_due() {
        let subscriber_account_id: T::AccountId = whitelisted_caller();
        let subscriber_cid_number = benchmark_cid::<T>(&subscriber_account_id);
        let key = (subscriber_cid_number, IssuerKey::Platform);
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
