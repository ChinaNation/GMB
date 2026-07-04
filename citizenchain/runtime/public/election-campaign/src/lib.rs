#![cfg_attr(not(feature = "std"), no_std)]
//! # 公权选举业务模块 (election-campaign)
//!
//! 本 pallet 是公权选举的业务壳，只负责承载“什么机构能发起什么选举、
//! 候选/选民快照如何生成、选举结果如何回写业务真源”等业务规则的接入位置。
//! 选举投票的创建、投票、计票、超时结算和清理流程必须继续归属
//! `election-vote`，本模块不得复刻任何投票流程。
//!
//! 当前版本只接入 runtime metadata，占位为后续真实选举规则服务：
//! - 不开放任何选举创建 extrinsic；
//! - 不调用 `election-vote`；
//! - 不写入 `public-admins` 或法定代表人；
//! - 不实现普选/互选的具体规则。

pub use pallet::*;

/// 模块标识前缀。后续如需把业务数据写入 votingengine ProposalData，必须使用本 tag。
pub const MODULE_TAG: &[u8] = b"ele-camp1";

/// 选举业务活动模式。具体含义由本模块后续规则解释，投票流程仍交给 election-vote。
#[derive(
    codec::Encode,
    codec::Decode,
    codec::DecodeWithMemTracking,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    scale_info::TypeInfo,
    frame_support::pallet_prelude::MaxEncodedLen,
)]
pub enum ElectionCampaignMode {
    /// 普选活动：由具备投票身份的公民按作用域投票。
    Popular,
    /// 互选活动：由机构现任成员或管理员在快照内投票。
    Mutual,
}

/// 选举业务活动状态。当前仅作为骨架类型，真实状态机后续再接入。
#[derive(
    codec::Encode,
    codec::Decode,
    codec::DecodeWithMemTracking,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    scale_info::TypeInfo,
    frame_support::pallet_prelude::MaxEncodedLen,
)]
pub enum ElectionCampaignStatus {
    /// 活动草稿或预留状态。
    Draft,
    /// 已创建选举投票提案。
    Opened,
    /// election-vote 已生成结果快照。
    ResultReady,
    /// 选举投票未通过或结果无效。
    Rejected,
    /// 业务活动已关闭。
    Closed,
}

/// 选举业务活动元数据骨架。
///
/// 本结构只定义业务壳未来需要保存的字段，不在当前版本写入 storage。
/// `vote_proposal_id` 对应 election-vote 生成的提案 ID；`organizer_account`
/// 表示组织选举的机构账户；`target_account` 表示被选职位所属机构账户。
#[derive(
    codec::Encode,
    codec::Decode,
    codec::DecodeWithMemTracking,
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_info::TypeInfo,
    frame_support::pallet_prelude::MaxEncodedLen,
)]
pub struct ElectionCampaignMeta<AccountId, BlockNumber, OfficeCode> {
    pub vote_proposal_id: u64,
    pub election_mode: ElectionCampaignMode,
    pub organizer_code: primitives::cid::code::InstitutionCode,
    pub organizer_account: AccountId,
    pub target_code: primitives::cid::code::InstitutionCode,
    pub target_account: AccountId,
    pub office_code: OfficeCode,
    pub rule_id: u32,
    pub seat_count: u16,
    pub term_start: BlockNumber,
    pub term_end: BlockNumber,
    pub campaign_status: ElectionCampaignStatus,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// 真实选举业务尚未接入。
        ElectionCampaignNotImplemented,
    }

    impl<T: Config> Pallet<T> {
        /// 返回选举业务模块标识，供测试和文档对齐。
        pub fn module_tag() -> &'static [u8] {
            crate::MODULE_TAG
        }

        /// 当前 runtime 只接入骨架，真实选举业务尚未启用。
        pub fn is_enabled() -> bool {
            false
        }
    }
}
