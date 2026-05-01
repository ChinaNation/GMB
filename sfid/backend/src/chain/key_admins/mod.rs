//! key-admins 推链能力(过渡态保留)。
//!
//! 当前由 [`crate::key_admins`] 模块的 admin handler 调用本目录的 helper:
//! - `rotate_sfid_keys` 推链(主备账户轮换)
//! - `set_sheng_signing_pubkey` 推链(省级签名密钥注册/清除)
//! - `state_getStorage` 通用 chain 读取(供 `chain::balance` 等共用)
//! - `fetch_chain_keyring_from_chain` 启动期同步链上 keyring 状态
//!
//! key-admins 的密钥派生 / 加密落盘 / admin 鉴权 / RSA 盲签等业务能力
//! 仍留在 [`crate::key_admins`],与本目录解耦——本目录只承载"和链通信"的部分。
//!
//! 本目录代码与上层调用方 1:1 对接,不做语义改动;待链端补齐 chain pull 模式后,
//! `submit_*` 系列函数应被删除,届时 admin 流程改为返回待签 payload 给外部签名。

pub(crate) mod chain_keyring_query;
pub(crate) mod rotate;
pub(crate) mod sheng_signing;
pub(crate) mod state_query;

pub(crate) use chain_keyring_query::fetch_chain_keyring_from_chain;
pub(crate) use rotate::submit_rotate_sfid_keys_extrinsic;
pub(crate) use sheng_signing::submit_set_sheng_signing_pubkey_with_client;
pub(crate) use state_query::call_chain_state_get_storage;
