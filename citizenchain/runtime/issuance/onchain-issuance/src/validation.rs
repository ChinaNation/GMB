//! 入参校验工具。
//!
//! 三大校验入口:
//! - `ensure_issuer_allowed` — 发行人主体类型必须 ∈ {SfidInstitution, PersonalDuoqian}
//! - `ensure_decimals_in_range` — decimals 必须落在 [0, 18]
//! - `contains_blacklisted_word` — name / symbol / description 字段不可命中黑名单
//! - `ensure_class_supported` — 第一期只支持 Plain,Pegged 直接 reject

use crate::types::AssetClass;
use primitives::derive::{parse_subject_id, SubjectKind};
use sp_std::vec::Vec;

/// decimals 范围铁律:`0..=18`(与 ERC-20 主流上限对齐,与 GMB 8 位兼容)。
pub const MIN_DECIMALS: u8 = 0;
pub const MAX_DECIMALS: u8 = 18;

/// 校验发行人主体类型(必须 SfidInstitution 0x02 或 PersonalDuoqian 0x03)。
///
/// 中文注释:Builtin 0x01(国储会等)与 OnchainAsset 0x04(代币本身)不允许直接发币;
/// 0x04 是用户代币 storage key 派生位,不是发行人主体身份。
pub fn ensure_issuer_allowed(subject_id: &[u8; 48]) -> Result<(), &'static str> {
    let (kind, _) = parse_subject_id(subject_id).ok_or("invalid_subject_id")?;
    match kind {
        SubjectKind::SfidInstitution | SubjectKind::PersonalDuoqian => Ok(()),
        _ => Err("issuer_not_allowed"),
    }
}

/// 校验 decimals 在合法区间。
pub fn ensure_decimals_in_range(decimals: u8) -> Result<(), &'static str> {
    if (MIN_DECIMALS..=MAX_DECIMALS).contains(&decimals) {
        Ok(())
    } else {
        Err("decimals_out_of_range")
    }
}

/// 校验资产 class 是否被第一期支持。
///
/// 中文注释:Pegged 协议位预留,Phase 2 启用前一律 reject,避免锚定语义滑入。
pub fn ensure_class_supported(class: &AssetClass) -> Result<(), &'static str> {
    match class {
        AssetClass::Plain => Ok(()),
        AssetClass::Pegged => Err("unsupported_asset_class"),
    }
}

/// 检查字段是否命中黑名单(忽略大小写,中文直接字节匹配)。
///
/// 中文注释:输入字段先全部 ASCII 小写化,再与黑名单逐词 substring 匹配。
/// 中文 UTF-8 字节固定,直接匹配。本函数是 O(N×M) 朴素扫描,
/// 词表规模小(<256 条)+ 字段长度小(<256 字节)下足够,无需 Aho-Corasick。
pub fn contains_blacklisted_word(field: &[u8], blacklist: &[Vec<u8>]) -> bool {
    let lowered: Vec<u8> = field
        .iter()
        .map(|&b| if b.is_ascii_uppercase() { b + 32 } else { b })
        .collect();
    blacklist.iter().any(|word| {
        if word.is_empty() || word.len() > lowered.len() {
            return false;
        }
        lowered.windows(word.len()).any(|w| w == word.as_slice())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::derive::{
        build_subject_id, subject_id_from_account, subject_id_from_onchain_asset,
    };

    #[test]
    fn issuer_only_accepts_0x02_and_0x03() {
        // 0x02 SfidInstitution → ok
        let sfid = build_subject_id(SubjectKind::SfidInstitution, b"CN-110000-0001").unwrap();
        assert!(ensure_issuer_allowed(&sfid).is_ok());

        // 0x03 PersonalDuoqian → ok
        let acc: [u8; 32] = [0x77; 32];
        let pers = subject_id_from_account(&acc);
        assert!(ensure_issuer_allowed(&pers).is_ok());

        // 0x01 Builtin → reject
        let builtin =
            build_subject_id(SubjectKind::Builtin, b"GFR-LN001-CB0X-944805165-2026").unwrap();
        assert!(ensure_issuer_allowed(&builtin).is_err());

        // 0x04 OnchainAsset → reject
        let onchain = subject_id_from_onchain_asset(1);
        assert!(ensure_issuer_allowed(&onchain).is_err());
    }

    #[test]
    fn decimals_boundary() {
        assert!(ensure_decimals_in_range(0).is_ok());
        assert!(ensure_decimals_in_range(8).is_ok());
        assert!(ensure_decimals_in_range(18).is_ok());
        assert!(ensure_decimals_in_range(19).is_err());
        assert!(ensure_decimals_in_range(255).is_err());
    }

    #[test]
    fn class_only_plain() {
        assert!(ensure_class_supported(&AssetClass::Plain).is_ok());
        assert!(ensure_class_supported(&AssetClass::Pegged).is_err());
    }

    #[test]
    fn blacklist_hits_case_insensitive_ascii() {
        let blacklist = vec![b"usd".to_vec()];
        assert!(contains_blacklisted_word(b"usd-token", &blacklist));
        assert!(contains_blacklisted_word(b"USD-Token", &blacklist));
        assert!(contains_blacklisted_word(b"Some USD here", &blacklist));
        assert!(!contains_blacklisted_word(b"safecoin", &blacklist));
    }

    #[test]
    fn blacklist_hits_chinese_bytes() {
        // 「人民币」UTF-8 字节序
        let rmb_bytes = b"\xe4\xba\xba\xe6\xb0\x91\xe5\xb8\x81".to_vec();
        let blacklist = vec![rmb_bytes];
        // 「数字人民币」字段命中「人民币」
        let field = b"\xe6\x95\xb0\xe5\xad\x97\xe4\xba\xba\xe6\xb0\x91\xe5\xb8\x81";
        assert!(contains_blacklisted_word(field, &blacklist));
    }
}
