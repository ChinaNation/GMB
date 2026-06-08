//! 私权机构清算行资格规则。
//!
//! 中文注释:清算行资格属于私权机构业务,公权机构和公共主体服务不保存该规则。

use crate::subjects::model::Institution;

// ─── 清算行资格白名单(2026-04-24, ADR-007) ─────────────────────
//
// 仅"私法人股份公司"和"从属于私法人股份公司的非法人"有资格成为清算行。
// 详见 memory/04-decisions/ADR-007-clearing-bank-three-phase.md
// 与 memory/05-modules/sfid/clearing-bank-eligibility.md。
//
// 规则:
//   S + sub_type=JOINT_STOCK            → ✅
//   F + parent.S + parent.JOINT_STOCK → ✅
//   其他                                   → ❌

/// 清算行资格白名单判定:仅允许"私法人股份公司"及其下属非法人。
///
/// - `inst.subject_property == "S"`:必须 `sub_type == "JOINT_STOCK"`
/// - `inst.subject_property == "F"`:`parent` 必须存在,`parent.subject_property == "S"` 且 `parent.sub_type == "JOINT_STOCK"`
/// - 其他 subject_property:一律不允许
///
/// `parent` 由调用方按需提供(F 才需要;S / 其他可传 `None`)。
/// 跨省 parent 查询由调用方通过 `subjects` 结构化表完成,
/// 本函数只做纯逻辑判定,便于单测。
#[allow(dead_code)]
pub fn is_clearing_bank_eligible(
    inst: &Institution,
    parent: Option<&Institution>,
) -> bool {
    match inst.subject_property.as_str() {
        "S" => inst.sub_type.as_deref() == Some("JOINT_STOCK"),
        "F" => match parent {
            Some(p) => p.subject_property == "S" && p.sub_type.as_deref() == Some("JOINT_STOCK"),
            None => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;

    use crate::number::InstitutionCategory;
    // ─── 清算行资格白名单(ADR-007)─────────────────────────────

    /// 测试 fixture:按所需字段构造一个最小机构样本。
    /// `subject_property`/`sub_type`/`parent_sfid_number` 是判定关键字段,其他用合理默认值。
    fn fixture_institution(
        subject_property: &str,
        sub_type: Option<&str>,
        parent_sfid_number: Option<&str>,
    ) -> Institution {
        Institution {
            sfid_number: match subject_property {
                "F" => "AH001-FCB0P-123456789-2026".to_string(),
                "G" => "AH001-GCB0V-123456789-2026".to_string(),
                _ => "AH001-SCB0V-123456789-2026".to_string(),
            },
            institution_name: Some("测试机构".to_string()),
            sfid_name: Some("测试机构".to_string()),
            short_name: Some("测试机构".to_string()),
            status: "ACTIVE".to_string(),
            category: InstitutionCategory::PrivateInstitution,
            subject_property: subject_property.to_string(),
            p1: if sub_type == Some("NON_PROFIT") {
                "0".to_string()
            } else {
                "1".to_string()
            },
            province: "广东省".to_string(),
            city: "广州市".to_string(),
            town: String::new(),
            province_code: "GD".to_string(),
            city_code: "001".to_string(),
            town_code: String::new(),
            institution_code: "CB".to_string(),
            org_code: None,
            sub_type: sub_type.map(|s| s.to_string()),
            parent_sfid_number: parent_sfid_number.map(|s| s.to_string()),
            legal_rep_name: None,
            legal_rep_sfid_number: None,
            legal_rep_photo_path: None,
            legal_rep_photo_name: None,
            legal_rep_photo_mime: None,
            legal_rep_photo_size: None,
            created_by: "test".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn clearing_bank_eligible_s_subject_joint_stock() {
        // case 1: S + JOINT_STOCK → ✅
        let inst = fixture_institution("S", Some("JOINT_STOCK"), None);
        assert!(is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_s_subject_limited_liability_rejected() {
        // case 2: S + LIMITED_LIABILITY → ❌
        let inst = fixture_institution("S", Some("LIMITED_LIABILITY"), None);
        assert!(!is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_s_subject_non_profit_rejected() {
        // case 3: S + NON_PROFIT → ❌
        let inst = fixture_institution("S", Some("NON_PROFIT"), None);
        assert!(!is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_f_subject_with_jointstock_parent() {
        // case 4: F + parent(S + JOINT_STOCK) → ✅
        let parent = fixture_institution("S", Some("JOINT_STOCK"), None);
        let inst = fixture_institution("F", None, Some(&parent.sfid_number));
        assert!(is_clearing_bank_eligible(&inst, Some(&parent)));
    }

    #[test]
    fn clearing_bank_eligible_f_subject_with_non_jointstock_parent_rejected() {
        // case 5: F + parent(S + LIMITED_LIABILITY) → ❌
        let parent = fixture_institution("S", Some("LIMITED_LIABILITY"), None);
        let inst = fixture_institution("F", None, Some(&parent.sfid_number));
        assert!(!is_clearing_bank_eligible(&inst, Some(&parent)));
    }

    #[test]
    fn clearing_bank_eligible_f_subject_without_parent_rejected() {
        // case 6: F + 缺 parent(查不到 / 未设置 parent_sfid_number) → ❌
        let inst = fixture_institution("F", None, None);
        assert!(!is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_other_subject_rejected() {
        // G 等其他 subject_property 一律拒绝。
        let gov_inst = fixture_institution("G", None, None);
        assert!(!is_clearing_bank_eligible(&gov_inst, None));
        let sf = fixture_institution("SF", None, None);
        assert!(!is_clearing_bank_eligible(&sf, None));
    }

    #[test]
    fn clearing_bank_eligible_f_subject_with_g_subject_parent_rejected() {
        // F 即使 parent 是 G 也不允许(必须 S + JOINT_STOCK)
        let parent = fixture_institution("G", None, None);
        let inst = fixture_institution("F", None, Some(&parent.sfid_number));
        assert!(!is_clearing_bank_eligible(&inst, Some(&parent)));
    }
}
