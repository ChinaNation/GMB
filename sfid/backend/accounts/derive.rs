//! DUOQIAN 机构账户地址派生。
//!
//! 与链端 `citizenchain/runtime/primitives/src/core_const.rs`
//! `derive_duoqian_account` + `organization-manage` 的账户名路由完全对齐,用于:
//!   - 创建账户时立即在本地算出 `duoqian_account`,无需等激活上链
//!   - 激活成功后做 receipt 地址 ↔ 本地派生值的一致性断言,抓链端路由 / domain 漂移
//!
//! ## DUOQIAN 协议
//!
//! | op_tag | 账户类型            | account_name              | preimage 是否含 account_name |
//! |--------|---------------------|---------------------------|------------------------------|
//! | 0x00   | 主账户              | `"主账户"`(UTF-8 中文)   | 否                           |
//! | 0x01   | 费用账户            | `"费用账户"`(UTF-8 中文) | 否                           |
//! | 0x02   | 永久质押            | `"永久质押"`(UTF-8 中文) | 否                           |
//! | 0x03   | 安全基金            | `"安全基金"`(UTF-8 中文) | 否                           |
//! | 0x04   | 两和基金            | `"两和基金"`(UTF-8 中文) | 否                           |
//! | 0x06   | 用户自定义其他账户  | 用户输入的任意非空字符串 | 是                           |
//!
//! ```text
//! preimage = b"DUOQIAN"                      // 7 字节 domain
//!         || op_tag                          // 1 字节 (0x00 / 0x01 / 0x02 / 0x03 / 0x04 / 0x06)
//!         || ss58.to_le_bytes()              // 2 字节 (2027 LE = [0xEB, 0x07])
//!         || sfid_number.as_bytes()          // 变长
//!         || account_name.as_bytes()         // 仅 0x06 追加;0x00 / 0x01 不追加
//! duoqian_account = blake2b_256(preimage)    // 32 字节
//! ```
//!
//! ## 链端唯一真源
//!
//! - `primitives/src/core_const.rs`:`DUOQIAN = b"DUOQIAN"`、`OP_MAIN = 0x00`、
//!   `OP_FEE = 0x01`、`OP_STAKE = 0x02`、`OP_AN = 0x03`、`OP_HE = 0x04`、
//!   `OP_INSTITUTION = 0x06`、`derive_duoqian_account(op_tag, ss58, payload)`
//!   = `blake2_256(DUOQIAN || op_tag || ss58.to_le_bytes() || payload)`
//! - `organization-manage/src/lib.rs` `derive_institution_account`:
//!   `payload = sfid_number || name_suffix`,`Main`/`Fee`/制度账户的 name_suffix 为空
//! - `organization-manage/src/lib.rs` `role_from_account_name`:
//!   保留账户名走固定角色;其他非空 → `Named(account_name)`

use primitives::core_const::{
    derive_duoqian_account as derive_duoqian_account_bytes, OP_AN, OP_FEE, OP_HE, OP_INSTITUTION,
    OP_MAIN, OP_STAKE, SS58_FORMAT,
};

/// 主账户保留名(UTF-8 字节,9 字节)。链端同字节常量。
const RESERVED_NAME_MAIN: &str = "主账户";
/// 费用账户保留名(UTF-8 字节,12 字节)。
const RESERVED_NAME_FEE: &str = "费用账户";
/// 省储行永久质押账户保留名。
const RESERVED_NAME_STAKE: &str = "永久质押";
/// 国储会安全基金账户保留名。
const RESERVED_NAME_ANQUAN: &str = "安全基金";
/// 国储会两和基金账户保留名。
const RESERVED_NAME_HE: &str = "两和基金";

/// 全部 5 个受限保留账户名(单一源,与链端字节对齐)。
/// 自定义账户判定:account_name 命中其一即非自定义(走各自 op_tag),否则为
/// `OP_INSTITUTION` 自定义命名账户。citizenapp BFF 据此过滤 custom_account_names。
pub(crate) const RESERVED_ACCOUNT_NAMES: [&str; 5] = [
    RESERVED_NAME_MAIN,
    RESERVED_NAME_FEE,
    RESERVED_NAME_STAKE,
    RESERVED_NAME_ANQUAN,
    RESERVED_NAME_HE,
];

