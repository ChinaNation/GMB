#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency, Blake2_128Concat};
use frame_system::pallet_prelude::*;
use national_institutional_registry::{InstitutionAccess, InstitutionId, INSTITUTION_ADMIN_COUNT};
use scale_info::TypeInfo;
use sp_runtime::traits::{SaturatedConversion, Saturating, Zero};

const PAYMENT_PASS_THRESHOLD: u32 = 3;

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PaymentProposal<AccountId, Balance, BlockNumber, Memo> {
    pub institution: InstitutionId,
    pub to: AccountId,
    pub amount: Balance,
    pub proposer: AccountId,
    pub approve_count: u32,
    pub start: BlockNumber,
    pub end: BlockNumber,
    pub executed: bool,
    pub rejected: bool,
    pub memo: Memo,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    pub type MemoOf<T> = BoundedVec<u8, <T as Config>::MaxMemoLen>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        type InstitutionRegistry: InstitutionAccess<Self::AccountId>;

        #[pallet::constant]
        type MaxMemoLen: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_payment_id)]
    pub type NextPaymentId<T> = StorageMap<_, Blake2_128Concat, InstitutionId, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn payment)]
    pub type Payments<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (InstitutionId, u64),
        PaymentProposal<T::AccountId, BalanceOf<T>, BlockNumberFor<T>, MemoOf<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn voted)]
    pub type Voted<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, (InstitutionId, u64), Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PaymentProposed { institution: InstitutionId, payment_id: u64, proposer: T::AccountId, to: T::AccountId, amount: BalanceOf<T> },
        PaymentApproved { institution: InstitutionId, payment_id: u64, who: T::AccountId, approve_count: u32 },
        PaymentExecuted { institution: InstitutionId, payment_id: u64, to: T::AccountId, amount: BalanceOf<T> },
        PaymentRejectedTimeout { institution: InstitutionId, payment_id: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        InstitutionNotActive,
        NotInstitutionAdmin,
        PaymentNotFound,
        AlreadyVoted,
        AlreadyExecuted,
        AlreadyRejected,
        PaymentTimedOutRejected,
        ZeroAmount,
        NotEnoughApprovals,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 中文注释：机构管理员发起支付提案；发起人首签计入1票。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 4))]
        pub fn propose_payment(
            origin: OriginFor<T>,
            institution: InstitutionId,
            to: T::AccountId,
            amount: BalanceOf<T>,
            memo: MemoOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);
            ensure!(T::InstitutionRegistry::is_institution_active(institution), Error::<T>::InstitutionNotActive);
            ensure!(T::InstitutionRegistry::is_institution_admin(institution, &who), Error::<T>::NotInstitutionAdmin);

            let payment_id = NextPaymentId::<T>::get(institution);
            NextPaymentId::<T>::insert(institution, payment_id.saturating_add(1));

            let now = <frame_system::Pallet<T>>::block_number();
            let end = now.saturating_add((primitives::count_const::VOTING_DURATION_BLOCKS as u64).saturated_into());

            let proposal = PaymentProposal {
                institution,
                to: to.clone(),
                amount,
                proposer: who.clone(),
                approve_count: 1,
                start: now,
                end,
                executed: false,
                rejected: false,
                memo,
            };

            Payments::<T>::insert((institution, payment_id), proposal);
            Voted::<T>::insert((institution, payment_id), &who, true);

            Self::deposit_event(Event::<T>::PaymentProposed { institution, payment_id, proposer: who, to, amount });
            Ok(())
        }

        /// 中文注释：机构管理员签名确认；达到>=3票时自动执行转账。
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(6, 6))]
        pub fn approve_payment(
            origin: OriginFor<T>,
            institution: InstitutionId,
            payment_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(T::InstitutionRegistry::is_institution_active(institution), Error::<T>::InstitutionNotActive);
            ensure!(T::InstitutionRegistry::is_institution_admin(institution, &who), Error::<T>::NotInstitutionAdmin);
            ensure!(!Voted::<T>::get((institution, payment_id), &who), Error::<T>::AlreadyVoted);

            Payments::<T>::try_mutate((institution, payment_id), |maybe| -> DispatchResult {
                let proposal = maybe.as_mut().ok_or(Error::<T>::PaymentNotFound)?;
                ensure!(!proposal.executed, Error::<T>::AlreadyExecuted);
                ensure!(!proposal.rejected, Error::<T>::AlreadyRejected);

                let now = <frame_system::Pallet<T>>::block_number();
                if now > proposal.end {
                    proposal.rejected = true;
                    Self::deposit_event(Event::<T>::PaymentRejectedTimeout { institution, payment_id });
                    return Err(Error::<T>::PaymentTimedOutRejected.into());
                }

                Voted::<T>::insert((institution, payment_id), &who, true);
                proposal.approve_count = proposal.approve_count.saturating_add(1);

                Self::deposit_event(Event::<T>::PaymentApproved {
                    institution,
                    payment_id,
                    who: who.clone(),
                    approve_count: proposal.approve_count,
                });

                if proposal.approve_count >= PAYMENT_PASS_THRESHOLD {
                    let from = T::InstitutionRegistry::institution_account(institution)
                        .ok_or(Error::<T>::InstitutionNotActive)?;

                    T::Currency::transfer(
                        &from,
                        &proposal.to,
                        proposal.amount,
                        frame_support::traits::ExistenceRequirement::KeepAlive,
                    )?;

                    proposal.executed = true;

                    Self::deposit_event(Event::<T>::PaymentExecuted {
                        institution,
                        payment_id,
                        to: proposal.to.clone(),
                        amount: proposal.amount,
                    });
                }

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// 中文注释：供 runtime 手续费提取器预判：本次 approve 是否会触发实际转账。
        pub fn preview_execute_amount(
            who: &T::AccountId,
            institution: InstitutionId,
            payment_id: u64,
        ) -> Result<BalanceOf<T>, DispatchError> {
            ensure!(T::InstitutionRegistry::is_institution_active(institution), Error::<T>::InstitutionNotActive);
            ensure!(T::InstitutionRegistry::is_institution_admin(institution, who), Error::<T>::NotInstitutionAdmin);
            ensure!(!Voted::<T>::get((institution, payment_id), who), Error::<T>::AlreadyVoted);

            let proposal = Payments::<T>::get((institution, payment_id)).ok_or(Error::<T>::PaymentNotFound)?;
            ensure!(!proposal.executed, Error::<T>::AlreadyExecuted);
            ensure!(!proposal.rejected, Error::<T>::AlreadyRejected);

            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now <= proposal.end, Error::<T>::PaymentTimedOutRejected);

            let next_count = proposal.approve_count.saturating_add(1);
            ensure!(next_count >= PAYMENT_PASS_THRESHOLD, Error::<T>::NotEnoughApprovals);
            Ok(proposal.amount)
        }

        pub fn institution_account_of(institution: InstitutionId) -> Option<T::AccountId> {
            T::InstitutionRegistry::institution_account(institution)
        }

        pub fn required_admin_count() -> u32 {
            INSTITUTION_ADMIN_COUNT
        }

        pub fn pass_threshold() -> u32 {
            PAYMENT_PASS_THRESHOLD
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32};
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, BuildStorage};
    use std::{cell::RefCell, thread_local};

    type Balance = u128;
    type Block = frame_system::mocking::MockBlock<Test>;

    thread_local! {
        static ACTIVE: RefCell<bool> = const { RefCell::new(true) };
    }

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
        pub type Balances = pallet_balances;

        #[runtime::pallet_index(2)]
        pub type Payment = pallet;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Nonce = u64;
        type AccountData = pallet_balances::AccountData<Balance>;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = ConstU32<0>;
        type MaxReserves = ConstU32<0>;
        type ReserveIdentifier = [u8; 8];
        type Balance = Balance;
        type RuntimeEvent = RuntimeEvent;
        type DustRemoval = ();
        type ExistentialDeposit = frame_support::traits::ConstU128<1>;
        type AccountStore = System;
        type WeightInfo = ();
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxFreezes = frame_support::traits::VariantCountOf<RuntimeFreezeReason>;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
    }

    pub struct TestRegistry;
    impl InstitutionAccess<u64> for TestRegistry {
        fn institution_account(id: InstitutionId) -> Option<u64> {
            if id == *b"GZGZF000" { Some(100) } else { None }
        }
        fn is_institution_admin(id: InstitutionId, who: &u64) -> bool {
            id == *b"GZGZF000" && matches!(*who, 1 | 2 | 3 | 4 | 5)
        }
        fn is_institution_active(id: InstitutionId) -> bool {
            id == *b"GZGZF000" && ACTIVE.with(|v| *v.borrow())
        }
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type InstitutionRegistry = TestRegistry;
        type MaxMemoLen = ConstU32<64>;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("build storage should succeed");
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(100, 10_000), (200, 1), (1, 100), (2, 100), (3, 100)],
            dev_accounts: None,
        }
        .assimilate_storage(&mut storage)
        .expect("balances genesis should assimilate");
        storage.into()
    }

    #[test]
    fn payment_executes_on_third_signature() {
        new_test_ext().execute_with(|| {
            let memo: BoundedVec<u8, ConstU32<64>> = b"pay".to_vec().try_into().expect("fit");
            assert_ok!(Payment::propose_payment(
                RuntimeOrigin::signed(1),
                *b"GZGZF000",
                200,
                1000,
                memo,
            ));

            assert_ok!(Payment::approve_payment(RuntimeOrigin::signed(2), *b"GZGZF000", 0));
            assert_eq!(Balances::free_balance(100), 10_000);

            assert_ok!(Payment::approve_payment(RuntimeOrigin::signed(3), *b"GZGZF000", 0));
            assert_eq!(Balances::free_balance(100), 9_000);
            assert_eq!(Balances::free_balance(200), 1_001);
        });
    }

    #[test]
    fn timeout_rejects() {
        new_test_ext().execute_with(|| {
            let memo: BoundedVec<u8, ConstU32<64>> = b"pay".to_vec().try_into().expect("fit");
            assert_ok!(Payment::propose_payment(RuntimeOrigin::signed(1), *b"GZGZF000", 200, 1000, memo));
            frame_system::Pallet::<Test>::set_block_number(primitives::count_const::VOTING_DURATION_BLOCKS as u64 + 2);
            assert_noop!(
                Payment::approve_payment(RuntimeOrigin::signed(2), *b"GZGZF000", 0),
                Error::<Test>::PaymentTimedOutRejected
            );
        });
    }
}
