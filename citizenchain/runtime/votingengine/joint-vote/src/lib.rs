//! # 联合投票 pallet (joint-vote)
//!
//! 国储会 / 省储会 / 省储行的加权多签投票模式 + 联合公投两阶段:
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

use primitives::china::china_cb::CHINA_CB;
use primitives::china::china_ch::CHINA_CH;
use primitives::count_const::{
    JOINT_VOTE_PASS_THRESHOLD, NRC_JOINT_VOTE_WEIGHT, PRB_JOINT_VOTE_WEIGHT, PRC_JOINT_VOTE_WEIGHT,
};

use votingengine::Proposal;

pub mod jointinternal;
pub mod jointreferendum;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

pub use pallet::*;

// ──────────────────────────────────────────────────────────────────
// 跨阶段共用纯函数(jointinternal 与 jointreferendum 都引用)
// ──────────────────────────────────────────────────────────────────

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

// ──────────────────────────────────────────────────────────────────
// pallet block(Config / storage / event / error / extrinsic)
// ──────────────────────────────────────────────────────────────────

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

    /// 联合公投记录:(proposal_id, CID 绑定哈希) → 赞成/反对。
    #[pallet::storage]
    pub type ReferendumVotesByBindingId<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, T::Hash, bool, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn referendum_tally)]
    pub type ReferendumTallies<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, votingengine::VoteCountU64, ValueQuery>;

    /// 总人口快照 nonce 防重放(全局维度)。
    #[pallet::storage]
    #[pallet::getter(fn used_population_snapshot_nonce)]
    pub type UsedPopulationSnapshotNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

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
    pub struct PreparedPopulationSnapshot<BlockNumber, Hash> {
        /// 中文注释：联合公投阶段可投票总人数，由投票引擎验签后缓存。
        pub eligible_total: u64,
        /// 中文注释：人口快照 nonce 哈希，用于审计和防重放。
        pub nonce_hash: Hash,
        /// 中文注释：准备快照所在区块。
        pub prepared_at: BlockNumber,
    }

    /// 已验签的人口快照缓存：发起联合提案时由投票引擎消费。
    #[pallet::storage]
    #[pallet::getter(fn pending_population_snapshot)]
    pub type PendingPopulationSnapshots<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        PreparedPopulationSnapshot<BlockNumberFor<T>, T::Hash>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 联合投票人口快照已由投票引擎验签并缓存。
        PopulationSnapshotPrepared {
            who: T::AccountId,
            eligible_total: u64,
            nonce_hash: T::Hash,
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
        /// 联合公投已投出一票(binding_id 为 CID 哈希)。
        ReferendumVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            binding_id: T::Hash,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 联合公投总分母未设置(eligible_total == 0)。
        CitizenEligibleTotalNotSet,
        /// 人口快照参数无效(nonce 为空/已使用/签名验证失败)。
        InvalidPopulationSnapshot,
        /// 发起联合提案前尚未准备人口快照。
        PopulationSnapshotNotPrepared,
        /// 人口快照不是当前区块准备的快照,不能代表提案发起时刻的公民分母。
        PopulationSnapshotNotCurrent,
        /// CID 资格校验未通过(binding_id 未绑定或不匹配)。
        CidNotEligible,
        /// CID 投票凭证验签失败或已被消费。
        InvalidCidVoteCredential,
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

        /// 联合公投阶段:CID 持有者按 >50% 严格多数投票。
        /// 业务实现挂在 [`super::jointreferendum`]。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_referendum())]
        pub fn cast_referendum(
            origin: OriginFor<T>,
            proposal_id: u64,
            binding_id: T::Hash,
            nonce: votingengine::pallet::VoteNonceOf<T>,
            signature: votingengine::pallet::VoteSignatureOf<T>,
            issuer_cid_number: BoundedVec<u8, ConstU32<128>>,
            issuer_main_account: T::AccountId,
            signer_pubkey: [u8; 32],
            scope_province_name: BoundedVec<u8, ConstU32<64>>,
            scope_city_name: BoundedVec<u8, ConstU32<64>>,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_jointreferendum_vote(
                who,
                proposal_id,
                binding_id,
                nonce,
                signature,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
                approve,
            )
        }

        /// 准备联合投票人口快照。
        ///
        /// 中文注释：人口快照、联合签名、nonce 防重放全部属于投票引擎。
        /// 业务模块只能在随后创建提案时消费已准备快照，不能再透传这些字段。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::prepare_joint_population_snapshot())]
        pub fn prepare_joint_population_snapshot(
            origin: OriginFor<T>,
            eligible_total: u64,
            snapshot_nonce: votingengine::pallet::VoteNonceOf<T>,
            signature: votingengine::pallet::VoteSignatureOf<T>,
            issuer_cid_number: BoundedVec<u8, ConstU32<128>>,
            issuer_main_account: T::AccountId,
            signer_pubkey: [u8; 32],
            scope_province_name: BoundedVec<u8, ConstU32<64>>,
            scope_city_name: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_prepare_population_snapshot(
                who,
                eligible_total,
                snapshot_nonce,
                signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &signer_pubkey,
                scope_province_name.as_slice(),
                scope_city_name.as_slice(),
            )
        }
    }
}

// ──────────────────────────────────────────────────────────────────
// trait 实现 — 业务方法住在 jointinternal / jointreferendum 子模块
// ──────────────────────────────────────────────────────────────────

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
        let result = ReferendumVotesByBindingId::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }

    fn cleanup_joint_terminal(proposal_id: u64) {
        JointTallies::<T>::remove(proposal_id);
        ReferendumTallies::<T>::remove(proposal_id);
    }
}
