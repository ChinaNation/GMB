//! 私权机构清算行资格规则。
//!
//! 中文注释:清算行资格属于私权机构业务,公权机构和公共主体服务不保存该规则。

use crate::subjects::model::MultisigInstitution;

// ─── 清算行资格白名单(2026-04-24, ADR-007) ─────────────────────
//
// 仅"私法人股份公司"和"从属于私法人股份公司的非法人"有资格成为清算行。
// 详见 memory/04-decisions/ADR-007-clearing-bank-three-phase.md
// 与 memory/05-modules/sfid/clearing-bank-eligibility.md。
//
// 规则:
//   SFR + sub_type=JOINT_STOCK            → ✅
//   FFR + parent.SFR + parent.JOINT_STOCK → ✅
//   其他                                   → ❌

/// 清算行资格白名单判定:仅允许"私法人股份公司"及其下属非法人。
///
/// - `inst.a3 == "SFR"`:必须 `sub_type == "JOINT_STOCK"`
/// - `inst.a3 == "FFR"`:`parent` 必须存在,`parent.a3 == "SFR"` 且 `parent.sub_type == "JOINT_STOCK"`
/// - 其他 a3(GFR / SF 等):一律不允许
///
/// `parent` 由调用方按需提供(FFR 才需要;SFR / 其他可传 `None`)。
/// 跨省 parent 查询由 caller 通过 sharded_store.read_province 完成,
/// 本函数只做纯逻辑判定,便于单测。
#[allow(dead_code)]
pub fn is_clearing_bank_eligible(
    inst: &MultisigInstitution,
    parent: Option<&MultisigInstitution>,
) -> bool {
    match inst.a3.as_str() {
        "SFR" => inst.sub_type.as_deref() == Some("JOINT_STOCK"),
        "FFR" => match parent {
            Some(p) => p.a3 == "SFR" && p.sub_type.as_deref() == Some("JOINT_STOCK"),
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
    use crate::subjects::InstitutionChainStatus;

    // ─── 清算行资格白名单(ADR-007)─────────────────────────────

    /// 测试 fixture:按所需字段构造一个最小机构样本。
    /// `a3`/`sub_type`/`parent_sfid_number` 是判定关键字段,其他用合理默认值。
    fn fixture_institution(
        a3: &str,
        sub_type: Option<&str>,
        parent_sfid_number: Option<&str>,
    ) -> MultisigInstitution {
        MultisigInstitution {
            sfid_number: format!("{a3}-GD-CB01-000000000-20260101"),
            institution_name: Some("测试机构".to_string()),
            category: InstitutionCategory::PrivateInstitution,
            source: None,
            institution_level: None,
            a3: a3.to_string(),
            p1: if sub_type == Some("NON_PROFIT") {
                "0".to_string()
            } else {
                "1".to_string()
            },
            province: "广东省".to_string(),
            city: "广州市".to_string(),
            province_code: "GD".to_string(),
            city_code: "001".to_string(),
            institution_code: "CB".to_string(),
            sub_type: sub_type.map(|s| s.to_string()),
            parent_sfid_number: parent_sfid_number.map(|s| s.to_string()),
            chain_status: InstitutionChainStatus::NotRegistered,
            chain_tx_hash: None,
            chain_block_number: None,
            chain_synced_at: None,
            created_by: "test".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn clearing_bank_eligible_sfr_joint_stock() {
        // case 1: SFR + JOINT_STOCK → ✅
        let inst = fixture_institution("SFR", Some("JOINT_STOCK"), None);
        assert!(is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_sfr_limited_liability_rejected() {
        // case 2: SFR + LIMITED_LIABILITY → ❌
        let inst = fixture_institution("SFR", Some("LIMITED_LIABILITY"), None);
        assert!(!is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_sfr_non_profit_rejected() {
        // case 3: SFR + NON_PROFIT → ❌
        let inst = fixture_institution("SFR", Some("NON_PROFIT"), None);
        assert!(!is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_ffr_with_jointstock_parent() {
        // case 4: FFR + parent(SFR + JOINT_STOCK) → ✅
        let parent = fixture_institution("SFR", Some("JOINT_STOCK"), None);
        let inst = fixture_institution("FFR", None, Some(&parent.sfid_number));
        assert!(is_clearing_bank_eligible(&inst, Some(&parent)));
    }

    #[test]
    fn clearing_bank_eligible_ffr_with_non_jointstock_parent_rejected() {
        // case 5: FFR + parent(SFR + LIMITED_LIABILITY) → ❌
        let parent = fixture_institution("SFR", Some("LIMITED_LIABILITY"), None);
        let inst = fixture_institution("FFR", None, Some(&parent.sfid_number));
        assert!(!is_clearing_bank_eligible(&inst, Some(&parent)));
    }

    #[test]
    fn clearing_bank_eligible_ffr_without_parent_rejected() {
        // case 6: FFR + 缺 parent(查不到 / 未设置 parent_sfid_number) → ❌
        let inst = fixture_institution("FFR", None, None);
        assert!(!is_clearing_bank_eligible(&inst, None));
    }

    #[test]
    fn clearing_bank_eligible_other_a3_rejected() {
        // GFR / SF 等其他 a3 一律 ❌
        let gfr = fixture_institution("GFR", None, None);
        assert!(!is_clearing_bank_eligible(&gfr, None));
        let sf = fixture_institution("SF", None, None);
        assert!(!is_clearing_bank_eligible(&sf, None));
    }

    #[test]
    fn clearing_bank_eligible_ffr_with_gfr_parent_rejected() {
        // FFR 即使 parent 是 GFR 也不允许(必须 SFR + JOINT_STOCK)
        let parent = fixture_institution("GFR", None, None);
        let inst = fixture_institution("FFR", None, Some(&parent.sfid_number));
        assert!(!is_clearing_bank_eligible(&inst, Some(&parent)));
    }
}
