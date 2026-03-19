//! 全节点铸块与发行常量=pow_const.rs

/// PoW 初始难度（创世难度，链启动后由动态难度调整算法自动维护）。
/// 开发链极低难度，单机秒出。
#[cfg(not(feature = "dev-chain"))]
pub const POW_INITIAL_DIFFICULTY: u64 = 1_000_000; // 正式链
#[cfg(feature = "dev-chain")]
pub const POW_INITIAL_DIFFICULTY: u64 = 100; // 开发链：极低难度

/// 难度调整周期（每隔多少块调整一次难度）。
/// 600 块 × 6 分钟 = 60 小时调整一次，兼顾响应速度与稳定性。
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 600;

/// 单次调整最大倍率上限（防止难度暴涨）：新难度不超过旧难度的 4 倍。
pub const DIFFICULTY_MAX_ADJUST_FACTOR: u64 = 4;

/// 单次调整最小倍率下限（防止难度暴跌）：新难度不低于旧难度的 1/4。
pub const DIFFICULTY_MIN_ADJUST_FACTOR: u64 = 4;

/// 目标调整窗口总时长（毫秒）= 调整间隔块数 × 目标出块时间。
/// 用于难度调整公式的分母基准值。
pub const DIFFICULTY_TARGET_WINDOW_MS: u64 =
    DIFFICULTY_ADJUSTMENT_INTERVAL as u64 * MILLISECS_PER_BLOCK;

/// 一、全节点铸块，每个区块奖励金额（单位：分）
pub const FULLNODE_BLOCK_REWARD: u128 = 999_900; // 每个区块奖励：9999.00 元 = 999_900 分

/// 二、全节点发行区块范围
pub const FULLNODE_REWARD_START_BLOCK: u32 = 1; // 全节点奖励起始区块高度（含）
pub const FULLNODE_REWARD_END_BLOCK: u32 = 9_999_999; // 全节点奖励结束区块高度（含）

/// 三、全节点发行总量（用于审计/校验）
pub const FULLNODE_REWARD_BLOCK_COUNT: u32 = // 全节点发行区块总数
    FULLNODE_REWARD_END_BLOCK - FULLNODE_REWARD_START_BLOCK + 1;
pub const FULLNODE_TOTAL_ISSUANCE: u128 = // 全节点发行总量（单位：分）， = 999_900 * 9_999_999
    FULLNODE_BLOCK_REWARD * FULLNODE_REWARD_BLOCK_COUNT as u128;

/// 四. 区块与时间参数（Block & Time）
///
/// 开发链（`dev-chain` feature）：30 秒出块，加速开发调试。
/// 正式链（默认）：6 分钟出块。
#[cfg(not(feature = "dev-chain"))]
pub const MILLISECS_PER_BLOCK: u64 = 360_000; // 正式链：360,000 毫秒（6 分钟）
#[cfg(feature = "dev-chain")]
pub const MILLISECS_PER_BLOCK: u64 = 30_000; // 开发链：30,000 毫秒（30 秒）

pub const MINUTES_PER_BLOCK: u64 = if MILLISECS_PER_BLOCK >= 60_000 {
    MILLISECS_PER_BLOCK / 60_000
} else {
    1 // 开发链出块时间不足 1 分钟，视为 1
};
pub const BLOCKS_PER_HOUR: u64 = 3_600_000 / MILLISECS_PER_BLOCK; // 每小时区块数
pub const BLOCKS_PER_DAY: u64 = BLOCKS_PER_HOUR * 24; // 每天区块数
pub const BLOCKS_PER_YEAR: u64 = BLOCKS_PER_DAY * 365; // 每年区块数
