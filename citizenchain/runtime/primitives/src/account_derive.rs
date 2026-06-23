//! 账户地址派生唯一真源 = account_derive.rs (ADR-024 Tier 1)
//!
//! 把账户派生的 op_tag、5 个受限保留名、name→种类路由、每种 payload 字段拼装、
//! 以及唯一派生入口全部收敛到本模块。其它任何 crate / 模块禁止再本地重声明或重拼
//! `DUOQIAN || op_tag || ss58 || payload`，一律调本模块。
//!
//! 域分隔符 `DUOQIAN`(与签名共用)仍留在 `core_const`,本模块 import 使用。
//! 地址派生 preimage = DUOQIAN (7B) || op_tag (1B) || ss58 (2B little-endian) || payload。
//! `address = BLAKE2-256(preimage)` → 32 字节 AccountId。
//!
//! Dart 侧(citizenapp `account_derivation.dart` + 两份 `reserved_account_names.dart`)
//! 是本模块的手写镜像,无编译期保证;靠金标向量
//! (`tests/fixtures/account_derive_vectors.json`,CI 脚本 `tools/sync_account_derive_vectors.sh`)
//! 逐字节断言对齐,防止跨语言漂移。新增 op_tag / 账户种类只改本模块 + 刷新金标。
//!
//! 改名 `DUOQIAN→GMB`(域字节变更,会改地址)+ `OP_INSTITUTION→OP_NAME` 的字节值
//! 属 ADR-024 Tier 3,与 T3/T4 末尾创世一起做,本批不动(`OP_NAME` 名已改、值仍 0x06)。

use crate::core_const::DUOQIAN; // 域共享(签名也用),留在 core_const
use sp_core::hashing::blake2_256;
use sp_std::vec::Vec;

// ── 地址派生 op_tag (0x00-0x0F) ──
// 每个 op_tag 单一派生公式,不得复用。OP_MAIN / OP_FEE 覆盖所有机构,保留名
// "主账户"/"费用账户" 必须强制走这两个 tag,禁止落到 OP_NAME。
// OP_NAME 仅容纳 CID 机构的自定义命名账户。
pub const OP_MAIN: u8 = 0x00; // 所有机构主账户 · input: cid_number
pub const OP_FEE: u8 = 0x01; // 所有机构费用账户 · input: cid_number
pub const OP_STAKE: u8 = 0x02; // 永久质押 · input: cid_number
pub const OP_AN: u8 = 0x03; // 安全基金 · input: cid_number
pub const OP_HE: u8 = 0x04; // 两和基金 · input: cid_number
pub const OP_PERSONAL: u8 = 0x05; // 个人多签账户 · input: creator_32 || account_name
pub const OP_NAME: u8 = 0x06; // CID 机构自定义命名账户 · input: cid_number || account_name(原 OP_INSTITUTION,值不变)

/// 机构账户受限注册保留名(单一权威源)。
///
/// - `主账户` / `费用账户`:每个机构强制生成的默认账户,创建时强制路由
///   `OP_MAIN`/`OP_FEE`,不得作为自定义命名账户。
/// - `永久质押` / `安全基金` / `两和基金`:制度专属账户,普通 CID 机构禁止注册,
///   account_name 命中即拒绝(`ReservedAccountName`)。
/// 5 个保留名的 UTF-8 字符串原始字面(全仓**唯一**字面源)。链端按 `&[u8]` 比对、
/// 后端 build_default_accounts 等按 `&str` 取用,一律 import 这里,禁止任何位置另写字面。
pub const RESERVED_NAME_MAIN_STR: &str = "主账户";
pub const RESERVED_NAME_FEE_STR: &str = "费用账户";
pub const RESERVED_NAME_STAKE_STR: &str = "永久质押";
pub const RESERVED_NAME_ANQUAN_STR: &str = "安全基金";
pub const RESERVED_NAME_HE_STR: &str = "两和基金";

pub const RESERVED_NAME_MAIN: &[u8] = RESERVED_NAME_MAIN_STR.as_bytes();
pub const RESERVED_NAME_FEE: &[u8] = RESERVED_NAME_FEE_STR.as_bytes();
pub const RESERVED_NAME_STAKE: &[u8] = RESERVED_NAME_STAKE_STR.as_bytes();
pub const RESERVED_NAME_ANQUAN: &[u8] = RESERVED_NAME_ANQUAN_STR.as_bytes();
pub const RESERVED_NAME_HE: &[u8] = RESERVED_NAME_HE_STR.as_bytes();

/// 全部 5 个受限保留名,供各端遍历校验。
pub const RESERVED_ACCOUNT_NAMES: [&[u8]; 5] = [
    RESERVED_NAME_MAIN,
    RESERVED_NAME_FEE,
    RESERVED_NAME_STAKE,
    RESERVED_NAME_ANQUAN,
    RESERVED_NAME_HE,
];

/// account_name 是否为"禁止注册"的制度专属保留名(永久质押/安全基金/两和基金)。
///
/// 主账户/费用账户不在此列:它们走强制默认路由,不是"禁止"而是"强制"。不 trim。
pub fn is_forbidden_account_name(name: &[u8]) -> bool {
    name == RESERVED_NAME_STAKE || name == RESERVED_NAME_ANQUAN || name == RESERVED_NAME_HE
}

