#![cfg_attr(not(feature = "std"), no_std)]
//! 管理员权限治理模块（admins-change）
//! - 本模块只负责“管理员集合变更”这一类业务事项
//! - 投票流程本身由 votingengine 提供（内部投票）
//! - 约束：治理机构固定人数，仅允许等长更换；动态账户允许增删改。
//!   阈值校验、保存和更新统一由 votingengine/internal-vote 负责。

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
use sp_runtime::{traits::Zero, DispatchError, RuntimeDebug};
use sp_std::collections::btree_set::BTreeSet;

use primitives::china::china_cb::CHINA_CB;
use primitives::china::china_ch::CHINA_CH;
use primitives::china::china_jc::CHINA_JC;
use primitives::china::china_jy::CHINA_JY;
use primitives::china::china_lf::CHINA_LF;
use primitives::china::china_sf::CHINA_SF;
use primitives::china::china_zf::CHINA_ZF;
use primitives::count_const::{NRC_ADMIN_COUNT, PRB_ADMIN_COUNT, PRC_ADMIN_COUNT};
use votingengine::{
    types::{ORG_NRC, ORG_OTH, ORG_PRB, ORG_PRC, ORG_PUP, ORG_REN},
    InternalVoteResultCallback, ProposalExecutionOutcome, PROPOSAL_KIND_INTERNAL, STAGE_INTERNAL,
    STATUS_EXECUTION_FAILED, STATUS_PASSED, STATUS_REJECTED, STATUS_VOTING,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
/// 中文注释：tag 带 schema 版本号；开发期不兼容旧管理员替换提案数据。
pub const MODULE_TAG: &[u8] = b"adm-set-v1";

#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(AccountId, AdminList))]
pub struct AdminSetChangeAction<AccountId, AdminList> {
    /// 目标多签账户地址（内置治理机构/个人账户/机构账户）。
    pub account: AccountId,
    /// 提案通过后写入的完整管理员集合。
    pub admins: AdminList,
    /// 提案通过后写入投票引擎的动态阈值；固定治理机构必须等于制度固定阈值。
    pub new_threshold: u32,
}

/// 管理员账户类型。所有需要内部投票的多签账户都在本模块统一登记。
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
pub enum AdminAccountKind {
    /// 国储会、省储会、省储行等创世内置机构。
    BuiltinInstitution,
    /// 用户自建的个人多签账户。
    PersonalAccount,
    /// CID 机构下面的某个具体账户。
    InstitutionAccount,
}

/// 管理员账户生命周期。
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
pub enum AdminAccountStatus {
    /// 创建提案投票中；投票引擎可读取管理员快照。
    Pending,
    /// 已激活，可发起常规治理/转账/管理员更换。
    Active,
    /// 已关闭，管理员不再有效。
    Closed,
}

/// 统一管理员账户记录。
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
pub struct AdminAccount<AdminList, AccountId, BlockNumber> {
    pub org: u8,
    pub kind: AdminAccountKind,
    pub admins: AdminList,
    pub creator: AccountId,
    pub created_at: BlockNumber,
    pub updated_at: BlockNumber,
    pub status: AdminAccountStatus,
}

/// 管理员账户生命周期写入口。
///
/// 中文注释：这里是跨 pallet 唯一允许写 Pending/Active/Closed 生命周期的 API。
/// 裸存储 mutator 保持 crate 内私有；调用方必须提供 votingengine 提案上下文，
/// 由 admins-change 再校验 owner、机构、状态和回调作用域。
pub trait AdminAccountLifecycle<AccountId> {
    fn create_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: AccountId,
        org: u8,
        kind: AdminAccountKind,
        admins: Vec<AccountId>,
        creator: AccountId,
    ) -> DispatchResult;

    fn activate_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: AccountId,
    ) -> DispatchResult;

    fn remove_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: AccountId,
    ) -> DispatchResult;

    fn close_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: AccountId,
    ) -> DispatchResult;
}

