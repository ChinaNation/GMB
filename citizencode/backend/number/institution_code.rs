#![allow(dead_code)]

//! 机构类型代码枚举(cid 号第三段 T2 部分)
//!
//! 中文注释:
//! - ZG 中国      — 人类主体来源分类
//! - ZF 政府      — 政府机关
//! - LF 立法院
//! - SF 司法院
//! - JC 监察院
//! - JY 教育委员会
//! - CB 储备委员会
//! - CH 储备银行
//! - TG 他国      — 人类主体来源分类
//! - GT 个体经营
//! - GP 无限合伙
//! - LP 有限合伙
//! - GQ 股权公司
//! - GF 股份公司
//! - GY 公益组织
//! - AS 注册协会
//!
//! 不同主体属性对 InstitutionCode 有硬约束(见 `generator.rs` 里的 generate_cid_number)。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum InstitutionCode {
    ZG,
    ZF,
    LF,
    SF,
    JC,
    JY,
    CB,
    CH,
    TG,
    GT,
    GP,
    LP,
    GQ,
    GF,
    GY,
    AS,
}

impl InstitutionCode {
    /// 从字符串解析英文代码或中文全称。
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "ZG" | "中国" => Some(Self::ZG),
            "ZF" | "政府" => Some(Self::ZF),
            "LF" | "立法院" => Some(Self::LF),
            "SF" | "司法院" => Some(Self::SF),
            "JC" | "监察院" => Some(Self::JC),
            "JY" | "教育委员会" | "公民教育委员会" => Some(Self::JY),
            "CB" | "储备委员会" | "公民储备委员会" => Some(Self::CB),
            "CH" | "储备银行" | "公民储备银行" => Some(Self::CH),
            "TG" | "他国" => Some(Self::TG),
            "GT" | "个体经营" => Some(Self::GT),
            "GP" | "无限合伙" => Some(Self::GP),
            "LP" | "有限合伙" => Some(Self::LP),
            "GQ" | "股权公司" | "股权有限公司" | "有限责任公司" => Some(Self::GQ),
            "GF" | "股份公司" | "股份有限公司" => Some(Self::GF),
            "GY" | "公益组织" => Some(Self::GY),
            "AS" | "注册协会" => Some(Self::AS),
            _ => None,
        }
    }

    /// 返回 cid 码里使用的 2 字符代码。
    pub fn as_code(self) -> &'static str {
        match self {
            Self::ZG => "ZG",
            Self::ZF => "ZF",
            Self::LF => "LF",
            Self::SF => "SF",
            Self::JC => "JC",
            Self::JY => "JY",
            Self::CB => "CB",
            Self::CH => "CH",
            Self::TG => "TG",
            Self::GT => "GT",
            Self::GP => "GP",
            Self::LP => "LP",
            Self::GQ => "GQ",
            Self::GF => "GF",
            Self::GY => "GY",
            Self::AS => "AS",
        }
    }

    /// 中文显示标签(用于 UI / 日志)。
    pub fn label_zh(self) -> &'static str {
        match self {
            Self::ZG => "中国",
            Self::ZF => "政府",
            Self::LF => "立法院",
            Self::SF => "司法院",
            Self::JC => "监察院",
            Self::JY => "教育委员会",
            Self::CB => "储备委员会",
            Self::CH => "储备银行",
            Self::TG => "他国",
            Self::GT => "个体经营",
            Self::GP => "无限合伙",
            Self::LP => "有限合伙",
            Self::GQ => "股权公司",
            Self::GF => "股份公司",
            Self::GY => "公益组织",
            Self::AS => "注册协会",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_english_and_chinese() {
        assert_eq!(InstitutionCode::from_str("ZF"), Some(InstitutionCode::ZF));
        assert_eq!(InstitutionCode::from_str("政府"), Some(InstitutionCode::ZF));
        assert_eq!(
            InstitutionCode::from_str("公民储备委员会"),
            Some(InstitutionCode::CB)
        );
        assert_eq!(InstitutionCode::from_str("xyz"), None);
    }
}
