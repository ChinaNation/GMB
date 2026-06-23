//! CID 号确定性种子 + 撞号重试的唯一真源。
//!
//! 中文注释(单源铁律):
//! 三类 CID 的「种子约定 + 撞号重试」全部收敛在本文件,调用方一律调它,
//! 不再各自拼种子/写重试循环。底层号码结构仍由 `generator::generate_cid_number`
//! 单源生成;本文件只负责「喂什么 account_pubkey 种子」+「碰撞后怎么换种子重试」。
//!
//! 三类种子约定(逐字节复刻原调用方,行为中性):
//! - 公权机构(政府模板,创世确定性):`official_institution_cid`
//!   种子 = `GOV-{scope}-{province}-{city}-{town}-{institution}`,**无重试**(创世确定性,
//!   原 `gov/service.rs` 行为)。
//! - 公民人(绑定兜底):`citizen_cid`
//!   种子 = `wallet_pubkey`(`retry==0`)或 `wallet_pubkey#{retry}`,1000 次 DB 查重重试。
//! - 机构动态注册:`dynamic_institution_cid`
//!   种子 = 随机 UUIDv4,1000 次 DB 查重重试 + 格式校验。
//!
//! DB 查重一律倒置为 `exists_fn: impl Fn(&str) -> Result<bool, E>` 回调,
//! 本模块不依赖 `AppState`/数据库类型,纯函数 + 回调。

use crate::number::generator::{generate_cid_number, GenerateCidInput};
use crate::number::validator::validate_cid_number_format;

/// 撞号重试上限(公民人 / 动态机构共用)。
const COLLISION_RETRY_LIMIT: u32 = 1000;

