//! election-vote 本地类型。
//!
//! 本文件只保存“本次选举的运行态快照”。总统、院长、任期、
//! 候选来源等业务规则由发起业务模块或未来选举法模块解释后传入，
//! election-vote 不把这些规则写成常量真源。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use votingengine::types::CidNumber;

/// 选举模式：普选或机构内部互选。
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum ElectionMode {
    /// 公民按行政区/机构范围投票。
    Popular,
    /// 机构现任成员/管理员在成员快照内投票。
    Mutual,
}

impl ElectionMode {
    pub const fn stage(self) -> u8 {
        match self {
            ElectionMode::Popular => votingengine::STAGE_ELECTION_POPULAR,
            ElectionMode::Mutual => votingengine::STAGE_ELECTION_MUTUAL,
        }
    }
}

/// 创建选举时固化的职位快照。
///
/// `office_code` 是业务模块给出的职位编码，例如总统、参议员、
/// 院长等；本 pallet 只保存快照，不解释职位规则。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ElectionMeta<OfficeCode> {
    pub mode: ElectionMode,
    /// 普选固定人口作用域；互选为 None，资格来自 VotePlan 岗位任职快照。
    pub population_scope: Option<votingengine::PopulationScope>,
    /// 发起机构 CID；机构码只允许从该 CID 解析，不在载荷或存储中保留第二身份真源。
    pub actor_cid_number: CidNumber,
    /// 任职目标机构 CID。
    pub target_cid_number: CidNumber,
    pub office_code: OfficeCode,
    pub rule_id: u32,
    pub seat_count: u16,
    /// 任期开始日（自纪元起天数），与 entity 任职字段单位一致。
    pub term_start: u32,
    /// 任期结束日（自纪元起天数），与 entity 任职字段单位一致。
    pub term_end: u32,
}

/// 选举计票汇总。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ElectionTally {
    pub casted: u32,
}

/// 当选结果项。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ElectionWinner<AccountId> {
    pub account: AccountId,
    pub votes: u32,
    pub seat_index: u16,
}
