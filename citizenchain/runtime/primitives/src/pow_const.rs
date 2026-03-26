//! 全节点铸块与发行常量=pow_const.rs
//!
//! # 出块时间与难度
//! 运行时真实出块目标时间由 genesis-pallet 链上存储控制（创世期 30s / 运行期 6min）。
//! 本文件的 MILLISECS_PER_BLOCK 仅作为创世占位和 node 层 fallback 默认值，
//! 固定为创世期值 30,000 ms，不再通过 cfg(feature) 分裂。

/// PoW 初始难度（创世难度，链启动后由动态难度调整算法自动维护）。
/// 固定为低难度，确保创世期单机可出块。运行期通过难度调整算法自动攀升。
pub const POW_INITIAL_DIFFICULTY: u64 = 100;

/// 难度调整周期（每隔多少块调整一次难度）。
/// 600 块 × 30 秒 = 5 小时（创世期），600 块 × 6 分钟 = 60 小时（运行期）。
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 600;

/// 单次调整最大倍率上限（防止难度暴涨）：新难度不超过旧难度的 4 倍。
pub const DIFFICULTY_MAX_ADJUST_FACTOR: u64 = 4;

/// 单次调整最小倍率下限（防止难度暴跌）：新难度不低于旧难度的 1/4。
pub const DIFFICULTY_MIN_ADJUST_FACTOR: u64 = 4;

/// 目标调整窗口总时长（毫秒），仅用于 benchmark 和 test。
/// 生产代码已改为从 genesis-pallet 链上存储读取。
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
/// 编译期占位，固定为创世期值（30 秒）。
/// 运行时真实出块目标时间由 genesis-pallet::TargetBlockTimeMs 链上存储控制。
/// node 层矿工门控通过 GenesisPalletApi::target_block_time_ms() Runtime API 读取。
pub const MILLISECS_PER_BLOCK: u64 = 30_000; // 30,000 毫秒（30 秒）

/// 以下派生常量基于创世期 30 秒出块计算。
/// 运行期切换到 6 分钟后，涉及时间的链上逻辑需从 genesis-pallet 读取真实值。
pub const SECONDS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1_000;
pub const BLOCKS_PER_HOUR: u64 = 3_600_000 / MILLISECS_PER_BLOCK; // 每小时区块数（创世期 120）
pub const BLOCKS_PER_DAY: u64 = BLOCKS_PER_HOUR * 24; // 每天区块数（创世期 2,880）
pub const BLOCKS_PER_YEAR: u64 = BLOCKS_PER_DAY * 365; // 每年区块数（创世期 1,051,200）