/// CID 种子构造失败原因。`E` 为调用方 `exists_fn` 的查重错误类型。
#[derive(Debug)]
pub enum SeedCidError<E> {
    /// 底层号码生成失败(机构码非法、行政区缺失等),`&'static str` 同 generator。
    Generate(&'static str),
    /// 生成的号码格式校验失败(理论不可达,留作纵深防御),同 generator/validator。
    Validate(&'static str),
    /// 调用方 DB 查重回调返回的错误。
    Exists(E),
    /// 1000 次重试仍碰撞,桶饱和。
    Exhausted,
}

/// 公权机构(政府模板)CID — 确定性种子,**无重试**。
///
/// 中文注释:逐字节复刻原 `gov/service.rs::push_area_template_target` 的
/// `account_seed = "GOV-{scope}-{province_code}-{city_code}-{town_code}-{institution_code}"`,
/// 创世幂等故不重试;碰撞概率由 (机构码,省,市,年) 桶 + 种子唯一性保证。
/// `exists_fn` 形参保留(供未来守卫/审计),当前调用方传 `|_| Ok(false)` 即等价原行为。
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
    let account_seed =
        format!("GOV-{scope}-{province_code}-{city_code}-{town_code}-{institution_code}");
    let cid = generate_cid_number(GenerateCidInput {
        account_pubkey: account_seed.as_str(),
        p1: "0",
        province_name,
        city_name,
        institution: institution_code,
    })
    .map_err(SeedCidError::Generate)?;
    // 创世确定性:不重试。保留查重回调供守卫,碰撞即报错(原行为下永不触发)。
    if exists_fn(&cid).map_err(SeedCidError::Exists)? {
        return Err(SeedCidError::Exhausted);
    }
    Ok(cid)
}

/// 市公安局(CPOL)CID — 历史确定性种子 `PS-{province_code}-{city_code}`,创世无重试。
///
/// 中文注释:逐字节复刻原 `gov/service.rs::generate_public_security_cid`:
/// 种子 `PS-{省码}-{市码}`,机构码固定 `CPOL`,`p1="0"`。不得改成 GOV-CITY 模板种子,
/// 否则平移既有公安局 CID。`exists_fn` 形参保留(供守卫),创世传 `|_| Ok(false)` 等价原行为。
pub fn public_security_cid<E>(
    province_code: &str,
    city_code: &str,
    province_name: &str,
    city_name: &str,
    exists_fn: impl Fn(&str) -> Result<bool, E>,
) -> Result<String, SeedCidError<E>> {
    let account_seed = format!("PS-{province_code}-{city_code}");
    let cid = generate_cid_number(GenerateCidInput {
        account_pubkey: account_seed.as_str(),
        p1: "0",
        province_name,
        city_name,
        institution: "CPOL",
    })
    .map_err(SeedCidError::Generate)?;
    if exists_fn(&cid).map_err(SeedCidError::Exists)? {
        return Err(SeedCidError::Exhausted);
    }
    Ok(cid)
}

/// 公民人(绑定兜底)CID — `wallet_pubkey` 种子 + 1000 次重试。
///
/// 中文注释:逐字节复刻原 `citizens/binding.rs::generate_unique_citizen_cid`:
/// `retry==0` 用裸 `wallet_pubkey`,否则 `wallet_pubkey#{retry}`;机构码固定 `CTZN`,
/// `p1="1"`,`city_name="省辖市"`(市级段被 generator 固定为 000)。
pub fn citizen_cid<E>(
    wallet_pubkey: &str,
    province_name: &str,
    exists_fn: impl Fn(&str) -> Result<bool, E>,
) -> Result<String, SeedCidError<E>> {
    for retry in 0..COLLISION_RETRY_LIMIT {
        let attempt_pubkey = if retry == 0 {
            wallet_pubkey.to_string()
        } else {
            format!("{wallet_pubkey}#{retry}")
        };
        let candidate = generate_cid_number(GenerateCidInput {
            account_pubkey: attempt_pubkey.as_str(),
            // 公民人:个人主体码 CTZN(固定盈利,p1 被忽略,市级段固定 000)。
            p1: "1",
            province_name,
            city_name: "省辖市",
            institution: "CTZN",
        })
        .map_err(SeedCidError::Generate)?;
        if !exists_fn(&candidate).map_err(SeedCidError::Exists)? {
            return Ok(candidate);
        }
    }
    Err(SeedCidError::Exhausted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::generator::{generate_cid_number, GenerateCidInput};
    use std::convert::Infallible;

    fn never_exists(_: &str) -> Result<bool, Infallible> {
        Ok(false)
    }

    // 中文注释:逐字节复刻铁证 —— official 种子拼装与原 gov/service.rs 完全一致。
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
        let inline_seed = "GOV-NATIONAL-GD-001--CGOV";
        let expected = generate_cid_number(GenerateCidInput {
            account_pubkey: inline_seed,
            p1: "0",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "CGOV",
        })
        .expect("inline cid");
        assert_eq!(from_seed, expected);
    }

    // 中文注释:确定性 —— 同输入两次产出同号。
    #[test]
    fn official_is_deterministic() {
        let a = official_institution_cid(
            "NATIONAL",
            "GD",
            "001",
            "",
            "CGOV",
            "广东省",
            "荔湾市",
            never_exists,
        )
        .unwrap();
        let b = official_institution_cid(
            "NATIONAL",
            "GD",
            "001",
            "",
            "CGOV",
            "广东省",
            "荔湾市",
            never_exists,
        )
        .unwrap();
        assert_eq!(a, b);
    }

    // 中文注释:公安局 PS-{省码}-{市码} 种子与原 gov/service.rs::generate_public_security_cid 逐字节一致。
    #[test]
    fn public_security_seed_matches_inline_byte_for_byte() {
        let from_seed =
            public_security_cid("GD", "001", "广东省", "荔湾市", never_exists).expect("ps cid");
        let inline = generate_cid_number(GenerateCidInput {
            account_pubkey: "PS-GD-001",
            p1: "0",
            province_name: "广东省",
            city_name: "荔湾市",
            institution: "CPOL",
        })
        .unwrap();
        assert_eq!(from_seed, inline);
    }

    // 中文注释:公民人 retry==0 用裸 wallet_pubkey,与原 binding.rs 一致。
    #[test]
    fn citizen_first_attempt_matches_inline() {
        let from_seed = citizen_cid("0xabc", "广东省", never_exists).expect("citizen cid");
        let expected = generate_cid_number(GenerateCidInput {
            account_pubkey: "0xabc",
            p1: "1",
            province_name: "广东省",
            city_name: "省辖市",
            institution: "CTZN",
        })
        .expect("inline citizen cid");
        assert_eq!(from_seed, expected);
    }

    // 中文注释:碰撞时切到 wallet_pubkey#1 种子(逐字节复刻)。
    #[test]
    fn citizen_retries_with_hash_suffix_on_collision() {
        let first = generate_cid_number(GenerateCidInput {
            account_pubkey: "0xdef",
            p1: "1",
            province_name: "广东省",
            city_name: "省辖市",
            institution: "CTZN",
        })
        .unwrap();
        let second_seed_expected = generate_cid_number(GenerateCidInput {
            account_pubkey: "0xdef#1",
            p1: "1",
            province_name: "广东省",
            city_name: "省辖市",
            institution: "CTZN",
        })
        .unwrap();
        // 第一号当作已存在,强制走 #1 重试。
        let got = citizen_cid("0xdef", "广东省", |c| Ok::<bool, Infallible>(c == first)).unwrap();
        assert_eq!(got, second_seed_expected);
    }

    // 中文注释:1000 次全碰撞 → Exhausted。
    #[test]
    fn citizen_exhausts_when_all_collide() {
        let r = citizen_cid("0x000", "广东省", |_| Ok::<bool, Infallible>(true));
        assert!(matches!(r, Err(SeedCidError::Exhausted)));
    }

    // 中文注释:动态机构 —— 随机 UUID 种子,返回已校验归一化号且格式合法。
    #[test]
    fn dynamic_returns_validated_cid() {
        let cid = dynamic_institution_cid("广东省", "荔湾市", "CGOV", "0", never_exists)
            .expect("dyn cid");
        assert!(crate::number::validate_cid_number_format(&cid).is_ok());
    }
}

/// 机构动态注册 CID — 随机 UUIDv4 种子 + 1000 次重试 + 格式校验。
///
/// 中文注释:逐字节复刻原 `subjects/registration.rs` 的随机 UUID 循环:
/// 每轮 `Uuid::new_v4()` 作 `account_pubkey`,生成后 `validate_cid_number_format`
/// 归一化,再用 `exists_fn` 查重;返回**已校验归一化**的号(原行为是把校验后的
/// `cid` 用于后续 DB 写入,故此处返回校验后的串)。
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
