//! Benchmark setup for the votingengine pallet.
//!
//! 引擎核心保留 lifecycle extrinsic(finalize_proposal / retry_passed_proposal /
//! cancel_passed_proposal),mode-specific 投票/创建 benchmark 由 internal-vote /
//! joint-vote / election-vote sub-pallet 各自承担。
//!
//! 待 substrate-benchmark-cli 完成真实跑测后填充具体权重数据。
#![cfg(feature = "runtime-benchmarks")]
