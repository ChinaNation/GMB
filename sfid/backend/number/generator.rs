//! SFID 号生成器
//!
//! 中文注释:
//! 这是 sfid 系统**唯一**的 SFID 号生成入口,供所有业务模块调用:
//! - `cpms` 公安局 CPMS 站点 SFID 生成
//! - `subjects`                   法人/非法人主体 SFID 生成
//! - `citizens::binding`          公民绑定兜底 SFID 生成
//! - `core::runtime_ops`      seed 阶段 SFID 生成
//!
//! 生成的 SFID 号结构见 `number/validator.rs` 顶部注释。

use blake2::{digest::consts::U32, Blake2b, Digest};
use chrono::Utc;

use crate::china::{city_code_by_name, province_code_by_name};
use crate::number::category::SubjectProperty;
use crate::number::institution_code::InstitutionCode;
use crate::number::validator::sfid_checksum;

type Blake2b256 = Blake2b<U32>;

const RESERVED_PROVINCE_CITY_CODE: &str = "000";

pub struct GenerateSfidInput<'a> {
    pub account_pubkey: &'a str,
    pub subject_property: &'a str,
    pub p1: &'a str,
    pub province_name: &'a str,
    pub city_name: &'a str,
    pub institution: &'a str,
}

fn hash_text(input: &str) -> u32 {
    let digest = Blake2b256::digest(input.as_bytes());
    let mut out = [0_u8; 4];
    out.copy_from_slice(&digest[..4]);
    u32::from_le_bytes(out)
}

fn resolve_p1(p1: &str) -> Result<&'static str, &'static str> {
    let v = p1.trim();
    match v {
        "0" | "非盈利" => Ok("0"),
        "1" | "盈利" => Ok("1"),
        _ => Err("p1 must be 0/1"),
    }
}

pub fn generate_sfid_number(input: GenerateSfidInput<'_>) -> Result<String, &'static str> {
    if input.account_pubkey.trim().is_empty()
        || input.subject_property.trim().is_empty()
        || input.province_name.trim().is_empty()
        || input.city_name.trim().is_empty()
        || input.institution.trim().is_empty()
    {
        return Err(
            "account_pubkey, subject_property, province_name, city_name, institution are required",
        );
    }

    let subject_property = SubjectProperty::from_str(input.subject_property)
        .ok_or("subject_property must be one of M/Z/N/G/S/F")?;
    let t2 = InstitutionCode::from_str(input.institution)
        .ok_or("institution must be a registered SFID institution code")?;
    let p1 = match subject_property {
        SubjectProperty::M | SubjectProperty::Z => "1",
        SubjectProperty::G => "0",
        SubjectProperty::N | SubjectProperty::S | SubjectProperty::F => resolve_p1(input.p1)?,
    };
    if subject_property == SubjectProperty::G
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
        return Err("public legal subject requires institution in ZF/LF/SF/JC/JY/CB");
    }
    if matches!(subject_property, SubjectProperty::M | SubjectProperty::N)
        && t2 != InstitutionCode::ZG
    {
        return Err("citizen/smart person subject requires institution ZG");
    }
    if subject_property == SubjectProperty::Z && t2 != InstitutionCode::TG {
        return Err("natural person subject requires institution TG");
    }
    if subject_property == SubjectProperty::S
        && !matches!(
            t2,
            InstitutionCode::LP
                | InstitutionCode::GQ
                | InstitutionCode::GF
                | InstitutionCode::GY
                | InstitutionCode::AS
                | InstitutionCode::JY
        )
    {
        return Err("private legal subject requires institution in LP/GQ/GF/GY/AS or education JY");
    }
    if subject_property == SubjectProperty::F
        && !matches!(
            t2,
            InstitutionCode::GT | InstitutionCode::GP | InstitutionCode::JY | InstitutionCode::ZG
        )
    {
        return Err("unincorporated subject requires institution in GT/GP, education JY, or public subordinate ZG");
    }
    // 中文注释:D4 段只取年份,生成结果固定符合 R5-K3P1C1-N9-D4。
    // 同 (主体属性, 省, 市, 机构, year) 5 元组共享 10 亿 n9 桶,
    // 单省 1.5 亿人口仅占 15% 桶填充,搭配调用方 1000 次重试基本不会撞光。
    let d = Utc::now().format("%Y").to_string();
    let province_code = province_code_by_name(input.province_name)
        .ok_or("province not found in code table")?
        .to_string();
    // 中文注释:公民/自然人/智能人的公开编码只精确到省,市级段统一固定为 000。
    let city_code = if matches!(
        subject_property,
        SubjectProperty::M | SubjectProperty::Z | SubjectProperty::N
    ) {
        RESERVED_PROVINCE_CITY_CODE.to_string()
    } else {
        city_code_by_name(input.province_name, input.city_name)
            .ok_or("city not found in province code table")?
            .to_string()
    };
    let normalized_city_for_hash = if matches!(
        subject_property,
        SubjectProperty::M | SubjectProperty::Z | SubjectProperty::N
    ) {
        RESERVED_PROVINCE_CITY_CODE
    } else {
        input.city_name
    };
    let k1 = subject_property.as_code();
    let t2 = t2.as_code();
    let r5 = format!("{province_code}{city_code}");
    let n9 = format!(
        "{:09}",
        (hash_text(&format!(
            "{}|{}|{}|{}|{}|{}",
            input.account_pubkey,
            k1,
            input.province_name,
            normalized_city_for_hash,
            input.institution,
            d
        )) as usize)
            % 1_000_000_000
    );
    let k3 = format!("{k1}{t2}");
    let payload = format!("{r5}{k3}{p1}{n9}{d}");
    let c1 = sfid_checksum(&payload);
    Ok(format!("{r5}-{k3}{p1}{c1}-{n9}-{d}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citizen_uses_reserved_province_city_code() {
        let code = generate_sfid_number(GenerateSfidInput {
            account_pubkey: "0x1234",
            subject_property: "M",
            p1: "1",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "ZG",
        })
        .expect("citizen sfid should generate");

        assert_eq!(code.split('-').next(), Some("GD000"));
    }

    #[test]
    fn public_legal_keeps_real_city_code() {
        let code = generate_sfid_number(GenerateSfidInput {
            account_pubkey: "0x5678",
            subject_property: "G",
            p1: "0",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "ZF",
        })
        .expect("public legal sfid should generate");

        assert_eq!(code.split('-').next(), Some("GD001"));
    }

    #[test]
    fn example_checksum_matches_current_payload() {
        assert_eq!(sfid_checksum("LN001GCB09448051652026"), '5');
    }
}
