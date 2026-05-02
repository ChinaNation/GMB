//! 中文注释:SFID 后端与链上省管理员能力的统一交互目录。
//!
//! 本目录对应 runtime `sfid-system/src/sheng_admins/`,集中存放省管理员
//! 三槽名册、签名公钥激活/轮换、冷钱包待签缓存等链交互代码。非链上的
//! 省管理员业务仍放在 `crate::sheng_admins/`。
//!
//! ## 职责
//!
//! - `query.rs`:拉链上 `ShengAdmins[Province][Slot]` 三槽名册。
//! - `add_backup.rs` / `remove_backup.rs`:提交省管理员 backup 槽变更 extrinsic。
//! - `activate_signer.rs` / `rotate_signer.rs`:提交签名公钥激活与轮换 extrinsic。
//! - `pending_signs.rs`:冷钱包双步签名 prepare/submit-sig 的 nonce 暂存。
//! - `handler.rs`:HTTP handler 与公开 pull endpoint。

pub(crate) mod activate_signer;
pub(crate) mod add_backup;
pub(crate) mod handler;
pub(crate) mod pending_signs;
pub(crate) mod query;
pub(crate) mod remove_backup;
pub(crate) mod rotate_signer;
