//! 公民轻节点发行规则（CIIC 认证发行）
//! 说明：
//! - 本规则属于【系统增发】
//! - 仅在“首次完成 CIIC 认证”时触发
//! - 每个 CIIC 身份仅可领取一次
//! - 奖励金额与认证顺序强相关

/// =======================
/// 一、发行总量与阶段
/// =======================

/// 可获得奖励的公民轻节点最大数量
pub const CITIZEN_LIGHTNODE_MAX_COUNT: u64 = 1_443_497_378;

/// 高额奖励阶段节点数量（前 1%）
pub const CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT: u64 = 14_436_417;

/// =======================
/// 二、认证奖励金额（单位：分）
/// =======================

/// 前期高额认证奖励：9999.00 元
pub const CITIZEN_LIGHTNODE_HIGH_REWARD: u128 = 999_900;

/// 后期常规认证奖励：999.00 元
pub const CITIZEN_LIGHTNODE_NORMAL_REWARD: u128 = 99_900;

/// =======================
/// 三、认证发行规则
/// =======================

/// 每个 CIIC 身份是否只能领取一次奖励
pub const CITIZEN_LIGHTNODE_ONE_TIME_ONLY: bool = true;