/// op_tag + payload 字段 schema 的唯一权威映射。
///
/// 三种 payload schema(异构是账户种类本质,不抹平):
/// - 机构 主/费/质押/安全/两和 → `cid_number`
/// - 机构自定义 → `cid_number || account_name`
/// - 个人多签 → `creator(32B) || account_name`
#[derive(Clone, Copy, Debug)]
pub enum AccountKind<'a> {
    InstitutionMain {
        cid_number: &'a [u8],
    },
    InstitutionFee {
        cid_number: &'a [u8],
    },
    InstitutionStake {
        cid_number: &'a [u8],
    },
    InstitutionAnquan {
        cid_number: &'a [u8],
    },
    InstitutionHe {
        cid_number: &'a [u8],
    },
    InstitutionNamed {
        cid_number: &'a [u8],
        account_name: &'a [u8],
    },
    Personal {
        creator: &'a [u8; 32],
        account_name: &'a [u8],
    },
}

impl<'a> AccountKind<'a> {
    /// 该账户种类对应的 op_tag。
    pub const fn op_tag(&self) -> u8 {
        match self {
            AccountKind::InstitutionMain { .. } => OP_MAIN,
            AccountKind::InstitutionFee { .. } => OP_FEE,
            AccountKind::InstitutionStake { .. } => OP_STAKE,
            AccountKind::InstitutionAnquan { .. } => OP_AN,
            AccountKind::InstitutionHe { .. } => OP_HE,
            AccountKind::InstitutionNamed { .. } => OP_NAME,
            AccountKind::Personal { .. } => OP_PERSONAL,
        }
    }

    /// payload 字段拼装的唯一处。
    fn payload(&self) -> Vec<u8> {
        match self {
            AccountKind::InstitutionMain { cid_number }
            | AccountKind::InstitutionFee { cid_number }
            | AccountKind::InstitutionStake { cid_number }
            | AccountKind::InstitutionAnquan { cid_number }
            | AccountKind::InstitutionHe { cid_number } => cid_number.to_vec(),
            AccountKind::InstitutionNamed {
                cid_number,
                account_name,
            } => {
                let mut payload = Vec::with_capacity(cid_number.len() + account_name.len());
                payload.extend_from_slice(cid_number);
                payload.extend_from_slice(account_name);
                payload
            }
            AccountKind::Personal {
                creator,
                account_name,
            } => {
                let mut payload = Vec::with_capacity(creator.len() + account_name.len());
                payload.extend_from_slice(*creator);
                payload.extend_from_slice(account_name);
                payload
            }
        }
    }

    /// 账户地址唯一派生入口。
    ///
    /// preimage = DUOQIAN (7B) || op_tag (1B) || ss58 (2B little-endian) || payload。
    /// `address = BLAKE2-256(preimage)`。任何账户(主/费/质押/安全/两和/个人多签/
    /// 机构自定义)都必须经本入口派生,禁止在其它模块重拼 preimage。
    pub fn derive(&self, ss58: u16) -> [u8; 32] {
        let ss58_le = ss58.to_le_bytes();
        let payload = self.payload();
        let mut preimage = Vec::with_capacity(DUOQIAN.len() + 1 + ss58_le.len() + payload.len());
        preimage.extend_from_slice(DUOQIAN);
        preimage.push(self.op_tag());
        preimage.extend_from_slice(&ss58_le);
        preimage.extend_from_slice(&payload);
        blake2_256(&preimage)
    }
}

/// 唯一路由表:name → AccountKind。
///
/// 主/费/质押/安全/两和 各自映射到专属种类;其它非空名 → `InstitutionNamed`;
/// 空名 → `None`。只做派生路由,不做"能否注册"校验(注册策略见
/// `is_registrable_custom_name`)。
pub fn institution_kind_by_name<'a>(
    cid_number: &'a [u8],
    name: &'a [u8],
) -> Option<AccountKind<'a>> {
    if name.is_empty() {
        return None;
    }
    if name == RESERVED_NAME_MAIN {
        return Some(AccountKind::InstitutionMain { cid_number });
    }
    if name == RESERVED_NAME_FEE {
        return Some(AccountKind::InstitutionFee { cid_number });
    }
    if name == RESERVED_NAME_STAKE {
        return Some(AccountKind::InstitutionStake { cid_number });
    }
    if name == RESERVED_NAME_ANQUAN {
        return Some(AccountKind::InstitutionAnquan { cid_number });
    }
    if name == RESERVED_NAME_HE {
        return Some(AccountKind::InstitutionHe { cid_number });
    }
    Some(AccountKind::InstitutionNamed {
        cid_number,
        account_name: name,
    })
}

/// 注册策略(非派生):account_name 能否作为机构自定义命名账户注册。
///
/// 空 / 主账户 / 费用账户 / 制度专属(质押/安全/两和) 一律不可作自定义名。不 trim。
pub fn is_registrable_custom_name(name: &[u8]) -> bool {
    !name.is_empty()
        && name != RESERVED_NAME_MAIN
        && name != RESERVED_NAME_FEE
        && !is_forbidden_account_name(name)
}
