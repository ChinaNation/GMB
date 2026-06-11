//! 非法人机构能力。
//!
//! 中文注释:非法人机构不是独立法人,必须从属于一个具有法人资格的主体(创建即挂,
//! 不存在未挂靠非法人)。公权机构和私权机构都可以拥有从属非法人机构,
//! 所以能力放在 `subjects/uninorg`。
//!
//! 本模块是非法人挂靠规则的单一权威源,创建(create_institution)、改挂
//! (update_institution)和所属法人搜索(search_parent_institutions 的 SQL 预过滤)
//! 三处必须与这里同源,缺一处就有绕过口:
//! - 父级属性:仅私法人(S)/公法人(G)可作所属法人;
//! - 代码一致性:F+JY(教育分校) ⇔ 父级是教育委员会学校(手动 JY 行);
//! - 地域规则:见 [`parent_locality_rule`];
//! - 盈利属性继承:见 [`inherited_p1`]。

pub(crate) fn is_unincorporated_subject(subject_property: &str) -> bool {
    subject_property == "F"
}

pub(crate) fn requires_parent(subject_property: &str) -> bool {
    is_unincorporated_subject(subject_property)
}

pub(crate) fn can_attach_to_parent_subject(parent_subject_property: &str) -> bool {
    matches!(parent_subject_property, "S" | "G")
}

pub(crate) fn parent_subject_requirement_message() -> &'static str {
    "所属法人必须是私法人(S)或公法人(G)"
}

/// 非法人落位地域规则(由所属法人的行政层级决定)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentLocalityRule {
    /// 全国不限(私法人父级、国家级公权机构父级)
    Nationwide,
    /// 同省(省级公权机构父级)
    SameProvince,
    /// 同市(市/镇级公权机构、手动公权机构、教育委员会学校父级)
    SameCity,
}

/// 判定父级是否教育委员会学校(手动 JY 行,公立 G+JY / 私立 S+JY)。
/// 自动生成的监管本体(公民教育委员会等)org_code 非空,不算学校。
pub(crate) fn parent_is_education_school(
    parent_institution_code: &str,
    parent_org_code: Option<&str>,
) -> bool {
    parent_institution_code == "JY" && parent_org_code.is_none()
}

/// 所属法人的地域规则:
/// - 私法人(S)父级 → 全国不限(唯一允许跨省市;S+JY 学校例外,分校与本部同市);
/// - 教育委员会学校父级 → 同市;
/// - 公法人(G)父级按行政层级:org_code 为空(手动公权机构)或 CITY_/TOWN_ 前缀 → 同市,
///   PROVINCE_ 前缀 → 同省,NATIONAL_/MINISTRY_/FEDERAL_ 前缀(国家级)→ 全国;
///   未知前缀防御性按最严的同市处理,不放权。
pub(crate) fn parent_locality_rule(
    parent_subject_property: &str,
    parent_institution_code: &str,
    parent_org_code: Option<&str>,
) -> ParentLocalityRule {
    if parent_is_education_school(parent_institution_code, parent_org_code) {
        return ParentLocalityRule::SameCity;
    }
    if parent_subject_property == "S" {
        return ParentLocalityRule::Nationwide;
    }
    // 公法人(G)按 org_code 前缀判层级
    match parent_org_code {
        None => ParentLocalityRule::SameCity,
        Some(code) if code.starts_with("PROVINCE_") => ParentLocalityRule::SameProvince,
        Some(code)
            if code.starts_with("NATIONAL_")
                || code.starts_with("MINISTRY_")
                || code.starts_with("FEDERAL_") =>
        {
            ParentLocalityRule::Nationwide
        }
        // CITY_/TOWN_ 及一切未知前缀 → 同市
        Some(_) => ParentLocalityRule::SameCity,
    }
}

/// 校验非法人落位省市是否符合所属法人的地域规则,违规时返回提示文案。
pub(crate) fn locality_violation(
    rule: ParentLocalityRule,
    parent_province: &str,
    parent_city: &str,
    f_province: &str,
    f_city: &str,
) -> Option<&'static str> {
    match rule {
        ParentLocalityRule::Nationwide => None,
        ParentLocalityRule::SameProvince => {
            if parent_province == f_province {
                None
            } else {
                Some("省级公权机构所属的非法人只能落位本省")
            }
        }
        ParentLocalityRule::SameCity => {
            if parent_province == f_province && parent_city == f_city {
                None
            } else {
                Some("该所属法人的非法人只能落位同一市")
            }
        }
    }
}

