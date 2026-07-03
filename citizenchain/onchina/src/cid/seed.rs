//! onchina CID 种子 + 撞号重试唯一入口。
//!
//!
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
    // 直接调 primitives 生成器并钉死创世年份:公权机构号是创世直铸集,
    // 必须与链上派生逐字节一致;走按“当前年份”的适配层会在跨年后漂移。
    let cid = primitives::cid::generator::generate_cid_number(
        primitives::cid::generator::GenerateCidNumberInput {
            account_pubkey: account_seed.as_str(),
            p1: "0",
            province_code,
            province_name,
            city_code,
            city_name,
            year: primitives::cid::official_derive::GENESIS_INSTITUTION_YEAR,
            institution: institution_code,
        },
    )
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

#[cfg(test)]
mod official_source_tests {
    use super::*;

    /// 创世派生(primitives::official_derive)与 onchina 官方号派生必须逐字节同源:
    /// 全量 596,517 机构逐个复算,号完全一致(含年份钉死后跨年不漂移)。
    #[test]
    fn onchina_official_cid_matches_primitives_derivation() {
        let mut total = 0usize;
        primitives::cid::official_derive::for_each_public_institution_detailed(|item| {
            total += 1;
            let cid = official_institution_cid::<std::convert::Infallible>(
                item.scope,
                item.province_code,
                item.city_code,
                item.town_code,
                item.template.institution_code,
                item.province_name,
                item.city_name,
                |_| Ok(false),
            )
            .expect("onchina official cid derives");
            assert_eq!(
                cid, item.cid_number,
                "同源漂移: scope={} code={} area={}",
                item.scope, item.template.institution_code, item.display_area_name
            );
        });
        assert_eq!(
            total,
            primitives::cid::official_derive::public_institution_derived_count()
        );
    }
}
