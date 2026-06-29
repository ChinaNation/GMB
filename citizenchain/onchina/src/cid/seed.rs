//! onchina CID 种子 + 撞号重试唯一入口。
//!
//! 中文注释:
//! 确定性 account seed 的字节格式由 `primitives::cid::seed` 保护;本文件只负责
//! 调用 onchina 生成适配层、执行数据库查重回调和动态机构 UUID 重试。

use crate::cid::{
    generator::{generate_cid_number, GenerateCidInput},
    validate_cid_number_format,
};

/// 撞号重试上限(动态机构共用)。
const COLLISION_RETRY_LIMIT: u32 = 1000;

/// CID 种子构造失败原因。`E` 为调用方 `exists_fn` 的查重错误类型。
#[derive(Debug)]
pub enum SeedCidError<E> {
    /// 底层号码生成失败(机构码非法、行政区缺失等)。
    Generate(&'static str),
    /// 生成的号码格式校验失败(理论不可达,留作纵深防御)。
    Validate(&'static str),
    /// 调用方 DB 查重回调返回的错误。
    Exists(E),
    /// 1000 次重试仍碰撞,桶饱和。
    Exhausted,
}

/// 公权机构(政府模板)CID — 确定性种子,无重试。
pub fn official_institution_cid<E>(
    scope: &str,
    province_code: &str,
    city_code: &str,
    town_code: &str,
    institution_code: &str,
    province_name: &str,
    city_name: &str,
    exists_fn: impl Fn(&str) -> Result<bool, E>,
) -> Result<String, SeedCidError<E>> {
    let account_seed = primitives::cid::seed::official_institution_account_seed(
        scope,
        province_code,
        city_code,
        town_code,
        institution_code,
    );
    let cid = generate_cid_number(GenerateCidInput {
        account_pubkey: account_seed.as_str(),
        p1: "0",
        province_name,
        city_name,
        institution: institution_code,
    })
    .map_err(SeedCidError::Generate)?;
    if exists_fn(&cid).map_err(SeedCidError::Exists)? {
        return Err(SeedCidError::Exhausted);
    }
    Ok(cid)
}

/// 机构动态注册 CID — 随机 UUIDv4 种子 + 1000 次重试 + 格式校验。
pub fn dynamic_institution_cid<E>(
    province_name: &str,
    city_name: &str,
    institution_code: &str,
    p1: &str,
    exists_fn: impl Fn(&str) -> Result<bool, E>,
) -> Result<String, SeedCidError<E>> {
    for _ in 0..COLLISION_RETRY_LIMIT {
        let random_account = uuid::Uuid::new_v4().to_string();
        let cid = generate_cid_number(GenerateCidInput {
            account_pubkey: random_account.as_str(),
            p1,
            province_name,
            city_name,
            institution: institution_code,
        })
        .map_err(SeedCidError::Generate)?;
        let cid = validate_cid_number_format(cid.as_str()).map_err(SeedCidError::Validate)?;
        if !exists_fn(&cid).map_err(SeedCidError::Exists)? {
            return Ok(cid);
        }
    }
    Err(SeedCidError::Exhausted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    fn never_exists(_: &str) -> Result<bool, Infallible> {
        Ok(false)
    }

    #[test]
    fn official_seed_matches_inline_byte_for_byte() {
        let from_seed = official_institution_cid(
            "NATIONAL",
            "GD",
            "001",
            "",
            "CGOV",
            "广东省",
            "荔湾市",
            never_exists,
        )
        .expect("official cid");
        let expected = generate_cid_number(GenerateCidInput {
            account_pubkey: "GOV-NATIONAL-GD-001--CGOV",
            p1: "0",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "CGOV",
        })
        .expect("inline cid");
        assert_eq!(from_seed, expected);
    }

    #[test]
    fn city_police_uses_official_city_seed() {
        let from_seed = official_institution_cid(
            "CITY",
            "GD",
            "001",
            "",
            "CPOL",
            "广东省",
            "荔湾市",
            never_exists,
        )
        .expect("official city police cid");
        let inline = generate_cid_number(GenerateCidInput {
            account_pubkey: "GOV-CITY-GD-001--CPOL",
            p1: "0",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "CPOL",
        })
        .unwrap();
        assert_eq!(from_seed, inline);
    }

    #[test]
    fn dynamic_returns_validated_cid() {
        let cid = dynamic_institution_cid("广东省", "荔湾市", "CGOV", "0", never_exists)
            .expect("dyn cid");
        assert!(crate::cid::validate_cid_number_format(&cid).is_ok());
    }
}
