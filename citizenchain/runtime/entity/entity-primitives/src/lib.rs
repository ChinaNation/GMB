#![cfg_attr(not(feature = "std"), no_std)]

//! 实体生命周期共用类型与 trait。
//!
//! 本 crate 是 `runtime/entity` 下唯一共享模块，只放无 storage 的类型与接口。
//! 公权机构、私权机构、个人多签分别在各自 pallet 保存生命周期状态；
//! 下游模块通过这里的 trait 做统一查询，不直接读取某个实体 pallet 的 storage。

extern crate alloc;

use alloc::vec::Vec;
use primitives::cid::code::InstitutionCode;

// 中文注释:机构与个人多签共用的账户合法性、保留地址、保护地址检查 trait
// 仍以 primitives::multisig 为唯一真源，entity-primitives 只做实体侧统一出口。
pub use primitives::multisig::{AccountValidator, ProtectedSourceChecker, ReservedAccountGuard};

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

impl<CidNumber> InstitutionCidQuery<CidNumber> for () {
    fn cid_exists(_cid_number: &CidNumber) -> bool {
        false
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
        account_names: &[Vec<u8>],
        nonce: &Nonce,
        signature: &Signature,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool;

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
        _account_names: &[Vec<u8>],
        _nonce: &Nonce,
        _signature: &Signature,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
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
/// 中文注释:签名验真只证明某个管理员签过登记凭证;本 trait 额外证明该签发机构
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
