//! 清算行(L2)扫码支付清算 pallet benchmarks。
//!
//! Step 2b-iv-b 清理后,老省储行 `submit_offchain_batch` / `enqueue_offchain_batch`
//! / `process_queued_batch` 已从 pallet 中物理删除,对应 benchmark 亦同步移除。
//!
//! 当前 `weights.rs` 已接入非零保守权重和 `T::WeightInfo`,不再是空权重占位。
//! 这里保留 benchmark 模块入口,等待专用 benchmarking runtime WASM 准备后,
//! 再按同名 `WeightInfo` 方法生成正式权重文件。

#![cfg(feature = "runtime-benchmarks")]

// 中文注释：当前 crate 暂未提供可执行 benchmark。这里不能保留空的
// `#[benchmarks]` 模块，否则 frame-benchmarking 宏会生成非法代码并阻断
// runtime-benchmarks 构建。正式补齐 offchain-transaction benchmark 时，再恢复
// `bind_clearing_bank` / `deposit` / `withdraw` / `switch_bank` 等入口。
