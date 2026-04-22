//! 清算行(L2)扫码支付清算 pallet 权重。
//!
//! Step 2b-iv-b 清理后,老省储行 `submit_offchain_batch` / `enqueue_offchain_batch`
//! / `process_queued_batch` 的权重方法已随对应 Call 一并删除。当前 pallet 的 call
//! 权重全部在 `lib.rs` 的 `#[pallet::weight]` 里用 `T::DbWeight` 直接估算,本文件
//! 保留**空 trait + 空实现**,作为后续接入 `frame-benchmarking` 自动生成时的锚点。

use frame_support::weights::Weight;

/// 权重接口(保留空壳,为将来 benchmark 扩展预留)。
pub trait WeightInfo {
    /// 占位:后续按批次大小生成 `submit_offchain_batch_v2` 权重。
    fn submit_offchain_batch_v2(_items: u32) -> Weight {
        Weight::zero()
    }
}

impl WeightInfo for () {}
