//! 公民身份业务模块。
//!
//! 公民电子护照档案、状态扫码、CitizenApp 电子护照状态查询
//! 等业务统一收口在此目录。
//! 公民模型和公民查询 handler 归属本目录。

/// 中文注释:注册局直接录入公民并直接发护照入口。
pub(crate) mod admin_entry;
/// 中文注释:公民模块联合投票本地人数查询接口。
pub(crate) mod chain_joint_vote;
/// 中文注释:公民投票资格查询接口。
pub(crate) mod chain_vote;
pub(crate) mod handler;
pub(crate) mod model;
/// 中文注释:OnChina 自持的护照号与护照有效期生成逻辑。
pub(crate) mod passport_no;
pub(crate) mod vote;

#[allow(unused_imports)]
pub(crate) use model::*;
