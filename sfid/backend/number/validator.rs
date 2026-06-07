#![allow(dead_code)]

//! SFID 号格式校验
//!
//! 中文注释:
//! sfid 号结构(4 段,连字符分隔,ASCII 大写字母 + 数字):
//! ```
//! R5(5) - K3P1C1(5) - N9(9) - D4(4)
//! 例:LN001 - GCB05    - 944805165 - 2026
//! ```
//!
//! - R5:5 字符,省代码(2) + 市代码(3)
//! - K3:3 字符,K1 主体属性(M/Z/N/G/S/F) + T2 机构代码(ZG/ZF/LF/SF/JC/JY/CB/CH/TG)
//! - P1:1 字符,盈利属性(0/1)
//! - C1:1 字符,校验位,按原校验算法对 `R5 + K3 + P1 + N9 + D4` 生成
//! - N9:9 字符,全数字 hash 号
//! - D4:4 字符,生成年份 YYYY
//!
//! 容量分析:同 (主体属性, 省, 市, 机构, year) 5 元组下,n9 是 hash mod 10^9 = 10 亿桶。
//! 单省最大人口 1.5 亿(15% 填充),撞了由调用方 1000 次重试逃逸,基本不可能用尽。
//!
//! 链端 `register_sfid_institution` 对 sfid_number 的 BoundedVec 上限也是 96 字节,
//! 与本文件 SFID_NUMBER_MAX_BYTES 保持一致。

use crate::number::category::SubjectProperty;
use crate::number::institution_code::InstitutionCode;

const CHECKSUM_ALPHABET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub const SFID_NUMBER_SEGMENT_COUNT: usize = 4;
pub const SFID_NUMBER_SEGMENT_R5_LEN: usize = 5;
pub const SFID_NUMBER_SEGMENT_K3P1C1_LEN: usize = 5;
pub const SFID_NUMBER_SEGMENT_N9_LEN: usize = 9;
pub const SFID_NUMBER_SEGMENT_D4_LEN: usize = 4;
pub const SFID_NUMBER_MAX_BYTES: usize = 96;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfidNumberParts {
    pub r5: String,
    pub k1: String,
    pub t2: String,
    pub k3: String,
    pub p1: String,
    pub c1: char,
    pub n9: String,
    pub d4: String,
}

pub(crate) fn sfid_checksum(payload: &str) -> char {
    let mut total: usize = 0;
    for (idx, ch) in payload.chars().enumerate() {
        let pos = CHECKSUM_ALPHABET.find(ch).unwrap_or(0);
        total = (total + (idx + 1) * pos) % 36;
    }
    CHECKSUM_ALPHABET.as_bytes()[total] as char
}

/// 校验 sfid 号字符串格式,通过返回标准化后的字符串(trim + 保持大小写)。
/// 失败返回静态错误字符串,便于调用方直接透传给 HTTP 响应。
pub fn validate_sfid_number_format(raw: &str) -> Result<String, &'static str> {
    parse_sfid_number_parts(raw).map(|_| raw.trim().to_string())
}

/// 解析并校验新版 sfid_number,通过后返回拆分后的协议字段。
pub fn parse_sfid_number_parts(raw: &str) -> Result<SfidNumberParts, &'static str> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err("sfid_number is required");
    }
    if !normalized.is_ascii() {
        return Err("sfid_number must be ascii");
    }
    if normalized.len() > SFID_NUMBER_MAX_BYTES {
        return Err("sfid_number length exceeds chain max");
    }
    if normalized
        .bytes()
        .any(|b| !(b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-'))
    {
        return Err("sfid_number charset invalid");
    }
    let segments = normalized.split('-').collect::<Vec<_>>();
    if segments.len() != SFID_NUMBER_SEGMENT_COUNT {
        return Err("sfid_number format invalid");
    }
    if segments[0].len() != SFID_NUMBER_SEGMENT_R5_LEN
        || !segments[0]
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err("sfid_number r5 segment invalid");
    }
    if segments[1].len() != SFID_NUMBER_SEGMENT_K3P1C1_LEN
        || !segments[1]
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err("sfid_number k3p1c1 segment invalid");
    }
    let k3p1c1 = segments[1];
    let k1 = &k3p1c1[0..1];
    let t2 = &k3p1c1[1..3];
    let p1 = &k3p1c1[3..4];
    let c1 = k3p1c1.as_bytes()[4] as char;
    if SubjectProperty::from_str(k1).is_none() {
        return Err("sfid_number k1 subject_property invalid");
    }
    if InstitutionCode::from_str(t2).is_none() {
        return Err("sfid_number t2 institution invalid");
    }
    if !matches!(p1, "0" | "1") {
        return Err("sfid_number p1 profit_property invalid");
    }
    if segments[2].len() != SFID_NUMBER_SEGMENT_N9_LEN
        || !segments[2].chars().all(|c| c.is_ascii_digit())
    {
        return Err("sfid_number n9 segment invalid");
    }
    if segments[3].len() != SFID_NUMBER_SEGMENT_D4_LEN
        || !segments[3].chars().all(|c| c.is_ascii_digit())
    {
        return Err("sfid_number date segment invalid");
    }
    let payload = format!(
        "{}{}{}{}{}{}",
        segments[0], k1, t2, p1, segments[2], segments[3]
    );
    if sfid_checksum(&payload) != c1 {
        return Err("sfid_number checksum invalid");
    }
    Ok(SfidNumberParts {
        r5: segments[0].to_string(),
        k1: k1.to_string(),
        t2: t2.to_string(),
        k3: format!("{k1}{t2}"),
        p1: p1.to_string(),
        c1,
        n9: segments[2].to_string(),
        d4: segments[3].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_format() {
        let r = validate_sfid_number_format("LN001-GCB05-944805165-2026");
        assert_eq!(r, Ok("LN001-GCB05-944805165-2026".to_string()));
    }

    #[test]
    fn rejects_bad_segment_count() {
        assert!(validate_sfid_number_format("LN001-GCB05-944805165").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(validate_sfid_number_format("").is_err());
        assert!(validate_sfid_number_format("   ").is_err());
    }

    #[test]
    fn rejects_lowercase_subject_property() {
        assert!(validate_sfid_number_format("LN001-gCB05-944805165-2026").is_err());
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            validate_sfid_number_format("  LN001-GCB05-944805165-2026  "),
            Ok("LN001-GCB05-944805165-2026".to_string())
        );
    }

    #[test]
    fn rejects_extra_segment_format() {
        assert!(validate_sfid_number_format("LN001-GCB05-944805165-2026-EXTRA").is_err());
    }

    #[test]
    fn parses_protocol_parts() {
        let parts = parse_sfid_number_parts("LN001-GCB05-944805165-2026").unwrap();
        assert_eq!(parts.r5, "LN001");
        assert_eq!(parts.k1, "G");
        assert_eq!(parts.t2, "CB");
        assert_eq!(parts.k3, "GCB");
        assert_eq!(parts.p1, "0");
        assert_eq!(parts.c1, '5');
    }
}
