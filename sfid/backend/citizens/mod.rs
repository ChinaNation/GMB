//! 公民身份业务模块。
//!
//! 公民端身份绑定凭证签发、链上绑定推送、状态扫码、wuminapp 投票账户登记
//! 等业务统一收口在此目录;phase23d 由 `operate/` 整体迁入。
//! 公民模型和公民查询 handler 归属本目录,不再放在 `models` 或 `scope`。

pub(crate) mod binding;
/// 中文注释:公民模块与区块链交互的绑定推链入口,按 `chain_` 文件规则归属本模块。
pub(crate) mod chain_binding;
/// 中文注释:公民模块联合投票人口快照凭证接口。
pub(crate) mod chain_joint_vote;
/// 中文注释:公民投票凭证签发接口。
pub(crate) mod chain_vote;
pub(crate) mod cpms_qr;
pub(crate) mod handler;
pub(crate) mod model;
pub(crate) mod status;
pub(crate) mod vote;

#[allow(unused_imports)]
pub(crate) use model::*;
