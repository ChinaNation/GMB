//! # 内部投票 pallet (internal-vote)
//!
//! 所有机构与个人多签共用的“管理员一人一票”投票程序。
//!
//! 本模块负责内部投票模式准入、机构上下文、管理员快照、计票和终态，不判断
//! 某个机构能否发起转账、销毁或密钥变更等具体业务。有效准入必须同时通过
//! 投票引擎的模式校验与调用方业务 pallet 的业务权限校验。
//!
//! 共用基础设施(Proposals 主 storage / 双层 ID / 反向索引 / 状态机骨架 / 快照 / 锁 / 清理)
//! 仍归 [`votingengine`] 引擎核心,本 pallet 通过 `Config: votingengine::Config` 直接访问。
//!
//! 本 pallet 自有:
//! - storage:`InternalVotesByAccount` / `InternalTallies` / `InternalThresholdSnapshot`
//! - event:`InternalVoteCast`
//! - error:`InvalidInternalCode` / `MissingThresholdSnapshot` / `InvalidThresholdSnapshot`
//! - extrinsic:`cast(proposal_id, approve)`
//! - 业务函数:`do_create_internal_proposal*` / `do_internal_vote` / `do_finalize_internal_timeout`
//! - trait impl:`InternalVoteEngine`(供业务 pallet 创建提案)
//! - trait impl:`InternalProposalFinalizer`(votingengine 主 pallet finalize 路径反向调用)
//! - trait impl:`InternalCleanupHandler`(votingengine 主 pallet cleanup 状态机反向调用)

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure,
    pallet_prelude::{BoundedVec, DispatchResult},
    storage::{with_transaction, TransactionOutcome},
};
use scale_info::TypeInfo;
use sp_runtime::traits::{SaturatedConversion, Saturating};
use sp_runtime::{DispatchError, RuntimeDebug};

use primitives::cid::china::china_cb::CHINA_CB;
use primitives::cid::china::china_ch::CHINA_CH;
use primitives::cid::china::china_sf::CHINA_SF;
use primitives::cid::china::china_zf::CHINA_ZF;
use primitives::count_const::VOTING_DURATION_BLOCKS;

use votingengine::{
    pallet::{AdminSnapshot, Proposals},
    types::{
        fixed_governance_pass_threshold, institution_code_from_cid_number, is_personal_code,
        is_registered_multisig_code, is_valid_governance_code, InstitutionCode,
        ProposalSubjectCidNumbers, FRG, NJD, NRC, PRB, PRC,
    },
    InternalAdminProvider, InternalProposalMutexKind, Proposal, PROPOSAL_KIND_INTERNAL,
    STAGE_INTERNAL, STATUS_EXECUTED, STATUS_EXECUTION_FAILED, STATUS_PASSED, STATUS_REJECTED,
};

pub mod weights;

mod cleanup;
mod proposal;
mod threshold;
mod vote;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

pub use pallet::*;

#[cfg(test)]
mod tests;

