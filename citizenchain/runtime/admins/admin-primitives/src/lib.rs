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
use sp_runtime::RuntimeDebug;

/// 固定治理公权机构码,唯一真源在 `primitives::cid::code`。
pub use primitives::cid::code::{FRG, NJD};

/// 管理员集合所属机构 CID 号类型。
pub type AdminCidNumber = BoundedVec<u8, ConstU32<CID_NUMBER_MAX_BYTES>>;

/// 管理员姓、名各自的最大字节数；姓名只用于展示，不参与权限判断。
pub const ADMIN_PERSON_NAME_MAX_BYTES: u32 = 128;
/// 无法取得真实姓氏时使用的唯一默认姓。
pub const DEFAULT_ADMIN_FAMILY_NAME: &[u8] = "管理".as_bytes();
/// 无法取得真实名字时使用的唯一默认名。
pub const DEFAULT_ADMIN_GIVEN_NAME: &[u8] = "员".as_bytes();
/// 与链上中国公民字段同名的姓。
pub type FamilyName = BoundedVec<u8, ConstU32<ADMIN_PERSON_NAME_MAX_BYTES>>;
/// 与链上中国公民字段同名的名。
pub type GivenName = BoundedVec<u8, ConstU32<ADMIN_PERSON_NAME_MAX_BYTES>>;

/// 私权机构与个人多签管理员人员记录。
///
/// `admin_account` 是唯一授权字段；`family_name`、`given_name` 与链上中国公民姓名
/// 字段逐字对齐，只承载人员姓名。机构岗位和任职由 entity 独立保存，个人多签也复用
/// 本结构，不再保存纯账户管理员数组。
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
pub struct Admin<AccountId> {
    pub admin_account: AccountId,
    pub family_name: FamilyName,
    pub given_name: GivenName,
}

/// 公权机构管理员人员记录。
///
/// `cid_number` 是对 `citizen-identity` 公民身份真源的引用；当前创世资料不完整时允许
/// 为空。姓名同样允许为空，且不得用展示占位值冒充真实公民资料。授权仍只使用
/// `admin_account`，公民 CID 与姓名都不能直接产生岗位权限。
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
pub struct PublicAdmin<AccountId> {
    pub admin_account: AccountId,
    pub cid_number: AdminCidNumber,
    pub family_name: FamilyName,
    pub given_name: GivenName,
}

impl<AccountId> Admin<AccountId> {
    /// 将缺失的姓、名分别规范化为“管理”“员”；账户始终由调用方强制提供。
    pub fn normalize_names(mut self) -> Self {
        if self.family_name.is_empty() {
            self.family_name = FamilyName::truncate_from(DEFAULT_ADMIN_FAMILY_NAME.to_vec());
        }
        if self.given_name.is_empty() {
            self.given_name = GivenName::truncate_from(DEFAULT_ADMIN_GIVEN_NAME.to_vec());
        }
        self
    }
}

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

/// 个人多签管理员集合生命周期。
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
/// 本结构只保存管理员人员记录及必要路由状态；岗位、任期和任职来源全部归
/// `entity` 的岗位任职存储。机构管理员没有“创建人、创建时间、更新时间”字段。
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
    /// 按钱包账户去重的管理员人员集合。
    pub admins: AdminList,
}

/// 个人多签管理员集合记录。
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
    /// 个人多签没有机构 CID，固定为空。
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

/// 个人多签管理员集合生命周期写入口。
///
/// 个人多签创建、注销等业务 pallet 只能通过此 trait 请求 personal-admins
/// 写入 Pending/Active/Closed，机构管理员不使用本生命周期模型。
pub trait AdminAccountLifecycle<AccountId, AdminItem = AccountId> {
    fn create_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        personal_account: AccountId,
        cid_number: Vec<u8>,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<AdminItem>,
        creator: AccountId,
    ) -> DispatchResult;

    fn activate_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        personal_account: AccountId,
    ) -> DispatchResult;

    fn remove_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        personal_account: AccountId,
    ) -> DispatchResult;

    fn close_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        personal_account: AccountId,
    ) -> DispatchResult;
}

/// 机构管理员集合写入口。
///
/// 机构的来源、岗位和任职全部由 entity 表达，因此该接口不接收 `creator`，也不承担
/// 个人多签的创建语义。公权与私权 entity 只通过本接口原子写入管理员人员集合。
pub trait InstitutionAdminLifecycle<AccountId, AdminItem = Admin<AccountId>> {
    /// 注册局或机构治理写入有效管理员人员集合；机构阈值由 entity 独立保存。
    fn set_institution_admins(
        module_tag: &[u8],
        cid_number: Vec<u8>,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<AdminItem>,
    ) -> DispatchResult;
}

/// 机构管理员集合统一查询口。
///
/// 机构身份只使用 CID；账户地址不能作为本 trait 的查询 key。
pub trait InstitutionAdminQuery<AccountId> {
    fn institution_admins_exist(institution_code: InstitutionCode, cid_number: &[u8]) -> bool;

    fn is_institution_admin(
        institution_code: InstitutionCode,
        cid_number: &[u8],
        who: &AccountId,
    ) -> bool;

