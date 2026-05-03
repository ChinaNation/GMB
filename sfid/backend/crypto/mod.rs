//! 中文注释:SFID 后端低层加密工具集。
//!
//! 本目录承载与具体业务无关的 sr25519 / 哈希工具,放在业务模块
//! (sheng_admins / institutions / citizens / cpms) 之下,避免业务模块互相依赖。
//!
//! 这里仅保留 sr25519、公钥规范化等低层加密辅助,业务模块不得重复实现。

pub(crate) mod pubkey;
pub(crate) mod sr25519;
