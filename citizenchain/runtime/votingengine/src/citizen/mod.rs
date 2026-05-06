//! 公民投票模式(全新功能,Phase 3 实现)。
//!
//! 与 `joint::jointreferendum`(联合公投)不同:
//! - 联合公投是 yes/no 二元投票 + 全国 SFID 公民快照,只在联合投票兜底时使用
//! - 公民投票是**多候选选举**(election)+ **多签发方公民快照**(SFID / 教育局 / 公司...),
//!   由公权机构(ORG_PUP)和其他注册机构(ORG_OTH)发起
//!
//! 模式与规则:
//! - `BallotRule::Referendum { threshold_percent }` — 二元公投
//! - `BallotRule::PluralityElection` — 简单多数选举(得票最多者胜)
//! - `BallotRule::MajorityElection` — 绝对多数选举(>50% 才能胜)
//! - `BallotRule::Runoff` — 两轮决选
//!
//! Scope(投票范围)由签发方提供:
//! - SFID 系统:行政区(省/市/县)
//! - 教育局系统:辖区教师/学生
//! - 公司系统:股东名册
//! - ...
//!
//! Phase 1 阶段:本目录只是**空骨架**,所有实现都是 stub。
//! Phase 3 才会:
//! 1. 添加候选人/计票/胜出判定逻辑
//! 2. 扩展 `PopulationSnapshotVerifier` 为多签发方版本
//! 3. 接入 votingengine 主流程的 PROPOSAL_KIND_CITIZEN=2

pub mod ballot;
pub mod candidates;
pub mod rules;
pub mod scope;
pub mod tally;
