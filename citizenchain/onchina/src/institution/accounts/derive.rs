//! GMB 机构账户地址派生(后端薄适配)。
//!
//! 账户派生唯一真源 = 链端 `primitives::account_derive`:op_tag、
//! 5 个受限保留名、name→种类路由、payload 字段拼装、唯一派生入口 `AccountKind::derive`
//! 全部收敛在那里。本模块仅做 `&str` → `&[u8]`
//! 适配 + hex 编码,用于:
//!   - 创建账户时立即在本地算出规范 `account_id`，无需等激活上链
//!   - 激活成功后做 receipt 地址 ↔ 本地派生值的一致性断言,抓链端路由 / domain 漂移
//!
//! ## GMB 协议(定义见 `primitives::account_derive`)
//!
//! | op_tag | 账户类型            | account_name              | preimage 是否含 account_name |
//! |--------|---------------------|---------------------------|------------------------------|
//! | 0x00   | 主账户              | `"主账户"`(UTF-8 中文)   | 否                           |
//! | 0x01   | 费用账户            | `"费用账户"`(UTF-8 中文) | 否                           |
//! | 0x02   | 永久质押            | `"永久质押"`(UTF-8 中文) | 否                           |
//! | 0x03   | 安全基金            | `"安全基金"`(UTF-8 中文) | 否                           |
//! | 0x04   | 两和基金            | `"两和基金"`(UTF-8 中文) | 否                           |
//! | 0x06   | 清算账户            | `"清算账户"`(UTF-8 中文) | 否                           |
//! | 0x07   | 用户自定义其他账户  | 用户输入的任意非空字符串 | 是                           |
//!
//! ```text
//! preimage = b"GMB"                          // 3 字节 domain
//!         || op_tag                          // 1 字节 (0x00 / 0x01 / 0x02 / 0x03 / 0x04 / 0x06 / 0x07)
//!         || ss58.to_le_bytes()              // 2 字节 (2027 LE = [0xEB, 0x07])
//!         || cid_number.as_bytes()          // 变长
//!         || account_name.as_bytes()         // 仅 0x07 追加;0x00 / 0x01 / 0x06 不追加
//! account_id = 0x || hex(blake2b_256(preimage))
//! ```

use primitives::account_derive::{self, RESERVED_ACCOUNT_NAMES as RESERVED_ACCOUNT_NAME_BYTES};
use primitives::core_const::SS58_FORMAT;

/// 全部 6 个受限保留账户名(单一源 = 链端 `account_derive`,UTF-8 字符串视图)。
///
/// 自定义账户判定:account_name 命中其一即非自定义(走各自 op_tag),否则为
/// `OP_NAME` 自定义命名账户。CitizenApp BFF 据此过滤 custom_account_names。
pub(crate) fn reserved_account_names() -> [String; 6] {
    [
        String::from_utf8_lossy(RESERVED_ACCOUNT_NAME_BYTES[0]).into_owned(),
        String::from_utf8_lossy(RESERVED_ACCOUNT_NAME_BYTES[1]).into_owned(),
        String::from_utf8_lossy(RESERVED_ACCOUNT_NAME_BYTES[2]).into_owned(),
        String::from_utf8_lossy(RESERVED_ACCOUNT_NAME_BYTES[3]).into_owned(),
        String::from_utf8_lossy(RESERVED_ACCOUNT_NAME_BYTES[4]).into_owned(),
        String::from_utf8_lossy(RESERVED_ACCOUNT_NAME_BYTES[5]).into_owned(),
    ]
}

/// 按 `account_name` 路由并派生机构账户的规范 `account_id`。
///
/// 路由 / op_tag / payload 拼装全部委托给 `account_derive::institution_kind_by_name`
/// + `AccountKind::derive`(唯一真源)。
///
/// 返回 `None` 当 `account_name` 为空串(与链端 `EmptyAccountName` 对齐的前置拒绝,
/// 不做 trim:链端按原始字节派生,本端必须字节对齐)。
///
/// ### 路由(定义在 `account_derive`)
/// - `"主账户"` → `OP_MAIN`(preimage 不含 account_name)
/// - `"费用账户"` → `OP_FEE`(preimage 不含 account_name)
/// - `"永久质押"` → `OP_STAKE`(preimage 不含 account_name)
/// - `"安全基金"` → `OP_SAFETY`(preimage 不含 account_name)
/// - `"两和基金"` → `OP_HE`(preimage 不含 account_name)
/// - `"清算账户"` → `OP_CLEARING`(preimage 不含 account_name)
/// - 其他非空 → `OP_NAME`(preimage 追加 account_name 字节)
pub fn derive_account_id(cid_number: &str, account_name: &str) -> Option<String> {
    derive_account_bytes(cid_number, account_name)
        .map(|account_id| format!("0x{}", hex::encode(account_id)))
}

