//! # 联合投票 pallet (joint-vote)
//!
//! 国家储委会 / 省储委会 / 省储行的加权多签投票模式 + 联合公投两阶段:
//! - [`jointinternal`]:内部投票阶段 — 业务函数 `do_create_joint_proposal` /
//!   `do_joint_vote` / `do_finalize_joint_timeout` 等。
//! - [`jointreferendum`]:联合公投阶段 — 业务函数 `do_jointreferendum_vote` /
//!   `do_finalize_jointreferendum_timeout`。
//!
//! 共用基础设施仍归 [`votingengine`] 引擎核心,本 pallet 通过
//! `Config: votingengine::Config` 直接访问 `Proposals` / `AdminSnapshot` 等共用 storage。

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode;
use frame_support::pallet_prelude::DispatchResult;
use sp_runtime::DispatchError;

use primitives::cid::china::china_cb::CHINA_CB;
use primitives::cid::china::china_ch::CHINA_CH;
use primitives::count_const::{
    JOINT_VOTE_PASS_THRESHOLD, NRC_JOINT_VOTE_WEIGHT, PRB_JOINT_VOTE_WEIGHT, PRC_JOINT_VOTE_WEIGHT,
};

use votingengine::{PopulationScope, Proposal};

pub mod jointinternal;
pub mod jointreferendum;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

pub use pallet::*;
// 跨阶段共用纯函数(jointinternal 与 jointreferendum 都引用)
pub(crate) fn decode_account<T: frame_system::Config>(raw: &[u8; 32]) -> Option<T::AccountId> {
    T::AccountId::decode(&mut &raw[..]).ok()
}

pub(crate) fn nrc_account<T: frame_system::Config>() -> Option<T::AccountId> {
    CHINA_CB
        .first()
        .and_then(|n| decode_account::<T>(&n.main_account))
}

fn raw_account_matches<T: frame_system::Config>(raw: &[u8; 32], id: &T::AccountId) -> bool {
    decode_account::<T>(raw).as_ref() == Some(id)
}

/// 机构多签账户 → 联合投票票权(NRC=19 / PRC=1×43 / PRB=1×43，总票权=105)。
pub fn institution_info<T: frame_system::Config>(id: &T::AccountId) -> Option<u32> {
    if CHINA_CB
        .first()
        .map(|n| raw_account_matches::<T>(&n.main_account, id))
        .unwrap_or(false)
    {
        return Some(NRC_JOINT_VOTE_WEIGHT);
    }
    if CHINA_CB
        .iter()
        .skip(1)
        .any(|n| raw_account_matches::<T>(&n.main_account, id))
    {
        return Some(PRC_JOINT_VOTE_WEIGHT);
    }
    if CHINA_CH
        .iter()
        .any(|n| raw_account_matches::<T>(&n.main_account, id))
    {
        return Some(PRB_JOINT_VOTE_WEIGHT);
    }
    None
}

/// 105 票全票通过判定。
pub fn is_joint_unanimous(yes_weight: u32) -> bool {
    yes_weight >= JOINT_VOTE_PASS_THRESHOLD
}

/// 联合公投通过判定:严格 > 50%。
pub fn is_jointreferendum_vote_passed(yes_votes: u64, eligible_total: u64) -> bool {
    if eligible_total == 0 {
        return false;
    }
    (yes_votes as u128).saturating_mul(100) > (eligible_total as u128).saturating_mul(50)
}

