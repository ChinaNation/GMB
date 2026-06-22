use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::DecodeWithMemTracking;
use primitives::institution_code::InstitutionCode;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// CID 机构登记反向索引项：account → (cid_number, account_name)。
///
/// 由 `register_cid_institution` extrinsic 写入,后续创建/查询机构多签时
/// 用作反向校验。
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
pub struct RegisteredInstitution<CidNumber, AccountName> {
    pub cid_number: CidNumber,
    pub account_name: AccountName,
}

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

/// 机构信息。管理员更换账户由主账户派生,机构本身只保存归属与展示信息。
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
    pub cid_full_name: AccountName,
    pub main_account: AccountId,
    pub fee_account: AccountId,
    /// 管理员更换使用的机构码：机构账户只能是公权/私权法人机构码。
    pub institution_code: InstitutionCode,
    pub admins_len: u32,
    pub threshold: u32,
    pub admins: AdminList,
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

/// 关闭机构多签账户提案的业务数据(存入投票引擎 ProposalData)。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct CloseInstitutionAction<AccountId> {
    pub account: AccountId,
    pub beneficiary: AccountId,
    pub proposer: AccountId,
    /// 注销作用域:`SCOPE_INSTITUTION`(关主账户=级联关整个机构)/ `SCOPE_ACCOUNT`(只关该非主账户)。
    pub scope: u8,
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
pub struct CreateInstitutionAction<
    CidNumber,
    AccountName,
    AccountId,
    Balance,
    AdminList,
    AccountList,
> {
    pub cid_number: CidNumber,
    pub cid_full_name: AccountName,
    pub main_account: AccountId,
    pub fee_account: AccountId,
    pub proposer: AccountId,
    /// 创建阶段写入 pending admin account 的机构账户机构码。
    pub institution_code: InstitutionCode,
    pub admins_len: u32,
    pub threshold: u32,
    pub admins: AdminList,
    pub accounts: AccountList,
    pub initial_total: Balance,
    pub fee: Balance,
    pub reserve_total: Balance,
}
