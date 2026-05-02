// 中文注释:任务卡 1 基础设施,部分 API 在任务卡 2 才会有调用点。
#![allow(dead_code)]

//! SFID 号格式校验
//!
//! 中文注释:
//! sfid 号结构(5 段,连字符分隔,ASCII 大写字母 + 数字):
//! ```
//! A3(3) - R5(5) - T2P1C1(4) - N9(9) - D8(8)
//! 例:GFR  - AH001 - ZF0X     - 898100720 - 20260407
//! ```
//!
//! - A3:3 字符,主体属性代码(GMR/ZRR/ZNR/GFR/SFR/FFR)
//! - R5:5 字符,省代码(2) + 市代码(3)
//! - T2P1C1:4 字符,机构代码(2) + 盈利属性(1) + 校验码(1)
//! - N9:9 字符,全数字 hash 号
//! - D8:8 字符,生成日期 YYYYMMDD
//!
//! 链端 `register_sfid_institution` 对 sfid_id 的 BoundedVec 上限也是 96 字节,
//! 与本文件 SFID_ID_MAX_BYTES 保持一致。

pub const SFID_ID_SEGMENT_COUNT: usize = 5;
pub const SFID_ID_SEGMENT_A3_LEN: usize = 3;
pub const SFID_ID_SEGMENT_R5_LEN: usize = 5;
pub const SFID_ID_SEGMENT_T2P1C1_LEN: usize = 4;
pub const SFID_ID_SEGMENT_N9_LEN: usize = 9;
pub const SFID_ID_SEGMENT_D8_LEN: usize = 8;
pub const SFID_ID_MAX_BYTES: usize = 96;

/// 校验 sfid 号字符串格式,通过返回标准化后的字符串(trim + 保持大小写)。
/// 失败返回静态错误字符串,便于调用方直接透传给 HTTP 响应。
pub fn validate_sfid_id_format(raw: &str) -> Result<String, &'static str> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err("site_sfid is required");
    }
    if !normalized.is_ascii() {
        return Err("site_sfid must be ascii");
    }
    if normalized.len() > SFID_ID_MAX_BYTES {
        return Err("site_sfid length exceeds chain max");
    }
    if normalized
        .bytes()
        .any(|b| !(b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-'))
    {
        return Err("site_sfid charset invalid");
    }
    let segments = normalized.split('-').collect::<Vec<_>>();
    if segments.len() != SFID_ID_SEGMENT_COUNT {
        return Err("site_sfid format invalid");
    }
    if segments[0].len() != SFID_ID_SEGMENT_A3_LEN
        || !segments[0].chars().all(|c| c.is_ascii_uppercase())
    {
        return Err("site_sfid a3 segment invalid");
    }
    if segments[1].len() != SFID_ID_SEGMENT_R5_LEN
        || !segments[1]
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err("site_sfid r5 segment invalid");
    }
    if segments[2].len() != SFID_ID_SEGMENT_T2P1C1_LEN
        || !segments[2]
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err("site_sfid t2p1c1 segment invalid");
    }
    if segments[3].len() != SFID_ID_SEGMENT_N9_LEN
        || !segments[3].chars().all(|c| c.is_ascii_digit())
    {
        return Err("site_sfid n9 segment invalid");
    }
    if segments[4].len() != SFID_ID_SEGMENT_D8_LEN
        || !segments[4].chars().all(|c| c.is_ascii_digit())
    {
        return Err("site_sfid date segment invalid");
    }
    Ok(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_format() {
        let r = validate_sfid_id_format("GFR-AH001-ZF0X-898100720-20260407");
        assert_eq!(r, Ok("GFR-AH001-ZF0X-898100720-20260407".to_string()));
    }

    #[test]
    fn rejects_bad_segment_count() {
        assert!(validate_sfid_id_format("GFR-AH001-ZF0X-898100720").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(validate_sfid_id_format("").is_err());
        assert!(validate_sfid_id_format("   ").is_err());
    }

    #[test]
    fn rejects_lowercase_a3() {
        assert!(validate_sfid_id_format("gfr-AH001-ZF0X-898100720-20260407").is_err());
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            validate_sfid_id_format("  GFR-AH001-ZF0X-898100720-20260407  "),
            Ok("GFR-AH001-ZF0X-898100720-20260407".to_string())
        );
    }
}
