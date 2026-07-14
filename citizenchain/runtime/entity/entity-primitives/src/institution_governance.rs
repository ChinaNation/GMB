//! 机构治理结果共用协议。
//!
//! 具体业务只负责形成结果；`entity` 负责校验机构、岗位、任职、法定代表人和
//! `admins` 派生不变量。本协议不包含提名、选举、预算等业务规则，也不提供
//! 外部 extrinsic，因此新增或删除业务时不需要改变 entity storage。

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_support::pallet_prelude::DecodeWithMemTracking;
use primitives::cid::code::InstitutionCode;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::{InstitutionAssignmentSource, InstitutionAssignmentStatus, InstitutionRoleStatus};

/// 动态岗位的目标定义。
///
/// `role_code` 是稳定键；已有岗位只能更新公开名称、任期要求和状态，不能换码。
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct InstitutionRoleChange {
    pub role_code: Vec<u8>,
    pub role_name: Vec<u8>,
    pub term_required: bool,
    pub role_status: InstitutionRoleStatus,
}

/// 单条目标任职。
///
/// 每个管理员独立携带任期和来源，避免整体换届接口把存量成员错误改成同一任期。
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct InstitutionAssignmentTarget<AccountId> {
    pub admin_account: AccountId,
    pub term_start: u32,
    pub term_end: u32,
    pub assignment_source: InstitutionAssignmentSource,
    pub assignment_source_ref: Vec<u8>,
    pub assignment_status: InstitutionAssignmentStatus,
}

/// 一个岗位的完整目标任职集合。
///
/// 未出现在结果中的岗位保持不变；出现在结果中的岗位按本集合整体替换。动态岗位
/// 可以提交空集合表示暂时空缺，固定创世岗位仍必须满足协议席位数。
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct InstitutionRoleAssignmentChange<AccountId> {
    pub role_code: Vec<u8>,
    pub assignments: Vec<InstitutionAssignmentTarget<AccountId>>,
}

/// 法定代表人公开信息目标值。
///
/// 三个字段只能整体设置；没有“只改姓名/CID/账户”或使用管理员首位回退的路径。
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct InstitutionLegalRepresentativeChange<AccountId> {
    pub legal_representative_name: Vec<u8>,
    pub legal_representative_cid_number: Vec<u8>,
    pub legal_representative_account: AccountId,
}

/// 业务模块交给 entity 的机构治理最终结果。
///
/// 一个结果可以同时调整多个岗位、多个岗位任职及法定代表人；entity 必须在同一
/// storage transaction 内完成写入和 admins 派生，任一步失败则全部回滚。
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct InstitutionGovernanceResult<AccountId> {
    pub institution_code: InstitutionCode,
    pub institution_account: AccountId,
    pub role_changes: Vec<InstitutionRoleChange>,
    pub assignment_changes: Vec<InstitutionRoleAssignmentChange<AccountId>>,
    pub legal_representative_change: Option<InstitutionLegalRepresentativeChange<AccountId>>,
    /// 指向产生本结果的登记、选举、投票或其他业务记录；不存在 `creator` 字段。
    pub result_source_ref: Vec<u8>,
}

/// 已完成业务结果写入 entity 的唯一跨模块入口。
pub trait InstitutionGovernanceResultHandler<AccountId> {
    fn apply_institution_governance_result(
        result: InstitutionGovernanceResult<AccountId>,
    ) -> DispatchResult;
}

impl<AccountId> InstitutionGovernanceResultHandler<AccountId> for () {
    fn apply_institution_governance_result(
        _result: InstitutionGovernanceResult<AccountId>,
    ) -> DispatchResult {
        Err(sp_runtime::DispatchError::Other(
            "InstitutionGovernanceResultHandlerNotConfigured",
        ))
    }
}
