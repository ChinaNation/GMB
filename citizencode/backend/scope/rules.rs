//! 注册局机构范围规则(ADR-008 后)
//!
//! 中文注释:根据登录账户所属注册局机构 + 所属省 + 所属市,派生该账户能看到和操作的
//! 数据范围(VisibleScope)。所有 list/CRUD API 都应当先派生 scope,再用它
//! 过滤数据。
//!
//! 中文注释:当前只保留 FederalRegistry / CityRegistry 两类注册局机构范围。

use crate::admins::login::AdminAuthContext;
use crate::admins::model::RegistryOrgCode;

/// 登录管理员可见的数据范围。
#[derive(Debug, Clone)]
pub struct VisibleScope {
    /// 可见省份列表。当前最少 1 个(本省),保留 Vec 是为了将来扩展跨省视图。
    pub provinces: Vec<String>,
    /// 可见城市列表。空 vec 表示"不限市"(FEDERAL_REGISTRY)。
    pub cities: Vec<String>,
    /// 是否可以增删改。当前两角色在自己范围内都能写,保留字段为将来扩展只读角色。
    pub can_write: bool,
    /// 进入 tab 时是否跳过省份列表直接进入详情。
    /// - FederalRegistry: true(直接进本省)
    /// - CityRegistry: true(直接进本市,同时跳过市列表)
    pub skip_province_list: bool,
    /// 进入 tab 时是否跳过市列表(仅 CITY_REGISTRY)。
    pub skip_city_list: bool,
    /// 锁定的省名称(FEDERAL_REGISTRY / CITY_REGISTRY 进入时自动填)。
    pub locked_province_name: Option<String>,
    /// 锁定的市名称(CITY_REGISTRY 进入时自动填)。
    pub locked_city_name: Option<String>,
}

impl VisibleScope {
    /// FederalRegistry:只看本省,可在本省写。
    pub fn federal_registry(province: String) -> Self {
        Self {
            provinces: vec![province.clone()],
            cities: vec![],
            can_write: true,
            skip_province_list: true,
            skip_city_list: false,
            locked_province_name: Some(province),
            locked_city_name: None,
        }
    }

    /// CityRegistry:只看本市,可在本市写。
    pub fn city_registry(province: String, city: String) -> Self {
        Self {
            provinces: vec![province.clone()],
            cities: vec![city.clone()],
            can_write: true,
            skip_province_list: true,
            skip_city_list: true,
            locked_province_name: Some(province),
            locked_city_name: Some(city),
        }
    }

    /// 判断某省是否在范围内。
    pub fn includes_province(&self, province: &str) -> bool {
        self.provinces.is_empty() || self.provinces.iter().any(|p| p == province)
    }

    /// 判断某市是否在范围内。city 为空字符串时视为"不限市"。
    pub fn includes_city(&self, city: &str) -> bool {
        self.cities.is_empty() || self.cities.iter().any(|c| c == city)
    }
}

/// 根据登录管理员上下文派生 VisibleScope。
///
/// 中文注释:FederalRegistry 缺 scope_province_name 或 CityRegistry 缺 scope_city_name 时,会
/// fallback 到最严格的"零范围"(provinces=["<INVALID>"]),这样过滤后返回空列表,
/// 避免误放行。调用方应当在 require_admin_* 里先校验必要字段。
pub fn get_visible_scope(ctx: &AdminAuthContext) -> VisibleScope {
    match ctx.registry_org_code {
        RegistryOrgCode::FederalRegistry => {
            let province = ctx
                .scope_province_name
                .clone()
                .unwrap_or_else(|| "__FEDERAL_REGISTRY_MISSING_PROVINCE__".to_string());
            VisibleScope::federal_registry(province)
        }
        RegistryOrgCode::CityRegistry => {
            let province = ctx
                .scope_province_name
                .clone()
                .unwrap_or_else(|| "__CITY_REGISTRY_MISSING_PROVINCE__".to_string());
            let city = ctx
                .scope_city_name
                .clone()
                .unwrap_or_else(|| "__CITY_REGISTRY_MISSING_CITY__".to_string());
            VisibleScope::city_registry(province, city)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn federal_registry_limited_to_province() {
        let s = VisibleScope::federal_registry("安徽省".to_string());
        assert!(s.includes_province("安徽省"));
        assert!(!s.includes_province("广东省"));
        assert!(s.includes_city("合肥市"));
        assert!(s.skip_province_list);
        assert!(!s.skip_city_list);
    }

    #[test]
    fn city_registry_limited_to_city() {
        let s = VisibleScope::city_registry("安徽省".to_string(), "合肥市".to_string());
        assert!(s.includes_province("安徽省"));
        assert!(!s.includes_province("广东省"));
        assert!(s.includes_city("合肥市"));
        assert!(!s.includes_city("芜湖市"));
        assert!(s.skip_province_list);
        assert!(s.skip_city_list);
    }
}
