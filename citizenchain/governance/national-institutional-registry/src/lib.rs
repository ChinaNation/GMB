#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, Blake2_128Concat};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{SaturatedConversion, Saturating};

pub type InstitutionId = [u8; 8];
pub const INSTITUTION_ADMIN_COUNT: u32 = 5;
pub const ADMIN_REPLACE_PASS_THRESHOLD: u32 = 3;

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Institution<AccountId, Name, BlockNumber> {
    pub id: InstitutionId,
    pub name: Name,
    pub account: AccountId,
    pub admins: BoundedVec<AccountId, ConstU32<INSTITUTION_ADMIN_COUNT>>,
    pub active: bool,
    pub created_at: BlockNumber,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct AdminReplaceProposal<AccountId, BlockNumber> {
    pub institution_id: InstitutionId,
    pub old_admin: AccountId,
    pub new_admin: AccountId,
    pub proposer: AccountId,
    pub approve_count: u32,
    pub start: BlockNumber,
    pub end: BlockNumber,
    pub executed: bool,
    pub rejected: bool,
}

/// 中文注释：供支付模块查询机构账户与管理员权限的统一接口。
pub trait InstitutionAccess<AccountId> {
    fn institution_account(id: InstitutionId) -> Option<AccountId>;
    fn is_institution_admin(id: InstitutionId, who: &AccountId) -> bool;
    fn is_institution_active(id: InstitutionId) -> bool;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_std::vec::Vec;

    pub type NameOf<T> = BoundedVec<u8, <T as Config>::MaxInstitutionNameLen>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxInstitutionNameLen: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn institution)]
    pub type Institutions<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionId, Institution<T::AccountId, NameOf<T>, BlockNumberFor<T>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn institution_id_by_account)]
    pub type InstitutionIdByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, InstitutionId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_admin_replace_id)]
    pub type NextAdminReplaceId<T> = StorageMap<_, Blake2_128Concat, InstitutionId, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn admin_replace_proposal)]
    pub type AdminReplaceProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (InstitutionId, u64),
        AdminReplaceProposal<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn admin_replace_voted)]
    pub type AdminReplaceVoted<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (InstitutionId, u64),
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub institutions: Vec<(InstitutionId, Vec<u8>, T::AccountId, [T::AccountId; INSTITUTION_ADMIN_COUNT as usize])>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { institutions: Vec::new() }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (id, name_raw, account, admins_arr) in self.institutions.iter() {
                let name: NameOf<T> = name_raw.clone().try_into().expect("institution name too long in genesis");
                let admins_vec: Vec<T::AccountId> = admins_arr.to_vec();
                let admins: BoundedVec<T::AccountId, ConstU32<INSTITUTION_ADMIN_COUNT>> =
                    admins_vec.try_into().expect("genesis admins must be 5");

                let institution = Institution {
                    id: *id,
                    name,
                    account: account.clone(),
                    admins,
                    active: true,
                    created_at: Zero::zero(),
                };

                Institutions::<T>::insert(id, institution);
                InstitutionIdByAccount::<T>::insert(account.clone(), *id);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        InstitutionRegistered { id: InstitutionId, account: T::AccountId, who: T::AccountId },
        InstitutionDisabled { id: InstitutionId, who: T::AccountId },
        AdminReplaceProposed {
            institution_id: InstitutionId,
            proposal_id: u64,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
            proposer: T::AccountId,
        },
        AdminReplaceApproved {
            institution_id: InstitutionId,
            proposal_id: u64,
            who: T::AccountId,
            approve_count: u32,
        },
        AdminReplaced {
            institution_id: InstitutionId,
            proposal_id: u64,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        },
        AdminReplaceTimedOutRejected {
            institution_id: InstitutionId,
            proposal_id: u64,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InstitutionAlreadyExists,
        InstitutionNotFound,
        InstitutionAlreadyDisabled,
        InvalidAdminsCount,
        DuplicateAdmins,
        CallerNotAdmin,
        AccountAlreadyBound,
        EmptyName,
        OldAdminNotFound,
        NewAdminAlreadyExists,
        AdminReplaceProposalNotFound,
        AdminReplaceAlreadyVoted,
        AdminReplaceAlreadyExecuted,
        AdminReplaceAlreadyRejected,
        AdminReplaceTimedOutRejected,
        InstitutionNotActive,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 中文注释：注册新机构（任意签名账户均可发起，链上动态新增无需 runtime 升级）。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(5, 4))]
        pub fn register_institution(
            origin: OriginFor<T>,
            id: InstitutionId,
            name: NameOf<T>,
            account: T::AccountId,
            admins: BoundedVec<T::AccountId, ConstU32<INSTITUTION_ADMIN_COUNT>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!name.is_empty(), Error::<T>::EmptyName);
            ensure!(!Institutions::<T>::contains_key(id), Error::<T>::InstitutionAlreadyExists);
            ensure!(!InstitutionIdByAccount::<T>::contains_key(&account), Error::<T>::AccountAlreadyBound);
            ensure!(admins.len() as u32 == INSTITUTION_ADMIN_COUNT, Error::<T>::InvalidAdminsCount);

            // 中文注释：要求5个管理员互不重复。
            let mut dedup: sp_std::collections::btree_set::BTreeSet<T::AccountId> = sp_std::collections::btree_set::BTreeSet::new();
            for a in admins.iter() {
                ensure!(dedup.insert(a.clone()), Error::<T>::DuplicateAdmins);
            }

            let institution = Institution {
                id,
                name,
                account: account.clone(),
                admins,
                active: true,
                created_at: <frame_system::Pallet<T>>::block_number(),
            };

            Institutions::<T>::insert(id, institution);
            InstitutionIdByAccount::<T>::insert(account.clone(), id);

            Self::deposit_event(Event::<T>::InstitutionRegistered { id, account, who });
            Ok(())
        }

        /// 中文注释：停用机构（软删除，保留历史）。
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
        pub fn disable_institution(origin: OriginFor<T>, id: InstitutionId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Institutions::<T>::try_mutate(id, |maybe| -> DispatchResult {
                let institution = maybe.as_mut().ok_or(Error::<T>::InstitutionNotFound)?;
                ensure!(institution.active, Error::<T>::InstitutionAlreadyDisabled);
                ensure!(institution.admins.iter().any(|a| a == &who), Error::<T>::CallerNotAdmin);
                institution.active = false;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::InstitutionDisabled { id, who });
            Ok(())
        }

        /// 中文注释：发起机构管理员替换提案，发起人首签计 1 票，阈值 >=3 通过。
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 4))]
        pub fn propose_replace_admin(
            origin: OriginFor<T>,
            institution_id: InstitutionId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let institution = Institutions::<T>::get(institution_id).ok_or(Error::<T>::InstitutionNotFound)?;
            ensure!(institution.active, Error::<T>::InstitutionNotActive);
            ensure!(institution.admins.iter().any(|a| a == &who), Error::<T>::CallerNotAdmin);
            ensure!(institution.admins.iter().any(|a| a == &old_admin), Error::<T>::OldAdminNotFound);
            ensure!(!institution.admins.iter().any(|a| a == &new_admin), Error::<T>::NewAdminAlreadyExists);

            let proposal_id = NextAdminReplaceId::<T>::get(institution_id);
            NextAdminReplaceId::<T>::insert(institution_id, proposal_id.saturating_add(1));

            let now = <frame_system::Pallet<T>>::block_number();
            let end = now.saturating_add((primitives::count_const::VOTING_DURATION_BLOCKS as u64).saturated_into());
            let proposal = AdminReplaceProposal {
                institution_id,
                old_admin: old_admin.clone(),
                new_admin: new_admin.clone(),
                proposer: who.clone(),
                approve_count: 1,
                start: now,
                end,
                executed: false,
                rejected: false,
            };

            AdminReplaceProposals::<T>::insert((institution_id, proposal_id), proposal);
            AdminReplaceVoted::<T>::insert((institution_id, proposal_id), &who, true);

            Self::deposit_event(Event::<T>::AdminReplaceProposed {
                institution_id,
                proposal_id,
                old_admin,
                new_admin,
                proposer: who,
            });
            Ok(())
        }

        /// 中文注释：管理员对替换提案签名；达到 >=3 票后立即执行替换。
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().reads_writes(6, 6))]
        pub fn approve_replace_admin(
            origin: OriginFor<T>,
            institution_id: InstitutionId,
            proposal_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let institution = Institutions::<T>::get(institution_id).ok_or(Error::<T>::InstitutionNotFound)?;
            ensure!(institution.active, Error::<T>::InstitutionNotActive);
            ensure!(institution.admins.iter().any(|a| a == &who), Error::<T>::CallerNotAdmin);
            ensure!(!AdminReplaceVoted::<T>::get((institution_id, proposal_id), &who), Error::<T>::AdminReplaceAlreadyVoted);

            AdminReplaceProposals::<T>::try_mutate(
                (institution_id, proposal_id),
                |maybe_proposal| -> DispatchResult {
                    let proposal = maybe_proposal.as_mut().ok_or(Error::<T>::AdminReplaceProposalNotFound)?;
                    ensure!(!proposal.executed, Error::<T>::AdminReplaceAlreadyExecuted);
                    ensure!(!proposal.rejected, Error::<T>::AdminReplaceAlreadyRejected);

                    let now = <frame_system::Pallet<T>>::block_number();
                    if now > proposal.end {
                        proposal.rejected = true;
                        Self::deposit_event(Event::<T>::AdminReplaceTimedOutRejected {
                            institution_id,
                            proposal_id,
                        });
                        return Err(Error::<T>::AdminReplaceTimedOutRejected.into());
                    }

                    AdminReplaceVoted::<T>::insert((institution_id, proposal_id), &who, true);
                    proposal.approve_count = proposal.approve_count.saturating_add(1);
                    Self::deposit_event(Event::<T>::AdminReplaceApproved {
                        institution_id,
                        proposal_id,
                        who: who.clone(),
                        approve_count: proposal.approve_count,
                    });

                    if proposal.approve_count >= ADMIN_REPLACE_PASS_THRESHOLD {
                        Institutions::<T>::try_mutate(institution_id, |maybe_institution| -> DispatchResult {
                            let institution = maybe_institution.as_mut().ok_or(Error::<T>::InstitutionNotFound)?;
                            ensure!(institution.active, Error::<T>::InstitutionNotActive);
                            ensure!(institution.admins.iter().any(|a| a == &proposal.old_admin), Error::<T>::OldAdminNotFound);
                            ensure!(!institution.admins.iter().any(|a| a == &proposal.new_admin), Error::<T>::NewAdminAlreadyExists);

                            for admin in institution.admins.iter_mut() {
                                if *admin == proposal.old_admin {
                                    *admin = proposal.new_admin.clone();
                                    break;
                                }
                            }
                            Ok(())
                        })?;

                        proposal.executed = true;
                        Self::deposit_event(Event::<T>::AdminReplaced {
                            institution_id,
                            proposal_id,
                            old_admin: proposal.old_admin.clone(),
                            new_admin: proposal.new_admin.clone(),
                        });
                    }
                    Ok(())
                },
            )
        }
    }
}

