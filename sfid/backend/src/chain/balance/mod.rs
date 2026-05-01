//! 链余额查询(给前端 keyring 视图主账户行实时显示)。
//!
//! - HTTP 端点:`GET /api/v1/admin/chain/balance?account_pubkey=<hex>`
//! - 数据流:admin UI → SFID → 全节点 `state_getStorage(System::Account)` → free 余额
//! - 调用方:仅 SFID 自家管理后台 keyring 视图
//!
//! 本目录是 chain ↔ SFID 交互能力之一,按"chain pull"铁律组织在 chain/ 下。

pub(crate) mod dto;
pub(crate) mod handler;

pub(crate) use handler::admin_query_chain_balance;
