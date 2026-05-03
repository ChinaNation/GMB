#![cfg_attr(not(feature = "std"), no_std)]
//! 管理员权限治理模块（admins-change）
//! - 本模块只负责“更换管理员”这一类业务事项
//! - 投票流程本身由 voting-engine 提供（内部投票）
//! - 约束：仅替换，不增删；且仅能在本机构范围内更换

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{
    ensure,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::StorageVersion,
    Blake2_128Concat,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, RuntimeDebug};
use sp_std::collections::btree_set::BTreeSet;

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use primitives::count_const::{
    NRC_ADMIN_COUNT, NRC_INTERNAL_THRESHOLD, PRB_ADMIN_COUNT, PRB_INTERNAL_THRESHOLD,
    PRC_ADMIN_COUNT, PRC_INTERNAL_THRESHOLD,
};
use voting_engine::{
    internal_vote::{ORG_DUOQIAN, ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId, InternalVoteResultCallback, ProposalExecutionOutcome,
    PROPOSAL_KIND_INTERNAL, STAGE_INTERNAL, STATUS_EXECUTION_FAILED, STATUS_PASSED,
    STATUS_REJECTED, STATUS_VOTING,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
/// 中文注释：tag 带 schema 版本号；开发期不兼容旧 `adm-rep` 提案数据。
pub const MODULE_TAG: &[u8] = b"adm-rep-v1";

#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
pub struct AdminReplacementAction<AccountId> {
    /// 目标机构（机构标识 pallet_id）
    pub institution: InstitutionPalletId,
    /// 被替换的管理员
    pub old_admin: AccountId,
    /// 新管理员
    pub new_admin: AccountId,
}

/// 管理员主体类型。所有需要内部投票的多签主体都在本模块统一登记。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum AdminSubjectKind {
    /// 国储会、省储会、省储行等创世内置机构。
    BuiltinInstitution,
    /// SFID 系统登记后在链上注册的机构多签。
    SfidInstitution,
    /// 用户自建的个人多签。
    PersonalDuoqian,
}

/// 管理员主体生命周期。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum AdminSubjectStatus {
    /// 创建提案投票中；投票引擎可读取管理员快照。
    Pending,
    /// 已激活，可发起常规治理/转账/管理员更换。
    Active,
    /// 已关闭，管理员不再有效。
    Closed,
}

/// 统一管理员主体记录。
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
#[scale_info(skip_type_params(AdminList))]
pub struct AdminInstitution<AdminList, AccountId, BlockNumber> {
    pub org: u8,
    pub kind: AdminSubjectKind,
    pub admins: AdminList,
    pub threshold: u32,
    pub creator: AccountId,
    pub created_at: BlockNumber,
    pub updated_at: BlockNumber,
    pub status: AdminSubjectStatus,
}

/// 管理员主体生命周期写入口。
///
/// 中文注释：这里是跨 pallet 唯一允许写 Pending/Active/Closed 生命周期的 API。
/// 裸存储 mutator 保持 crate 内私有；调用方必须提供 voting-engine 提案上下文，
/// 由 admins-change 再校验 owner、机构、状态和回调作用域。
pub trait SubjectLifecycle<AccountId> {
    fn create_pending_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: InstitutionPalletId,
        org: u8,
        kind: AdminSubjectKind,
        admins: Vec<AccountId>,
        threshold: u32,
        creator: AccountId,
    ) -> DispatchResult;

    fn activate_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: InstitutionPalletId,
    ) -> DispatchResult;

    fn remove_pending_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: InstitutionPalletId,
    ) -> DispatchResult;

    fn close_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: InstitutionPalletId,
    ) -> DispatchResult;
}

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

fn nrc_pallet_id_bytes() -> Option<InstitutionPalletId> {
    // 中文注释：国储会ID统一从常量数组读取并转码。
    CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
}

fn expected_admin_count(org: u8) -> Option<u32> {
    match org {
        ORG_NRC => Some(NRC_ADMIN_COUNT),
        ORG_PRC => Some(PRC_ADMIN_COUNT),
        ORG_PRB => Some(PRB_ADMIN_COUNT),
        _ => None,
    }
}

