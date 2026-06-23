//! 公民身份业务模块。
//!
//! 公民电子护照绑定、状态扫码、CitizenApp 电子护照状态查询
//! 等业务统一收口在此目录。
//! 公民模型和公民查询 handler 归属本目录。

pub(crate) mod binding;
/// 中文注释:公民模块联合投票人口快照凭证接口。
pub(crate) mod chain_joint_vote;
/// 中文注释:公民投票凭证签发接口。
pub(crate) mod chain_vote;
pub(crate) mod handler;
pub(crate) mod model;
pub(crate) mod status_export_import;
pub(crate) mod vote;

#[allow(unused_imports)]
pub(crate) use model::*;
