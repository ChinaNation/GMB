//! 机构多签管理。
//!
//! 中文注释:
//! - 本目录只服务 OrganizationManage 机构多签创建、CID 凭证拉取、机构详情查询。
//! - 清算行节点声明、支付清算、批次结算仍留在 `transaction/offchain_transaction/` 目录。
//! - 普通注册机构仍由 CitizenApp 操作;这里不承接个人多签、转账或国储会安全基金等业务。

pub mod chain;
pub mod cid;
pub mod commands;
pub mod signing;
pub mod types;
