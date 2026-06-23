//! CID 号生成器
//!
//! 中文注释:
//! 这是 cid 系统**唯一**的 CID 号生成入口(`generate_cid_number`)。业务模块的
//! 「种子约定 + 撞号重试」统一经 `number::seed` 调本入口:
//! - `subjects::registration`     经 `seed::dynamic_institution_cid`(随机 UUID + 重试)
//! - `citizens::binding`          经 `seed::citizen_cid`(wallet_pubkey + 重试)
//! - `gov::service`               经 `seed::official_institution_cid`(GOV 模板,确定性无重试)
//!   (`gov::service::generate_public_security_cid` 的 CPOL `PS-` 种子另属一类,仍直调本入口)
//!
//! 生成的 CID 号结构见 `number/validator.rs` 顶部注释。
//! 主体属性(K1)已从号码删除,由机构码自带语义;盈利属性由机构码 `profit_policy()` 决定,
//! 仅 Variable/InheritParent 策略的码读取 `p1` 入参(非法人组织由调用方传入父级盈利属性)。

use blake2::{digest::consts::U32, Blake2b, Digest};
use chrono::Utc;

use crate::china::{city_code_by_name, province_code_by_name};
use crate::number::code::{InstitutionCode, ProfitPolicy};
use crate::number::validator::{checksum_char_m1, checksum_char_mod36};

type Blake2b256 = Blake2b<U32>;

const RESERVED_PROVINCE_CITY_CODE: &str = "000";

pub struct GenerateCidInput<'a> {
    pub account_pubkey: &'a str,
    /// 盈利输入,仅 Variable(注册协会/智能人)与 InheritParent(非法人组织,传父级)
    /// 策略的机构码读取;固定盈利策略的码忽略本字段。取值 0/1 或 非盈利/盈利。
    pub p1: &'a str,
    pub province_name: &'a str,
    pub city_name: &'a str,
    /// 机构码(3 或 4 字符代码,或中文类型标签),全仓库机构分类唯一真源。
    pub institution: &'a str,
}

fn hash_text(input: &str) -> u32 {
    let digest = Blake2b256::digest(input.as_bytes());
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

pub fn generate_cid_number(input: GenerateCidInput<'_>) -> Result<String, &'static str> {
    if input.account_pubkey.trim().is_empty()
        || input.province_name.trim().is_empty()
        || input.city_name.trim().is_empty()
        || input.institution.trim().is_empty()
    {
        return Err("account_pubkey, province_name, city_name, institution are required");
    }

    let code = InstitutionCode::from_str(input.institution)
        .ok_or("institution must be a registered CID institution code")?;
    if code == InstitutionCode::Pmul {
        return Err("personal multisig (PMUL) has no cid number");
    }

    // 盈利属性由机构码策略决定;可变/继承策略读取入参。
    let profit = match code.profit_policy() {
        ProfitPolicy::NonProfit => false,
        ProfitPolicy::Profit => true,
        ProfitPolicy::Variable | ProfitPolicy::InheritParent => resolve_profit(input.p1)?,
    };

    // 公民人/自然人/智能人的公开编码只精确到省,市级段统一固定为 000。
    let person_level = code.is_person();

    let d = Utc::now().format("%Y").to_string();
    let province_code = province_code_by_name(input.province_name)
        .ok_or("province not found in code table")?
        .to_string();
    let city_code = if person_level {
        RESERVED_PROVINCE_CITY_CODE.to_string()
    } else {
        city_code_by_name(input.province_name, input.city_name)
            .ok_or("city not found in province code table")?
            .to_string()
    };
    let normalized_city_for_hash = if person_level {
        RESERVED_PROVINCE_CITY_CODE
    } else {
        input.city_name
    };

    let code_str = code.as_code();
    let r5 = format!("{province_code}{city_code}");
    // 同 (机构码, 省, 市, year) 4 元组共享 10 亿 n9 桶,调用方 1000 次重试逃逸碰撞。
    let n9 = format!(
        "{:09}",
        (hash_text(&format!(
            "{}|{}|{}|{}|{}",
            input.account_pubkey, code_str, input.province_name, normalized_city_for_hash, d
        )) as usize)
            % 1_000_000_000
    );

    if code.is_three_char() {
        // A 布局:码(3) + 盈利位(0/1,按盈利策略) + 校验(mod-36)。
        // 国家/省部/公立大学非盈利→0;私立大学(SUN)可变,按实例 0/1。
        let profit_char = if profit { "1" } else { "0" };
        let payload = format!("{r5}{code_str}{profit_char}{n9}{d}");
        let c = checksum_char_mod36(&payload);
        Ok(format!("{r5}-{code_str}{profit_char}{c}-{n9}-{d}"))
    } else {
        // B 布局:码(4) + M1(数字=盈利/字母=非盈利,值=校验)。
        let payload = format!("{r5}{code_str}{n9}{d}");
        let m1 = checksum_char_m1(&payload, profit);
        Ok(format!("{r5}-{code_str}{m1}-{n9}-{d}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citizen_uses_reserved_province_city_code() {
        let code = generate_cid_number(GenerateCidInput {
            account_pubkey: "0x1234",
            p1: "1",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "CTZN",
        })
        .expect("citizen cid should generate");
        assert_eq!(code.split('-').next(), Some("GD000"));
    }

    #[test]
    fn public_legal_keeps_real_city_code() {
        let code = generate_cid_number(GenerateCidInput {
            account_pubkey: "0x5678",
            p1: "0",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "CGOV",
        })
        .expect("public legal cid should generate");
        assert_eq!(code.split('-').next(), Some("GD001"));
    }

    #[test]
    fn three_char_national_layout_shape() {
        let code = generate_cid_number(GenerateCidInput {
            account_pubkey: "0x9999",
            p1: "0",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "NRC",
        })
        .expect("nrc cid should generate");
        let seg2 = code.split('-').nth(1).unwrap();
        assert_eq!(seg2.len(), 5);
        assert_eq!(&seg2[0..3], "NRC");
        assert_eq!(&seg2[3..4], "0"); // 盈利位恒 0,index3 数字 → A 布局
    }

    #[test]
    fn pmul_has_no_number() {
        let r = generate_cid_number(GenerateCidInput {
            account_pubkey: "0x1",
            p1: "0",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "PMUL",
        });
        assert!(r.is_err());
    }
}
