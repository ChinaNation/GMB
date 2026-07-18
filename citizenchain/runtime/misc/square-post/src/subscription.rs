//! 会员订阅纯类型 + 订阅/取消/定价解析逻辑。
//!
//! 职责边界（死规则）：
//! - 本文件只放订阅相关的纯数据类型与 `subscribe`/`cancel` 的业务体。
//! - 自动扣款在 [`crate::billing`]；链上不做任何日历/周期计算。
//! - 时间只以原始 unix 毫秒时间戳存储（`SubscriptionState::last_charged_at`），
//!   "到期没到期""付费到什么年月日"全部由本机（CitizenApp）读时间戳自行计算，链上不解释。
//! - **平台会员**：三档定义与价格在链上（`PlatformPrice`），续扣现读链上价。
//! - **创作者会员**：档定义（名称/档种类/月季年周期/价格）**全部链下**（App 本地设 + Cloudflare 存）；
//!   链上只存最小订阅记录（价/时间戳/状态）做付款；续扣价由续订触发方（keeper）按创作者当前价带入。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::pallet::{
    Config, Error, Event, Pallet, PlatformCidNumber, PlatformPrice, Subscriptions,
};
use entity_primitives::InstitutionMultisigQuery;
use frame_support::ensure;
use sp_runtime::{DispatchError, DispatchResult};

/// 平台会员三档。链上定义，价格经技术公司治理写入 `PlatformPrice`。
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

/// 订阅档位选择。
///
/// - 平台用 `Level`（携带档位，续扣现读 `PlatformPrice[level]`）。
/// - 创作者用 `CreatorPrice`（携带价，分）：创作者档在链下，链上只记价快照，续扣价由 keeper 现带。
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
    /// 创作者当前价（分）。
    CreatorPrice(u128),
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

/// 单条订阅链上最小记录。
///
/// `last_charged_at` 是上次成功扣款的 unix 毫秒时间戳，仅供本机计算日历到期日；链上不解释、不比较。
/// 取消精确时刻由 `Cancelled` 事件承载，不在本记录重复存。
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
    /// 平台=Level(用于续扣现读价)；创作者=CreatorPrice(记录用，续扣价由 keeper 现带)。
    pub plan: SubscriptionPlan,
    /// 最近成功扣款金额（分）。
    pub price_fen: u128,
    /// 上次成功扣款时间戳（unix 毫秒），供本机算日历。
    pub last_charged_at: u64,
    /// 订阅状态。
    pub status: SubscriptionStatus,
}

impl<T: Config> Pallet<T> {
    /// 订阅（幂等"确保 Active"语义），首扣即时完成。
    ///
    /// - 创作者臂：`who` 不能订自己；**创作者必须是有效平台会员（链上强制）**。
    /// - 无记录 → 开新单立即首扣；同档 Active → 幂等 no-op；同档 Cancelled → 翻 Active 不重扣；
    ///   换档 / PastDue → 重新开单立即首扣。
    pub(crate) fn do_subscribe(
        who: T::AccountId,
        issuer: IssuerKey<T::AccountId>,
        plan: SubscriptionPlan,
    ) -> DispatchResult {
        if let IssuerKey::Creator(ref creator) = issuer {
            ensure!(creator != &who, Error::<T>::CannotSubscribeSelf);
            // 点3：创作者必须是当前有效平台会员，链上强制（非 Cloudflare）。
            let creator_is_member = Subscriptions::<T>::get((creator.clone(), IssuerKey::Platform))
                .map(|state| state.status == SubscriptionStatus::Active)
                .unwrap_or(false);
            ensure!(creator_is_member, Error::<T>::CreatorNotPlatformMember);
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
    pub(crate) fn do_cancel(who: T::AccountId, issuer: IssuerKey<T::AccountId>) -> DispatchResult {
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
    ///
    /// - 平台：价现读 `PlatformPrice[level]`（链上真源），收款方=费用账户。
    /// - 创作者：价=`plan` 携带的 `CreatorPrice`（>0），收款方=创作者本人，全额零折算。
    pub(crate) fn resolve_price_and_payee(
        issuer: &IssuerKey<T::AccountId>,
        plan: &SubscriptionPlan,
    ) -> Result<(u128, T::AccountId), DispatchError> {
        match (issuer, plan) {
            (IssuerKey::Platform, SubscriptionPlan::Level(level)) => {
                let price =
                    PlatformPrice::<T>::get(level).ok_or(Error::<T>::PlatformPriceNotSet)?;
                let cid = PlatformCidNumber::<T>::get().ok_or(Error::<T>::PlatformNotBound)?;
                let payee = T::InstitutionAccountQuery::lookup_institution_account(
                    cid.as_slice(),
                    primitives::account_derive::RESERVED_NAME_FEE,
                )
                .ok_or(Error::<T>::PlatformNotBound)?;
                Ok((price, payee))
            }
            (IssuerKey::Creator(creator), SubscriptionPlan::CreatorPrice(amount)) => {
                ensure!(*amount > 0, Error::<T>::ZeroPrice);
                Ok((*amount, creator.clone()))
            }
            _ => Err(Error::<T>::PlanIssuerMismatch.into()),
        }
    }
}