/// 按 `account_name` 路由并派生机构账户的 `duoqian_account`(小写 hex,32 字节 → 64 字符)。
///
/// 返回 `None` 当 `account_name` 为空串(与链端 `EmptyAccountName` 对齐的前置拒绝,
/// 不做 trim:链端按原始字节派生,本端必须字节对齐)。
///
/// ### 路由
/// - `"主账户"` → `OP_MAIN`(preimage 不含 account_name)
/// - `"费用账户"` → `OP_FEE`(preimage 不含 account_name)
/// - `"永久质押"` → `OP_STAKE`(preimage 不含 account_name)
/// - `"安全基金"` → `OP_AN`(preimage 不含 account_name)
/// - `"两和基金"` → `OP_HE`(preimage 不含 account_name)
/// - 其他非空 → `OP_INSTITUTION`(preimage 追加 account_name 字节)
pub fn derive_duoqian_account(sfid_number: &str, account_name: &str) -> Option<String> {
    let name = account_name;
    if name.is_empty() {
        return None;
    }
    let (op_tag, name_suffix): (u8, &[u8]) = if name == RESERVED_NAME_MAIN {
        (OP_MAIN, &[])
    } else if name == RESERVED_NAME_FEE {
        (OP_FEE, &[])
    } else if name == RESERVED_NAME_STAKE {
        (OP_STAKE, &[])
    } else if name == RESERVED_NAME_ANQUAN {
        (OP_AN, &[])
    } else if name == RESERVED_NAME_HE {
        (OP_HE, &[])
    } else {
        (OP_INSTITUTION, name.as_bytes())
    };
    let sfid_bytes = sfid_number.as_bytes();
    let mut payload = Vec::with_capacity(sfid_bytes.len() + name_suffix.len());
    payload.extend_from_slice(sfid_bytes);
    payload.extend_from_slice(name_suffix);
    let digest = derive_duoqian_account_bytes(op_tag, SS58_FORMAT, &payload);
    Some(hex::encode(digest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_account_preimage_excludes_name() {
        // 两个不同 sfid 的"主账户"地址必定不同(sfid 参与派生)
        let a = derive_duoqian_account("AH001-SZG1P-123456789-2026", "主账户").unwrap();
        let b = derive_duoqian_account("BJ001-SZG1I-987654321-2026", "主账户").unwrap();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
    }

    #[test]
    fn main_and_fee_differ_for_same_sfid() {
        // 同一 sfid 的主账户 / 费用账户地址不同(op_tag 不同)
        let sfid = "AH001-SZG1P-123456789-2026";
        let main = derive_duoqian_account(sfid, "主账户").unwrap();
        let fee = derive_duoqian_account(sfid, "费用账户").unwrap();
        assert_ne!(main, fee);
    }

    #[test]
    fn named_uses_account_name_in_preimage() {
        // 自定义账户名不同 → 地址不同;空 / 保留名按路由规则处理
        let sfid = "AH001-SZG1P-123456789-2026";
        let wage = derive_duoqian_account(sfid, "工资账户").unwrap();
        let case = derive_duoqian_account(sfid, "办案账户").unwrap();
        assert_ne!(wage, case);
        // 英文 "Main" / "Fee" 作为 Named 走 OP_INSTITUTION,不应等于主/费账户地址
        let named_main = derive_duoqian_account(sfid, "Main").unwrap();
        let reserved_main = derive_duoqian_account(sfid, "主账户").unwrap();
        assert_ne!(named_main, reserved_main);
    }

    #[test]
    fn reserved_policy_accounts_use_dedicated_tags() {
        let sfid = "LN001-GCB05-944805165-2026";
        let stake = derive_duoqian_account(sfid, "永久质押").unwrap();
        let anquan = derive_duoqian_account(sfid, "安全基金").unwrap();
        let he = derive_duoqian_account(sfid, "两和基金").unwrap();
        let named_stake = derive_duoqian_account_bytes(
            OP_INSTITUTION,
            SS58_FORMAT,
            "LN001-GCB05-944805165-2026永久质押".as_bytes(),
        );
        assert_ne!(stake, hex::encode(named_stake));
        assert_ne!(stake, anquan);
        assert_ne!(anquan, he);
    }

    #[test]
    fn empty_name_returns_none() {
        // 仅空串返回 None;不做 trim,纯空白串按字节参与派生(与链端字节对齐)
        assert!(derive_duoqian_account("sfid", "").is_none());
        assert!(derive_duoqian_account("sfid", "   ").is_some());
    }

    #[test]
    fn deterministic() {
        // 同输入必定同输出(幂等)
        let sfid = "AH001-SZG1P-123456789-2026";
        let a = derive_duoqian_account(sfid, "主账户").unwrap();
        let b = derive_duoqian_account(sfid, "主账户").unwrap();
        assert_eq!(a, b);
    }
}
