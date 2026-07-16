#![cfg_attr(not(feature = "std"), no_std)]
//! 管理员共用原语。
//!
//! 本 crate 只放管理员共用类型、trait 与分类策略，不放业务 storage，
//! 也不直接创建任何 pallet。`public-admins`、`private-admins` 和
//! `personal-admins` 必须在各自模块内维护自己的管理员状态。

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{dispatch::DispatchResult, traits::ConstU32, BoundedVec};
use primitives::cid::code::{
    is_fixed_governance_code, is_private_legal_code, is_public_legal_code, is_unincorporated_code,
    InstitutionCode, PMUL,
};
use primitives::core_const::CID_NUMBER_MAX_BYTES;
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, RuntimeDebug};

/// 固定治理公权机构码,唯一真源在 `primitives::cid::code`。
pub use primitives::cid::code::{FRG, NJD};

/// 管理员集合所属机构 CID 号类型。
pub type AdminCidNumber = BoundedVec<u8, ConstU32<CID_NUMBER_MAX_BYTES>>;

/// 管理员集合所属类型。
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
pub enum AdminAccountKind {
    /// 公权机构管理员；固定治理机构也是公权机构，只是创世写入并采用固定阈值。
    PublicInstitution,
    /// 私权机构管理员。
    ///
    /// 非法人不是私权同义词;上层必须按所属法人归属把非法人路由到
    /// public-admins 或 private-admins。
    PrivateInstitution,
    /// 个人多签管理员。
    PersonalMultisig,
}

/// 管理员集合生命周期。
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
pub enum AdminAccountStatus {
    /// 创建提案投票中；投票引擎可读取管理员快照。
    Pending,
    /// 已激活，可发起常规治理、转账或管理员更换。
    Active,
    /// 已关闭，管理员集合不再有效。
    Closed,
}

/// 机构管理员集合。
///
/// 本结构只保存机构管理员钱包账户及必要路由状态；姓名、CID、岗位、任期和来源
/// 全部归 `entity` 的岗位任职存储。机构管理员没有“创建人、创建时间、更新时间”字段。
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
#[scale_info(skip_type_params(AdminList))]
pub struct InstitutionAdmins<AdminList> {
    /// 机构码，用于把查询路由到对应公权或私权业务。
    pub institution_code: InstitutionCode,
    /// 去重后的管理员钱包账户集合。
    pub admins: AdminList,
}

/// 管理员集合记录。
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
#[scale_info(skip_type_params(AdminList))]
pub struct AdminAccount<AdminList, AccountId, BlockNumber> {
    /// 管理员集合所属机构 CID 号;个人多签没有机构 CID 时为空。
    pub cid_number: AdminCidNumber,
    pub institution_code: InstitutionCode,
    pub kind: AdminAccountKind,
    pub admins: AdminList,
    pub creator: AccountId,
    pub created_at: BlockNumber,
    pub updated_at: BlockNumber,
    pub status: AdminAccountStatus,
}

/// 管理员集合变更提案业务数据。
#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(AccountId, AdminList))]
pub struct AdminSetChangeAction<AccountId, AdminList> {
    /// 个人多签账户。机构管理员变更统一按 CID，不使用本类型。
    pub personal_account: AccountId,
    /// 提案通过后写入的完整管理员集合。
    pub admins: AdminList,
    /// 提案通过后写入投票引擎的动态阈值；固定治理机构必须等于制度固定阈值。
    pub new_threshold: u32,
}

