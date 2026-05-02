//! 中文注释:sfid-system pallet 的省管理员子域。
//!
//! 目录名与 SFID 前后端保持一致:所有省管理员三槽、签名公钥、payload
//! 与迁移提示都从这里暴露;`lib.rs` 只保留 FRAME storage/call 壳。

pub mod migration;
pub mod payload;
pub mod types;
