//! 按 VisibleScope 过滤泛型列表
//!
//! 中文注释:凡带 province/city 字段的记录类型都可以 impl `HasProvinceCity`,
//! 然后通过 `filter_by_scope(rows, &scope)` 自动过滤。避免每个 list handler
//! 都手写一遍省/市过滤逻辑。

use crate::scope::rules::VisibleScope;

/// 记录类型实现此 trait 后可被 filter_by_scope 处理。
pub trait HasProvinceCity {
    fn province(&self) -> &str;
    /// 市为空字符串时视为"不限市",scope 检查时会放行。
    fn city(&self) -> &str;
}

/// 按 scope 过滤列表,返回范围内的记录。
pub fn filter_by_scope<T: HasProvinceCity + Clone>(rows: &[T], scope: &VisibleScope) -> Vec<T> {
    rows.iter()
        .filter(|r| scope.includes_province(r.province()) && scope.includes_city(r.city()))
        .cloned()
        .collect()
}

/// 按 scope 过滤 HashMap,返回范围内的记录 Vec。
pub fn filter_map_by_scope<T: HasProvinceCity + Clone, K>(
    map: &std::collections::HashMap<K, T>,
    scope: &VisibleScope,
) -> Vec<T> {
    map.values()
        .filter(|r| scope.includes_province(r.province()) && scope.includes_city(r.city()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Row {
        province: String,
        city: String,
    }

    impl HasProvinceCity for Row {
        fn province(&self) -> &str {
            &self.province
        }
        fn city(&self) -> &str {
            &self.city
        }
    }

    fn row(p: &str, c: &str) -> Row {
        Row {
            province: p.into(),
            city: c.into(),
        }
    }

    #[test]
    fn key_admin_sees_all() {
        let rows = vec![row("安徽省", "合肥市"), row("广东省", "广州市")];
        let scope = VisibleScope::key_admin();
        let filtered = filter_by_scope(&rows, &scope);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn sheng_admin_only_own_province() {
        let rows = vec![
            row("安徽省", "合肥市"),
            row("安徽省", "芜湖市"),
            row("广东省", "广州市"),
        ];
        let scope = VisibleScope::sheng_admin("安徽省".to_string());
        let filtered = filter_by_scope(&rows, &scope);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.province == "安徽省"));
    }

    #[test]
    fn shi_admin_only_own_city() {
        let rows = vec![
            row("安徽省", "合肥市"),
            row("安徽省", "芜湖市"),
            row("广东省", "广州市"),
        ];
        let scope = VisibleScope::shi_admin("安徽省".to_string(), "合肥市".to_string());
        let filtered = filter_by_scope(&rows, &scope);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].city, "合肥市");
    }
}
