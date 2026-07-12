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
    InstitutionCode, NRC, PMUL, PRB, PRC,
};
use primitives::core_const::CID_NUMBER_MAX_BYTES;
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, RuntimeDebug};

/// 固定治理公权机构码,唯一真源在 `primitives::cid::code`。
pub use primitives::cid::code::{FRG, NJD};

/// 管理员资料里姓名/职务的最大字节长度(与实体生命周期模块 `MaxAccountNameLength` 一致)。
pub const ADMIN_NAME_MAX_BYTES: u32 = 128;

/// 管理员岗位代码最大字节长度。
pub const ADMIN_ROLE_CODE_MAX_BYTES: u32 = 64;

/// 管理员任职来源追溯 ID 最大字节长度。
pub const ADMIN_SOURCE_REF_MAX_BYTES: u32 = 128;

/// 护宪大法官职务字面量。护宪成员解析只认本常量,禁止各处手写字符串。
/// 单源 = [`primitives::governance_skeleton::ROLE_CONSTITUTION_GUARD`](与节点骨架守卫 I6、
/// 创世 role-by-index 逐字节共用),消除多份字面量。
pub use primitives::governance_skeleton::ROLE_CONSTITUTION_GUARD as ADMIN_ROLE_CONSTITUTION_GUARD;
/// 首席大法官职务字面量。
pub const ADMIN_ROLE_CHIEF_JUSTICE: &[u8] = "首席大法官".as_bytes();
/// 次席大法官职务字面量。
pub const ADMIN_ROLE_DEPUTY_CHIEF_JUSTICE: &[u8] = "次席大法官".as_bytes();
/// 大法官职务字面量。
pub const ADMIN_ROLE_JUSTICE: &[u8] = "大法官".as_bytes();

/// 管理员资料里实名 CID 号最大字节长度(与全仓 `CID_NUMBER_MAX_BYTES` 一致)。
pub const ADMIN_CID_NUMBER_MAX_BYTES: u32 = CID_NUMBER_MAX_BYTES;

/// 管理员集合所属机构 CID 号类型。
pub type AdminCidNumber = BoundedVec<u8, ConstU32<CID_NUMBER_MAX_BYTES>>;

/// 管理员任职事实的来源。
///
/// 佐证 `AdminProfile` 的岗位/任期/姓名由哪条治理路径产生;供 CitizenApp 展示。
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
pub enum AdminSource {
    /// 创世写入。
    Genesis,
    /// 注册局录入。
    Registry,
    /// 内部投票产生。
    InternalVote,
    /// 机构内部互选产生。
    MutualElection,
    /// 普选产生。
    PopularElection,
    /// 提名任免产生。
    NominationAppointment,
}

/// 单个机构管理员的链上公开资料。
///
/// `admin_account` 是密码学账户(投票/多签资格本身);`admin_cid_number`
/// 是注册局签发、与真人一一绑定的实名锚。岗位制度归 entity 模块,
/// 本结构只记录某个具体管理员正在担任哪个岗位以及这次任职事实的来源。
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
pub struct AdminProfile<AccountId> {
    /// 管理员密码学账户,∈ 机构管理员集合。
    pub admin_account: AccountId,
    /// 管理员实名锚:注册局签发的 CID 号。
    pub admin_cid_number: BoundedVec<u8, ConstU32<ADMIN_CID_NUMBER_MAX_BYTES>>,
    /// 姓名快照,来自注册局-公民列表。
    pub admin_name: BoundedVec<u8, ConstU32<ADMIN_NAME_MAX_BYTES>>,
    /// 岗位代码,引用 entity 模块中该机构的岗位定义。
    pub role_code: BoundedVec<u8, ConstU32<ADMIN_ROLE_CODE_MAX_BYTES>>,
    /// 岗位名称快照,用于跨端展示和历史留痕。
    pub role_name: BoundedVec<u8, ConstU32<ADMIN_NAME_MAX_BYTES>>,
    /// 任期开始(天数自纪元;无任期填 0)。
    pub term_start: u32,
    /// 任期结束(天数自纪元;无任期填 0)。
    pub term_end: u32,
    /// 本次任职事实的来源。
    pub admin_source: AdminSource,
    /// 来源追溯 ID:注册局操作、投票提案、选举或提名任免记录。
    pub admin_source_ref: BoundedVec<u8, ConstU32<ADMIN_SOURCE_REF_MAX_BYTES>>,
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
    /// 管理员集合所属根账户。机构为主账户，个人多签为个人多签账户本身。
    pub admin_root_account_id: AccountId,
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

/// 管理员集合统一查询口。
///
/// runtime 用一个路由实现把读请求分发到 public/private/personal
/// 各自 pallet；业务模块只依赖本 trait，不直接依赖某个具体管理员 storage。
pub trait AdminAccountQuery<AccountId> {
    /// 是否为创世封存机构账户。非创世模块默认返回 false。
    fn is_genesis_protected(_account: &AccountId) -> bool {
        false
    }

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

