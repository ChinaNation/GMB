#![cfg_attr(not(feature = "std"), no_std)]
//! 公权机构管理员钱包集合模块（public-admins）。
//!
//! 岗位和任职归 entity，投票流程归 votingengine；本模块只保存由有效任职派生的
//! `admins` 钱包集合，并在任职结果生效时保持既有投票阈值不变。

extern crate alloc;

use alloc::vec::Vec;
use frame_support::{
    ensure,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::StorageVersion,
    Blake2_128Concat,
};
use frame_system::pallet_prelude::*;
use sp_std::collections::btree_set::BTreeSet;

use admin_primitives::{
    can_store_public_admin_code, AdminAccountKind, AdminAccountStatus, AdminCidNumber,
    InstitutionAdminAccount, InstitutionAdminAccountLifecycle,
};
use entity_primitives::InstitutionMultisigQuery;
use votingengine::{types::InstitutionCode, PROPOSAL_KIND_INTERNAL, STAGE_INTERNAL, STATUS_PASSED};

pub use pallet::*;

/// public-admins pallet on-chain storage 版本。
/// 全新创世直接使用纯账户布局，不承载机构岗位资料或省级分组副本。
const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

#[frame_support::pallet]
pub mod pallet {
    use super::*;
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
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 仅账户的管理员集合(用于纯账户语义的边界 helper)。
    pub type AdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAdminsPerInstitution>;

    pub type AdminAccountOf<T> = InstitutionAdminAccount<AdminsOf<T>>;

    /// 公权机构管理员表：保存所有公权机构管理员集合。
    ///
    /// 创世来源只影响初始写入位置,运行期管理员治理统一归本模块。
    #[pallet::storage]
    #[pallet::getter(fn admin_account_of)]
    pub type AdminAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AdminAccountOf<T>, OptionQuery>;

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
        /// 多签账户管理员配置已关闭。
        AdminAccountClosed {
            account: T::AccountId,
            institution_code: InstitutionCode,
        },
        /// 注册局直设机构管理员(绕过内部投票,原子写 Active + 注册动态阈值)。
        AdminAccountRegistryDirectSet {
            account: T::AccountId,
            institution_code: InstitutionCode,
            admins_len: u32,
            threshold: u32,
            created: bool,
        },
        /// entity 岗位任职结果已同步到纯管理员钱包集合。
        AdminAccountsSyncedFromAssignments {
            account: T::AccountId,
            institution_code: InstitutionCode,
            admins_len: u32,
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
        /// 提案绑定机构与管理员更换动作不一致
        ProposalInstitutionMismatch,
        /// 提案绑定组织与管理员账户不一致
        ProposalCodeMismatch,
        /// 管理员账户状态不是 Active
        AdminAccountNotActive,
        /// 内置治理机构永远不可关闭
        BuiltinAdminAccountCannotClose,
        /// 管理员账户类型与 institution_code 不匹配
        InvalidAdminAccountKind,
        /// 阈值不合法
        InvalidThreshold,
        /// 动态机构缺少既有 Active 投票阈值，禁止任职结果暗中创建新制度。
        MissingDynamicThreshold,
        /// 管理员重复
        DuplicateAdmin,
        /// 管理员账户生命周期写入缺少有效 votingengine 提案作用域
        InvalidAdminAccountLifecycleScope,
    }

    impl<T: Config> Pallet<T> {
        fn validate_admins_len_for_account(
            kind: AdminAccountKind,
            institution_code: InstitutionCode,
            cid_number: &[u8],
            main_account: &[u8],
            admins_len: usize,
        ) -> DispatchResult {
            ensure!(
                kind == AdminAccountKind::PublicInstitution,
                Error::<T>::InvalidAdminAccountKind
            );
            match admin_primitives::expected_fixed_governance_admins_len(
                institution_code,
                cid_number,
                main_account,
            ) {
                Some(expected) => {
                    ensure!(
                        admins_len == expected as usize,
                        Error::<T>::InvalidAdminsLen
                    )
                }
                None => match primitives::institution_constraints::member_composition_by_identity(
                    institution_code,
                    cid_number,
                    main_account,
                ) {
                    Some(spec) => ensure!(
                        admins_len >= spec.min_members as usize
                            && admins_len <= spec.max_members as usize,
                        Error::<T>::InvalidAdminsLen
                    ),
                    None => {
                        ensure!(admins_len >= 2, Error::<T>::InvalidAdminsLen);
                        ensure!(
                            admins_len <= <T as Config>::MaxAdminsPerInstitution::get() as usize,
                            Error::<T>::InvalidAdminsLen
                        );
                    }
                },
            }
            Ok(())
        }