    fn institution_admins(
        institution_code: InstitutionCode,
        cid_number: &[u8],
    ) -> Option<Vec<AccountId>>;

    fn institution_admins_len(institution_code: InstitutionCode, cid_number: &[u8]) -> Option<u32>;
}

impl<AccountId> InstitutionAdminQuery<AccountId> for () {
    fn institution_admins_exist(_institution_code: InstitutionCode, _cid_number: &[u8]) -> bool {
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

/// 公权管理员非空公民 CID 与钱包绑定查询。
///
/// 实现只能读取 `citizen-identity` 真源；public-admins 不得自行生成或修正公民 CID。
pub trait CitizenIdentityBindingQuery<AccountId> {
    fn matches_citizen_account(cid_number: &[u8], account: &AccountId) -> bool;
}

impl<AccountId> CitizenIdentityBindingQuery<AccountId> for () {
    fn matches_citizen_account(_cid_number: &[u8], _account: &AccountId) -> bool {
        false
    }
}

/// 个人多签管理员集合查询口。
///
/// 机构管理员已经使用 CID 专用查询；本 trait 只服务个人多签账户。
pub trait AdminAccountQuery<AccountId> {
    fn active_admin_account_exists(
        institution_code: InstitutionCode,
        personal_account: AccountId,
    ) -> bool;

    fn is_active_account_admin(
        institution_code: InstitutionCode,
        personal_account: AccountId,
        who: &AccountId,
    ) -> bool;

    fn active_account_admins(
        institution_code: InstitutionCode,
        personal_account: AccountId,
    ) -> Option<Vec<AccountId>>;

    /// 返回个人多签完整管理员人员记录；授权调用方仍只能比较账户。
    fn active_account_admin_records(
        institution_code: InstitutionCode,
        personal_account: AccountId,
    ) -> Option<Vec<Admin<AccountId>>>;

    fn active_account_admins_len(
        institution_code: InstitutionCode,
        personal_account: AccountId,
    ) -> Option<u32>;

    fn pending_account_exists_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> bool {
        false
    }

    fn is_pending_account_admin_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
        _who: &AccountId,
    ) -> bool {
        false
    }

    fn pending_account_admins_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> Option<Vec<AccountId>> {
        None
    }

    fn pending_account_admin_records_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> Option<Vec<Admin<AccountId>>> {
        None
    }

    fn pending_account_admins_len_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> Option<u32> {
        None
    }
}

impl<AccountId> AdminAccountQuery<AccountId> for () {
    fn active_admin_account_exists(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> bool {
        false
    }

    fn is_active_account_admin(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
        _who: &AccountId,
    ) -> bool {
        false
    }

    fn active_account_admins(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> Option<Vec<AccountId>> {
        None
    }

    fn active_account_admin_records(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> Option<Vec<Admin<AccountId>>> {
        None
    }

    fn active_account_admins_len(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> Option<u32> {
        None
    }

    fn pending_account_exists_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> bool {
        false
    }

    fn is_pending_account_admin_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
        _who: &AccountId,
    ) -> bool {
        false
    }

    fn pending_account_admins_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> Option<Vec<AccountId>> {
        None
    }

    fn pending_account_admin_records_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
    ) -> Option<Vec<Admin<AccountId>>> {
        None
    }

    fn pending_account_admins_len_for_snapshot(
        _institution_code: InstitutionCode,
        _personal_account: AccountId,
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

    /// 私权管理员声明序固定为账户、姓、名，机构值只在前面增加机构码。
    #[test]
    fn institution_admin_account_field_order_matches_node_guard() {
        use codec::Encode;

        let admin = Admin {
            admin_account: 7u8,
            family_name: FamilyName::truncate_from("张".as_bytes().to_vec()),
            given_name: GivenName::truncate_from("三".as_bytes().to_vec()),
        };
        let value = InstitutionAdmins {
            institution_code: *b"NRCG",
            admins: vec![admin.clone()],
        };
        assert_eq!(
            value.encode(),
            (*b"NRCG", vec![(7u8, admin.family_name, admin.given_name)]).encode()
        );
    }

    #[test]
    fn public_admin_field_order_is_account_cid_family_given() {
        use codec::Encode;

        let cid_number = AdminCidNumber::truncate_from(b"GZ000-CTZN6-198805200-2026".to_vec());
        let admin = PublicAdmin {
            admin_account: 7u8,
            cid_number: cid_number.clone(),
            family_name: FamilyName::truncate_from("程".as_bytes().to_vec()),
            given_name: GivenName::truncate_from("伟".as_bytes().to_vec()),
        };
        assert_eq!(
            admin.encode(),
            (
                7u8,
                cid_number,
                admin.family_name.clone(),
                admin.given_name.clone()
            )
                .encode()
        );
    }

    #[test]
    fn missing_person_name_uses_management_default() {
        let admin = Admin {
            admin_account: 1u8,
            family_name: FamilyName::default(),
            given_name: GivenName::default(),
        }
        .normalize_names();
        assert_eq!(admin.family_name.as_slice(), "管理".as_bytes());
        assert_eq!(admin.given_name.as_slice(), "员".as_bytes());
    }
}