/// 内部提案语义分类。
///
/// 这是投票引擎内部状态，不是业务模块自定义类型；用于在业务执行成功后
/// 激活/删除动态阈值，避免业务模块自己维护投票阈值。
#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub enum InternalProposalRole {
    General,
    LifecycleCreate,
    LifecycleClose,
    AdminChange,
}

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct PendingAdminChangeThreshold<AccountId> {
    pub institution_code: InstitutionCode,
    pub account: AccountId,
    pub new_admins_len: u32,
    pub new_threshold: u32,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// 重新创世直接使用最终 proposal_id 键控布局，不保留开发期旧存储迁移。
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

    /// 内部投票记录:(proposal_id, 管理员公钥) → 赞成/反对。防止同一管理员重复投票。
    #[pallet::storage]
    pub type InternalVotesByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn internal_tally)]
    pub type InternalTallies<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, votingengine::VoteCountU32, ValueQuery>;

    /// 内部投票阈值快照:提案创建时锁定阈值,投票期间不受账户状态变化影响。
    #[pallet::storage]
    pub type InternalThresholdSnapshot<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, u32, OptionQuery>;

    /// 注册多签待激活动态阈值:proposal_id -> threshold。
    ///
    /// 注册提案发起时写入，提案执行成功后移动到 ActiveDynamicThresholds。
    #[pallet::storage]
    pub type PendingDynamicThresholds<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, u32, OptionQuery>;

    /// 注册多签已激活动态阈值:(institution_code, account) -> threshold。
    ///
    /// 一般内部投票只从这里读取动态阈值，不再读取 admins 模块。
    #[pallet::storage]
    pub type ActiveDynamicThresholds<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        InstitutionCode,
        Blake2_128Concat,
        T::AccountId,
        u32,
        OptionQuery,
    >;

    /// 管理员变更提案待应用的新动态阈值。
    #[pallet::storage]
    pub type PendingAdminChangeThresholds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        PendingAdminChangeThreshold<T::AccountId>,
        OptionQuery,
    >;

    /// 内部提案语义分类。用于终态副作用，不交给业务模块判断。
    #[pallet::storage]
    pub type InternalProposalRoles<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, InternalProposalRole, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 内部投票已投出一票。
        InternalVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 内部投票的机构类型不合法。
        InvalidInternalCode,
        /// 内部投票阈值快照缺失。
        MissingThresholdSnapshot,
        /// 内部投票阈值与管理员快照人数不匹配。
        InvalidThresholdSnapshot,
        /// 注册多签动态阈值不满足严格过半规则。
        InvalidDynamicThreshold,
        /// 动态阈值配置缺失。
        MissingDynamicThreshold,
    }

    use crate::weights::WeightInfo;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 内部投票:管理员一人一票。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::cast())]
        pub fn cast(origin: OriginFor<T>, proposal_id: u64, approve: bool) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_internal_vote(who, proposal_id, approve)
        }
    }
}
// 内部判定 helper
fn decode_account<T: Config>(raw: &[u8; 32]) -> Option<T::AccountId> {
    T::AccountId::decode(&mut &raw[..]).ok()
}

fn is_valid_account_context<T: Config>(
    institution_code: InstitutionCode,
    account_context: T::AccountId,
) -> bool {
    match institution_code {
        NRC => CHINA_CB
            .first()
            .and_then(|n| decode_account::<T>(&n.main_account))
            .map(|nrc| account_context == nrc)
            .unwrap_or(false),
        PRC => CHINA_CB
            .iter()
            .skip(1)
            .filter_map(|n| decode_account::<T>(&n.main_account))
            .any(|pid| pid == account_context),
        PRB => CHINA_CH
            .iter()
            .filter_map(|n| decode_account::<T>(&n.main_account))
            .any(|pid| pid == account_context),
        // FRG 是一个机构、一个主账户、215 名管理员。省域 5 人岗位组属于具体
        // 注册业务权限，不得在通用内部投票引擎中把 FRG 误判成“管理员恰好 5 人”。
        FRG => CHINA_ZF
            .iter()
            .find(|n| institution_code_from_cid_number(n.cid_number) == Some(FRG))
            .and_then(|n| decode_account::<T>(&n.main_account))
            .map(|frg| account_context == frg)
            .unwrap_or(false),
        NJD => CHINA_SF
            .iter()
            .find(|n| institution_code_from_cid_number(n.cid_number) == Some(NJD))
            .and_then(|n| decode_account::<T>(&n.main_account))
            .map(|njd| account_context == njd)
            .unwrap_or(false),
        c if is_registered_multisig_code(&c) => {
            <T as votingengine::Config>::InternalAdminProvider::get_admin_list(c, account_context)
                .is_some()
        }
        _ => false,
    }
}

fn is_internal_admin<T: Config>(
    institution_code: InstitutionCode,
    institution: T::AccountId,
    who: &T::AccountId,
) -> bool {
    <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
        institution_code,
        institution,
        who,
    )
}

