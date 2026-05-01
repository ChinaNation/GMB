//! 省级管理员目录的 chain 交互能力。
//!
//! 当前仅一个用法:KEY_ADMIN 替换某省登录管理员时,**清除**该省链上 signing
//! pubkey,确保新管理员首次登录时按 bootstrap 流程重新生成密钥并推链。
//!
//! 上层调用方仍是 [`crate::sheng_admins::catalog::replace_sheng_admin`] handler,
//! 本目录只承载"和链通信"的部分。

pub(crate) mod clear_sheng_signing;

pub(crate) use clear_sheng_signing::clear_sheng_signing_pubkey_on_chain;
