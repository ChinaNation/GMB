//! CID 号核心生成协议。
//! 这里只处理字节协议;SQLite、时间、UUID 和查重由 registry 负责。

use alloc::{format, string::String};

use sp_core::hashing::blake2_256;

use crate::cid::{
    code::{self, ProfitPolicy},
    number::{checksum_char_m1, checksum_char_mod36},
};

/// 个人主体公开编码只精确到省。
pub const RESERVED_PROVINCE_CITY_CODE: &str = "000";

pub struct GenerateCidNumberInput<'a> {
    pub account_pubkey: &'a str,
    /// 可变/继承盈利策略读取的 0/1 输入。
    pub p1: &'a str,
    /// 两位省级行政区代码。
    pub province_code: &'a str,
    /// 省级行政区名称,N9 hash 使用。
    pub province_name: &'a str,
    /// 三位市级行政区代码。
    pub city_code: &'a str,
    /// 市级行政区名称,N9 hash 使用。
    pub city_name: &'a str,
    /// 生成年份 YYYY。
    pub year: &'a str,
    /// 机构码或机构简称。
    pub institution: &'a str,
}

fn hash_text(input: &str) -> u32 {
    let digest = blake2_256(input.as_bytes());
    let mut out = [0_u8; 4];
    out.copy_from_slice(&digest[..4]);
    u32::from_le_bytes(out)
}

fn resolve_profit(p1: &str) -> Result<bool, &'static str> {
    match p1.trim() {
        "0" | "非盈利" => Ok(false),
        "1" | "盈利" => Ok(true),
        _ => Err("p1 must be 0/1 for variable/inherit institution code"),
    }
}

fn valid_ascii_code(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}

pub fn generate_cid_number(input: GenerateCidNumberInput<'_>) -> Result<String, &'static str> {
    if input.account_pubkey.trim().is_empty()
        || input.province_code.trim().is_empty()
        || input.province_name.trim().is_empty()
        || input.city_name.trim().is_empty()
        || input.year.trim().is_empty()
        || input.institution.trim().is_empty()
    {
        return Err(
            "account_pubkey, province_code, province_name, city_name, year, institution are required",
        );
    }
    if !valid_ascii_code(input.province_code, 2) {
        return Err("province_code must be 2 uppercase ascii chars");
    }
    if input.year.len() != 4 || !input.year.bytes().all(|b| b.is_ascii_digit()) {
        return Err("year must be YYYY");
    }

    let institution_code = code::institution_code_from_str(input.institution)
        .ok_or("institution must be a registered CID institution code")?;
    if institution_code == code::PMUL {
        return Err("personal multisig (PMUL) has no cid number");
    }

    // 盈利属性由机构码策略决定。
    let profit = match code::profit_policy(&institution_code) {
        Some(ProfitPolicy::NonProfit) => false,
        Some(ProfitPolicy::Profit) => true,
        Some(ProfitPolicy::Variable | ProfitPolicy::InheritParent) => resolve_profit(input.p1)?,
        None => return Err("institution profit policy missing"),
    };

    let person_level = code::is_person_code(&institution_code);
    let city_code = if person_level {
        RESERVED_PROVINCE_CITY_CODE
    } else {
        if !valid_ascii_code(input.city_code, 3) {
            return Err("city_code must be 3 uppercase ascii chars");
        }
        input.city_code
    };
    let normalized_city_for_hash = if person_level {
        RESERVED_PROVINCE_CITY_CODE
    } else {
        input.city_name
    };

    let code_str =
        code::institution_code_text(&institution_code).ok_or("institution code text missing")?;
    let r5 = format!("{}{}", input.province_code, city_code);
    // 同一分类四元组共享 10 亿 n9 桶;碰撞由 registry 处理。
    let n9 = format!(
        "{:09}",
        (hash_text(&format!(
            "{}|{}|{}|{}|{}",
            input.account_pubkey,
            code_str,
            input.province_name,
            normalized_city_for_hash,
            input.year
        )) as usize)
            % 1_000_000_000
    );

    if code::is_three_char_code(&institution_code) {
        let profit_char = if profit { "1" } else { "0" };
        let payload = format!("{r5}{code_str}{profit_char}{n9}{}", input.year);
        let c = checksum_char_mod36(&payload);
        Ok(format!(
            "{r5}-{code_str}{profit_char}{c}-{n9}-{}",
            input.year
        ))
    } else {
        let payload = format!("{r5}{code_str}{n9}{}", input.year);
        let m1 = checksum_char_m1(&payload, profit);
        Ok(format!("{r5}-{code_str}{m1}-{n9}-{}", input.year))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen(institution: &str, p1: &str) -> String {
        generate_cid_number(GenerateCidNumberInput {
            account_pubkey: "0xabcd",
            p1,
            province_code: "GD",
            province_name: "广东省",
            city_code: "001",
            city_name: "荔湾市",
            year: "2026",
            institution,
        })
        .expect("cid should generate")
    }

    #[test]
    fn citizen_uses_reserved_province_city_code() {
        let code = gen("CTZN", "1");
        assert_eq!(code.split('-').next(), Some("GD000"));
    }

    #[test]
    fn public_legal_keeps_real_city_code() {
        let code = gen("CGOV", "0");
        assert_eq!(code.split('-').next(), Some("GD001"));
    }

    #[test]
    fn three_char_national_layout_shape() {
        let code = gen("NRC", "0");
        let seg2 = code.split('-').nth(1).unwrap();
        assert_eq!(seg2.len(), 5);
        assert_eq!(&seg2[0..3], "NRC");
        assert_eq!(&seg2[3..4], "0");
    }

    #[test]
    fn pmul_has_no_number() {
        let r = generate_cid_number(GenerateCidNumberInput {
            account_pubkey: "0x1",
            p1: "0",
            province_code: "GD",
            province_name: "广东省",
            city_code: "001",
            city_name: "荔湾市",
            year: "2026",
            institution: "PMUL",
        });
        assert!(r.is_err());
    }
}
