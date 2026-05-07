//! 个人多签管理 pallet benchmark 占位。
//!
//! D 阶段(2026-05-06)起,此处仅保留模块编译入口;实际 benchmark 用例待 follow-up:
//! - propose_create / propose_close / cleanup_rejected_proposal
//!
//! 当前 weights.rs 用零权重占位与零 benchmark 等价,不影响 runtime 功能。

#![cfg(feature = "runtime-benchmarks")]

// 中文注释:暂无 benchmark 用例;空文件保持模块编译入口存在,供 lib.rs 内
// `#[cfg(feature = "runtime-benchmarks")] mod benchmarks;` 引用通过。
// 后续补 benchmark 用例时,使用 `#[frame_benchmarking::v2::benchmarks]` 装饰一组
// 带 `#[extrinsic_call]` 标注的函数即可。
