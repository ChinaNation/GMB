use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::DecodeWithMemTracking;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 机构及机构账户生命周期。
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
pub enum InstitutionLifecycleStatus {
    /// 创建提案投票中，资金已从创建者账户 reserve。
    Pending,
    /// 投票通过，初始资金已划入机构账户。
    Active,
    /// 机构已注销。当前第1步暂不开放机构整体注销，只预留状态语义。
    Closed,
}

/// 机构级多签信息。管理员和阈值绑定到机构，而不是绑定到单个账户名。
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
pub struct InstitutionInfo<AdminList, AccountId, BlockNumber, AccountName> {
    pub institution_name: AccountName,
    pub main_address: AccountId,
    pub fee_address: AccountId,
    pub admin_count: u32,
    pub threshold: u32,
    pub duoqian_admins: AdminList,
    pub creator: AccountId,
    pub created_at: BlockNumber,
    pub status: InstitutionLifecycleStatus,
    pub account_count: u32,
}

/// 机构下某个账户名对应的链上账户信息。
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
pub struct InstitutionAccountInfo<AccountId, Balance, BlockNumber> {
    pub address: AccountId,
    pub initial_balance: Balance,
    pub status: InstitutionLifecycleStatus,
    pub is_default: bool,
    pub created_at: BlockNumber,
}

/// 创建机构时用户填写的账户初始余额项。
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
pub struct InstitutionInitialAccount<AccountName, Balance> {
    pub account_name: AccountName,
    pub amount: Balance,
}

/// 写入提案业务数据的账户项，保存已经派生好的地址，避免执行阶段重新解释账户名。
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
pub struct CreateInstitutionAccount<AccountName, AccountId, Balance> {
    pub account_name: AccountName,
    pub address: AccountId,
    pub amount: Balance,
    pub is_default: bool,
}

/// 机构创建提案业务数据。
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
#[scale_info(skip_type_params(AdminList, AccountList))]
pub struct CreateInstitutionAction<SfidId, AccountName, AccountId, Balance, AdminList, AccountList>
{
    pub sfid_id: SfidId,
    pub institution_name: AccountName,
    pub main_address: AccountId,
    pub fee_address: AccountId,
    pub proposer: AccountId,
    pub admin_count: u32,
    pub threshold: u32,
    pub duoqian_admins: AdminList,
    pub accounts: AccountList,
    pub initial_total: Balance,
    pub fee: Balance,
    pub reserve_total: Balance,
}
