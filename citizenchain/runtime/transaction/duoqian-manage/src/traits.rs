//! 共用的 trait 抽象层。
//!
//! 这些 trait 在个人多签和机构多签业务中都会用到，与 Pallet 内部状态无关，
//! 由 runtime Config 注入实现，便于测试 mock 与生产分离。

extern crate alloc;
use alloc::vec::Vec;

/// 账户地址合法性抽象：用于校验 duoqian_address 是否为本链合法哈希地址。
pub trait DuoqianAddressValidator<AccountId> {
    fn is_valid(address: &AccountId) -> bool;
}

impl<AccountId> DuoqianAddressValidator<AccountId> for () {
    fn is_valid(_address: &AccountId) -> bool {
        true
    }
}

/// 保留地址校验抽象：用于拦截制度保留地址被 duoqian 抢注册。
pub trait DuoqianReservedAddressChecker<AccountId> {
    fn is_reserved(address: &AccountId) -> bool;
}

impl<AccountId> DuoqianReservedAddressChecker<AccountId> for () {
    fn is_reserved(_address: &AccountId) -> bool {
        false
    }
}

/// 转出源地址保护：用于禁止制度保留地址作为资金转出源。
pub trait ProtectedSourceChecker<AccountId> {
    fn is_protected(address: &AccountId) -> bool;
}

impl<AccountId> ProtectedSourceChecker<AccountId> for () {
    fn is_protected(_address: &AccountId) -> bool {
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
/// - 中文注释:签名业务字段收口为 sfid_id / institution_name / account_names[]。
pub trait SfidInstitutionVerifier<AccountName, Nonce, Signature> {
    fn verify_institution_registration(
        sfid_id: &[u8],
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
        _sfid_id: &[u8],
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