/// admins-change pallet on-chain storage 版本。
/// 全新创世口径:创世即终态布局,storage 版本恒为 v1,不承载任何历史迁移。
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

fn decode_account<T: frame_system::Config>(raw: &[u8; 32]) -> Option<T::AccountId> {
    T::AccountId::decode(&mut &raw[..]).ok()
}

fn nrc_account<T: frame_system::Config>() -> Option<T::AccountId> {
    CHINA_CB
        .first()
        .and_then(|n| decode_account::<T>(&n.main_account))
}

fn expected_admins_len(org: u8) -> Option<u32> {
    match org {
        ORG_NRC => Some(NRC_ADMIN_COUNT),
        ORG_PRC => Some(PRC_ADMIN_COUNT),
        ORG_PRB => Some(PRB_ADMIN_COUNT),
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
        /// 单个机构账户管理员最大数量上限（用于 BoundedVec，运行时目标值 1989）
        type MaxAdminsPerInstitution: Get<u32>;

        #[pallet::constant]
        /// 单个个人账户管理员最大数量上限（运行时目标值 64）
        type MaxPersonalAccountAdmins: Get<u32>;

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

    pub type AdminAccountOf<T> =
        AdminAccount<AdminsOf<T>, <T as frame_system::Config>::AccountId, BlockNumberFor<T>>;

    /// 统一管理员账户表：多签账户地址 → 管理员和生命周期。
    ///
    /// 创世时写入国储会、省储会、省储行；个人账户由 `personal-manage` 写入；
    /// 机构账户由 `organization-manage` 在后续账户级改造中写入，投票通过后激活。
    #[pallet::storage]
    #[pallet::getter(fn admin_account_of)]
    pub type AdminAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AdminAccountOf<T>, OptionQuery>;

    /// 中文注释:创世初始机构封存表（CID 系统根基,永不可注销关闭）。
    ///
    /// 仅 `build()` 写入 china_cb/ch/zf/sf/jc/jy/lf 的机构主账户(联邦注册局、治理机构、
    /// 顶层政府/立法/司法/监察/教育);创世后无任何 extrinsic 可改。organization-manage
    /// 的关闭入口据此硬拒(见 `is_genesis_protected` + `ensure_closeable`)。行政区生成、
    /// 由 organization-manage 创建出来的机构(市注册局/公安局/公司)不在此表,可正常注销。
    #[pallet::storage]
    pub type ProtectedGenesisAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

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
    /// 所有 panic 都携带 `cid_number` 便于运维定位是哪条记录出错。
    fn build_builtin_institution<T: Config>(
        cid_number: &'static str,
        org: u8,
        raw_admins: &'static [[u8; 32]],
    ) -> AdminAccountOf<T> {
        let admins: Vec<T::AccountId> = raw_admins
            .iter()
            .map(|raw| {
                T::AccountId::decode(&mut &raw[..]).unwrap_or_else(|_| {
                    panic!("genesis: cid_number {} 管理员账号 decode 失败", cid_number)
                })
            })
            .collect();
        let bounded: AdminsOf<T> = admins.try_into().unwrap_or_else(|_| {
            panic!(
                "genesis: cid_number {} 管理员数量超过 MaxAdminsPerInstitution",
                cid_number
            )
        });
        let creator = bounded.first().cloned().unwrap_or_else(|| {
            panic!(
                "genesis: cid_number {} 内置机构必须至少 1 个管理员",
                cid_number
            )
        });
        AdminAccount {
            org,
            kind: AdminAccountKind::BuiltinInstitution,
            admins: bounded,
            creator,
            created_at: Zero::zero(),
            updated_at: Zero::zero(),
            status: AdminAccountStatus::Active,
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
            assert!(
                <T as Config>::MaxPersonalAccountAdmins::get() >= 2,
                "MaxPersonalAccountAdmins must be >= 2"
            );
            assert!(
                <T as Config>::MaxAdminsPerInstitution::get()
                    >= <T as Config>::MaxPersonalAccountAdmins::get(),
                "MaxAdminsPerInstitution must cover the physical BoundedVec maximum"
            );
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for node in CHINA_CB.iter() {
                let Some(institution) = decode_account::<T>(&node.main_account) else {
                    panic!("genesis: cid_number {} 主账户 decode 失败", node.cid_number);
                };
                let org = if Some(institution.clone()) == nrc_account::<T>() {
                    ORG_NRC
                } else {
                    ORG_PRC
                };
                ProtectedGenesisAccounts::<T>::insert(institution.clone(), ());
                AdminAccounts::<T>::insert(
                    institution,
                    build_builtin_institution::<T>(node.cid_number, org, node.admins),
                );
            }

            for node in CHINA_CH.iter() {
                let Some(institution) = decode_account::<T>(&node.main_account) else {
                    panic!("genesis: cid_number {} 主账户 decode 失败", node.cid_number);
                };
                ProtectedGenesisAccounts::<T>::insert(institution.clone(), ());
                AdminAccounts::<T>::insert(
                    institution,
                    build_builtin_institution::<T>(node.cid_number, ORG_PRB, node.admins),
                );
            }

            // 中文注释:公权机构(政府/立法/司法/监察/教育)创世内置管理员统一写入 admins-change,
            // org 一律 ORG_PUP(动态账户,管理员变更走 propose_admin_set_change)。
            // 总统府联邦注册局随 CHINA_ZF 一并写入,链上单一真源,CID 不再内置注册局管理员。
            macro_rules! insert_pup_builtin {
                ($arr:expr) => {
                    for node in $arr.iter() {
                        let Some(institution) = decode_account::<T>(&node.main_account) else {
                            panic!(
                                "genesis: cid_number {} 主账户 decode 失败",
                                node.cid_number
                            );
                        };
                        ProtectedGenesisAccounts::<T>::insert(institution.clone(), ());
                        AdminAccounts::<T>::insert(
                            institution,
                            build_builtin_institution::<T>(node.cid_number, ORG_PUP, node.admins),
                        );
                    }
                };
            }
            insert_pup_builtin!(CHINA_ZF);
            insert_pup_builtin!(CHINA_SF);
            insert_pup_builtin!(CHINA_JC);
            insert_pup_builtin!(CHINA_JY);
            insert_pup_builtin!(CHINA_LF);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 已发起管理员集合变更提案（并已在投票引擎创建内部提案）
        AdminSetChangeProposed {
            proposal_id: u64,
            org: u8,
            account: T::AccountId,
            proposer: T::AccountId,
            old_admins_len: u32,
            new_admins_len: u32,
            new_threshold: u32,
        },
        /// 提案达到通过状态但自动执行失败（投票不回滚）
        AdminSetChangeExecutionFailed { proposal_id: u64 },
        /// 管理员集合已完成执行
        AdminSetChanged {
            proposal_id: u64,
            account: T::AccountId,
            admins_len: u32,
            threshold: u32,
        },
        /// 多签账户管理员配置已写入 Pending。
        AdminAccountPendingCreated {
            account: T::AccountId,
            org: u8,
            kind: AdminAccountKind,
            creator: T::AccountId,
            admins_len: u32,
        },
        /// 多签账户管理员配置已激活。
        AdminAccountActivated { account: T::AccountId, org: u8 },
        /// Pending 多签账户管理员配置已清理。
        AdminAccountPendingRemoved { account: T::AccountId, org: u8 },
        /// 多签账户管理员配置已关闭。
        AdminAccountClosed { account: T::AccountId, org: u8 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 无效机构
        InvalidInstitution,
        /// 机构类型与 org 参数不匹配
        InstitutionOrgMismatch,
        /// 管理员数量不符合固定人数约束
        InvalidAdminsLen,
        /// 非该机构管理员，无权限
        UnauthorizedAdmin,
        /// 管理员集合没有发生变化
        AdminSetUnchanged,
        /// 找不到与投票提案绑定的管理员集合变更动作
        ProposalActionNotFound,
        /// 投票尚未通过，不能执行替换
        ProposalNotPassed,
        /// 提案类型不是内部投票
        InvalidProposalKind,
        /// 提案阶段不是内部投票阶段
        InvalidProposalStage,
        /// 提案绑定机构与管理员更换动作不一致
        ProposalInstitutionMismatch,
        /// 提案绑定组织与管理员账户不一致
        ProposalOrgMismatch,
        /// 管理员账户已存在
        InstitutionAlreadyExists,
        /// 管理员账户状态不是 Pending
        AdminAccountNotPending,
        /// 管理员账户状态不是 Active
        AdminAccountNotActive,
        /// 内置治理机构永远不可关闭
        BuiltinAdminAccountCannotClose,
        /// 管理员账户类型与 org 不匹配
        InvalidAdminAccountKind,
        /// 阈值不合法
        InvalidThreshold,
        /// 管理员重复
        DuplicateAdmin,
        /// 管理员账户生命周期写入缺少有效 votingengine 提案作用域
        InvalidAdminAccountLifecycleScope,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_admin_set_change())]
        pub fn propose_admin_set_change(
            origin: OriginFor<T>,
            org: u8,
            account: T::AccountId,
            admins: AdminsOf<T>,
            new_threshold: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1) 校验管理员账户已激活且 org 匹配。
            let current =
                AdminAccounts::<T>::get(account.clone()).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                current.status == AdminAccountStatus::Active,
                Error::<T>::AdminAccountNotActive
            );
            ensure!(current.org == org, Error::<T>::InstitutionOrgMismatch);

            // 2) 校验发起人与目标管理员集合合法性。
            let current_admins = current.admins.clone().into_inner();
            ensure!(current_admins.contains(&who), Error::<T>::UnauthorizedAdmin);
            Self::validate_admin_set_for_account(current.kind, current.org, admins.as_slice())?;
            ensure!(
                !Self::same_admin_set(current_admins.as_slice(), admins.as_slice()),
                Error::<T>::AdminSetUnchanged
            );
            // 3) 在同一个链上事务中创建投票提案、互斥锁和业务数据。
            with_transaction(|| {
                let action = AdminSetChangeAction {
                    account: account.clone(),
                    admins: admins.clone(),
                    new_threshold,
                };
                let encoded = action.encode();
                let proposal_id =
                    match T::InternalVoteEngine::create_admin_change_internal_proposal_with_data(
                        who.clone(),
                        org,
                        account.clone(),
                        admins.len() as u32,
                        new_threshold,
                        crate::MODULE_TAG,
                        encoded,
                    ) {
                        Ok(proposal_id) => proposal_id,
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    };

                Self::deposit_event(Event::<T>::AdminSetChangeProposed {
                    proposal_id,
                    org,
                    account,
                    proposer: who,
                    old_admins_len: current_admins.len() as u32,
                    new_admins_len: admins.len() as u32,
                    new_threshold,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// 中文注释:账户是否为创世封存的初始机构(CID 系统根基,永不可注销关闭)。
        /// 供 organization-manage 关闭入口做硬保护;数据由 `build()` 写入 `ProtectedGenesisAccounts`。
        pub fn is_genesis_protected(account: &T::AccountId) -> bool {
            ProtectedGenesisAccounts::<T>::contains_key(account)
        }

        fn validate_admins_len_for_account(
            kind: AdminAccountKind,
            org: u8,
            admins_len: usize,
        ) -> DispatchResult {
            match kind {
                AdminAccountKind::BuiltinInstitution => match expected_admins_len(org) {
                    // 治理机构固定人数约束：国储会19，省储会9，省储行9。
                    Some(expected) => ensure!(
                        admins_len == expected as usize,
                        Error::<T>::InvalidAdminsLen
                    ),
                    // 中文注释:PUP 创世机构(如联邦注册局)管理员数动态(自治可增减),
                    // 走与 InstitutionAccount 相同的可变上限,不锁固定人数。org 已由
                    // ensure_account_kind_matches_org 限定,此处 None 即 ORG_PUP。
                    None => {
                        ensure!(admins_len >= 2, Error::<T>::InvalidAdminsLen);
                        ensure!(
                            admins_len <= <T as Config>::MaxAdminsPerInstitution::get() as usize,
                            Error::<T>::InvalidAdminsLen
                        );
                    }
                },
                AdminAccountKind::PersonalAccount => {
                    ensure!(admins_len >= 2, Error::<T>::InvalidAdminsLen);
                    ensure!(
                        admins_len <= <T as Config>::MaxPersonalAccountAdmins::get() as usize,
                        Error::<T>::InvalidAdminsLen
                    );
                }
                AdminAccountKind::InstitutionAccount => {
                    ensure!(admins_len >= 2, Error::<T>::InvalidAdminsLen);
                    ensure!(
                        admins_len <= <T as Config>::MaxAdminsPerInstitution::get() as usize,
                        Error::<T>::InvalidAdminsLen
                    );
                }
            }
            Ok(())
        }

        fn validate_admin_set_for_account(
            kind: AdminAccountKind,
            org: u8,
            admins: &[T::AccountId],
        ) -> DispatchResult {
            Self::ensure_account_kind_matches_org(kind, org)?;
            Self::validate_admins_len_for_account(kind, org, admins.len())?;
            Self::ensure_unique_admins(admins)?;
            Ok(())
        }

        fn ensure_unique_admins(admins: &[T::AccountId]) -> DispatchResult {
            let mut seen = BTreeSet::new();
            for admin in admins {
                ensure!(seen.insert(admin.clone()), Error::<T>::DuplicateAdmin);
            }
            Ok(())
        }

        fn same_admin_set(left: &[T::AccountId], right: &[T::AccountId]) -> bool {
            if left.len() != right.len() {
                return false;
            }
            let left_set: BTreeSet<T::AccountId> = left.iter().cloned().collect();
            let right_set: BTreeSet<T::AccountId> = right.iter().cloned().collect();
            left_set == right_set
        }

        fn ensure_account_kind_matches_org(kind: AdminAccountKind, org: u8) -> DispatchResult {
            match kind {
                AdminAccountKind::BuiltinInstitution => {
                    // 中文注释:创世内置机构含治理机构(NRC/PRC/PRB)与公权机构(PUP,如联邦
                    // 注册局/顶层政府)。PUP 内置走动态管理员自治(propose_admin_set_change),
                    // 故同样接受 ORG_PUP;人数约束在 validate_admins_len_for_account 内分流。
                    ensure!(
                        matches!(org, ORG_NRC | ORG_PRC | ORG_PRB | ORG_PUP),
                        Error::<T>::InvalidAdminAccountKind
                    );
                }
                AdminAccountKind::PersonalAccount => {
                    ensure!(org == ORG_REN, Error::<T>::InvalidAdminAccountKind);
                }
                AdminAccountKind::InstitutionAccount => {
                    ensure!(
                        matches!(org, ORG_PUP | ORG_OTH),
                        Error::<T>::InvalidAdminAccountKind
                    );
                }
            }
            Ok(())
        }

        pub(crate) fn ensure_lifecycle_proposal(
            proposal_id: u64,
            module_tag: &[u8],
            institution: T::AccountId,
            org: u8,
            expected_status: u8,
            require_callback_scope: bool,
        ) -> DispatchResult {
            ensure!(
                votingengine::Pallet::<T>::is_proposal_owner(proposal_id, module_tag),
                Error::<T>::InvalidAdminAccountLifecycleScope
            );
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::InvalidAdminAccountLifecycleScope)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL,
                Error::<T>::InvalidAdminAccountLifecycleScope
            );
            ensure!(
                proposal.stage == STAGE_INTERNAL,
                Error::<T>::InvalidAdminAccountLifecycleScope
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
                Error::<T>::InvalidAdminAccountLifecycleScope
            );
            if require_callback_scope {
                ensure!(
                    votingengine::Pallet::<T>::is_callback_execution_scope(proposal_id),
                    Error::<T>::InvalidAdminAccountLifecycleScope
                );
            }
            Ok(())
        }

        /// 写入 Pending 管理员账户。
        ///
        /// 中文注释：生命周期写入只能经 `AdminAccountLifecycle` trait 做提案上下文校验后进入。
        pub(crate) fn do_create_pending_admin_account(
            institution: T::AccountId,
            org: u8,
            kind: AdminAccountKind,
            admins: Vec<T::AccountId>,
            creator: T::AccountId,
        ) -> DispatchResult {
            ensure!(
                !AdminAccounts::<T>::contains_key(institution.clone()),
                Error::<T>::InstitutionAlreadyExists
            );
            Self::validate_admin_set_for_account(kind, org, &admins)?;

            let bounded: AdminsOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            let now = frame_system::Pallet::<T>::block_number();
            let admins_len = bounded.len() as u32;
            AdminAccounts::<T>::insert(
                institution.clone(),
                AdminAccount {
                    org,
                    kind,
                    admins: bounded,
                    creator: creator.clone(),
                    created_at: now,
                    updated_at: now,
                    status: AdminAccountStatus::Pending,
                },
            );
            Self::deposit_event(Event::<T>::AdminAccountPendingCreated {
                account: institution,
                org,
                kind,
                creator,
                admins_len,
            });
            Ok(())
        }

        /// 将 Pending 管理员账户激活。
        pub(crate) fn do_activate_admin_account(institution: T::AccountId) -> DispatchResult {
            let org = AdminAccounts::<T>::try_mutate(
                institution.clone(),
                |maybe| -> Result<u8, DispatchError> {
                    let account = maybe.as_mut().ok_or(Error::<T>::InvalidInstitution)?;
                    ensure!(
                        account.status == AdminAccountStatus::Pending,
                        Error::<T>::AdminAccountNotPending
                    );
                    account.status = AdminAccountStatus::Active;
                    account.updated_at = frame_system::Pallet::<T>::block_number();
                    Ok(account.org)
                },
            )?;
            Self::deposit_event(Event::<T>::AdminAccountActivated {
                account: institution,
                org,
            });
            Ok(())
        }

        /// 清理尚未激活的 Pending 管理员账户。
        pub(crate) fn do_remove_pending_admin_account(institution: T::AccountId) -> DispatchResult {
            // 中文注释：Pending 清理必须命中真实账户，避免不存在账户被静默当作清理成功。
            let account = AdminAccounts::<T>::get(institution.clone())
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                account.status == AdminAccountStatus::Pending,
                Error::<T>::AdminAccountNotPending
            );
            let org = account.org;
            AdminAccounts::<T>::remove(institution.clone());
            Self::deposit_event(Event::<T>::AdminAccountPendingRemoved {
                account: institution,
                org,
            });
            Ok(())
        }

        /// 关闭已激活管理员账户。
        pub(crate) fn do_close_admin_account(institution: T::AccountId) -> DispatchResult {
            let account = AdminAccounts::<T>::get(institution.clone())
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                account.status == AdminAccountStatus::Active,
                Error::<T>::AdminAccountNotActive
            );
            // 中文注释：NRC/PRC/PRB 是制度内置治理账户，生命周期不能被删除。
            ensure!(
                !matches!(account.kind, AdminAccountKind::BuiltinInstitution),
                Error::<T>::BuiltinAdminAccountCannotClose
            );
            let org = account.org;
            // 中文注释：动态多签注销完成后不保留 Closed 当前状态墓碑；
            // 同名确定性地址可在资金清空后重新走全新的注册流程。
            AdminAccounts::<T>::remove(institution.clone());
            Self::deposit_event(Event::<T>::AdminAccountClosed {
                account: institution,
                org,
            });
            Ok(())
        }

        fn admin_account_with_status(
            org: u8,
            institution: T::AccountId,
            status: AdminAccountStatus,
        ) -> Option<AdminAccountOf<T>> {
            let account = AdminAccounts::<T>::get(institution)?;
            if account.org != org || account.status != status {
                return None;
            }
            // 中文注释：读侧也要执行账户类型边界校验，避免升级前写入的旧脏数据
            // 继续通过 active/pending 查询 API 被其他业务模块当作有效管理员账户。
            if Self::ensure_account_kind_matches_org(account.kind, account.org).is_err() {
                return None;
            }
            Some(account)
        }

        /// 查询 Active 账户是否存在。普通业务账户合法性判断只使用 Active 账户。
        pub fn active_admin_account_exists(org: u8, institution: T::AccountId) -> bool {
            Self::admin_account_with_status(org, institution, AdminAccountStatus::Active).is_some()
        }

        /// 查询 Active 账户管理员权限。普通业务授权只能使用 Active 账户。
        pub fn is_active_account_admin(
            org: u8,
            institution: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            let Some(account) =
                Self::admin_account_with_status(org, institution, AdminAccountStatus::Active)
            else {
                return false;
            };
            account.admins.iter().any(|admin| admin == who)
        }

        /// 读取 Active 账户管理员列表。普通业务提案创建和投票快照默认使用此 API。
        pub fn active_account_admins(
            org: u8,
            institution: T::AccountId,
        ) -> Option<Vec<T::AccountId>> {
            let account =
                Self::admin_account_with_status(org, institution, AdminAccountStatus::Active)?;
            Some(account.admins.into_inner())
        }

        /// 读取 Active 账户管理员数量。普通业务阈值兜底判断只能使用 Active 账户。
        pub fn active_account_admins_len(org: u8, institution: T::AccountId) -> Option<u32> {
            let account =
                Self::admin_account_with_status(org, institution, AdminAccountStatus::Active)?;
            Some(account.admins.len() as u32)
        }

        /// 查询 Pending 账户是否存在。仅用于创建/激活该账户时判断账户合法性。
        pub fn pending_account_exists_for_snapshot(org: u8, institution: T::AccountId) -> bool {
            Self::admin_account_with_status(org, institution, AdminAccountStatus::Pending).is_some()
        }

        /// 查询 Pending 账户管理员权限。仅用于创建/激活该账户时锁定投票快照。
        pub fn is_pending_account_admin_for_snapshot(
            org: u8,
            institution: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            let Some(account) =
                Self::admin_account_with_status(org, institution, AdminAccountStatus::Pending)
            else {
                return false;
            };
            account.admins.iter().any(|admin| admin == who)
        }

        /// 读取 Pending 账户管理员列表。仅供投票引擎 Pending 创建入口写快照。
        pub fn pending_account_admins_for_snapshot(
            org: u8,
            institution: T::AccountId,
        ) -> Option<Vec<T::AccountId>> {
            let account =
                Self::admin_account_with_status(org, institution, AdminAccountStatus::Pending)?;
            Some(account.admins.into_inner())
        }

        /// 读取 Pending 账户管理员数量。仅用于创建/激活该账户的快照语义。
        pub fn pending_account_admins_len_for_snapshot(
            org: u8,
            institution: T::AccountId,
        ) -> Option<u32> {
            let account =
                Self::admin_account_with_status(org, institution, AdminAccountStatus::Pending)?;
            Some(account.admins.len() as u32)
        }

        pub(crate) fn try_execute_set_change_from_action(
            proposal_id: u64,
            action: AdminSetChangeAction<T::AccountId, AdminsOf<T>>,
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

            let account = AdminAccounts::<T>::get(action.account.clone())
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                account.status == AdminAccountStatus::Active,
                Error::<T>::AdminAccountNotActive
            );
            ensure!(
                proposal.internal_institution == Some(action.account.clone()),
                Error::<T>::ProposalInstitutionMismatch
            );
            ensure!(
                proposal.internal_org == Some(account.org),
                Error::<T>::ProposalOrgMismatch
            );
            votingengine::Pallet::<T>::ensure_admin_set_mutation_lock_owner(
                account.org,
                action.account.clone(),
                proposal_id,
            )?;
            let current_admins = account.admins.clone().into_inner();
            Self::validate_admin_set_for_account(
                account.kind,
                account.org,
                action.admins.as_slice(),
            )?;
            ensure!(
                !Self::same_admin_set(current_admins.as_slice(), action.admins.as_slice()),
                Error::<T>::AdminSetUnchanged
            );
            AdminAccounts::<T>::mutate(action.account.clone(), |maybe| {
                if let Some(account) = maybe {
                    account.admins = action.admins.clone();
                    account.updated_at = frame_system::Pallet::<T>::block_number();
                }
            });

            Self::deposit_event(Event::<T>::AdminSetChanged {
                proposal_id,
                account: action.account,
                admins_len: action.admins.len() as u32,
                threshold: action.new_threshold,
            });

            Ok(())
        }
    }
}

