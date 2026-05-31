//! 中文注释:管理员角色 / 实体 + 管理员列表与维护接口 DTO。
//!
//! 中文注释:当前只保留 ShengAdmin / ShiAdmin 两个管理员角色。
//! 省级管理员采用同级模型;代码内置初始省级管理员只承担不可删除安全根职责。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 中文注释:两种管理员角色(ADR-008 后)
//   - ShengAdmin → 省级管理员(每省 N 人;内置初始管理员不可删除) 目录 admins/
//   - ShiAdmin   → 市级管理员(每市 N 人)                         目录 admins/
// 序列化为 SHENG_ADMIN / SHI_ADMIN,数据库字段值同。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminRole {
    ShengAdmin,
    ShiAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminUser {
    pub(crate) id: u64,
    pub(crate) admin_pubkey: String,
    #[serde(default)]
    pub(crate) admin_name: String,
    pub(crate) role: AdminRole,
    /// 中文注释:初始省级管理员由代码内置,不可删除;后续新增管理员为 false。
    pub(crate) built_in: bool,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
    /// ShiAdmin 所属的市名称（仅 ShiAdmin 必填，其他角色为空字符串）
    #[serde(default)]
    pub(crate) city: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OperatorRow {
    pub(crate) id: u64,
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    pub(crate) role: AdminRole,
    pub(crate) built_in: bool,
    pub(crate) created_by: String,
    pub(crate) created_by_name: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) city: String,
}

#[derive(Serialize)]
pub(crate) struct OperatorListOutput {
    pub(crate) total: usize,
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) rows: Vec<OperatorRow>,
}

// 省级管理员对外行(API 序列化)。
// 中文注释:管理员只有存在/删除,不存在停用状态。
#[derive(Serialize)]
pub(crate) struct ShengAdminRow {
    pub(crate) id: u64,
    pub(crate) province: String,
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    pub(crate) built_in: bool,
    pub(crate) created_at: DateTime<Utc>,
    /// 最近一次更新时间，None 表示从未更新
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateOperatorInput {
    pub(crate) admin_pubkey: String,
    pub(crate) admin_name: String,
    /// ShiAdmin 所属的市，必填，且必须属于 created_by 对应机构管理员的省份（不可为省辖市）
    pub(crate) city: String,
    /// 可选：指定该 operator 归属的机构管理员 pubkey。
    /// ShengAdmin 调用时若指定则必须等于自己 pubkey，否则 403。
    /// 不指定则默认为调用者自身。
    #[serde(default)]
    pub(crate) created_by: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ListQuery {
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}
