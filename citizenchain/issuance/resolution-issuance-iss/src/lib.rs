#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

use alloc::vec::Vec;
use frame_support::dispatch::DispatchResult;
use frame_support::pallet_prelude::StorageVersion;
use frame_support::weights::{constants::RocksDbWeight, Weight};
use sp_runtime::DispatchError;

pub trait ResolutionIssuanceExecutor<AccountId, Amount> {
    fn execute_resolution_issuance(
        proposal_id: u64,
        reason: Vec<u8>,
        total_amount: Amount,
        allocations: Vec<(AccountId, Amount)>,
    ) -> DispatchResult;
}

impl<AccountId, Amount> ResolutionIssuanceExecutor<AccountId, Amount> for () {
    fn execute_resolution_issuance(
        _proposal_id: u64,
        _reason: Vec<u8>,
        _total_amount: Amount,
        _allocations: Vec<(AccountId, Amount)>,
    ) -> DispatchResult {
        Err(DispatchError::Other(
            "ResolutionIssuanceExecutorNotConfigured",
        ))
    }
}

pub trait WeightInfo {
    fn execute_resolution_issuance(reason_len: u32, allocation_count: u32) -> Weight;
    fn clear_executed() -> Weight;
    fn set_paused() -> Weight;
}

impl WeightInfo for () {
    fn execute_resolution_issuance(reason_len: u32, allocation_count: u32) -> Weight {
        let reason_len = reason_len as u64;
        let allocation_count = allocation_count as u64;
        Weight::from_parts(120_000_000, 4_096)
            .saturating_add(Weight::from_parts(20_000_000, 256).saturating_mul(allocation_count))
            .saturating_add(Weight::from_parts(40_000, 1).saturating_mul(reason_len))
            .saturating_add(
                RocksDbWeight::get().reads_writes(4 + allocation_count, 5 + allocation_count),
            )
    }

    fn clear_executed() -> Weight {
        Weight::from_parts(10_000_000, 128)
            .saturating_add(RocksDbWeight::get().reads_writes(1, 2))
    }

