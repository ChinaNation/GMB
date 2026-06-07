#![allow(dead_code)]

//! 机构类型代码枚举(sfid 号第三段 T2 部分)
//!
//! 中文注释:
//! - ZG 中国      — 国家整体
//! - ZF 政府      — 政府机关
//! - LF 立法院
//! - SF 司法院
//! - JC 监察院
//! - JY 教育委员会
//! - CB 储备委员会
//! - CH 储备银行
//! - TG 他国      — 其他国家
//!
//! 不同主体属性对 InstitutionCode 有硬约束(见 `generator.rs` 里的 generate_sfid_number)。
//! 中文注释:私权机构允许 `JY` 表示教育委员会类型学校机构,不是学校内部组织。

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
            _ => None,
        }
    }

    /// 返回 sfid 码里使用的 2 字符代码。
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
