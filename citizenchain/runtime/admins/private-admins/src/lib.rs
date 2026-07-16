#![cfg_attr(not(feature = "std"), no_std)]
//! 私权机构管理员钱包集合模块。
//!
//! 机构唯一身份是 CID。本模块只保存 `AdminAccounts[cid_number] -> admins`；
//! 岗位和任职归 entity，投票阈值归 votingengine，机构账户不参与管理员寻址。

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
    can_store_private_admin_code, AdminAccountKind, AdminCidNumber, InstitutionAdminLifecycle,
    InstitutionAdminQuery, InstitutionAdmins,
};
use votingengine::types::InstitutionCode;

pub use pallet::*;

/// breaking runtime 直接重新创世，不提供旧账户 key 布局迁移。
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
        type MaxAdminsPerInstitution: Get<u32>;

        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    pub type AdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAdminsPerInstitution>;
    pub type InstitutionAdminsOf<T> = InstitutionAdmins<AdminsOf<T>>;

    /// 私权机构管理员集合。CID 是唯一 key；value 不重复保存 CID 或生命周期状态。
    #[pallet::storage]
    #[pallet::getter(fn institution_admins_of)]
    pub type AdminAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, AdminCidNumber, InstitutionAdminsOf<T>, OptionQuery>;

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
        InstitutionAdminsSet {
            cid_number: AdminCidNumber,
            institution_code: InstitutionCode,
            admins_len: u32,
            threshold: u32,
            created: bool,
        },
        InstitutionAdminsSyncedFromAssignments {
            cid_number: AdminCidNumber,
            institution_code: InstitutionCode,
            admins_len: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidInstitution,
        InstitutionCodeMismatch,
        InvalidAdminsLen,
        InvalidAdminAccountKind,
        InvalidThreshold,
        MissingDynamicThreshold,
        DuplicateAdmin,
    }

    impl<T: Config> Pallet<T> {
        fn bound_cid(cid_number: Vec<u8>) -> Result<AdminCidNumber, DispatchError> {
            cid_number
                .try_into()
                .map_err(|_| Error::<T>::InvalidInstitution.into())
        }

        fn bounded_cid(cid_number: &[u8]) -> Option<AdminCidNumber> {
            cid_number.to_vec().try_into().ok()
        }

        fn validate_admin_set(
            kind: AdminAccountKind,
            institution_code: InstitutionCode,
            admins: &[T::AccountId],
        ) -> DispatchResult {
            ensure!(
                kind == AdminAccountKind::PrivateInstitution
                    && can_store_private_admin_code(&institution_code),
                Error::<T>::InvalidAdminAccountKind
            );
            ensure!(admins.len() >= 2, Error::<T>::InvalidAdminsLen);
            ensure!(
                admins.len() <= <T as Config>::MaxAdminsPerInstitution::get() as usize,
                Error::<T>::InvalidAdminsLen
            );
            let mut seen = BTreeSet::new();
            for admin in admins {
                ensure!(seen.insert(admin.clone()), Error::<T>::DuplicateAdmin);
            }
            Ok(())
        }

        pub(crate) fn do_set_institution_admins(
            cid_number: Vec<u8>,
            institution_code: InstitutionCode,
            kind: AdminAccountKind,
            admins: Vec<T::AccountId>,
            threshold: u32,
        ) -> DispatchResult {
            let cid_number = Self::bound_cid(cid_number)?;
            Self::validate_admin_set(kind, institution_code, &admins)?;
            let bounded: AdminsOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            let admins_len = bounded.len() as u32;

            with_transaction(|| {
                if let Err(err) =
                    T::InternalVoteEngine::register_active_institution_threshold_direct(
                        institution_code,
                        cid_number.to_vec(),
                        admins_len,
                        threshold,
                    )
                {
                    return TransactionOutcome::Rollback(Err(err));
                }
                let created = match AdminAccounts::<T>::get(&cid_number) {
                    Some(existing) => {
                        if existing.institution_code != institution_code {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::InstitutionCodeMismatch.into(),
                            ));
                        }
                        false
                    }
                    None => true,
                };
                AdminAccounts::<T>::insert(
                    &cid_number,
                    InstitutionAdmins {
                        institution_code,
                        admins: bounded,
                    },
                );
                Self::deposit_event(Event::<T>::InstitutionAdminsSet {
                    cid_number,
                    institution_code,
                    admins_len,
                    threshold,
                    created,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }

        pub(crate) fn do_sync_institution_admins_from_assignments(
            cid_number: Vec<u8>,
            institution_code: InstitutionCode,
            admins: Vec<T::AccountId>,
        ) -> DispatchResult {
            let cid_number = Self::bound_cid(cid_number)?;
            Self::validate_admin_set(
                AdminAccountKind::PrivateInstitution,
                institution_code,
                &admins,
            )?;
            let existing =
                AdminAccounts::<T>::get(&cid_number).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                existing.institution_code == institution_code,
                Error::<T>::InstitutionCodeMismatch
            );
            let bounded: AdminsOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            let admins_len = bounded.len() as u32;
            let threshold = T::InternalVoteEngine::active_institution_threshold(
                institution_code,
                cid_number.as_slice(),
            )
            .ok_or(Error::<T>::MissingDynamicThreshold)?;

            with_transaction(|| {
                if T::InternalVoteEngine::register_active_institution_threshold_direct(
                    institution_code,
                    cid_number.to_vec(),
                    admins_len,
                    threshold,
                )
                .is_err()
                {
                    return TransactionOutcome::Rollback(Err(Error::<T>::InvalidThreshold.into()));
                }
                AdminAccounts::<T>::insert(
                    &cid_number,
                    InstitutionAdmins {
                        institution_code,
                        admins: bounded,
                    },
                );
                Self::deposit_event(Event::<T>::InstitutionAdminsSyncedFromAssignments {
                    cid_number,
                    institution_code,
                    admins_len,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }

        pub(crate) fn get_institution_admins(
            institution_code: InstitutionCode,
            cid_number: &[u8],
        ) -> Option<InstitutionAdminsOf<T>> {
            let cid_number = Self::bounded_cid(cid_number)?;
            let value = AdminAccounts::<T>::get(cid_number)?;
            if value.institution_code != institution_code
                || !can_store_private_admin_code(&institution_code)
            {
                return None;
            }
            Some(value)
        }
    }
}

impl<T: pallet::Config> InstitutionAdminLifecycle<T::AccountId> for pallet::Pallet<T> {
    fn set_institution_admins(
        _module_tag: &[u8],
        cid_number: Vec<u8>,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<T::AccountId>,
        threshold: u32,
    ) -> DispatchResult {
        Self::do_set_institution_admins(cid_number, institution_code, kind, admins, threshold)
    }

    fn sync_institution_admins_from_assignments(
        _module_tag: &[u8],
        cid_number: Vec<u8>,
        institution_code: InstitutionCode,
        admins: Vec<T::AccountId>,
    ) -> DispatchResult {
        Self::do_sync_institution_admins_from_assignments(cid_number, institution_code, admins)
    }
}

impl<T: pallet::Config> InstitutionAdminQuery<T::AccountId> for pallet::Pallet<T> {
    fn institution_admins_exist(institution_code: InstitutionCode, cid_number: &[u8]) -> bool {
        Self::get_institution_admins(institution_code, cid_number).is_some()
    }

    fn is_institution_admin(
        institution_code: InstitutionCode,
        cid_number: &[u8],
        who: &T::AccountId,
    ) -> bool {
        Self::get_institution_admins(institution_code, cid_number)
            .map(|value| value.admins.iter().any(|admin| admin == who))
            .unwrap_or(false)
    }

    fn institution_admins(
        institution_code: InstitutionCode,
        cid_number: &[u8],
    ) -> Option<Vec<T::AccountId>> {
        Self::get_institution_admins(institution_code, cid_number)
            .map(|value| value.admins.into_inner())
    }

    fn institution_admins_len(institution_code: InstitutionCode, cid_number: &[u8]) -> Option<u32> {
        Self::get_institution_admins(institution_code, cid_number)
            .map(|value| value.admins.len() as u32)
    }
}

#[cfg(test)]
mod tests;
