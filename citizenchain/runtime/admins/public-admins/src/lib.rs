#![cfg_attr(not(feature = "std"), no_std)]
//! 管理员权限治理模块（public-admins）
//! - 本模块只负责“管理员集合变更”这一类业务事项
//! - 投票流程本身由 votingengine 提供（内部投票）
//! - 约束：治理机构固定人数，仅允许等长更换；动态账户允许增删改。
//!   阈值校验、保存和更新统一由 votingengine/internal-vote 负责。

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::{
    ensure,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::StorageVersion,
    Blake2_128Concat,
};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchError;
use sp_std::collections::btree_set::BTreeSet;

use admin_primitives::{
    can_store_public_admin_code, AdminAccount, AdminAccountKind, AdminAccountLifecycle,
    AdminAccountStatus, AdminProfile, AdminSetChangeAction,
};
use entity_primitives::InstitutionMultisigQuery;
use primitives::cid::{
    china::china_zf::CHINA_ZF,
    code::PROVINCE_CODE_INFOS,
    code::{institution_code_from_cid_number, ProvinceCode, FRG},
};
use votingengine::{
    types::InstitutionCode, InternalVoteResultCallback, ProposalExecutionOutcome, ProposalSubject,
    PROPOSAL_KIND_INTERNAL, STAGE_INTERNAL, STATUS_EXECUTION_FAILED, STATUS_PASSED,
    STATUS_REJECTED, STATUS_VOTING,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
/// tag 带 schema 版本号。
pub const MODULE_TAG: &[u8] = b"pub-adm1";

/// public-admins pallet on-chain storage 版本。
/// 全新创世口径:创世即终态布局,storage 版本恒为 v1,不承载任何历史迁移。
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);
const FEDERAL_REGISTRY_PROVINCE_GROUP_SIZE: usize =
    primitives::count_const::FRG_PROVINCE_GROUP_ADMIN_COUNT as usize;
const FEDERAL_REGISTRY_PROVINCE_GROUP_THRESHOLD: u32 =
    primitives::count_const::FRG_INTERNAL_THRESHOLD;
const FEDERAL_REGISTRY_PROVINCE_ACCOUNT_PREFIX: &[u8] = b"GMB:FRG-PROVINCE:";

fn decode_account<T: frame_system::Config>(raw: &[u8; 32]) -> Option<T::AccountId> {
    T::AccountId::decode(&mut &raw[..]).ok()
}

/// 联邦注册局省级治理组虚拟账户。
///
/// 该账户不是机构资金账户,只作为投票引擎的内部投票根账户使用。
/// 同一省 5 名 FRG 管理员围绕此账户创建管理员更换提案,代码级固定阈值 3/5。
fn federal_registry_province_group_account<T: frame_system::Config>(
    province_code: ProvinceCode,
) -> Option<T::AccountId> {
    let mut payload = Vec::with_capacity(
        FEDERAL_REGISTRY_PROVINCE_ACCOUNT_PREFIX
            .len()
            .saturating_add(province_code.len()),
    );
    payload.extend_from_slice(FEDERAL_REGISTRY_PROVINCE_ACCOUNT_PREFIX);
    payload.extend_from_slice(&province_code);
    let raw = sp_io::hashing::blake2_256(&payload);
    decode_account::<T>(&raw)
}

/// 联邦注册局机构主账户(创世内置:`CHINA_ZF` 中 FRG 节点的 `main_account`)。
fn federal_registry_account<T: frame_system::Config>() -> Option<T::AccountId> {
    CHINA_ZF.iter().find_map(
        |node| match institution_code_from_cid_number(node.cid_number) {
            Some(code) if code == FRG => decode_account::<T>(&node.main_account),
            _ => None,
        },
    )
}

