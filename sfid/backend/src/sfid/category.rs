// 中文注释:任务卡 1 基础设施,部分 API 在任务卡 2 才会有调用点。
#![allow(dead_code)]

//! 机构分类枚举 — 用于任务卡 2 的"机构 vs 账户"两层模型。
//!
//! 中文注释:
//! sfid 系统把所有注册型多签机构按业务域分为 3 类,便于前端按 tab 过滤和
//! 按角色授权。分类规则由 `classify(a3, code, institution_name)` 决定:
//!
//! - PublicSecurity  公安局        GFR + ZF + institution_name == "公民安全局"
//! - GovInstitution  公权机构      GFR + 其他(非公安局)
//! - PrivateInstitution 私权机构   SFR / FFR
//!
//! 本任务卡(任务卡 1)先建好枚举和函数占位,真正的调用点在任务卡 2 落地。

use serde::{Deserialize, Serialize};

use crate::sfid::a3::A3;
use crate::sfid::institution_code::InstitutionCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstitutionCategory {
    /// 公安局(GFR + ZF + institution_name == "公民安全局")
    PublicSecurity,
    /// 公权机构(GFR 类,但不是公安局的那部分)
    GovInstitution,
    /// 私权机构(SFR / FFR)
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

/// 中文注释:公安局硬编码的机构名称常量(任务卡 2 用来判定 category)。
pub const PUBLIC_SECURITY_INSTITUTION_NAME: &str = "公民安全局";

/// 按 (a3, code, institution_name) 三元组决定机构分类。
/// 规则优先级:公安局 > 公权机构 > 私权机构。
///
/// 返回 None 的情况:a3 与 code 的组合不属于任何一个注册型多签机构
/// (例如 GMR + ZG 是公民个人,不是机构)。
pub fn classify(
    a3: A3,
    code: InstitutionCode,
    institution_name: &str,
) -> Option<InstitutionCategory> {
    match a3 {
        A3::GFR => {
            // 公法人类:区分公安局和其他公权机构
            if code == InstitutionCode::ZF && institution_name == PUBLIC_SECURITY_INSTITUTION_NAME {
                Some(InstitutionCategory::PublicSecurity)
            } else {
                Some(InstitutionCategory::GovInstitution)
            }
        }
        A3::SFR | A3::FFR => Some(InstitutionCategory::PrivateInstitution),
        // 公民人/自然人/智能人不是机构
        A3::GMR | A3::ZRR | A3::ZNR => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_security_exact_match() {
        assert_eq!(
            classify(A3::GFR, InstitutionCode::ZF, "公民安全局"),
            Some(InstitutionCategory::PublicSecurity)
        );
    }

    #[test]
    fn gov_institution_is_not_public_security() {
        // 同样 GFR+ZF 但机构名不同 → 公权机构
        assert_eq!(
            classify(A3::GFR, InstitutionCode::ZF, "某某市府"),
            Some(InstitutionCategory::GovInstitution)
        );
        // GFR + 非 ZF → 公权机构
        assert_eq!(
            classify(A3::GFR, InstitutionCode::LF, "某立法机构"),
            Some(InstitutionCategory::GovInstitution)
        );
    }

    #[test]
    fn private_institution_for_sfr_ffr() {
        assert_eq!(
            classify(A3::SFR, InstitutionCode::ZG, "某公司"),
            Some(InstitutionCategory::PrivateInstitution)
        );
        assert_eq!(
            classify(A3::FFR, InstitutionCode::ZG, "某非法人组织"),
            Some(InstitutionCategory::PrivateInstitution)
        );
    }

    #[test]
    fn citizen_types_return_none() {
        assert_eq!(classify(A3::GMR, InstitutionCode::ZG, "任意"), None);
        assert_eq!(classify(A3::ZRR, InstitutionCode::TG, "任意"), None);
        assert_eq!(classify(A3::ZNR, InstitutionCode::ZG, "任意"), None);
    }
}
