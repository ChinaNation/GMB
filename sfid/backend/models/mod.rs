//! 中文注释:SFID 后端全局共享模型 facade。
//!
//! 2026-05-02 models/scope 边界整改后,本目录只保留跨业务共享的数据结构。
//! 公民、CPMS、SFID 元信息、机构链状态等业务 DTO 已归还各自功能模块。
//!
//! - role:管理员角色 / 状态 / Operator DTO / ShengAdmin 行
//! - error:HTTP API 通用响应 / 错误 / 健康检查输出包装
//! - store:进程内 `Store` 聚合体 + 敏感种子 + 指标 / 审计 / 链请求回执 /
//!   异步绑定回调 / 公民奖励 / 投票验证缓存

pub(crate) mod error;
pub(crate) mod role;
pub(crate) mod store;

#[allow(unused_imports)]
pub(crate) use error::*;
#[allow(unused_imports)]
pub(crate) use role::*;
#[allow(unused_imports)]
pub(crate) use store::*;
