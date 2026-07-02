//! 立法院模块数据类型(ADR-027):法律层级 / 状态 / 表决类型 / 立法动作枚举。
//!
//! 中文注释:这里只放与泛型 `T` 无关的纯枚举;带 `BoundedVec` 上限的法律结构体
//! (Article / Clause / Item / Law / LawVersion)因依赖 `Config` 常量,定义在 `lib.rs` 的 pallet 模块内。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::RuntimeDebug;
use scale_info::TypeInfo;

/// 法律层级。宪法为最高层级,只能由国家立法院按宪法第十九条修改。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum Tier {
    /// 宪法(最高层级)
    Constitution,
    /// 国家级法律(国家立法院)
    National,
    /// 省级法律(省立法院)
    Provincial,
    /// 市级法律(市立法会)
    Municipal,
}

/// 法律状态机。
///
/// 中文注释:`Voting` 阶段由投票引擎的提案状态承载,不在 Law 上重复表达;
/// 旧版本被新版本替代后留在 `LawVersions` 历史里,不单独标 Law 状态。
/// 故 Law 实际可达状态为 Pending(通过待生效)/ Effective(生效)/ Repealed(废止)。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum LawStatus {
    /// 已通过、未到生效时间
    Pending,
    /// 生效中
    Effective,
    /// 已废止
    Repealed,
}

/// 表决类型(公民宪法第45/46条,ADR-027 当前五类立法表决)。
/// 映射为投票引擎接口的 `u8`,保持引擎与业务枚举解耦。教育变体阈值同非教育同级,
/// 仅提案机构(教委会)与表决院路由不同。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum VoteType {
    /// 常规案(>80% 参与,≥60% 赞成)
    Regular,
    /// 常规教育案(教委会起草;阈值同常规案)
    RegularEducation,
    /// 重要案(>90% 参与,≥70% 赞成)
    Major,
    /// 重要教育案(教委会起草;阈值同重要案)
    MajorEducation,
    /// 特别案(全员 ≥70% 赞成 + 强制公民投票),含核心修宪;教育类不适用
    Special,
}

impl VoteType {
    /// 转为投票引擎接口使用的 u8,值与 `votingengine::types::LEG_VOTE_*` 对齐
    /// (常规 0 / 常规教育 1 / 重要 2 / 重要教育 3 / 特别 4)。
    pub fn as_u8(&self) -> u8 {
        match self {
            VoteType::Regular => 0,
            VoteType::RegularEducation => 1,
            VoteType::Major => 2,
            VoteType::MajorEducation => 3,
            VoteType::Special => 4,
        }
    }

    /// 是否教育类(教委会起草、走教委会→参议会 / 市教委会→市立法会 路由)。
    pub fn is_education(&self) -> bool {
        matches!(self, VoteType::RegularEducation | VoteType::MajorEducation)
    }
}

/// 立法动作。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum LawAction {
    /// 立法(新法)
    Enact,
    /// 修法
    Amend,
    /// 废法
    Repeal,
}