impl<T: pallet::Config> AdminAccountLifecycle<T::AccountId> for pallet::Pallet<T> {
    fn create_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: T::AccountId,
        org: u8,
        kind: AdminAccountKind,
        admins: Vec<T::AccountId>,
        creator: T::AccountId,
    ) -> DispatchResult {
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            institution.clone(),
            org,
            STATUS_VOTING,
            false,
        )?;
        Self::do_create_pending_admin_account(institution, org, kind, admins, creator)
    }

    fn activate_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: T::AccountId,
    ) -> DispatchResult {
        let account = pallet::AdminAccounts::<T>::get(institution.clone())
            .ok_or(pallet::Error::<T>::InvalidInstitution)?;
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            institution.clone(),
            account.org,
            STATUS_PASSED,
            true,
        )?;
        Self::do_activate_admin_account(institution)
    }

    fn remove_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: T::AccountId,
    ) -> DispatchResult {
        let account = pallet::AdminAccounts::<T>::get(institution.clone())
            .ok_or(pallet::Error::<T>::InvalidInstitution)?;
        let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
            .ok_or(pallet::Error::<T>::InvalidAdminAccountLifecycleScope)?;
        ensure!(
            matches!(proposal.status, STATUS_REJECTED | STATUS_EXECUTION_FAILED),
            pallet::Error::<T>::InvalidAdminAccountLifecycleScope
        );
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            institution.clone(),
            account.org,
            proposal.status,
            false,
        )?;
        Self::do_remove_pending_admin_account(institution)
    }

    fn close_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: T::AccountId,
    ) -> DispatchResult {
        let account = pallet::AdminAccounts::<T>::get(institution.clone())
            .ok_or(pallet::Error::<T>::InvalidInstitution)?;
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            institution.clone(),
            account.org,
            STATUS_PASSED,
            true,
        )?;
        Self::do_close_admin_account(institution)
    }
}

// ──── 投票终态回调:把已通过的管理员集合变更提案落地到链上 ────
//
// 投票统一由投票引擎承担,提案通过(或否决)经
// [`votingengine::InternalVoteResultCallback`] 广播回来。
// 本 Executor 按 `ProposalOwner` 认领本模块提案，`ProposalData` 只保存裸业务 action。
//
// 设计要点:
// - `approved = true` 时执行 `try_execute_set_change`,失败发 `AdminSetChangeExecutionFailed`
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
        let action = AdminSetChangeAction::<T::AccountId, AdminsOf<T>>::decode(&mut &raw[..])
            .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;

        // Step 3:执行替换。管理员集合变更失败属于数据/状态已不匹配，直接交给投票引擎失败终态。
        match pallet::Pallet::<T>::try_execute_set_change_from_action(proposal_id, action) {
            Ok(()) => Ok(ProposalExecutionOutcome::Executed),
            Err(_) => {
                pallet::Pallet::<T>::deposit_event(
                    pallet::Event::<T>::AdminSetChangeExecutionFailed { proposal_id },
                );
                Ok(ProposalExecutionOutcome::FatalFailed)
            }
        }
    }
}

#[cfg(test)]
mod tests;