    /// 读取活跃机构管理员的完整公开资料(姓名/职务/任期/实名 CID)。
    ///
    /// 仅公权/私权机构管理员模块返回资料;个人多签与默认实现返回 None。
    /// 投票/多签资格判定仍用 `active_account_admins`(只取账户),本方法专供展示。
    fn active_account_admin_profiles(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> Option<Vec<AdminProfile<AccountId>>> {
        None
    }

    fn active_account_admins_len(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<u32>;

    fn pending_account_exists_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> bool;

    fn is_pending_account_admin_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
        who: &AccountId,
    ) -> bool;

    fn pending_account_admins_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<Vec<AccountId>>;

    fn pending_account_admins_len_for_snapshot(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<u32>;

    fn legal_representative(
        institution_code: InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<AccountId>;
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

    fn legal_representative(
        _institution_code: InstitutionCode,
        _admin_root_account_id: AccountId,
    ) -> Option<AccountId> {
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

/// 固定治理公权机构的固定管理员人数。
///
/// FRG 的固定人数语义是"单个省行政区组 5 人",不是全局 215 人平铺账户。
pub fn expected_fixed_governance_admins_len(code: InstitutionCode) -> Option<u32> {
    use primitives::count_const::{
        FRG_PROVINCE_GROUP_ADMIN_COUNT, NJD_ADMIN_COUNT, NRC_ADMIN_COUNT, PRB_ADMIN_COUNT,
        PRC_ADMIN_COUNT,
    };
    match code {
        NRC => Some(NRC_ADMIN_COUNT),
        PRC => Some(PRC_ADMIN_COUNT),
        PRB => Some(PRB_ADMIN_COUNT),
        FRG => Some(FRG_PROVINCE_GROUP_ADMIN_COUNT),
        NJD => Some(NJD_ADMIN_COUNT),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 节点骨架守卫按声明序解码 `kind`/`status`(单字节判别值),此处与
    /// `primitives::governance_skeleton` 的共享常量交叉钉死:任一枚举重排即测试红。
    #[test]
    fn scale_discriminants_match_governance_skeleton() {
        assert_eq!(
            AdminAccountKind::PublicInstitution as u8,
            primitives::governance_skeleton::KIND_PUBLIC_INSTITUTION
        );
        assert_eq!(
            AdminAccountStatus::Active as u8,
            primitives::governance_skeleton::STATUS_ACTIVE
        );
    }

    /// 护宪职务字面量单源(re-export 自 primitives)。
    #[test]
    fn constitution_guard_role_is_single_sourced() {
        assert_eq!(
            ADMIN_ROLE_CONSTITUTION_GUARD,
            primitives::governance_skeleton::ROLE_CONSTITUTION_GUARD
        );
    }

    /// `AdminAccount` 的声明序就是 PublicAdmins 链上值格式，NodeGuard 按这一顺序完整解码。
    #[test]
    fn admin_account_field_order_matches_node_guard() {
        use codec::Encode;

        let value = AdminAccount {
            cid_number: b"NRC-CID".to_vec().try_into().expect("cid"),
            institution_code: *b"NRCG",
            kind: AdminAccountKind::PublicInstitution,
            admins: vec![1u8, 2u8],
            creator: [3u8; 32],
            created_at: 4u32,
            updated_at: 5u32,
            status: AdminAccountStatus::Active,
        };
        assert_eq!(
            value.encode(),
            (
                b"NRC-CID".to_vec(),
                *b"NRCG",
                AdminAccountKind::PublicInstitution,
                vec![1u8, 2u8],
                [3u8; 32],
                4u32,
                5u32,
                AdminAccountStatus::Active,
            )
                .encode()
        );
    }
}