fn active_internal_threshold<T: Config>(
    institution_code: InstitutionCode,
    institution: T::AccountId,
) -> Option<u32> {
    match institution_code {
        NRC | PRC | PRB | FRG | NJD => fixed_governance_pass_threshold(&institution_code),
        c if primitives::institution_constraints::is_permanent_singleton_code(&c) => None,
        c if is_registered_multisig_code(&c) => ActiveDynamicThresholds::<T>::get(c, institution),
        _ => None,
    }
}
// 业务方法
// trait 实现
impl<T: Config> votingengine::InternalVoteEngine<T::AccountId> for Pallet<T> {
    fn create_general_internal_proposal_with_data(
        who: T::AccountId,
        institution_code: InstitutionCode,
        institution: T::AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id = match Self::do_create_general_internal_proposal(
                who.clone(),
                institution_code,
                institution,
                subject_cid_numbers,
            ) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            match Self::register_data_and_auto_approve(who, proposal_id, module_tag, data) {
                Ok(id) => TransactionOutcome::Commit(Ok(id)),
                Err(err) => TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_lifecycle_internal_proposal_with_data(
        who: T::AccountId,
        institution_code: InstitutionCode,
        institution: T::AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id = match Self::do_create_lifecycle_internal_proposal(
                who.clone(),
                institution_code,
                institution,
                subject_cid_numbers,
            ) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            match Self::register_data_and_auto_approve(who, proposal_id, module_tag, data) {
                Ok(id) => TransactionOutcome::Commit(Ok(id)),
                Err(err) => TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_registered_account_create_proposal_with_data(
        who: T::AccountId,
        institution_code: InstitutionCode,
        institution: T::AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        admins: sp_std::vec::Vec<T::AccountId>,
        dynamic_threshold: u32,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id = match Self::do_create_registered_account_create_proposal(
                who.clone(),
                institution_code,
                institution,
                subject_cid_numbers,
                admins,
                dynamic_threshold,
            ) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            match Self::register_data_and_auto_approve(who, proposal_id, module_tag, data) {
                Ok(id) => TransactionOutcome::Commit(Ok(id)),
                Err(err) => TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_admin_change_internal_proposal_with_data(
        who: T::AccountId,
        institution_code: InstitutionCode,
        institution: T::AccountId,
        subject_cid_numbers: sp_std::vec::Vec<sp_std::vec::Vec<u8>>,
        new_admins_len: u32,
        new_threshold: u32,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id = match Self::do_create_admin_change_internal_proposal(
                who.clone(),
                institution_code,
                institution,
                subject_cid_numbers,
                new_admins_len,
                new_threshold,
            ) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            match Self::register_data_and_auto_approve(who, proposal_id, module_tag, data) {
                Ok(id) => TransactionOutcome::Commit(Ok(id)),
                Err(err) => TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn register_active_dynamic_threshold_direct(
        institution_code: InstitutionCode,
        institution: T::AccountId,
        admins_len: u32,
        threshold: u32,
    ) -> DispatchResult {
        // 直设阈值只针对注册动态多签主体；固定治理机构走代码级阈值，六个国家
        // 单例按每个提案的 admins 快照计算严格过半，二者都不允许写本 storage。
        ensure!(
            is_registered_multisig_code(&institution_code)
                && !primitives::institution_constraints::is_permanent_singleton_code(
                    &institution_code
                ),
            Error::<T>::InvalidInternalCode
        );
        Self::ensure_dynamic_threshold(admins_len, threshold)?;
        ActiveDynamicThresholds::<T>::insert(institution_code, institution, threshold);
        Ok(())
    }

    fn active_dynamic_threshold(
        institution_code: InstitutionCode,
        institution: T::AccountId,
    ) -> Option<u32> {
        if primitives::institution_constraints::is_permanent_singleton_code(&institution_code) {
            return None;
        }
        ActiveDynamicThresholds::<T>::get(institution_code, institution)
    }

    fn configured_dynamic_threshold(
        proposal_id: u64,
        institution_code: InstitutionCode,
        institution: T::AccountId,
    ) -> Option<u32> {
        if primitives::institution_constraints::is_permanent_singleton_code(&institution_code) {
            return None;
        }
        PendingDynamicThresholds::<T>::get(proposal_id)
            .or_else(|| ActiveDynamicThresholds::<T>::get(institution_code, institution))
    }
}

impl<T: Config>
    votingengine::traits::InternalProposalFinalizer<
        frame_system::pallet_prelude::BlockNumberFor<T>,
        T::AccountId,
    > for Pallet<T>
{
    fn finalize_internal_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_internal_timeout(proposal, proposal_id)
    }
}
