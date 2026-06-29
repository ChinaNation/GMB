//! 入参校验工具。
//!
//! 三大校验入口:
//! - `ensure_issuer_allowed` — 发行人必须是机构多签账户地址
//! - `ensure_decimals_in_range` — decimals 必须落在 [0, 18]
//! - `contains_blacklisted_word` — name / symbol / description 字段不可命中黑名单
//! - `ensure_class_supported` — 第一期只支持 Plain,Pegged 直接 reject

use crate::types::AssetClass;
use sp_std::vec::Vec;

/// decimals 范围铁律:`0..=18`(与 ERC-20 主流上限对齐,与 GMB 8 位兼容)。
pub const MIN_DECIMALS: u8 = 0;
pub const MAX_DECIMALS: u8 = 18;

/// 校验发行机构账户地址。
///
/// 中文注释：具体“是否为已注册机构多签、发起人是否为该账户管理员”由 pallet 调用
/// admins 模块 / 实体生命周期模块的账户级接口完成；本函数只拒绝空编码。
pub fn ensure_issuer_allowed<AccountId: codec::Encode>(
    issuer_account: &AccountId,
) -> Result<(), &'static str> {
    if issuer_account.encode().is_empty() {
        Err("issuer_not_allowed")
    } else {
        Ok(())
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
/// 中文注释:Pegged 协议位预留,当前一律 reject,避免锚定语义滑入。
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
    use codec::Encode;

    #[test]
    fn issuer_accepts_account_id() {
        let acc: [u8; 32] = [0x77; 32];
        assert!(ensure_issuer_allowed(&acc).is_ok());
        assert!(!acc.encode().is_empty());
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