/// 按 `account_name` 路由并派生机构账户地址的 32 字节 `AccountId`(唯一真源同上)。
///
/// 返回 `None` 当 `account_name` 为空串(与链端 `EmptyAccountName` 对齐)。
/// 关闭账户提案需要账户与主账户的裸 32 字节 AccountId(SCALE 无长度前缀),用本函数取。
pub(crate) fn derive_account_bytes(cid_number: &str, account_name: &str) -> Option<[u8; 32]> {
    account_derive::institution_kind_by_name(cid_number.as_bytes(), account_name.as_bytes())
        .map(|kind| kind.derive(SS58_FORMAT))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::account_derive::{AccountKind, OP_NAME};

    #[test]
    fn main_account_preimage_excludes_name() {
        // 两个不同 cid 的"主账户"地址必定不同(cid 参与派生)
        let a = derive_account_id("AH001-SZG1P-123456789-2026", "主账户").unwrap();
        let b = derive_account_id("BJ001-SZG1I-987654321-2026", "主账户").unwrap();
        assert_eq!(a.len(), 66);
        assert_ne!(a, b);
    }

    #[test]
    fn main_and_fee_differ_for_same_cid() {
        // 同一 cid 的主账户 / 费用账户地址不同(op_tag 不同)
        let cid = "AH001-SZG1P-123456789-2026";
        let main = derive_account_id(cid, "主账户").unwrap();
        let fee = derive_account_id(cid, "费用账户").unwrap();
        assert_ne!(main, fee);
    }

    #[test]
    fn named_uses_account_name_in_preimage() {
        // 自定义账户名不同 → 地址不同;空 / 保留名按路由规则处理
        let cid = "AH001-SZG1P-123456789-2026";
        let wage = derive_account_id(cid, "工资账户").unwrap();
        let case = derive_account_id(cid, "办案账户").unwrap();
        assert_ne!(wage, case);
        // 英文 "Main" / "Fee" 作为 Named 走 OP_NAME,不应等于主/费账户地址
        let named_main = derive_account_id(cid, "Main").unwrap();
        let reserved_main = derive_account_id(cid, "主账户").unwrap();
        assert_ne!(named_main, reserved_main);
    }

    #[test]
    fn reserved_policy_accounts_use_dedicated_tags() {
        let cid = "LN001-NRC0G-944805165-2026";
        let stake = derive_account_id(cid, "永久质押").unwrap();
        let safety_fund = derive_account_id(cid, "安全基金").unwrap();
        let he = derive_account_id(cid, "两和基金").unwrap();
        // 同名走 OP_NAME(cid||name)的地址不应等于走专属 OP_STAKE 的地址
        let named_stake = hex::encode(
            AccountKind::InstitutionNamed {
                cid_number: cid.as_bytes(),
                account_name: "永久质押".as_bytes(),
            }
            .derive(SS58_FORMAT),
        );
        assert_ne!(stake, named_stake);
        assert_ne!(stake, safety_fund);
        assert_ne!(safety_fund, he);
        // 显式确认自定义命名走 OP_NAME=0x00(永久冻结,不随新增协议账户移动)
        assert_eq!(
            AccountKind::InstitutionNamed {
                cid_number: cid.as_bytes(),
                account_name: "办案账户".as_bytes(),
            }
            .op_tag(),
            OP_NAME
        );
    }

    #[test]
    fn empty_name_returns_none() {
        // 仅空串返回 None;不做 trim,纯空白串按字节参与派生(与链端字节对齐)
        assert!(derive_account_id("cid", "").is_none());
        assert!(derive_account_id("cid", "   ").is_some());
    }

    #[test]
    fn deterministic() {
        // 同输入必定同输出(幂等)
        let cid = "AH001-SZG1P-123456789-2026";
        let a = derive_account_id(cid, "主账户").unwrap();
        let b = derive_account_id(cid, "主账户").unwrap();
        assert_eq!(a, b);
    }
}
