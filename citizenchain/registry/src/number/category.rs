#![allow(dead_code)]

//! 机构分类 — 由机构码(InstitutionCode)直接派生,供前端按 tab 过滤和按角色授权。
//!
//! 中文注释:机构类别一律由机构码判定。分类规则由 `classify(code, cid_full_name)` 决定:
//!
//! - GovInstitution     公权机构    公法人(含市公安局 Cpol,公安局不再单列分类)
//! - PrivateInstitution 私权机构    私法人 / 非法人
//!
//! 公权机构只分「注册局(FRG/CREG,独立 tab)」与「非注册局(其余全部,公权机构 tab)」,
//! 注册局之分由机构码区分,公安局与民生厅同列。

use serde::{Deserialize, Serialize};

use crate::number::code::{self, InstitutionCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstitutionCategory {
    /// 公权机构(公法人类,含市公安局)
    GovInstitution,
    /// 私权机构(私法人 / 非法人)
    PrivateInstitution,
}

impl InstitutionCategory {
    /// 机构分类中文名称。
    pub fn category_name_zh(self) -> &'static str {
        match self {
            Self::GovInstitution => "公权机构",
            Self::PrivateInstitution => "私权机构",
        }
    }
}

/// 按机构码决定机构分类。规则优先级:公权机构 > 私权机构。
/// `cid_full_name` 是统一分类入口参数,当前不参与判定。
///
/// 返回 None:机构码不是注册型机构(个人主体 CTZN/NATP/SMTP、个人多签 PMUL)。
pub fn classify(code: InstitutionCode, _cid_full_name: &str) -> Option<InstitutionCategory> {
    if code::is_person(&code) || code == code::PMUL {
        return None;
    }
    // 中文注释:市公安局(CPOL)是公法人,回归普通公权机构,不再单列分类。
    if code::is_public_legal(&code) || code::is_city_police(&code) {
        Some(InstitutionCategory::GovInstitution)
    } else {
        // 私法人 / 非法人
        Some(InstitutionCategory::PrivateInstitution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_police_is_gov_institution() {
        // 中文注释:市公安局(CPOL)已折叠为普通公权机构,不再单列分类。
        assert_eq!(
            classify(*b"CPOL", "广州市公民安全局"),
            Some(InstitutionCategory::GovInstitution)
        );
    }

    #[test]
    fn gov_institution_for_public_legal() {
        assert_eq!(
            classify(*b"CGOV", "某某市政府"),
            Some(InstitutionCategory::GovInstitution)
        );
        assert_eq!(
            classify(*b"PLG\0", "某省立法院"),
            Some(InstitutionCategory::GovInstitution)
        );
    }

    #[test]
    fn private_institution_for_private_and_unincorporated() {
        assert_eq!(
            classify(*b"SFGQ", "某股权公司"),
            Some(InstitutionCategory::PrivateInstitution)
        );
        assert_eq!(
            classify(*b"UNIN", "某非法人组织"),
            Some(InstitutionCategory::PrivateInstitution)
        );
    }

    #[test]
    fn person_types_return_none() {
        assert_eq!(classify(*b"CTZN", "任意"), None);
        assert_eq!(classify(*b"NATP", "任意"), None);
        assert_eq!(classify(*b"SMTP", "任意"), None);
        assert_eq!(classify(code::PMUL, "任意"), None);
    }
}
