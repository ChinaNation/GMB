#![cfg_attr(not(feature = "std"), no_std)]
//! 管理员权限治理模块（admins-change）
//! - 本模块只负责“更换管理员”这一类业务事项
//! - 投票流程本身由 voting-engine 提供（内部投票）
//! - 约束：仅替换，不增删；且仅能在本机构范围内更换

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::StorageVersion, Blake2_128Concat};
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
    InstitutionPalletId, InternalVoteResultCallback, STATUS_EXECUTED, STATUS_PASSED,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"adm-rep";

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

const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

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
        #[cfg(feature = "std")]
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
        /// 管理员更换提案已提交一票
        AdminReplacementVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
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
        /// 管理员主体已存在
        InstitutionAlreadyExists,
        /// 管理员主体状态不是 Pending
        SubjectNotPending,
        /// 管理员主体状态不是 Active
        SubjectNotActive,
        /// 管理员主体类型与 org 不匹配
        InvalidSubjectKind,
        /// 阈值不合法
        InvalidThreshold,
        /// 管理员重复
        DuplicateAdmin,
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

            // 3) 在投票引擎中创建内部投票提案，并记录业务动作
            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;

            let action = AdminReplacementAction {
                institution,
                old_admin: old_admin.clone(),
                new_admin: new_admin.clone(),
            };
            let mut encoded = Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&action.encode());
            voting_engine::Pallet::<T>::store_proposal_data(proposal_id, encoded)?;
            voting_engine::Pallet::<T>::store_proposal_meta(
                proposal_id,
                frame_system::Pallet::<T>::block_number(),
            );

            Self::deposit_event(Event::<T>::AdminReplacementProposed {
                proposal_id,
                org,
                institution,
                proposer: who,
                old_admin,
                new_admin,
            });
            Ok(())
        }

        /// 任意人触发"已通过提案"的执行,用于自动执行失败后的补救重试。
        ///
        /// Phase 2 整改后投票一律走 `VotingEngine::internal_vote` 公开 call;
        /// 通过后由本模块的 `InternalVoteExecutor` 自动触发 `try_execute_replacement`。
        /// 若自动执行失败(如存储暂时不一致),任何签名账户都可以调用本 call 重试。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::execute_admin_replacement())]
        pub fn execute_admin_replacement(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::try_execute_replacement(proposal_id)
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

        /// 写入 Pending 管理员主体。创建多签机构/个人多签时先调用本方法，
        /// 让投票引擎能在创建提案时锁定管理员快照。
        pub fn create_pending_subject(
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
        pub fn activate_subject(institution: InstitutionPalletId) -> DispatchResult {
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
        pub fn remove_pending_subject(institution: InstitutionPalletId) -> DispatchResult {
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
        pub fn close_subject(institution: InstitutionPalletId) -> DispatchResult {
            Institutions::<T>::try_mutate(institution, |maybe| -> DispatchResult {
                let subject = maybe.as_mut().ok_or(Error::<T>::InvalidInstitution)?;
                ensure!(
                    subject.status == AdminSubjectStatus::Active,
                    Error::<T>::SubjectNotActive
                );
                subject.status = AdminSubjectStatus::Closed;
                subject.updated_at = frame_system::Pallet::<T>::block_number();
                Ok(())
            })?;
            Self::deposit_event(Event::<T>::AdminSubjectClosed { institution });
            Ok(())
        }

        pub fn is_subject_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            let Some(subject) = Institutions::<T>::get(institution) else {
                return false;
            };
            if subject.org != org || subject.status == AdminSubjectStatus::Closed {
                return false;
            }
            subject.admins.iter().any(|admin| admin == who)
        }

        pub fn subject_admins(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<Vec<T::AccountId>> {
            let subject = Institutions::<T>::get(institution)?;
            if subject.org != org || subject.status == AdminSubjectStatus::Closed {
                return None;
            }
            Some(subject.admins.into_inner())
        }

        pub fn subject_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            let subject = Institutions::<T>::get(institution)?;
            if subject.org != org || subject.status == AdminSubjectStatus::Closed {
                return None;
            }
            Some(subject.threshold)
        }

        pub fn subject_admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            let subject = Institutions::<T>::get(institution)?;
            if subject.org != org || subject.status == AdminSubjectStatus::Closed {
                return None;
            }
            Some(subject.admins.len() as u32)
        }

        /// 从投票引擎 ProposalData 中读取并解码本模块的业务数据。
        /// 先校验 MODULE_TAG 前缀，防止跨模块误解码。
        fn load_proposal_data(proposal_id: u64) -> Option<AdminReplacementAction<T::AccountId>> {
            let raw = voting_engine::Pallet::<T>::get_proposal_data(proposal_id)?;
            let tag = crate::MODULE_TAG;
            if raw.len() < tag.len() || &raw[..tag.len()] != tag {
                return None;
            }
            AdminReplacementAction::decode(&mut &raw[tag.len()..]).ok()
        }

        pub(crate) fn try_execute_replacement(proposal_id: u64) -> DispatchResult {
            let action =
                Self::load_proposal_data(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            Self::try_execute_replacement_from_action(proposal_id, action)
        }

        pub(crate) fn try_execute_replacement_from_action(
            proposal_id: u64,
            action: AdminReplacementAction<T::AccountId>,
        ) -> DispatchResult {
            // 仅在内部投票提案状态为 PASSED 时执行替换
            let proposal = voting_engine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
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

            // 标记为已执行，防止双重执行
            voting_engine::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;

            Ok(())
        }
    }
}

// ──── 投票终态回调:把已通过的管理员替换提案落地到链上 ────
//
// Phase 2 整改后业务模块不再自行处理投票,提案通过(或否决)由投票引擎
// 通过 [`voting_engine::InternalVoteResultCallback`] 广播回来。
// 本 Executor 按 `MODULE_TAG` 前缀认领本模块的提案,非己方直接 Ok(()) skip。
//
// 设计要点:
// - `approved = true` 时执行 `try_execute_replacement`,失败发 `AdminReplacementExecutionFailed`
//   事件但不返回 Err(否则投票引擎会回滚状态,票数白投);
// - `approved = false` 下本模块没有独立存储需要清理,直接 Ok(()) 返回;
// - 数据层异常(解码失败、MODULE_TAG 不匹配)返回 Err,触发 set_status_and_emit 回滚,
//   避免错误状态被提交。
pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(proposal_id: u64, approved: bool) -> DispatchResult {
        // Step 1:认领 — 检查 ProposalData 是否以 MODULE_TAG 开头。
        let raw = match voting_engine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(()), // 非本模块提案
        };

        if !approved {
            // 否决:无独立存储需清理(ProposalData 由投票引擎延迟清理)。
            return Ok(());
        }

        // Step 2:解码 action。异常视为数据层问题,回滚投票状态。
        let action =
            AdminReplacementAction::<T::AccountId>::decode(&mut &raw[crate::MODULE_TAG.len()..])
                .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;

        // Step 3:打业务执行时间戳(首次进入 PASSED)。
        voting_engine::Pallet::<T>::set_proposal_passed(
            proposal_id,
            frame_system::Pallet::<T>::block_number(),
        );

        // Step 4:执行替换。失败发事件,不回滚 — 提案保留 PASSED 状态,
        //         任何签名账户可通过 execute_admin_replacement 重试。
        if pallet::Pallet::<T>::try_execute_replacement_from_action(proposal_id, action).is_err() {
            pallet::Pallet::<T>::deposit_event(
                pallet::Event::<T>::AdminReplacementExecutionFailed { proposal_id },
            );
        }
        Ok(())
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
        internal_vote::{ORG_NRC, ORG_PRB, ORG_PRC},
        STATUS_PASSED, STATUS_REJECTED,
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
            pallet::Pallet::<Test>::is_subject_admin(org, institution, who)
        }

        fn get_admin_list(org: u8, institution: InstitutionPalletId) -> Option<Vec<AccountId32>> {
            if !matches!(org, ORG_NRC | ORG_PRC | ORG_PRB) {
                return None;
            }
            pallet::Pallet::<Test>::subject_admins(org, institution)
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
        type MaxCleanupStepsPerBlock = ConstU32<8>;
        type CleanupKeysPerStep = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<256>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        // Phase 2 整改:mock runtime 必须把本模块的 Executor 挂上,
        // 否则内部提案通过后业务执行回调不会触发,端到端测试失败。
        type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalThresholdProvider = ();
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
        storage.into()
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

    /// 测试辅助:走投票引擎公开 `internal_vote` extrinsic 投票(Phase 2 后的统一入口)。
    ///
    /// 替代旧的 `AdminsChange::vote_admin_replacement`——业务模块不再持有投票 call,
    /// 所有管理员通过投票引擎的公开 call 直接投票,通过后由 `InternalVoteExecutor` 回调
    /// 执行业务。
    fn cast_vote(who: AccountId32, proposal_id: u64, approve: bool) -> DispatchResult {
        voting_engine::Pallet::<Test>::internal_vote(
            RuntimeOrigin::signed(who),
            proposal_id,
            approve,
        )
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
            assert_eq!(proposal.status, STATUS_PASSED);
            let data = voting_engine::Pallet::<Test>::get_proposal_data(pid)
                .expect("proposal data should exist");
            let tag = MODULE_TAG;
            assert!(data.len() >= tag.len() && &data[..tag.len()] == tag);
            let _action = AdminReplacementAction::<AccountId32>::decode(&mut &data[tag.len()..])
                .expect("should decode");
            assert_noop!(
                AdminsChange::execute_admin_replacement(RuntimeOrigin::signed(nrc_admin(0)), pid),
                Error::<Test>::OldAdminNotFound
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
                Error::<Test>::ProposalNotPassed
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
    fn execute_admin_replacement_succeeds_after_failed_auto_execute() {
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
                STATUS_PASSED
            );
            assert!(voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some());

            Institutions::<Test>::mutate(institution, |maybe| {
                let subject = maybe.as_mut().expect("institution should exist");
                let admins = &mut subject.admins;
                let restore_pos = admins
                    .iter()
                    .position(|a| a == &nrc_admin(18))
                    .expect("temporary admin marker should exist");
                admins[restore_pos] = old_admin.clone();
            });

            assert_ok!(AdminsChange::execute_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid
            ));
            let admins = current_admins(institution);
            assert!(admins.iter().any(|a| a == &new_admin));
            assert!(!admins.iter().any(|a| a == &old_admin));
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