    fn set_paused() -> Weight {
        Weight::from_parts(5_000_000, 64)
            .saturating_add(RocksDbWeight::get().reads_writes(1, 2))
    }
}

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        storage::with_storage_layer,
        traits::{Currency, Imbalance},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{CheckedAdd, Hash, Zero};

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type AllocationOf<T> = BoundedVec<
        RecipientAmount<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        <T as Config>::MaxAllocations,
    >;

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
    pub struct RecipientAmount<AccountId, Balance> {
        pub recipient: AccountId,
        pub amount: Balance,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        /// 仅允许治理模块（resolution-issuance-gov）触发执行。
        type ExecuteOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// 维护入口：用于执行记录清理等运维动作。
        type MaintenanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        #[pallet::constant]
        type MaxReasonLen: Get<u32>;

        #[pallet::constant]
        type MaxAllocations: Get<u32>;

        #[pallet::constant]
        type MaxTotalIssuance: Get<BalanceOf<Self>>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// proposal_id 是否已经执行，用于防重放。
    #[pallet::storage]
    pub type Executed<T: Config> = StorageMap<_, Twox64Concat, u64, BlockNumberFor<T>, OptionQuery>;

    /// proposal_id 是否历史上执行过（永久防重放）。
    #[pallet::storage]
    pub type EverExecuted<T: Config> = StorageMap<_, Twox64Concat, u64, (), OptionQuery>;

    /// 决议发行累计执行量（用于审计）。
    #[pallet::storage]
    pub type TotalIssued<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type Paused<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ResolutionIssuanceExecuted {
            proposal_id: u64,
            total_amount: BalanceOf<T>,
            recipient_count: u32,
            reason_hash: T::Hash,
            allocations_hash: T::Hash,
        },
        ExecutedCleared {
            proposal_id: u64,
        },
        PausedSet {
            paused: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        AlreadyExecuted,
        AlreadyInState,
        EmptyReason,
        EmptyAllocations,
        TooManyAllocations,
        ZeroAmount,
        AllocationOverflow,
        TotalMismatch,
        TotalIssuedOverflow,
        ReasonTooLong,
        BelowExistentialDeposit,
        DepositFailed,
        ExceedsTotalIssuanceCap,
        NotExecuted,
        PalletPaused,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        #[cfg(feature = "std")]
        fn integrity_test() {
            assert!(
                T::MaxAllocations::get() > 0,
                "MaxAllocations must be greater than 0"
            );
            assert!(
                !T::MaxTotalIssuance::get().is_zero(),
                "MaxTotalIssuance must be greater than 0"
            );
            assert!(
                T::MaxReasonLen::get() > 0,
                "MaxReasonLen must be greater than 0"
            );
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 决议发行执行：治理模块通过后调用本函数执行铸币。
        /// 注意：本模块不处理提案/投票，仅负责执行。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::execute_resolution_issuance(reason.len() as u32, allocations.len() as u32))]
        pub fn execute_resolution_issuance(
            origin: OriginFor<T>,
            proposal_id: u64,
            reason: ReasonOf<T>,
            total_amount: BalanceOf<T>,
            allocations: AllocationOf<T>,
        ) -> DispatchResult {
            T::ExecuteOrigin::ensure_origin(origin)?;
            Self::do_execute(
                proposal_id,
                reason.as_slice(),
                total_amount,
                allocations.as_slice(),
            )
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::clear_executed())]
        pub fn clear_executed(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            T::MaintenanceOrigin::ensure_origin(origin)?;
            ensure!(
                Executed::<T>::contains_key(proposal_id),
                Error::<T>::NotExecuted
            );
            Executed::<T>::remove(proposal_id);
            Self::deposit_event(Event::<T>::ExecutedCleared { proposal_id });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::set_paused())]
        pub fn set_paused(origin: OriginFor<T>, paused: bool) -> DispatchResult {
            T::MaintenanceOrigin::ensure_origin(origin)?;
            ensure!(Paused::<T>::get() != paused, Error::<T>::AlreadyInState);
            Paused::<T>::put(paused);
            Self::deposit_event(Event::<T>::PausedSet { paused });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn do_execute(
            proposal_id: u64,
            reason: &[u8],
            total_amount: BalanceOf<T>,
            allocations: &[RecipientAmount<T::AccountId, BalanceOf<T>>],
        ) -> DispatchResult {
            with_storage_layer(|| {
                Self::do_execute_inner(proposal_id, reason, total_amount, allocations)
            })
        }

        fn do_execute_inner(
            proposal_id: u64,
            reason: &[u8],
            total_amount: BalanceOf<T>,
            allocations: &[RecipientAmount<T::AccountId, BalanceOf<T>>],
        ) -> DispatchResult {
            ensure!(!Paused::<T>::get(), Error::<T>::PalletPaused);
            ensure!(
                !EverExecuted::<T>::contains_key(proposal_id),
                Error::<T>::AlreadyExecuted
            );
            ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
            ensure!(
                reason.len() <= T::MaxReasonLen::get() as usize,
                Error::<T>::ReasonTooLong
            );
            ensure!(!allocations.is_empty(), Error::<T>::EmptyAllocations);
            ensure!(
                allocations.len() <= T::MaxAllocations::get() as usize,
                Error::<T>::TooManyAllocations
            );

            let existential_deposit = T::Currency::minimum_balance();
            let mut sum: BalanceOf<T> = Zero::zero();
            for item in allocations {
                ensure!(!item.amount.is_zero(), Error::<T>::ZeroAmount);
                ensure!(
                    item.amount >= existential_deposit,
                    Error::<T>::BelowExistentialDeposit
                );
                sum = sum
                    .checked_add(&item.amount)
                    .ok_or(Error::<T>::AllocationOverflow)?;
            }

            ensure!(sum == total_amount, Error::<T>::TotalMismatch);

            // 中文注释：先做累计量溢出校验，再执行发币，避免出现“先发币后报错”的不一致风险。
            let new_total = TotalIssued::<T>::get()
                .checked_add(&total_amount)
                .ok_or(Error::<T>::TotalIssuedOverflow)?;
            ensure!(
                new_total <= T::MaxTotalIssuance::get(),
                Error::<T>::ExceedsTotalIssuanceCap
            );

            let mut total_imbalance =
                <<T as Config>::Currency as Currency<T::AccountId>>::PositiveImbalance::zero();
            for item in allocations {
                let imbalance = T::Currency::deposit_creating(&item.recipient, item.amount);
                ensure!(imbalance.peek() == item.amount, Error::<T>::DepositFailed);
                total_imbalance.subsume(imbalance);
            }
            drop(total_imbalance);

            let current_block = frame_system::Pallet::<T>::block_number();
            EverExecuted::<T>::insert(proposal_id, ());
            Executed::<T>::insert(proposal_id, current_block);
            TotalIssued::<T>::put(new_total);

            let reason_hash = T::Hashing::hash(reason);
            let allocations_hash = T::Hashing::hash_of(&allocations);
            Self::deposit_event(Event::<T>::ResolutionIssuanceExecuted {
                proposal_id,
                total_amount,
                recipient_count: allocations.len() as u32,
                reason_hash,
                allocations_hash,
            });

            Ok(())
        }
    }

    impl<T: Config> ResolutionIssuanceExecutor<T::AccountId, BalanceOf<T>> for Pallet<T> {
        fn execute_resolution_issuance(
            proposal_id: u64,
            reason: Vec<u8>,
            total_amount: BalanceOf<T>,
            allocations: Vec<(T::AccountId, BalanceOf<T>)>,
        ) -> DispatchResult {
            let mapped: Vec<RecipientAmount<T::AccountId, BalanceOf<T>>> = allocations
                .into_iter()
                .map(|(recipient, amount)| RecipientAmount { recipient, amount })
                .collect();

            Self::do_execute(
                proposal_id,
                reason.as_slice(),
                total_amount,
                mapped.as_slice(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        assert_noop, assert_ok, derive_impl,
        traits::{ConstU128, ConstU32, Currency},
    };
    use frame_system as system;
    use sp_runtime::{
        traits::{Hash as HashT, IdentityLookup},
        BuildStorage,
    };

    type AccountId = u64;
    type Balance = u128;
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
        pub type Balances = pallet_balances;

        #[runtime::pallet_index(2)]
        pub type ResolutionIssuanceIss = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type AccountData = pallet_balances::AccountData<Balance>;
    }

    impl pallet_balances::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<10>;
        type AccountStore = System;
        type MaxLocks = ConstU32<0>;
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxFreezes = ConstU32<0>;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
        type WeightInfo = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type ExecuteOrigin = frame_system::EnsureRoot<AccountId>;
        type MaintenanceOrigin = frame_system::EnsureRoot<AccountId>;
        type MaxReasonLen = ConstU32<128>;
        type MaxAllocations = ConstU32<4>;
        type MaxTotalIssuance = ConstU128<1_000_000>;
        type WeightInfo = ();
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");
        storage.into()
    }

    fn alloc(recipient: AccountId, amount: u128) -> pallet::RecipientAmount<AccountId, Balance> {
        pallet::RecipientAmount { recipient, amount }
    }

    #[test]
    fn execute_via_trait_updates_balances_and_markers() {
        new_test_ext().execute_with(|| {
            let allocations = vec![(10, 30), (20, 70)];
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                1, b"ok".to_vec(), 100, allocations
            ));

            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(10), 30);
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(20), 70);
            assert_eq!(pallet::Executed::<Test>::get(1), Some(0));
            assert!(pallet::EverExecuted::<Test>::contains_key(1));
            assert_eq!(pallet::TotalIssued::<Test>::get(), 100);
        });
    }

    #[test]
    fn replay_is_rejected() {
        new_test_ext().execute_with(|| {
            let allocations = vec![(10, 100)];
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                2,
                b"a".to_vec(),
                100,
                allocations.clone()
            ));
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(
                    2, b"b".to_vec(), 100, allocations
                ),
                pallet::Error::<Test>::AlreadyExecuted
            );
        });
    }

    #[test]
    fn total_mismatch_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(
                    3, b"x".to_vec(), 100, vec![(10, 40), (20, 50)]
                ),
                pallet::Error::<Test>::TotalMismatch
            );
        });
    }

    #[test]
    fn zero_amount_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(
                    16,
                    b"x".to_vec(),
                    20,
                    vec![(10, 20), (20, 0)]
                ),
                pallet::Error::<Test>::ZeroAmount
            );
        });
    }

    #[test]
    fn empty_allocations_via_trait_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(17, b"x".to_vec(), 0, vec![]),
                pallet::Error::<Test>::EmptyAllocations
            );
        });
    }

    #[test]
    fn too_many_allocations_is_rejected() {
        new_test_ext().execute_with(|| {
            let many = vec![(1, 10), (2, 10), (3, 10), (4, 10), (5, 10)];
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(
                    4, b"x".to_vec(), 50, many
                ),
                pallet::Error::<Test>::TooManyAllocations
            );
        });
    }

    #[test]
    fn allocation_sum_overflow_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(
                    21,
                    b"x".to_vec(),
                    u128::MAX,
                    vec![(10, u128::MAX), (20, 10)]
                ),
                pallet::Error::<Test>::AllocationOverflow
            );
        });
    }

    #[test]
    fn overflow_is_rejected_before_mint() {
        new_test_ext().execute_with(|| {
            pallet::TotalIssued::<Test>::put(u128::MAX - 5);
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(
                    5, b"x".to_vec(), 10, vec![(10, 10)]
                ),
                pallet::Error::<Test>::TotalIssuedOverflow
            );

            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(10), 0);
            assert_eq!(pallet::Executed::<Test>::get(5), None);
            assert_eq!(pallet::TotalIssued::<Test>::get(), u128::MAX - 5);
        });
    }

    #[test]
    fn trait_path_rejects_reason_too_long() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(
                    7, vec![b'x'; 129], 10, vec![(10, 10)]
                ),
                pallet::Error::<Test>::ReasonTooLong
            );
        });
    }

    #[test]
    fn empty_reason_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(
                    26, vec![], 10, vec![(10, 10)]
                ),
                pallet::Error::<Test>::EmptyReason
            );
        });
    }

    #[test]
    fn cap_exceeded_is_rejected_before_mint() {
        new_test_ext().execute_with(|| {
            pallet::TotalIssued::<Test>::put(1_000_000 - 5);
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(
                    8, b"x".to_vec(), 10, vec![(10, 10)]
                ),
                pallet::Error::<Test>::ExceedsTotalIssuanceCap
            );

            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(10), 0);
            assert_eq!(pallet::Executed::<Test>::get(8), None);
            assert_eq!(pallet::TotalIssued::<Test>::get(), 1_000_000 - 5);
        });
    }

    #[test]
    fn clear_executed_requires_existing_key() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                pallet::Pallet::<Test>::clear_executed(RuntimeOrigin::root(), 99),
                pallet::Error::<Test>::NotExecuted
            );
        });
    }

    #[test]
    fn clear_executed_removes_marker() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                9, b"ok".to_vec(), 10, vec![(10, 10)]
            ));
            assert_eq!(pallet::Executed::<Test>::get(9), Some(0));

            assert_ok!(pallet::Pallet::<Test>::clear_executed(
                RuntimeOrigin::root(),
                9
            ));
            assert_eq!(pallet::Executed::<Test>::get(9), None);
        });
    }

    #[test]
    fn non_root_cannot_clear_executed() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                pallet::Pallet::<Test>::clear_executed(RuntimeOrigin::signed(1), 9),
                sp_runtime::DispatchError::BadOrigin
            );
            assert_noop!(
                pallet::Pallet::<Test>::set_paused(RuntimeOrigin::signed(1), true),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn clear_executed_emits_event() {
        new_test_ext().execute_with(|| {
            frame_system::Pallet::<Test>::set_block_number(1);
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                22, b"ok".to_vec(), 10, vec![(10, 10)]
            ));

            assert_ok!(pallet::Pallet::<Test>::clear_executed(
                RuntimeOrigin::root(),
                22
            ));
            let last_event = frame_system::Pallet::<Test>::events()
                .last()
                .expect("event should exist")
                .event
                .clone();
            assert_eq!(
                last_event,
                RuntimeEvent::ResolutionIssuanceIss(pallet::Event::<Test>::ExecutedCleared {
                    proposal_id: 22
                })
            );
        });
    }

    #[test]
    fn clear_executed_works_while_paused() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                30, b"ok".to_vec(), 10, vec![(10, 10)]
            ));
            assert_ok!(pallet::Pallet::<Test>::set_paused(
                RuntimeOrigin::root(),
                true
            ));
            assert_ok!(pallet::Pallet::<Test>::clear_executed(
                RuntimeOrigin::root(),
                30
            ));
            assert_eq!(pallet::Executed::<Test>::get(30), None);
        });
    }

    #[test]
    fn clear_executed_does_not_affect_other_proposals() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                31, b"ok".to_vec(), 10, vec![(10, 10)]
            ));
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                32, b"ok".to_vec(), 10, vec![(20, 10)]
            ));

            assert_ok!(pallet::Pallet::<Test>::clear_executed(
                RuntimeOrigin::root(),
                31
            ));
            assert_eq!(pallet::Executed::<Test>::get(31), None);
            assert_eq!(pallet::Executed::<Test>::get(32), Some(0));
        });
    }

    #[test]
    fn paused_blocks_execution_and_can_resume() {
        new_test_ext().execute_with(|| {
            assert_ok!(pallet::Pallet::<Test>::set_paused(
                RuntimeOrigin::root(),
                true
            ));
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(23, b"ok".to_vec(), 10, vec![(10, 10)]),
                pallet::Error::<Test>::PalletPaused
            );
            assert_ok!(pallet::Pallet::<Test>::set_paused(
                RuntimeOrigin::root(),
                false
            ));
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                23, b"ok".to_vec(), 10, vec![(10, 10)]
            ));
        });
    }

    #[test]
    fn set_paused_emits_event() {
        new_test_ext().execute_with(|| {
            frame_system::Pallet::<Test>::set_block_number(1);
            assert_ok!(pallet::Pallet::<Test>::set_paused(
                RuntimeOrigin::root(),
                true
            ));
            let last_event = frame_system::Pallet::<Test>::events()
                .last()
                .expect("event should exist")
                .event
                .clone();
            assert_eq!(
                last_event,
                RuntimeEvent::ResolutionIssuanceIss(
                    pallet::Event::<Test>::PausedSet { paused: true }
                )
            );
        });
    }

    #[test]
    fn set_paused_same_state_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_ok!(pallet::Pallet::<Test>::set_paused(
                RuntimeOrigin::root(),
                true
            ));
            assert_noop!(
                pallet::Pallet::<Test>::set_paused(RuntimeOrigin::root(), true),
                pallet::Error::<Test>::AlreadyInState
            );
        });
    }

    #[test]
    fn paused_has_priority_over_already_executed() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                33, b"ok".to_vec(), 10, vec![(10, 10)]
            ));
            assert_ok!(pallet::Pallet::<Test>::set_paused(
                RuntimeOrigin::root(),
                true
            ));
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(33, b"ok".to_vec(), 10, vec![(10, 10)]),
                pallet::Error::<Test>::PalletPaused
            );
        });
    }

    #[test]
    fn deposit_failure_is_rejected() {
        new_test_ext().execute_with(|| {
            let _ = pallet_balances::Pallet::<Test>::deposit_creating(&10, u128::MAX);
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(24, b"ok".to_vec(), 10, vec![(10, 10)]),
                pallet::Error::<Test>::DepositFailed
            );
        });
    }

    #[test]
    fn clear_executed_does_not_allow_replay() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                10, b"ok".to_vec(), 20, vec![(10, 20)]
            ));
            assert_ok!(pallet::Pallet::<Test>::clear_executed(
                RuntimeOrigin::root(),
                10
            ));

            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(10, b"ok".to_vec(), 20, vec![(10, 20)]),
                pallet::Error::<Test>::AlreadyExecuted
            );
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(10), 20);
            assert_eq!(pallet::TotalIssued::<Test>::get(), 20);
        });
    }

    #[test]
    fn amount_below_ed_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(11, b"ok".to_vec(), 9, vec![(10, 9)]),
                pallet::Error::<Test>::BelowExistentialDeposit
            );
        });
    }

    #[test]
    fn prevalidation_prevents_partial_mint_on_ed_error() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                    AccountId,
                    Balance,
                >>::execute_resolution_issuance(12, b"ok".to_vec(), 25, vec![(10, 20), (20, 5)]),
                pallet::Error::<Test>::BelowExistentialDeposit
            );
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(10), 0);
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(20), 0);
            assert_eq!(pallet::TotalIssued::<Test>::get(), 0);
        });
    }

    #[test]
    fn duplicate_recipient_allocations_accumulate() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                13,
                b"ok".to_vec(),
                100,
                vec![(10, 40), (10, 60)]
            ));
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(10), 100);
            assert_eq!(pallet::TotalIssued::<Test>::get(), 100);
        });
    }

    #[test]
    fn total_issued_accumulates_across_proposals() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                14, b"a".to_vec(), 20, vec![(10, 20)]
            ));
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                15, b"b".to_vec(), 30, vec![(20, 30)]
            ));

            assert_eq!(pallet::TotalIssued::<Test>::get(), 50);
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(10), 20);
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(20), 30);
        });
    }

    #[test]
    fn max_allocations_boundary_passes() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                18,
                b"ok".to_vec(),
                40,
                vec![(10, 10), (20, 10), (30, 10), (40, 10)]
            ));
            assert_eq!(pallet::TotalIssued::<Test>::get(), 40);
        });
    }

    #[test]
    fn cap_boundary_exactly_reached_is_allowed() {
        new_test_ext().execute_with(|| {
            pallet::TotalIssued::<Test>::put(1_000_000 - 20);
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                19, b"ok".to_vec(), 20, vec![(10, 20)]
            ));
            assert_eq!(pallet::TotalIssued::<Test>::get(), 1_000_000);
        });
    }

    #[test]
    fn event_fields_are_emitted_correctly() {
        new_test_ext().execute_with(|| {
            frame_system::Pallet::<Test>::set_block_number(1);
            let reason = b"audit".to_vec();
            let allocations = vec![(10, 20), (20, 30)];
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                20,
                reason.clone(),
                50,
                allocations.clone()
            ));

            let reason_hash = <Test as frame_system::Config>::Hashing::hash(reason.as_slice());
            let mapped: Vec<pallet::RecipientAmount<AccountId, Balance>> = allocations
                .into_iter()
                .map(|(recipient, amount)| pallet::RecipientAmount { recipient, amount })
                .collect();
            let allocations_hash = <Test as frame_system::Config>::Hashing::hash_of(&mapped);

            let last_event = frame_system::Pallet::<Test>::events()
                .last()
                .expect("event should exist")
                .event
                .clone();
            assert_eq!(Executed::<Test>::get(20), Some(1));
            assert_eq!(
                last_event,
                RuntimeEvent::ResolutionIssuanceIss(
                    pallet::Event::<Test>::ResolutionIssuanceExecuted {
                        proposal_id: 20,
                        total_amount: 50,
                        recipient_count: 2,
                        reason_hash,
                        allocations_hash,
                    }
                )
            );
        });
    }

    #[test]
    fn reason_exactly_at_max_len_passes() {
        new_test_ext().execute_with(|| {
            let reason = vec![b'x'; 128];
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<
                AccountId,
                Balance,
            >>::execute_resolution_issuance(
                25, reason, 10, vec![(10, 10)]
            ));
        });
    }

    #[test]
    fn extrinsic_requires_root_origin() {
        new_test_ext().execute_with(|| {
            let reason: pallet::ReasonOf<Test> = b"rs".to_vec().try_into().expect("fit");
            let allocations: pallet::AllocationOf<Test> =
                vec![alloc(10, 10)].try_into().expect("fit");

            assert_noop!(
                pallet::Pallet::<Test>::execute_resolution_issuance(
                    RuntimeOrigin::signed(1),
                    6,
                    reason.clone(),
                    10,
                    allocations.clone()
                ),
                sp_runtime::DispatchError::BadOrigin
            );

            assert_ok!(pallet::Pallet::<Test>::execute_resolution_issuance(
                RuntimeOrigin::root(),
                6,
                reason,
                10,
                allocations
            ));
        });
    }

    #[test]
    fn extrinsic_path_is_blocked_when_paused() {
        new_test_ext().execute_with(|| {
            let reason: pallet::ReasonOf<Test> = b"rs".to_vec().try_into().expect("fit");
            let allocations: pallet::AllocationOf<Test> =
                vec![alloc(10, 10)].try_into().expect("fit");
            assert_ok!(pallet::Pallet::<Test>::set_paused(
                RuntimeOrigin::root(),
                true
            ));
            assert_noop!(
                pallet::Pallet::<Test>::execute_resolution_issuance(
                    RuntimeOrigin::root(),
                    34,
                    reason,
                    10,
                    allocations
                ),
                pallet::Error::<Test>::PalletPaused
            );
        });
    }
}
