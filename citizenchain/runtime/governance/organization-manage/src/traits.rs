//! 机构管理的专属 trait 抽象层(SFID 验签)。
//!
//! 通用的地址校验 / 资金保护 3 个 trait(`DuoqianAccountValidator`/
//! `DuoqianReservedAccountChecker`/`ProtectedSourceChecker`)已提到 `primitives::multisig`,
//! 由 personal-manage / organization-manage / duoqian-transfer 共用,
//! 这里仅保留机构专属的 SFID 注册验签抽象。

extern crate alloc;
use alloc::vec::Vec;

// 中文注释:机构多签与个人多签共用账户校验 trait,单一来源在 primitives::multisig。
pub use primitives::multisig::{
    DuoqianAccountValidator, DuoqianReservedAccountChecker, ProtectedSourceChecker,
};

/// 机构多签账户的查询 trait,供 duoqian-transfer / runtime config 等下游调用。
///
/// 输入任意机构账户(主/费用/自创)都返回该账户的 admin 配置,
/// 实现:`AccountRegisteredSfid[addr]` → `Institutions[sfid].org`
/// → `admins-change::AdminAccounts[addr]`。
/// 与 personal-manage::PersonalMultisigQuery 对仗,duoqian-transfer 通过 union
/// 查询(先 personal、再 institution)定位多签 admin 配置。
pub trait InstitutionMultisigQuery<AccountId> {
    /// 返回机构账户管理员账户 org。机构账户只能是 ORG_PUP 或 ORG_OTH。
    fn lookup_org(addr: &AccountId) -> Option<u8>;

    fn lookup_admin_config(
        addr: &AccountId,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId>>;
    fn is_active(addr: &AccountId) -> bool;
}

impl<AccountId> InstitutionMultisigQuery<AccountId> for () {
    fn lookup_org(_addr: &AccountId) -> Option<u8> {
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

/// SFID 机构登记验签抽象。
///
/// 中文注释:签发身份统一为机构模型。runtime 必须用 `issuer_main_account`
/// 读取 admins-change 的 `admins` 真源,确认 `signer_pubkey` 属于该机构管理员,
/// 再用 `signer_pubkey` 对业务 payload 验签。
pub trait SfidInstitutionVerifier<AccountId, AccountName, Nonce, Signature> {
    fn verify_institution_registration(
        sfid_number: &[u8],
        sfid_full_name: &AccountName,
        account_names: &[Vec<u8>],
        nonce: &Nonce,
        signature: &Signature,
        issuer_sfid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool;
}

impl<AccountId, AccountName, Nonce, Signature>
    SfidInstitutionVerifier<AccountId, AccountName, Nonce, Signature> for ()
{
    fn verify_institution_registration(
        _sfid_number: &[u8],
        _sfid_full_name: &AccountName,
        _account_names: &[Vec<u8>],
        _nonce: &Nonce,
        _signature: &Signature,
        _issuer_sfid_number: &[u8],
        _issuer_main_account: &AccountId,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        false
    }
}
