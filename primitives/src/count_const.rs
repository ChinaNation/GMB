//! 投票治理常量=count_const.rs
//! - 内部投票 ≠ 联合投票，内部投票只产出内部「是否通过」的结果
//! - 联合投票只统计内部投票「是否通过」的结果，内部投票通过则该机构在联合投票中投赞成票，否则为反对票

use crate::{pow_const, reserve_nodes_const, shengbank_nodes_const};

/// 一、机构基础数量
pub const NRC_ADMIN_COUNT: u32 = 19;                // 国储会管理员数量
pub const PRC_ADMIN_COUNT: u32 = 9;                 // 单个省储会管理员数量
pub const PRB_ADMIN_COUNT: u32 = 9;                 // 单个省储行管理员数量
pub const NRC_COUNT: u32 =1;                        // 国储会数量
pub const PRC_COUNT: u32 = (reserve_nodes_const::RESERVE_NODES.len() - 1) as u32;       // 初始省储会数量（总储会-国储会）
pub const PRB_COUNT: u32 = shengbank_nodes_const::SHENG_BANK_NODES.len() as u32;        // 初始省储行数量（来自省储行数组）

/// 二、内部投票（只用于内部投票的阈值）
pub const NRC_INTERNAL_THRESHOLD: u32 = 13;         // 国储会内部投票通过阈值
pub const PRC_INTERNAL_THRESHOLD: u32 = 6;          // 省储会内部投票通过阈值
pub const PRB_INTERNAL_THRESHOLD: u32 = 6;          // 省储行内部投票通过阈值

/// 三、联合投票（只看内部投票结果）
pub const NRC_JOINT_VOTE_WEIGHT: u32 = 19;          // 国储会在联合投票中的票数（仅当国储会内部投票通过，通过=19票，未通过=0票）
pub const PRC_JOINT_VOTE_WEIGHT: u32 = 1;           // 单个省储会在联合投票中的票数（仅当省储会内部投票通过）
pub const PRB_JOINT_VOTE_WEIGHT: u32 = 1;           // 单个省储行在联合投票中的票数（仅当省储行内部投票通过）
pub const JOINT_VOTE_TOTAL: u32 =
    NRC_JOINT_VOTE_WEIGHT + (PRC_COUNT * PRC_JOINT_VOTE_WEIGHT) + (PRB_COUNT * PRB_JOINT_VOTE_WEIGHT);      // 联合投票总票数
pub const JOINT_VOTE_PASS_THRESHOLD: u32 = 105;     // 联合投票通过条件：全票通过则立即执行，非全票通过则进入公民投票流程

/// 四、公民投票（仅联合投票未通过的则进入公民投票流程）
pub const CITIZEN_VOTE_PASS_PERCENT: u32 = 50;      // 公民投票通过比例（百分比），大于50%则通过，否则则否决

/// 五、投票时限（单位：区块数）
pub const VOTING_DURATION_DAYS: u32 = 30;           // 投票默认期限30天，本期限用于审计与展示
pub const BLOCKS_PER_DAY: u32 = pow_const::BLOCKS_PER_DAY as u32;       // 每天区块数（统一来源：pow_const）
pub const VOTING_DURATION_BLOCKS: u32 =
    BLOCKS_PER_DAY as u32 * VOTING_DURATION_DAYS;   // 投票默认期限（区块）= 30 * BLOCKS_PER_DAY
pub const VOTING_TIMEOUT_IS_REJECTED: bool = true;  // 投票是否在超时后自动否决，是

/// 六、投票阶段定义（流程控制）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoteStage {
    Internal,   // 内部投票
    Joint,      // 联合投票
    Citizen,    // 公民投票
}

/// 七、决议发行常量
pub const RESOLUTION_ISSUANCE_MAX_REASON_LEN: u32 = 1024;               // 决议发行理由最大长度
pub const RESOLUTION_ISSUANCE_MAX_ALLOCATIONS: u32 = PRB_COUNT;         // 决议发行单次最大分配条目数（与省储行数量一致）