/// 管理员集合生命周期写入口。
///
/// 机构账户创建、注销和个人多签创建、注销等业务 pallet 只能通过此 trait
/// 请求管理员模块写入 Pending/Active/Closed，不能直接改各管理员模块 storage。
pub trait AdminAccountLifecycle<AccountId, AdminItem = AccountId> {
    fn create_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: AccountId,
        cid_number: Vec<u8>,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<AdminItem>,
        creator: AccountId,
    ) -> DispatchResult;

    fn activate_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: AccountId,
    ) -> DispatchResult;

    fn remove_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: AccountId,
    ) -> DispatchResult;

    fn close_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: AccountId,
    ) -> DispatchResult;

    /// 注册局直设入口:原子写入 Active 管理员账户(创建或更新)并注册动态阈值,**绕过内部投票**。
    ///
    /// 仅供注册局注册机构时同步写入目标机构管理员集合;不是机构自改管理员的治理入口。
    /// 调用方负责上层注册局授权校验;本 trait 实现方负责:
    /// ① 写 Active `AdminAccount`(账户不存在则创建,存在则更新 admins);
    /// ② 把 `threshold` 同步注册进 votingengine 动态阈值(否则该账户后续内部投票阈值缺失);
    /// ③ 维护任何反向索引。默认实现不支持,public-admins/private-admins 按机构类型接入。
    fn set_active_admin_account_direct(
        _module_tag: &[u8],
        _admin_root_account_id: AccountId,
        _cid_number: Vec<u8>,
        _institution_code: InstitutionCode,
        _kind: AdminAccountKind,
        _admins: Vec<AdminItem>,
        _threshold: u32,
        _creator: AccountId,
    ) -> DispatchResult {
        Err(DispatchError::Other(
            "SetActiveAdminAccountDirectNotSupported",
        ))
    }
}

/// 机构管理员集合写入口。
///
/// 机构的来源、岗位和任职全部由 entity 表达，因此该接口不接收 `creator`，也不承担
/// 个人多签的创建语义。公权与私权 entity 只通过本接口原子写入纯钱包账户集合。
pub trait InstitutionAdminLifecycle<AccountId> {
    /// 注册局直设机构的有效管理员账户，并同步登记动态投票阈值。
    fn set_institution_admins(
        module_tag: &[u8],
        cid_number: Vec<u8>,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<AccountId>,
        threshold: u32,
    ) -> DispatchResult;

    /// entity 任职结果生效后同步机构管理员钱包集合。
    ///
    /// 调用方不传阈值：固定机构继续使用编译期固定阈值，动态机构继续使用当前 Active
    /// 动态阈值，避免岗位任职结果越权修改投票制度。
    fn sync_institution_admins_from_assignments(
        module_tag: &[u8],
        cid_number: Vec<u8>,
        institution_code: InstitutionCode,
        admins: Vec<AccountId>,
    ) -> DispatchResult;
}

/// 机构管理员集合统一查询口。
///
/// 机构身份只使用 CID；账户地址不能作为本 trait 的查询 key。
pub trait InstitutionAdminQuery<AccountId> {
    fn institution_admins_exist(
        institution_code: InstitutionCode,
        cid_number: &[u8],
    ) -> bool;

    fn is_institution_admin(
        institution_code: InstitutionCode,
        cid_number: &[u8],
        who: &AccountId,
    ) -> bool;

    fn institution_admins(
        institution_code: InstitutionCode,
        cid_number: &[u8],
    ) -> Option<Vec<AccountId>>;

    fn institution_admins_len(
        institution_code: InstitutionCode,
        cid_number: &[u8],
    ) -> Option<u32>;
}

impl<AccountId> InstitutionAdminQuery<AccountId> for () {
    fn institution_admins_exist(
        _institution_code: InstitutionCode,
        _cid_number: &[u8],
    ) -> bool {
        false
    }

    fn is_institution_admin(
        _institution_code: InstitutionCode,
        _cid_number: &[u8],
        _who: &AccountId,
    ) -> bool {
        false
    }

    fn institution_admins(
        _institution_code: InstitutionCode,
        _cid_number: &[u8],
    ) -> Option<Vec<AccountId>> {
        None
    }

    fn institution_admins_len(
        _institution_code: InstitutionCode,
        _cid_number: &[u8],
    ) -> Option<u32> {
        None
    }
}

