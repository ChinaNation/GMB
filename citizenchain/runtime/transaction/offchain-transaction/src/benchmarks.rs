//! 清算行(L2)扫码支付清算 pallet benchmarks。
//!
//! Step 2b-iv-b 清理后,老省储行 `submit_offchain_batch` / `enqueue_offchain_batch`
//! / `process_queued_batch` 已从 pallet 中物理删除,对应 benchmark 亦同步移除。
//!
//! 当前 `weights.rs` 已接入非零保守权重和 `T::WeightInfo`,不再是空权重占位。
//! 这里保留 benchmark 模块入口,等待专用 benchmarking runtime WASM 准备后,
//! 再按同名 `WeightInfo` 方法生成正式权重文件。

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;

#[benchmarks]
mod benchmarks {
    // 正式 benchmark 生成入口:后续在 benchmark runtime 中补
    // `bind_clearing_bank` / `deposit` / `withdraw` / `switch_bank` /
    // `submit_offchain_batch_v2` / `propose_l2_fee_rate` / `set_max_l2_fee_rate` /
    // 节点声明三类 Call 的自动 benchmark。
    //
    // 注意:这些 benchmark 需要构造真实 `SfidAccountQuery` 对应的链上机构、
    // 管理员与清算行节点声明。offchain-transaction crate 为避免循环依赖,
    // 不直接依赖 duoqian-manage,因此正式生成应在完整 runtime 层执行。
}
