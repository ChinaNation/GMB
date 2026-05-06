//! Benchmark setup for the votingengine pallet.
//!
//! 引擎核心保留 lifecycle extrinsic(finalize_proposal / retry_passed_proposal /
//! cancel_passed_proposal),mode-specific 投票/创建 benchmark 由 internal-vote /
//! joint-vote / citizen-vote sub-pallet 各自承担。
//!
//! TODO:等 substrate-benchmark-cli 真实跑测后填充。
#![cfg(feature = "runtime-benchmarks")]
