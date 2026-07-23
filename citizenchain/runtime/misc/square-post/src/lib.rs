#![cfg_attr(not(feature = "std"), no_std)]
//! # 广场内容业务模块 (square-post)
//!
//! 本 pallet 是「广场内容域」的唯一链上模块，承载两类职责：
//! 1. 广场动态链上发布索引（`publish_post`）——只记录发布事实、内容哈希和存储回执，
//!    媒体内容必须保存在独立内容存储网络，不得写入 runtime storage。
//! 2. 会员订阅自动扣款核（公民币单轨）——用户签名订阅后由 runtime 按共识时间戳和
//!    真实 UTC 公历自动扣款，只有用户签名取消才撤销持续扣款授权。
//!
//! 订阅类型和公历换算见 [`subscription`]，自动扣款见 [`billing`]。
//! 开发期零用户、重新创世模型：平台三档价由创世播种，不设任何迁移。

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod billing;
pub mod proposal;
pub mod subscription;
pub mod weights;

pub use subscription::{
    BillingPeriod, CreatorTier, CreatorTiers, IssuerKey, MembershipLevel, PeriodPrice,
    PeriodPrices, SubscriptionPlan, SubscriptionState, SubscriptionStatus, SuspendReason, TierId,
};

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// 统一投票引擎 ProposalData 的模块前缀。
pub const MODULE_TAG: &[u8] = b"sqr-sub";

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
    /// 普通动态，所有账户都可以发布。
    Normal = 0,
    /// 竞选动态，必须是已绑定 CID 的认证公民账户。
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
    pub owner_account_id: AccountId,
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
    fn cid_number(owner_account_id: &AccountId) -> Option<Vec<u8>>;
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

    pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
    pub(crate) const FREEDOM_PRICE_FEN: u128 = 199_900;
    pub(crate) const DEMOCRACY_PRICE_FEN: u128 = 599_900;
    pub(crate) const SPARK_PRICE_FEN: u128 = 5_999_900;

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
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

        /// 链上共识挂钟（`pallet_timestamp`）；自动扣款只使用该共识时间源。
        type TimeProvider: UnixTime;

        /// 机构账户查询：平台会员收款方=公民链基金会费用账户由此派生。
        type InstitutionAccountQuery: entity_primitives::InstitutionMultisigQuery<Self::AccountId>;

        /// 平台调价只能经统一内部投票引擎创建提案。
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        /// 公民链基金会岗位业务授权真源。
        type InstitutionRoleAuthorization: entity_primitives::InstitutionRoleAuthorizationQuery<
            Self::AccountId,
        >;

        #[pallet::constant]
        type MaxSquarePostIdLen: Get<u32>;
        #[pallet::constant]
        type MaxSquareCidNumberLen: Get<u32>;
        #[pallet::constant]
        type MaxSquareStorageReceiptIdLen: Get<u32>;
        /// 单区块最多处理的到期周期数；包含链停后恢复时的历史到期周期。
        #[pallet::constant]
        type MaxSubscriptionRenewalsPerBlock: Get<u32>;
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 以 Worker 预生成的 post_id 作为链上发布索引主键。
    #[pallet::storage]
    pub type SquarePosts<T: Config> =
        StorageMap<_, Blake2_128Concat, PostIdOf<T>, SquarePostOf<T>, OptionQuery>;

    /// 每个账户累计成功发布数量，供轻量统计和测试校验。
    #[pallet::storage]
    pub type PublishedPostCountByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

    /// 订阅关系真源：`(订阅者, 收款主体) -> 订阅状态`。单 hasher 元组键。
    #[pallet::storage]
    pub type Subscriptions<T: Config> =
        StorageMap<_, Blake2_128Concat, SubKeyOf<T>, SubscriptionState, OptionQuery>;

    /// 自动扣款时间索引。第一键使用大端时间戳，使 storage 迭代顺序等于真实时间顺序。
    #[pallet::storage]
    pub type RenewalSchedule<T: Config> =
        StorageDoubleMap<_, Identity, [u8; 8], Blake2_128Concat, SubKeyOf<T>, (), OptionQuery>;

    /// 订阅关系到当前扣款时间的反向索引，供取消和换档精确移除旧调度。
    #[pallet::storage]
    pub type RenewalIndex<T: Config> =
        StorageMap<_, Blake2_128Concat, SubKeyOf<T>, u64, OptionQuery>;

    /// 平台三档价（分）。链上可变真源，由公民链基金会经治理写入。
    #[pallet::storage]
    pub type PlatformPrice<T: Config> =
        StorageMap<_, Twox64Concat, MembershipLevel, u128, OptionQuery>;

    /// 创作者链上付款套餐；名称、说明、权益和媒体仍只保存在 Cloudflare/D1。
    #[pallet::storage]
    pub type CreatorPlans<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CreatorTiers, ValueQuery>;

    /// 平台三档价创世播种：重新创世时无条件写入默认价，之后仅经统一内部投票调整。
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub _marker: PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _marker: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            PlatformPrice::<T>::insert(MembershipLevel::Freedom, FREEDOM_PRICE_FEN);
            PlatformPrice::<T>::insert(MembershipLevel::Democracy, DEMOCRACY_PRICE_FEN);
            PlatformPrice::<T>::insert(MembershipLevel::Spark, SPARK_PRICE_FEN);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 广场动态发布交易已入块。
        SquarePostPublished {
            post_id: PostIdOf<T>,
            owner_account_id: T::AccountId,
            cid_number: Option<CidNumberOf<T>>,
            post_category: SquarePostCategory,
            content_hash: [u8; 32],
            storage_receipt_id: StorageReceiptIdOf<T>,
            storage_until: u64,
            created_block: BlockNumberFor<T>,
        },
        /// 首扣或 runtime 自动续费已经完成，并登记下一次真实公历扣款时间。
        SubscriptionCharged {
            subscriber_account_id: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            plan: SubscriptionPlan,
            price_fen: u128,
            charged_at: u64,
            paid_until: u64,
        },
        /// 已取消但尚未到期的相同计划恢复，当前已付周期不重复扣款。
        SubscriptionResumed {
            subscriber_account_id: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            paid_until: u64,
        },
        /// 续费被挂起（创作者改价待再签名 / 余额不足待充值再签），保留粉丝关系、退出续费调度。
        SubscriptionSuspended {
            subscriber_account_id: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            reason: SuspendReason,
            suspended_at: u64,
        },
        /// 创作者改价后订阅者到期前再签名，已授权价更新为当前价、当前周期不重复扣款。
        SubscriptionReconsented {
            subscriber_account_id: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            authorized_price_fen: u128,
        },
        /// 创作者掉平台会员，其粉丝订阅暂停扣费但保留、仍留调度，创作者恢复即自动续。
        SubscriptionCreatorPaused {
            subscriber_account_id: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            paused_at: u64,
        },
        /// 公历换算失效等真实失败，自动扣款已经终止。
        SubscriptionRenewalStopped {
            subscriber_account_id: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            stopped_at: u64,
        },
        /// 取消后 `paid_until` 之前的已付权益仍有效。
        SubscriptionCancelled {
            subscriber_account_id: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            paid_until: u64,
        },
        /// 换挡立即生效：`charged_now` 为本次折算实际扣款额，`paid_until` 为新到期时间。
        SubscriptionPlanChanged {
            subscriber_account_id: T::AccountId,
            issuer: IssuerKey<T::AccountId>,
            new_plan: SubscriptionPlan,
            charged_now: u128,
            paid_until: u64,
        },
        /// 创作者已经覆盖式更新自己的链上付款套餐。
        CreatorPlansSet {
            creator_account_id: T::AccountId,
            tier_count: u32,
        },
        /// 平台价格调整提案已交给统一投票引擎。
        PlatformPriceChangeProposed {
            proposal_id: u64,
            actor_cid_number: votingengine::types::CidNumber,
            membership_level: MembershipLevel,
            new_price_fen: u128,
        },
        /// 统一投票通过后，平台价格已落地。
        PlatformPriceChanged {
            proposal_id: u64,
            membership_level: MembershipLevel,
            old_price_fen: Option<u128>,
            new_price_fen: u128,
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
        /// 收款主体与签名条款不匹配。
        PlanIssuerMismatch,
        /// 平台该档价未设置。
        PlatformPriceNotSet,
        /// 公民链基金会费用账户不可派生（平台 CID 为创世固定常量，正常不发生，保留 fail-closed）。
        PlatformNotBound,
        /// 创作者不是当前有效平台会员（不能被订阅）。
        CreatorNotPlatformMember,
        /// 价格必须大于 0。
        ZeroPrice,
        /// 交易签名价已不等于当前平台价，客户端必须刷新后重新签名。
        SignedPriceChanged,
        /// 已付款周期内条款锁定，不在 runtime 做换档折算。
        TermsLocked,
        /// 共识时间戳增加真实公历周期后超出可表示范围。
        CalendarOverflow,
        /// 创作者付款档位不存在或缺少目标周期价格。
        CreatorPlanNotFound,
        EmptyTierId,
        DuplicateTierId,
        DuplicateBillingPeriod,
        TooManyCreatorTiers,
        /// 平台调价发起 CID 不是公民链基金会。
        NotPlatformInstitution,
        /// CID 无法解析为合法机构码。
        InvalidInstitution,
        /// 平台价格必须大于 0。
        InvalidPlatformPrice,
        /// 投票提案数据不存在或不属于本模块。
        ProposalActionNotFound,
        /// 投票尚未通过或提案作用域不匹配。
        ProposalNotPassed,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 到期续费在 on_idle 按当块剩余权重尽量排空，不静态预留最坏权重。
        /// Timestamp inherent 已于 on_idle 前写入，`now_ms` 可用。
        fn on_idle(_n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let per = T::WeightInfo::process_one_due();
            let cap = u64::from(T::MaxSubscriptionRenewalsPerBlock::get());
            let per_ref = per.ref_time();
            // 权重未计量（如测试 `()` 权重）时按 backstop 上限排空；否则按当块剩余权重估算笔数。
            let limit = if per_ref == 0 {
                cap
            } else {
                core::cmp::min(remaining_weight.ref_time() / per_ref, cap)
            } as u32;
            if limit == 0 {
                return Weight::zero();
            }
            let processed = Self::process_due_subscriptions(Self::now_ms(), limit);
            per.saturating_mul(u64::from(processed))
        }

        #[cfg(feature = "try-runtime")]
        fn try_state(_n: BlockNumberFor<T>) -> Result<(), sp_runtime::TryRuntimeError> {
            // 运行期不变量（非迁移专属）：订阅状态与到期调度双向索引必须一致。
            for (key, state) in Subscriptions::<T>::iter() {
                match state.subscription_status {
                    // 留在调度里的两态：CreatorPaused 的重试 due 不等于 paid_until，
                    // 故只校验「有调度项且双向一致」，不再绑定 paid_until。
                    SubscriptionStatus::Active | SubscriptionStatus::CreatorPaused => {
                        ensure!(
                            matches!(
                                RenewalIndex::<T>::get(&key),
                                Some(due) if RenewalSchedule::<T>::contains_key(due.to_be_bytes(), &key)
                            ),
                            "square-post try_state: scheduled subscription missing renewal entry"
                        );
                    }
                    SubscriptionStatus::Cancelled
                    | SubscriptionStatus::Terminated
                    | SubscriptionStatus::Suspended => {
                        ensure!(
                            !RenewalIndex::<T>::contains_key(&key),
                            "square-post try_state: inactive subscription remains scheduled"
                        );
                    }
                }
            }
            for (due_key, key, ()) in RenewalSchedule::<T>::iter() {
                ensure!(
                    RenewalIndex::<T>::get(&key) == Some(u64::from_be_bytes(due_key)),
                    "square-post try_state: renewal reverse index mismatch"
                );
            }
            Ok(())
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发布广场动态链上索引。
        ///
        /// `owner_account_id` 由 signed origin 派生；`cid_number` 由链上公民身份派生。
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
            let owner_account_id = ensure_signed(origin)?;
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

            let cid_number = Self::cid_number_for_owner(&owner_account_id)?;
            if post_category == SquarePostCategory::Campaign {
                ensure!(cid_number.is_some(), Error::<T>::CampaignRequiresCitizen);
            }

            let created_block = frame_system::Pallet::<T>::block_number();
            let square_post = SquarePost {
                post_id: post_id.clone(),
                owner_account_id: owner_account_id.clone(),
                cid_number: cid_number.clone(),
                post_category,
                content_hash,
                storage_receipt_id: storage_receipt_id.clone(),
                storage_until,
                created_block,
            };

            SquarePosts::<T>::insert(&post_id, square_post);
            PublishedPostCountByAccount::<T>::mutate(&owner_account_id, |count| {
                *count = count.saturating_add(1);
            });

            Self::deposit_event(Event::SquarePostPublished {
                post_id,
                owner_account_id,
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
            expected_price_fen: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_subscribe(who, issuer, plan, expected_price_fen)
        }

        /// 取消订阅（写 Cancelled 保留记录，停止续扣）。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::cancel())]
        pub fn cancel(origin: OriginFor<T>, issuer: IssuerKey<T::AccountId>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cancel(who, issuer)
        }

        /// 创作者覆盖式更新自己的链上付款套餐。
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::set_creator_plans(tiers.len() as u32))]
        pub fn set_creator_plans(origin: OriginFor<T>, tiers: Vec<CreatorTier>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_set_creator_plans(who, tiers)
        }

        /// 当前周期内只登记待切换计划；已到期时立即按目标当前价扣款。
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::change_subscription_plan())]
        pub fn change_subscription_plan(
            origin: OriginFor<T>,
            issuer: IssuerKey<T::AccountId>,
            new_plan: SubscriptionPlan,
            expected_price_fen: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_change_subscription_plan(who, issuer, new_plan, expected_price_fen)
        }

        /// 发起平台价格调整内部投票。
        /// 投票资格、岗位有效选民快照、机构阈值、计票和终态推进全部由统一投票引擎处理。
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::propose_set_platform_price())]
        pub fn propose_set_platform_price(
            origin: OriginFor<T>,
            actor_cid_number: votingengine::types::CidNumber,
            proposer_role_code: votingengine::types::RoleCode,
            membership_level: MembershipLevel,
            new_price_fen: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            crate::proposal::propose_price_change::<T>(
                who,
                actor_cid_number,
                proposer_role_code,
                membership_level,
                new_price_fen,
            )
        }
    }

    impl<T: Config> Pallet<T> {
        fn cid_number_for_owner(
            owner_account_id: &T::AccountId,
        ) -> Result<Option<CidNumberOf<T>>, Error<T>> {
            T::CitizenIdentity::cid_number(owner_account_id)
                .map(CidNumberOf::<T>::try_from)
                .transpose()
                .map_err(|_| Error::<T>::FieldTooLong)
        }
    }
}

pub use proposal::InternalVoteExecutor;

#[cfg(test)]
mod tests;
