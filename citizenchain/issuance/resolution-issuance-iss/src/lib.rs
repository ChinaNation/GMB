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

            let mut sum = 0u128;
            for item in allocations {
                ensure!(item.amount > 0, Error::<T>::ZeroAmount);
                sum = sum
                    .checked_add(item.amount)
                    .ok_or(Error::<T>::AllocationOverflow)?;
            }

            ensure!(sum == total_amount, Error::<T>::TotalMismatch);

            for item in allocations {
                let amount: BalanceOf<T> = item.amount.saturated_into();
                let _imbalance = T::Currency::deposit_creating(&item.recipient, amount);
            }

            Executed::<T>::insert(proposal_id, true);

            let new_total = TotalIssued::<T>::get()
                .checked_add(total_amount)
                .ok_or(Error::<T>::TotalIssuedOverflow)?;
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
