//! # 内部投票 pallet (internal-vote)
//!
//! 治理机构 / 注册多签的"管理员一人一票"投票模式。
//!
//! 共用基础设施(Proposals 主 storage / 双层 ID / 反向索引 / 状态机骨架 / 快照 / 锁 / 清理)
//! 仍归 [`votingengine`] 引擎核心,本 pallet 通过 `Config: votingengine::Config` 直接访问。
//!
//! 本 pallet 自有:
//! - storage:`InternalVotesByAccount` / `InternalTallies` / `InternalThresholdSnapshot`
//! - event:`InternalVoteCast`
//! - error:`InvalidInternalOrg` / `MissingThresholdSnapshot` / `InvalidThresholdSnapshot`
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

use primitives::china::china_cb::CHINA_CB;
use primitives::china::china_ch::CHINA_CH;
use primitives::count_const::VOTING_DURATION_BLOCKS;

use votingengine::{
    pallet::{AdminSnapshot, Proposals},
    types::{
        fixed_governance_pass_threshold, is_registered_multisig_org, is_valid_org, ORG_NRC,
        ORG_PRB, ORG_PRC,
    },
    InternalAdminProvider, InternalProposalMutexKind, Proposal, PROPOSAL_KIND_INTERNAL,
    STAGE_INTERNAL, STATUS_EXECUTED, STATUS_EXECUTION_FAILED, STATUS_PASSED, STATUS_REJECTED,
};

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

pub use pallet::*;

#[cfg(test)]
mod tests;

/// 内部提案语义分类。
///
/// 中文注释：这是投票引擎内部状态，不是业务模块自定义类型；用于在业务执行成功后
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
    pub org: u8,
    pub account: AccountId,
    pub new_admins_len: u32,
    pub new_threshold: u32,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// pallet 自身 StorageVersion。
    /// v2:动态阈值归属 internal-vote，删除业务显式投票阈值入口。
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

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

    /// 注册多签待激活动态阈值:(org, account) -> threshold。
    ///
    /// 中文注释：注册提案发起时写入，提案执行成功后移动到 ActiveDynamicThresholds。
    #[pallet::storage]
    pub type PendingDynamicThresholds<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u8, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    /// 注册多签已激活动态阈值:(org, account) -> threshold。
    ///
    /// 中文注释：一般内部投票只从这里读取动态阈值，不再读取 admins-change。
    #[pallet::storage]
    pub type ActiveDynamicThresholds<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u8, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

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
        InvalidInternalOrg,
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

// ──────────────────────────────────────────────────────────────────
// 内部判定 helper
// ──────────────────────────────────────────────────────────────────

fn decode_account<T: Config>(raw: &[u8; 32]) -> Option<T::AccountId> {
    T::AccountId::decode(&mut &raw[..]).ok()
}

fn is_valid_internal_institution<T: Config>(org: u8, institution: T::AccountId) -> bool {
    match org {
        ORG_NRC => CHINA_CB
            .first()
            .and_then(|n| decode_account::<T>(&n.main_account))
            .map(|nrc| institution == nrc)
            .unwrap_or(false),
        ORG_PRC => CHINA_CB
            .iter()
            .skip(1)
            .filter_map(|n| decode_account::<T>(&n.main_account))
            .any(|pid| pid == institution),
        ORG_PRB => CHINA_CH
            .iter()
            .filter_map(|n| decode_account::<T>(&n.main_account))
            .any(|pid| pid == institution),
        org if is_registered_multisig_org(org) => {
            <T as votingengine::Config>::InternalAdminProvider::get_admin_list(org, institution)
                .is_some()
        }
        _ => false,
    }
}

fn is_internal_admin<T: Config>(org: u8, institution: T::AccountId, who: &T::AccountId) -> bool {
    <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(org, institution, who)
}

