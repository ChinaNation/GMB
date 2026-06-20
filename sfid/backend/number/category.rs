#![allow(dead_code)]

//! 主体属性与机构分类枚举 — SFID 编号协议和机构目录共用的基础分类。
//!
//! 中文注释:
//! 新版 SFID 编号把主体属性放入第二段 `K3P1C1` 的首字符 `K1`。这里集中维护
//! `K1` 的合法值,并把所有注册型多签机构按业务域分为 3 类,便于前端按 tab
//! 过滤和按角色授权。分类规则由 `classify(subject_property, code, sfid_full_name)` 决定:
//!
//! - PublicSecurity     公安局      公法人 + ZF + sfid_full_name 以"公安局"结尾
//! - GovInstitution     公权机构    公法人 + 其他(非公安局)
//! - PrivateInstitution 私权机构    私法人 / 非法人

use serde::{Deserialize, Serialize};

use crate::number::institution_code::InstitutionCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SubjectProperty {
    /// 公民
    M,
    /// 自然人
    Z,
    /// 智能人
    N,
    /// 公法人
    G,
    /// 私法人
    S,
    /// 非法人
    F,
}

impl SubjectProperty {
    /// 从字符串解析新版主体属性代码或中文全称。
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "M" | "公民" => Some(Self::M),
            "Z" | "自然人" => Some(Self::Z),
            "N" | "智能人" => Some(Self::N),
            "G" | "公法人" => Some(Self::G),
            "S" | "私法人" => Some(Self::S),
            "F" | "非法人" => Some(Self::F),
            _ => None,
        }
    }

    /// 返回 SFID 编号 `K1` 使用的 1 字符代码。
    pub fn as_code(self) -> &'static str {
        match self {
            Self::M => "M",
            Self::Z => "Z",
            Self::N => "N",
            Self::G => "G",
            Self::S => "S",
            Self::F => "F",
        }
    }

    /// 中文显示标签(用于 UI / 日志)。
    pub fn label_zh(self) -> &'static str {
        match self {
            Self::M => "公民",
            Self::Z => "自然人",
            Self::N => "智能人",
            Self::G => "公法人",
            Self::S => "私法人",
            Self::F => "非法人",
        }
    }
}

/// 返回全部 6 种主体属性枚举(用于前端下拉 / 分类初始化)。
pub fn all_subject_properties() -> &'static [SubjectProperty] {
    &[
        SubjectProperty::M,
        SubjectProperty::Z,
        SubjectProperty::N,
        SubjectProperty::G,
        SubjectProperty::S,
        SubjectProperty::F,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstitutionCategory {
    /// 公安局(公法人 + ZF + 名称以"公安局"结尾)
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

/// 按 (subject_property, code, sfid_full_name) 三元组决定机构分类。
/// 规则优先级:公安局 > 公权机构 > 私权机构。
///
/// 返回 None 的情况:主体属性与 code 的组合不属于任何一个注册型多签机构
/// (例如公民 + ZG 是个人身份,不是机构)。
pub fn classify(
    subject_property: SubjectProperty,
    code: InstitutionCode,
    sfid_full_name: &str,
) -> Option<InstitutionCategory> {
    match subject_property {
        SubjectProperty::G => {
            // 公法人类:区分公安局和其他公权机构。
            if code == InstitutionCode::ZF
                && sfid_full_name.ends_with(PUBLIC_SECURITY_INSTITUTION_SUFFIX)
            {
                Some(InstitutionCategory::PublicSecurity)
            } else {
                Some(InstitutionCategory::GovInstitution)
            }
        }
        SubjectProperty::S | SubjectProperty::F => Some(InstitutionCategory::PrivateInstitution),
        // 公民/自然人/智能人不是注册型多签机构。
        SubjectProperty::M | SubjectProperty::Z | SubjectProperty::N => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_security_exact_match() {
        assert_eq!(
            classify(SubjectProperty::G, InstitutionCode::ZF, "广州市公安局"),
            Some(InstitutionCategory::PublicSecurity)
        );
    }

    #[test]
    fn gov_institution_is_not_public_security() {
        // 同样公法人+ZF 但机构名不同 → 公权机构
        assert_eq!(
            classify(SubjectProperty::G, InstitutionCode::ZF, "某某市府"),
            Some(InstitutionCategory::GovInstitution)
        );
        // 公法人 + 非 ZF → 公权机构
        assert_eq!(
            classify(SubjectProperty::G, InstitutionCode::LF, "某立法机构"),
            Some(InstitutionCategory::GovInstitution)
        );
    }

    #[test]
    fn private_institution_for_private_and_unincorporated() {
        assert_eq!(
            classify(SubjectProperty::S, InstitutionCode::ZG, "某公司"),
            Some(InstitutionCategory::PrivateInstitution)
        );
        assert_eq!(
            classify(SubjectProperty::F, InstitutionCode::ZG, "某非法人组织"),
            Some(InstitutionCategory::PrivateInstitution)
        );
    }

    #[test]
    fn citizen_types_return_none() {
        assert_eq!(
            classify(SubjectProperty::M, InstitutionCode::ZG, "任意"),
            None
        );
        assert_eq!(
            classify(SubjectProperty::Z, InstitutionCode::TG, "任意"),
            None
        );
        assert_eq!(
            classify(SubjectProperty::N, InstitutionCode::ZG, "任意"),
            None
        );
    }

    #[test]
    fn subject_property_parses_current_codes_only() {
        assert_eq!(SubjectProperty::from_str("G"), Some(SubjectProperty::G));
        assert_eq!(
            SubjectProperty::from_str("公法人"),
            Some(SubjectProperty::G)
        );
        assert_eq!(SubjectProperty::from_str("g"), None);
        assert_eq!(SubjectProperty::from_str("xyz"), None);
    }
}
