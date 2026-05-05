//! 投票引擎 runtime 升级迁移合集。
//!
//! 各 spec_version 升级对应一个 `vN.rs` 模块,实现 `frame_support::traits::OnRuntimeUpgrade`。
//! 升级版本号在 `runtime/src/lib.rs` 的 `RuntimeVersion::spec_version` 中递增。
//!
//! - **v1**(双层 ID 改造):主键纯单调 u64 + ProposalDisplayId + 4 张反向索引;
//!   `on_runtime_upgrade` 期间回填存量提案的展示号与索引,主键保持不变。

pub mod v1;
