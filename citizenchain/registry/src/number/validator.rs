#![allow(dead_code)]

//! CID 号格式校验
//!
//! 中文注释:
//! cid 号结构(4 段,连字符分隔,ASCII 大写字母 + 数字):
//! ```
//! R5(5) - 段二(5) - N9(9) - D4(4)
//! ```
//! 段二(核心段)按机构码长度分两种布局,靠**段二 index 3** 字符是数字/字母分流:
//! - A 国家/省部(3 字符码): `码(3) + 盈利位(1,恒 0,数字) + 校验(1, mod-36)`
//! - B 其他(4 字符码):      `码(4) + M1(1)`,M1 数字=盈利(校验 mod-10)/字母=非盈利(校验 mod-26)
//!
//! - R5:5 字符,省代码(2) + 市代码(3)
//! - N9:9 字符,全数字 hash 号
//! - D4:4 字符,生成年份 YYYY
//!
//! 主体属性(K1)已从号码删除,机构类别一律由机构码自身语义派生。
//! cid_number 字节上限唯一权威源 = `primitives::core_const::CID_NUMBER_MAX_BYTES`。

use crate::number::code::{self, InstitutionCode, ProfitPolicy};
use primitives::core_const::CID_NUMBER_MAX_BYTES;

const CHECKSUM_ALPHABET: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub const CID_NUMBER_SEGMENT_COUNT: usize = 4;
pub const CID_NUMBER_SEGMENT_R5_LEN: usize = 5;
/// 段二(核心段)长度,两种布局都恒为 5 字符。
pub const CID_NUMBER_SEGMENT_K3P1C1_LEN: usize = 5;
pub const CID_NUMBER_SEGMENT_N9_LEN: usize = 9;
pub const CID_NUMBER_SEGMENT_D4_LEN: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CidNumberParts {
    pub r5: String,
    /// 机构码(全仓库机构分类唯一真源)。
    pub institution: InstitutionCode,
    /// 机构码原文(3 或 4 字符)。
    pub institution_code_text: String,
    /// 盈利属性(3 字符布局恒 false;4 字符布局由 M1 数字/字母解出)。
    pub profit: bool,
    /// 校验位(3 字符布局 = mod-36 字符;4 字符布局 = M1)。
    pub checksum: char,
    pub n9: String,
    pub d4: String,
}

/// 校验和累加器(不取模,留给上层按 10/26/36 取模)。
pub(crate) fn checksum_acc(payload: &str) -> usize {
    let mut total: usize = 0;
    for (idx, ch) in payload.chars().enumerate() {
        let pos = CHECKSUM_ALPHABET
            .iter()
            .position(|&b| b == ch as u8)
            .unwrap_or(0);
        total = total.wrapping_add((idx + 1) * pos);
    }
    total
}

/// 3 字符布局校验位:mod-36,返回 `0-9A-Z` 单字符。
pub(crate) fn checksum_char_mod36(payload: &str) -> char {
    CHECKSUM_ALPHABET[checksum_acc(payload) % 36] as char
}

/// 4 字符布局 M1:盈利→数字(mod-10),非盈利→字母(mod-26)。
pub(crate) fn checksum_char_m1(payload: &str, profit: bool) -> char {
    if profit {
        (b'0' + (checksum_acc(payload) % 10) as u8) as char
    } else {
        (b'A' + (checksum_acc(payload) % 26) as u8) as char
    }
}

/// 校验 cid 号字符串格式,通过返回标准化后的字符串(trim + 保持大小写)。
pub fn validate_cid_number_format(raw: &str) -> Result<String, &'static str> {
    parse_cid_number_parts(raw).map(|_| raw.trim().to_string())
}

