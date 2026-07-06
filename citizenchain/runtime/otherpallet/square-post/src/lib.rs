#![cfg_attr(not(feature = "std"), no_std)]
//! # 广场动态链上发布索引模块 (square-post)
//!
//! 本 pallet 只记录广场动态的链上发布事实、内容哈希和存储回执。
//! 图片、视频、正文附件和 manifest 必须保存在独立内容存储网络，不得写入 runtime storage。

pub use pallet::*;
pub mod weights;

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
    use frame_support::{ensure, pallet_prelude::*};
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

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 认证公民身份读取入口。
        type CitizenIdentity: SquarePostCitizenIdentityProvider<Self::AccountId>;

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

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 广场动态发布交易已入块。
        ///
        /// 事件只包含发布索引和哈希；媒体内容仍由 R2/Worker 负责。
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发布广场动态链上索引。
        ///
        /// `owner_account` 由 signed origin 派生；`cid_number` 由链上公民身份派生。
        /// 本调用不接收正文、图片或视频内容。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::publish_square_post())]
        pub fn publish_square_post(
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
