#![cfg_attr(not(feature = "std"), no_std)]
//! 管理员权限治理模块（admins-change）
//! - 本模块只负责“更换管理员”这一类业务事项
//! - 投票流程本身由 votingengine 提供（内部投票）
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
use primitives::derive::subject_id_from_sfid_number;
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, RuntimeDebug};
use sp_std::collections::btree_set::BTreeSet;

use primitives::china::china_cb::CHINA_CB;
use primitives::china::china_ch::CHINA_CH;
use primitives::count_const::{
    NRC_ADMIN_COUNT, NRC_INTERNAL_THRESHOLD, PRB_ADMIN_COUNT, PRB_INTERNAL_THRESHOLD,
    PRC_ADMIN_COUNT, PRC_INTERNAL_THRESHOLD,
};
use votingengine::{
    types::{ORG_NRC, ORG_PRB, ORG_PRC, ORG_REN},
    InternalVoteResultCallback, ProposalExecutionOutcome, SubjectId, PROPOSAL_KIND_INTERNAL,
    STAGE_INTERNAL, STATUS_EXECUTION_FAILED, STATUS_PASSED, STATUS_REJECTED, STATUS_VOTING,
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
    pub institution: SubjectId,
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
pub struct AdminSubject<AdminList, AccountId, BlockNumber> {
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
/// 裸存储 mutator 保持 crate 内私有；调用方必须提供 votingengine 提案上下文，
/// 由 admins-change 再校验 owner、机构、状态和回调作用域。
pub trait SubjectLifecycle<AccountId> {
    fn create_pending_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: SubjectId,
        org: u8,
        kind: AdminSubjectKind,
        admins: Vec<AccountId>,
        threshold: u32,
        creator: AccountId,
    ) -> DispatchResult;

    fn activate_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: SubjectId,
    ) -> DispatchResult;

    fn remove_pending_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: SubjectId,
    ) -> DispatchResult;

    fn close_subject_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: SubjectId,
    ) -> DispatchResult;
}

/// admins-change pallet on-chain storage 版本。
///
/// - v0:历史值,本仓库未保留 v0 代码痕迹。
/// - v1:命名为 `Institutions` 的 StorageMap。
/// - v2:storage 改名 `Institutions` → `Subjects`(2026-05-06 C 阶段命名修正)。
///
/// 升级路径:`migrations::v1::MigrateV1ToV2` 物理 move_prefix,门控走 on_chain_storage_version。
const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

pub mod migrations;

