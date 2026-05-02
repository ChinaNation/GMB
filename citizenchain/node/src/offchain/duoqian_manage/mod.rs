//! 清算行注册机构多签管理。
//!
//! 中文注释:
//! - 本目录只服务"清算行注册机构"的多签创建、SFID 凭证拉取、机构详情查询。
//! - 普通注册机构仍由 wuminapp 操作;这里不承接普通机构、治理机构或国储会
//!   安全基金等其它业务。

pub mod chain;
pub mod commands;
pub mod sfid;
pub mod signing;
