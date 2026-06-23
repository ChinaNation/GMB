//! 核心常量 = core_const.rs

/// 1. 货币基础参数（Economic Base）
pub const TOKEN_SYMBOL: &str = "GMB"; // 公民币符号
pub const TOKEN_DECIMALS: u32 = 2; // 精度：2 位（元 / 分制），1 GMB = 100 FEN
pub const TOKEN_MIN_UNIT: u128 = 1; // 最小计价单位（1 分）
pub const SS58_FORMAT: u16 = 2027; // 地址格式前缀（SS58）
pub const CHAIN_NAME: &str = "CitizenChain"; // 链显示名称
pub const CHAIN_ID: &str = "citizenchain"; // 链唯一 ID（chain spec id）
pub const SUPPORT_URL: &str = "https://www.crcfrcn.com"; // 官方支持网址
pub const BLOCK_HASH_COUNT: u32 = 2400; // 最近区块哈希保留数量
pub const NORMAL_DISPATCH_PERCENT: u32 = 75; // 普通交易可用区块权重比例
pub const MAX_BLOCK_BYTES: u32 = 100 * 1024 * 1024; // 单区块最大字节数：100MB

/// 2. 省储行质押年利率模型 (Annual Interest Rate)
pub const PROVINCIALBANK_INITIAL_INTEREST_BP: u32 = 100; // 省储行初始年利率（第一年）：1.00%
pub const PROVINCIALBANK_INTEREST_DECREASE_BP: u32 = 1; // 年利率递减值：0.01%
pub const PROVINCIALBANK_INTEREST_DURATION_YEARS: u32 = 100; // 利率递减年限（100 年后归零）
pub const ENABLE_PROVINCIALBANK_INTEREST_DECAY: bool = true; // 是否启用逐年递减利率模型

// 3. 安全与反滥用参数（Security）
pub const ACCOUNT_EXISTENTIAL_DEPOSIT: u128 = 111; // 账户存在最低余额（Existential Deposit），余额 < 111 分 → 链上账户状态被删除，剩余余额销毁
pub const ALLOW_ZERO_BALANCE_ACCOUNT: bool = false; // 是否允许零余额账户存在（必须关闭）
pub const ENABLE_DUST_CLEANUP: bool = true; // 是否允许 Dust 回收（必须开启）
pub const ALLOW_LOCAL_ADDRESS_GENERATION: bool = true; // 是否允许无限地址本地生成（链下）

/// 4. 统一签名/派生域铁律（Unified Signature & Derivation Domain）
/// 全仓库地址派生（BLAKE2-256）+ 签名 payload（sr25519）统一使用 `GMB` 前缀，后接 1 字节 `op_tag` 做子命名空间。
/// 地址派生 preimage = GMB (3B) || op_tag (1B) || ss58 (2B little-endian) || payload_bytes。
/// 地址派生：`address = BLAKE2-256(preimage)` → 32 字节 AccountId。
/// 签名 payload：`message = BLAKE2-256(SCALE.encode(tuple("GMB", ..., payload_fields)))`。
///
/// 账户地址派生的 op_tag、5 保留名、name→种类路由、payload 拼装与唯一派生入口
/// (`AccountKind::derive`) 在单一真源 `account_derive`；
/// 本文件账户派生相关只剩两项:① 域分隔符 `GMB`(地址派生 + 签名共用),
/// ② 签名 payload op_tag(`OP_SIGN_*`,0x10-0x1F,非账户派生)。
pub const GMB: &[u8; 3] = b"GMB";

/// 5. CID 机构号(cid_number)链上/链下统一最大字节数（单一权威源）。
/// 真实格式 `R5-K3P1C1-N9-D4` 定长 26 字节（如 `LN001-NRC0G-944805165-2026`），
/// 取 32 留余量。链端 `MaxCidNumberLength`、CID 后端、各端测试一律 import 本常量，
/// 禁止任何位置另写死长度值。
pub const CID_NUMBER_MAX_BYTES: u32 = 32;

// 签名 payload op_tag (0x10-0x1F) — 单一权威源在 `crate::sign`。
// 此处 re-export 供 `core_const::OP_SIGN_*` 调用路径使用。新增 op_tag 只改 `sign`。
pub use crate::sign::{OP_SIGN_BIND, OP_SIGN_DEREGISTER, OP_SIGN_INST, OP_SIGN_POP, OP_SIGN_VOTE};
