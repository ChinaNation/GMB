#![allow(dead_code)]

//! 机构分类 — 由机构码(InstitutionCode)直接派生,供前端按 tab 过滤和按角色授权。
//!
//! 中文注释:主体属性(旧 K1)已从 CID 号删除,机构类别一律由机构码判定,不再有独立的
//! 主体属性枚举。分类规则由 `classify(code, cid_full_name)` 决定:
//!
//! - PublicSecurity     公安局      公法人 + 专用公安局码 Cpol
//! - GovInstitution     公权机构    公法人 + 其他(非 Cpol)
//! - PrivateInstitution 私权机构    私法人 / 非法人

use serde::{Deserialize, Serialize};

use crate::number::code::InstitutionCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstitutionCategory {
    /// 公安局(公法人 + 专用公安局码 Cpol)
    PublicSecurity,
    /// 公权机构(公法人类,但不是公安局的那部分)
    GovInstitution,
    /// 私权机构(私法人 / 非法人)
    PrivateInstitution,
}

impl InstitutionCategory {
    /// 中文显示标签。
    pub fn label_zh(self) -> &'static str {
        match self {
            Self::PublicSecurity => "公安局",
            Self::GovInstitution => "公权机构",
            Self::PrivateInstitution => "私权机构",
        }
    }
}

/// 中文注释:公安局确定性目录使用"xx市公安局"命名,这里只保存稳定后缀。
pub const PUBLIC_SECURITY_INSTITUTION_SUFFIX: &str = "公安局";

/// 按机构码决定机构分类。规则优先级:公安局 > 公权机构 > 私权机构。
/// `cid_full_name` 仅保留参数兼容,不参与判定(公安局已有专用码 Cpol)。
///
/// 返回 None:机构码不是注册型机构(个人主体 CTZN/NATP/SMTP、个人多签 PMUL)。
pub fn classify(code: InstitutionCode, _cid_full_name: &str) -> Option<InstitutionCategory> {
    if code.is_person() || code == InstitutionCode::Pmul {
        return None;
    }
    if code == InstitutionCode::Cpol {
        return Some(InstitutionCategory::PublicSecurity);
    }
    if code.is_public_legal() {
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
    fn public_security_by_dedicated_code() {
        assert_eq!(
            classify(InstitutionCode::Cpol, "广州市公安局"),
            Some(InstitutionCategory::PublicSecurity)
        );
    }

    #[test]
    fn gov_institution_is_not_public_security() {
        assert_eq!(
            classify(InstitutionCode::Cgov, "某某市政府"),
            Some(InstitutionCategory::GovInstitution)
        );
        assert_eq!(
            classify(InstitutionCode::Plg, "某省立法院"),
            Some(InstitutionCategory::GovInstitution)
        );
    }

    #[test]
    fn private_institution_for_private_and_unincorporated() {
        assert_eq!(
            classify(InstitutionCode::Sfgq, "某股权公司"),
            Some(InstitutionCategory::PrivateInstitution)
        );
        assert_eq!(
            classify(InstitutionCode::Unin, "某非法人组织"),
            Some(InstitutionCategory::PrivateInstitution)
        );
    }

    #[test]
    fn person_types_return_none() {
        assert_eq!(classify(InstitutionCode::Ctzn, "任意"), None);
        assert_eq!(classify(InstitutionCode::Natp, "任意"), None);
        assert_eq!(classify(InstitutionCode::Smtp, "任意"), None);
        assert_eq!(classify(InstitutionCode::Pmul, "任意"), None);
    }
}
