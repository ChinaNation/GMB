//! SFID 号生成器
//!
//! 中文注释:
//! 这是 sfid 系统**唯一**的 SFID 号生成入口,供所有业务模块调用:
//! - `sheng-admins::institutions` 公安局/公权机构 SFID 生成
//! - `sheng-admins::multisig`     多签机构 SFID 生成
//! - `operate::binding`           公民绑定兜底 SFID 生成
//! - `app_core::runtime_ops`      seed 阶段 SFID 生成
//!
//! 生成的 SFID 号结构见 `sfid/validator.rs` 顶部注释。

use blake2::{digest::consts::U32, Blake2b, Digest};
use chrono::Utc;

use crate::sfid::a3::resolve_a3;
use crate::sfid::institution_code::resolve_org_type;
use crate::sfid::province::{city_code_by_name, province_code_by_name};

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

    let a3 = resolve_a3(input.a3)?;
    let t2 = resolve_org_type(input.institution)?;
    let p1 = match a3 {
        "GMR" | "ZRR" => "1",
        "GFR" => "0",
        "ZNR" | "SFR" | "FFR" => resolve_p1(input.p1)?,
        _ => return Err("a3 not supported"),
    };
    if a3 == "GFR" && !matches!(t2, "ZF" | "LF" | "SF" | "JC" | "JY" | "CB") {
        return Err("GFR requires institution in ZF/LF/SF/JC/JY/CB");
    }
    if matches!(a3, "GMR" | "ZNR") && t2 != "ZG" {
        return Err("GMR/ZNR requires institution ZG");
    }
    if a3 == "ZRR" && t2 != "TG" {
        return Err("ZRR requires institution TG");
    }
    if a3 == "SFR" && !matches!(t2, "ZG" | "CH" | "TG") {
        return Err("SFR requires institution in ZG/CH/TG");
    }
    if a3 == "FFR" && !matches!(t2, "ZG" | "TG") {
        return Err("FFR requires institution in ZG/TG");
    }
    let d = Utc::now().format("%Y%m%d").to_string();
    let province_code = province_code_by_name(input.province)
        .ok_or("province not found in code table")?
        .to_string();
    // 中文注释:公民人/自然人/智能人的公开编码只精确到省,市级段统一固定为 000。
    let city_code = if matches!(a3, "GMR" | "ZRR" | "ZNR") {
        RESERVED_PROVINCE_CITY_CODE.to_string()
    } else {
        city_code_by_name(input.province, input.city)
            .ok_or("city not found in province code table")?
            .to_string()
    };
    let normalized_city_for_hash = if matches!(a3, "GMR" | "ZRR" | "ZNR") {
        RESERVED_PROVINCE_CITY_CODE
    } else {
        input.city
    };
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
