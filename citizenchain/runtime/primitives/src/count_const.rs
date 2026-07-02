//! 投票治理常量=count_const.rs
//! - 内部投票 ≠ 联合投票，内部投票只产出内部「是否通过」的结果
//! - 联合投票只统计内部投票「是否通过」的结果，内部投票通过则该机构在联合投票中投赞成票，否则为反对票
//! - 所有投票类型支持提前通过和提前否决：赞成达到阈值立即通过，剩余票不足达到阈值立即否决
//! - 30 天超时为兜底机制，正常情况下投票结果在票数确定时即终结

use crate::pow_const;

/// 一、机构基础数量
pub const NRC_ADMIN_COUNT: u32 = 19; // 国储会管理员数量
pub const PRC_ADMIN_COUNT: u32 = 9; // 单个省储会管理员数量
pub const PRB_ADMIN_COUNT: u32 = 9; // 单个省储行管理员数量
pub const FRG_PROVINCE_GROUP_ADMIN_COUNT: u32 = 5; // 单个联邦注册局省级组管理员数量
pub const NJD_ADMIN_COUNT: u32 = 15; // 国家司法院创世公职人员数量
pub const PRC_COUNT: u32 = (crate::cid::china::china_cb::CHINA_CB.len() - 1) as u32; // 初始省储会数量（总储会-国储会）
pub const PRB_COUNT: u32 = crate::cid::china::china_ch::CHINA_CH.len() as u32; // 初始省储行数量（来自省储行数组）

/// 二、内部投票（只用于内部投票的阈值）
pub const NRC_INTERNAL_THRESHOLD: u32 = 13; // 国储会内部投票通过阈值
pub const PRC_INTERNAL_THRESHOLD: u32 = 6; // 省储会内部投票通过阈值
pub const PRB_INTERNAL_THRESHOLD: u32 = 6; // 省储行内部投票通过阈值
pub const FRG_INTERNAL_THRESHOLD: u32 = 3; // 联邦注册局省级组内部投票通过阈值
pub const NJD_INTERNAL_THRESHOLD: u32 = 8; // 国家司法院内部投票通过阈值

/// 三、联合投票（只看内部投票结果）
pub const NRC_JOINT_VOTE_WEIGHT: u32 = 19; // 国储会在联合投票中的票数（仅当国储会内部投票通过，通过=19票，未通过=0票）
pub const PRC_JOINT_VOTE_WEIGHT: u32 = 1; // 单个省储会在联合投票中的票数（仅当省储会内部投票通过）
pub const PRB_JOINT_VOTE_WEIGHT: u32 = 1; // 单个省储行在联合投票中的票数（仅当省储行内部投票通过）
pub const JOINT_VOTE_TOTAL: u32 = NRC_JOINT_VOTE_WEIGHT
    + (PRC_COUNT * PRC_JOINT_VOTE_WEIGHT)
    + (PRB_COUNT * PRB_JOINT_VOTE_WEIGHT); // 联合投票总票数
pub const JOINT_VOTE_PASS_THRESHOLD: u32 = 105; // 联合投票通过条件：全票通过则立即执行，非全票通过则进入公民投票流程

/// 四、投票时限（单位：区块数）
pub const VOTING_DURATION_DAYS: u32 = 30; // 投票默认期限30天
pub const BLOCKS_PER_DAY: u32 = pow_const::BLOCKS_PER_DAY as u32; // 每天区块数（统一来源：pow_const）
pub const VOTING_DURATION_BLOCKS: u32 = BLOCKS_PER_DAY as u32 * VOTING_DURATION_DAYS; // 投票默认期限（区块）= 30 * BLOCKS_PER_DAY

/// 五、决议发行常量
pub const RESOLUTION_ISSUANCE_MAX_REASON_LEN: u32 = 1024; // 决议发行理由最大长度
pub const RESOLUTION_ISSUANCE_MAX_ALLOCATIONS: u32 = PRC_COUNT; // 决议发行单次最大分配条目数（与省储会数量一致）

/// 六、立法院模块常量(ADR-027)
/// 宪法不可修改条款清单(公民宪法第十九条)。
/// tier=宪法 的法律,任何立法修改提案命中以下条号一律硬拒,永不可修改。
/// 单一真源,立法院模块 legislation-yuan 与未来客户端校验都引用此常量。
pub const IMMUTABLE_CONSTITUTION_ARTICLES: [u32; 8] = [1, 2, 3, 17, 19, 24, 34, 42];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joint_vote_total_matches_threshold() {
        // 联合投票总票数必须等于通过阈值（全票通过制）。
        assert_eq!(JOINT_VOTE_TOTAL, JOINT_VOTE_PASS_THRESHOLD);
    }
}
