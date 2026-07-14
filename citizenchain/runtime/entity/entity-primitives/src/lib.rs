#![cfg_attr(not(feature = "std"), no_std)]

//! 实体生命周期共用类型与 trait。
//!
//! 本 crate 是 `runtime/entity` 下唯一共享模块，集中机构生命周期共用类型
//! (`RegisteredInstitution`/`InstitutionInfo`/`InstitutionLifecycleStatus` 等,
//! 由 public-manage / private-manage re-export)与统一查询 trait,本 crate 不持有 storage。
//! 公权机构、私权机构、个人多签分别在各自 pallet 保存生命周期状态；
//! 下游模块通过这里的 trait 做统一查询，不直接读取某个实体 pallet 的 storage。

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::DecodeWithMemTracking;
use primitives::cid::code::InstitutionCode;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// 机构与个人多签共用的账户合法性、保留地址、保护地址检查 trait
// 仍以 primitives::multisig 为唯一真源，entity-primitives 只做实体侧统一出口。
pub use primitives::multisig::{AccountValidator, ProtectedSourceChecker, ReservedAccountGuard};

pub mod institution_governance;
pub mod institution_role;
pub use institution_governance::{
    InstitutionAssignmentTarget, InstitutionGovernanceResult, InstitutionGovernanceResultHandler,
    InstitutionLegalRepresentativeChange, InstitutionRoleAssignmentChange, InstitutionRoleChange,
};
pub use institution_role::{
    InstitutionAdminAssignment, InstitutionAssignmentSource, InstitutionAssignmentStatus,
    InstitutionRole, InstitutionRoleQuery, InstitutionRoleStatus, ASSIGNMENT_SOURCE_REF_MAX_BYTES,
    INSTITUTION_ROLE_CODE_MAX_BYTES,
};

// ===== 机构生命周期共用 storage 值类型(唯一真源) =====
// public-manage / private-manage 逐字段一致地复用以下类型;两 pallet 各自
// `pub use entity_primitives::{...}` 出口,保持既有对外 API(`public_manage::InstitutionInfo` 等)不变。
// 均为 SCALE 存储值类型:字段顺序、derive 集合、枚举判别值必须与既有链上编码一致。

/// CID 机构登记反向索引项：account → (cid_number, account_name)。
///
/// 由 `register_cid_public_institution` / `register_cid_private_institution` extrinsic 写入,
/// 后续创建/查询机构多签时用作反向校验。
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
/// 镇归属使用统一字段 `town_code`:镇行政区公权机构由注册局创建时写入,当前私权机构写空值;
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
pub struct InstitutionInfo<BlockNumber, AccountName, CidNumber, AccountId> {
    /// 机构全称。
    pub cid_full_name: AccountName,
    /// 机构简称。
    pub cid_short_name: AccountName,
    /// 所属镇代码。非镇行政区机构与当前私权机构写空值;镇行政区公权机构由注册局创建时写入。
    pub town_code: AccountName,
    /// 法定代表人公开姓名。创世没有真实任免资料时为 None。
    pub legal_representative_name: Option<AccountName>,
    /// 法定代表人唯一公民 CID。必须与姓名、账户同时存在或同时为空。
    pub legal_representative_cid_number: Option<CidNumber>,
    /// 法定代表人唯一钱包账户。不得从机构管理员首位账户回退生成。
    pub legal_representative_account: Option<AccountId>,
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

/// 关闭机构多签账户提案的业务数据(公权/私权通用,存入投票引擎 ProposalData)。
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

/// runtime 内实体生命周期分类。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityKind {
    /// 公权机构生命周期，由 `public-manage` 承载。
    PublicInstitution,
    /// 私权机构生命周期，由 `private-manage` 承载。
    PrivateInstitution,
    /// 个人多签生命周期，由 `personal-manage` 承载。
    PersonalMultisig,
}

