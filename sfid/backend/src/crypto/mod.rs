//! 中文注释:SFID 后端低层加密工具集。
//!
//! 本目录承载与具体业务无关的 sr25519 / 哈希工具,放在业务模块
//! (sheng_admins / institutions / citizens / cpms) 之下,避免业务模块互相依赖。
//!
//! 历史上这些 helper 散落在 `key-admins/chain_keyring.rs` 内。ADR-008 决议下
//! KEY_ADMIN 整角色废止(phase23e 子卡,2026-05-01),仅保留 sr25519 helper。

pub(crate) mod sr25519;
