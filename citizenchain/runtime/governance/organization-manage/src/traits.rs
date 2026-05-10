//! 机构管理的专属 trait 抽象层(SFID 验签)。
//!
//! 通用的地址校验 / 资金保护 3 个 trait(`DuoqianAddressValidator`/
//! `DuoqianReservedAddressChecker`/`ProtectedSourceChecker`)已提到 `primitives::traits`,
//! 由 personal-manage / organization-manage / duoqian-transfer 共用,
//! 这里仅保留机构专属的 SFID 注册验签抽象。

extern crate alloc;
use alloc::vec::Vec;

// 中文注释:为兼容下游 use 路径不动,以 re-export 形式从 primitives 导出 3 个共用 trait。
// 后续命名修正(任务卡 C)会把下游引用切到 primitives::traits 后再删 re-export。
pub use primitives::traits::{
    DuoqianAddressValidator, DuoqianReservedAddressChecker, ProtectedSourceChecker,
};

/// 机构多签账户的查询 trait,供 duoqian-transfer / runtime config 等下游调用。
///
/// 输入任意机构账户(主/费用/自创)都返回该账户的 admin 配置,
/// 实现:`AddressRegisteredSfid[addr]` → `Institutions[sfid].admin_org`
/// → `admins-change::Subjects[subject_id_from_institution_account(addr)]`。
/// 与 personal-manage::PersonalMultisigQuery 对仗,duoqian-transfer 通过 union
/// 查询(先 personal、再 institution)定位多签 admin 配置。
pub trait InstitutionMultisigQuery<AccountId> {
    /// 返回机构账户管理员主体 org。机构账户只能是 ORG_PUP 或 ORG_OTH。
    fn lookup_admin_org(addr: &AccountId) -> Option<u8>;

    fn lookup_admin_config(
        addr: &AccountId,
    ) -> Option<primitives::types::MultisigConfigSnapshot<AccountId>>;
    fn is_active(addr: &AccountId) -> bool;
}

impl<AccountId> InstitutionMultisigQuery<AccountId> for () {
    fn lookup_admin_org(_addr: &AccountId) -> Option<u8> {
        None
    }

    fn lookup_admin_config(
        _addr: &AccountId,
    ) -> Option<primitives::types::MultisigConfigSnapshot<AccountId>> {
        None
    }
    fn is_active(_addr: &AccountId) -> bool {
        false
    }
}

/// SFID 机构登记验签抽象（ADR-008 step2b 起按 (province, admin_pubkey) 二元组验签）。
///
/// runtime 查 `sfid_system::ShengSigningPubkey[(province, signer_admin_pubkey)]` 取得签名公钥后
/// 验签；该省 admin 在 `ShengAdmins` 花名册之外或尚未 activate signing pubkey 时验签必须失败。
///
/// 与 ADR-008 前的差异：
/// - 删除"用 SfidMainAccount 主公钥兜底"分支（链上 0 prior knowledge of SFID）；
/// - `province` 从可选项改为必填；
/// - 新增 `signer_admin_pubkey` 显式指明本次签名的省管理员公钥（main 或 backup_{1,2}）。
/// - 中文注释:签名业务字段收口为 sfid_number / institution_name / account_names[]。
pub trait SfidInstitutionVerifier<AccountName, Nonce, Signature> {
    fn verify_institution_registration(
        sfid_number: &[u8],
        institution_name: &AccountName,
        account_names: &[Vec<u8>],
        nonce: &Nonce,
        signature: &Signature,
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> bool;
}

impl<AccountName, Nonce, Signature> SfidInstitutionVerifier<AccountName, Nonce, Signature> for () {
    fn verify_institution_registration(
        _sfid_number: &[u8],
        _institution_name: &AccountName,
        _account_names: &[Vec<u8>],
        _nonce: &Nonce,
        _signature: &Signature,
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        false
    }
}
