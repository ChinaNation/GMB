//! 机构岗位与管理员任职共用类型。
//!
//! 岗位属于机构实体，管理员账户集合属于 `admins` 模组；两者通过任职记录绑定。
//! 本文件只定义跨 pallet 共用的 SCALE 类型和只读查询接口，不持有任何 storage。

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use frame_support::pallet_prelude::DecodeWithMemTracking;
use primitives::cid::code::InstitutionCode;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 岗位代码最大字节数。岗位代码是授权稳定键，不使用可变岗位名称进行鉴权。
pub const INSTITUTION_ROLE_CODE_MAX_BYTES: u32 = 64;

/// 任职来源引用最大字节数。
pub const ASSIGNMENT_SOURCE_REF_MAX_BYTES: u32 = 128;

/// 机构岗位当前状态。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum InstitutionRoleStatus {
    /// 岗位可接受任职并参与业务授权。
    Active,
    /// 岗位已停用；历史任职仍由链上事件保留。
    Inactive,
}

/// 管理员任职来源。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum InstitutionAssignmentSource {
    /// 创世常量写入。
    Genesis,
    /// 注册局依法登记写入。
    Registry,
    /// 普选最终结果写入。
    PopularElection,
    /// 机构内部互选最终结果写入。
    MutualElection,
    /// 提名任免最终结果写入。
    NominationAppointment,
}

/// 管理员任职当前状态。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum InstitutionAssignmentStatus {
    /// 当前有效任职。
    Active,
    /// 任职已经结束。
    Ended,
}

/// 单个机构的岗位定义。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub struct InstitutionRole<CidNumber, RoleCode, RoleName> {
    /// 所属机构 CID。
    pub cid_number: CidNumber,
    /// 机构内唯一且创建后不可变的岗位代码。
    pub role_code: RoleCode,
    /// 公开岗位名称；只用于展示，不参与授权。
    pub role_name: RoleName,
    /// 是否强制要求非零且有效的开始、结束日期。
    pub term_required: bool,
    /// 岗位当前状态。
    pub role_status: InstitutionRoleStatus,
}

/// 管理员在某机构岗位上的任职事实。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub struct InstitutionAdminAssignment<CidNumber, AccountId, RoleCode, SourceRef> {
    /// 任职机构 CID。
    pub cid_number: CidNumber,
    /// 管理员唯一钱包账户。
    pub admin_account: AccountId,
    /// 所任岗位稳定代码。
    pub role_code: RoleCode,
    /// 任期开始日（自纪元起的天数）；无任期岗位固定为 0。
    pub term_start: u32,
    /// 任期结束日（自纪元起的天数）；无任期岗位固定为 0。
    pub term_end: u32,
    /// 任职来源。
    pub assignment_source: InstitutionAssignmentSource,
    /// 投票、选举、任命或注册结果的追溯引用；创世可为空。
    pub assignment_source_ref: SourceRef,
    /// 任职当前状态。
    pub assignment_status: InstitutionAssignmentStatus,
}

/// 已完成治理流程交给 entity 的机构岗位任职结果。
///
/// 投票引擎只产生结果，不直接写 entity/admins storage。entity 校验机构和岗位后，
/// 把 `admin_accounts` 写成目标岗位的最新有效任职，再从全部有效任职派生机构 `admins`。
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct InstitutionAssignmentResult<AccountId> {
    /// 目标机构码，用于公权/私权路由及固定岗位席位校验。
    pub institution_code: InstitutionCode,
    /// 目标机构主账户，即 admins 模组中机构管理员集合的键。
    pub institution_account: AccountId,
    /// 目标岗位稳定代码。
    pub role_code: Vec<u8>,
    /// 本次结果产生的岗位管理员钱包，按结果顺序写入。
    pub admin_accounts: Vec<AccountId>,
    /// 任期开始日（自纪元起天数）。
    pub term_start: u32,
    /// 任期结束日（自纪元起天数）。
    pub term_end: u32,
    /// 任职制度来源。
    pub assignment_source: InstitutionAssignmentSource,
    /// 投票、选举或任命结果的追溯引用。
    pub assignment_source_ref: Vec<u8>,
}

