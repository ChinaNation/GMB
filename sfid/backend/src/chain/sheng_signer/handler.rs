//! 中文注释:`chain/sheng_signer/` 模块的 HTTP handler 占位。
//!
//! 真正 handler 各自定义在 `activation.rs::handler` / `rotation.rs::handler`,
//! 在 `main.rs` 路由表挂载。本文件只做 re-export,与 `chain/sheng_admin/handler.rs`
//! 风格对齐(便于 phase7 改造时统一找入口)。

#![allow(dead_code, unused_imports)]

pub(crate) use super::activation::handler as activate_handler;
pub(crate) use super::rotation::handler as rotate_handler;
