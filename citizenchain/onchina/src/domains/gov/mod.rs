//! 公权机构模块。
//!
//! 前后端统一使用 `gov` 命名。自动公权目录、确定性目录、
//! 公权机构列表入口都归这里;跨模块共享结构只通过机构内核复用。

pub(crate) mod chain_audit;
pub(crate) mod handler;
pub(crate) mod service;