fn active_internal_threshold<T: Config>(org: u8, institution: T::AccountId) -> Option<u32> {
    match org {
        ORG_NRC | ORG_PRC | ORG_PRB => fixed_governance_pass_threshold(org),
        org if is_registered_multisig_org(org) => {
            ActiveDynamicThresholds::<T>::get(org, institution)
        }
        _ => None,
    }
}

// ──────────────────────────────────────────────────────────────────
// 业务方法
// ──────────────────────────────────────────────────────────────────

impl<T: Config> Pallet<T> {
    fn internal_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    fn ensure_threshold_within_snapshot(admins_len: u32, threshold: u32) -> DispatchResult {
        // 中文注释：普通内部提案仍按账户当前阈值投票，但阈值必须能被本次管理员快照实际达成。
        ensure!(
            threshold > 0 && threshold <= admins_len,
            Error::<T>::InvalidThresholdSnapshot
        );
        Ok(())
    }

    fn ensure_all_admin_threshold(admins_len: u32, threshold: u32) -> DispatchResult {
        // 中文注释：账户链上注册与注销会改变账户生命周期，必须由该账户快照内全体管理员通过。
        ensure!(
            admins_len > 0 && threshold == admins_len,
            Error::<T>::InvalidThresholdSnapshot
        );
        Ok(())
    }

    fn ensure_dynamic_threshold(admins_len: u32, threshold: u32) -> DispatchResult {
        // 中文注释：动态阈值只允许严格过半，且不得超过管理员总数；统一用 u64 避免乘法溢出。
        ensure!(admins_len >= 2, Error::<T>::InvalidDynamicThreshold);
        ensure!(
            threshold > 0
                && threshold <= admins_len
                && u64::from(threshold).saturating_mul(2) > u64::from(admins_len),
            Error::<T>::InvalidDynamicThreshold
        );
        Ok(())
    }

    fn snapshot_admins_len_or_missing(
        proposal_id: u64,
        institution: T::AccountId,
    ) -> Result<u32, DispatchError> {
        <votingengine::Pallet<T>>::snapshot_admins_len(proposal_id, institution)
            .ok_or(votingengine::Error::<T>::MissingAdminSnapshot.into())
    }

    pub fn do_create_registered_account_create_proposal(
        who: T::AccountId,
        org: u8,
        institution: T::AccountId,
        admins: sp_std::vec::Vec<T::AccountId>,
        dynamic_threshold: u32,
    ) -> Result<u64, DispatchError> {
        ensure!(
            is_registered_multisig_org(org),
            Error::<T>::InvalidInternalOrg
        );
        ensure!(
            !admins.is_empty(),
            votingengine::Error::<T>::MissingAdminSnapshot
        );
        ensure!(
            admins.iter().any(|admin| admin == &who),
            votingengine::Error::<T>::NoPermission
        );
        for i in 0..admins.len() {
            for j in i.saturating_add(1)..admins.len() {
                ensure!(
                    admins[i] != admins[j],
                    votingengine::Error::<T>::InvalidInstitution
                );
            }
        }
        let admins_len = admins.len() as u32;
        Self::ensure_dynamic_threshold(admins_len, dynamic_threshold)?;
        let lifecycle_threshold = admins_len;
        let bounded_admins: BoundedVec<
            T::AccountId,
            <T as votingengine::Config>::MaxAdminsPerInstitution,
        > = admins
            .try_into()
            .map_err(|_| votingengine::Error::<T>::InvalidInstitution)?;

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());
        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_org: Some(org),
            internal_institution: Some(institution.clone()),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            if let Err(err) =
                votingengine::limit::try_add_active_proposal::<T>(institution.clone(), id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                id,
                org,
                institution.clone(),
                InternalProposalMutexKind::Regular,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }

