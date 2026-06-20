//! 多签治理跨 pallet 共用模块（trait 抽象 + 轻量类型）。
//!
//! 由 personal-manage / organization-manage / duoqian-transfer 共用，与 Pallet 内部状态无关：
//! - 地址校验 / 资金保护 trait 由 runtime Config 注入实现，便于测试 mock 与生产分离；
//! - 多签配置类型仅"裸结构 + 无业务逻辑"，避免业务 pallet 互相反向依赖。
//! 放在 primitives 是为了避免 personal-manage 反向依赖 organization-manage。

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::DecodeWithMemTracking;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

// ===== 一、地址校验 / 资金保护抽象 trait =====

/// 账户地址合法性抽象：用于校验 duoqian_account 是否为本链合法哈希地址。
pub trait DuoqianAccountValidator<AccountId> {
    fn is_valid(address: &AccountId) -> bool;
}

impl<AccountId> DuoqianAccountValidator<AccountId> for () {
    fn is_valid(_address: &AccountId) -> bool {
        true
    }
}

/// 保留地址校验抽象：用于拦截制度保留地址被 duoqian 抢注册。
pub trait DuoqianReservedAccountChecker<AccountId> {
    fn is_reserved(address: &AccountId) -> bool;
}

impl<AccountId> DuoqianReservedAccountChecker<AccountId> for () {
    fn is_reserved(_address: &AccountId) -> bool {
        false
    }
}

/// 转出源地址保护：用于禁止制度保留地址作为资金转出源。
pub trait ProtectedSourceChecker<AccountId> {
    fn is_protected(address: &AccountId) -> bool;
}

impl<AccountId> ProtectedSourceChecker<AccountId> for () {
    fn is_protected(_address: &AccountId) -> bool {
        false
    }
}

// ===== 二、多签账户管理员配置类型 =====

/// 多签账户的管理员配置快照。
/// 由 `PersonalMultisigQuery::lookup_admin_config` 与 `InstitutionMultisigQuery::lookup_admin_config` 返回，
/// duoqian-transfer 在 propose_transfer / propose_safety_fund_transfer / propose_sweep_to_main
/// 等治理流程里据此校验发起人是否在管理员列表内、阈值是否合法。
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
#[scale_info(skip_type_params(MaxAdmins))]
pub struct MultisigConfig<AccountId, MaxAdmins>
where
    MaxAdmins: frame_support::traits::Get<u32>,
{
    pub admins: frame_support::BoundedVec<AccountId, MaxAdmins>,
    pub admin_count: u32,
    pub threshold: u32,
}

/// 不带 BoundedVec 约束的简化版 MultisigConfig，供 trait 接口返回值用。
///
/// 业务 pallet 在 trait 方法中返回此版本（避免把 MaxAdmins 泛型暴露到 trait 边界），
/// duoqian-transfer 拿到后只需读 admins/threshold/admin_count 三个字段做校验。
#[derive(Clone, RuntimeDebug, PartialEq, Eq)]
pub struct MultisigConfigSnapshot<AccountId> {
    pub admins: Vec<AccountId>,
    pub admin_count: u32,
    pub threshold: u32,
}
