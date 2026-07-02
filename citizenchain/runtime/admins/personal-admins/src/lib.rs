#![cfg_attr(not(feature = "std"), no_std)]
//! 个人多签管理员 pallet。
//!
//! 本模块只负责个人多签账户的管理员集合真源:
//! - 保存 `AdminAccounts[personal_account]`。
//! - 执行个人多签管理员集合变更。
//! - 给 `personal-manage` 提供管理员生命周期写入口。
//! - 给 runtime / multisig-transfer 提供管理员查询入口。
//!
//! 个人多签账户创建、关闭、资金处理和 `PersonalAccounts` 状态只属于
//! `runtime/entity/personal-manage`。

extern crate alloc;

use admin_primitives::{
    AdminAccount, AdminAccountKind, AdminAccountLifecycle, AdminAccountQuery, AdminAccountStatus,
    AdminSetChangeAction,
};
use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::{
    ensure,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::StorageVersion,
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::collections::btree_set::BTreeSet;
use votingengine::{
    types::{InstitutionCode, PMUL},
    InternalVoteResultCallback, ProposalExecutionOutcome, ProposalSubject, PROPOSAL_KIND_INTERNAL,
    STAGE_INTERNAL, STATUS_EXECUTION_FAILED, STATUS_PASSED, STATUS_REJECTED, STATUS_VOTING,
};

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

#[cfg(test)]
mod tests;

pub mod weights;

/// 个人多签管理员模块标识。生命周期提案使用 personal-manage 的 `per-mgmt`,
/// 管理员集合变更提案使用本标识，避免两个模块误认领同一提案。
pub const MODULE_TAG: &[u8] = b"per-admin";

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use votingengine::InternalVoteEngine;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 内部投票引擎。
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        /// 单个个人多签账户管理员最大数量上限。
        #[pallet::constant]
        type MaxPersonalAccountAdmins: Get<u32>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    pub type AdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxPersonalAccountAdmins>;

    pub type AdminAccountOf<T> =
        AdminAccount<AdminsOf<T>, <T as frame_system::Config>::AccountId, BlockNumberFor<T>>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 个人多签管理员集合。key 为 personal_account。
    ///
    /// 个人多签不依赖 CID 资料，管理员真源只保存 AccountId 集合。
    /// 账户名、创建者和生命周期状态属于 personal-manage。
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

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {}
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 已发起个人多签管理员集合变更提案。
        AdminSetChangeProposed {
            proposal_id: u64,
            account: T::AccountId,
            proposer: T::AccountId,
            old_admins_len: u32,
            new_admins_len: u32,
            new_threshold: u32,
        },
        /// 个人多签管理员集合已完成执行。
        AdminSetChanged {
            proposal_id: u64,
            account: T::AccountId,
            admins_len: u32,
            threshold: u32,
        },
        /// 个人多签管理员集合提案通过后执行失败。
        AdminSetChangeExecutionFailed { proposal_id: u64 },
        /// 个人多签管理员账户已写入 Pending。
        AdminAccountPendingCreated {
            account: T::AccountId,
            creator: T::AccountId,
            admins_len: u32,
        },
        /// 个人多签管理员账户已激活。
        AdminAccountActivated { account: T::AccountId },
        /// Pending 个人多签管理员账户已清理。
        AdminAccountPendingRemoved { account: T::AccountId },
        /// 个人多签管理员账户已关闭。
        AdminAccountClosed { account: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        DuplicateAdmin,
        InvalidThreshold,
        PermissionDenied,
        InvalidAdminsLen,
        PersonalNotFound,
        PersonalNotActive,
        PersonalAlreadyExists,
        NotPersonalAccount,
        AdminSetUnchanged,
        ProposalActionNotFound,
        InvalidLifecycleScope,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起个人多签管理员集合变更提案。
        ///
        /// 本入口只改个人多签管理员集合。个人多签账户创建/关闭
        /// 必须走 personal-manage。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_admin_set_change())]
        pub fn propose_admin_set_change(
            origin: OriginFor<T>,
            institution_code: InstitutionCode,
            account: T::AccountId,
            admins: AdminsOf<T>,
            new_threshold: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(institution_code == PMUL, Error::<T>::NotPersonalAccount);
            let current =
                AdminAccounts::<T>::get(account.clone()).ok_or(Error::<T>::PersonalNotFound)?;
            ensure!(
                current.status == AdminAccountStatus::Active
                    && current.kind == AdminAccountKind::PersonalMultisig
                    && current.institution_code == PMUL,
                Error::<T>::NotPersonalAccount
            );
            let current_admins = current.admins.clone().into_inner();
            ensure!(current_admins.contains(&who), Error::<T>::PermissionDenied);
            Self::validate_admin_set_for_change(&admins, new_threshold)?;
            ensure!(
                !Self::same_admin_set(current_admins.as_slice(), admins.as_slice()),
                Error::<T>::AdminSetUnchanged
            );

            with_transaction(|| {
                let action = AdminSetChangeAction {
                    admin_root_account_id: account.clone(),
                    admins: admins.clone(),
                    new_threshold,
                };
                let proposal_id =
                    match T::InternalVoteEngine::create_admin_change_internal_proposal_with_data(
                        who.clone(),
                        PMUL,
                        account.clone(),
                        Vec::new(),
                        admins.len() as u32,
                        new_threshold,
                        crate::MODULE_TAG,
                        action.encode(),
                    ) {
                        Ok(proposal_id) => proposal_id,
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    };
                Self::deposit_event(Event::<T>::AdminSetChangeProposed {
                    proposal_id,
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
        fn validate_admin_set_for_change(
            admins: &AdminsOf<T>,
            new_threshold: u32,
        ) -> DispatchResult {
            let admins_len = admins.len() as u32;
            ensure!(admins_len >= 2, Error::<T>::InvalidAdminsLen);
            ensure!(
                admins_len <= <T as Config>::MaxPersonalAccountAdmins::get(),
                Error::<T>::InvalidAdminsLen
            );
            ensure!(
                new_threshold > 0
                    && new_threshold <= admins_len
                    && u64::from(new_threshold).saturating_mul(2) > u64::from(admins_len),
                Error::<T>::InvalidThreshold
            );
            Self::ensure_unique_admins(admins)?;
            Ok(())
        }

        fn ensure_unique_admins(admins: &AdminsOf<T>) -> DispatchResult {
            let mut seen = BTreeSet::new();
            for admin in admins.iter() {
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

        pub(crate) fn ensure_lifecycle_proposal(
            proposal_id: u64,
            module_tag: &[u8],
            account: T::AccountId,
            expected_status: u8,
            require_callback_scope: bool,
        ) -> DispatchResult {
            ensure!(
                votingengine::Pallet::<T>::is_proposal_owner(proposal_id, module_tag),
                Error::<T>::InvalidLifecycleScope
            );
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::InvalidLifecycleScope)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL,
                Error::<T>::InvalidLifecycleScope
            );
            ensure!(
                proposal.stage == STAGE_INTERNAL,
                Error::<T>::InvalidLifecycleScope
            );
            ensure!(
                proposal.account_context == Some(account),
                Error::<T>::InvalidLifecycleScope
            );
            ensure!(
                proposal.internal_code == Some(PMUL),
                Error::<T>::InvalidLifecycleScope
            );
            ensure!(
                proposal.status == expected_status,
                Error::<T>::InvalidLifecycleScope
            );
            if require_callback_scope {
                ensure!(
                    votingengine::Pallet::<T>::is_callback_execution_scope(proposal_id),
                    Error::<T>::InvalidLifecycleScope
                );
            }
            Ok(())
        }

        pub(crate) fn do_create_pending_admin_account(
            account: T::AccountId,
            kind: AdminAccountKind,
            admins: Vec<T::AccountId>,
            creator: T::AccountId,
        ) -> DispatchResult {
            ensure!(
                kind == AdminAccountKind::PersonalMultisig,
                Error::<T>::NotPersonalAccount
            );
            ensure!(
                !AdminAccounts::<T>::contains_key(account.clone()),
                Error::<T>::PersonalAlreadyExists
            );
            let bounded: AdminsOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            Self::ensure_unique_admins(&bounded)?;
            let now = frame_system::Pallet::<T>::block_number();
            let admins_len = bounded.len() as u32;
            AdminAccounts::<T>::insert(
                account.clone(),
                AdminAccount {
                    institution_code: PMUL,
                    kind,
                    admins: bounded,
                    creator: creator.clone(),
                    created_at: now,
                    updated_at: now,
                    status: AdminAccountStatus::Pending,
                },
            );
            Self::deposit_event(Event::<T>::AdminAccountPendingCreated {
                account,
                creator,
                admins_len,
            });
            Ok(())
        }

        pub(crate) fn do_activate_admin_account(account: T::AccountId) -> DispatchResult {
            AdminAccounts::<T>::try_mutate(account.clone(), |maybe| -> DispatchResult {
                let admin_account = maybe.as_mut().ok_or(Error::<T>::PersonalNotFound)?;
                ensure!(
                    admin_account.status == AdminAccountStatus::Pending,
                    Error::<T>::PersonalNotActive
                );
                admin_account.status = AdminAccountStatus::Active;
                admin_account.updated_at = frame_system::Pallet::<T>::block_number();
                Ok(())
            })?;
            Self::deposit_event(Event::<T>::AdminAccountActivated { account });
            Ok(())
        }

        pub(crate) fn do_remove_pending_admin_account(account: T::AccountId) -> DispatchResult {
            let admin_account =
                AdminAccounts::<T>::get(account.clone()).ok_or(Error::<T>::PersonalNotFound)?;
            ensure!(
                admin_account.status == AdminAccountStatus::Pending,
                Error::<T>::PersonalNotActive
            );
            AdminAccounts::<T>::remove(account.clone());
            Self::deposit_event(Event::<T>::AdminAccountPendingRemoved { account });
            Ok(())
        }

        pub(crate) fn do_close_admin_account(account: T::AccountId) -> DispatchResult {
            let admin_account =
                AdminAccounts::<T>::get(account.clone()).ok_or(Error::<T>::PersonalNotFound)?;
            ensure!(
                admin_account.status == AdminAccountStatus::Active,
                Error::<T>::PersonalNotActive
            );
            AdminAccounts::<T>::remove(account.clone());
            Self::deposit_event(Event::<T>::AdminAccountClosed { account });
            Ok(())
        }

        pub(crate) fn admin_account_with_status(
            institution_code: InstitutionCode,
            account: T::AccountId,
            status: AdminAccountStatus,
        ) -> Option<AdminAccountOf<T>> {
            if institution_code != PMUL {
                return None;
            }
            let admin_account = AdminAccounts::<T>::get(account)?;
            if admin_account.status == status
                && admin_account.kind == AdminAccountKind::PersonalMultisig
                && admin_account.institution_code == PMUL
            {
                Some(admin_account)
            } else {
                None
            }
        }

        pub fn active_admin_account_exists(
            institution_code: InstitutionCode,
            account: T::AccountId,
        ) -> bool {
            Self::admin_account_with_status(institution_code, account, AdminAccountStatus::Active)
                .is_some()
        }

        pub fn is_active_account_admin(
            institution_code: InstitutionCode,
            account: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            let Some(admin_account) = Self::admin_account_with_status(
                institution_code,
                account,
                AdminAccountStatus::Active,
            ) else {
                return false;
            };
            admin_account.admins.iter().any(|admin| admin == who)
        }

        pub fn active_account_admins(
            institution_code: InstitutionCode,
            account: T::AccountId,
        ) -> Option<Vec<T::AccountId>> {
            Some(
                Self::admin_account_with_status(
                    institution_code,
                    account,
                    AdminAccountStatus::Active,
                )?
                .admins
                .into_inner(),
            )
        }

        pub fn active_account_admins_len(
            institution_code: InstitutionCode,
            account: T::AccountId,
        ) -> Option<u32> {
            Some(
                Self::admin_account_with_status(
                    institution_code,
                    account,
                    AdminAccountStatus::Active,
                )?
                .admins
                .len() as u32,
            )
        }

        pub fn pending_account_exists_for_snapshot(
            institution_code: InstitutionCode,
            account: T::AccountId,
        ) -> bool {
            Self::admin_account_with_status(institution_code, account, AdminAccountStatus::Pending)
                .is_some()
        }

        pub fn is_pending_account_admin_for_snapshot(
            institution_code: InstitutionCode,
            account: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            let Some(admin_account) = Self::admin_account_with_status(
                institution_code,
                account,
                AdminAccountStatus::Pending,
            ) else {
                return false;
            };
            admin_account.admins.iter().any(|admin| admin == who)
        }

        pub fn pending_account_admins_for_snapshot(
            institution_code: InstitutionCode,
            account: T::AccountId,
        ) -> Option<Vec<T::AccountId>> {
            Some(
                Self::admin_account_with_status(
                    institution_code,
                    account,
                    AdminAccountStatus::Pending,
                )?
                .admins
                .into_inner(),
            )
        }

        pub fn pending_account_admins_len_for_snapshot(
            institution_code: InstitutionCode,
            account: T::AccountId,
        ) -> Option<u32> {
            Some(
                Self::admin_account_with_status(
                    institution_code,
                    account,
                    AdminAccountStatus::Pending,
                )?
                .admins
                .len() as u32,
            )
        }

        pub(crate) fn try_execute_set_change_from_action(
            proposal_id: u64,
            action: AdminSetChangeAction<T::AccountId, AdminsOf<T>>,
        ) -> DispatchResult {
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL && proposal.stage == STAGE_INTERNAL,
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.account_context == Some(action.admin_root_account_id.clone()),
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.internal_code == Some(PMUL),
                Error::<T>::ProposalActionNotFound
            );
            votingengine::Pallet::<T>::ensure_admin_set_mutation_lock_owner(
                ProposalSubject::PersonalAccount(action.admin_root_account_id.clone()),
                proposal_id,
            )?;

            let current = AdminAccounts::<T>::get(action.admin_root_account_id.clone())
                .ok_or(Error::<T>::PersonalNotFound)?;
            ensure!(
                current.status == AdminAccountStatus::Active
                    && current.kind == AdminAccountKind::PersonalMultisig
                    && current.institution_code == PMUL,
                Error::<T>::NotPersonalAccount
            );
            let current_admins = current.admins.clone().into_inner();
            Self::validate_admin_set_for_change(&action.admins, action.new_threshold)?;
            ensure!(
                !Self::same_admin_set(current_admins.as_slice(), action.admins.as_slice()),
                Error::<T>::AdminSetUnchanged
            );

            AdminAccounts::<T>::mutate(action.admin_root_account_id.clone(), |maybe| {
                if let Some(account) = maybe {
                    account.admins = action.admins.clone();
                    account.updated_at = frame_system::Pallet::<T>::block_number();
                }
            });
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

impl<T: pallet::Config> AdminAccountLifecycle<T::AccountId> for pallet::Pallet<T> {
    fn create_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: T::AccountId,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<T::AccountId>,
        creator: T::AccountId,
    ) -> DispatchResult {
        ensure!(
            institution_code == PMUL,
            pallet::Error::<T>::NotPersonalAccount
        );
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            admin_root_account_id.clone(),
            STATUS_VOTING,
            false,
        )?;
        Self::do_create_pending_admin_account(admin_root_account_id, kind, admins, creator)
    }

    fn activate_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: T::AccountId,
    ) -> DispatchResult {
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            admin_root_account_id.clone(),
            STATUS_PASSED,
            true,
        )?;
        Self::do_activate_admin_account(admin_root_account_id)
    }

    fn remove_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: T::AccountId,
    ) -> DispatchResult {
        let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
            .ok_or(pallet::Error::<T>::ProposalActionNotFound)?;
        ensure!(
            matches!(proposal.status, STATUS_REJECTED | STATUS_EXECUTION_FAILED),
            pallet::Error::<T>::ProposalActionNotFound
        );
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            admin_root_account_id.clone(),
            proposal.status,
            false,
        )?;
        Self::do_remove_pending_admin_account(admin_root_account_id)
    }

    fn close_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: T::AccountId,
    ) -> DispatchResult {
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            admin_root_account_id.clone(),
            STATUS_PASSED,
            true,
        )?;
        Self::do_close_admin_account(admin_root_account_id)
    }
}

impl<T: pallet::Config> AdminAccountQuery<T::AccountId> for pallet::Pallet<T> {
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
        _institution_code: InstitutionCode,
        _admin_root_account_id: T::AccountId,
    ) -> Option<T::AccountId> {
        None
    }
}

pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, sp_runtime::DispatchError> {
        if !votingengine::Pallet::<T>::is_proposal_owner(proposal_id, crate::MODULE_TAG) {
            return Ok(ProposalExecutionOutcome::Ignored);
        }
        let raw = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
            .ok_or(pallet::Error::<T>::ProposalActionNotFound)?;

        if !approved {
            return Ok(ProposalExecutionOutcome::Executed);
        }

        let action =
            AdminSetChangeAction::<T::AccountId, pallet::AdminsOf<T>>::decode(&mut &raw[..])
                .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
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
