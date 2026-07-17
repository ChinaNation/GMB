#![cfg_attr(not(feature = "std"), no_std)]
//! # 广场内容业务模块 (square-post)
//!
//! 本 pallet 是「广场内容域」的唯一链上模块，承载两类职责：
//! 1. 广场动态链上发布索引（`publish_post`）——只记录发布事实、内容哈希和存储回执，
//!    媒体内容必须保存在独立内容存储网络，不得写入 runtime storage。
//! 2. 会员订阅**自动扣费核**（公民币单轨）——平台会员与创作者会员共用一套订阅状态机
//!    与扣款引擎；runtime 只做「钱的流动」，换档折算/预览/门禁/权益计算/展示全部链下。
//!
//! 订阅相关类型与逻辑分布：类型与订阅/取消/定价见 [`subscription`]，自动扣款见 [`billing`]。

pub use pallet::*;
pub mod billing;
pub mod subscription;
pub mod weights;

pub use subscription::{
    CreatorTier, IssuerKey, MembershipLevel, SubscriptionPlan, SubscriptionState,
    SubscriptionStatus,
};

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// 广场发布分类。
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
pub enum SquarePostCategory {
    /// 普通动态，所有钱包账户都可以发布。
    Normal = 0,
    /// 竞选动态，必须是已绑定 CID 的认证公民钱包账户。
    Campaign = 1,
}

/// 广场动态链上索引。
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
pub struct SquarePost<AccountId, BlockNumber, PostId, CidNumber, StorageReceiptId> {
    pub post_id: PostId,
    pub owner_account: AccountId,
    pub cid_number: Option<CidNumber>,
    pub post_category: SquarePostCategory,
    pub content_hash: [u8; 32],
    pub storage_receipt_id: StorageReceiptId,
    pub storage_until: u64,
    pub created_block: BlockNumber,
}

/// 公民身份读取适配器。
///
/// runtime 负责把它接到 citizen-identity；本 pallet 不直接依赖具体身份 pallet。
pub trait SquarePostCitizenIdentityProvider<AccountId> {
    fn cid_number(owner_account: &AccountId) -> Option<Vec<u8>>;
}