/// 机构多签账户查询 trait，供交易、清算、验签等下游模块使用。
///
/// 输入任意机构账户地址，返回该账户所属机构 CID、机构码和管理员快照。
/// 公权/私权 pallet 各自实现本 trait，runtime 再提供聚合查询适配器。
pub trait InstitutionMultisigQuery<AccountId> {
    /// 返回机构账户所属唯一 CID。个人多签没有 CID,不得返回伪 CID。
    fn lookup_cid(addr: &AccountId) -> Option<Vec<u8>>;

    /// 返回机构账户所属机构码。
    fn lookup_org(addr: &AccountId) -> Option<InstitutionCode>;

    /// 返回机构账户当前管理员快照。
    fn lookup_admin_config(
        addr: &AccountId,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId>>;

    /// 返回机构账户生命周期是否为 Active。
    fn is_active(addr: &AccountId) -> bool;
}

/// 机构 CID 是否已在某个实体生命周期 pallet 中登记。
///
/// 用于 public/private 两个机构生命周期模块互相查询，防止同一 CID
/// 在两个模块中重复登记。本 trait 不持有 storage，不形成第二个共享真源。
pub trait InstitutionCidQuery<CidNumber> {
    /// CID 是否已存在。
    fn cid_exists(cid_number: &CidNumber) -> bool;
}

/// 机构法定代表人查询接口。
///
/// 法定代表人是机构公开信息，唯一真源位于 entity 的 `InstitutionInfo`；
/// admins 模块不得保存副本，也不得以首位管理员作为回退值。
pub trait InstitutionLegalRepresentativeQuery<AccountId> {
    /// 按机构码和任一机构账户读取当前已任命的法定代表人钱包账户。
    fn legal_representative(
        institution_code: InstitutionCode,
        institution: AccountId,
    ) -> Option<AccountId>;
}

impl<CidNumber> InstitutionCidQuery<CidNumber> for () {
    fn cid_exists(_cid_number: &CidNumber) -> bool {
        false
    }
}

impl<AccountId> InstitutionLegalRepresentativeQuery<AccountId> for () {
    fn legal_representative(
        _institution_code: InstitutionCode,
        _institution: AccountId,
    ) -> Option<AccountId> {
        None
    }
}

impl<AccountId> InstitutionMultisigQuery<AccountId> for () {
    fn lookup_cid(_addr: &AccountId) -> Option<Vec<u8>> {
        None
    }

    fn lookup_org(_addr: &AccountId) -> Option<InstitutionCode> {
        None
    }

    fn lookup_admin_config(
        _addr: &AccountId,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId>> {
        None
    }

    fn is_active(_addr: &AccountId) -> bool {
        false
    }
}

/// CID 机构登记与注销验签抽象。
///
/// runtime 必须用 `issuer_main_account` 读取 admins 模块的 `admins` 真源，
/// 确认 `signer_pubkey` 属于该机构管理员，再验业务 payload 签名。
pub trait CidInstitutionVerifier<AccountId, AccountName, Nonce, Signature> {
    /// 校验 CID 机构登记凭证。
    fn verify_institution_registration(
        cid_number: &[u8],
        cid_full_name: &AccountName,
        cid_short_name: &[u8],
        account_names: &[Vec<u8>],
        nonce: &Nonce,
        signature: &Signature,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
        town_code: &[u8],
    ) -> bool;

    /// 校验机构创建凭证。法定代表人三字段必须被同一签名覆盖，防止冷签前后被替换。
    fn verify_institution_creation(
        cid_number: &[u8],
        cid_full_name: &AccountName,
        cid_short_name: &[u8],
        legal_representative_name: &[u8],
        legal_representative_cid_number: &[u8],
        legal_representative_account: &AccountId,
        account_names: &[Vec<u8>],
        roles_payload: &[u8],
        assignments_payload: &[u8],
        nonce: &Nonce,
        signature: &Signature,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
        town_code: &[u8],
    ) -> bool {
        if legal_representative_name.is_empty() || legal_representative_cid_number.is_empty() {
            return false;
        }
        let _ = (
            legal_representative_account,
            roles_payload,
            assignments_payload,
        );
        Self::verify_institution_registration(
            cid_number,
            cid_full_name,
            cid_short_name,
            account_names,
            nonce,
            signature,
            issuer_cid_number,
            issuer_main_account,
            signer_pubkey,
            scope_province_name,
            scope_city_name,
            town_code,
        )
    }

