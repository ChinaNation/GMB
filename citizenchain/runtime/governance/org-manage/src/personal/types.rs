//! 个人多签类型定义。
//!
//! 第一类是"账户基本信息"——`DuoqianStatus` / `DuoqianAccount`,
//! 由 lib.rs 内 `DuoqianAccounts` storage map 引用,描述个人多签的账户状态、
//! 管理员列表、阈值、创建者、创建区块。
//!
//! 第二类是"提案业务数据"——`CreateDuoqianAction` / `CloseDuoqianAction`,
//! 在投票引擎 `ProposalData` 里 SCALE 编码存放,投票通过后由
//! `InternalVoteExecutor` 解码后执行业务。
//!
//! 第三类是"创建快照"——`PersonalDuoqianMeta`,投票通过后写入
//! `PersonalDuoqianInfo` 反向索引,记录 creator 与 account_name(地址派生原料)。

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 多签账户状态
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
pub enum DuoqianStatus {
    /// 提案投票中，尚未激活
    Pending,
    /// 已激活（投票通过并入金完成）
    Active,
}

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
pub struct DuoqianAccount<AdminList, AccountId, BlockNumber> {
    pub admin_count: u32,
    pub threshold: u32,
    pub duoqian_admins: AdminList,
    pub creator: AccountId,
    pub created_at: BlockNumber,
    pub status: DuoqianStatus,
}

/// 创建多签账户提案的业务数据（存入投票引擎 ProposalData）
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct CreateDuoqianAction<AccountId, Balance> {
    pub duoqian_address: AccountId,
    pub proposer: AccountId,
    pub admin_count: u32,
    pub threshold: u32,
    pub amount: Balance,
}

/// 关闭多签账户提案的业务数据
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct CloseDuoqianAction<AccountId> {
    pub duoqian_address: AccountId,
    pub beneficiary: AccountId,
    pub proposer: AccountId,
}

/// 个人多签账户元数据（存储在 `PersonalDuoqianInfo` 中）。
///
/// `creator + account_name` 是地址派生公式 `derive_personal_duoqian_address`
/// 的全部业务字段；本结构作为反向索引,用于从地址查回 creator/name。
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
pub struct PersonalDuoqianMeta<AccountId, AccountName> {
    pub creator: AccountId,
    pub account_name: AccountName,
}
