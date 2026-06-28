#![allow(dead_code)]

//! 机构展示分类 — 由机构码(InstitutionCode)直接派生,供前端按 tab 过滤。
//!
//! 中文注释:机构类别一律由机构码判定。分类规则由 `classify(code, cid_full_name)` 决定:
//!
//! - GovInstitution     公权机构 tab 桶    公法人机构;
//! - PrivateInstitution 私权机构 tab 桶    私法人机构,以及父级未知时的非法人初始落位。
//!
//! 注意:这不是法律主体分类。公法人、私法人、非法人、公民人、自然人、智能人
//! 是独立主体类型;非法人可从属于公法人或私法人,具体列表归属由 subjects 的
//! 父级属性规则分流。

use serde::{Deserialize, Serialize};

use crate::cid::code::{self, InstitutionCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstitutionCategory {
    /// 公权机构 tab 桶(公法人类)。
    GovInstitution,
    /// 私权机构 tab 桶(私法人类;非法人最终按父级属性分流)。
    PrivateInstitution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubjectLegalKind {
    /// 公法人。
    PublicLegal,
    /// 私法人。
    PrivateLegal,
    /// 非法人。
    Unincorporated,
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
    if code::is_person_code(&code) || code == code::PMUL {
        return None;
    }
    if code::is_public_legal_code(&code) {
        Some(InstitutionCategory::GovInstitution)
    } else {
        // 中文注释:这里只给无父级上下文的 tab 初始桶;非法人真实归属由父级公/私法人决定。
        Some(InstitutionCategory::PrivateInstitution)
    }
}

/// 按机构码派生法律主体类型。个人主体由公民模块处理,这里仅返回机构型主体。
pub fn legal_kind(code: InstitutionCode) -> Option<SubjectLegalKind> {
    if code::is_unincorporated_code(&code) {
        Some(SubjectLegalKind::Unincorporated)
    } else if code::is_public_legal_code(&code) {
        Some(SubjectLegalKind::PublicLegal)
    } else if code::is_private_legal_code(&code) {
        Some(SubjectLegalKind::PrivateLegal)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_police_is_gov_institution() {
        // 中文注释:CPOL 由公法人机构码自然归入公权机构,不再单列分类。
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
    fn private_tab_bucket_for_private_and_unincorporated_without_parent_context() {
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
    fn legal_kind_keeps_unincorporated_independent() {
        assert_eq!(legal_kind(*b"CGOV"), Some(SubjectLegalKind::PublicLegal));
        assert_eq!(legal_kind(*b"SFGQ"), Some(SubjectLegalKind::PrivateLegal));
        assert_eq!(legal_kind(*b"UNIN"), Some(SubjectLegalKind::Unincorporated));
    }

    #[test]
    fn person_types_return_none() {
        assert_eq!(classify(*b"CTZN", "任意"), None);
        assert_eq!(classify(*b"NATP", "任意"), None);
        assert_eq!(classify(*b"SMTP", "任意"), None);
        assert_eq!(classify(code::PMUL, "任意"), None);
    }
}
