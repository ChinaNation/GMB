//! 跨 pallet 共用的地址校验 / 资金保护抽象。
//!
//! 这些 trait 由 personal-manage / organization-manage / duoqian-transfer 共用，
//! 与 Pallet 内部状态无关，由 runtime Config 注入实现，便于测试 mock 与生产分离。
//! 提到 primitives 是为了避免 personal-manage 反向依赖 organization-manage。

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
