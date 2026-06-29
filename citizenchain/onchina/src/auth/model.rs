//! 机构管理员实体 + 管理员列表与维护接口 DTO。
//!
//! 管理员按机构码(`institution_code`,3/4 字符文本)归属机构;内置初始联邦注册局管理员
//! 承担不可删除安全根职责。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdminUser {
    pub(crate) id: u64,
    pub(crate) admin_account: String,
    #[serde(default)]
    pub(crate) admin_name: String,
    /// 所属机构码(3/4 字符文本,如 FRG/CREG/NLG)。
    pub(crate) institution_code: String,
    /// 中文注释:初始联邦注册局管理员由代码内置,不可删除;代码以外新增管理员为 false。
    pub(crate) built_in: bool,
    pub(crate) created_by: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
    /// 市级机构所属的市名称(市级机构必填,其它机构为空字符串)。
    #[serde(default)]
    pub(crate) city_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CityRegistryAdminRow {
    pub(crate) id: u64,
    pub(crate) admin_account: String,
    pub(crate) admin_name: String,
    pub(crate) institution_code: String,
    pub(crate) built_in: bool,
    pub(crate) created_by: String,
    pub(crate) created_by_name: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) city_name: String,
}

#[derive(Serialize)]
pub(crate) struct CityRegistryAdminListOutput {
    pub(crate) total: usize,
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) rows: Vec<CityRegistryAdminRow>,
}

// 联邦注册局管理员对外行(API 序列化)。
// 中文注释:管理员只有存在/删除,不存在停用状态。
#[derive(Serialize)]
pub(crate) struct FederalRegistryAdminRow {
    pub(crate) id: u64,
    pub(crate) province_name: String,
    pub(crate) admin_account: String,
    pub(crate) admin_name: String,
    pub(crate) built_in: bool,
    pub(crate) created_at: DateTime<Utc>,
    /// 最近一次更新时间，None 表示从未更新
    #[serde(default)]
    pub(crate) updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateCityRegistryAdminInput {
    pub(crate) admin_account: String,
    pub(crate) admin_name: String,
    /// CityRegistry 所属的市，必填，且必须属于 created_by 对应联邦注册局管理员的省份（不可为省辖市）
    pub(crate) city_name: String,
    /// 可选：指定该 city_registry 归属的联邦注册局管理员账户。
    /// FederalRegistry 调用时若指定则必须等于自己账户，否则 403。
    /// 不指定则默认为调用者自身。
    #[serde(default)]
    pub(crate) created_by: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ListQuery {
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}
