// 中文注释:任务卡 1 基础设施,部分 API 在任务卡 2 才会有调用点。
#![allow(dead_code)]

//! A3 主体属性枚举(sfid 号第一段)
//!
//! 中文注释:
//! - GMR 公民人:自然出生、取得国籍的人
//! - ZRR 自然人:未取得国籍的其他国家自然人
//! - ZNR 智能人:智能体
//! - GFR 公法人:政府/立法/司法/监察/教育委员会/储备委员会等公权机构
//! - SFR 私法人:注册型公司/社团等私权机构
//! - FFR 非法人:无独立法人地位的组织

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum A3 {
    GMR,
    ZRR,
    ZNR,
    GFR,
    SFR,
    FFR,
}

impl A3 {
    /// 从字符串解析(兼容英文代码和中文全称)。
    /// 与 legacy `resolve_a3(&str) -> Result<&'static str>` 语义等价。
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "GMR" | "公民人" => Some(Self::GMR),
            "ZRR" | "自然人" => Some(Self::ZRR),
            "ZNR" | "智能人" => Some(Self::ZNR),
            "GFR" | "公法人" => Some(Self::GFR),
            "SFR" | "私法人" => Some(Self::SFR),
            "FFR" | "非法人" => Some(Self::FFR),
            _ => None,
        }
    }

    /// 返回 sfid 码里使用的 3 字符代码(与 legacy `resolve_a3` 返回值一致)。
    pub fn as_code(self) -> &'static str {
        match self {
            Self::GMR => "GMR",
            Self::ZRR => "ZRR",
            Self::ZNR => "ZNR",
            Self::GFR => "GFR",
            Self::SFR => "SFR",
            Self::FFR => "FFR",
        }
    }

    /// 中文显示标签(用于 UI / 日志)。
    pub fn label_zh(self) -> &'static str {
        match self {
            Self::GMR => "公民人",
            Self::ZRR => "自然人",
            Self::ZNR => "智能人",
            Self::GFR => "公法人",
            Self::SFR => "私法人",
            Self::FFR => "非法人",
        }
    }
}

/// 返回全部 6 种 A3 枚举(用于前端下拉 / 分类初始化)。
pub fn all_a3() -> &'static [A3] {
    &[A3::GMR, A3::ZRR, A3::ZNR, A3::GFR, A3::SFR, A3::FFR]
}

/// 中文注释:legacy 字符串接口,供 `generator.rs` 继续调用保持行为不变。
/// 新代码应使用 `A3::from_str` + `.as_code()`。
pub(super) fn resolve_a3(a3: &str) -> Result<&'static str, &'static str> {
    A3::from_str(a3)
        .map(|v| v.as_code())
        .ok_or("a3 must be one of GMR/ZRR/ZNR/GFR/SFR/FFR")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_english_and_chinese() {
        assert_eq!(A3::from_str("GFR"), Some(A3::GFR));
        assert_eq!(A3::from_str("公法人"), Some(A3::GFR));
        assert_eq!(A3::from_str(" gfr "), None); // 必须大写
        assert_eq!(A3::from_str("xyz"), None);
    }

    #[test]
    fn as_code_matches_legacy() {
        for a3 in all_a3() {
            assert_eq!(resolve_a3(a3.as_code()).unwrap(), a3.as_code());
        }
    }
}
