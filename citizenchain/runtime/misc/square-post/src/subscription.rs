//! 平台订阅与创作者订阅的链上状态模型和真实公历换算。
//!
//! 用户签名订阅即建立持续自动扣款授权，只有签名取消才撤销。runtime 使用每个区块的
//! 共识时间戳计算真实 UTC 公历周期并自动扣款；不使用区块高度或固定天数表示周期。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use entity_primitives::InstitutionMultisigQuery;
use frame_support::{
    ensure,
    traits::{ConstU32, UnixTime},
    BoundedVec,
};
use scale_info::TypeInfo;
use sp_runtime::{traits::SaturatedConversion, DispatchError, RuntimeDebug};

use crate::pallet::{Config, CreatorPlans, Error, Pallet, PlatformPrice, Subscriptions};

/// 创作者付款档位编号只承担链上引用，不保存名称、说明或权益文案。
pub type TierId = BoundedVec<u8, ConstU32<32>>;
/// 每个创作者档位最多保存月、季、年三个真实公历周期价格。
pub type PeriodPrices = BoundedVec<PeriodPrice, ConstU32<3>>;
/// 单个创作者最多保存十个链上付款档位。
pub type CreatorTiers = BoundedVec<CreatorTier, ConstU32<10>>;

const MILLIS_PER_DAY: u64 = 86_400_000;

/// 平台会员三档。枚举顺序已经进入跨端 SCALE 协议，禁止调整。
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
    Freedom = 0,
    Democracy = 1,
    Spark = 2,
}

/// 订阅收款主体。创作者使用钱包账户，不使用链下 CID 或展示资料替代。
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
    Platform,
    Creator(AccountId),
}

/// 真实公历周期。runtime 按 UTC 年月日运算，不换算为固定天数或区块数。
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
pub enum BillingPeriod {
    Monthly = 0,
    Quarterly = 1,
    Yearly = 2,
}

impl BillingPeriod {
    fn calendar_months(self) -> i128 {
        match self {
            Self::Monthly => 1,
            Self::Quarterly => 3,
            Self::Yearly => 12,
        }
    }
}

/// 创作者某一真实公历周期的当前链上价格。
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
pub struct PeriodPrice {
    pub billing_period: BillingPeriod,
    pub price_fen: u128,
}

/// 创作者链上付款档位。展示字段只允许保存在 Cloudflare/D1。
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
pub struct CreatorTier {
    pub tier_id: TierId,
    pub prices_fen: PeriodPrices,
}

/// 平台与创作者共用的付款计划。
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
pub enum SubscriptionPlan {
    Platform {
        membership_level: MembershipLevel,
    },
    Creator {
        tier_id: TierId,
        billing_period: BillingPeriod,
    },
}

impl SubscriptionPlan {
    pub fn billing_period(&self) -> BillingPeriod {
        match self {
            Self::Platform { .. } => BillingPeriod::Monthly,
            Self::Creator { billing_period, .. } => *billing_period,
        }
    }
}

/// 余额不足或付款计划失效都进入 `Terminated`，不会自动重试。
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
pub enum SubscriptionStatus {
    Active = 0,
    Cancelled = 1,
    Terminated = 2,
}

/// 链上订阅真源。所有时间都是 UTC Unix 毫秒时间戳。
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
pub struct SubscriptionState {
    pub plan: SubscriptionPlan,
    pub pending_plan: Option<SubscriptionPlan>,
    /// 首次成功扣款所在区块的共识时间戳。
    pub started_at: u64,
    /// 最近一次成功扣款所在区块的共识时间戳。
    pub last_charged_at: u64,
    pub last_charged_price_fen: u128,
    /// 已付权益的独占到期上界，同时也是下一次自动扣款的计划时间。
    pub paid_until: u64,
    pub subscription_status: SubscriptionStatus,
}

/// 将 UTC Unix 毫秒时间戳增加一个真实公历周期。
///
/// 算法只使用确定性整数运算。目标月份没有原日期时使用该月最后一个有效日期，并保留
/// 日内的时、分、秒和毫秒。输入超出 `u64` 可表示范围时返回 `None`。
pub(crate) fn add_calendar_period(timestamp_ms: u64, period: BillingPeriod) -> Option<u64> {
    let days = timestamp_ms / MILLIS_PER_DAY;
    let millis_in_day = timestamp_ms % MILLIS_PER_DAY;
    let (year, month, day) = civil_from_days(i128::from(days));
    let month_index = year
        .checked_mul(12)?
        .checked_add(i128::from(month).checked_sub(1)?)?
        .checked_add(period.calendar_months())?;
    let target_year = month_index.div_euclid(12);
    let target_month = u8::try_from(month_index.rem_euclid(12).checked_add(1)?).ok()?;
    let target_day = day.min(days_in_month(target_year, target_month));
    let target_days = days_from_civil(target_year, target_month, target_day)?;
    let target_days = u64::try_from(target_days).ok()?;
    target_days
        .checked_mul(MILLIS_PER_DAY)?
        .checked_add(millis_in_day)
}

