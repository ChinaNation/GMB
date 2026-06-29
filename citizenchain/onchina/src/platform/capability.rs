//! 机构码 → 控制台能力位的唯一权威源(后端单源,经会话下发给前端镜像)。
//!
//! 决定「什么机构登录后显示什么 tab、能做什么」。这是权限的【声明式单源】,与后端实际执行
//! 边界(`requires_federal_admin` / `scope` / 链上 Active 集合)同源对齐;前端只据此 render-gating,
//! 不构成安全边界(后端始终对越权独立拒绝)。
//!
//! 本期内置 FRG(联邦注册局)/ CREG(市注册局);其它机构码返回空能力占位——其管理员登录后的
//! 具体功能后续实现时,在本表逐个补对应能力位。
//!
//! serde 字段名用 camelCase,与前端 `platform/capabilityMap.ts` 的 RoleCapabilities 逐字段对齐。

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CapabilitySet {
    pub(crate) can_view_citizens: bool,
    pub(crate) can_view_institutions: bool,
    pub(crate) can_view_private: bool,
    pub(crate) can_view_education: bool,
    pub(crate) can_view_federal_registry_admins: bool,
    pub(crate) can_view_city_registry_admins: bool,
    pub(crate) can_crud_city_registry_admins: bool,
    pub(crate) can_manage_institutions: bool,
    pub(crate) can_register_institutions: bool,
    pub(crate) can_business_write: bool,
    pub(crate) can_view_city_registry: bool,
    pub(crate) can_view_federal_registry: bool,
}

const EMPTY: CapabilitySet = CapabilitySet {
    can_view_citizens: false,
    can_view_institutions: false,
    can_view_private: false,
    can_view_education: false,
    can_view_federal_registry_admins: false,
    can_view_city_registry_admins: false,
    can_crud_city_registry_admins: false,
    can_manage_institutions: false,
    can_register_institutions: false,
    can_business_write: false,
    can_view_city_registry: false,
    can_view_federal_registry: false,
};

// 联邦注册局:管「联邦注册局 + 市注册局」两个 tab,可 CRUD 市注册局管理员;不录入公民/机构。
const FRG: CapabilitySet = CapabilitySet {
    can_view_citizens: false,
    can_view_institutions: false,
    can_view_private: false,
    can_view_education: false,
    can_view_federal_registry_admins: true,
    can_view_city_registry_admins: true,
    can_crud_city_registry_admins: true,
    can_manage_institutions: false,
    can_register_institutions: false,
    can_business_write: false,
    can_view_city_registry: true,
    can_view_federal_registry: true,
};

// 市注册局:录入公民/私权/教育/公权机构 + 看本市市注册局;不可见联邦 tab。
const CREG: CapabilitySet = CapabilitySet {
    can_view_citizens: true,
    can_view_institutions: true,
    can_view_private: true,
    can_view_education: true,
    can_view_federal_registry_admins: false,
    can_view_city_registry_admins: true,
    can_crud_city_registry_admins: false,
    can_manage_institutions: true,
    can_register_institutions: true,
    can_business_write: true,
    can_view_city_registry: true,
    can_view_federal_registry: false,
};

/// 机构码文本 → 能力集;未知机构码返回空能力(占位,后续逐个补)。
pub(crate) fn capabilities_for(institution_code: &str) -> CapabilitySet {
    match institution_code {
        "FRG" => FRG,
        "CREG" => CREG,
        _ => EMPTY,
    }
}