/// 非法人机构代码与父级类型一致性:教育分校(F+JY)的父级必须是学校本部,
/// 学校本部下也只能挂教育分校。违规时返回提示文案。
pub(crate) fn code_consistency_violation(
    f_institution_code: &str,
    parent_institution_code: &str,
    parent_org_code: Option<&str>,
) -> Option<&'static str> {
    let school_parent = parent_is_education_school(parent_institution_code, parent_org_code);
    if f_institution_code == "JY" && !school_parent {
        return Some("教育分校(教育委员会 JY)的所属法人必须是教育委员会学校");
    }
    if f_institution_code != "JY" && school_parent {
        return Some("教育委员会学校下只能挂教育分校(教育委员会 JY)");
    }
    None
}

/// 非法人盈利属性附属于所属法人:公法人父级恒非盈利(0),私法人父级继承其 p1。
pub(crate) fn inherited_p1(parent_subject_property: &str, parent_p1: &str) -> String {
    if parent_subject_property == "G" {
        "0".to_string()
    } else {
        parent_p1.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn education_school_parent_requires_same_city() {
        // 公立本部(G+JY 手动)和私立本部(S+JY 手动)都锁同市
        assert_eq!(
            parent_locality_rule("G", "JY", None),
            ParentLocalityRule::SameCity
        );
        assert_eq!(
            parent_locality_rule("S", "JY", None),
            ParentLocalityRule::SameCity
        );
    }

    #[test]
    fn private_parent_is_nationwide() {
        assert_eq!(
            parent_locality_rule("S", "ZG", None),
            ParentLocalityRule::Nationwide
        );
    }

    #[test]
    fn gov_parent_locality_follows_org_code_level() {
        // 手动公权机构(org_code 空) = 市级
        assert_eq!(
            parent_locality_rule("G", "ZF", None),
            ParentLocalityRule::SameCity
        );
        // 市级/镇级自动目录
        assert_eq!(
            parent_locality_rule("G", "ZF", Some("CITY_GOV")),
            ParentLocalityRule::SameCity
        );
        assert_eq!(
            parent_locality_rule("G", "ZF", Some("TOWN_GOV")),
            ParentLocalityRule::SameCity
        );
        // 监管本体(公民教育委员会)是市级公权机构,不是学校
        assert_eq!(
            parent_locality_rule("G", "JY", Some("CITY_EDU")),
            ParentLocalityRule::SameCity
        );
        // 省级
        assert_eq!(
            parent_locality_rule("G", "ZF", Some("PROVINCE_GOV")),
            ParentLocalityRule::SameProvince
        );
        // 国家级三类前缀
        for code in [
            "NATIONAL_LEGISLATURE",
            "MINISTRY_FOREIGN",
            "FEDERAL_REGISTRY",
        ] {
            assert_eq!(
                parent_locality_rule("G", "ZF", Some(code)),
                ParentLocalityRule::Nationwide
            );
        }
        // 未知前缀防御性收紧到同市
        assert_eq!(
            parent_locality_rule("G", "ZF", Some("PUBLIC_ORG")),
            ParentLocalityRule::SameCity
        );
    }

    #[test]
    fn locality_violation_checks_province_and_city() {
        assert!(locality_violation(
            ParentLocalityRule::Nationwide,
            "广东",
            "广州",
            "安徽",
            "合肥"
        )
        .is_none());
        assert!(locality_violation(
            ParentLocalityRule::SameProvince,
            "广东",
            "广州",
            "广东",
            "深圳"
        )
        .is_none());
        assert!(locality_violation(
            ParentLocalityRule::SameProvince,
            "广东",
            "广州",
            "安徽",
            "合肥"
        )
        .is_some());
        assert!(
            locality_violation(ParentLocalityRule::SameCity, "广东", "广州", "广东", "广州")
                .is_none()
        );
        assert!(
            locality_violation(ParentLocalityRule::SameCity, "广东", "广州", "广东", "深圳")
                .is_some()
        );
    }

    #[test]
    fn branch_school_code_must_match_school_parent() {
        // F+JY 必须挂学校本部
        assert!(code_consistency_violation("JY", "JY", None).is_none());
        assert!(code_consistency_violation("JY", "ZF", Some("CITY_GOV")).is_some());
        // 监管本体(org_code 非空)不算学校,F+JY 不能挂
        assert!(code_consistency_violation("JY", "JY", Some("CITY_EDU")).is_some());
        // 学校本部下只能挂分校
        assert!(code_consistency_violation("ZG", "JY", None).is_some());
        assert!(code_consistency_violation("ZG", "ZF", Some("CITY_GOV")).is_none());
    }

    #[test]
    fn p1_inherits_from_parent() {
        assert_eq!(inherited_p1("G", "0"), "0");
        assert_eq!(inherited_p1("G", "1"), "0"); // 公法人恒非盈利,容错
        assert_eq!(inherited_p1("S", "1"), "1");
        assert_eq!(inherited_p1("S", "0"), "0");
    }
}