            AdminSnapshot::<T>::insert(id, institution.clone(), bounded_admins);
            InternalThresholdSnapshot::<T>::insert(id, lifecycle_threshold);
            PendingDynamicThresholds::<T>::insert(org, institution, dynamic_threshold);
            InternalProposalRoles::<T>::insert(id, InternalProposalRole::LifecycleCreate);
            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_INTERNAL,
                STAGE_INTERNAL,
                end,
            );
            TransactionOutcome::Commit(Ok(id))
        })
    }

    pub fn do_create_general_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: T::AccountId,
    ) -> Result<u64, DispatchError> {
        Self::do_create_active_account_internal_proposal(
            who,
            org,
            institution.clone(),
            InternalProposalMutexKind::Regular,
            InternalProposalRole::General,
            None,
        )
    }

    pub fn do_create_lifecycle_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: T::AccountId,
    ) -> Result<u64, DispatchError> {
        ensure!(
            is_registered_multisig_org(org),
            Error::<T>::InvalidInternalOrg
        );
        Self::do_create_active_account_internal_proposal(
            who,
            org,
            institution,
            InternalProposalMutexKind::Regular,
            InternalProposalRole::LifecycleClose,
            Some(true),
        )
    }

    pub fn do_create_admin_change_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: T::AccountId,
        new_admins_len: u32,
        new_threshold: u32,
    ) -> Result<u64, DispatchError> {
        if is_registered_multisig_org(org) {
            Self::ensure_dynamic_threshold(new_admins_len, new_threshold)?;
        } else {
            ensure!(
                fixed_governance_pass_threshold(org) == Some(new_threshold),
                Error::<T>::InvalidDynamicThreshold
            );
        }
        let proposal_id = Self::do_create_active_account_internal_proposal(
            who,
            org,
            institution.clone(),
            InternalProposalMutexKind::AdminSetMutationExclusive,
            InternalProposalRole::AdminChange,
            Some(false),
        )?;
        if is_registered_multisig_org(org) {
            PendingAdminChangeThresholds::<T>::insert(
                proposal_id,
                PendingAdminChangeThreshold {
                    org,
                    account: institution,
                    new_admins_len,
                    new_threshold,
                },
            );
        }
        Ok(proposal_id)
    }

    fn do_create_active_account_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: T::AccountId,
        mutex_kind: InternalProposalMutexKind,
        role: InternalProposalRole,
        force_all_admin_threshold: Option<bool>,
    ) -> Result<u64, DispatchError> {
        ensure!(is_valid_org(org), Error::<T>::InvalidInternalOrg);
        ensure!(
            is_valid_internal_institution::<T>(org, institution.clone()),
            votingengine::Error::<T>::InvalidInstitution
        );
        ensure!(
            is_internal_admin::<T>(org, institution.clone(), &who),
            votingengine::Error::<T>::NoPermission
        );
        let active_threshold = active_internal_threshold::<T>(org, institution.clone())
            .ok_or(Error::<T>::InvalidInternalOrg)?;

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: votingengine::STATUS_VOTING,
            internal_org: Some(org),
            internal_institution: Some(institution.clone()),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            if let Err(err) =
                votingengine::limit::try_add_active_proposal::<T>(institution.clone(), id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                id,
                org,
                institution.clone(),
                mutex_kind,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }

            if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                id,
                org,
                institution.clone(),
                false,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            if !<votingengine::Pallet<T>>::is_admin_in_snapshot(id, institution.clone(), &who) {
                frame_support::defensive!(
                    "do_create_internal_proposal: proposer is missing from admin snapshot"
                );
                return TransactionOutcome::Rollback(Err(
                    votingengine::Error::<T>::NoPermission.into()
                ));
            }
            let snapshot_size = match Self::snapshot_admins_len_or_missing(id, institution.clone())
            {
                Ok(size) => size,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let threshold = if force_all_admin_threshold.unwrap_or(false) {
                snapshot_size
            } else {
                active_threshold
            };
            let threshold_check = if force_all_admin_threshold.unwrap_or(false) {
                Self::ensure_all_admin_threshold(snapshot_size, threshold)
            } else if is_registered_multisig_org(org) {
                Self::ensure_dynamic_threshold(snapshot_size, threshold)
            } else {
                Self::ensure_threshold_within_snapshot(snapshot_size, threshold)
            };
            if let Err(err) = threshold_check {
                return TransactionOutcome::Rollback(Err(err));
            }
            InternalThresholdSnapshot::<T>::insert(id, threshold);
            InternalProposalRoles::<T>::insert(id, role);

            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_INTERNAL,
                STAGE_INTERNAL,
                end,
            );
            TransactionOutcome::Commit(Ok(id))
        })
    }

    fn register_data_and_auto_approve(
        who: T::AccountId,
        proposal_id: u64,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        let now = <frame_system::Pallet<T>>::block_number();
        <votingengine::Pallet<T>>::register_proposal_data(proposal_id, module_tag, data, now)?;
        // 中文注释：发起人签名发起提案后，投票引擎在同一事务自动记一票赞成，
        // 用户不需要再发第二笔“同意”交易。
        Self::do_internal_vote(who, proposal_id, true)?;
        Ok(proposal_id)
    }

    fn proposal_org_account(proposal_id: u64) -> Result<(u8, T::AccountId), DispatchError> {
        let proposal =
            Proposals::<T>::get(proposal_id).ok_or(votingengine::Error::<T>::ProposalNotFound)?;
        let org = proposal
            .internal_org
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        let account = proposal
            .internal_institution
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        Ok((org, account))
    }

    fn apply_executed_threshold_side_effect(proposal_id: u64) -> DispatchResult {
        match InternalProposalRoles::<T>::get(proposal_id) {
            Some(InternalProposalRole::LifecycleCreate) => {
                let (org, account) = Self::proposal_org_account(proposal_id)?;
                let threshold = PendingDynamicThresholds::<T>::take(org, account.clone())
                    .ok_or(Error::<T>::MissingDynamicThreshold)?;
                ActiveDynamicThresholds::<T>::insert(org, account, threshold);
            }
            Some(InternalProposalRole::LifecycleClose) => {
                let (org, account) = Self::proposal_org_account(proposal_id)?;
                ActiveDynamicThresholds::<T>::remove(org, account);
            }
            Some(InternalProposalRole::AdminChange) => {
                if let Some(pending) = PendingAdminChangeThresholds::<T>::take(proposal_id) {
                    Self::ensure_dynamic_threshold(pending.new_admins_len, pending.new_threshold)?;
                    ActiveDynamicThresholds::<T>::insert(
                        pending.org,
                        pending.account,
                        pending.new_threshold,
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_terminal_threshold_cleanup(proposal_id: u64, status: u8) -> DispatchResult {
        match (InternalProposalRoles::<T>::get(proposal_id), status) {
            (
                Some(InternalProposalRole::LifecycleCreate),
                STATUS_REJECTED | STATUS_EXECUTION_FAILED,
            ) => {
                let (org, account) = Self::proposal_org_account(proposal_id)?;
                PendingDynamicThresholds::<T>::remove(org, account);
            }
            (
                Some(InternalProposalRole::AdminChange),
                STATUS_REJECTED | STATUS_EXECUTION_FAILED,
            ) => {
                PendingAdminChangeThresholds::<T>::remove(proposal_id);
            }
            (Some(_), STATUS_EXECUTED) | (None, _) => {}
            _ => {}
        }
        Ok(())
    }

    pub fn do_internal_vote(who: T::AccountId, proposal_id: u64, approve: bool) -> DispatchResult {
        let proposal = <votingengine::Pallet<T>>::ensure_open_proposal(proposal_id)?;

        ensure!(
            proposal.kind == PROPOSAL_KIND_INTERNAL,
            votingengine::Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == STAGE_INTERNAL,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            !InternalVotesByAccount::<T>::contains_key(proposal_id, &who),
            votingengine::Error::<T>::AlreadyVoted
        );
        let institution = proposal
            .internal_institution
            .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
        ensure!(
            <votingengine::Pallet<T>>::is_admin_in_snapshot(proposal_id, institution.clone(), &who),
            votingengine::Error::<T>::NoPermission
        );

        InternalVotesByAccount::<T>::insert(proposal_id, &who, approve);
        let tally = InternalTallies::<T>::mutate(proposal_id, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
            *tally
        });

        Self::deposit_event(Event::<T>::InternalVoteCast {
            proposal_id,
            who,
            approve,
        });

        let threshold = InternalThresholdSnapshot::<T>::get(proposal_id)
            .ok_or(Error::<T>::MissingThresholdSnapshot)?;
        if tally.yes >= threshold {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)?;
        } else {
            let admins_len =
                <votingengine::Pallet<T>>::snapshot_admins_len(proposal_id, institution)
                    .ok_or(votingengine::Error::<T>::MissingAdminSnapshot)?;
            let casted = tally.yes.saturating_add(tally.no);
            let remaining = admins_len.saturating_sub(casted);
            if tally.yes.saturating_add(remaining) < threshold {
                <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)?;
            }
        }

        Ok(())
    }

    pub fn do_finalize_internal_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_INTERNAL,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.status == votingengine::STATUS_VOTING,
            votingengine::Error::<T>::ProposalAlreadyFinalized
        );
        ensure!(
            <frame_system::Pallet<T>>::block_number() > proposal.end,
            votingengine::Error::<T>::VoteNotExpired
        );
        <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, votingengine::STATUS_REJECTED)
    }
}

