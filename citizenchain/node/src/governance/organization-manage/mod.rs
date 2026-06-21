//! 机构多签管理。
//!
//! 中文注释:
//! - 本目录只服务 OrganizationManage 机构多签创建、SFID 凭证拉取、机构详情查询。
//! - 清算行节点声明、支付清算、批次结算仍留在 `offchain/` 目录。
//! - 普通注册机构仍由 citizenapp 操作;这里不承接个人多签、转账或国储会安全基金等业务。

pub mod chain;
pub mod commands;
pub mod sfid;
pub mod signing;
pub mod types;
