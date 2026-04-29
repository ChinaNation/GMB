//! 扫码支付清算体系 Step 1 新增:清算行合法性判定。
//!
//! 中文注释:
//! - 清算行(L2)= SFR 私法人 或 FFR 非法人(两者都是"私权机构")。
//! - 清算行在 SFID 系统注册时生成 sfid_id,并在链上 `duoqian-manage` 注册
//!   主账户 + 费用账户两个多签账户。
//! - 本模块判定:某个地址能否作为"可被 L3 绑定的清算行主账户"。
//!
//! **解耦设计**:bank_check 不直接依赖 `duoqian-manage`,而是通过
//! `SfidAccountQuery` trait 抽象机构登记表。runtime 层实现该 trait(内部委托
//! 给 `duoqian-manage` 的 Storage),测试层可用 `()` 空实现或 mock,从
//! 而避免 pallet 之间形成强耦合、tests 需要完整 impl `duoqian_manage::Config`。

use frame_support::ensure;
use sp_std::vec::Vec;

use crate::{Config, Error};

// ------------------------------------------------------------
// 常量
// ------------------------------------------------------------

/// SFID 字符串中第一段 A3 的长度(如 "SFR" / "FFR")。
pub const A3_LEN: usize = 3;

/// 清算行"主账户"名称(字节形式,与 SFID 系统生成时逐字节一致)。
pub const ACCOUNT_NAME_MAIN: &[u8] = "主账户".as_bytes();
/// 清算行"费用账户"名称。
pub const ACCOUNT_NAME_FEE: &[u8] = "费用账户".as_bytes();

// ------------------------------------------------------------
// 机构登记表查询抽象
// ------------------------------------------------------------

/// 机构登记表查询抽象。
///
/// 运行时由 `duoqian-manage` 的 `AddressRegisteredSfid` / `SfidRegisteredAddress` /
/// `DuoqianAccounts` / `InstitutionMetadata` 四个 Storage 组合实现。测试可用 `()` 或 mock。
pub trait SfidAccountQuery<AccountId> {
    /// 地址 → (sfid_id 字节, account_name 字节)。未登记返回 None。
    fn account_info(addr: &AccountId) -> Option<(Vec<u8>, Vec<u8>)>;
    /// (sfid_id, account_name) → 地址。未登记返回 None。
    fn find_address(sfid_id: &[u8], account_name: &[u8]) -> Option<AccountId>;
    /// 该地址对应的多签账户是否处于 Active 状态。
    fn is_active(addr: &AccountId) -> bool;
    /// `who` 是否是 `bank` 对应 DuoqianAccount 的多签管理员之一。
    /// Step 2 新增:清算行费率提案 / 关闭等治理动作需校验管理员身份。
    fn is_admin_of(bank: &AccountId, who: &AccountId) -> bool;
    /// Step 2(2026-04-27, ADR-007)新增:清算行资格白名单判定。
    ///
    /// 给定一个清算行主账户地址,判定其所属机构是否满足"私法人股份公司
    /// 或其下属非法人"白名单:
    /// - 主账户对应机构 a3 == "SFR" ∧ sub_type == "JOINT_STOCK"
    /// - 或 a3 == "FFR" ∧ parent_sfid_id 指向 SFR + JOINT_STOCK 机构
    ///
    /// 实现委托给 runtime 层查 `InstitutionMetadata` storage,trait 层不
    /// 暴露 a3/sub_type/parent_sfid_id 等具体字段,保持 bank_check 解耦。
    fn is_clearing_bank_eligible(addr: &AccountId) -> bool;
    /// Step 2(2026-04-27, ADR-007)新增:节点是否已声明为清算行节点。
    ///
    /// 链上 `ClearingBankNodes` storage 由 sfid_id 索引;此方法接受主账户
    /// 地址参数,内部由实现层反查 sfid_id 后判定。
    fn is_registered_clearing_node(bank: &AccountId) -> bool;
}

/// 测试用 no-op 默认实现:一律返回未登记 / 未激活 / 无管理员权限 / 不合资格 / 未声明节点。
impl<AccountId> SfidAccountQuery<AccountId> for () {
    fn account_info(_addr: &AccountId) -> Option<(Vec<u8>, Vec<u8>)> {
        None
    }
    fn find_address(_sfid_id: &[u8], _account_name: &[u8]) -> Option<AccountId> {
        None
    }
    fn is_active(_addr: &AccountId) -> bool {
        false
    }
    fn is_admin_of(_bank: &AccountId, _who: &AccountId) -> bool {
        false
    }
    fn is_clearing_bank_eligible(_addr: &AccountId) -> bool {
        false
    }
    fn is_registered_clearing_node(_bank: &AccountId) -> bool {
        false
    }
}

// ------------------------------------------------------------
// 内部辅助
// ------------------------------------------------------------

