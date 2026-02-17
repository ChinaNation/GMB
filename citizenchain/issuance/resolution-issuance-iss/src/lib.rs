#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

pub trait ResolutionIssuanceExecutor<AccountId> {
    fn execute_resolution_issuance(
        proposal_id: u64,
        reason: Vec<u8>,
        total_amount: u128,
        allocations: Vec<(AccountId, u128)>,
    ) -> DispatchResult;
}

impl<AccountId> ResolutionIssuanceExecutor<AccountId> for () {
    fn execute_resolution_issuance(
        _proposal_id: u64,
        _reason: Vec<u8>,
        _total_amount: u128,
        _allocations: Vec<(AccountId, u128)>,
    ) -> DispatchResult {
        Err(DispatchError::Other(
            "ResolutionIssuanceExecutorNotConfigured",
        ))
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::Currency};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Hash, SaturatedConversion};

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type AllocationOf<T> = BoundedVec<
        RecipientAmount<<T as frame_system::Config>::AccountId>,
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
    pub struct RecipientAmount<AccountId> {
        pub recipient: AccountId,
        pub amount: u128,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        /// 仅允许治理模块（resolution-issuance-gov）触发执行。
        type ExecuteOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        #[pallet::constant]
        type MaxReasonLen: Get<u32>;

        #[pallet::constant]
        type MaxAllocations: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// proposal_id 是否已经执行，用于防重放。
    #[pallet::storage]
    #[pallet::getter(fn executed)]
    pub type Executed<T> = StorageMap<_, Blake2_128Concat, u64, bool, ValueQuery>;

    /// 决议发行累计执行量（用于审计）。
    #[pallet::storage]
    #[pallet::getter(fn total_issued)]
    pub type TotalIssued<T> = StorageValue<_, u128, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ResolutionIssuanceExecuted {
            proposal_id: u64,
            total_amount: u128,
            recipient_count: u32,
            reason_hash: T::Hash,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        AlreadyExecuted,
        EmptyAllocations,
        TooManyAllocations,
        ZeroAmount,
        AllocationOverflow,
        TotalMismatch,
        TotalIssuedOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 决议发行执行：治理模块通过后调用本函数执行铸币。
        /// 注意：本模块不处理提案/投票，仅负责执行。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2 + T::MaxAllocations::get() as u64))]
        pub fn execute_resolution_issuance(
            origin: OriginFor<T>,
            proposal_id: u64,
            reason: ReasonOf<T>,
            total_amount: u128,
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
    }

    impl<T: Config> Pallet<T> {
        fn do_execute(
            proposal_id: u64,
            reason: &[u8],
            total_amount: u128,
            allocations: &[RecipientAmount<T::AccountId>],
        ) -> DispatchResult {
            ensure!(
                !Executed::<T>::get(proposal_id),
                Error::<T>::AlreadyExecuted
            );
            ensure!(!allocations.is_empty(), Error::<T>::EmptyAllocations);
            ensure!(
                allocations.len() <= T::MaxAllocations::get() as usize,
                Error::<T>::TooManyAllocations
            );

            let mut sum = 0u128;
            for item in allocations {
                ensure!(item.amount > 0, Error::<T>::ZeroAmount);
                sum = sum
                    .checked_add(item.amount)
                    .ok_or(Error::<T>::AllocationOverflow)?;
            }

            ensure!(sum == total_amount, Error::<T>::TotalMismatch);

            // 中文注释：先做累计量溢出校验，再执行发币，避免出现“先发币后报错”的不一致风险。
            let new_total = TotalIssued::<T>::get()
                .checked_add(total_amount)
                .ok_or(Error::<T>::TotalIssuedOverflow)?;

            for item in allocations {
                let amount: BalanceOf<T> = item.amount.saturated_into();
                let _imbalance = T::Currency::deposit_creating(&item.recipient, amount);
            }

            Executed::<T>::insert(proposal_id, true);
            TotalIssued::<T>::put(new_total);

            let reason_hash = T::Hashing::hash(reason);
            Self::deposit_event(Event::<T>::ResolutionIssuanceExecuted {
                proposal_id,
                total_amount,
                recipient_count: allocations.len() as u32,
                reason_hash,
            });

            Ok(())
        }
    }

    impl<T: Config> ResolutionIssuanceExecutor<T::AccountId> for Pallet<T> {
        fn execute_resolution_issuance(
            proposal_id: u64,
            reason: Vec<u8>,
            total_amount: u128,
            allocations: Vec<(T::AccountId, u128)>,
        ) -> DispatchResult {
            let mapped: Vec<RecipientAmount<T::AccountId>> = allocations
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
    use frame_support::{assert_noop, assert_ok, derive_impl, traits::{ConstU128, ConstU32}};
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, BuildStorage};

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
        type ExistentialDeposit = ConstU128<1>;
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
        type MaxReasonLen = ConstU32<128>;
        type MaxAllocations = ConstU32<4>;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");
        storage.into()
    }

    fn alloc(recipient: AccountId, amount: u128) -> pallet::RecipientAmount<AccountId> {
        pallet::RecipientAmount { recipient, amount }
    }

    #[test]
    fn execute_via_trait_updates_balances_and_markers() {
        new_test_ext().execute_with(|| {
            let allocations = vec![(10, 30), (20, 70)];
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<AccountId>>::execute_resolution_issuance(
                1,
                b"ok".to_vec(),
                100,
                allocations
            ));

            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(10), 30);
            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(20), 70);
            assert!(pallet::Executed::<Test>::get(1));
            assert_eq!(pallet::TotalIssued::<Test>::get(), 100);
        });
    }

    #[test]
    fn replay_is_rejected() {
        new_test_ext().execute_with(|| {
            let allocations = vec![(10, 100)];
            assert_ok!(<pallet::Pallet<Test> as ResolutionIssuanceExecutor<AccountId>>::execute_resolution_issuance(
                2,
                b"a".to_vec(),
                100,
                allocations.clone()
            ));
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<AccountId>>::execute_resolution_issuance(
                    2,
                    b"b".to_vec(),
                    100,
                    allocations
                ),
                pallet::Error::<Test>::AlreadyExecuted
            );
        });
    }

    #[test]
    fn total_mismatch_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<AccountId>>::execute_resolution_issuance(
                    3,
                    b"x".to_vec(),
                    100,
                    vec![(10, 40), (20, 50)]
                ),
                pallet::Error::<Test>::TotalMismatch
            );
        });
    }

    #[test]
    fn too_many_allocations_is_rejected() {
        new_test_ext().execute_with(|| {
            let many = vec![(1, 1), (2, 1), (3, 1), (4, 1), (5, 1)];
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<AccountId>>::execute_resolution_issuance(
                    4,
                    b"x".to_vec(),
                    5,
                    many
                ),
                pallet::Error::<Test>::TooManyAllocations
            );
        });
    }

    #[test]
    fn overflow_is_rejected_before_mint() {
        new_test_ext().execute_with(|| {
            pallet::TotalIssued::<Test>::put(u128::MAX - 5);
            assert_noop!(
                <pallet::Pallet<Test> as ResolutionIssuanceExecutor<AccountId>>::execute_resolution_issuance(
                    5,
                    b"x".to_vec(),
                    10,
                    vec![(10, 10)]
                ),
                pallet::Error::<Test>::TotalIssuedOverflow
            );

            assert_eq!(pallet_balances::Pallet::<Test>::free_balance(10), 0);
            assert!(!pallet::Executed::<Test>::get(5));
            assert_eq!(pallet::TotalIssued::<Test>::get(), u128::MAX - 5);
        });
    }

    #[test]
    fn extrinsic_requires_root_origin() {
        new_test_ext().execute_with(|| {
            let reason: pallet::ReasonOf<Test> = b"rs".to_vec().try_into().expect("fit");
            let allocations: pallet::AllocationOf<Test> = vec![alloc(10, 10)]
                .try_into()
                .expect("fit");

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
}