/// 个人多签管理员集合查询口。
///
/// runtime 用一个路由实现把读请求分发到 public/private/personal
/// 各自 pallet；业务模块只依赖本 trait，不直接依赖某个具体管理员 storage。
pub trait AdminAccountQuery<AccountId> {
    fn active_admin_account_exists(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> bool;

    fn is_active_account_admin(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
        who: &AccountId,
    ) -> bool;

    fn active_account_admins(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<Vec<AccountId>>;

    fn active_account_admins_len(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<u32>;

    fn pending_account_exists_for_snapshot(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> bool {
        false
    }

    fn is_pending_account_admin_for_snapshot(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
        _who: &AccountId,
    ) -> bool {
        false
    }

    fn pending_account_admins_for_snapshot(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> Option<Vec<AccountId>> {
        None
    }

    fn pending_account_admins_len_for_snapshot(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> Option<u32> {
        None
    }
}

impl<AccountId> AdminAccountQuery<AccountId> for () {
    fn active_admin_account_exists(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> bool {
        false
    }

    fn is_active_account_admin(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
        _who: &AccountId,
    ) -> bool {
        false
    }

    fn active_account_admins(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> Option<Vec<AccountId>> {
        None
    }

    fn active_account_admins_len(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> Option<u32> {
        None
    }

    fn pending_account_exists_for_snapshot(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> bool {
        false
    }

    fn is_pending_account_admin_for_snapshot(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
        _who: &AccountId,
    ) -> bool {
        false
    }

    fn pending_account_admins_for_snapshot(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> Option<Vec<AccountId>> {
        None
    }

    fn pending_account_admins_len_for_snapshot(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> Option<u32> {
        None
    }
}

/// 判断机构码是否属于公权机构管理员模块。
pub fn is_public_admin_code(code: &InstitutionCode) -> bool {
    is_public_legal_code(code) || is_fixed_governance_code(code)
}

/// 判断机构码是否属于私权法人管理员模块。
pub fn is_private_admin_code(code: &InstitutionCode) -> bool {
    is_private_legal_code(code)
}

/// 判断机构码是否属于非法人机构管理员模块候选。
///
/// 非法人可隶属公法人或私法人;机构码本身不能决定管理员模块。
pub fn is_unincorporated_admin_code(code: &InstitutionCode) -> bool {
    is_unincorporated_code(code)
}

/// 判断机构码能否由公权管理员模块保存。
pub fn can_store_public_admin_code(code: &InstitutionCode) -> bool {
    is_public_admin_code(code) || is_unincorporated_admin_code(code)
}

/// 判断机构码能否由私权管理员模块保存。
pub fn can_store_private_admin_code(code: &InstitutionCode) -> bool {
    is_private_admin_code(code) || is_unincorporated_admin_code(code)
}

/// 判断机构码是否属于个人多签管理员模块。
pub fn is_personal_admin_code(code: &InstitutionCode) -> bool {
    *code == PMUL
}

/// 固定治理机构管理员人数；必须完整匹配机构码和 CID。
///
/// FRG 在 `admins` 中保存联邦注册局全部 215 名管理员；43 个省级 5 人岗位组
/// 由 `entity` 任职关系表达，不再维护第二套管理员分组 storage。
pub fn expected_fixed_governance_admins_len(
    code: InstitutionCode,
    cid_number: &[u8],
) -> Option<u32> {
    primitives::governance_skeleton::fixed_institution_by_identity(code, cid_number)
        .map(|institution| institution.expected_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 节点骨架守卫按声明序解码机构管理员 `status`，与协议清单交叉钉死。
    /// 机构管理员新布局没有 `kind`；个人多签的独立 `AdminAccountKind` 不属于该镜像。
    #[test]
    fn institution_admin_status_discriminant_matches_governance_skeleton() {
        assert_eq!(
            AdminAccountStatus::Active as u8,
            primitives::governance_skeleton::STATUS_ACTIVE
        );
    }

    /// `InstitutionAdminAccount` 的声明序就是机构 admins 链上值格式。
    #[test]
    fn institution_admin_account_field_order_matches_node_guard() {
        use codec::Encode;

        let value = InstitutionAdminAccount {
            cid_number: b"NRC-CID".to_vec().try_into().expect("cid"),
            institution_code: *b"NRCG",
            admins: vec![1u8, 2u8],
            status: AdminAccountStatus::Active,
        };
        assert_eq!(
            value.encode(),
            (
                b"NRC-CID".to_vec(),
                *b"NRCG",
                vec![1u8, 2u8],
                AdminAccountStatus::Active,
            )
                .encode()
        );
    }
}