/// 判定 SFID 编码字符串的 A3 段属于"私权机构"(SFR 或 FFR)。
///
/// 直接对字节做前缀匹配,不依赖 sfid-system 或 SFID 后端。
fn a3_is_private_institution(sfid_bytes: &[u8]) -> bool {
    if sfid_bytes.len() < A3_LEN {
        return false;
    }
    let a3 = &sfid_bytes[..A3_LEN];
    a3 == b"SFR" || a3 == b"FFR"
}

// ------------------------------------------------------------
// 对外 API
// ------------------------------------------------------------

/// 严格校验:某地址可作为"清算行主账户"被 L3 绑定。
///
/// Step 2(2026-04-27, ADR-007)起 6 重校验,任一失败即拒绝:
/// 1. 在链上 `AddressRegisteredSfid` 有机构登记
/// 2. `account_name` 段等于 "主账户"
/// 3. A3 ∈ {SFR, FFR}(字节级前缀判定)
/// 4. 对应 `DuoqianAccount.status == Active`
/// 5. **资格白名单**:满足 (SFR ∧ JOINT_STOCK) ∨ (FFR ∧ parent.SFR ∧ parent.JOINT_STOCK)
///    通过 `SfidAccountQuery::is_clearing_bank_eligible` 委托给 runtime 层查
///    `InstitutionMetadata` storage(详见 ADR-007)
/// 6. **节点已声明**:`sfid_id ∈ ClearingBankNodes`,确保该机构已加入清算网络
///    (用户不能绑定到"机构合法但未声明清算行节点"的机构)
pub fn ensure_can_be_bound<T: Config>(addr: &T::AccountId) -> Result<(), Error<T>> {
    let (sfid_bytes, account_name_bytes) =
        T::SfidAccountQuery::account_info(addr).ok_or(Error::<T>::NotRegisteredClearingBank)?;

    ensure!(
        account_name_bytes.as_slice() == ACCOUNT_NAME_MAIN,
        Error::<T>::NotMainAccount
    );

    ensure!(
        a3_is_private_institution(sfid_bytes.as_slice()),
        Error::<T>::NotPrivateInstitution
    );

    ensure!(
        T::SfidAccountQuery::is_active(addr),
        Error::<T>::ClearingBankNotActive
    );

    // Step 2 第 5 重:资格白名单(SFR-JOINT_STOCK / FFR-parent.SFR.JOINT_STOCK)
    ensure!(
        T::SfidAccountQuery::is_clearing_bank_eligible(addr),
        Error::<T>::NotEligibleForClearingBank
    );

    // Step 2 第 6 重:必须已声明清算行节点
    ensure!(
        T::SfidAccountQuery::is_registered_clearing_node(addr),
        Error::<T>::ClearingBankNotRegisteredAsNode
    );

    Ok(())
}

/// 反查"清算行费用账户"地址(Step 2 起由 `settlement.rs` 使用)。
///
/// 流程:
/// 1. 由主账户地址反查得到 `sfid_id`
/// 2. 用 `(sfid_id, "费用账户")` 查询费用账户地址
///
/// 若清算行注册时未同步创建费用账户,返回 `FeeAccountNotFound`。
pub fn fee_account_of<T: Config>(main_addr: &T::AccountId) -> Result<T::AccountId, Error<T>> {
    let (sfid_bytes, _) = T::SfidAccountQuery::account_info(main_addr)
        .ok_or(Error::<T>::NotRegisteredClearingBank)?;

    T::SfidAccountQuery::find_address(sfid_bytes.as_slice(), ACCOUNT_NAME_FEE)
        .ok_or(Error::<T>::FeeAccountNotFound)
}

/// 判定某地址是"清算行的任一账户"(主账户或费用账户,私权机构 + Active)。
///
/// 供 `institution-asset` 的 `can_spend` / `is_protected` 实现时使用。
pub fn is_clearing_bank_account<T: Config>(addr: &T::AccountId) -> bool {
    match T::SfidAccountQuery::account_info(addr) {
        Some((sfid, _)) => {
            a3_is_private_institution(sfid.as_slice()) && T::SfidAccountQuery::is_active(addr)
        }
        None => false,
    }
}

// ------------------------------------------------------------
// 单元测试
// ------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a3_sfr_accepted() {
        assert!(a3_is_private_institution(b"SFR-GD-SZ01-CB01-xxx"));
    }

    #[test]
    fn a3_ffr_accepted() {
        assert!(a3_is_private_institution(b"FFR-GD-SZ01-CB01-xxx"));
    }

    #[test]
    fn a3_gfr_rejected() {
        assert!(!a3_is_private_institution(b"GFR-GD-xxx"));
    }

    #[test]
    fn a3_too_short_rejected() {
        assert!(!a3_is_private_institution(b"SF"));
    }

    #[test]
    fn noop_impl_returns_none_and_inactive() {
        let addr: [u8; 32] = [0u8; 32];
        assert!(<() as SfidAccountQuery<[u8; 32]>>::account_info(&addr).is_none());
        assert!(<() as SfidAccountQuery<[u8; 32]>>::find_address(b"SFR", b"main").is_none());
        assert!(!<() as SfidAccountQuery<[u8; 32]>>::is_active(&addr));
    }
}
