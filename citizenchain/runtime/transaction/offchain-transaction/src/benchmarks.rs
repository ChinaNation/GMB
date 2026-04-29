//! 清算行(L2)扫码支付清算 pallet benchmarks。
//!
//! Step 2b-iv-b 清理后,老省储行 `submit_offchain_batch` / `enqueue_offchain_batch`
//! / `process_queued_batch` 已从 pallet 中物理删除,对应 benchmark 亦同步移除。
//! 新清算行 v2 路径的 benchmark 留给后续在 runtime 稳态运行后再接。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;

#[benchmarks]
mod benchmarks {
    // 占位:Step 3 起补 `bind_clearing_bank` / `deposit` / `withdraw` / `switch_bank`
    // / `submit_offchain_batch_v2` / `propose_l2_fee_rate` / `set_max_l2_fee_rate`
    // 的 benchmark。当前 call 权重在 pallet 内直接用 `T::DbWeight` 估算。
}
