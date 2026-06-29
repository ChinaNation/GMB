//! 机构码 → 控制台能力位的唯一权威源(后端单源,经会话下发给前端镜像)。
//!
//! 决定「什么机构登录后显示什么 tab、能做什么」。这是权限的【声明式单源】,与后端实际执行
//! 边界(`is_tier1_registry` 谓词 / `scope` / 链上 Active 集合)同源对齐;前端只据此 render-gating,
//! 不构成安全边界(后端始终对越权独立拒绝)。
//!
//! 机构类分发(`primitives::cid::code` 单源):
//! - Tier1 创世注册局(FRG):管「联邦 + 市注册局」两 tab,可 CRUD 市注册局管理员。
//! - Tier2 下级注册局(CREG,公权子集):录入公民/机构 + 看本市注册局。
//! - 其它创世机构(NRC/PRC/PRB/NJD):自治,不归本控制台 → 空能力。
//! - 其余公权/私权/非法人法人:本期只开只读「本机构管理员」位(`can_view_own_admins`);
//!   CRUD / 录入等具体功能待各机构功能落地时再开(机制就绪、不越权)。
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
    /// 只读「本机构管理员」位:非注册局法人登录后可查看本机构链上管理员列表(只读)。
    pub(crate) can_view_own_admins: bool,
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
    can_view_own_admins: false,
    can_manage_institutions: false,
    can_register_institutions: false,
    can_business_write: false,
    can_view_city_registry: false,
    can_view_federal_registry: false,
};

// Tier1 创世注册局(FRG):管「联邦注册局 + 市注册局」两个 tab,可 CRUD 市注册局管理员;不录入公民/机构。
const TIER1_REGISTRY: CapabilitySet = CapabilitySet {
    can_view_federal_registry_admins: true,
    can_view_city_registry_admins: true,
    can_crud_city_registry_admins: true,
    can_view_city_registry: true,
    can_view_federal_registry: true,
    ..EMPTY
};

// Tier2 下级注册局(CREG):录入公民/私权/教育/公权机构 + 看本市市注册局;不可见联邦 tab。
const SUBORDINATE_REGISTRY: CapabilitySet = CapabilitySet {
    can_view_citizens: true,
    can_view_institutions: true,
    can_view_private: true,
    can_view_education: true,
    can_view_city_registry_admins: true,
    can_manage_institutions: true,
    can_register_institutions: true,
    can_business_write: true,
    can_view_city_registry: true,
    ..EMPTY
};

// 普通法人(公权/私权/非法人,非注册局):本期只开只读「本机构管理员」位。
const OWN_ADMINS_READONLY: CapabilitySet = CapabilitySet {
    can_view_own_admins: true,
    ..EMPTY
};

/// 机构码文本 → 能力集。按机构类(`primitives::cid::code` 单源)分发,未知/不归控制台返回空能力。
pub(crate) fn capabilities_for(institution_code: &str) -> CapabilitySet {
    use primitives::cid::code::{
        institution_code_from_str, is_fixed_governance_code, is_private_legal_code,
        is_public_legal_code, is_unincorporated_code,
    };
    // Tier1/Tier2 注册局保留专属能力集(行为零变)。
    if crate::core::chain_runtime::is_tier1_registry(institution_code) {
        return TIER1_REGISTRY;
    }
    if crate::core::chain_runtime::is_subordinate_registry(institution_code) {
        return SUBORDINATE_REGISTRY;
    }
    let Some(code) = institution_code_from_str(institution_code) else {
        return EMPTY;
    };
    // 其它创世机构(NRC/PRC/PRB/NJD)自治,不归本控制台。
    if is_fixed_governance_code(&code) {
        return EMPTY;
    }
    // 其余公权/私权/非法人法人:只读本机构管理员位。
    if is_public_legal_code(&code) || is_private_legal_code(&code) || is_unincorporated_code(&code)
    {
        return OWN_ADMINS_READONLY;
    }
    EMPTY
}
