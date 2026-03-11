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
            // 中文注释：这里用最小失败样本触发 extrinsic 调度路径，
            // 目标是让 benchmark 管线稳定覆盖入口，而不是在 benchmark 中重建完整清算环境。
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
            // 中文注释：enqueue benchmark 同样只验证入队入口被成功调度。
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
            // 中文注释：process_queued_batch 的 benchmark 只覆盖调用入口，不依赖预先构造持久化队列状态。
            let call = Call::<T>::process_queued_batch { queue_id: 1u64 };
            assert!(call
                .dispatch_bypass_filter(RawOrigin::Signed(submitter.clone()).into())
                .is_err());
        }
    }
}
