use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::DecodeWithMemTracking;
use primitives::cid::code::InstitutionCode;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// CID 机构登记反向索引项：account → (cid_number, account_name)。
///
/// 由 `register_cid_private_institution` extrinsic 写入,后续创建/查询私权机构多签时
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
    /// 投票型生命周期处理中。机构注册创建不使用该状态。
    Pending,
    /// 机构已上链激活，初始资金已划入机构账户。
    Active,
    /// 机构已注销。当前第1步暂不开放机构整体注销，只预留状态语义。
    Closed,
}

/// 机构信息(链上最小集)。
///
/// 链上只保存全国可见的机构身份事实:`cid_number` 作 storage key 已编码省/市/机构码/法人/盈利;
/// 镇归属使用统一字段 `town_code`;当前私权机构注册先写空值,保留同形态解码。
/// 主账户/费用账户由 `(cid_number, 保留名)` 派生且常驻 `InstitutionAccounts`,故不在此重复存;
/// 管理员集合与动态阈值的长期真源在 admins 模块与 internal-vote,亦不在此存快照。
/// 公权/私权机构名称均以上链字段为准;OnChina 只保留查询缓存。
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
pub struct InstitutionInfo<BlockNumber, AccountName> {
    /// 机构全称。
    pub cid_full_name: AccountName,
    /// 机构简称。
    pub cid_short_name: AccountName,
    /// 所属镇代码。当前私权机构写空值;字段保持与 public-manage 同形态。
    pub town_code: AccountName,
    /// 管理员更换/路由使用的机构码:机构账户只能是公权/私权法人机构码。
    pub institution_code: InstitutionCode,
    /// 机构注册创建区块号。
    pub created_at: BlockNumber,
    /// 机构生命周期状态。
    pub status: InstitutionLifecycleStatus,
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

/// 关闭私权机构多签账户提案的业务数据(存入投票引擎 ProposalData)。
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

/// 机构注册交易的账户项，保存已经派生好的地址，避免重复解释账户名。
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
