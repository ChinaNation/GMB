#![cfg_attr(not(feature = "std"), no_std)]
//! 管理员共用原语。
//!
//! 中文注释：本 crate 只放管理员共用类型、trait 与分类策略，不放业务 storage，
//! 也不直接创建任何 pallet。`genesis-admins`、`public-admins`、`private-admins`
//! 和 `personal-admins` 必须在各自模块内维护自己的管理员状态。

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use primitives::cid::code::{
    is_fixed_governance_code, is_private_legal_code, is_public_legal_code, is_unincorporated_code,
    InstitutionCode, NRC, PMUL, PRB, PRC,
};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, RuntimeDebug};

/// 联邦注册局机构码。
pub const FRG: InstitutionCode = *b"FRG\0";

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
    /// 创世管理员：国储会、省储会、省储行、联邦注册局。
    GenesisInstitution,
    /// 非创世公权机构管理员。
    PublicInstitution,
    /// 私权机构管理员。
    ///
    /// 中文注释:非法人不是私权同义词;上层必须按所属法人归属把非法人路由到
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
/// 中文注释：机构账户创建、注销和个人多签创建、注销等业务 pallet 只能通过此 trait
/// 请求管理员模块写入 Pending/Active/Closed，不能直接改各管理员模块 storage。
pub trait AdminAccountLifecycle<AccountId> {
    fn create_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: AccountId,
        institution_code: InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<AccountId>,
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

    /// 特权直设入口:原子写入 Active 管理员账户(创建或更新)并注册动态阈值,**绕过内部投票**。
    ///
    /// 中文注释:仅供 Step3 去中心化鉴权的"联邦注册局给市注册局直接供给/更换管理员"场景。
    /// 调用方负责上层授权校验(who ∈ 联邦注册局 active admins);本 trait 实现方负责:
    /// ① 写 Active `AdminAccount`(账户不存在则创建,存在则更新 admins);
    /// ② 把 `threshold` 同步注册进 votingengine 动态阈值(否则该账户后续内部投票阈值缺失);
    /// ③ 维护任何反向索引。默认实现不支持,只有 public-admins 接入。
    fn set_active_admin_account_direct(
        _module_tag: &[u8],
        _admin_root_account_id: AccountId,
        _institution_code: InstitutionCode,
        _kind: AdminAccountKind,
        _admins: Vec<AccountId>,
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
/// 中文注释：runtime 用一个路由实现把读请求分发到 genesis/public/private/personal
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

/// 判断机构码是否属于创世管理员模块。
pub fn is_genesis_admin_code(code: &InstitutionCode) -> bool {
    is_fixed_governance_code(code) || *code == FRG
}

/// 判断机构码是否属于非创世公权机构管理员模块。
pub fn is_public_admin_code(code: &InstitutionCode) -> bool {
    is_public_legal_code(code) && !is_genesis_admin_code(code)
}

/// 判断机构码是否属于私权法人管理员模块。
pub fn is_private_admin_code(code: &InstitutionCode) -> bool {
    is_private_legal_code(code)
}

/// 判断机构码是否属于非法人机构管理员模块候选。
///
/// 中文注释:非法人可隶属公法人或私法人;机构码本身不能决定管理员模块。
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

/// 创世治理机构的固定管理员人数；联邦注册局不锁固定人数。
pub fn expected_genesis_admins_len(code: InstitutionCode) -> Option<u32> {
    use primitives::count_const::{NRC_ADMIN_COUNT, PRB_ADMIN_COUNT, PRC_ADMIN_COUNT};
    match code {
        NRC => Some(NRC_ADMIN_COUNT),
        PRC => Some(PRC_ADMIN_COUNT),
        PRB => Some(PRB_ADMIN_COUNT),
        _ => None,
    }
}