fn default_threshold(org: u8) -> Option<u32> {
    match org {
        ORG_NRC => Some(NRC_INTERNAL_THRESHOLD),
        ORG_PRC => Some(PRC_INTERNAL_THRESHOLD),
        ORG_PRB => Some(PRB_INTERNAL_THRESHOLD),
        _ => None,
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use voting_engine::InternalVoteEngine;

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        /// 单个机构管理员最大数量上限（用于 BoundedVec）
        type MaxAdminsPerInstitution: Get<u32>;

        /// 中文注释：内部投票引擎（返回真实 proposal_id，避免外部猜测 next_proposal_id）。
        type InternalVoteEngine: voting_engine::InternalVoteEngine<Self::AccountId>;

        /// 该 pallet 的可配置权重实现。
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    pub type AdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAdminsPerInstitution>;

    pub type AdminInstitutionOf<T> =
        AdminInstitution<AdminsOf<T>, <T as frame_system::Config>::AccountId, BlockNumberFor<T>>;

    /// 统一管理员主体表：subject_id → 管理员、阈值和生命周期。
    ///
    /// 创世时写入国储会、省储会、省储行；SFID 机构多签和个人多签由
    /// `duoqian-manage` 在创建提案阶段写入 Pending，投票通过后激活。
    #[pallet::storage]
    #[pallet::getter(fn institution_of)]
    pub type Institutions<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, AdminInstitutionOf<T>, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub _phantom: core::marker::PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _phantom: Default::default(),
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn integrity_test() {
            let required = NRC_ADMIN_COUNT.max(PRC_ADMIN_COUNT).max(PRB_ADMIN_COUNT);
            assert!(
                <T as Config>::MaxAdminsPerInstitution::get() >= required,
                "MaxAdminsPerInstitution must be >= largest expected admin count"
            );
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for node in CHINA_CB.iter() {
                let Some(institution) = reserve_pallet_id_to_bytes(node.shenfen_id) else {
                    continue;
                };
                let org = if Some(institution) == nrc_pallet_id_bytes() {
                    ORG_NRC
                } else {
                    ORG_PRC
                };
                let admins: Vec<T::AccountId> = node
                    .duoqian_admins
                    .iter()
                    .map(|raw| {
                        T::AccountId::decode(&mut &raw[..])
                            .expect("reserve admin account must decode")
                    })
                    .collect();
                let bounded: BoundedVec<T::AccountId, <T as Config>::MaxAdminsPerInstitution> =
                    admins
                        .try_into()
                        .expect("reserve admins must fit MaxAdminsPerInstitution");
                let creator = bounded
                    .get(0)
                    .cloned()
                    .expect("builtin institution must have admins");
                Institutions::<T>::insert(
                    institution,
                    AdminInstitution {
                        org,
                        kind: AdminSubjectKind::BuiltinInstitution,
                        admins: bounded,
                        threshold: default_threshold(org).expect("builtin org has threshold"),
                        creator,
                        created_at: Zero::zero(),
                        updated_at: Zero::zero(),
                        status: AdminSubjectStatus::Active,
                    },
                );
            }

            for node in CHINA_CH.iter() {
                let Some(institution) = shengbank_pallet_id_to_bytes(node.shenfen_id) else {
                    continue;
                };
                let admins: Vec<T::AccountId> = node
                    .duoqian_admins
                    .iter()
                    .map(|raw| {
                        T::AccountId::decode(&mut &raw[..])
                            .expect("shengbank admin account must decode")
                    })
                    .collect();
                let bounded: BoundedVec<T::AccountId, <T as Config>::MaxAdminsPerInstitution> =
                    admins
                        .try_into()
                        .expect("shengbank admins must fit MaxAdminsPerInstitution");
                let creator = bounded
                    .get(0)
                    .cloned()
                    .expect("builtin institution must have admins");
                Institutions::<T>::insert(
                    institution,
                    AdminInstitution {
                        org: ORG_PRB,
                        kind: AdminSubjectKind::BuiltinInstitution,
                        admins: bounded,
                        threshold: default_threshold(ORG_PRB).expect("PRB has threshold"),
                        creator,
                        created_at: Zero::zero(),
                        updated_at: Zero::zero(),
                        status: AdminSubjectStatus::Active,
                    },
                );
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 已发起管理员更换提案（并已在投票引擎创建内部提案）
        AdminReplacementProposed {
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        },
        /// 提案达到通过状态但自动执行失败（投票不回滚）
        AdminReplacementExecutionFailed { proposal_id: u64 },
        /// 管理员列表已完成替换执行
        AdminReplaced {
            proposal_id: u64,
            institution: InstitutionPalletId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        },
        /// 多签主体管理员配置已写入 Pending。
        AdminSubjectPendingCreated {
            institution: InstitutionPalletId,
            org: u8,
            kind: AdminSubjectKind,
            creator: T::AccountId,
            admin_count: u32,
            threshold: u32,
        },
        /// 多签主体管理员配置已激活。
        AdminSubjectActivated { institution: InstitutionPalletId },
        /// Pending 多签主体管理员配置已清理。
        AdminSubjectPendingRemoved { institution: InstitutionPalletId },
        /// 多签主体管理员配置已关闭。
        AdminSubjectClosed { institution: InstitutionPalletId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 无效机构
        InvalidInstitution,
        /// 机构类型与 org 参数不匹配
        InstitutionOrgMismatch,
        /// 管理员数量不符合固定人数约束
        InvalidAdminCount,
        /// 非该机构管理员，无权限
        UnauthorizedAdmin,
        /// 旧管理员不在当前名单中
        OldAdminNotFound,
        /// 新管理员已经在当前名单中
        NewAdminAlreadyExists,
        /// 找不到与投票提案绑定的管理员更换动作
        ProposalActionNotFound,
        /// 投票尚未通过，不能执行替换
        ProposalNotPassed,
        /// 提案类型不是内部投票
        InvalidProposalKind,
        /// 提案阶段不是内部投票阶段
        InvalidProposalStage,
        /// 提案绑定机构与管理员更换动作不一致
        ProposalInstitutionMismatch,
        /// 提案绑定组织与管理员主体不一致
        ProposalOrgMismatch,
        /// 管理员主体已存在
        InstitutionAlreadyExists,
        /// 管理员主体状态不是 Pending
        SubjectNotPending,
        /// 管理员主体状态不是 Active
        SubjectNotActive,
        /// 内置治理机构永远不可关闭
        BuiltinSubjectCannotClose,
        /// 管理员主体类型与 org 不匹配
        InvalidSubjectKind,
        /// 阈值不合法
        InvalidThreshold,
        /// 管理员重复
        DuplicateAdmin,
        /// 管理员主体生命周期写入缺少有效 voting-engine 提案作用域
        InvalidSubjectLifecycleScope,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_admin_replacement())]
        pub fn propose_admin_replacement(
            origin: OriginFor<T>,
            org: u8,
            institution: InstitutionPalletId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 中文注释：本入口只治理制度内置主体(NRC/PRC/PRB)的管理员替换。
            // ORG_DUOQIAN 的个人/机构多签主体由 duoqian-manage 维护生命周期,
            // 不能从通用管理员替换入口绕出第二条治理路径。
            ensure!(
                matches!(org, ORG_NRC | ORG_PRC | ORG_PRB),
                Error::<T>::InvalidSubjectKind
            );

            // 1) 校验管理员主体已激活且 org 匹配。
            let subject =
                Institutions::<T>::get(institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                subject.status == AdminSubjectStatus::Active,
                Error::<T>::SubjectNotActive
            );
            ensure!(subject.org == org, Error::<T>::InstitutionOrgMismatch);

            // 2) 校验发起人与替换参数合法性
            let admins = Self::admins_for_institution(institution)?;
            ensure!(admins.contains(&who), Error::<T>::UnauthorizedAdmin);
            ensure!(admins.contains(&old_admin), Error::<T>::OldAdminNotFound);
            ensure!(
                !admins.contains(&new_admin),
                Error::<T>::NewAdminAlreadyExists
            );

            // 3) 在同一个链上事务中创建投票提案、互斥锁和业务数据。
            with_transaction(|| {
                let action = AdminReplacementAction {
                    institution,
                    old_admin: old_admin.clone(),
                    new_admin: new_admin.clone(),
                };
                let encoded = action.encode();
                let proposal_id = match T::InternalVoteEngine::create_admin_set_mutation_internal_proposal_with_data(
                    who.clone(),
                    org,
                    institution,
                    crate::MODULE_TAG,
                    encoded,
                ) {
                    Ok(proposal_id) => proposal_id,
                    Err(err) => return TransactionOutcome::Rollback(Err(err)),
                };

                Self::deposit_event(Event::<T>::AdminReplacementProposed {
                    proposal_id,
                    org,
                    institution,
                    proposer: who,
                    old_admin,
                    new_admin,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }

        /// 任意人触发"已通过提案"的执行。
        ///
        /// Phase 2 整改后投票一律走 `VotingEngine::internal_vote` 公开 call;
        /// 通过后由本模块的 `InternalVoteExecutor` 自动触发 `try_execute_replacement`。
        /// 自动执行失败会进入 `STATUS_EXECUTION_FAILED` 终态,本 call 不再允许跨时期重试。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::execute_admin_replacement())]
        pub fn execute_admin_replacement(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            voting_engine::Pallet::<T>::retry_passed_proposal_for(&who, proposal_id)
        }
    }

    impl<T: Config> Pallet<T> {
        fn admins_for_institution(
            institution: InstitutionPalletId,
        ) -> Result<Vec<T::AccountId>, DispatchError> {
            // 中文注释：创世后只信任链上管理员状态，不再回退常量管理员。
            let stored =
                Institutions::<T>::get(institution).ok_or(Error::<T>::InvalidInstitution)?;
            Ok(stored.admins.into_inner())
        }

        fn validate_admin_count_for_subject(
            kind: AdminSubjectKind,
            org: u8,
            admins_len: usize,
        ) -> DispatchResult {
            if matches!(kind, AdminSubjectKind::BuiltinInstitution) {
                // 固定人数约束：国储会19，省储会9，省储行9
                let expected = expected_admin_count(org).ok_or(Error::<T>::InvalidInstitution)?;
                ensure!(
                    admins_len == expected as usize,
                    Error::<T>::InvalidAdminCount
                );
            } else {
                ensure!(admins_len >= 2, Error::<T>::InvalidAdminCount);
                ensure!(
                    admins_len <= <T as Config>::MaxAdminsPerInstitution::get() as usize,
                    Error::<T>::InvalidAdminCount
                );
            }
            Ok(())
        }

        fn ensure_unique_admins(admins: &[T::AccountId]) -> DispatchResult {
            let mut seen = BTreeSet::new();
            for admin in admins {
                ensure!(seen.insert(admin.clone()), Error::<T>::DuplicateAdmin);
            }
            Ok(())
        }

        fn ensure_subject_kind_matches_org(kind: AdminSubjectKind, org: u8) -> DispatchResult {
            match kind {
                AdminSubjectKind::BuiltinInstitution => {
                    ensure!(
                        matches!(org, ORG_NRC | ORG_PRC | ORG_PRB),
                        Error::<T>::InvalidSubjectKind
                    );
                }
                AdminSubjectKind::SfidInstitution | AdminSubjectKind::PersonalDuoqian => {
                    ensure!(org == ORG_DUOQIAN, Error::<T>::InvalidSubjectKind);
                }
            }
            Ok(())
        }

        fn validate_threshold(admin_count: u32, threshold: u32) -> DispatchResult {
            let min_threshold = core::cmp::max(2, admin_count.saturating_add(1) / 2);
            ensure!(
                threshold >= min_threshold && threshold <= admin_count,
                Error::<T>::InvalidThreshold
            );
            Ok(())
        }

        pub(crate) fn ensure_lifecycle_proposal(
            proposal_id: u64,
            module_tag: &[u8],
            institution: InstitutionPalletId,
            org: u8,
            expected_status: u8,
            require_callback_scope: bool,
        ) -> DispatchResult {
            ensure!(
                voting_engine::Pallet::<T>::is_proposal_owner(proposal_id, module_tag),
                Error::<T>::InvalidSubjectLifecycleScope
            );
            let proposal = voting_engine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::InvalidSubjectLifecycleScope)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL,
                Error::<T>::InvalidSubjectLifecycleScope
            );
            ensure!(
                proposal.stage == STAGE_INTERNAL,
                Error::<T>::InvalidSubjectLifecycleScope
            );
            ensure!(
                proposal.internal_institution == Some(institution),
                Error::<T>::ProposalInstitutionMismatch
            );
            ensure!(
                proposal.internal_org == Some(org),
                Error::<T>::ProposalOrgMismatch
            );
            ensure!(
                proposal.status == expected_status,
                Error::<T>::InvalidSubjectLifecycleScope
            );
            if require_callback_scope {
                ensure!(
                    voting_engine::Pallet::<T>::is_callback_execution_scope(proposal_id),
                    Error::<T>::InvalidSubjectLifecycleScope
                );
            }
            Ok(())
        }

        /// 写入 Pending 管理员主体。
        ///
        /// 中文注释：生命周期写入只能经 `SubjectLifecycle` trait 做提案上下文校验后进入。
        pub(crate) fn do_create_pending_subject(
            institution: InstitutionPalletId,
            org: u8,
            kind: AdminSubjectKind,
            admins: Vec<T::AccountId>,
            threshold: u32,
            creator: T::AccountId,
        ) -> DispatchResult {
            ensure!(
                !Institutions::<T>::contains_key(institution),
                Error::<T>::InstitutionAlreadyExists
            );
            Self::ensure_subject_kind_matches_org(kind, org)?;
            Self::validate_admin_count_for_subject(kind, org, admins.len())?;
            Self::ensure_unique_admins(&admins)?;
            Self::validate_threshold(admins.len() as u32, threshold)?;

            let bounded: AdminsOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminCount)?;
            let now = frame_system::Pallet::<T>::block_number();
            let admin_count = bounded.len() as u32;
            Institutions::<T>::insert(
                institution,
                AdminInstitution {
                    org,
                    kind,
                    admins: bounded,
                    threshold,
                    creator: creator.clone(),
                    created_at: now,
                    updated_at: now,
                    status: AdminSubjectStatus::Pending,
                },
            );
            Self::deposit_event(Event::<T>::AdminSubjectPendingCreated {
                institution,
                org,
                kind,
                creator,
                admin_count,
                threshold,
            });
            Ok(())
        }

        /// 将 Pending 管理员主体激活。
        pub(crate) fn do_activate_subject(institution: InstitutionPalletId) -> DispatchResult {
            Institutions::<T>::try_mutate(institution, |maybe| -> DispatchResult {
                let subject = maybe.as_mut().ok_or(Error::<T>::InvalidInstitution)?;
                ensure!(
                    subject.status == AdminSubjectStatus::Pending,
                    Error::<T>::SubjectNotPending
                );
                subject.status = AdminSubjectStatus::Active;
                subject.updated_at = frame_system::Pallet::<T>::block_number();
                Ok(())
            })?;
            Self::deposit_event(Event::<T>::AdminSubjectActivated { institution });
            Ok(())
        }

        /// 清理尚未激活的 Pending 管理员主体。
        pub(crate) fn do_remove_pending_subject(
            institution: InstitutionPalletId,
        ) -> DispatchResult {
            if let Some(subject) = Institutions::<T>::get(institution) {
                ensure!(
                    subject.status == AdminSubjectStatus::Pending,
                    Error::<T>::SubjectNotPending
                );
                Institutions::<T>::remove(institution);
                Self::deposit_event(Event::<T>::AdminSubjectPendingRemoved { institution });
            }
            Ok(())
        }

        /// 关闭已激活管理员主体。
        pub(crate) fn do_close_subject(institution: InstitutionPalletId) -> DispatchResult {
            Institutions::<T>::try_mutate(institution, |maybe| -> DispatchResult {
                let subject = maybe.as_mut().ok_or(Error::<T>::InvalidInstitution)?;
                ensure!(
                    subject.status == AdminSubjectStatus::Active,
                    Error::<T>::SubjectNotActive
                );
                // 中文注释：NRC/PRC/PRB 是制度内置治理主体，生命周期不能进入 Closed。
                ensure!(
                    !matches!(subject.kind, AdminSubjectKind::BuiltinInstitution),
                    Error::<T>::BuiltinSubjectCannotClose
                );
                subject.status = AdminSubjectStatus::Closed;
                subject.updated_at = frame_system::Pallet::<T>::block_number();
                Ok(())
            })?;
            Self::deposit_event(Event::<T>::AdminSubjectClosed { institution });
            Ok(())
        }

        fn subject_with_status(
            org: u8,
            institution: InstitutionPalletId,
            status: AdminSubjectStatus,
        ) -> Option<AdminInstitutionOf<T>> {
            let subject = Institutions::<T>::get(institution)?;
            if subject.org != org || subject.status != status {
                return None;
            }
            Some(subject)
        }

        /// 查询 Active 主体是否存在。普通业务主体合法性判断只使用 Active 主体。
        pub fn active_subject_exists(org: u8, institution: InstitutionPalletId) -> bool {
            Self::subject_with_status(org, institution, AdminSubjectStatus::Active).is_some()
        }

        /// 查询 Active 主体管理员权限。普通业务授权只能使用 Active 主体。
        pub fn is_active_subject_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            let Some(subject) =
                Self::subject_with_status(org, institution, AdminSubjectStatus::Active)
            else {
                return false;
            };
            subject.admins.iter().any(|admin| admin == who)
        }

        /// 读取 Active 主体管理员列表。普通业务提案创建和投票快照默认使用此 API。
        pub fn active_subject_admins(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<Vec<T::AccountId>> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Active)?;
            Some(subject.admins.into_inner())
        }

        /// 读取 Active 主体阈值。普通业务投票只能使用 Active 阈值。
        pub fn active_subject_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Active)?;
            Some(subject.threshold)
        }

        /// 读取 Active 主体管理员数量。普通业务阈值兜底判断只能使用 Active 主体。
        pub fn active_subject_admin_count(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<u32> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Active)?;
            Some(subject.admins.len() as u32)
        }

        /// 查询 Pending 主体是否存在。仅用于创建/激活该主体时判断主体合法性。
        pub fn pending_subject_exists_for_snapshot(
            org: u8,
            institution: InstitutionPalletId,
        ) -> bool {
            Self::subject_with_status(org, institution, AdminSubjectStatus::Pending).is_some()
        }

        /// 查询 Pending 主体管理员权限。仅用于创建/激活该主体时锁定投票快照。
        pub fn is_pending_subject_admin_for_snapshot(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            let Some(subject) =
                Self::subject_with_status(org, institution, AdminSubjectStatus::Pending)
            else {
                return false;
            };
            subject.admins.iter().any(|admin| admin == who)
        }

        /// 读取 Pending 主体管理员列表。仅供投票引擎 Pending 创建入口写快照。
        pub fn pending_subject_admins_for_snapshot(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<Vec<T::AccountId>> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Pending)?;
            Some(subject.admins.into_inner())
        }

        /// 读取 Pending 主体阈值。仅供投票引擎 Pending 创建入口写阈值快照。
        pub fn pending_subject_threshold_for_snapshot(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<u32> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Pending)?;
            Some(subject.threshold)
        }

        /// 读取 Pending 主体管理员数量。仅用于创建/激活该主体的快照语义。
        pub fn pending_subject_admin_count_for_snapshot(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<u32> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Pending)?;
            Some(subject.admins.len() as u32)
        }

        pub(crate) fn try_execute_replacement_from_action(
            proposal_id: u64,
            action: AdminReplacementAction<T::AccountId>,
        ) -> DispatchResult {
            // 中文注释：执行前同时校验投票引擎元数据与业务 action，避免跨模块误消费。
            let proposal = voting_engine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL,
                Error::<T>::InvalidProposalKind
            );
            ensure!(
                proposal.stage == STAGE_INTERNAL,
                Error::<T>::InvalidProposalStage
            );
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let subject =
                Institutions::<T>::get(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                subject.status == AdminSubjectStatus::Active,
                Error::<T>::SubjectNotActive
            );
            ensure!(
                proposal.internal_institution == Some(action.institution),
                Error::<T>::ProposalInstitutionMismatch
            );
            ensure!(
                proposal.internal_org == Some(subject.org),
                Error::<T>::ProposalOrgMismatch
            );
            voting_engine::Pallet::<T>::ensure_admin_set_mutation_lock_owner(
                subject.org,
                action.institution,
                proposal_id,
            )?;
            let mut admins = Self::admins_for_institution(action.institution)?;
            Self::validate_admin_count_for_subject(subject.kind, subject.org, admins.len())?;

            let old_pos = admins
                .iter()
                .position(|a| a == &action.old_admin)
                .ok_or(Error::<T>::OldAdminNotFound)?;
            ensure!(
                !admins.iter().any(|a| a == &action.new_admin),
                Error::<T>::NewAdminAlreadyExists
            );

            // 只替换，不增删：列表长度保持不变
            admins[old_pos] = action.new_admin.clone();

            let bounded: BoundedVec<T::AccountId, <T as Config>::MaxAdminsPerInstitution> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminCount)?;
            Institutions::<T>::mutate(action.institution, |maybe| {
                if let Some(subject) = maybe {
                    subject.admins = bounded;
                    subject.updated_at = frame_system::Pallet::<T>::block_number();
                }
            });

            Self::deposit_event(Event::<T>::AdminReplaced {
                proposal_id,
                institution: action.institution,
                old_admin: action.old_admin,
                new_admin: action.new_admin,
            });

            Ok(())
        }
    }
}

