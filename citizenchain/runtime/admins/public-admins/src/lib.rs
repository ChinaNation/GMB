#![cfg_attr(not(feature = "std"), no_std)]
//! 公权机构管理员钱包集合模块。
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
    can_store_public_admin_code, AdminAccountKind, AdminCidNumber, InstitutionAdmin,
    InstitutionAdminLifecycle, InstitutionAdminQuery, InstitutionAdmins,
};
use votingengine::types::InstitutionCode;

pub use pallet::*;

/// v4: 机构管理员集合从 `Vec<AccountId>` 升级为
/// `Vec<InstitutionAdmin { admin_name, admin_account }>`。
///
/// 正式链不得再依赖重建数据；旧 v2 存储在 runtime 升级时一次性翻译为目标结构，
/// 查询路径只认 v4 新结构，不保留双轨兼容。
const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

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

    pub type AdminsOf<T> = BoundedVec<
        InstitutionAdmin<<T as frame_system::Config>::AccountId>,
        <T as Config>::MaxAdminsPerInstitution,
    >;
    pub type InstitutionAdminsOf<T> = InstitutionAdmins<AdminsOf<T>>;
    type LegacyAdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAdminsPerInstitution>;
    type LegacyInstitutionAdminsOf<T> = InstitutionAdmins<LegacyAdminsOf<T>>;

    /// 公权机构管理员集合。CID 是唯一 key；value 不重复保存 CID 或生命周期状态。
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
        fn on_runtime_upgrade() -> Weight {
            let db = T::DbWeight::get();
            let on_chain = StorageVersion::get::<Pallet<T>>();
            if on_chain >= STORAGE_VERSION {
                return db.reads(1);
            }

            let mut migrated = 0u64;
            if on_chain <= StorageVersion::new(2) {
                AdminAccounts::<T>::translate::<LegacyInstitutionAdminsOf<T>, _>(
                    |_cid_number, legacy| {
                        migrated = migrated.saturating_add(1);
                        let admins = legacy
                            .admins
                            .into_inner()
                            .into_iter()
                            .map(|admin_account| InstitutionAdmin {
                                admin_name: admin_primitives::AdminName::truncate_from(
                                    admin_primitives::DEFAULT_ADMIN_NAME.to_vec(),
                                ),
                                admin_account,
                            })
                            .collect::<Vec<_>>();
                        let Ok(admins) = AdminsOf::<T>::try_from(admins) else {
                            return None;
                        };
                        Some(InstitutionAdmins {
                            institution_code: legacy.institution_code,
                            admins,
                        })
                    },
                );
            }

            STORAGE_VERSION.put::<Pallet<T>>();
            db.reads_writes(1u64.saturating_add(migrated), 1u64.saturating_add(migrated))
        }

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
        /// 注册局原子写入机构管理员集合与动态阈值。
        InstitutionAdminsSet {
            cid_number: AdminCidNumber,
            institution_code: InstitutionCode,
            admins_len: u32,
            threshold: u32,
            created: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidInstitution,
        InstitutionCodeMismatch,
        InvalidAdminsLen,
        InvalidAdminAccountKind,
        InvalidThreshold,
        DuplicateAdmin,
        InvalidAdminName,
    }

    impl<T: Config> Pallet<T> {
        fn ensure_unique_admins(admins: &[InstitutionAdmin<T::AccountId>]) -> DispatchResult {
            let mut seen = BTreeSet::new();
            for admin in admins {
                ensure!(!admin.admin_name.is_empty(), Error::<T>::InvalidAdminName);
                ensure!(
                    seen.insert(admin.admin_account.clone()),
                    Error::<T>::DuplicateAdmin
                );
            }
            Ok(())
        }

        fn validate_admin_set(
            kind: AdminAccountKind,
            institution_code: InstitutionCode,
            cid_number: &[u8],
            admins: &[InstitutionAdmin<T::AccountId>],
        ) -> DispatchResult {
            ensure!(
                kind == AdminAccountKind::PublicInstitution
                    && can_store_public_admin_code(&institution_code),
                Error::<T>::InvalidAdminAccountKind
            );
            match admin_primitives::expected_fixed_governance_admins_len(
                institution_code,
                cid_number,
            ) {
                Some(expected) => {
                    ensure!(
                        admins.len() == expected as usize,
                        Error::<T>::InvalidAdminsLen
                    )
                }
                None => match primitives::institution_constraints::member_composition_by_identity(
                    institution_code,
                    cid_number,
                ) {
                    Some(spec) => ensure!(
                        admins.len() >= spec.min_members as usize
                            && admins.len() <= spec.max_members as usize,
                        Error::<T>::InvalidAdminsLen
                    ),
                    None => {
                        ensure!(admins.len() >= 2, Error::<T>::InvalidAdminsLen);
                        ensure!(
                            admins.len() <= <T as Config>::MaxAdminsPerInstitution::get() as usize,
                            Error::<T>::InvalidAdminsLen
                        );
                    }
                },
            }
            Self::ensure_unique_admins(admins)
        }

        fn bound_cid(cid_number: Vec<u8>) -> Result<AdminCidNumber, DispatchError> {
            cid_number
                .try_into()
                .map_err(|_| Error::<T>::InvalidInstitution.into())
        }

        fn bounded_cid(cid_number: &[u8]) -> Option<AdminCidNumber> {
            cid_number.to_vec().try_into().ok()
        }

        fn validate_threshold_policy(
            cid_number: &AdminCidNumber,
            institution_code: InstitutionCode,
            admins_len: u32,
            threshold: u32,
        ) -> DispatchResult {
            if let Some(fixed) =
                primitives::cid::code::fixed_governance_pass_threshold(&institution_code)
            {
                ensure!(threshold == fixed, Error::<T>::InvalidThreshold);
                return Ok(());
            }
            if primitives::institution_constraints::singleton_by_identity(
                institution_code,
                cid_number.as_slice(),
            )
            .is_some()
            {
                ensure!(
                    threshold == admins_len / 2 + 1,
                    Error::<T>::InvalidThreshold
                );
                return Ok(());
            }
            T::InternalVoteEngine::register_active_institution_threshold_direct(
                institution_code,
                cid_number.to_vec(),
                admins_len,
                threshold,
            )
        }

        pub(crate) fn do_set_institution_admins(
            cid_number: Vec<u8>,
            institution_code: InstitutionCode,
            kind: AdminAccountKind,
            admins: Vec<InstitutionAdmin<T::AccountId>>,
            threshold: u32,
        ) -> DispatchResult {
            let cid_number = Self::bound_cid(cid_number)?;
            Self::validate_admin_set(kind, institution_code, cid_number.as_slice(), &admins)?;
            let bounded: AdminsOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            let admins_len = bounded.len() as u32;

            with_transaction(|| {
                if let Err(err) = Self::validate_threshold_policy(
                    &cid_number,
                    institution_code,
                    admins_len,
                    threshold,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
                let created = match AdminAccounts::<T>::get(&cid_number) {
                    Some(existing) => {
                        if existing.institution_code != institution_code {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::InstitutionCodeMismatch.into(),
                            ));
                        }
                        AdminAccounts::<T>::insert(
                            &cid_number,
                            InstitutionAdmins {
                                institution_code,
                                admins: bounded.clone(),
                            },
                        );
                        false
                    }
                    None => {
                        AdminAccounts::<T>::insert(
                            &cid_number,
                            InstitutionAdmins {
                                institution_code,
                                admins: bounded.clone(),
                            },
                        );
                        true
                    }
                };
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

        pub(crate) fn get_institution_admins(
            institution_code: InstitutionCode,
            cid_number: &[u8],
        ) -> Option<InstitutionAdminsOf<T>> {
            let cid_number = Self::bounded_cid(cid_number)?;
            let value = AdminAccounts::<T>::get(cid_number)?;
            if value.institution_code != institution_code
                || !can_store_public_admin_code(&institution_code)
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
        admins: Vec<InstitutionAdmin<T::AccountId>>,
        threshold: u32,
    ) -> DispatchResult {
        Self::do_set_institution_admins(cid_number, institution_code, kind, admins, threshold)
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
            .map(|value| value.admins.iter().any(|admin| &admin.admin_account == who))
            .unwrap_or(false)
    }

    fn institution_admins(
        institution_code: InstitutionCode,
        cid_number: &[u8],
    ) -> Option<Vec<T::AccountId>> {
        Self::get_institution_admins(institution_code, cid_number).map(|value| {
            value
                .admins
                .into_inner()
                .into_iter()
                .map(|admin| admin.admin_account)
                .collect()
        })
    }

    fn institution_admin_records(
        institution_code: InstitutionCode,
        cid_number: &[u8],
    ) -> Option<Vec<InstitutionAdmin<T::AccountId>>> {
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