impl<T: pallet::Config> InstitutionAccess<T::AccountId> for pallet::Pallet<T> {
    fn institution_account(id: InstitutionId) -> Option<T::AccountId> {
        pallet::Institutions::<T>::get(id).map(|i| i.account)
    }

    fn is_institution_admin(id: InstitutionId, who: &T::AccountId) -> bool {
        pallet::Institutions::<T>::get(id)
            .map(|i| i.active && i.admins.iter().any(|a| a == who))
            .unwrap_or(false)
    }

    fn is_institution_active(id: InstitutionId) -> bool {
        pallet::Institutions::<T>::get(id).map(|i| i.active).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32};
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, BuildStorage};

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
        pub type Registry = pallet;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Nonce = u64;
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxInstitutionNameLen = ConstU32<64>;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("storage should build")
            .into()
    }

    #[test]
    fn register_and_disable_work() {
        new_test_ext().execute_with(|| {
            let name: BoundedVec<u8, ConstU32<64>> = b"test".to_vec().try_into().expect("fit");
            let admins: BoundedVec<u64, ConstU32<INSTITUTION_ADMIN_COUNT>> =
                vec![1,2,3,4,5].try_into().expect("fit");

            assert_ok!(Registry::register_institution(
                RuntimeOrigin::signed(1),
                *b"GZGZF000",
                name,
                99,
                admins,
            ));

            assert!(Registry::is_institution_active(*b"GZGZF000"));
            assert_ok!(Registry::disable_institution(RuntimeOrigin::signed(2), *b"GZGZF000"));
            assert!(!Registry::is_institution_active(*b"GZGZF000"));

            assert_noop!(
                Registry::disable_institution(RuntimeOrigin::signed(2), *b"GZGZF000"),
                pallet::Error::<Test>::InstitutionAlreadyDisabled
            );
        });
    }

    #[test]
    fn replace_admin_passes_at_three_votes() {
        new_test_ext().execute_with(|| {
            let name: BoundedVec<u8, ConstU32<64>> = b"test".to_vec().try_into().expect("fit");
            let admins: BoundedVec<u64, ConstU32<INSTITUTION_ADMIN_COUNT>> =
                vec![1,2,3,4,5].try_into().expect("fit");

            assert_ok!(Registry::register_institution(
                RuntimeOrigin::signed(99),
                *b"GZGZF000",
                name,
                88,
                admins,
            ));

            assert_ok!(Registry::propose_replace_admin(
                RuntimeOrigin::signed(1),
                *b"GZGZF000",
                5,
                7,
            ));
            assert_ok!(Registry::approve_replace_admin(
                RuntimeOrigin::signed(2),
                *b"GZGZF000",
                0,
            ));
            assert_ok!(Registry::approve_replace_admin(
                RuntimeOrigin::signed(3),
                *b"GZGZF000",
                0,
            ));

            let institution = Registry::institution(*b"GZGZF000").expect("institution should exist");
            assert!(institution.admins.iter().any(|a| *a == 7));
            assert!(!institution.admins.iter().any(|a| *a == 5));
        });
    }

    #[test]
    fn replace_admin_timeout_rejects() {
        new_test_ext().execute_with(|| {
            let name: BoundedVec<u8, ConstU32<64>> = b"test".to_vec().try_into().expect("fit");
            let admins: BoundedVec<u64, ConstU32<INSTITUTION_ADMIN_COUNT>> =
                vec![1,2,3,4,5].try_into().expect("fit");

            assert_ok!(Registry::register_institution(
                RuntimeOrigin::signed(99),
                *b"GZGZF000",
                name,
                88,
                admins,
            ));
            assert_ok!(Registry::propose_replace_admin(
                RuntimeOrigin::signed(1),
                *b"GZGZF000",
                5,
                7,
            ));

            frame_system::Pallet::<Test>::set_block_number(primitives::count_const::VOTING_DURATION_BLOCKS as u64 + 2);
            assert_noop!(
                Registry::approve_replace_admin(RuntimeOrigin::signed(2), *b"GZGZF000", 0),
                pallet::Error::<Test>::AdminReplaceTimedOutRejected
            );
        });
    }
}