/// 解析并校验 cid_number,通过后返回拆分后的协议字段。
pub fn parse_cid_number_parts(raw: &str) -> Result<CidNumberParts, &'static str> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err("cid_number is required");
    }
    if !normalized.is_ascii() {
        return Err("cid_number must be ascii");
    }
    if normalized.len() > CID_NUMBER_MAX_BYTES as usize {
        return Err("cid_number length exceeds chain max");
    }
    if normalized
        .bytes()
        .any(|b| !(b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-'))
    {
        return Err("cid_number charset invalid");
    }
    let segments = normalized.split('-').collect::<Vec<_>>();
    if segments.len() != CID_NUMBER_SEGMENT_COUNT {
        return Err("cid_number format invalid");
    }
    if segments[0].len() != CID_NUMBER_SEGMENT_R5_LEN
        || !segments[0]
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err("cid_number r5 segment invalid");
    }
    if segments[1].len() != CID_NUMBER_SEGMENT_K3P1C1_LEN
        || !segments[1]
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err("cid_number core segment invalid");
    }
    if segments[2].len() != CID_NUMBER_SEGMENT_N9_LEN
        || !segments[2].chars().all(|c| c.is_ascii_digit())
    {
        return Err("cid_number n9 segment invalid");
    }
    if segments[3].len() != CID_NUMBER_SEGMENT_D4_LEN
        || !segments[3].chars().all(|c| c.is_ascii_digit())
    {
        return Err("cid_number date segment invalid");
    }

    let seg2 = segments[1];
    let seg2_bytes = seg2.as_bytes();
    let index3 = seg2_bytes[3] as char;

    let (institution, institution_code_text, profit, checksum) = if index3.is_ascii_digit() {
        // A 布局:3 字符码 + 盈利位 + 校验。
        let code = &seg2[0..3];
        let profit_char = &seg2[3..4];
        let checksum = seg2_bytes[4] as char;
        let institution = code::from_str(code).ok_or("cid_number institution code invalid")?;
        if !code::is_three_char(&institution) {
            return Err("cid_number 3-char layout code mismatch");
        }
        // 盈利位 0/1,须与机构码盈利策略一致(国家/省部/公立大学非盈利=0;私立大学可变)。
        let profit = match profit_char {
            "0" => false,
            "1" => true,
            _ => return Err("cid_number 3-char layout profit must be 0/1"),
        };
        match code::profit_policy(&institution) {
            ProfitPolicy::NonProfit if profit => {
                return Err("cid_number profit conflicts with code policy")
            }
            ProfitPolicy::Profit if !profit => {
                return Err("cid_number profit conflicts with code policy")
            }
            _ => {}
        }
        let payload = format!(
            "{}{}{}{}{}",
            segments[0], code, profit_char, segments[2], segments[3]
        );
        if checksum_char_mod36(&payload) != checksum {
            return Err("cid_number checksum invalid");
        }
        (institution, code.to_string(), profit, checksum)
    } else {
        // B 布局:4 字符码 + M1。
        let code = &seg2[0..4];
        let m1 = seg2_bytes[4] as char;
        let institution = code::from_str(code).ok_or("cid_number institution code invalid")?;
        if code::is_three_char(&institution) {
            return Err("cid_number 4-char layout code mismatch");
        }
        let profit = m1.is_ascii_digit();
        // M1 解出的盈利属性必须与机构码盈利策略一致。
        match code::profit_policy(&institution) {
            ProfitPolicy::NonProfit if profit => {
                return Err("cid_number profit conflicts with code policy")
            }
            ProfitPolicy::Profit if !profit => {
                return Err("cid_number profit conflicts with code policy")
            }
            _ => {}
        }
        let payload = format!("{}{}{}{}", segments[0], code, segments[2], segments[3]);
        if checksum_char_m1(&payload, profit) != m1 {
            return Err("cid_number checksum invalid");
        }
        (institution, code.to_string(), profit, m1)
    };

    Ok(CidNumberParts {
        r5: segments[0].to_string(),
        institution,
        institution_code_text,
        profit,
        checksum,
        n9: segments[2].to_string(),
        d4: segments[3].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::generator::{generate_cid_number, GenerateCidInput};

    fn gen(institution: &str, p1: &str, province: &str, city: &str) -> String {
        generate_cid_number(GenerateCidInput {
            account_pubkey: "0xabcd",
            p1,
            province_name: province,
            city_name: city,
            institution,
        })
        .expect("cid should generate")
    }

    #[test]
    fn roundtrip_three_char_layout() {
        // 国储会:3 字符码 NRC,非盈利,盈利位 0。
        let code = gen("NRC", "0", "广东省", "荔湾市");
        let parts = parse_cid_number_parts(&code).expect("must parse");
        assert_eq!(parts.institution, code::NRC);
        assert_eq!(parts.institution_code_text, "NRC");
        assert!(!parts.profit);
        assert!(validate_cid_number_format(&code).is_ok());
    }

    #[test]
    fn roundtrip_four_char_profit() {
        // 股权公司:4 字符码 SFGQ,固定盈利 → M1 数字。
        let code = gen("SFGQ", "1", "广东省", "荔湾市");
        let parts = parse_cid_number_parts(&code).expect("must parse");
        assert_eq!(parts.institution, *b"SFGQ");
        assert!(parts.profit);
        assert!(parts.checksum.is_ascii_digit());
    }

    #[test]
    fn roundtrip_four_char_nonprofit() {
        // 市政府:4 字符码 CGOV,非盈利 → M1 字母。
        let code = gen("CGOV", "0", "广东省", "荔湾市");
        let parts = parse_cid_number_parts(&code).expect("must parse");
        assert_eq!(parts.institution, *b"CGOV");
        assert!(!parts.profit);
        assert!(parts.checksum.is_ascii_uppercase());
    }

    #[test]
    fn all_codes_roundtrip_generate_parse() {
        // 遍历全部 92 码(除不发号的 PMUL):生成→解析必须还原同一机构码且格式校验通过。
        // 确定性覆盖三/四字符两种布局、盈利数字/字母 M1、3 字符盈利位与各档校验。
        for institution_code in code::ALL {
            if institution_code == code::PMUL {
                continue;
            }
            let number = generate_cid_number(GenerateCidInput {
                account_pubkey: "0xfeed",
                // 仅 Variable/InheritParent 策略读取;固定策略忽略。取 1 让可变码盈利一致。
                p1: "1",
                province_name: "广东省",
                city_name: "荔湾市",
                institution: code::as_code(&institution_code),
            })
            .unwrap_or_else(|e| {
                panic!("{} should generate: {e}", code::as_code(&institution_code))
            });
            let parts = parse_cid_number_parts(&number).unwrap_or_else(|e| {
                panic!("{} should parse: {e}", code::as_code(&institution_code))
            });
            assert_eq!(
                parts.institution,
                institution_code,
                "roundtrip code mismatch for {}",
                code::as_code(&institution_code)
            );
            assert!(validate_cid_number_format(&number).is_ok());
        }
    }

    #[test]
    fn rejects_bad_segment_count() {
        assert!(validate_cid_number_format("GD001-CGOVX-944805165").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(validate_cid_number_format("").is_err());
        assert!(validate_cid_number_format("   ").is_err());
    }

    #[test]
    fn rejects_lowercase() {
        let code = gen("CGOV", "0", "广东省", "荔湾市");
        let lowered = code.to_lowercase();
        assert!(validate_cid_number_format(&lowered).is_err());
    }

    #[test]
    fn rejects_legacy_format() {
        // 旧版号(含 K1 主体属性段)必须校验失败。
        assert!(validate_cid_number_format("LN001-NRC05-944805165-2026").is_err());
        assert!(validate_cid_number_format("GFR-AH001-ZF0X-898100720-2026").is_err());
    }

    #[test]
    fn rejects_tampered_checksum() {
        let code = gen("NRC", "0", "广东省", "荔湾市");
        // 改最后一个段二字符(校验位)使其失配。
        let chars: Vec<char> = code.chars().collect();
        // 段二在第一个 '-' 之后;找到第二段最后一位。
        let parts: Vec<&str> = code.split('-').collect();
        let seg2 = parts[1];
        let bad_checksum = if seg2.ends_with('0') { '1' } else { '0' };
        let tampered = format!(
            "{}-{}{}-{}-{}",
            parts[0],
            &seg2[..4],
            bad_checksum,
            parts[2],
            parts[3]
        );
        let _ = chars; // silence
        assert!(validate_cid_number_format(&tampered).is_err());
    }
}
