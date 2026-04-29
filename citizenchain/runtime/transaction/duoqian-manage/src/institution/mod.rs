//! 机构多签业务分区。
//!
//! 第1步先把机构级类型和目录边界拆出来；storage、call 和投票回调仍由 FRAME
//! pallet 宏所在的 `lib.rs` 承载，避免在一次改造里同时移动宏代码和业务语义。

pub mod accounts;
pub mod close;
pub mod create;
pub mod register;
pub mod types;
pub mod vote;

pub use types::*;
