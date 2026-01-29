//! ======================================================
//! GMB 投票治理常量
//! - 内部投票 ≠ 联合投票
//! - 内部投票只产出「是否通过」
//! - 联合投票只统计内部通过的机构
//! ======================================================

/// =======================
/// 一、机构基础数量
/// =======================

/// 国储会管理员数量
pub const NRC_ADMIN_COUNT: u32 = 19;

/// 单个省储会管理员数量
pub const PRC_ADMIN_COUNT: u32 = 9;

/// 单个省储行管理员数量
pub const PRB_ADMIN_COUNT: u32 = 9;

/// 省储会数量
pub const PRC_COUNT: u32 = 43;

/// 省储行数量
pub const PRB_COUNT: u32 = 43;


/// =======================
/// 二、内部投票阈值（只用于内部投票）
/// =======================

/// 国储会内部投票通过阈值
pub const NRC_INTERNAL_THRESHOLD: u32 = 13;

/// 省储会内部投票通过阈值
pub const PRC_INTERNAL_THRESHOLD: u32 = 6;

/// 省储行内部投票通过阈值
pub const PRB_INTERNAL_THRESHOLD: u32 = 6;


/// =======================
/// 三、联合投票结构（只看内部投票结果）
/// =======================

/// 国储会在联合投票中的票数（仅当内部投票通过）
pub const NRC_JOINT_VOTE_WEIGHT: u32 = 19;

/// 单个省储会在联合投票中的票数（仅当内部投票通过）
pub const PRC_JOINT_VOTE_WEIGHT: u32 = 1;

/// 单个省储行在联合投票中的票数（仅当内部投票通过）
pub const PRB_JOINT_VOTE_WEIGHT: u32 = 1;

/// 联合投票总票数
/// 19 + 43 + 43 = 105
pub const JOINT_VOTE_TOTAL: u32 = 105;

/// 联合投票通过条件：必须全票
pub const JOINT_VOTE_PASS_THRESHOLD: u32 = 105;


/// =======================
/// 四、公民投票
/// =======================

/// 公民投票通过比例（百分比）
pub const PUBLIC_VOTE_PASS_PERCENT: u32 = 50;


/// =======================
/// 五、投票阶段定义（流程控制）
/// =======================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoteStage {
    /// 内部投票
    Internal,

    /// 联合投票
    Joint,

    /// 公民投票
    Public,
}