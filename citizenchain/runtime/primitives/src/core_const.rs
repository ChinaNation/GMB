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
pub const SHENGBANK_INITIAL_INTEREST_BP: u32 = 100; // 省储行初始年利率（第一年）：1.00%
pub const SHENGBANK_INTEREST_DECREASE_BP: u32 = 1; // 年利率递减值：0.01%
pub const SHENGBANK_INTEREST_DURATION_YEARS: u32 = 100; // 利率递减年限（100 年后归零）
pub const ENABLE_SHENGBANK_INTEREST_DECAY: bool = true; // 是否启用逐年递减利率模型

// 3. 安全与反滥用参数（Security）
pub const ACCOUNT_EXISTENTIAL_DEPOSIT: u128 = 111; // 账户存在最低余额（Existential Deposit），余额 < 111 分 → 链上账户状态被删除，剩余余额销毁
pub const ALLOW_ZERO_BALANCE_ACCOUNT: bool = false; // 是否允许零余额账户存在（必须关闭）
pub const ENABLE_DUST_CLEANUP: bool = true; // 是否允许 Dust 回收（必须开启）
pub const ALLOW_LOCAL_ADDRESS_GENERATION: bool = true; // 是否允许无限地址本地生成（链下）

use sp_core::hashing::blake2_256;
use sp_std::vec::Vec;

/// 4. 统一签名/派生域铁律（Unified Signature & Derivation Domain）
/// 全仓库地址派生（BLAKE2-256）+ 签名 payload（sr25519）统一使用 `DUOQIAN` 前缀，后接 1 字节 `op_tag` 做子命名空间。
/// 地址派生 preimage = DUOQIAN (7B) || op_tag (1B) || ss58 (2B little-endian) || payload_bytes。
/// 地址派生：`address = BLAKE2-256(preimage)` → 32 字节 AccountId。
/// 签名 payload：`message = BLAKE2-256(SCALE.encode(tuple("DUOQIAN", ..., payload_fields)))`。
pub const DUOQIAN: &[u8; 7] = b"DUOQIAN";

// 地址派生 op_tag (0x00-0x0F)
// 每个 op_tag 单一派生公式，不得复用，OP_MAIN / OP_FEE 覆盖所有机构，保留名 "主账户"/"费用账户"
// 必须强制走这两个 tag，禁止落到 OP_INSTITUTION。OP_INSTITUTION 仅容纳 CID 机构的自定义命名账户。
pub const OP_MAIN: u8 = 0x00; // 所有机构主账户 · input: ss58 || cid_number
pub const OP_FEE: u8 = 0x01; // 所有机构费用账户 · input: ss58 || cid_number
pub const OP_STAKE: u8 = 0x02; // 永久质押 · input: ss58 || cid_number
pub const OP_AN: u8 = 0x03; // 安全基金 · input: ss58 || cid_number
pub const OP_HE: u8 = 0x04; // 两和基金 · input: ss58 || cid_number
pub const OP_PERSONAL: u8 = 0x05; // 个人多签账户 · input: ss58 || creator_32 || account_name
pub const OP_INSTITUTION: u8 = 0x06; // CID 机构自定义命名账户 · input: ss58 || cid_number || account_name
                                     //（account_name 非空且不得为 "主账户"/"费用账户"/"永久质押"/"安全基金"/"两和基金" 等保留角色名）

/// CID 机构号(cid_number)链上/链下统一最大字节数（单一权威源）。
///
/// 真实格式 `R5-K3P1C1-N9-D4` 定长 26 字节（如 `LN001-GCB05-944805165-2026`），
/// 取 32 留余量。链端 `MaxCidNumberLength`、CID 后端、各端测试一律 import 本常量，
/// 禁止任何位置另写死长度值。
pub const CID_NUMBER_MAX_BYTES: u32 = 32;

/// 机构账户受限注册保留名（单一权威源）。
///
/// - `主账户` / `费用账户`：每个机构强制生成的默认账户，创建时强制路由
///   `OP_MAIN`/`OP_FEE`，不得作为自定义命名账户。
/// - `永久质押` / `安全基金` / `两和基金`：制度专属账户，普通 CID 机构禁止注册，
///   account_name 命中即拒绝（`ReservedAccountName`）。
pub const RESERVED_NAME_MAIN: &[u8] = "主账户".as_bytes();
pub const RESERVED_NAME_FEE: &[u8] = "费用账户".as_bytes();
pub const RESERVED_NAME_STAKE: &[u8] = "永久质押".as_bytes();
pub const RESERVED_NAME_ANQUAN: &[u8] = "安全基金".as_bytes();
pub const RESERVED_NAME_HE: &[u8] = "两和基金".as_bytes();

/// 全部 5 个受限保留名，供各端遍历校验。
pub const RESERVED_ACCOUNT_NAMES: [&[u8]; 5] = [
    RESERVED_NAME_MAIN,
    RESERVED_NAME_FEE,
    RESERVED_NAME_STAKE,
    RESERVED_NAME_ANQUAN,
    RESERVED_NAME_HE,
];

/// account_name 是否为"禁止注册"的制度专属保留名（永久质押/安全基金/两和基金）。
///
/// 主账户/费用账户不在此列：它们走强制默认路由，不是"禁止"而是"强制"。
pub fn is_forbidden_account_name(name: &[u8]) -> bool {
    name == RESERVED_NAME_STAKE || name == RESERVED_NAME_ANQUAN || name == RESERVED_NAME_HE
}

/// DUOQIAN 账户地址唯一派生入口。
///
/// 中文注释：任何主账户、费用账户、永久质押、安全基金、两和基金、
/// 个人多签和机构自定义账户都必须调用本函数；禁止在其它模块重新拼接
/// `DUOQIAN || op_tag || ss58 || payload`。
pub fn derive_account(op_tag: u8, ss58: u16, payload: &[u8]) -> [u8; 32] {
    let ss58_le = ss58.to_le_bytes();
    let mut preimage = Vec::with_capacity(DUOQIAN.len() + 1 + ss58_le.len() + payload.len());
    preimage.extend_from_slice(DUOQIAN);
    preimage.push(op_tag);
    preimage.extend_from_slice(&ss58_le);
    preimage.extend_from_slice(payload);
    blake2_256(&preimage)
}

// 签名 payload op_tag (0x10-0x1F)
pub const OP_SIGN_BIND: u8 = 0x10; // 公民身份绑定
pub const OP_SIGN_VOTE: u8 = 0x11; // 公民投票
pub const OP_SIGN_POP: u8 = 0x12; // 人口快照
pub const OP_SIGN_INST: u8 = 0x13; // CID 机构登记
pub const OP_SIGN_DEREGISTER: u8 = 0x14; // CID 机构/账户注销凭证(注册局签发,链端 close 验签)
                                   // 所有治理投票一律走 `InternalVote::cast` 公开 call,业务模块
                                   // 新业务从 0x18 起分配,签名域 op_tag 空间共 0x10-0x1F。