fn is_leap_year(year: i128) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

fn days_in_month(year: i128, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Howard Hinnant civil calendar algorithm；输入为 Unix epoch 之后的天数。
fn civil_from_days(days_since_epoch: i128) -> (i128, u8, u8) {
    let z = days_since_epoch + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (year, month as u8, day as u8)
}

fn days_from_civil(year: i128, month: u8, day: u8) -> Option<i128> {
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    let adjusted_year = year - i128::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let month_prime = i128::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + i128::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era.checked_mul(146_097)?
        .checked_add(day_of_era)?
        .checked_sub(719_468)
}

impl<T: Config> Pallet<T> {
    /// 当前区块共识时间戳，单位 UTC Unix 毫秒。
    pub(crate) fn now_ms() -> u64 {
        T::TimeProvider::now().as_millis().saturated_into::<u64>()
    }

    /// 已付款且尚未到期的 Active/Cancelled 平台订阅都继续提供本周期权益。
    pub(crate) fn has_effective_platform_subscription(account: &T::AccountId, now: u64) -> bool {
        Subscriptions::<T>::get((account.clone(), IssuerKey::Platform))
            .map(|state| {
                matches!(
                    state.subscription_status,
                    SubscriptionStatus::Active | SubscriptionStatus::Cancelled
                ) && now < state.paid_until
            })
            .unwrap_or(false)
    }

    /// 从链上当前价格和计划解析本次收款账户。每次真实扣款都重新调用，禁止永久锁价。
    pub(crate) fn current_price_and_payee(
        issuer: &IssuerKey<T::AccountId>,
        plan: &SubscriptionPlan,
        now: u64,
    ) -> Result<(u128, T::AccountId), DispatchError> {
        match (issuer, plan) {
            (IssuerKey::Platform, SubscriptionPlan::Platform { membership_level }) => {
                let price = PlatformPrice::<T>::get(membership_level)
                    .ok_or(Error::<T>::PlatformPriceNotSet)?;
                ensure!(price > 0, Error::<T>::ZeroPrice);
                // 平台订阅机构永久固定为创世技术公司，CID 单源自创世常量，不读可写存储。
                let cid = primitives::cid::china::citizenchain::CITIZENCHAIN_TECHNOLOGY
                    .cid_number
                    .as_bytes();
                let payee = T::InstitutionAccountQuery::lookup_institution_account(
                    cid,
                    primitives::account_derive::RESERVED_NAME_FEE,
                )
                .ok_or(Error::<T>::PlatformNotBound)?;
                Ok((price, payee))
            }
            (
                IssuerKey::Creator(creator),
                SubscriptionPlan::Creator {
                    tier_id,
                    billing_period,
                },
            ) => {
                ensure!(
                    Self::has_effective_platform_subscription(creator, now),
                    Error::<T>::CreatorNotPlatformMember
                );
                let tiers = CreatorPlans::<T>::get(creator);
                let tier = tiers
                    .iter()
                    .find(|tier| &tier.tier_id == tier_id)
                    .ok_or(Error::<T>::CreatorPlanNotFound)?;
                let price = tier
                    .prices_fen
                    .iter()
                    .find(|price| price.billing_period == *billing_period)
                    .map(|price| price.price_fen)
                    .ok_or(Error::<T>::CreatorPlanNotFound)?;
                ensure!(price > 0, Error::<T>::ZeroPrice);
                Ok((price, creator.clone()))
            }
            _ => Err(Error::<T>::PlanIssuerMismatch.into()),
        }
    }

    /// 校验创作者覆盖式付款套餐，拒绝空 id、零价和重复周期。
    pub(crate) fn validate_creator_tiers(tiers: &[CreatorTier]) -> Result<(), DispatchError> {
        for (index, tier) in tiers.iter().enumerate() {
            ensure!(!tier.tier_id.is_empty(), Error::<T>::EmptyTierId);
            ensure!(!tier.prices_fen.is_empty(), Error::<T>::CreatorPlanNotFound);
            ensure!(
                !tiers[..index]
                    .iter()
                    .any(|existing| existing.tier_id == tier.tier_id),
                Error::<T>::DuplicateTierId
            );
            for (price_index, price) in tier.prices_fen.iter().enumerate() {
                ensure!(price.price_fen > 0, Error::<T>::ZeroPrice);
                ensure!(
                    !tier.prices_fen[..price_index]
                        .iter()
                        .any(|existing| existing.billing_period == price.billing_period),
                    Error::<T>::DuplicateBillingPeriod
                );
            }
        }
        Ok(())
    }
}
