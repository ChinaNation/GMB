//! 公权机构模块。
//!
//! 中文注释:前后端统一使用 `gov` 命名。自动公权目录、公安局确定性目录、
//! 公权机构列表入口都归这里;跨模块共享结构只通过机构内核复用。

pub(crate) mod handler;
pub(crate) mod service;