/// 已完成治理结果写入 entity 的唯一跨模块入口。
pub trait InstitutionAssignmentResultHandler<AccountId> {
    fn apply_institution_assignment_result(
        result: InstitutionAssignmentResult<AccountId>,
    ) -> DispatchResult;
}

impl<AccountId> InstitutionAssignmentResultHandler<AccountId> for () {
    fn apply_institution_assignment_result(
        _result: InstitutionAssignmentResult<AccountId>,
    ) -> DispatchResult {
        Err(sp_runtime::DispatchError::Other(
            "InstitutionAssignmentResultHandlerNotConfigured",
        ))
    }
}

/// 机构岗位与任职只读接口。
///
/// 业务模块必须按“机构 CID + 稳定岗位代码 + 有效任职”鉴权，不能比较岗位名称。
pub trait InstitutionRoleQuery<AccountId> {
    /// 管理员是否在指定机构的指定岗位上有效任职。
    fn is_active_assignment(cid_number: &[u8], admin: &AccountId, role_code: &[u8]) -> bool;

    /// 读取指定机构、指定岗位的全部有效管理员账户。
    fn active_accounts_for_role(cid_number: &[u8], role_code: &[u8]) -> Vec<AccountId>;

    /// 读取某管理员在指定机构的全部有效岗位代码。
    fn active_role_codes(cid_number: &[u8], admin: &AccountId) -> Vec<Vec<u8>>;
}

impl<AccountId> InstitutionRoleQuery<AccountId> for () {
    fn is_active_assignment(_cid_number: &[u8], _admin: &AccountId, _role_code: &[u8]) -> bool {
        false
    }

    fn active_accounts_for_role(_cid_number: &[u8], _role_code: &[u8]) -> Vec<AccountId> {
        Vec::new()
    }

    fn active_role_codes(_cid_number: &[u8], _admin: &AccountId) -> Vec<Vec<u8>> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Node Guard 按枚举声明序解码岗位和任职状态；任一重排都必须先修改节点协议。
    #[test]
    fn status_discriminants_match_governance_skeleton() {
        assert_eq!(
            InstitutionRoleStatus::Active as u8,
            primitives::governance_skeleton::ROLE_STATUS_ACTIVE
        );
        assert_eq!(
            InstitutionAssignmentStatus::Active as u8,
            primitives::governance_skeleton::ASSIGNMENT_STATUS_ACTIVE
        );
    }

    /// 岗位字段声明序就是 `PublicManage::InstitutionRoles` 的链上 SCALE 值格式。
    #[test]
    fn institution_role_field_order_matches_node_guard() {
        let role = InstitutionRole {
            cid_number: b"CID-1".to_vec(),
            role_code: b"ROLE-1".to_vec(),
            role_name: "岗位一".as_bytes().to_vec(),
            term_required: true,
            role_status: InstitutionRoleStatus::Active,
        };
        assert_eq!(
            role.encode(),
            (
                b"CID-1".to_vec(),
                b"ROLE-1".to_vec(),
                "岗位一".as_bytes().to_vec(),
                true,
                InstitutionRoleStatus::Active,
            )
                .encode()
        );
    }

    /// 任职字段声明序就是 `PublicManage::InstitutionRoleAssignments` 元素的链上 SCALE 格式。
    #[test]
    fn institution_assignment_field_order_matches_node_guard() {
        let assignment = InstitutionAdminAssignment {
            cid_number: b"CID-1".to_vec(),
            admin_account: [7u8; 32],
            role_code: b"ROLE-1".to_vec(),
            term_start: 10,
            term_end: 20,
            assignment_source: InstitutionAssignmentSource::PopularElection,
            assignment_source_ref: b"VOTE-1".to_vec(),
            assignment_status: InstitutionAssignmentStatus::Active,
        };
        assert_eq!(
            assignment.encode(),
            (
                b"CID-1".to_vec(),
                [7u8; 32],
                b"ROLE-1".to_vec(),
                10u32,
                20u32,
                InstitutionAssignmentSource::PopularElection,
                b"VOTE-1".to_vec(),
                InstitutionAssignmentStatus::Active,
            )
                .encode()
        );
    }
}
