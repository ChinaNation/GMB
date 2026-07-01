//! 法律案(本轮实现):章节条款提案 + 院内/两院表决 + 签署。
//!
//! 中文注释:Phase 1 增量交付裸 SCALE call-data 编码器——
//! - `chain_propose`:`propose_enact/amend/repeal_law`(pallet 27);
//! - `chain_vote`:`cast_house_vote` 等表决/签署(pallet 28)。
//! 复用 `core::institution_call` 的「构造 call data → origin 冷签 → CitizenWallet 提交」通道,
//! 逐字节交叉校验链端 SCALE。HTTP DTO(model)、`service` 组织提案数据、链读(`chain_read`)、
//! 冷签 prepare/commit 随 Phase 1B 后续增量落地。

/// 中文注释:立法冷签动作——组织 ChainCall → 链交易 sign_request(扫码上链)。
pub(crate) mod action;
pub(crate) mod chain_propose;
/// 中文注释:法律链读——链上 Law/LawVersion SCALE 解码镜像 + → 展示 DTO。
pub(crate) mod chain_read;
pub(crate) mod chain_vote;
/// 中文注释:法律案 HTTP DTO + 章节条款 ↔ 链编码器入参转换 + 读模型。
pub(crate) mod model;
/// 中文注释:法律案宪法路由(层级×是否教育 → houses/executive/legislature 机构码)。
pub(crate) mod routing;
/// 中文注释:法律案提案组织(请求 + 路由 + 账户解析 → call-data)+ 写入边界 scope 前置。
pub(crate) mod service;