    /// 校验 CID 机构注销凭证。
    fn verify_institution_deregistration(
        scope: u8,
        cid_number: &[u8],
        account_name: &[u8],
        target_account: &AccountId,
        nonce: &Nonce,
        signature: &Signature,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
    ) -> bool;
}

impl<AccountId, AccountName, Nonce, Signature>
    CidInstitutionVerifier<AccountId, AccountName, Nonce, Signature> for ()
{
    fn verify_institution_registration(
        _cid_number: &[u8],
        _cid_full_name: &AccountName,
        _cid_short_name: &[u8],
        _account_names: &[Vec<u8>],
        _nonce: &Nonce,
        _signature: &Signature,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
        _town_code: &[u8],
    ) -> bool {
        false
    }

    fn verify_institution_deregistration(
        _scope: u8,
        _cid_number: &[u8],
        _account_name: &[u8],
        _target_account: &AccountId,
        _nonce: &Nonce,
        _signature: &Signature,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId,
        _signer_pubkey: &[u8; 32],
    ) -> bool {
        false
    }
}

/// 注册局登记权限抽象。
///
/// 签名验真只证明某个管理员签过登记凭证;本 trait 额外证明该签发机构
/// 对目标机构有登记权。public/private manage 只依赖这个抽象,具体 FRG/CREG 省市规则
/// 由 runtime 统一实现,避免业务 pallet 复制行政区与创世管理员细节。
pub trait RegistryAuthority<AccountId> {
    /// 当前 origin 是否可按签发凭证登记目标机构。
    fn can_register_institution(
        registrar: &AccountId,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        target_cid_number: &[u8],
        target_institution_code: InstitutionCode,
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool;
}

impl<AccountId> RegistryAuthority<AccountId> for () {
    fn can_register_institution(
        _registrar: &AccountId,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId,
        _signer_pubkey: &[u8; 32],
        _target_cid_number: &[u8],
        _target_institution_code: InstitutionCode,
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        false
    }
}

#[cfg(test)]
mod scale_contract_tests {
    use super::*;
    use codec::Encode;

    /// NodeGuard 以单字节判别值镜像机构生命周期；枚举重排会改变链上存储编码，必须立即测试失败。
    #[test]
    fn institution_lifecycle_discriminants_match_node_guard() {
        assert_eq!(InstitutionLifecycleStatus::Pending.encode(), vec![0]);
        assert_eq!(InstitutionLifecycleStatus::Active.encode(), vec![1]);
        assert_eq!(InstitutionLifecycleStatus::Closed.encode(), vec![2]);
    }

    /// 机构记录的声明序就是 SCALE 存储契约，NodeGuard 按同一顺序完整解码。
    #[test]
    fn institution_info_field_order_matches_node_guard() {
        let value = InstitutionInfo {
            cid_full_name: b"full".to_vec(),
            cid_short_name: b"short".to_vec(),
            town_code: b"001".to_vec(),
            legal_representative_name: Some(b"representative".to_vec()),
            legal_representative_cid_number: Some(b"citizen-cid".to_vec()),
            legal_representative_account: Some([9u8; 32]),
            institution_code: *b"NRCG",
            created_at: 7u32,
            status: InstitutionLifecycleStatus::Active,
        };
        assert_eq!(
            value.encode(),
            (
                b"full".to_vec(),
                b"short".to_vec(),
                b"001".to_vec(),
                Some(b"representative".to_vec()),
                Some(b"citizen-cid".to_vec()),
                Some([9u8; 32]),
                *b"NRCG",
                7u32,
                InstitutionLifecycleStatus::Active,
            )
                .encode()
        );
    }
}
