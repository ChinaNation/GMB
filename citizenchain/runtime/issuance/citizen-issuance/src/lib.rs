//! # 公民轻节点认证奖励发行模块 (citizen-issuance)
//!
//! 本模块在 CID 绑定成功时，通过 `OnCidBound` 回调自动发放一次性认证奖励。
//!
//! ## 核心规则
//! - 双重防重：按 `binding_id` + 按账户，防止同一身份或同一账户重复领奖。
//! - 阶梯奖励：前 `CITIZEN_ISSUANCE_HIGH_REWARD_COUNT` 人获高额奖励，之后降为常规奖励。
//! - 总量硬顶：累计发放人数达到 `CITIZEN_ISSUANCE_MAX_COUNT` 后停止发放。
//! - 本模块不暴露任何 extrinsic，所有触发均来自上游回调。

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use cid_system::{OnCidBound, OnCidBoundWeight};
    use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, Imbalance},
        Blake2_128Concat,
    };
    use scale_info::TypeInfo;
    use sp_runtime::traits::{SaturatedConversion, Zero};
    use sp_runtime::RuntimeDebug;

    use crate::weights::WeightInfo;
    use primitives::citizen_const::{
        CITIZEN_ISSUANCE_HIGH_REWARD, CITIZEN_ISSUANCE_HIGH_REWARD_COUNT,
        CITIZEN_ISSUANCE_MAX_COUNT, CITIZEN_ISSUANCE_NORMAL_REWARD, CITIZEN_ISSUANCE_ONE_TIME_ONLY,
    };

    // 中文注释：链上规则强制“一次性奖励”，禁止通过配置关闭该约束。
    const _: () = assert!(
        CITIZEN_ISSUANCE_ONE_TIME_ONLY,
        "CITIZEN_ISSUANCE_ONE_TIME_ONLY must be true"
    );
    // 中文注释：奖励金额属于制度常量，必须在编译期保持非零。
    const _: () = assert!(
        CITIZEN_ISSUANCE_HIGH_REWARD > 0,
        "CITIZEN_ISSUANCE_HIGH_REWARD must be greater than zero"
    );
    // 中文注释：常规奖励同样不允许配置成零，零奖励只保留运行时类型转换兜底。
    const _: () = assert!(
        CITIZEN_ISSUANCE_NORMAL_REWARD > 0,
        "CITIZEN_ISSUANCE_NORMAL_REWARD must be greater than zero"
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
    /// 中文注释：按账户维度再做一次防重，避免同一账户换绑 CID 后再次领奖。
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
        /// 中文注释：该账户已通过其他 CID 领取过奖励，换绑不可再领。
        AccountAlreadyRewarded,
        /// 中文注释：奖励常量已由编译期断言锁定为非零；该分支只兜底 Balance 转换异常。
        ZeroRewardConfigured,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 中文注释：CID 绑定成功后，认证发行模块执行一次奖励发放。
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

    /// 中文注释：本模块不暴露 extrinsic，奖励发放由 OnCidBound 回调驱动，无需用户直接调用。
    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    impl<T: Config> Pallet<T> {
        /// 调用方在 weight 宏中引用此值。
        pub fn on_cid_bound_weight() -> Weight {
            // 中文注释：上游 bind_cid 在申报 weight 时会叠加这里的回调预算。
            T::WeightInfo::on_cid_bound()
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
            if rewarded_count >= CITIZEN_ISSUANCE_MAX_COUNT {
                return Err(SkipReason::MaxCountReached);
            }

            // 中文注释：奖励档位完全由全局累计人数决定，避免链下参与者各自推导口径不一致。
            let reward_amount = if rewarded_count < CITIZEN_ISSUANCE_HIGH_REWARD_COUNT {
                CITIZEN_ISSUANCE_HIGH_REWARD
            } else {
                CITIZEN_ISSUANCE_NORMAL_REWARD
            };

            let reward: BalanceOf<T> = reward_amount.saturated_into();
            debug_assert!(
                !reward.is_zero(),
                "citizen issuance reward constants must stay greater than zero"
            );
            // 中文注释：制度奖励常量已编译期锁定非零，这里保留为 Balance 类型转换后的防御性兜底。
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

    /// 中文注释：实现 cid-system 的绑定回调，在 CID 绑定成功后自动尝试发放认证奖励。
    impl<T: Config> OnCidBound<T::AccountId, T::Hash> for Pallet<T> {
        fn on_cid_bound(who: &T::AccountId, binding_id: T::Hash) {
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

    /// 中文注释：向上游提供回调的 weight 预算，供 bind_cid 在申报交易权重时叠加。
    impl<T: Config> OnCidBoundWeight for Pallet<T> {
        fn on_cid_bound_weight() -> Weight {
            Pallet::<T>::on_cid_bound_weight()
        }
    }
}

#[cfg(test)]
mod tests;
