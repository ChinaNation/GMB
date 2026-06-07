//! DUOQIAN 机构账户地址派生。
//!
//! 与链端 `citizenchain/runtime/primitives/src/core_const.rs`
//! `derive_duoqian_account` + `organization-manage` 的账户名路由完全对齐,用于:
//!   - 创建账户时立即在本地算出 `duoqian_address`,无需等激活上链
//!   - 激活成功后做 receipt 地址 ↔ 本地派生值的一致性断言,抓链端路由 / domain 漂移
//!
//! ## DUOQIAN 协议
//!
//! | op_tag | 账户类型            | account_name              | preimage 是否含 account_name |
//! |--------|---------------------|---------------------------|------------------------------|
//! | 0x00   | 主账户              | `"主账户"`(UTF-8 中文)   | 否                           |
//! | 0x01   | 费用账户            | `"费用账户"`(UTF-8 中文) | 否                           |
//! | 0x06   | 用户自定义其他账户  | 用户输入的任意非空字符串  | 是                           |
//!
//! ```text
//! preimage = b"DUOQIAN"                      // 7 字节 domain
//!         || op_tag                          // 1 字节 (0x00 / 0x01 / 0x06)
//!         || ss58.to_le_bytes()              // 2 字节 (2027 LE = [0xEB, 0x07])
//!         || sfid_number.as_bytes()          // 变长
//!         || account_name.as_bytes()         // 仅 0x06 追加;0x00 / 0x01 不追加
//! duoqian_address = blake2b_256(preimage)    // 32 字节
//! ```
//!
//! ## 链端唯一真源
//!
//! - `primitives/src/core_const.rs`:`DUOQIAN = b"DUOQIAN"`、`OP_MAIN = 0x00`、
//!   `OP_FEE = 0x01`、`OP_INSTITUTION = 0x06`、`derive_duoqian_account(op_tag, ss58, payload)`
//!   = `blake2_256(DUOQIAN || op_tag || ss58.to_le_bytes() || payload)`
//! - `organization-manage/src/lib.rs` `derive_institution_address`:
//!   `payload = sfid_number || name_suffix`,`Main`/`Fee` 的 name_suffix 为空
//! - `organization-manage/src/lib.rs` `role_from_account_name`:
//!   `"主账户".as_bytes() → Main`;`"费用账户".as_bytes() → Fee`;其他非空 → `Named(account_name)`

use primitives::core_const::{
    derive_duoqian_account, OP_FEE, OP_INSTITUTION, OP_MAIN, SS58_FORMAT,
};

/// 主账户保留名(UTF-8 字节,9 字节)。链端同字节常量。
const RESERVED_NAME_MAIN: &str = "主账户";
/// 费用账户保留名(UTF-8 字节,12 字节)。
const RESERVED_NAME_FEE: &str = "费用账户";

/// 按 `account_name` 路由并派生机构账户的 `duoqian_address`(小写 hex,32 字节 → 64 字符)。
///
/// 返回 `None` 当 `account_name` 去空后为空串(与链端 `EmptyAccountName` 对齐的前置拒绝)。
///
/// ### 路由
/// - `"主账户"` → `OP_MAIN`(preimage 不含 account_name)
/// - `"费用账户"` → `OP_FEE`(preimage 不含 account_name)
/// - 其他非空 → `OP_INSTITUTION`(preimage 追加 account_name 字节)
pub fn derive_duoqian_address(sfid_number: &str, account_name: &str) -> Option<String> {
    let name = account_name.trim();
    if name.is_empty() {
        return None;
    }
    let (op_tag, name_suffix): (u8, &[u8]) = if name == RESERVED_NAME_MAIN {
        (OP_MAIN, &[])
    } else if name == RESERVED_NAME_FEE {
        (OP_FEE, &[])
    } else {
        (OP_INSTITUTION, name.as_bytes())
    };
    let sfid_bytes = sfid_number.as_bytes();
    let mut payload = Vec::with_capacity(sfid_bytes.len() + name_suffix.len());
    payload.extend_from_slice(sfid_bytes);
    payload.extend_from_slice(name_suffix);
    let digest = derive_duoqian_account(op_tag, SS58_FORMAT, &payload);
    Some(hex::encode(digest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_account_preimage_excludes_name() {
        // 两个不同 sfid 的"主账户"地址必定不同(sfid 参与派生)
        let a = derive_duoqian_address("AH001-SZG1P-123456789-2026", "主账户").unwrap();
        let b = derive_duoqian_address("BJ001-SZG1I-987654321-2026", "主账户").unwrap();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
    }

    #[test]
    fn main_and_fee_differ_for_same_sfid() {
        // 同一 sfid 的主账户 / 费用账户地址不同(op_tag 不同)
        let sfid = "AH001-SZG1P-123456789-2026";
        let main = derive_duoqian_address(sfid, "主账户").unwrap();
        let fee = derive_duoqian_address(sfid, "费用账户").unwrap();
        assert_ne!(main, fee);
    }

    #[test]
    fn named_uses_account_name_in_preimage() {
        // 自定义账户名不同 → 地址不同;空 / 保留名按路由规则处理
        let sfid = "AH001-SZG1P-123456789-2026";
        let wage = derive_duoqian_address(sfid, "工资账户").unwrap();
        let case = derive_duoqian_address(sfid, "办案账户").unwrap();
        assert_ne!(wage, case);
        // 英文 "Main" / "Fee" 作为 Named 走 OP_INSTITUTION,不应等于主/费账户地址
        let named_main = derive_duoqian_address(sfid, "Main").unwrap();
        let reserved_main = derive_duoqian_address(sfid, "主账户").unwrap();
        assert_ne!(named_main, reserved_main);
    }

    #[test]
    fn empty_name_returns_none() {
        assert!(derive_duoqian_address("sfid", "").is_none());
        assert!(derive_duoqian_address("sfid", "   ").is_none());
    }

    #[test]
    fn deterministic() {
        // 同输入必定同输出(幂等)
        let sfid = "AH001-SZG1P-123456789-2026";
        let a = derive_duoqian_address(sfid, "主账户").unwrap();
        let b = derive_duoqian_address(sfid, "主账户").unwrap();
        assert_eq!(a, b);
    }
}
