//! 中文注释:SFID 后端通用 DTO/状态聚合体的 facade。
//!
//! Phase 23a 拆分:把 1021 行 `models/mod.rs` 按语义切到子文件,本 mod.rs
//! 只剩 `pub mod` + `pub(crate) use ::*` re-export,确保 `crate::models::*`
//! glob 在 `main.rs:51 pub(crate) use models::*;` 下行为零变化。
//!
//! - role:管理员角色 / 状态 / Operator DTO / ShengAdmin 行
//! - slot:省管理员槽位枚举 re-export(实际定义在 `crate::sheng_admins::province_admins`)
//! - session:登录态 DTO 占位(目前位于 `crate::login`)
//! - permission:权限决策 DTO 占位(目前由 `crate::scope` 直接消费 AdminRole)
//! - error:HTTP API 通用响应 / 错误 / 健康检查输出包装
//! - store:进程内 `Store` 聚合体 + 敏感种子 + 指标 / 审计 / 链请求回执 /
//!   异步绑定回调 / 公民奖励 / 投票验证缓存 / Keyring 轮换 / 机构 & 账户链状态
//! - citizen:公民身份记录、绑定状态机、绑定/解绑 API、wuminapp 投票账户、扫码 QR 载荷
//! - cpms:CPMS 站点凭证、安装 token、QR1/QR2/QR3/QR4 载荷、匿名证书
//! - meta:SFID 行政区 / 选项元数据(管理员控制台元信息接口)

pub(crate) mod citizen;
pub(crate) mod cpms;
pub(crate) mod error;
pub(crate) mod meta;
pub(crate) mod permission;
pub(crate) mod role;
pub(crate) mod session;
pub(crate) mod slot;
pub(crate) mod store;

#[allow(unused_imports)]
pub(crate) use citizen::*;
#[allow(unused_imports)]
pub(crate) use cpms::*;
#[allow(unused_imports)]
pub(crate) use error::*;
#[allow(unused_imports)]
pub(crate) use meta::*;
#[allow(unused_imports)]
pub(crate) use role::*;
#[allow(unused_imports)]
pub(crate) use slot::*;
#[allow(unused_imports)]
pub(crate) use store::*;
