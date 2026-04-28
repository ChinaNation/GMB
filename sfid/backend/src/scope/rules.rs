//! 三角色权限范围规则
//!
//! 中文注释:根据登录管理员的角色 + 所属省 + 所属市,派生该角色能看到和操作的
//! 数据范围(VisibleScope)。所有 list/CRUD API 都应当先派生 scope,再用它
//! 过滤数据。
//!
//! 见 `feedback_sfid_three_roles_naming.md` 的三角色命名铁律。

use crate::login::AdminAuthContext;
use crate::models::AdminRole;

/// 登录管理员可见的数据范围。
#[derive(Debug, Clone)]
pub struct VisibleScope {
    /// 可见省份列表。空 vec 表示"全国"(KEY_ADMIN)。
    pub provinces: Vec<String>,
    /// 可见城市列表。空 vec 表示"不限市"(KEY_ADMIN + SHENG_ADMIN)。
    pub cities: Vec<String>,
    /// 是否可以增删改。当前三角色在自己范围内都能写,保留字段为将来扩展只读角色。
    pub can_write: bool,
    /// 进入 tab 时是否跳过省份列表直接进入详情。
    /// - KeyAdmin: false(看 43 省卡片)
    /// - ShengAdmin: true(直接进本省)
    /// - ShiAdmin: true(直接进本市,同时跳过市列表)
    pub skip_province_list: bool,
    /// 进入 tab 时是否跳过市列表(仅 SHI_ADMIN)。
    pub skip_city_list: bool,
    /// 锁定的省份(SHENG_ADMIN / SHI_ADMIN 进入时自动填)。
    pub locked_province: Option<String>,
    /// 锁定的市(SHI_ADMIN 进入时自动填)。
    pub locked_city: Option<String>,
}

impl VisibleScope {
    /// KeyAdmin:看全国,可写。
    pub fn key_admin() -> Self {
        Self {
            provinces: vec![],
            cities: vec![],
            can_write: true,
            skip_province_list: false,
            skip_city_list: false,
            locked_province: None,
            locked_city: None,
        }
    }

    /// ShengAdmin:只看本省,可在本省写。
    pub fn sheng_admin(province: String) -> Self {
        Self {
            provinces: vec![province.clone()],
            cities: vec![],
            can_write: true,
            skip_province_list: true,
            skip_city_list: false,
            locked_province: Some(province),
            locked_city: None,
        }
    }

    /// ShiAdmin:只看本市,可在本市写。
    pub fn shi_admin(province: String, city: String) -> Self {
        Self {
            provinces: vec![province.clone()],
            cities: vec![city.clone()],
            can_write: true,
            skip_province_list: true,
            skip_city_list: true,
            locked_province: Some(province),
            locked_city: Some(city),
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
/// 中文注释:ShengAdmin 缺 admin_province 或 ShiAdmin 缺 admin_city 时,会
/// fallback 到最严格的"零范围"(provinces=["<INVALID>"]),这样过滤后返回空列表,
/// 避免误放行。调用方应当在 require_admin_* 里先校验必要字段。
pub fn get_visible_scope(ctx: &AdminAuthContext) -> VisibleScope {
    match ctx.role {
        AdminRole::KeyAdmin => VisibleScope::key_admin(),
        AdminRole::ShengAdmin => {
            let province = ctx
                .admin_province
                .clone()
                .unwrap_or_else(|| "__SHENG_ADMIN_MISSING_PROVINCE__".to_string());
            VisibleScope::sheng_admin(province)
        }
        AdminRole::ShiAdmin => {
            let province = ctx
                .admin_province
                .clone()
                .unwrap_or_else(|| "__SHI_ADMIN_MISSING_PROVINCE__".to_string());
            let city = ctx
                .admin_city
                .clone()
                .unwrap_or_else(|| "__SHI_ADMIN_MISSING_CITY__".to_string());
            VisibleScope::shi_admin(province, city)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_admin_sees_all() {
        let s = VisibleScope::key_admin();
        assert!(s.includes_province("任意省"));
        assert!(s.includes_city("任意市"));
        assert!(!s.skip_province_list);
    }

    #[test]
    fn sheng_admin_limited_to_province() {
        let s = VisibleScope::sheng_admin("安徽省".to_string());
        assert!(s.includes_province("安徽省"));
        assert!(!s.includes_province("广东省"));
        assert!(s.includes_city("合肥市"));
        assert!(s.skip_province_list);
        assert!(!s.skip_city_list);
    }

    #[test]
    fn shi_admin_limited_to_city() {
        let s = VisibleScope::shi_admin("安徽省".to_string(), "合肥市".to_string());
        assert!(s.includes_province("安徽省"));
        assert!(!s.includes_province("广东省"));
        assert!(s.includes_city("合肥市"));
        assert!(!s.includes_city("芜湖市"));
        assert!(s.skip_province_list);
        assert!(s.skip_city_list);
    }
}