fn federal_registry_cid() -> Option<Vec<u8>> {
    CHINA_ZF
        .iter()
        .find(|node| institution_code_from_cid_number(node.cid_number) == Some(FRG))
        .map(|node| node.cid_number.as_bytes().to_vec())
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
        /// 内部投票引擎（返回真实 proposal_id，避免外部猜测 next_proposal_id）。
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        /// 机构账户 → CID 查询入口。机构管理员变更提案必须以 CID 为主体真源。
        type InstitutionQuery: InstitutionMultisigQuery<Self::AccountId>;

        /// 该 pallet 的可配置权重实现。
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 仅账户的管理员集合(用于纯账户语义的边界 helper)。
    pub type AdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAdminsPerInstitution>;

    /// 管理员资料集合(链上真存储:每个管理员一条 `AdminProfile`)。
    pub type AdminProfilesOf<T> = BoundedVec<
        AdminProfile<<T as frame_system::Config>::AccountId>,
        <T as Config>::MaxAdminsPerInstitution,
    >;

    pub type AdminAccountOf<T> =
        AdminAccount<AdminProfilesOf<T>, <T as frame_system::Config>::AccountId, BlockNumberFor<T>>;

    /// 公权机构管理员表：保存所有公权机构管理员集合。
    ///
    /// 创世来源只影响初始写入位置,运行期管理员治理统一归本模块。
    #[pallet::storage]
    #[pallet::getter(fn admin_account_of)]
    pub type AdminAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AdminAccountOf<T>, OptionQuery>;

    /// 联邦注册局省级管理员组：province_code -> 5 人管理员集合。
    ///
    /// FRG 总计 215 名管理员按 43 省拆成 43 个 5 人组。
    /// 每个省级组单独作为内部投票根账户,换本省管理员只由本省 5 人组 3/5 投票。
    #[pallet::storage]
    pub type FederalRegistryProvinceGroups<T: Config> =
        StorageMap<_, Blake2_128Concat, ProvinceCode, AdminAccountOf<T>, OptionQuery>;

    /// 联邦注册局省级管理员组账户反向索引：group_account -> province_code。
    #[pallet::storage]
    pub type FederalRegistryProvinceGroupAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ProvinceCode, OptionQuery>;

    /// 机构法定代表人(机构首脑;ADR-027 立法签署人)。键 = 机构账户,值 = 法定代表人账户。
    ///
    /// 必为该机构 Active admins 之一(写入时校验)。未显式设置时,
    /// `legal_representative()` 回退到 admins[0](创世首位管理员=机构首脑占位),
    /// 由治理(public-admins)显式指定后覆盖。仅治理/签署语境读取。
    #[pallet::storage]
    pub type LegalRepresentatives<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, OptionQuery>;

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
            assert!(
                <T as Config>::MaxAdminsPerInstitution::get() >= 2,
                "MaxAdminsPerInstitution must be >= 2"
            );
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {}
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 已发起管理员集合变更提案（并已在投票引擎创建内部提案）
        AdminSetChangeProposed {
            proposal_id: u64,
            institution_code: InstitutionCode,
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
            institution_code: InstitutionCode,
            kind: AdminAccountKind,
            creator: T::AccountId,
            admins_len: u32,
        },
        /// 多签账户管理员配置已激活。
        AdminAccountActivated {
            account: T::AccountId,
            institution_code: InstitutionCode,
        },
        /// Pending 多签账户管理员配置已清理。
        AdminAccountPendingRemoved {
            account: T::AccountId,
            institution_code: InstitutionCode,
        },
        /// 多签账户管理员配置已关闭。
        AdminAccountClosed {
            account: T::AccountId,
            institution_code: InstitutionCode,
        },
        /// 注册局直设机构管理员(绕过内部投票,原子写 Active + 注册动态阈值)。
        AdminAccountRegistryDirectSet {
            account: T::AccountId,
            institution_code: InstitutionCode,
            creator: T::AccountId,
            admins_len: u32,
            threshold: u32,
            created: bool,
        },
        /// 联邦注册局省级管理员组更换提案已发起。
        FederalRegistryProvinceAdminSetChangeProposed {
            proposal_id: u64,
            province_code: ProvinceCode,
            account: T::AccountId,
            proposer: T::AccountId,
            old_admins_len: u32,
            new_admins_len: u32,
            new_threshold: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 无效机构
        InvalidInstitution,
        /// 机构类型与 institution_code 参数不匹配
        InstitutionCodeMismatch,
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
        ProposalCodeMismatch,
        /// 管理员账户已存在
        InstitutionAlreadyExists,
        /// 管理员账户状态不是 Pending
        AdminAccountNotPending,
        /// 管理员账户状态不是 Active
        AdminAccountNotActive,
        /// 内置治理机构永远不可关闭
        BuiltinAdminAccountCannotClose,
        /// 管理员账户类型与 institution_code 不匹配
        InvalidAdminAccountKind,
        /// 阈值不合法
        InvalidThreshold,
        /// 管理员重复
        DuplicateAdmin,
        /// 管理员账户生命周期写入缺少有效 votingengine 提案作用域
        InvalidAdminAccountLifecycleScope,
        /// 联邦注册局管理员更换必须走省级 5 人组治理入口
        FederalRegistryRequiresProvinceGroup,
        /// 省级代码不存在或没有对应联邦注册局管理员组
        InvalidProvinceGroup,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_admin_set_change())]
        pub fn propose_admin_set_change(
            origin: OriginFor<T>,
            institution_code: InstitutionCode,
            account: T::AccountId,
            admins: AdminProfilesOf<T>,
            new_threshold: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                institution_code != FRG,
                Error::<T>::FederalRegistryRequiresProvinceGroup
            );

            // 1) 校验管理员账户已激活且机构码匹配。
            let current =
                AdminAccounts::<T>::get(account.clone()).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                current.status == AdminAccountStatus::Active,
                Error::<T>::AdminAccountNotActive
            );
            ensure!(
                current.institution_code == institution_code,
                Error::<T>::InstitutionCodeMismatch
            );

            // 2) 校验发起人与目标管理员集合合法性(账户语义校验取 profile.account)。
            let current_admins: Vec<T::AccountId> =
                current.admins.iter().map(|p| p.account.clone()).collect();
            let new_admins: Vec<T::AccountId> = admins.iter().map(|p| p.account.clone()).collect();
            ensure!(current_admins.contains(&who), Error::<T>::UnauthorizedAdmin);
            Self::validate_admin_set_for_account(
                current.kind,
                current.institution_code,
                new_admins.as_slice(),
            )?;
            ensure!(
                !Self::same_admin_set(current_admins.as_slice(), new_admins.as_slice()),
                Error::<T>::AdminSetUnchanged
            );
            let subject_cid_numbers =
                alloc::vec![T::InstitutionQuery::lookup_cid(&account)
                    .ok_or(Error::<T>::InvalidInstitution)?];
            // 3) 在同一个链上事务中创建投票提案、互斥锁和业务数据。
            with_transaction(|| {
                let action = AdminSetChangeAction {
                    admin_root_account_id: account.clone(),
                    admins: admins.clone(),
                    new_threshold,
                };
                let encoded = action.encode();
                let proposal_id =
                    match T::InternalVoteEngine::create_admin_change_internal_proposal_with_data(
                        who.clone(),
                        institution_code,
                        account.clone(),
                        subject_cid_numbers,
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
                    institution_code,
                    account,
                    proposer: who,
                    old_admins_len: current_admins.len() as u32,
                    new_admins_len: new_admins.len() as u32,
                    new_threshold,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }

        /// 联邦注册局省级管理员组更换提案。
        ///
        /// FRG 管理员按省分成 43 个 5 人组。本入口只允许
        /// 本省组内管理员发起本省组的管理员更换,投票引擎快照为该省 5 人组,
        /// 阈值固定严格过半 3/5,不会再让全联邦注册局 215 人一起投票。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_admin_set_change())]
        pub fn propose_federal_registry_province_admin_set_change(
            origin: OriginFor<T>,
            province_code: ProvinceCode,
            admins: AdminProfilesOf<T>,
            new_threshold: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let group_account = federal_registry_province_group_account::<T>(province_code)
                .ok_or(Error::<T>::InvalidProvinceGroup)?;
            let current = FederalRegistryProvinceGroups::<T>::get(province_code)
                .ok_or(Error::<T>::InvalidProvinceGroup)?;
            ensure!(
                current.status == AdminAccountStatus::Active,
                Error::<T>::AdminAccountNotActive
            );
            ensure!(
                current.institution_code == FRG,
                Error::<T>::InstitutionCodeMismatch
            );
            ensure!(
                FederalRegistryProvinceGroupAccounts::<T>::get(group_account.clone())
                    == Some(province_code),
                Error::<T>::InvalidProvinceGroup
            );

            let current_admins: Vec<T::AccountId> =
                current.admins.iter().map(|p| p.account.clone()).collect();
            let new_admins: Vec<T::AccountId> = admins.iter().map(|p| p.account.clone()).collect();
            ensure!(current_admins.contains(&who), Error::<T>::UnauthorizedAdmin);
            Self::validate_federal_registry_province_admin_set(
                new_admins.as_slice(),
                new_threshold,
            )?;
            ensure!(
                !Self::same_admin_set(current_admins.as_slice(), new_admins.as_slice()),
                Error::<T>::AdminSetUnchanged
            );
            let subject_cid_numbers =
                alloc::vec![federal_registry_cid().ok_or(Error::<T>::InvalidInstitution)?];

            with_transaction(|| {
                let action = AdminSetChangeAction {
                    admin_root_account_id: group_account.clone(),
                    admins: admins.clone(),
                    new_threshold,
                };
                let encoded = action.encode();
                let proposal_id =
                    match T::InternalVoteEngine::create_admin_change_internal_proposal_with_data(
                        who.clone(),
                        FRG,
                        group_account.clone(),
                        subject_cid_numbers,
                        admins.len() as u32,
                        new_threshold,
                        crate::MODULE_TAG,
                        encoded,
                    ) {
                        Ok(proposal_id) => proposal_id,
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    };

                Self::deposit_event(Event::<T>::FederalRegistryProvinceAdminSetChangeProposed {
                    proposal_id,
                    province_code,
                    account: group_account,
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
        fn validate_admins_len_for_account(
            kind: AdminAccountKind,
            institution_code: InstitutionCode,
            admins_len: usize,
        ) -> DispatchResult {
            ensure!(
                kind == AdminAccountKind::PublicInstitution,
                Error::<T>::InvalidAdminAccountKind
            );
            match admin_primitives::expected_fixed_governance_admins_len(institution_code) {
                Some(expected) => {
                    ensure!(
                        admins_len == expected as usize,
                        Error::<T>::InvalidAdminsLen
                    )
                }
                None => {
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
            institution_code: InstitutionCode,
            admins: &[T::AccountId],
        ) -> DispatchResult {
            Self::ensure_account_kind_matches_org(kind, institution_code)?;
            ensure!(
                institution_code != FRG,
                Error::<T>::FederalRegistryRequiresProvinceGroup
            );
            Self::validate_admins_len_for_account(kind, institution_code, admins.len())?;
            Self::ensure_unique_admins(admins)?;
            Ok(())
        }

        fn validate_federal_registry_province_admin_set(
            admins: &[T::AccountId],
            new_threshold: u32,
        ) -> DispatchResult {
            // FRG 省级组是制度固定的 5 人治理单元；
            // 阈值来自代码级固定阈值 FRG=3，不会扩大到 215 人全局投票。
            ensure!(
                admins.len() == FEDERAL_REGISTRY_PROVINCE_GROUP_SIZE,
                Error::<T>::InvalidAdminsLen
            );
            ensure!(
                new_threshold == FEDERAL_REGISTRY_PROVINCE_GROUP_THRESHOLD,
                Error::<T>::InvalidThreshold
            );
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

        fn institution_subject_from_cid(
            cid_number: Vec<u8>,
        ) -> Result<ProposalSubject<T::AccountId>, DispatchError> {
            let mut bounded =
                votingengine::Pallet::<T>::bound_subject_cid_numbers(alloc::vec![cid_number])?;
            let cid_number = bounded
                .pop()
                .ok_or(votingengine::Error::<T>::InvalidInstitution)?;
            Ok(ProposalSubject::InstitutionCid(cid_number))
        }

        fn ensure_account_kind_matches_org(
            kind: AdminAccountKind,
            institution_code: InstitutionCode,
        ) -> DispatchResult {
            ensure!(
                kind == AdminAccountKind::PublicInstitution
                    && can_store_public_admin_code(&institution_code),
                Error::<T>::InvalidAdminAccountKind
            );
            Ok(())
        }

        fn aggregate_federal_registry_admin_account() -> Option<AdminAccountOf<T>> {
            let mut admins: Vec<AdminProfile<T::AccountId>> = Vec::new();
            for province in PROVINCE_CODE_INFOS.iter() {
                let group = FederalRegistryProvinceGroups::<T>::get(province.province_code)?;
                if group.status != AdminAccountStatus::Active || group.institution_code != FRG {
                    return None;
                }
                admins.extend(group.admins.into_iter());
            }
            let bounded: AdminProfilesOf<T> = admins.try_into().ok()?;
            let creator = bounded.first()?.account.clone();
            Some(AdminAccount {
                institution_code: FRG,
                kind: AdminAccountKind::PublicInstitution,
                admins: bounded,
                creator,
                created_at: BlockNumberFor::<T>::default(),
                updated_at: BlockNumberFor::<T>::default(),
                status: AdminAccountStatus::Active,
            })
        }

        fn account_for_mutation(
            account: T::AccountId,
        ) -> Option<(AdminAccountOf<T>, Option<ProvinceCode>)> {
            if let Some(province_code) = FederalRegistryProvinceGroupAccounts::<T>::get(&account) {
                let group = FederalRegistryProvinceGroups::<T>::get(province_code)?;
                return Some((group, Some(province_code)));
            }
            AdminAccounts::<T>::get(account).map(|admin_account| (admin_account, None))
        }

        pub(crate) fn ensure_lifecycle_proposal(
            proposal_id: u64,
            module_tag: &[u8],
            institution: T::AccountId,
            institution_code: InstitutionCode,
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
                proposal.account_context == Some(institution),
                Error::<T>::ProposalInstitutionMismatch
            );
            ensure!(
                proposal.internal_code == Some(institution_code),
                Error::<T>::ProposalCodeMismatch
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
        /// 生命周期写入只能经 `AdminAccountLifecycle` trait 做提案上下文校验后进入。
        pub(crate) fn do_create_pending_admin_account(
            institution: T::AccountId,
            institution_code: InstitutionCode,
            kind: AdminAccountKind,
            admins: Vec<AdminProfile<T::AccountId>>,
            creator: T::AccountId,
        ) -> DispatchResult {
            ensure!(
                !AdminAccounts::<T>::contains_key(institution.clone()),
                Error::<T>::InstitutionAlreadyExists
            );
            let admin_accounts: Vec<T::AccountId> =
                admins.iter().map(|p| p.account.clone()).collect();
            Self::validate_admin_set_for_account(kind, institution_code, &admin_accounts)?;

            let bounded: AdminProfilesOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            let now = frame_system::Pallet::<T>::block_number();
            let admins_len = bounded.len() as u32;
            AdminAccounts::<T>::insert(
                institution.clone(),
                AdminAccount {
                    institution_code,
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
                institution_code,
                kind,
                creator,
                admins_len,
            });
            Ok(())
        }

        /// 将 Pending 管理员账户激活。
        pub(crate) fn do_activate_admin_account(institution: T::AccountId) -> DispatchResult {
            let institution_code = AdminAccounts::<T>::try_mutate(
                institution.clone(),
                |maybe| -> Result<InstitutionCode, DispatchError> {
                    let account = maybe.as_mut().ok_or(Error::<T>::InvalidInstitution)?;
                    ensure!(
                        account.status == AdminAccountStatus::Pending,
                        Error::<T>::AdminAccountNotPending
                    );
                    account.status = AdminAccountStatus::Active;
                    account.updated_at = frame_system::Pallet::<T>::block_number();
                    Ok(account.institution_code)
                },
            )?;
            Self::deposit_event(Event::<T>::AdminAccountActivated {
                account: institution,
                institution_code,
            });
            Ok(())
        }

        /// 清理尚未激活的 Pending 管理员账户。
        pub(crate) fn do_remove_pending_admin_account(institution: T::AccountId) -> DispatchResult {
            // Pending 清理必须命中真实账户，避免不存在账户被静默当作清理成功。
            let account = AdminAccounts::<T>::get(institution.clone())
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                account.status == AdminAccountStatus::Pending,
                Error::<T>::AdminAccountNotPending
            );
            let institution_code = account.institution_code;
            AdminAccounts::<T>::remove(institution.clone());
            Self::deposit_event(Event::<T>::AdminAccountPendingRemoved {
                account: institution,
                institution_code,
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
            let institution_code = account.institution_code;
            // 动态多签注销完成后不保留 Closed 当前状态墓碑；
            // 同名确定性地址可在资金清空后重新走全新的注册流程。
            AdminAccounts::<T>::remove(institution.clone());
            Self::deposit_event(Event::<T>::AdminAccountClosed {
                account: institution,
                institution_code,
            });
            Ok(())
        }

        /// 注册局直设:原子写 Active 管理员账户(创建或更新)+ 注册动态阈值。
        ///
        /// 绕过内部投票,但不绕过单源——管理员落 `AdminAccounts`、阈值落
        /// votingengine 动态阈值,二者在同一链上事务内原子提交。上层授权由
        /// public/private-manage 的注册局权限校验承担,本函数只做机构边界校验。
        pub(crate) fn do_set_active_admin_account_direct(
            institution: T::AccountId,
            institution_code: InstitutionCode,
            kind: AdminAccountKind,
            admins: Vec<AdminProfile<T::AccountId>>,
            threshold: u32,
            creator: T::AccountId,
        ) -> DispatchResult {
            // 1) 机构类型 / 管理员集合(人数、唯一、类型)边界校验(账户语义取 profile.account)。
            let admin_accounts: Vec<T::AccountId> =
                admins.iter().map(|p| p.account.clone()).collect();
            Self::validate_admin_set_for_account(kind, institution_code, &admin_accounts)?;
            let bounded: AdminProfilesOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            let admins_len = bounded.len() as u32;
            let now = frame_system::Pallet::<T>::block_number();

            with_transaction(|| {
                // 2) 先注册动态阈值(内部按严格过半校验);失败整体回滚。
                if let Err(err) = T::InternalVoteEngine::register_active_dynamic_threshold_direct(
                    institution_code,
                    institution.clone(),
                    admins_len,
                    threshold,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }

                // 3) 原子写 Active 账户:不存在则创建,存在则更新 admins 并强制 Active。
                let created = match AdminAccounts::<T>::get(institution.clone()) {
                    Some(existing) => {
                        if existing.institution_code != institution_code {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::InstitutionCodeMismatch.into(),
                            ));
                        }
                        if existing.kind != kind {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::InvalidAdminAccountKind.into(),
                            ));
                        }
                        AdminAccounts::<T>::mutate(institution.clone(), |maybe| {
                            if let Some(account) = maybe {
                                account.admins = bounded.clone();
                                account.status = AdminAccountStatus::Active;
                                account.updated_at = now;
                            }
                        });
                        false
                    }
                    None => {
                        AdminAccounts::<T>::insert(
                            institution.clone(),
                            AdminAccount {
                                institution_code,
                                kind,
                                admins: bounded.clone(),
                                creator: creator.clone(),
                                created_at: now,
                                updated_at: now,
                                status: AdminAccountStatus::Active,
                            },
                        );
                        true
                    }
                };

                Self::deposit_event(Event::<T>::AdminAccountRegistryDirectSet {
                    account: institution.clone(),
                    institution_code,
                    creator: creator.clone(),
                    admins_len,
                    threshold,
                    created,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }

        fn admin_account_with_status(
            institution_code: InstitutionCode,
            institution: T::AccountId,
            status: AdminAccountStatus,
        ) -> Option<AdminAccountOf<T>> {
            let account = AdminAccounts::<T>::get(institution.clone()).or_else(|| {
                if institution_code != FRG {
                    return None;
                }
                if let Some(province_code) =
                    FederalRegistryProvinceGroupAccounts::<T>::get(&institution)
                {
                    return FederalRegistryProvinceGroups::<T>::get(province_code);
                }
                if federal_registry_account::<T>() == Some(institution) {
                    return Self::aggregate_federal_registry_admin_account();
                }
                None
            })?;
            if account.institution_code != institution_code || account.status != status {
                return None;
            }
            // 读侧也要执行账户类型边界校验，避免升级前写入的旧脏数据
            // 继续通过 active/pending 查询 API 被其他业务模块当作有效管理员账户。
            if Self::ensure_account_kind_matches_org(account.kind, account.institution_code)
                .is_err()
            {
                return None;
            }
            Some(account)
        }

        /// 查询 Active 账户是否存在。普通业务账户合法性判断只使用 Active 账户。
        pub fn active_admin_account_exists(
            institution_code: InstitutionCode,
            institution: T::AccountId,
        ) -> bool {
            Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Active,
            )
            .is_some()
        }

        /// 查询 Active 账户管理员权限。普通业务授权只能使用 Active 账户。
        pub fn is_active_account_admin(
            institution_code: InstitutionCode,
            institution: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            let Some(account) = Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Active,
            ) else {
                return false;
            };
            account.admins.iter().any(|admin| &admin.account == who)
        }

        /// 读取 Active 账户管理员账户列表(投票/多签资格语义,取 profile.account)。
        ///
        /// 普通业务提案创建和投票快照默认使用此 API;返回的是账户而非资料,
        /// 内部投票一人一票、多签转账、组织管理查配置全部零改动。
        pub fn active_account_admins(
            institution_code: InstitutionCode,
            institution: T::AccountId,
        ) -> Option<Vec<T::AccountId>> {
            let account = Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Active,
            )?;
            Some(account.admins.iter().map(|p| p.account.clone()).collect())
        }

        /// 读取 Active 账户管理员完整资料列表(展示路径,含姓名/职务/任期/实名 CID)。
        pub fn active_account_admin_profiles(
            institution_code: InstitutionCode,
            institution: T::AccountId,
        ) -> Option<Vec<AdminProfile<T::AccountId>>> {
            let account = Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Active,
            )?;
            Some(account.admins.into_inner())
        }

        /// 读取机构法定代表人(机构首脑;ADR-027 立法签署人)。
        ///
        /// 优先取显式指定的法定代表人(校验仍为 Active admins 之一);
        /// 未指定则回退到 admins[0](创世首位=机构首脑占位)。Active 账户专用。
        pub fn legal_representative(
            institution_code: InstitutionCode,
            institution: T::AccountId,
        ) -> Option<T::AccountId> {
            let admins = Self::active_account_admins(institution_code, institution.clone())?;
            match LegalRepresentatives::<T>::get(&institution) {
                // 显式指定且仍在现任 admins 内 → 采用;否则回退首位(换届后旧代表人失效)。
                Some(rep) if admins.iter().any(|a| a == &rep) => Some(rep),
                _ => admins.into_iter().next(),
            }
        }

        /// 设置机构法定代表人(治理通过后写入;校验必为 Active admins 之一)。
        pub fn set_legal_representative(
            institution_code: InstitutionCode,
            institution: T::AccountId,
            representative: T::AccountId,
        ) -> DispatchResult {
            let admins = Self::active_account_admins(institution_code, institution.clone())
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                admins.iter().any(|a| a == &representative),
                Error::<T>::UnauthorizedAdmin
            );
            LegalRepresentatives::<T>::insert(&institution, &representative);
            Ok(())
        }

        /// 读取 Active 账户管理员数量。普通业务阈值兜底判断只能使用 Active 账户。
        pub fn active_account_admins_len(
            institution_code: InstitutionCode,
            institution: T::AccountId,
        ) -> Option<u32> {
            let account = Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Active,
            )?;
            Some(account.admins.len() as u32)
        }

        /// 查询 Pending 账户是否存在。仅用于创建/激活该账户时判断账户合法性。
        pub fn pending_account_exists_for_snapshot(
            institution_code: InstitutionCode,
            institution: T::AccountId,
        ) -> bool {
            Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Pending,
            )
            .is_some()
        }

        /// 查询 Pending 账户管理员权限。仅用于创建/激活该账户时锁定投票快照。
        pub fn is_pending_account_admin_for_snapshot(
            institution_code: InstitutionCode,
            institution: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            let Some(account) = Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Pending,
            ) else {
                return false;
            };
            account.admins.iter().any(|admin| &admin.account == who)
        }

        /// 读取 Pending 账户管理员账户列表(取 profile.account)。仅供投票引擎 Pending 创建入口写快照。
        pub fn pending_account_admins_for_snapshot(
            institution_code: InstitutionCode,
            institution: T::AccountId,
        ) -> Option<Vec<T::AccountId>> {
            let account = Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Pending,
            )?;
            Some(account.admins.iter().map(|p| p.account.clone()).collect())
        }

        /// 读取 Pending 账户管理员数量。仅用于创建/激活该账户的快照语义。
        pub fn pending_account_admins_len_for_snapshot(
            institution_code: InstitutionCode,
            institution: T::AccountId,
        ) -> Option<u32> {
            let account = Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Pending,
            )?;
            Some(account.admins.len() as u32)
        }

        pub(crate) fn try_execute_set_change_from_action(
            proposal_id: u64,
            action: AdminSetChangeAction<T::AccountId, AdminProfilesOf<T>>,
        ) -> DispatchResult {
            // 执行前同时校验投票引擎元数据与业务 action，避免跨模块误消费。
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

            let (account, province_group) =
                Self::account_for_mutation(action.admin_root_account_id.clone())
                    .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                account.status == AdminAccountStatus::Active,
                Error::<T>::AdminAccountNotActive
            );
            ensure!(
                proposal.account_context == Some(action.admin_root_account_id.clone()),
                Error::<T>::ProposalInstitutionMismatch
            );
            ensure!(
                proposal.internal_code == Some(account.institution_code),
                Error::<T>::ProposalCodeMismatch
            );
            let cid_number = if province_group.is_some() {
                federal_registry_cid().ok_or(Error::<T>::InvalidInstitution)?
            } else {
                T::InstitutionQuery::lookup_cid(&action.admin_root_account_id)
                    .ok_or(Error::<T>::InvalidInstitution)?
            };
            let subject = Self::institution_subject_from_cid(cid_number)?;
            votingengine::Pallet::<T>::ensure_admin_set_mutation_lock_owner(subject, proposal_id)?;
            let current_admins: Vec<T::AccountId> =
                account.admins.iter().map(|p| p.account.clone()).collect();
            let new_admins: Vec<T::AccountId> =
                action.admins.iter().map(|p| p.account.clone()).collect();
            if province_group.is_some() {
                Self::validate_federal_registry_province_admin_set(
                    new_admins.as_slice(),
                    action.new_threshold,
                )?;
            } else {
                Self::validate_admin_set_for_account(
                    account.kind,
                    account.institution_code,
                    new_admins.as_slice(),
                )?;
            }
            ensure!(
                !Self::same_admin_set(current_admins.as_slice(), new_admins.as_slice()),
                Error::<T>::AdminSetUnchanged
            );
            if let Some(province_code) = province_group {
                FederalRegistryProvinceGroups::<T>::mutate(province_code, |maybe| {
                    if let Some(account) = maybe {
                        account.admins = action.admins.clone();
                        account.updated_at = frame_system::Pallet::<T>::block_number();
                    }
                });
            } else {
                AdminAccounts::<T>::mutate(action.admin_root_account_id.clone(), |maybe| {
                    if let Some(account) = maybe {
                        account.admins = action.admins.clone();
                        account.updated_at = frame_system::Pallet::<T>::block_number();
                    }
                });
            }

            Self::deposit_event(Event::<T>::AdminSetChanged {
                proposal_id,
                account: action.admin_root_account_id,
                admins_len: action.admins.len() as u32,
                threshold: action.new_threshold,
            });

            Ok(())
        }
    }
}