// ──────────────────────────────────────────────────────────────────
// trait 实现
// ──────────────────────────────────────────────────────────────────

impl<T: Config> votingengine::InternalVoteEngine<T::AccountId> for Pallet<T> {
    fn create_general_internal_proposal_with_data(
        who: T::AccountId,
        org: u8,
        institution: T::AccountId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id =
                match Self::do_create_general_internal_proposal(who.clone(), org, institution) {
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
        org: u8,
        institution: T::AccountId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id =
                match Self::do_create_lifecycle_internal_proposal(who.clone(), org, institution) {
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
        org: u8,
        institution: T::AccountId,
        admins: sp_std::vec::Vec<T::AccountId>,
        dynamic_threshold: u32,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id = match Self::do_create_registered_account_create_proposal(
                who.clone(),
                org,
                institution,
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
        org: u8,
        institution: T::AccountId,
        new_admins_len: u32,
        new_threshold: u32,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        with_transaction(|| {
            let proposal_id = match Self::do_create_admin_change_internal_proposal(
                who.clone(),
                org,
                institution,
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

    fn active_dynamic_threshold(org: u8, institution: T::AccountId) -> Option<u32> {
        ActiveDynamicThresholds::<T>::get(org, institution)
    }

    fn configured_dynamic_threshold(org: u8, institution: T::AccountId) -> Option<u32> {
        ActiveDynamicThresholds::<T>::get(org, institution.clone())
            .or_else(|| PendingDynamicThresholds::<T>::get(org, institution))
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

impl<T: Config> votingengine::traits::InternalCleanupHandler for Pallet<T> {
    fn on_internal_proposal_executed(proposal_id: u64) -> DispatchResult {
        Self::apply_executed_threshold_side_effect(proposal_id)
    }

    fn on_internal_proposal_terminal(proposal_id: u64, status: u8) -> DispatchResult {
        Self::apply_terminal_threshold_cleanup(proposal_id, status)
    }

    fn cleanup_internal_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::traits::CleanupChunkResult {
        let result = InternalVotesByAccount::<T>::clear_prefix(proposal_id, limit, None);
        let has_remaining = result.maybe_cursor.is_some();
        (result.unique, has_remaining)
    }

    fn cleanup_internal_terminal(proposal_id: u64) {
        InternalTallies::<T>::remove(proposal_id);
        InternalThresholdSnapshot::<T>::remove(proposal_id);
        PendingAdminChangeThresholds::<T>::remove(proposal_id);
        InternalProposalRoles::<T>::remove(proposal_id);
    }
}