impl<AccountId> SquarePostCitizenIdentityProvider<AccountId> for () {
    fn cid_number(_owner_account: &AccountId) -> Option<Vec<u8>> {
        None
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::{
        ensure,
        pallet_prelude::*,
        traits::{Currency, UnixTime},
    };
    use frame_system::pallet_prelude::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    pub type PostIdOf<T> = BoundedVec<u8, <T as Config>::MaxSquarePostIdLen>;
    pub type CidNumberOf<T> = BoundedVec<u8, <T as Config>::MaxSquareCidNumberLen>;
    pub type StorageReceiptIdOf<T> = BoundedVec<u8, <T as Config>::MaxSquareStorageReceiptIdLen>;
    pub type SquarePostOf<T> = SquarePost<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        PostIdOf<T>,
        CidNumberOf<T>,
        StorageReceiptIdOf<T>,
    >;

    /// 扣款余额类型。
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    /// 订阅关系键：`(订阅者账户, 收款主体)`，单 hasher 元组键（对齐 BFF storageMapKey）。
    pub type SubKeyOf<T> = (
        <T as frame_system::Config>::AccountId,
        IssuerKey<<T as frame_system::Config>::AccountId>,
    );

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 认证公民身份读取入口。
        type CitizenIdentity: SquarePostCitizenIdentityProvider<Self::AccountId>;

        /// 公民币扣款/收款货币。
        type Currency: Currency<Self::AccountId>;

        /// 链上共识挂钟（`pallet_timestamp`）；只用于记录扣款时间戳，链上不做日历运算。
        type TimeProvider: UnixTime;

        /// 机构账户查询：平台会员收款方=技术公司费用账户由此派生。
        type InstitutionAccountQuery: entity_primitives::InstitutionMultisigQuery<Self::AccountId>;

        /// 单个创作者档位数量上限（FRAME 强制的存储大小上限，非价格护栏）。
        #[pallet::constant]
        type MaxCreatorTiers: Get<u32>;

        #[pallet::constant]
        type MaxSquarePostIdLen: Get<u32>;
        #[pallet::constant]
        type MaxSquareCidNumberLen: Get<u32>;
        #[pallet::constant]
        type MaxSquareStorageReceiptIdLen: Get<u32>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 以 Worker 预生成的 post_id 作为链上发布索引主键。
    #[pallet::storage]
    pub type SquarePosts<T: Config> =
        StorageMap<_, Blake2_128Concat, PostIdOf<T>, SquarePostOf<T>, OptionQuery>;

    /// 每个钱包账户累计成功发布数量，供轻量统计和测试校验。
    #[pallet::storage]
    pub type PublishedPostCountByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

    /// 订阅关系真源：`(订阅者, 收款主体) -> 订阅状态`。单 hasher 元组键。
    #[pallet::storage]
    pub type Subscriptions<T: Config> =
        StorageMap<_, Blake2_128Concat, SubKeyOf<T>, SubscriptionState, OptionQuery>;

    /// 平台三档价（分）。链上可变真源，由技术公司经治理写入（治理在后续步接入）。
    #[pallet::storage]
    pub type PlatformPrice<T: Config> =
        StorageMap<_, Twox64Concat, MembershipLevel, u128, OptionQuery>;

    /// 技术公司 CID 绑定；`None` = 未绑定 → 平台轨 fail-closed 挂起（`PlatformNotBound`）。
    #[pallet::storage]
    pub type PlatformCidNumber<T: Config> = StorageValue<_, CidNumberOf<T>, OptionQuery>;

    /// 创作者档位表：`创作者钱包账户 -> 档位集合`。键=任意钱包账户，无 CID 要求。
    #[pallet::storage]
    pub type CreatorPlans<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<CreatorTier, T::MaxCreatorTiers>,
        ValueQuery,
    >;

    /// 续订触发方账户（keeper）。`charge_due` 仅允许此账户调用；`None` = 未设 → 续扣挂起。
    #[pallet::storage]
    pub type BillingKeeper<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 广场动态发布交易已入块。
        SquarePostPublished {
            post_id: PostIdOf<T>,
            owner_account: T::AccountId,
            cid_number: Option<CidNumberOf<T>>,
            post_category: SquarePostCategory,
            content_hash: [u8; 32],
            storage_receipt_id: StorageReceiptIdOf<T>,
            storage_until: u64,
            created_block: BlockNumberFor<T>,
        },
        /// 订阅已建立 / 恢复 Active。
        Subscribed {
            subscriber: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            plan: SubscriptionPlan,
        },
        /// 一次成功扣款（首扣或续扣）。
        Charged {
            subscriber: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            amount: BalanceOf<T>,
        },
        /// 续扣失败，订阅已翻 PastDue（欠费即停）。
        ChargeFailed {
            subscriber: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
        },
        /// 订阅已取消（记录保留）。
        Cancelled {
            subscriber: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyPostId,
        FieldTooLong,
        DuplicatePostId,
        EmptyContentHash,
        EmptyStorageReceiptId,
        EmptyStorageUntil,
        CampaignRequiresCitizen,
        /// 不能订阅自己（创作者订阅）。
        CannotSubscribeSelf,
        /// 订阅记录不存在。
        SubscriptionNotFound,
        /// 收款主体与档位类型不匹配。
        PlanIssuerMismatch,
        /// 平台该档价未设置。
        PlatformPriceNotSet,
        /// 技术公司 CID 未绑定 / 费用账户不可派生。
        PlatformNotBound,
        /// 创作者该档位不存在。
        CreatorTierNotFound,
        /// 调用者不是续订触发方（keeper）。
        NotBillingKeeper,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发布广场动态链上索引。
        ///
        /// `owner_account` 由 signed origin 派生；`cid_number` 由链上公民身份派生。
        /// 本调用不接收正文、图片或视频内容。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::publish_post())]
        pub fn publish_post(
            origin: OriginFor<T>,
            post_id: Vec<u8>,
            post_category: SquarePostCategory,
            content_hash: [u8; 32],
            storage_receipt_id: Vec<u8>,
            storage_until: u64,
        ) -> DispatchResult {
            let owner_account = ensure_signed(origin)?;
            ensure!(!post_id.is_empty(), Error::<T>::EmptyPostId);
            ensure!(
                content_hash.iter().any(|byte| *byte != 0),
                Error::<T>::EmptyContentHash
            );
            ensure!(
                !storage_receipt_id.is_empty(),
                Error::<T>::EmptyStorageReceiptId
            );
            ensure!(storage_until > 0, Error::<T>::EmptyStorageUntil);

            let post_id = PostIdOf::<T>::try_from(post_id).map_err(|_| Error::<T>::FieldTooLong)?;
            let storage_receipt_id = StorageReceiptIdOf::<T>::try_from(storage_receipt_id)
                .map_err(|_| Error::<T>::FieldTooLong)?;
            ensure!(
                !SquarePosts::<T>::contains_key(&post_id),
                Error::<T>::DuplicatePostId
            );

            let cid_number = Self::cid_number_for_owner(&owner_account)?;
            if post_category == SquarePostCategory::Campaign {
                ensure!(cid_number.is_some(), Error::<T>::CampaignRequiresCitizen);
            }

            let created_block = frame_system::Pallet::<T>::block_number();
            let square_post = SquarePost {
                post_id: post_id.clone(),
                owner_account: owner_account.clone(),
                cid_number: cid_number.clone(),
                post_category,
                content_hash,
                storage_receipt_id: storage_receipt_id.clone(),
                storage_until,
                created_block,
            };

            SquarePosts::<T>::insert(&post_id, square_post);
            PublishedPostCountByAccount::<T>::mutate(&owner_account, |count| {
                *count = count.saturating_add(1);
            });

            Self::deposit_event(Event::SquarePostPublished {
                post_id,
                owner_account,
                cid_number,
                post_category,
                content_hash,
                storage_receipt_id,
                storage_until,
                created_block,
            });
            Ok(())
        }

        /// 订阅平台会员或创作者会员（热钱包标准 extrinsic + 生物识别）。首扣即时完成。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::subscribe())]
        pub fn subscribe(
            origin: OriginFor<T>,
            issuer: IssuerKey<T::AccountId>,
            plan: SubscriptionPlan,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_subscribe(who, issuer, plan)
        }

        /// 取消订阅（写 Cancelled 保留记录，停止续扣）。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::cancel())]
        pub fn cancel(origin: OriginFor<T>, issuer: IssuerKey<T::AccountId>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cancel(who, issuer)
        }

        // call_index(3) = set_creator_plans 预留（第2步创作者会员）。

        /// 续扣：仅允许续订触发方（keeper）调用；收到即扣一次，链上不判到期。
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::charge_due())]
        pub fn charge_due(
            origin: OriginFor<T>,
            subscriber: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                BillingKeeper::<T>::get().as_ref() == Some(&who),
                Error::<T>::NotBillingKeeper
            );
            Self::do_charge_due(subscriber, issuer)
        }

        // call_index(5) = propose_set_platform_price 预留（第3步平台改价治理）。
    }

    impl<T: Config> Pallet<T> {
        fn cid_number_for_owner(
            owner_account: &T::AccountId,
        ) -> Result<Option<CidNumberOf<T>>, Error<T>> {
            T::CitizenIdentity::cid_number(owner_account)
                .map(CidNumberOf::<T>::try_from)
                .transpose()
                .map_err(|_| Error::<T>::FieldTooLong)
        }
    }
}

#[cfg(test)]
mod tests;