fn nrc_subject_id() -> Option<SubjectId> {
    // 中文注释：国储会ID统一从常量数组读取并转码。
    CHINA_CB
        .first()
        .and_then(|n| subject_id_from_sfid_number(n.sfid_number))
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
    use votingengine::InternalVoteEngine;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        /// 单个机构管理员最大数量上限（用于 BoundedVec）
        type MaxAdminsPerInstitution: Get<u32>;

        /// 中文注释：内部投票引擎（返回真实 proposal_id，避免外部猜测 next_proposal_id）。
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        /// 该 pallet 的可配置权重实现。
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    pub type AdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAdminsPerInstitution>;

    pub type AdminSubjectOf<T> =
        AdminSubject<AdminsOf<T>, <T as frame_system::Config>::AccountId, BlockNumberFor<T>>;

    /// 统一管理员主体表：subject_id → 管理员、阈值和生命周期。
    ///
    /// 创世时写入国储会、省储会、省储行；SFID 机构多签由 `organization-manage`
    /// 写入，个人多签由 `personal-manage` 写入，投票通过后激活。
    #[pallet::storage]
    #[pallet::getter(fn subject_of)]
    pub type Subjects<T: Config> =
        StorageMap<_, Blake2_128Concat, SubjectId, AdminSubjectOf<T>, OptionQuery>;

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

    /// 构造内置机构（国储会/省储会/省储行）创世记录。
    ///
    /// 中文注释：创世期 panic 是设计意图——`CHINA_CB` / `CHINA_CH` 常量错配
    /// 或 `MaxAdminsPerInstitution` 不足时立即拒绝起链，绝不允许带病启动。
    /// 所有 panic 都携带 `sfid_number` 便于运维定位是哪条记录出错。
    fn build_builtin_institution<T: Config>(
        sfid_number: &'static str,
        org: u8,
        raw_admins: &'static [[u8; 32]],
    ) -> AdminSubjectOf<T> {
        let admins: Vec<T::AccountId> = raw_admins
            .iter()
            .map(|raw| {
                T::AccountId::decode(&mut &raw[..]).unwrap_or_else(|_| {
                    panic!(
                        "genesis: sfid_number {} 管理员账号 decode 失败",
                        sfid_number
                    )
                })
            })
            .collect();
        let bounded: AdminsOf<T> = admins.try_into().unwrap_or_else(|_| {
            panic!(
                "genesis: sfid_number {} 管理员数量超过 MaxAdminsPerInstitution",
                sfid_number
            )
        });
        let creator = bounded.first().cloned().unwrap_or_else(|| {
            panic!(
                "genesis: sfid_number {} 内置机构必须至少 1 个管理员",
                sfid_number
            )
        });
        let threshold =
            default_threshold(org).unwrap_or_else(|| panic!("genesis: org {} 没有默认阈值", org));
        AdminSubject {
            org,
            kind: AdminSubjectKind::BuiltinInstitution,
            admins: bounded,
            threshold,
            creator,
            created_at: Zero::zero(),
            updated_at: Zero::zero(),
            status: AdminSubjectStatus::Active,
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
                let Some(institution) = subject_id_from_sfid_number(node.sfid_number) else {
                    continue;
                };
                let org = if Some(institution) == nrc_subject_id() {
                    ORG_NRC
                } else {
                    ORG_PRC
                };
                Subjects::<T>::insert(
                    institution,
                    build_builtin_institution::<T>(node.sfid_number, org, node.duoqian_admins),
                );
            }

            for node in CHINA_CH.iter() {
                let Some(institution) = subject_id_from_sfid_number(node.sfid_number) else {
                    continue;
                };
                Subjects::<T>::insert(
                    institution,
                    build_builtin_institution::<T>(node.sfid_number, ORG_PRB, node.duoqian_admins),
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
            institution: SubjectId,
            proposer: T::AccountId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        },
        /// 提案达到通过状态但自动执行失败（投票不回滚）
        AdminReplacementExecutionFailed { proposal_id: u64 },
        /// 管理员列表已完成替换执行
        AdminReplaced {
            proposal_id: u64,
            institution: SubjectId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        },
        /// 多签主体管理员配置已写入 Pending。
        AdminSubjectPendingCreated {
            institution: SubjectId,
            org: u8,
            kind: AdminSubjectKind,
            creator: T::AccountId,
            admin_count: u32,
            threshold: u32,
        },
        /// 多签主体管理员配置已激活。
        AdminSubjectActivated { institution: SubjectId },
        /// Pending 多签主体管理员配置已清理。
        AdminSubjectPendingRemoved { institution: SubjectId },
        /// 多签主体管理员配置已关闭。
        AdminSubjectClosed { institution: SubjectId },
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
        /// 管理员主体生命周期写入缺少有效 votingengine 提案作用域
        InvalidSubjectLifecycleScope,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_admin_replacement())]
        pub fn propose_admin_replacement(
            origin: OriginFor<T>,
            org: u8,
            institution: SubjectId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 中文注释：本入口只治理制度内置主体(NRC/PRC/PRB)的管理员替换。
            // ORG_REN 的个人/机构多签主体由 organization-manage / personal-manage 维护生命周期,
            // 不能从通用管理员替换入口绕出第二条治理路径。
            ensure!(
                matches!(org, ORG_NRC | ORG_PRC | ORG_PRB),
                Error::<T>::InvalidSubjectKind
            );

            // 1) 校验管理员主体已激活且 org 匹配。
            let subject = Subjects::<T>::get(institution).ok_or(Error::<T>::InvalidInstitution)?;
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

        // call_index = 1 已废弃: execute_admin_replacement 已统一到
        // VotingEngine::retry_passed_proposal —— 前端必须直接调用投票引擎
        // 的 retry/cancel 入口,业务 pallet 不再保留任何 wrapper extrinsic。
    }

    impl<T: Config> Pallet<T> {
        fn admins_for_institution(
            institution: SubjectId,
        ) -> Result<Vec<T::AccountId>, DispatchError> {
            // 中文注释：创世后只信任链上管理员状态，不再回退常量管理员。
            let stored = Subjects::<T>::get(institution).ok_or(Error::<T>::InvalidInstitution)?;
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
                    ensure!(org == ORG_REN, Error::<T>::InvalidSubjectKind);
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
            institution: SubjectId,
            org: u8,
            expected_status: u8,
            require_callback_scope: bool,
        ) -> DispatchResult {
            ensure!(
                votingengine::Pallet::<T>::is_proposal_owner(proposal_id, module_tag),
                Error::<T>::InvalidSubjectLifecycleScope
            );
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
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
                    votingengine::Pallet::<T>::is_callback_execution_scope(proposal_id),
                    Error::<T>::InvalidSubjectLifecycleScope
                );
            }
            Ok(())
        }

        /// 写入 Pending 管理员主体。
        ///
        /// 中文注释：生命周期写入只能经 `SubjectLifecycle` trait 做提案上下文校验后进入。
        pub(crate) fn do_create_pending_subject(
            institution: SubjectId,
            org: u8,
            kind: AdminSubjectKind,
            admins: Vec<T::AccountId>,
            threshold: u32,
            creator: T::AccountId,
        ) -> DispatchResult {
            ensure!(
                !Subjects::<T>::contains_key(institution),
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
            Subjects::<T>::insert(
                institution,
                AdminSubject {
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
        pub(crate) fn do_activate_subject(institution: SubjectId) -> DispatchResult {
            Subjects::<T>::try_mutate(institution, |maybe| -> DispatchResult {
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
        pub(crate) fn do_remove_pending_subject(institution: SubjectId) -> DispatchResult {
            if let Some(subject) = Subjects::<T>::get(institution) {
                ensure!(
                    subject.status == AdminSubjectStatus::Pending,
                    Error::<T>::SubjectNotPending
                );
                Subjects::<T>::remove(institution);
                Self::deposit_event(Event::<T>::AdminSubjectPendingRemoved { institution });
            }
            Ok(())
        }

        /// 关闭已激活管理员主体。
        pub(crate) fn do_close_subject(institution: SubjectId) -> DispatchResult {
            Subjects::<T>::try_mutate(institution, |maybe| -> DispatchResult {
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
            institution: SubjectId,
            status: AdminSubjectStatus,
        ) -> Option<AdminSubjectOf<T>> {
            let subject = Subjects::<T>::get(institution)?;
            if subject.org != org || subject.status != status {
                return None;
            }
            Some(subject)
        }

        /// 查询 Active 主体是否存在。普通业务主体合法性判断只使用 Active 主体。
        pub fn active_subject_exists(org: u8, institution: SubjectId) -> bool {
            Self::subject_with_status(org, institution, AdminSubjectStatus::Active).is_some()
        }

        /// 查询 Active 主体管理员权限。普通业务授权只能使用 Active 主体。
        pub fn is_active_subject_admin(
            org: u8,
            institution: SubjectId,
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
        pub fn active_subject_admins(org: u8, institution: SubjectId) -> Option<Vec<T::AccountId>> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Active)?;
            Some(subject.admins.into_inner())
        }

        /// 读取 Active 主体阈值。普通业务投票只能使用 Active 阈值。
        pub fn active_subject_threshold(org: u8, institution: SubjectId) -> Option<u32> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Active)?;
            Some(subject.threshold)
        }

        /// 读取 Active 主体管理员数量。普通业务阈值兜底判断只能使用 Active 主体。
        pub fn active_subject_admin_count(org: u8, institution: SubjectId) -> Option<u32> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Active)?;
            Some(subject.admins.len() as u32)
        }

        /// 查询 Pending 主体是否存在。仅用于创建/激活该主体时判断主体合法性。
        pub fn pending_subject_exists_for_snapshot(org: u8, institution: SubjectId) -> bool {
            Self::subject_with_status(org, institution, AdminSubjectStatus::Pending).is_some()
        }

        /// 查询 Pending 主体管理员权限。仅用于创建/激活该主体时锁定投票快照。
        pub fn is_pending_subject_admin_for_snapshot(
            org: u8,
            institution: SubjectId,
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
            institution: SubjectId,
        ) -> Option<Vec<T::AccountId>> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Pending)?;
            Some(subject.admins.into_inner())
        }

        /// 读取 Pending 主体阈值。仅供投票引擎 Pending 创建入口写阈值快照。
        pub fn pending_subject_threshold_for_snapshot(
            org: u8,
            institution: SubjectId,
        ) -> Option<u32> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Pending)?;
            Some(subject.threshold)
        }

        /// 读取 Pending 主体管理员数量。仅用于创建/激活该主体的快照语义。
        pub fn pending_subject_admin_count_for_snapshot(
            org: u8,
            institution: SubjectId,
        ) -> Option<u32> {
            let subject = Self::subject_with_status(org, institution, AdminSubjectStatus::Pending)?;
            Some(subject.admins.len() as u32)
        }

        pub(crate) fn try_execute_replacement_from_action(
            proposal_id: u64,
            action: AdminReplacementAction<T::AccountId>,
        ) -> DispatchResult {
            // 中文注释：执行前同时校验投票引擎元数据与业务 action，避免跨模块误消费。
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
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
                Subjects::<T>::get(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
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
            votingengine::Pallet::<T>::ensure_admin_set_mutation_lock_owner(
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
            Subjects::<T>::mutate(action.institution, |maybe| {
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
        institution: SubjectId,
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
        institution: SubjectId,
    ) -> DispatchResult {
        let subject = pallet::Subjects::<T>::get(institution)
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
        institution: SubjectId,
    ) -> DispatchResult {
        let subject = pallet::Subjects::<T>::get(institution)
            .ok_or(pallet::Error::<T>::InvalidInstitution)?;
        let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
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
        institution: SubjectId,
    ) -> DispatchResult {
        let subject = pallet::Subjects::<T>::get(institution)
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
// 投票统一由投票引擎承担,提案通过(或否决)经
// [`votingengine::InternalVoteResultCallback`] 广播回来。
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
        if !votingengine::Pallet::<T>::is_proposal_owner(proposal_id, crate::MODULE_TAG) {
            return Ok(ProposalExecutionOutcome::Ignored);
        }
        let raw = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
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
mod tests;
