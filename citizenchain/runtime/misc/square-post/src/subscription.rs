//! 会员订阅纯类型 + 订阅/取消/定价解析逻辑。
//!
//! 职责边界（死规则）：
//! - 本文件只放订阅相关的纯数据类型与 `subscribe`/`cancel` 的业务体。
//! - 自动扣款在 [`crate::billing`]；链上不做任何日历/周期计算。
//! - 时间只以原始 unix 毫秒时间戳存储（`SubscriptionState::last_charged_at`），
//!   "到期没到期""付费到什么年月日"全部由本机（CitizenApp）读时间戳自行计算，链上不解释。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::pallet::{
    Config, CreatorPlans, Error, Event, Pallet, PlatformCidNumber, PlatformPrice, Subscriptions,
};
use entity_primitives::InstitutionMultisigQuery;
use frame_support::ensure;
use sp_runtime::{DispatchError, DispatchResult};

/// 平台会员三档。语义与官网法币轨共享同一套档位（价源各自独立、不跨折算）。
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
#[repr(u8)]
pub enum MembershipLevel {
    /// 自由会员。
    Freedom = 0,
    /// 民主会员。
    Democracy = 1,
    /// 薪火会员。
    Spark = 2,
}

/// 订阅收款主体。
///
/// - `Platform`：平台会员，收款方=技术公司**费用账户**（由 `PlatformCidNumber` 派生 OP_FEE）。
/// - `Creator(account)`：创作者会员，收款方=创作者本人钱包账户（任意钱包账户，无 CID 要求）。
///
/// SCALE 布局锁定：`tag(1B)` `[+32B AccountId]`。五端逐字节一致。
#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum IssuerKey<AccountId> {
    /// 平台会员。
    Platform,
    /// 创作者会员，携带创作者钱包账户。
    Creator(AccountId),
}

/// 订阅档位选择。平台用 `Level`、创作者用 `Tier`（档位码）。SCALE 2 字节。
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
pub enum SubscriptionPlan {
    /// 平台档位。
    Level(MembershipLevel),
    /// 创作者档位码。
    Tier(u8),
}

/// 订阅状态。`Cancelled` 保留记录支持续订，不删。
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
pub enum SubscriptionStatus {
    /// 生效中。
    Active,
    /// 欠费即停（扣款失败翻此态，不重试不续扣）。
    PastDue,
    /// 已取消（记录保留，供续订恢复 Active）。
    Cancelled,
}

/// 单条订阅链上状态。
///
/// `last_charged_at` 是上次成功扣款的 unix 毫秒时间戳，仅供本机计算日历到期日；链上不解释、不比较。
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
pub struct SubscriptionState {
    /// 档位选择。
    pub plan: SubscriptionPlan,
    /// 最近成功扣款金额（分）快照，仅展示/审计，不作下次扣款依据。
    pub price_fen: u128,
    /// 上次成功扣款时间戳（unix 毫秒），供本机算日历。
    pub last_charged_at: u64,
    /// 订阅状态。
    pub status: SubscriptionStatus,
}

/// 创作者单档：档位码 + 月价（分）。不存展示名（展示名/介绍全链下）。SCALE 17 字节。
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
pub struct CreatorTier {
    /// 档位码（App 连续赋号，与订阅 `Tier(tier_code)` 同义）。
    pub tier_code: u8,
    /// 月价（分）。
    pub price_fen: u128,
}

impl<T: Config> Pallet<T> {
    /// 订阅（幂等"确保 Active"语义），首扣即时完成。
    ///
    /// - 无记录 → 开新单，立即首扣。
    /// - 同档 Active → 幂等 no-op，不重复扣款。
    /// - 同档 Cancelled → 只翻回 Active，不重扣（续订不二次收费）。
    /// - 换档 / PastDue → 视为重新开单，立即首扣。
    pub(crate) fn do_subscribe(
        who: T::AccountId,
        issuer: IssuerKey<T::AccountId>,
        plan: SubscriptionPlan,
    ) -> DispatchResult {
        if let IssuerKey::Creator(ref creator) = issuer {
            ensure!(creator != &who, Error::<T>::CannotSubscribeSelf);
        }
        let key = (who.clone(), issuer.clone());
        if let Some(existing) = Subscriptions::<T>::get(&key) {
            if existing.plan == plan && existing.status == SubscriptionStatus::Active {
                return Ok(());
            }
            if existing.plan == plan && existing.status == SubscriptionStatus::Cancelled {
                Subscriptions::<T>::mutate(&key, |slot| {
                    if let Some(state) = slot {
                        state.status = SubscriptionStatus::Active;
                    }
                });
                Self::deposit_event(Event::Subscribed {
                    subscriber: who,
                    issuer,
                    plan,
                });
                return Ok(());
            }
        }
        let now = Self::now_ms();
        Self::try_charge(&who, &issuer, plan, now)?;
        Self::deposit_event(Event::Subscribed {
            subscriber: who,
            issuer,
            plan,
        });
        Ok(())
    }

    /// 取消：写 `Cancelled` 保留记录（供续订恢复），幂等。
    pub(crate) fn do_cancel(
        who: T::AccountId,
        issuer: IssuerKey<T::AccountId>,
    ) -> DispatchResult {
        let key = (who.clone(), issuer.clone());
        Subscriptions::<T>::try_mutate(&key, |slot| -> DispatchResult {
            let state = slot.as_mut().ok_or(Error::<T>::SubscriptionNotFound)?;
            state.status = SubscriptionStatus::Cancelled;
            Ok(())
        })?;
        Self::deposit_event(Event::Cancelled {
            subscriber: who,
            issuer,
        });
        Ok(())
    }

    /// 唯一定价 / 收款方解析入口，首扣与续扣共用；现读现算，杜绝第二价源。
    pub(crate) fn resolve_price_and_payee(
        issuer: &IssuerKey<T::AccountId>,
        plan: &SubscriptionPlan,
    ) -> Result<(u128, T::AccountId), DispatchError> {
        match (issuer, plan) {
            (IssuerKey::Platform, SubscriptionPlan::Level(level)) => {
                let price =
                    PlatformPrice::<T>::get(level).ok_or(Error::<T>::PlatformPriceNotSet)?;
                let cid = PlatformCidNumber::<T>::get().ok_or(Error::<T>::PlatformNotBound)?;
                // 平台收款方 = 技术公司「费用账户」（OP_FEE），非主账户。
                let payee = T::InstitutionAccountQuery::lookup_institution_account(
                    cid.as_slice(),
                    primitives::account_derive::RESERVED_NAME_FEE,
                )
                .ok_or(Error::<T>::PlatformNotBound)?;
                Ok((price, payee))
            }
            (IssuerKey::Creator(creator), SubscriptionPlan::Tier(tier_code)) => {
                let tier = CreatorPlans::<T>::get(creator)
                    .into_iter()
                    .find(|t| t.tier_code == *tier_code)
                    .ok_or(Error::<T>::CreatorTierNotFound)?;
                // 创作者收款方 = 本人钱包账户，全额转，零折算。
                Ok((tier.price_fen, creator.clone()))
            }
            _ => Err(Error::<T>::PlanIssuerMismatch.into()),
        }
    }
}