/// 联合公投否决判定:反对票 ≥ 50% 即否决。
pub fn is_jointreferendum_vote_rejected(no_votes: u64, eligible_total: u64) -> bool {
    if eligible_total == 0 {
        return false;
    }
    (no_votes as u128).saturating_mul(100) >= (eligible_total as u128).saturating_mul(50)
}
// pallet block(Config / storage / event / error / extrinsic)
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// pallet 自身 StorageVersion。
    /// 全新创世口径:创世即终态布局,storage 版本恒为 v1,不承载历史迁移。
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 联合投票内部投票阶段管理员级记录:(proposal_id, (机构, 管理员公钥)) → 赞成/反对。
    #[pallet::storage]
    pub type JointVotesByAdmin<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        (T::AccountId, T::AccountId),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn joint_institution_tally)]
    pub type JointInstitutionTallies<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        votingengine::VoteCountU32,
        ValueQuery,
    >;

    /// 联合投票机构级汇总:(proposal_id, 机构) → 赞成/反对。
    #[pallet::storage]
    pub type JointVotesByInstitution<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn joint_tally)]
    pub type JointTallies<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, votingengine::VoteCountU32, ValueQuery>;

    /// 联合公投记录:(proposal_id, 公民钱包账户) → 赞成/反对。
    #[pallet::storage]
    pub type ReferendumVotesByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn referendum_tally)]
    pub type ReferendumTallies<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, votingengine::VoteCountU64, ValueQuery>;

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    pub struct PreparedPopulationSnapshot<BlockNumber> {
        /// 联合公投阶段可投票总人数，由投票引擎从链上公民身份模块读取后缓存。
        pub eligible_total: u64,
        /// 人口统计作用域，后续联合公投资格按同一作用域读取。
        pub scope: PopulationScope,
        /// 准备快照所在区块。
        pub prepared_at: BlockNumber,
    }

    /// 已准备的人口快照缓存：发起联合提案时由投票引擎消费。
    #[pallet::storage]
    #[pallet::getter(fn pending_population_snapshot)]
    pub type PendingPopulationSnapshots<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        PreparedPopulationSnapshot<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// 联合公投提案的人口作用域：proposal_id → scope。
    #[pallet::storage]
    pub type ReferendumScopes<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, PopulationScope, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 联合投票人口快照已由投票引擎读取并缓存。
        PopulationSnapshotPrepared {
            who: T::AccountId,
            eligible_total: u64,
            scope: PopulationScope,
        },
        /// 联合投票中某机构管理员已投出一票。
        JointAdminVoteCast {
            proposal_id: u64,
            institution: T::AccountId,
            who: T::AccountId,
            approve: bool,
        },
        /// 联合投票中某机构已形成最终结果(赞成/反对)。
        JointInstitutionVoteFinalized {
            proposal_id: u64,
            institution: T::AccountId,
            approved: bool,
        },
        /// 联合公投已投出一票。
        ReferendumVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 联合公投总分母未设置(eligible_total == 0)。
        CitizenEligibleTotalNotSet,
        /// 人口快照参数无效或作用域没有可投票公民。
        InvalidPopulationSnapshot,
        /// 发起联合提案前尚未准备人口快照。
        PopulationSnapshotNotPrepared,
        /// 人口快照不是当前区块准备的快照,不能代表提案发起时刻的公民分母。
        PopulationSnapshotNotCurrent,
        /// 公民身份投票资格校验未通过。
        CitizenNotEligible,
        /// 公投作用域缺失。
        PopulationScopeMissing,
    }

    use crate::weights::WeightInfo;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 联合投票内部投票阶段:NRC/PRC/PRB 管理员按机构投票。
        /// 业务实现挂在 [`super::jointinternal`]。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_admin())]
        pub fn cast_admin(
            origin: OriginFor<T>,
            proposal_id: u64,
            institution: T::AccountId,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_joint_vote(who, proposal_id, institution, approve)
        }

        /// 联合公投阶段:链上公民身份持有者按 >50% 严格多数投票。
        /// 业务实现挂在 [`super::jointreferendum`]。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_referendum())]
        pub fn cast_referendum(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_jointreferendum_vote(who, proposal_id, approve)
        }

        /// 准备联合投票人口快照。
        ///
        /// 人口快照由投票引擎从 citizen-identity 链上状态直接读取。
        /// 业务模块只能在随后创建提案时消费已准备快照，不能再透传这些字段。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::prepare_joint_population_snapshot())]
        pub fn prepare_joint_population_snapshot(
            origin: OriginFor<T>,
            scope: PopulationScope,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_prepare_joint_population_snapshot(who, scope)
        }
    }
}
// trait 实现 — 业务方法住在 jointinternal / jointreferendum 子模块
impl<T: Config> votingengine::JointVoteEngine<T::AccountId> for Pallet<T> {
    fn create_joint_proposal(who: T::AccountId) -> Result<u64, DispatchError> {
        Self::do_create_joint_proposal(who)
    }

    fn create_joint_proposal_with_data(
        who: T::AccountId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        frame_support::storage::with_transaction(|| {
            let proposal_id = match Self::do_create_joint_proposal(who) {
                Ok(id) => id,
                Err(err) => return frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            };
            let now = <frame_system::Pallet<T>>::block_number();
            match <votingengine::Pallet<T>>::register_proposal_data(
                proposal_id,
                module_tag,
                data,
                now,
            ) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_joint_proposal_with_data_and_object(
        who: T::AccountId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
        object_kind: u8,
        object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        frame_support::storage::with_transaction(|| {
            let proposal_id = match Self::do_create_joint_proposal(who) {
                Ok(id) => id,
                Err(err) => return frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            };
            let now = <frame_system::Pallet<T>>::block_number();
            if let Err(err) = <votingengine::Pallet<T>>::register_proposal_data(
                proposal_id,
                module_tag,
                data,
                now,
            ) {
                return frame_support::storage::TransactionOutcome::Rollback(Err(err));
            }
            match <votingengine::Pallet<T>>::store_proposal_object(
                proposal_id,
                object_kind,
                object_data,
            ) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            }
        })
    }
}

impl<T: Config>
    votingengine::traits::JointProposalFinalizer<
        frame_system::pallet_prelude::BlockNumberFor<T>,
        T::AccountId,
    > for Pallet<T>
{
    fn finalize_joint_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_joint_timeout(proposal, proposal_id)
    }

    fn finalize_jointreferendum_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_jointreferendum_timeout(proposal, proposal_id)
    }
}

impl<T: Config> votingengine::traits::JointCleanupHandler for Pallet<T> {
    fn cleanup_joint_admin_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::traits::CleanupChunkResult {
        let result = JointVotesByAdmin::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }
    fn cleanup_joint_institution_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::traits::CleanupChunkResult {
        let result = JointVotesByInstitution::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }
    fn cleanup_joint_institution_tallies_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::traits::CleanupChunkResult {
        let result = JointInstitutionTallies::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }
    fn cleanup_referendum_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::traits::CleanupChunkResult {
        let result = ReferendumVotesByAccount::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }

    fn cleanup_joint_terminal(proposal_id: u64) {
        JointTallies::<T>::remove(proposal_id);
        ReferendumTallies::<T>::remove(proposal_id);
        ReferendumScopes::<T>::remove(proposal_id);
    }
}
