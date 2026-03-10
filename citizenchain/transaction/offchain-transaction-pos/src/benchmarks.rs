//! 链下交易手续费模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_support::{traits::UnfilteredDispatchable, BoundedVec};
use frame_system::RawOrigin;
use voting_engine_system::InstitutionPalletId;

use crate::{BatchOf, BatchSignatureOf, Call, Config, Pallet};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_offchain_batch() {
        let submitter: T::AccountId = frame_benchmarking::account("submitter", 0, 0);
        let institution: InstitutionPalletId = [0u8; 48];
        let batch: BatchOf<T> = BoundedVec::default();
        let signature: BatchSignatureOf<T> = BoundedVec::default();

        #[block]
        {
            let call = Call::<T>::submit_offchain_batch {
                institution,
                batch_seq: 1u64,
                batch: batch.clone(),
                batch_signature: signature.clone(),
            };
            assert!(call
                .dispatch_bypass_filter(RawOrigin::Signed(submitter.clone()).into())
                .is_err());
        }
    }

    #[benchmark]
    fn enqueue_offchain_batch() {
        let submitter: T::AccountId = frame_benchmarking::account("submitter", 1, 0);
        let institution: InstitutionPalletId = [0u8; 48];
        let batch: BatchOf<T> = BoundedVec::default();
        let signature: BatchSignatureOf<T> = BoundedVec::default();

        #[block]
        {
            let call = Call::<T>::enqueue_offchain_batch {
                institution,
                batch_seq: 1u64,
                batch: batch.clone(),
                batch_signature: signature.clone(),
            };
            assert!(call
                .dispatch_bypass_filter(RawOrigin::Signed(submitter.clone()).into())
                .is_err());
        }
    }

    #[benchmark]
    fn process_queued_batch() {
        let submitter: T::AccountId = frame_benchmarking::account("submitter", 2, 0);

        #[block]
        {
            let call = Call::<T>::process_queued_batch { queue_id: 1u64 };
            assert!(call
                .dispatch_bypass_filter(RawOrigin::Signed(submitter.clone()).into())
                .is_err());
        }
    }
}
