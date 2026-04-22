//! DUOQIAN_V1 机构账户地址派生。
//!
//! 与链端 `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs`
//! `Pallet::derive_institution_address` + `role_from_account_name` 完全对齐的
//! 纯 Rust 实现(无需 substrate 依赖),用于:
//!   - 创建账户时立即在本地算出 `duoqian_address`,无需等激活上链
//!   - 激活成功后做 receipt 地址 ↔ 本地派生值的一致性断言,抓链端路由 / domain 漂移
//!
//! ## DUOQIAN_V1 协议(3 种账户共用一个域)
//!
//! | op_tag | 账户类型            | account_name              | preimage 是否含 account_name |
//! |--------|---------------------|---------------------------|------------------------------|
//! | 0x00   | 主账户              | `"主账户"`(UTF-8 中文)   | 否                           |
//! | 0x01   | 费用账户            | `"费用账户"`(UTF-8 中文) | 否                           |
//! | 0x05   | 用户自定义其他账户  | 用户输入的任意非空字符串  | 是                           |
//!
//! ```text
//! preimage = b"DUOQIAN_V1"                  // 10 字节 domain
//!         || op_tag                         // 1 字节 (0x00 / 0x01 / 0x05)
//!         || ss58_prefix.to_le_bytes()      // 2 字节 (2027 LE = [0xEB, 0x07])
//!         || sfid_id.as_bytes()             // 变长
//!         || account_name.as_bytes()        // 仅 0x05 追加;0x00 / 0x01 不追加
//! duoqian_address = blake2b_256(preimage)   // 32 字节
//! ```
//!
//! ## 链端事实依据
//!
//! - `primitives/core_const.rs:45-55`:`DUOQIAN_DOMAIN = b"DUOQIAN_V1"`、
//!   `OP_MAIN = 0x00`、`OP_FEE = 0x01`、`OP_INSTITUTION = 0x05`
//! - `duoqian-manage-pow/src/lib.rs:1199-1223`:`derive_institution_address`
//! - `duoqian-manage-pow/src/lib.rs:1234-1247`:`role_from_account_name`
//!   按字节匹配:`"主账户".as_bytes() → Role::Main`;`"费用账户".as_bytes() → Role::Fee`;
//!   其他非空 → `Role::Named(account_name)`

use blake2::{digest::consts::U32, Blake2b, Digest};

/// Domain 前缀(10 字节)。
pub const DUOQIAN_DOMAIN: &[u8; 10] = b"DUOQIAN_V1";

/// 主账户 op_tag。
pub const OP_MAIN: u8 = 0x00;
/// 费用账户 op_tag。
pub const OP_FEE: u8 = 0x01;
/// 用户自定义机构账户 op_tag。
pub const OP_INSTITUTION: u8 = 0x05;

/// citizenchain 全局 SS58 前缀(little-endian 编码入 preimage)。
pub const SS58_PREFIX: u16 = 2027;

/// 主账户保留名(UTF-8 字节,9 字节)。链端同字节常量。
pub const RESERVED_NAME_MAIN: &str = "主账户";
/// 费用账户保留名(UTF-8 字节,12 字节)。
pub const RESERVED_NAME_FEE: &str = "费用账户";

/// 按 `account_name` 路由并派生机构账户的 `duoqian_address`(小写 hex,32 字节 → 64 字符)。
///
/// 返回 `None` 当 `account_name` 去空后为空串(与链端 `EmptyAccountName` 对齐的前置拒绝)。
///
/// ### 路由
/// - `"主账户"` → `OP_MAIN`(preimage 不含 account_name)
/// - `"费用账户"` → `OP_FEE`(preimage 不含 account_name)
/// - 其他非空 → `OP_INSTITUTION`(preimage 追加 account_name 字节)
pub fn derive_duoqian_address(sfid_id: &str, account_name: &str) -> Option<String> {
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
    let ss58_le = SS58_PREFIX.to_le_bytes();
    let sfid_bytes = sfid_id.as_bytes();
    let mut buf = Vec::with_capacity(
        DUOQIAN_DOMAIN.len() + 1 + ss58_le.len() + sfid_bytes.len() + name_suffix.len(),
    );
    buf.extend_from_slice(DUOQIAN_DOMAIN);
    buf.push(op_tag);
    buf.extend_from_slice(&ss58_le);
    buf.extend_from_slice(sfid_bytes);
    buf.extend_from_slice(name_suffix);
    let digest = Blake2b::<U32>::digest(&buf);
    Some(hex::encode(digest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_account_preimage_excludes_name() {
        // 两个不同 sfid 的"主账户"地址必定不同(sfid 参与派生)
        let a = derive_duoqian_address("SFR-AH001-ZG0X-123456789-20260101", "主账户").unwrap();
        let b = derive_duoqian_address("SFR-BJ001-ZG0X-987654321-20260101", "主账户").unwrap();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
    }

    #[test]
    fn main_and_fee_differ_for_same_sfid() {
        // 同一 sfid 的主账户 / 费用账户地址不同(op_tag 不同)
        let sfid = "SFR-AH001-ZG0X-123456789-20260101";
        let main = derive_duoqian_address(sfid, "主账户").unwrap();
        let fee = derive_duoqian_address(sfid, "费用账户").unwrap();
        assert_ne!(main, fee);
    }

    #[test]
    fn named_uses_account_name_in_preimage() {
        // 自定义账户名不同 → 地址不同;空 / 保留名按路由规则处理
        let sfid = "SFR-AH001-ZG0X-123456789-20260101";
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
        let sfid = "SFR-AH001-ZG0X-123456789-20260101";
        let a = derive_duoqian_address(sfid, "主账户").unwrap();
        let b = derive_duoqian_address(sfid, "主账户").unwrap();
        assert_eq!(a, b);
    }
}
