#![cfg(test)]

//! election-vote 最小单元测试入口。
//!
//! 中文注释：完整 runtime mock 会在接入业务 provider 后补；当前先覆盖纯计票规则，
//! 保证“多候选、多席位、同票拒绝”的框架语义稳定。

// 具体测试位于 `tally.rs` 的纯函数测试中。
