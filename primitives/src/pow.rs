//! Pow 共识常量（GMB 全节点挖矿规则）

/// 初始难度（示例，后面可根据区块平均时间微调）
pub const POW_INITIAL_DIFFICULTY: u128 = 100000;

/// 难度调整间隔（单位：区块）
pub const POW_ADJUST_BLOCK_INTERVAL: u32 = 600;  // 每 600 区块调整一次

/// 目标出块时间（秒）
pub const POW_TARGET_BLOCK_TIME: u32 = 6;