impl<T: pallet::Config> AdminAccountLifecycle<T::AccountId, AdminProfile<T::AccountId>>
    for pallet::Pallet<T>
{
    fn create_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        institution: T::AccountId,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<AdminProfile<T::AccountId>>,
        creator: T::AccountId,
    ) -> DispatchResult {
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            institution.clone(),
            institution_code,
            STATUS_VOTING,
            false,
        )?;
        Self::do_create_pending_admin_account(institution, institution_code, kind, admins, creator)
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
            account.institution_code,
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
            account.institution_code,
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
            account.institution_code,
            STATUS_PASSED,
            true,
        )?;
        Self::do_close_admin_account(institution)
    }

    fn set_active_admin_account_direct(
        _module_tag: &[u8],
        admin_root_account_id: T::AccountId,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<AdminProfile<T::AccountId>>,
        threshold: u32,
        creator: T::AccountId,
    ) -> DispatchResult {
        Self::do_set_active_admin_account_direct(
            admin_root_account_id,
            institution_code,
            kind,
            admins,
            threshold,
            creator,
        )
    }
}

impl<T: pallet::Config> admin_primitives::AdminAccountQuery<T::AccountId> for pallet::Pallet<T> {
    fn active_admin_account_exists(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> bool {
        Self::active_admin_account_exists(institution_code, admin_root_account_id)
    }

    fn is_active_account_admin(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
        who: &T::AccountId,
    ) -> bool {
        Self::is_active_account_admin(institution_code, admin_root_account_id, who)
    }

    fn active_account_admins(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<Vec<T::AccountId>> {
        Self::active_account_admins(institution_code, admin_root_account_id)
    }

    fn active_account_admin_profiles(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<Vec<AdminProfile<T::AccountId>>> {
        Self::active_account_admin_profiles(institution_code, admin_root_account_id)
    }

    fn active_account_admins_len(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<u32> {
        Self::active_account_admins_len(institution_code, admin_root_account_id)
    }

    fn pending_account_exists_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> bool {
        Self::pending_account_exists_for_snapshot(institution_code, admin_root_account_id)
    }

    fn is_pending_account_admin_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
        who: &T::AccountId,
    ) -> bool {
        Self::is_pending_account_admin_for_snapshot(institution_code, admin_root_account_id, who)
    }

    fn pending_account_admins_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<Vec<T::AccountId>> {
        Self::pending_account_admins_for_snapshot(institution_code, admin_root_account_id)
    }

    fn pending_account_admins_len_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<u32> {
        Self::pending_account_admins_len_for_snapshot(institution_code, admin_root_account_id)
    }

    fn legal_representative(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<T::AccountId> {
        Self::legal_representative(institution_code, admin_root_account_id)
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
        let action =
            AdminSetChangeAction::<T::AccountId, pallet::AdminProfilesOf<T>>::decode(&mut &raw[..])
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
