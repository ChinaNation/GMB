//! GMB 区块链核心常量定义

use sp_runtime::Perbill;

// =======================================================
// 1. 货币基础参数（Economic Base）
// =======================================================

/// 代币符号
pub const TOKEN_SYMBOL: &str = "GMB";

/// 精度：2 位（元 / 分制），1 GMB = 100 FEN
pub const TOKEN_DECIMALS: u32 = 2;

/// 最小计价单位（1 分）
pub const TOKEN_MIN_UNIT: u128 = 1;

// =======================================================
// 2. 交易手续费模型（Fee Model）
// =======================================================

/// 链上交易费率：0.1%
pub const ONCHAIN_FEE_RATE: Perbill = Perbill::from_parts(1_000_000); // 0.1%

/// 链上交易最小手续费：0.1 元
pub const ONCHAIN_MIN_FEE: u128 = 10;

/// 链上交易费全节点分成比例：80%
pub const ONCHAIN_FEE_FULLNODE_PERCENT: u32 = 80;

/// 链上交易费国储会分成比例：10%
pub const ONCHAIN_FEE_NRC_PERCENT: u32 = 10;

/// 链上交易费黑洞销毁比例：10%
pub const ONCHAIN_FEE_BLACKHOLE_PERCENT: u32 = 10;

/// 链下交易最小手续费：0.01 元
pub const OFFCHAIN_MIN_FEE: u128 = 1;

/// 链下交易费率下限：0.01%
pub const OFFCHAIN_FEE_RATE_MIN: Perbill = Perbill::from_parts(100_000); // 0.01%

/// 链下交易费率上限：0.1%
pub const OFFCHAIN_FEE_RATE_MAX: Perbill = Perbill::from_parts(1_000_000); // 0.1%

// =======================================================
// 3. 省储行质押年利率模型 (Annual Interest Rate)
// =======================================================

/// 省储行初始年利率（第一年）：5.00%
pub const SHENGBANK_INITIAL_INTEREST_BP: u32 = 500; 

/// 年利率递减值：0.05%
pub const SHENGBANK_INTEREST_DECREASE_BP: u32 = 5;

/// 利率递减年限（100 年后归零）
pub const SHENGBANK_INTEREST_DURATION_YEARS: u32 = 100;

/// 是否启用逐年递减利率模型
pub const ENABLE_SHENGBANK_INTEREST_DECAY: bool = true;

// =======================================================
// 4. 安全与反滥用参数（Security）
// =======================================================

/// 账户存在最低余额（Existential Deposit），余额 < 111 分 → 链上账户状态被删除，剩余余额销毁
pub const ACCOUNT_EXISTENTIAL_DEPOSIT: u128 = 111;

/// 是否允许零余额账户存在（必须关闭）
pub const ALLOW_ZERO_BALANCE_ACCOUNT: bool = false;

/// 是否允许 Dust 回收（必须开启）
pub const ENABLE_DUST_CLEANUP: bool = true;

/// 是否允许无限地址本地生成（链下）
pub const ALLOW_LOCAL_ADDRESS_GENERATION: bool = true;