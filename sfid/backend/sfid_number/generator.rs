//! SFID 号生成器
//!
//! 中文注释:
//! 这是 sfid 系统**唯一**的 SFID 号生成入口,供所有业务模块调用:
//! - `cpms` 公安局 CPMS 站点 SFID 生成
//! - `subjects`                   法人/非法人主体 SFID 生成
//! - `citizens::binding`          公民绑定兜底 SFID 生成
//! - `app_core::runtime_ops`      seed 阶段 SFID 生成
//!
//! 生成的 SFID 号结构见 `sfid_number/validator.rs` 顶部注释。

use blake2::{digest::consts::U32, Blake2b, Digest};
use chrono::Utc;

use crate::china::{city_code_by_name, province_code_by_name};
use crate::sfid_number::a3::A3;
use crate::sfid_number::institution_code::InstitutionCode;

type Blake2b256 = Blake2b<U32>;

const ALPHABET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const RESERVED_PROVINCE_CITY_CODE: &str = "000";

pub struct GenerateSfidInput<'a> {
    pub account_pubkey: &'a str,
    pub a3: &'a str,
    pub p1: &'a str,
    pub province: &'a str,
    pub city: &'a str,
    pub institution: &'a str,
}

fn hash_text(input: &str) -> u32 {
    let digest = Blake2b256::digest(input.as_bytes());
    let mut out = [0_u8; 4];
    out.copy_from_slice(&digest[..4]);
    u32::from_le_bytes(out)
}

fn checksum(payload: &str) -> char {
    let mut total: usize = 0;
    for (idx, ch) in payload.chars().enumerate() {
        let pos = ALPHABET.find(ch).unwrap_or(0);
        total = (total + (idx + 1) * pos) % 36;
    }
    ALPHABET.as_bytes()[total] as char
}

fn resolve_p1(p1: &str) -> Result<&'static str, &'static str> {
    let v = p1.trim();
    match v {
        "0" | "非盈利" => Ok("0"),
        "1" | "盈利" => Ok("1"),
        _ => Err("p1 must be 0/1"),
    }
}

pub fn generate_sfid_code(input: GenerateSfidInput<'_>) -> Result<String, &'static str> {
    if input.account_pubkey.trim().is_empty()
        || input.a3.trim().is_empty()
        || input.province.trim().is_empty()
        || input.city.trim().is_empty()
        || input.institution.trim().is_empty()
    {
        return Err("account_pubkey, a3, province, city, institution are required");
    }

    let a3 = A3::from_str(input.a3).ok_or("a3 must be one of GMR/ZRR/ZNR/GFR/SFR/FFR")?;
    let t2 = InstitutionCode::from_str(input.institution)
        .ok_or("institution must be one of ZG/ZF/LF/SF/JC/JY/CB/CH/TG")?;
    let p1 = match a3 {
        A3::GMR | A3::ZRR => "1",
        A3::GFR => "0",
        A3::ZNR | A3::SFR | A3::FFR => resolve_p1(input.p1)?,
    };
    if a3 == A3::GFR
        && !matches!(
            t2,
            InstitutionCode::ZF
                | InstitutionCode::LF
                | InstitutionCode::SF
                | InstitutionCode::JC
                | InstitutionCode::JY
                | InstitutionCode::CB
        )
    {
        return Err("GFR requires institution in ZF/LF/SF/JC/JY/CB");
    }
    if matches!(a3, A3::GMR | A3::ZNR) && t2 != InstitutionCode::ZG {
        return Err("GMR/ZNR requires institution ZG");
    }
    if a3 == A3::ZRR && t2 != InstitutionCode::TG {
        return Err("ZRR requires institution TG");
    }
    if a3 == A3::SFR
        && !matches!(
            t2,
            InstitutionCode::ZG | InstitutionCode::JY | InstitutionCode::CH | InstitutionCode::TG
        )
    {
        return Err("SFR requires institution in ZG/JY/CH/TG");
    }
    if a3 == A3::FFR
        && !matches!(
            t2,
            InstitutionCode::ZG | InstitutionCode::JY | InstitutionCode::TG
        )
    {
        return Err("FFR requires institution in ZG/JY/TG");
    }
    // 中文注释:D4 段只取年份(2026-05-07 改造,从 D8 缩为 D4)。
    // 同 (a3, 省, 市, 机构, year) 5 元组共享 10 亿 n9 桶,
    // 单省 1.5 亿人口仅占 15% 桶填充,搭配调用方 1000 次重试基本不会撞光。
    let d = Utc::now().format("%Y").to_string();
    let province_code = province_code_by_name(input.province)
        .ok_or("province not found in code table")?
        .to_string();
    // 中文注释:公民人/自然人/智能人的公开编码只精确到省,市级段统一固定为 000。
    let city_code = if matches!(a3, A3::GMR | A3::ZRR | A3::ZNR) {
        RESERVED_PROVINCE_CITY_CODE.to_string()
    } else {
        city_code_by_name(input.province, input.city)
            .ok_or("city not found in province code table")?
            .to_string()
    };
    let normalized_city_for_hash = if matches!(a3, A3::GMR | A3::ZRR | A3::ZNR) {
        RESERVED_PROVINCE_CITY_CODE
    } else {
        input.city
    };
    let a3 = a3.as_code();
    let t2 = t2.as_code();
    let r5 = format!("{province_code}{city_code}");
    let n9 = format!(
        "{:09}",
        (hash_text(&format!(
            "{}|{}|{}|{}|{}|{}",
            input.account_pubkey,
            a3,
            input.province,
            normalized_city_for_hash,
            input.institution,
            d
        )) as usize)
            % 1_000_000_000
    );
    let payload = format!("{a3}{r5}{t2}{p1}{n9}{d}");
    let c1 = checksum(&payload);
    Ok(format!("{a3}-{r5}-{t2}{p1}{c1}-{n9}-{d}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmr_uses_reserved_province_city_code() {
        let code = generate_sfid_code(GenerateSfidInput {
            account_pubkey: "0x1234",
            a3: "GMR",
            p1: "1",
            province: "广东省",
            city: "广州市",
            institution: "ZG",
        })
        .expect("gmr sfid should generate");

        assert_eq!(code.split('-').nth(1), Some("GD000"));
    }

    #[test]
    fn gfr_keeps_real_city_code() {
        let code = generate_sfid_code(GenerateSfidInput {
            account_pubkey: "0x5678",
            a3: "GFR",
            p1: "0",
            province: "广东省",
            city: "广州市",
            institution: "ZF",
        })
        .expect("gfr sfid should generate");

        assert_eq!(code.split('-').nth(1), Some("GD001"));
    }
}
