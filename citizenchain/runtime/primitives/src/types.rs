//! 跨 pallet 共用的轻量数据类型。
//!
//! 仅放"裸结构 + 无业务逻辑"的小类型，避免业务 pallet 互相反向依赖。
//! 当前承载：多签账户管理员配置 `MultisigConfig`，由 personal-manage / organization-manage
//! 通过各自 trait 返回，duoqian-transfer 在转账治理时统一消费。

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::DecodeWithMemTracking;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// 多签账户的管理员配置快照。
///
/// 由 `PersonalMultisigQuery::lookup_admin_config` 与
/// `InstitutionMultisigQuery::lookup_admin_config` 返回，
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