        pub(crate) fn validate_admin_set_for_account(
            kind: AdminAccountKind,
            institution_code: InstitutionCode,
            cid_number: &[u8],
            main_account: &[u8],
            admins: &[T::AccountId],
        ) -> DispatchResult {
            Self::ensure_account_kind_matches_org(kind, institution_code)?;
            Self::validate_admins_len_for_account(
                kind,
                institution_code,
                cid_number,
                main_account,
                admins.len(),
            )?;
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
            cid_number: Vec<u8>,
            institution_code: InstitutionCode,
            kind: AdminAccountKind,
            admins: Vec<T::AccountId>,
            threshold: u32,
        ) -> DispatchResult {
            // 1) 机构类型和管理员钱包集合边界校验。
            let cid_number: AdminCidNumber = cid_number
                .try_into()
                .map_err(|_| Error::<T>::InvalidInstitution)?;
            let main_account = institution.encode();
            Self::validate_admin_set_for_account(
                kind,
                institution_code,
                cid_number.as_slice(),
                &main_account,
                &admins,
            )?;
            let bounded: AdminsOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            let admins_len = bounded.len() as u32;

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
                        if existing.cid_number != cid_number {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::InvalidInstitution.into(),
                            ));
                        }
                        if existing.institution_code != institution_code {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::InstitutionCodeMismatch.into(),
                            ));
                        }
                        AdminAccounts::<T>::mutate(institution.clone(), |maybe| {
                            if let Some(account) = maybe {
                                account.admins = bounded.clone();
                                account.status = AdminAccountStatus::Active;
                            }
                        });
                        false
                    }
                    None => {
                        AdminAccounts::<T>::insert(
                            institution.clone(),
                            InstitutionAdminAccount {
                                cid_number: cid_number.clone(),
                                institution_code,
                                admins: bounded.clone(),
                                status: AdminAccountStatus::Active,
                            },
                        );
                        true
                    }
                };

                Self::deposit_event(Event::<T>::AdminAccountRegistryDirectSet {
                    account: institution.clone(),
                    institution_code,
                    admins_len,
                    threshold,
                    created,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }

        /// entity 任职结果生效后，同步管理员钱包并保持现有阈值制度。
        pub(crate) fn do_sync_active_admins_from_assignments(
            institution: T::AccountId,
            cid_number: Vec<u8>,
            institution_code: InstitutionCode,
            admins: Vec<T::AccountId>,
        ) -> DispatchResult {
            let cid_number: AdminCidNumber = cid_number
                .try_into()
                .map_err(|_| Error::<T>::InvalidInstitution)?;
            let main_account = institution.encode();
            Self::validate_admin_set_for_account(
                AdminAccountKind::PublicInstitution,
                institution_code,
                cid_number.as_slice(),
                &main_account,
                &admins,
            )?;
            let bounded: AdminsOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            let admins_len = bounded.len() as u32;
            let fixed_threshold =
                primitives::cid::code::fixed_governance_pass_threshold(&institution_code);
            let permanent_singleton = primitives::institution_constraints::singleton_by_identity(
                institution_code,
                cid_number.as_slice(),
                &main_account,
            )
            .is_some();
            let first_composition =
                AdminAccounts::<T>::get(institution.clone()).is_none() && permanent_singleton;
            // 固定五类机构使用代码级固定阈值；六个国家单例不保存账户级阈值，
            // 普通内部事项由 internal-vote 在创建提案时按 admins 快照计算严格过半。
            let dynamic_threshold = if fixed_threshold.is_none() && !permanent_singleton {
                Some(
                    T::InternalVoteEngine::active_dynamic_threshold(
                        institution_code,
                        institution.clone(),
                    )
                    .ok_or(Error::<T>::MissingDynamicThreshold)?,
                )
            } else {
                None
            };

            with_transaction(|| {
                let existing = AdminAccounts::<T>::get(institution.clone());
                if let Some(existing) = &existing {
                    if existing.cid_number != cid_number
                        || existing.institution_code != institution_code
                    {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::InstitutionCodeMismatch.into(),
                        ));
                    }
                    if existing.status != AdminAccountStatus::Active {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::AdminAccountNotActive.into(),
                        ));
                    }
                } else if !first_composition {
                    return TransactionOutcome::Rollback(
                        Err(Error::<T>::InvalidInstitution.into()),
                    );
                }
                if let Some(threshold) = dynamic_threshold {
                    if T::InternalVoteEngine::register_active_dynamic_threshold_direct(
                        institution_code,
                        institution.clone(),
                        admins_len,
                        threshold,
                    )
                    .is_err()
                    {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::InvalidThreshold.into()
                        ));
                    }
                }
                if existing.is_some() {
                    AdminAccounts::<T>::mutate(institution.clone(), |maybe| {
                        if let Some(account) = maybe {
                            account.admins = bounded.clone();
                        }
                    });
                } else {
                    AdminAccounts::<T>::insert(
                        institution.clone(),
                        InstitutionAdminAccount {
                            cid_number: cid_number.clone(),
                            institution_code,
                            admins: bounded.clone(),
                            status: AdminAccountStatus::Active,
                        },
                    );
                }
                Self::deposit_event(Event::<T>::AdminAccountsSyncedFromAssignments {
                    account: institution,
                    institution_code,
                    admins_len,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }

        fn admin_account_with_status(
            institution_code: InstitutionCode,
            institution: T::AccountId,
            status: AdminAccountStatus,
        ) -> Option<AdminAccountOf<T>> {
            let account = AdminAccounts::<T>::get(institution)?;
            if account.institution_code != institution_code || account.status != status {
                return None;
            }
            // 读侧也要执行账户类型边界校验，避免升级前写入的旧脏数据
            // 继续通过 active/pending 查询 API 被其他业务模块当作有效管理员账户。
            if Self::ensure_account_kind_matches_org(
                AdminAccountKind::PublicInstitution,
                account.institution_code,
            )
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
            account.admins.iter().any(|admin| admin == who)
        }

        /// 读取 Active 账户管理员钱包列表。
        pub fn active_account_admins(
            institution_code: InstitutionCode,
            institution: T::AccountId,
        ) -> Option<Vec<T::AccountId>> {
            let account = Self::admin_account_with_status(
                institution_code,
                institution,
                AdminAccountStatus::Active,
            )?;
            Some(account.admins.into_inner())
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
    }
}

impl<T: pallet::Config> InstitutionAdminAccountLifecycle<T::AccountId> for pallet::Pallet<T> {
    fn close_institution_admin_account_for_proposal(
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

    fn set_active_institution_admin_account(
        _module_tag: &[u8],
        admin_root_account_id: T::AccountId,
        cid_number: Vec<u8>,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<T::AccountId>,
        threshold: u32,
    ) -> DispatchResult {
        Self::do_set_active_admin_account_direct(
            admin_root_account_id,
            cid_number,
            institution_code,
            kind,
            admins,
            threshold,
        )
    }

    fn sync_active_institution_admins_from_assignments(
        _module_tag: &[u8],
        admin_root_account_id: T::AccountId,
        cid_number: Vec<u8>,
        institution_code: InstitutionCode,
        admins: Vec<T::AccountId>,
    ) -> DispatchResult {
        Self::do_sync_active_admins_from_assignments(
            admin_root_account_id,
            cid_number,
            institution_code,
            admins,
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

    fn active_account_admins_len(
        institution_code: InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<u32> {
        Self::active_account_admins_len(institution_code, admin_root_account_id)
    }
}

#[cfg(test)]
mod tests;
