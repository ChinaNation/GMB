//! GMB 全节点铸块与发行常量

/// =======================
/// 一、全节点区块奖励金额
/// =======================

/// 每个区块奖励金额（单位：分）
/// 9999.00 元 = 999_900 分
pub const FULLNODE_BLOCK_REWARD: u128 = 999_900;

/// =======================
/// 二、全节点发行区块范围
/// =======================

/// 全节点奖励起始区块高度（含）
pub const FULLNODE_REWARD_START_BLOCK: u32 = 1;

/// 全节点奖励结束区块高度（含）
pub const FULLNODE_REWARD_END_BLOCK: u32 = 9_999_999;

/// =======================
/// 三、全节点发行总量（用于审计/校验）
/// =======================

/// 全节点发行区块总数
pub const FULLNODE_REWARD_BLOCK_COUNT: u32 =
    FULLNODE_REWARD_END_BLOCK - FULLNODE_REWARD_START_BLOCK + 1;

/// 全节点发行总量（单位：分）
/// = 999_900 * 9_999_999
pub const FULLNODE_TOTAL_ISSUANCE: u128 =
    FULLNODE_BLOCK_REWARD * FULLNODE_REWARD_BLOCK_COUNT as u128;

// =======================================================
// 四. 区块与时间参数（Block & Time）
// =======================================================

/// 目标区块时间：360,000 毫秒（即 6 分钟）
pub const MILLISECS_PER_BLOCK: u64 = 360_000;

/// 每分钟区块数（6 分钟一个块，所以每分钟约为 1/6 个块）
pub const BLOCKS_PER_MINUTE: u64 = 60_000 / MILLISECS_PER_BLOCK; 

/// 每小时区块数 (60 / 6 = 10 个块)
pub const BLOCKS_PER_HOUR: u64 = 10;

/// 每天区块数 (10 * 24 = 240 个块)
pub const BLOCKS_PER_DAY: u64 = BLOCKS_PER_HOUR * 24;

/// 每年区块数（365 天：240 * 365 = 87,600 个块）
pub const BLOCKS_PER_YEAR: u64 = BLOCKS_PER_DAY * 365;