impl<T: pallet::Config> SubjectLifecycle<T::AccountId> for pallet::Pallet<T> {
    fn create_pending_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: InstitutionPalletId,
        org: u8,
        kind: AdminSubjectKind,
        admins: Vec<T::AccountId>,
        threshold: u32,
        creator: T::AccountId,
    ) -> DispatchResult {
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            institution,
            org,
            STATUS_VOTING,
            false,
        )?;
        Self::do_create_pending_subject(institution, org, kind, admins, threshold, creator)
    }

    fn activate_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: InstitutionPalletId,
    ) -> DispatchResult {
        let subject = pallet::Institutions::<T>::get(institution)
            .ok_or(pallet::Error::<T>::InvalidInstitution)?;
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            institution,
            subject.org,
            STATUS_PASSED,
            true,
        )?;
        Self::do_activate_subject(institution)
    }

    fn remove_pending_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: InstitutionPalletId,
    ) -> DispatchResult {
        let subject = pallet::Institutions::<T>::get(institution)
            .ok_or(pallet::Error::<T>::InvalidInstitution)?;
        let proposal = voting_engine::Pallet::<T>::proposals(proposal_id)
            .ok_or(pallet::Error::<T>::InvalidSubjectLifecycleScope)?;
        ensure!(
            matches!(proposal.status, STATUS_REJECTED | STATUS_EXECUTION_FAILED),
            pallet::Error::<T>::InvalidSubjectLifecycleScope
        );
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            institution,
            subject.org,
            proposal.status,
            false,
        )?;
        Self::do_remove_pending_subject(institution)
    }

    fn close_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: InstitutionPalletId,
    ) -> DispatchResult {
        let subject = pallet::Institutions::<T>::get(institution)
            .ok_or(pallet::Error::<T>::InvalidInstitution)?;
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            institution,
            subject.org,
            STATUS_PASSED,
            true,
        )?;
        Self::do_close_subject(institution)
    }
}

