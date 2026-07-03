//! 全节点铸块与发行常量。
//! 真实出块目标时间由 genesis-pallet 链上存储控制。

/// PoW 创世难度。
pub const POW_INITIAL_DIFFICULTY: u64 = 100;

/// 难度调整周期。
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 600;

/// 单次调整最大倍率。
pub const DIFFICULTY_MAX_ADJUST_FACTOR: u64 = 4;

/// 单次调整最小倍率分母。
pub const DIFFICULTY_MIN_ADJUST_FACTOR: u64 = 4;

/// benchmark/test 用目标窗口。
pub const DIFFICULTY_TARGET_WINDOW_MS: u64 =
    DIFFICULTY_ADJUSTMENT_INTERVAL as u64 * MILLISECS_PER_BLOCK;

/// 全节点区块奖励,单位:分。
pub const FULLNODE_BLOCK_REWARD: u128 = 999_900; // 每个区块奖励：9999.00 元 = 999_900 分

// 全节点发行区块范围。
pub const FULLNODE_REWARD_START_BLOCK: u32 = 1; // 全节点奖励起始区块高度（含）
pub const FULLNODE_REWARD_END_BLOCK: u32 = 9_999_999; // 全节点奖励结束区块高度（含）

// 全节点发行总量。
pub const FULLNODE_REWARD_BLOCK_COUNT: u32 = // 全节点发行区块总数
    FULLNODE_REWARD_END_BLOCK - FULLNODE_REWARD_START_BLOCK + 1;
pub const FULLNODE_TOTAL_ISSUANCE: u128 = // 全节点发行总量（单位：分）， = 999_900 * 9_999_999
    FULLNODE_BLOCK_REWARD * FULLNODE_REWARD_BLOCK_COUNT as u128;

// 区块与时间参数。
pub const MILLISECS_PER_BLOCK: u64 = 30_000; // 30,000 毫秒（30 秒）

/// 投票到期等区块计数按运行期 6 分钟出块计算。
pub const SECONDS_PER_BLOCK: u64 = 360; // 运行期出块间隔：6 分钟 = 360 秒
pub const BLOCKS_PER_HOUR: u64 = 3_600 / SECONDS_PER_BLOCK; // 每小时区块数：10
pub const BLOCKS_PER_DAY: u64 = BLOCKS_PER_HOUR * 24; // 每天区块数：240

/// 省储行年度利息结算周期,单位:区块。
pub const BLOCKS_PER_YEAR: u64 = 87_600;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullnode_total_issuance_is_consistent() {
        // 全节点发行总量 = 区块奖励 × 发行区块数。
        assert_eq!(
            FULLNODE_TOTAL_ISSUANCE,
            FULLNODE_BLOCK_REWARD * FULLNODE_REWARD_BLOCK_COUNT as u128
        );
    }
}
