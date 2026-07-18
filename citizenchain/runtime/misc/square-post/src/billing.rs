//! 自动扣款引擎。
//!
//! 职责边界（死规则）：
//! - runtime 只负责「钱的流动」：现读价/收款方 → 原子转账 → 记录状态。
//! - 链上**不做**任何日历/周期/到期判断：`charge_due` 由续订触发方（链下 keeper）调用，
//!   收到触发即扣一次；「到期没到期」由本机/Cloudflare 读时间戳算好后再触发。
//! - 首扣（`subscribe`）与续扣（`charge_due`）共用唯一原子路径 [`Pallet::try_charge`]，
//!   杜绝两套扣款逻辑漂移。
//! - 价源分层：**平台价链上真源**（续扣现读 `PlatformPrice`）；**创作者价链下真源**
//!   （续扣由 keeper 按创作者当前价带入 → 改价后续扣自然走新价）。

use crate::pallet::{BalanceOf, Config, Error, Event, Pallet, Subscriptions};
use crate::subscription::{IssuerKey, SubscriptionPlan, SubscriptionState, SubscriptionStatus};
use frame_support::{
    ensure,
    storage::with_storage_layer,
    traits::{Currency, ExistenceRequirement, UnixTime},
};
use sp_runtime::{DispatchResult, SaturatedConversion};

impl<T: Config> Pallet<T> {
    /// 当前 unix 毫秒时间戳（链上共识挂钟 `pallet_timestamp`）。
    pub(crate) fn now_ms() -> u64 {
        T::TimeProvider::now().as_millis().saturated_into::<u64>()
    }

    /// 原子扣款：现读价/收款方 → 转账 → 写状态。首扣与续扣的唯一执行路径。
    ///
    /// 全过程在 `with_storage_layer` 内：任一步失败整笔回滚（已转账/已写状态全部撤销），
    /// 不留「已订阅未扣款」悬空态。
    pub(crate) fn try_charge(
        subscriber: &T::AccountId,
        issuer: &IssuerKey<T::AccountId>,
        plan: SubscriptionPlan,
        now: u64,
    ) -> DispatchResult {
        with_storage_layer(|| -> DispatchResult {
            let (price_fen, payee) = Self::resolve_price_and_payee(issuer, &plan)?;
            let amount: BalanceOf<T> = price_fen.saturated_into();
            // KeepAlive：扣额使付款人低于存在性余额即拒。
            T::Currency::transfer(subscriber, &payee, amount, ExistenceRequirement::KeepAlive)?;
            let key = (subscriber.clone(), issuer.clone());
            Subscriptions::<T>::insert(
                &key,
                SubscriptionState {
                    plan,
                    price_fen,
                    last_charged_at: now,
                    status: SubscriptionStatus::Active,
                },
            );
            Self::deposit_event(Event::Charged {
                subscriber: subscriber.clone(),
                issuer: issuer.clone(),
                amount,
            });
            Ok(())
        })
    }

    /// 续扣：收到续订触发方调用即扣一次（链上零到期判断）。
    ///
    /// - 平台：`amount` 必须为 `None`（平台价链上真源，不接受 keeper 带价），现读 `PlatformPrice`。
    /// - 创作者：`amount` 必须为 `Some(当前价)`（创作者价链下真源，keeper 带入），改价后走新价。
    ///
    /// 扣款失败 → 写 `PastDue`（在 `try_charge` 回滚层之外，欠费即停不重试）。
    /// 本函数整体返回 `Ok`：续扣失败不是 extrinsic 失败，状态已记 `PastDue`。
    pub(crate) fn do_charge_due(
        subscriber: T::AccountId,
        issuer: IssuerKey<T::AccountId>,
        amount: Option<u128>,
    ) -> DispatchResult {
        let key = (subscriber.clone(), issuer.clone());
        let state = Subscriptions::<T>::get(&key).ok_or(Error::<T>::SubscriptionNotFound)?;
        // 已取消的订阅不续扣。
        ensure!(
            state.status != SubscriptionStatus::Cancelled,
            Error::<T>::SubscriptionNotFound
        );
        let plan = match issuer {
            IssuerKey::Platform => {
                ensure!(amount.is_none(), Error::<T>::PlanIssuerMismatch);
                state.plan // Level(level) → resolve 现读 PlatformPrice[level]
            }
            IssuerKey::Creator(_) => {
                let current = amount.ok_or(Error::<T>::CreatorPriceRequired)?;
                SubscriptionPlan::CreatorPrice(current)
            }
        };
        if Self::try_charge(&subscriber, &issuer, plan, Self::now_ms()).is_err() {
            Subscriptions::<T>::mutate(&key, |slot| {
                if let Some(s) = slot {
                    s.status = SubscriptionStatus::PastDue;
                }
            });
            Self::deposit_event(Event::ChargeFailed { subscriber, issuer });
        }
        Ok(())
    }
}
