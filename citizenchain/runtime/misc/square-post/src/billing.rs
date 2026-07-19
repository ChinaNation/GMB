//! runtime 内部真实公历自动扣款引擎。
//!
//! 用户签名订阅后，链上订阅状态就是持续扣款授权；只有签名取消才撤销。每个区块结束时
//! 使用已经写入的共识时间戳处理到期调度，不需要 CitizenApp、Cloudflare 或外部交易。

use crate::{
    pallet::{
        BalanceOf, Config, CreatorPlans, Error, Event, Pallet, RenewalIndex, RenewalSchedule,
        SubKeyOf, Subscriptions,
    },
    subscription::{
        add_calendar_period, CreatorTier, CreatorTiers, IssuerKey, SubscriptionPlan,
        SubscriptionState, SubscriptionStatus,
    },
};
use frame_support::{
    ensure,
    storage::with_storage_layer,
    traits::{Currency, ExistenceRequirement},
};
use sp_runtime::{traits::SaturatedConversion, DispatchResult};
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    /// 订阅并立即完成首次扣款。已有 Active 同计划幂等；已取消但尚未到期时恢复调度且不重扣。
    pub(crate) fn do_subscribe(
        subscriber: T::AccountId,
        issuer: IssuerKey<T::AccountId>,
        plan: SubscriptionPlan,
        expected_price_fen: u128,
    ) -> DispatchResult {
        Self::ensure_subscription_ready()?;
        if let IssuerKey::Creator(creator) = &issuer {
            ensure!(creator != &subscriber, Error::<T>::CannotSubscribeSelf);
        }
        let now = Self::now_ms();
        let key = (subscriber.clone(), issuer.clone());
        if let Some(mut state) = Subscriptions::<T>::get(&key) {
            if state.subscription_status == SubscriptionStatus::Active {
                ensure!(state.plan == plan, Error::<T>::TermsLocked);
                return Ok(());
            }
            if state.subscription_status == SubscriptionStatus::Cancelled && now < state.paid_until
            {
                ensure!(state.plan == plan, Error::<T>::TermsLocked);
                state.subscription_status = SubscriptionStatus::Active;
                Subscriptions::<T>::insert(&key, state.clone());
                Self::schedule_renewal(&key, state.paid_until);
                Self::deposit_event(Event::SubscriptionResumed {
                    subscriber,
                    issuer,
                    paid_until: state.paid_until,
                });
                return Ok(());
            }
        }
        Self::charge_and_schedule(subscriber, issuer, plan, Some(expected_price_fen), true)
    }

    /// 用户主动订阅或到期后签名更换计划时的原子扣款路径。
    fn charge_and_schedule(
        subscriber: T::AccountId,
        issuer: IssuerKey<T::AccountId>,
        plan: SubscriptionPlan,
        expected_price_fen: Option<u128>,
        reset_started_at: bool,
    ) -> DispatchResult {
        with_storage_layer(|| -> DispatchResult {
            let now = Self::now_ms();
            let (price_fen, payee) = Self::current_price_and_payee(&issuer, &plan, now)?;
            if let Some(expected) = expected_price_fen {
                ensure!(expected == price_fen, Error::<T>::SignedPriceChanged);
            }
            let paid_until = add_calendar_period(now, plan.billing_period())
                .ok_or(Error::<T>::CalendarOverflow)?;
            let amount: BalanceOf<T> = price_fen.saturated_into();
            T::Currency::transfer(&subscriber, &payee, amount, ExistenceRequirement::KeepAlive)?;

            let key = (subscriber.clone(), issuer.clone());
            let previous = Subscriptions::<T>::get(&key);
            Self::unschedule_renewal(&key);
            let started_at = if reset_started_at {
                now
            } else {
                previous
                    .as_ref()
                    .map(|state| state.started_at)
                    .unwrap_or(now)
            };
            Subscriptions::<T>::insert(
                &key,
                SubscriptionState {
                    plan: plan.clone(),
                    pending_plan: None,
                    started_at,
                    last_charged_at: now,
                    last_charged_price_fen: price_fen,
                    paid_until,
                    subscription_status: SubscriptionStatus::Active,
                },
            );
            Self::schedule_renewal(&key, paid_until);
            Self::deposit_event(Event::SubscriptionCharged {
                subscriber,
                issuer,
                plan,
                price_fen,
                charged_at: now,
                paid_until,
            });
            Ok(())
        })
    }

    /// 按时间戳有序处理到期任务。链曾停止出块时，逾期周期按顺序继续处理直到追上当前时间。
    pub(crate) fn process_due_subscriptions(now: u64, limit: u32) -> u32 {
        let mut processed = 0u32;
        while processed < limit {
            let Some((due_key, subscription_key)) = RenewalSchedule::<T>::iter_keys().next() else {
                break;
            };
            let due_at = u64::from_be_bytes(due_key);
            if due_at > now {
                break;
            }
            RenewalSchedule::<T>::remove(due_key, &subscription_key);
            RenewalIndex::<T>::remove(&subscription_key);
            Self::process_one_due(subscription_key, due_at, now);
            processed = processed.saturating_add(1);
        }
        processed
    }

    fn process_one_due(key: SubKeyOf<T>, due_at: u64, now: u64) {
        let Some(mut state) = Subscriptions::<T>::get(&key) else {
            return;
        };
        if state.subscription_status != SubscriptionStatus::Active || state.paid_until != due_at {
            return;
        }
        let (subscriber, issuer) = key.clone();
        let plan = state
            .pending_plan
            .clone()
            .unwrap_or_else(|| state.plan.clone());
        let Ok((price_fen, payee)) = Self::current_price_and_payee(&issuer, &plan, now) else {
            state.pending_plan = None;
            state.subscription_status = SubscriptionStatus::Terminated;
            Subscriptions::<T>::insert(&key, state);
            Self::deposit_event(Event::SubscriptionRenewalStopped {
                subscriber,
                issuer,
                stopped_at: now,
            });
            return;
        };
        let Some(paid_until) = add_calendar_period(due_at, plan.billing_period()) else {
            state.pending_plan = None;
            state.subscription_status = SubscriptionStatus::Terminated;
            Subscriptions::<T>::insert(&key, state);
            Self::deposit_event(Event::SubscriptionRenewalStopped {
                subscriber,
                issuer,
                stopped_at: now,
            });
            return;
        };
        let amount: BalanceOf<T> = price_fen.saturated_into();
        if T::Currency::transfer(&subscriber, &payee, amount, ExistenceRequirement::KeepAlive)
            .is_err()
        {
            state.pending_plan = None;
            state.subscription_status = SubscriptionStatus::Terminated;
            Subscriptions::<T>::insert(&key, state);
            Self::deposit_event(Event::SubscriptionPaymentFailed {
                subscriber,
                issuer,
                attempted_price_fen: price_fen,
                attempted_at: now,
            });
            return;
        }

        state.plan = plan.clone();
        state.pending_plan = None;
        state.last_charged_at = now;
        state.last_charged_price_fen = price_fen;
        state.paid_until = paid_until;
        state.subscription_status = SubscriptionStatus::Active;
        Subscriptions::<T>::insert(&key, state);
        Self::schedule_renewal(&key, paid_until);
        Self::deposit_event(Event::SubscriptionCharged {
            subscriber,
            issuer,
            plan,
            price_fen,
            charged_at: now,
            paid_until,
        });
    }

    pub(crate) fn schedule_renewal(key: &SubKeyOf<T>, due_at: u64) {
        Self::unschedule_renewal(key);
        RenewalSchedule::<T>::insert(due_at.to_be_bytes(), key, ());
        RenewalIndex::<T>::insert(key, due_at);
    }

    pub(crate) fn unschedule_renewal(key: &SubKeyOf<T>) {
        if let Some(previous) = RenewalIndex::<T>::take(key) {
            RenewalSchedule::<T>::remove(previous.to_be_bytes(), key);
        }
    }

    /// 签名取消是撤销自动扣款授权的唯一方式；当前已付款权益不缩短。
    pub(crate) fn do_cancel(
        subscriber: T::AccountId,
        issuer: IssuerKey<T::AccountId>,
    ) -> DispatchResult {
        Self::ensure_subscription_ready()?;
        let key = (subscriber.clone(), issuer.clone());
        let paid_until = Subscriptions::<T>::try_mutate(&key, |slot| {
            let state = slot.as_mut().ok_or(Error::<T>::SubscriptionNotFound)?;
            state.pending_plan = None;
            state.subscription_status = SubscriptionStatus::Cancelled;
            Ok::<_, Error<T>>(state.paid_until)
        })?;
        Self::unschedule_renewal(&key);
        Self::deposit_event(Event::SubscriptionCancelled {
            subscriber,
            issuer,
            paid_until,
        });
        Ok(())
    }

    /// 未到期时登记下周期计划；已终止或已过期时按目标当前价立即扣款并重新调度。
    pub(crate) fn do_change_subscription_plan(
        subscriber: T::AccountId,
        issuer: IssuerKey<T::AccountId>,
        new_plan: SubscriptionPlan,
        expected_price_fen: u128,
    ) -> DispatchResult {
        Self::ensure_subscription_ready()?;
        let key = (subscriber.clone(), issuer.clone());
        let mut state = Subscriptions::<T>::get(&key).ok_or(Error::<T>::SubscriptionNotFound)?;
        let now = Self::now_ms();
        let (current_price, _) = Self::current_price_and_payee(&issuer, &new_plan, now)?;
        ensure!(
            current_price == expected_price_fen,
            Error::<T>::SignedPriceChanged
        );
        if now < state.paid_until {
            state.pending_plan = Some(new_plan.clone());
            state.subscription_status = SubscriptionStatus::Active;
            let paid_until = state.paid_until;
            Subscriptions::<T>::insert(&key, state);
            Self::schedule_renewal(&key, paid_until);
            Self::deposit_event(Event::SubscriptionPlanChangePending {
                subscriber,
                issuer,
                new_plan,
            });
            return Ok(());
        }
        Self::charge_and_schedule(subscriber, issuer, new_plan, Some(expected_price_fen), true)
    }

    /// 创作者覆盖式写入自己的链上付款套餐；展示资料仍只在 Cloudflare/D1。
    pub(crate) fn do_set_creator_plans(
        creator: T::AccountId,
        tiers: Vec<CreatorTier>,
    ) -> DispatchResult {
        Self::ensure_subscription_ready()?;
        ensure!(
            Self::has_effective_platform_subscription(&creator, Self::now_ms()),
            Error::<T>::CreatorNotPlatformMember
        );
        Self::validate_creator_tiers(&tiers)?;
        let bounded = CreatorTiers::try_from(tiers).map_err(|_| Error::<T>::TooManyCreatorTiers)?;
        CreatorPlans::<T>::insert(&creator, bounded.clone());
        Self::deposit_event(Event::CreatorPlansSet {
            creator,
            tier_count: bounded.len() as u32,
        });
        Ok(())
    }
}
