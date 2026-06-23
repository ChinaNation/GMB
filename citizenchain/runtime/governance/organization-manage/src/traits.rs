//! 机构管理的专属 trait 抽象层(CID 验签)。
//!
//! 通用的地址校验 / 资金保护 3 个 trait(`AccountValidator`/
//! `ReservedAccountGuard`/`ProtectedSourceChecker`)已提到 `primitives::multisig`,
//! 由 personal-manage / organization-manage / multisig-transfer 共用,
//! 这里仅保留机构专属的 CID 注册验签抽象。

extern crate alloc;
use alloc::vec::Vec;
use primitives::code::InstitutionCode;

// 中文注释:机构多签与个人多签共用账户校验 trait,单一来源在 primitives::multisig。
pub use primitives::multisig::{AccountValidator, ProtectedSourceChecker, ReservedAccountGuard};

/// 机构多签账户的查询 trait,供 multisig-transfer / runtime config 等下游调用。
///
/// 输入任意机构账户(主/费用/自创)都返回该账户的 admin 配置,
/// 实现:`AccountRegisteredCid[addr]` → `Institutions[cid].institution_code`
/// → `admins-change::AdminAccounts[addr]`。
/// 与 personal-manage::PersonalMultisigQuery 对仗,multisig-transfer 通过 union
/// 查询(先 personal、再 institution)定位多签 admin 配置。
pub trait InstitutionMultisigQuery<AccountId> {
    /// 返回机构账户管理员账户机构码。机构账户只能是公权/私权法人机构码。
    fn lookup_org(addr: &AccountId) -> Option<InstitutionCode>;

    fn lookup_admin_config(
        addr: &AccountId,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId>>;
    fn is_active(addr: &AccountId) -> bool;
}

impl<AccountId> InstitutionMultisigQuery<AccountId> for () {
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

/// CID 机构登记验签抽象。
///
/// 中文注释:签发身份统一为机构模型。runtime 必须用 `issuer_main_account`
/// 读取 admins-change 的 `admins` 真源,确认 `signer_pubkey` 属于该机构管理员,
/// 再用 `signer_pubkey` 对业务 payload 验签。
pub trait CidInstitutionVerifier<AccountId, AccountName, Nonce, Signature> {
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

    /// 中文注释:注销凭证验签(注册局在 CID 注销机构/账户后签发,机构管理员发起链上
    /// close 时携带)。与 `verify_institution_registration` 对称;`target_account` 与
    /// `scope`(0=整机构/1=单账户)进签名,防止换账户/换范围重放。
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