// ──── 投票终态回调:把已通过的管理员替换提案落地到链上 ────
//
// Phase 2 整改后业务模块不再自行处理投票,提案通过(或否决)由投票引擎
// 通过 [`voting_engine::InternalVoteResultCallback`] 广播回来。
// 本 Executor 按 `ProposalOwner` 认领本模块提案，`ProposalData` 只保存裸业务 action。
//
// 设计要点:
// - `approved = true` 时执行 `try_execute_replacement`,失败发 `AdminReplacementExecutionFailed`
//   事件但不返回 Err(否则投票引擎会回滚状态,票数白投);
// - `approved = false` 下本模块没有独立存储需要清理,直接 Ok(()) 返回;
// - 数据层异常(ProposalOwner 匹配但 data 缺失/解码失败)返回 Err,触发 set_status_and_emit 回滚,
//   避免错误状态被提交。
pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, sp_runtime::DispatchError> {
        // Step 1:认领 — 检查 ProposalOwner，避免再依赖 ProposalData 的 MODULE_TAG 前缀。
        if !voting_engine::Pallet::<T>::is_proposal_owner(proposal_id, crate::MODULE_TAG) {
            return Ok(ProposalExecutionOutcome::Ignored);
        }
        let raw = voting_engine::Pallet::<T>::get_proposal_data(proposal_id)
            .ok_or(pallet::Error::<T>::ProposalActionNotFound)?;

        if !approved {
            // 否决:无独立存储需清理(ProposalData 由投票引擎延迟清理)。
            return Ok(ProposalExecutionOutcome::Executed);
        }

        // Step 2:解码 action。异常视为数据层问题,回滚投票状态。
        let action = AdminReplacementAction::<T::AccountId>::decode(&mut &raw[..])
            .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;

        // Step 3:执行替换。管理员集合变更失败属于数据/状态已不匹配，直接交给投票引擎失败终态。
        match pallet::Pallet::<T>::try_execute_replacement_from_action(proposal_id, action) {
            Ok(()) => Ok(ProposalExecutionOutcome::Executed),
            Err(_) => {
                pallet::Pallet::<T>::deposit_event(
                    pallet::Event::<T>::AdminReplacementExecutionFailed { proposal_id },
                );
                Ok(ProposalExecutionOutcome::FatalFailed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32};
    use frame_system as system;
    use primitives::china::china_cb::{
        shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
    };
    use primitives::china::china_ch::{
        shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
    };
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
    use voting_engine::{
        internal_vote::{ORG_DUOQIAN, ORG_NRC, ORG_PRB, ORG_PRC},
        InternalVoteEngine, STATUS_EXECUTED, STATUS_EXECUTION_FAILED, STATUS_PASSED,
        STATUS_REJECTED,
    };

    type Block = frame_system::mocking::MockBlock<Test>;

    #[frame_support::runtime]
    mod runtime {
        #[runtime::runtime]
        #[runtime::derive(
            RuntimeCall,
            RuntimeEvent,
            RuntimeError,
            RuntimeOrigin,
            RuntimeFreezeReason,
            RuntimeHoldReason,
            RuntimeSlashReason,
            RuntimeLockId,
            RuntimeTask,
            RuntimeViewFunction
        )]
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;

        #[runtime::pallet_index(1)]
        pub type VotingEngine = voting_engine;

        #[runtime::pallet_index(2)]
        pub type AdminsChange = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    pub struct TestSfidEligibility;
    impl voting_engine::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
        for TestSfidEligibility
    {
        fn is_eligible(
            _binding_id: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
        ) -> bool {
            true
        }

        fn verify_and_consume_vote_credential(
            _binding_id: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
            _proposal_id: u64,
            _nonce: &[u8],
            _signature: &[u8],
            _province: &[u8],
            _signer_admin_pubkey: &[u8; 32],
        ) -> bool {
            true
        }
    }

    pub struct TestPopulationSnapshotVerifier;
    impl
        voting_engine::PopulationSnapshotVerifier<
            AccountId32,
            voting_engine::pallet::VoteNonceOf<Test>,
            voting_engine::pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            _eligible_total: u64,
            _nonce: &voting_engine::pallet::VoteNonceOf<Test>,
            _signature: &voting_engine::pallet::VoteSignatureOf<Test>,
            _province: &[u8],
            _signer_admin_pubkey: &[u8; 32],
        ) -> bool {
            true
        }
    }

    pub struct TestInternalAdminProvider;
    impl voting_engine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            if !matches!(org, ORG_NRC | ORG_PRC | ORG_PRB) {
                return false;
            }
            pallet::Pallet::<Test>::is_active_subject_admin(org, institution, who)
        }

        fn get_admin_list(org: u8, institution: InstitutionPalletId) -> Option<Vec<AccountId32>> {
            if !matches!(org, ORG_NRC | ORG_PRC | ORG_PRB) {
                return None;
            }
            pallet::Pallet::<Test>::active_subject_admins(org, institution)
        }
    }

    pub struct TestInternalThresholdProvider;
    impl voting_engine::InternalThresholdProvider for TestInternalThresholdProvider {
        fn pass_threshold(org: u8, _institution: InstitutionPalletId) -> Option<u32> {
            voting_engine::internal_vote::fixed_governance_pass_threshold(org)
        }
    }

    pub struct TestTimeProvider;
    impl frame_support::traits::UnixTime for TestTimeProvider {
        fn now() -> core::time::Duration {
            core::time::Duration::from_secs(1_782_864_000) // 2026-07-01
        }
    }

    impl voting_engine::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type MaxProposalsPerExpiry = ConstU32<128>;
        type MaxInternalProposalMutexBindings = ConstU32<256>;
        type MaxActiveProposals = ConstU32<10>;
        type MaxCleanupStepsPerBlock = ConstU32<8>;
        type MaxCleanupQueueBucketLimit = ConstU32<50>;
        type MaxCleanupScheduleOffset = ConstU32<100>;
        type CleanupKeysPerStep = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<256>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type MaxModuleTagLen = ConstU32<32>;
        type MaxManualExecutionAttempts = ConstU32<3>;
        type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
        type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
        type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        // Phase 2 整改:mock runtime 必须把本模块的 Executor 挂上,
        // 否则内部提案通过后业务执行回调不会触发,端到端测试失败。
        type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalThresholdProvider = TestInternalThresholdProvider;
        type InternalAdminCountProvider = ();
        type MaxAdminsPerInstitution = ConstU32<32>;
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxAdminsPerInstitution = ConstU32<32>;
        type InternalVoteEngine = voting_engine::Pallet<Test>;
        type WeightInfo = ();
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");
        GenesisConfig::<Test>::default()
            .assimilate_storage(&mut storage)
            .expect("admins-change genesis should assimilate");
        let mut ext: sp_io::TestExternalities = storage.into();
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    fn nrc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[0].duoqian_admins[index])
    }

    fn prc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[1].duoqian_admins[index])
    }

    fn nrc_pallet_id() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
            .expect("NRC shenfen_id should map to valid shenfen_id institution id")
    }

    fn prc_pallet_id() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id)
            .expect("prc pallet_id should be valid shenfen_id institution id")
    }

    fn prb_pallet_id() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(CHINA_CH[0].shenfen_id)
            .expect("prb pallet_id should be valid shenfen_id institution id")
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CH[0].duoqian_admins[index])
    }

    fn pending_subject_id() -> InstitutionPalletId {
        [42u8; 48]
    }

    /// 获取最近一次 create_internal_proposal 分配的 proposal_id。
    fn last_proposal_id() -> u64 {
        voting_engine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
    }

    fn current_admins(institution: InstitutionPalletId) -> Vec<AccountId32> {
        Institutions::<Test>::get(institution)
            .expect("admin subject should be stored")
            .admins
            .into_inner()
    }

    fn mark_proposal_passed_without_callback(proposal_id: u64) {
        voting_engine::pallet::Proposals::<Test>::mutate(proposal_id, |maybe| {
            let proposal = maybe.as_mut().expect("proposal should exist");
            proposal.status = STATUS_PASSED;
        });
        let now = System::block_number();
        voting_engine::ProposalExecutionRetryStates::<Test>::insert(
            proposal_id,
            voting_engine::ExecutionRetryState {
                manual_attempts: 0,
                first_auto_failed_at: now,
                retry_deadline: now,
                last_attempt_at: None,
            },
        );
    }

    /// 测试辅助:走投票引擎公开 `internal_vote` extrinsic 投票(Phase 2 后的统一入口)。
    ///
    /// 替代旧的业务模块专属投票入口——业务模块不再持有投票 call,
    /// 所有管理员通过投票引擎的公开 call 直接投票,通过后由 `InternalVoteExecutor` 回调
    /// 执行业务。
    fn cast_vote(who: AccountId32, proposal_id: u64, approve: bool) -> DispatchResult {
        voting_engine::Pallet::<Test>::internal_vote(
            RuntimeOrigin::signed(who),
            proposal_id,
            approve,
        )
    }

    fn finalized_event_count(proposal_id: u64, expected_status: u8) -> usize {
        System::events()
            .into_iter()
            .filter(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::VotingEngine(voting_engine::Event::ProposalFinalized {
                        proposal_id: event_id,
                        status,
                    }) if *event_id == proposal_id && *status == expected_status
                )
            })
            .count()
    }

    #[test]
    fn pending_subject_is_not_exposed_to_active_business_api() {
        new_test_ext().execute_with(|| {
            let institution = pending_subject_id();
            let admin_a = AccountId32::new([211u8; 32]);
            let admin_b = AccountId32::new([212u8; 32]);

            assert_ok!(AdminsChange::do_create_pending_subject(
                institution,
                ORG_DUOQIAN,
                AdminSubjectKind::PersonalDuoqian,
                vec![admin_a.clone(), admin_b.clone()],
                2,
                admin_a.clone()
            ));

            assert!(!AdminsChange::is_active_subject_admin(
                ORG_DUOQIAN,
                institution,
                &admin_a
            ));
            assert!(AdminsChange::active_subject_admins(ORG_DUOQIAN, institution).is_none());
            assert_eq!(
                AdminsChange::pending_subject_admins_for_snapshot(ORG_DUOQIAN, institution)
                    .expect("pending snapshot admins should exist"),
                vec![admin_a.clone(), admin_b.clone()]
            );
            assert_eq!(
                AdminsChange::pending_subject_threshold_for_snapshot(ORG_DUOQIAN, institution),
                Some(2)
            );

            assert_ok!(AdminsChange::do_activate_subject(institution));
            assert!(AdminsChange::is_active_subject_admin(
                ORG_DUOQIAN,
                institution,
                &admin_a
            ));
            assert!(
                AdminsChange::pending_subject_admins_for_snapshot(ORG_DUOQIAN, institution)
                    .is_none()
            );
        });
    }

    #[test]
    fn subject_lifecycle_trait_requires_voting_engine_scope_for_activation() {
        new_test_ext().execute_with(|| {
            let institution = pending_subject_id();
            let admin_a = AccountId32::new([201u8; 32]);
            let admin_b = AccountId32::new([202u8; 32]);
            let proposal_id = <voting_engine::Pallet<Test> as InternalVoteEngine<
                AccountId32,
            >>::create_pending_subject_internal_proposal_with_snapshot_data(
                admin_a.clone(),
                ORG_DUOQIAN,
                institution,
                vec![admin_a.clone(), admin_b.clone()],
                2,
                b"dq-mgmt",
                b"subject-create".to_vec(),
            )
            .expect("pending subject proposal should be created");

            assert_ok!(AdminsChange::create_pending_subject_for_proposal(
                proposal_id,
                b"dq-mgmt",
                institution,
                ORG_DUOQIAN,
                AdminSubjectKind::PersonalDuoqian,
                vec![admin_a.clone(), admin_b],
                2,
                admin_a.clone()
            ));

            assert_noop!(
                AdminsChange::activate_subject_for_proposal(proposal_id, b"dq-mgmt", institution),
                Error::<Test>::InvalidSubjectLifecycleScope
            );

            voting_engine::pallet::Proposals::<Test>::mutate(proposal_id, |maybe| {
                let proposal = maybe.as_mut().expect("proposal should exist");
                proposal.status = STATUS_PASSED;
            });
            assert_noop!(
                AdminsChange::activate_subject_for_proposal(proposal_id, b"dq-mgmt", institution),
                Error::<Test>::InvalidSubjectLifecycleScope
            );

            voting_engine::pallet::CallbackExecutionScopes::<Test>::insert(proposal_id, ());
            assert_ok!(AdminsChange::activate_subject_for_proposal(
                proposal_id,
                b"dq-mgmt",
                institution
            ));
            voting_engine::pallet::CallbackExecutionScopes::<Test>::remove(proposal_id);
        });
    }

    #[test]
    fn builtin_subjects_cannot_be_closed() {
        new_test_ext().execute_with(|| {
            for (institution, org, admin) in [
                (nrc_pallet_id(), ORG_NRC, nrc_admin(0)),
                (prc_pallet_id(), ORG_PRC, prc_admin(0)),
                (prb_pallet_id(), ORG_PRB, prb_admin(0)),
            ] {
                assert_noop!(
                    AdminsChange::do_close_subject(institution),
                    Error::<Test>::BuiltinSubjectCannotClose
                );

                let subject = Institutions::<Test>::get(institution)
                    .expect("builtin subject should remain stored");
                assert_eq!(subject.kind, AdminSubjectKind::BuiltinInstitution);
                assert_eq!(subject.status, AdminSubjectStatus::Active);
                assert!(AdminsChange::is_active_subject_admin(
                    org,
                    institution,
                    &admin
                ));
            }
        });
    }

    #[test]
    fn dynamic_subjects_can_be_closed() {
        new_test_ext().execute_with(|| {
            for (offset, kind) in [
                (0u8, AdminSubjectKind::PersonalDuoqian),
                (1u8, AdminSubjectKind::SfidInstitution),
            ] {
                let mut institution = pending_subject_id();
                institution[0] = institution[0].saturating_add(offset);
                let admin_a = AccountId32::new([221u8.saturating_add(offset); 32]);
                let admin_b = AccountId32::new([231u8.saturating_add(offset); 32]);

                assert_ok!(AdminsChange::do_create_pending_subject(
                    institution,
                    ORG_DUOQIAN,
                    kind,
                    vec![admin_a.clone(), admin_b],
                    2,
                    admin_a.clone()
                ));
                assert_ok!(AdminsChange::do_activate_subject(institution));
                assert_ok!(AdminsChange::do_close_subject(institution));

                let subject = Institutions::<Test>::get(institution)
                    .expect("dynamic subject should remain stored");
                assert_eq!(subject.kind, kind);
                assert_eq!(subject.status, AdminSubjectStatus::Closed);
                assert!(!AdminsChange::is_active_subject_admin(
                    ORG_DUOQIAN,
                    institution,
                    &admin_a
                ));
                assert!(AdminsChange::active_subject_admins(ORG_DUOQIAN, institution).is_none());
            }
        });
    }

    #[test]
    fn duoqian_subjects_cannot_use_admin_replacement_entry() {
        new_test_ext().execute_with(|| {
            for (offset, kind) in [
                (0u8, AdminSubjectKind::PersonalDuoqian),
                (1u8, AdminSubjectKind::SfidInstitution),
            ] {
                let mut institution = pending_subject_id();
                institution[0] = institution[0].saturating_add(10u8.saturating_add(offset));
                let admin_a = AccountId32::new([41u8.saturating_add(offset); 32]);
                let admin_b = AccountId32::new([51u8.saturating_add(offset); 32]);
                let new_admin = AccountId32::new([61u8.saturating_add(offset); 32]);

                assert_ok!(AdminsChange::do_create_pending_subject(
                    institution,
                    ORG_DUOQIAN,
                    kind,
                    vec![admin_a.clone(), admin_b.clone()],
                    2,
                    admin_a.clone()
                ));
                assert_ok!(AdminsChange::do_activate_subject(institution));

                assert_noop!(
                    AdminsChange::propose_admin_replacement(
                        RuntimeOrigin::signed(admin_a.clone()),
                        ORG_DUOQIAN,
                        institution,
                        admin_b,
                        new_admin
                    ),
                    Error::<Test>::InvalidSubjectKind
                );
            }
        });
    }

    #[test]
    fn nrc_replacement_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([99u8; 32]);

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(cast_vote(nrc_admin(i), pid, true));
            }

            let admins = current_admins(institution);
            assert!(admins.iter().any(|a| a == &new_admin));
            assert!(!admins.iter().any(|a| a == &old_admin));
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTED
            );
            assert_eq!(finalized_event_count(pid, STATUS_EXECUTED), 1);
        });
    }

    #[test]
    fn non_nrc_admin_cannot_propose_nrc_replacement() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_noop!(
                AdminsChange::propose_admin_replacement(
                    RuntimeOrigin::signed(prc_admin(0)),
                    ORG_NRC,
                    institution,
                    nrc_admin(1),
                    AccountId32::new([77u8; 32])
                ),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn non_nrc_admin_cannot_vote_nrc_replacement() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([88u8; 32])
            ));
            let pid = last_proposal_id();

            assert_noop!(
                cast_vote(prc_admin(0), pid, true),
                voting_engine::pallet::Error::<Test>::NoPermission
            );
        });
    }

    #[test]
    fn replaced_new_admin_can_propose_next_replacement() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([66u8; 32]);

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin,
                new_admin.clone()
            ));
            let pid = last_proposal_id();
            for i in 0..13 {
                assert_ok!(cast_vote(nrc_admin(i), pid, true));
            }

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(new_admin),
                ORG_NRC,
                institution,
                nrc_admin(2),
                AccountId32::new([67u8; 32])
            ));
        });
    }

    #[test]
    fn prc_replacement_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_admin = prc_admin(1);
            let new_admin = AccountId32::new([55u8; 32]);

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));
            let pid = last_proposal_id();

            // 省储会内部投票阈值：>=6
            for i in 0..6 {
                assert_ok!(cast_vote(prc_admin(i), pid, true));
            }

            let admins = current_admins(institution);
            assert!(admins.iter().any(|a| a == &new_admin));
            assert!(!admins.iter().any(|a| a == &old_admin));
        });
    }

    #[test]
    fn prb_replacement_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prb_pallet_id();
            let old_admin = prb_admin(1);
            let new_admin = AccountId32::new([56u8; 32]);

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRB,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));
            let pid = last_proposal_id();

            // 省储行内部投票阈值：>=6
            for i in 0..6 {
                assert_ok!(cast_vote(prb_admin(i), pid, true));
            }

            let admins = current_admins(institution);
            assert!(admins.iter().any(|a| a == &new_admin));
            assert!(!admins.iter().any(|a| a == &old_admin));
        });
    }

    #[test]
    fn non_prc_admin_cannot_propose_or_vote_prc_replacement() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();

            assert_noop!(
                AdminsChange::propose_admin_replacement(
                    RuntimeOrigin::signed(prb_admin(0)),
                    ORG_PRC,
                    institution,
                    prc_admin(1),
                    AccountId32::new([57u8; 32])
                ),
                Error::<Test>::UnauthorizedAdmin
            );

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRC,
                institution,
                prc_admin(1),
                AccountId32::new([58u8; 32])
            ));
            let pid = last_proposal_id();

            assert_noop!(
                cast_vote(prb_admin(0), pid, true),
                voting_engine::pallet::Error::<Test>::NoPermission
            );
        });
    }

    #[test]
    fn non_prb_admin_cannot_propose_or_vote_prb_replacement() {
        new_test_ext().execute_with(|| {
            let institution = prb_pallet_id();

            assert_noop!(
                AdminsChange::propose_admin_replacement(
                    RuntimeOrigin::signed(prc_admin(0)),
                    ORG_PRB,
                    institution,
                    prb_admin(1),
                    AccountId32::new([59u8; 32])
                ),
                Error::<Test>::UnauthorizedAdmin
            );

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRB,
                institution,
                prb_admin(1),
                AccountId32::new([60u8; 32])
            ));
            let pid = last_proposal_id();

            assert_noop!(
                cast_vote(prc_admin(0), pid, true),
                voting_engine::pallet::Error::<Test>::NoPermission
            );
        });
    }

    #[test]
    fn regular_internal_proposal_blocks_admin_replacement() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(<voting_engine::Pallet<Test> as InternalVoteEngine<
                AccountId32,
            >>::create_internal_proposal(
                nrc_admin(0), ORG_NRC, institution,
            ));

            assert_noop!(
                AdminsChange::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(1)),
                    ORG_NRC,
                    institution,
                    nrc_admin(2),
                    AccountId32::new([89u8; 32])
                ),
                voting_engine::pallet::Error::<Test>::RegularInternalProposalActive
            );
        });
    }

    #[test]
    fn vote_does_not_rollback_when_auto_execute_fails() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([61u8; 32]);

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin
            ));
            let pid = last_proposal_id();

            Institutions::<Test>::mutate(institution, |maybe| {
                let subject = maybe.as_mut().expect("institution should exist");
                let admins = &mut subject.admins;
                let pos = admins
                    .iter()
                    .position(|a| a == &old_admin)
                    .expect("old_admin should be in admins");
                admins[pos] = nrc_admin(18);
            });

            for i in [0usize, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] {
                assert_ok!(cast_vote(nrc_admin(i), pid, true));
            }

            let proposal =
                voting_engine::Pallet::<Test>::proposals(pid).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_EXECUTION_FAILED);
            assert_eq!(finalized_event_count(pid, STATUS_EXECUTION_FAILED), 1);
            assert!(
                voting_engine::Pallet::<Test>::internal_proposal_mutex(ORG_NRC, institution)
                    .is_none()
            );
            let data = voting_engine::Pallet::<Test>::get_proposal_data(pid)
                .expect("proposal data should exist");
            assert!(voting_engine::Pallet::<Test>::is_proposal_owner(
                pid, MODULE_TAG
            ));
            let _action = AdminReplacementAction::<AccountId32>::decode(&mut &data[..])
                .expect("should decode");
            assert_noop!(
                AdminsChange::execute_admin_replacement(RuntimeOrigin::signed(nrc_admin(0)), pid),
                voting_engine::pallet::Error::<Test>::ProposalNotRetryable
            );
        });
    }

    #[test]
    fn org_mismatch_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                AdminsChange::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_PRC,
                    nrc_pallet_id(),
                    nrc_admin(1),
                    AccountId32::new([74u8; 32])
                ),
                Error::<Test>::InstitutionOrgMismatch
            );
        });
    }

    #[test]
    fn reject_vote_does_not_trigger_execution() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([75u8; 32]);

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));
            let pid = last_proposal_id();

            assert_ok!(cast_vote(nrc_admin(2), pid, false));

            let admins = current_admins(institution);
            assert!(admins.iter().any(|a| a == &old_admin));
            assert!(!admins.iter().any(|a| a == &new_admin));
            assert!(
                voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some(),
                "proposal data should exist"
            );
        });
    }

    #[test]
    fn propose_fails_when_old_admin_missing() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                AdminsChange::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    nrc_pallet_id(),
                    AccountId32::new([201u8; 32]),
                    AccountId32::new([202u8; 32])
                ),
                Error::<Test>::OldAdminNotFound
            );
        });
    }

    #[test]
    fn propose_fails_when_new_admin_already_exists() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                AdminsChange::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    nrc_pallet_id(),
                    nrc_admin(1),
                    nrc_admin(2)
                ),
                Error::<Test>::NewAdminAlreadyExists
            );
        });
    }

    #[test]
    fn executed_proposal_cannot_be_executed_again() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([203u8; 32])
            ));
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(cast_vote(nrc_admin(i), pid, true));
            }

            assert_noop!(
                AdminsChange::execute_admin_replacement(RuntimeOrigin::signed(nrc_admin(0)), pid),
                voting_engine::pallet::Error::<Test>::ProposalNotRetryable
            );
        });
    }

    #[test]
    fn rejected_proposal_does_not_block_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([206u8; 32])
            ));
            let pid1 = last_proposal_id();

            let end = voting_engine::Pallet::<Test>::proposals(pid1)
                .expect("proposal should exist")
                .end;
            System::set_block_number(end + 1);
            assert_ok!(voting_engine::Pallet::<Test>::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid1
            ));
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid1)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );

            // 中文注释：投票引擎全局限额管控后，被拒绝的提案不再阻塞同机构新提案。
            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(2),
                AccountId32::new([207u8; 32])
            ));
        });
    }

    #[test]
    fn failed_auto_execute_enters_terminal_status_and_cannot_retry() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([208u8; 32]);

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));
            let pid = last_proposal_id();

            Institutions::<Test>::mutate(institution, |maybe| {
                let subject = maybe.as_mut().expect("institution should exist");
                let admins = &mut subject.admins;
                let old_pos = admins
                    .iter()
                    .position(|a| a == &old_admin)
                    .expect("old_admin should be in admins");
                admins[old_pos] = nrc_admin(18);
            });

            for i in [0usize, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] {
                assert_ok!(cast_vote(nrc_admin(i), pid, true));
            }

            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTION_FAILED
            );
            assert!(voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some());
            assert!(
                voting_engine::Pallet::<Test>::internal_proposal_mutex(ORG_NRC, institution)
                    .is_none()
            );

            Institutions::<Test>::mutate(institution, |maybe| {
                let subject = maybe.as_mut().expect("institution should exist");
                let admins = &mut subject.admins;
                let restore_pos = admins
                    .iter()
                    .position(|a| a == &nrc_admin(18))
                    .expect("temporary admin marker should exist");
                admins[restore_pos] = old_admin.clone();
            });

            assert_noop!(
                AdminsChange::execute_admin_replacement(RuntimeOrigin::signed(nrc_admin(0)), pid),
                voting_engine::pallet::Error::<Test>::ProposalNotRetryable
            );
            let admins = current_admins(institution);
            assert!(!admins.iter().any(|a| a == &new_admin));
            assert!(admins.iter().any(|a| a == &old_admin));
        });
    }

    #[test]
    fn execute_admin_replacement_rejects_wrong_proposal_kind_or_stage() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([209u8; 32])
            ));
            let pid = last_proposal_id();
            mark_proposal_passed_without_callback(pid);

            voting_engine::pallet::Proposals::<Test>::mutate(pid, |maybe| {
                let proposal = maybe.as_mut().expect("proposal should exist");
                proposal.kind = voting_engine::PROPOSAL_KIND_JOINT;
            });
            assert_noop!(
                AdminsChange::execute_admin_replacement(RuntimeOrigin::signed(nrc_admin(0)), pid),
                voting_engine::pallet::Error::<Test>::ProposalOwnerMissing
            );

            voting_engine::pallet::Proposals::<Test>::mutate(pid, |maybe| {
                let proposal = maybe.as_mut().expect("proposal should exist");
                proposal.kind = voting_engine::PROPOSAL_KIND_INTERNAL;
                proposal.stage = voting_engine::STAGE_JOINT;
            });
            assert_ok!(AdminsChange::execute_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid
            ));
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTION_FAILED
            );
        });
    }

    #[test]
    fn execute_admin_replacement_rejects_proposal_metadata_mismatch() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([210u8; 32])
            ));
            let pid = last_proposal_id();
            mark_proposal_passed_without_callback(pid);

            voting_engine::pallet::Proposals::<Test>::mutate(pid, |maybe| {
                let proposal = maybe.as_mut().expect("proposal should exist");
                proposal.internal_institution = Some(prc_pallet_id());
            });
            assert_noop!(
                AdminsChange::execute_admin_replacement(RuntimeOrigin::signed(nrc_admin(0)), pid),
                voting_engine::pallet::Error::<Test>::NoPermission
            );

            voting_engine::pallet::Proposals::<Test>::mutate(pid, |maybe| {
                let proposal = maybe.as_mut().expect("proposal should exist");
                proposal.internal_institution = Some(institution);
                proposal.internal_org = Some(ORG_PRC);
            });
            assert_ok!(AdminsChange::execute_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid
            ));
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTION_FAILED
            );
        });
    }

    #[test]
    fn vote_below_threshold_does_not_trigger_execution() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([204u8; 32]);

            assert_ok!(AdminsChange::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));
            let pid = last_proposal_id();

            assert_ok!(cast_vote(nrc_admin(2), pid, true));

            let admins = current_admins(institution);
            assert!(admins.iter().any(|a| a == &old_admin));
            assert!(!admins.iter().any(|a| a == &new_admin));
            assert!(
                voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some(),
                "proposal data should exist"
            );
        });
    }

    #[test]
    fn invalid_institution_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                AdminsChange::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    [0u8; 48],
                    nrc_admin(1),
                    AccountId32::new([205u8; 32])
                ),
                Error::<Test>::InvalidInstitution
            );
        });
    }
}
