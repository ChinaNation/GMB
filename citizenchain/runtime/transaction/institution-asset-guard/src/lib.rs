//! # 机构账户资金操作白名单公共模块 (institution-asset-guard)
//!
//! 本模块不是 pallet，不含 storage 或 extrinsic。
//! 仅提供 `InstitutionAssetAction` 枚举和 `InstitutionAssetGuard` trait，
//! 供 `duoqian-manage-pow`、`duoqian-transfer-pow`、`offchain-transaction-pos` 复用。
//! 实际放行/拒绝规则由 runtime 的 `RuntimeInstitutionAssetGuard` 实现。
//!
//! # Safety
//!
//! 默认 `()` 实现为 **fail-open（全放行）**，仅适用于测试。
//! 生产 runtime 必须配置为 `RuntimeInstitutionAssetGuard`，否则资金白名单层将完全失效。
//! runtime 层应有集成测试锁定 keyless/duoqian/fee_account/普通账户 的允许矩阵。

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// 机构账户资金动作枚举。
///
/// 这里只描述“内部动钱”的执行动作，不描述提案、投票、管理员变更等纯治理动作。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum InstitutionAssetAction {
    /// 机构多签转账执行：从 `duoqian_address` 向外部收款地址转账，并扣手续费。
    DuoqianTransferExecute,
    /// 多签账户关闭执行：把 `duoqian_address` 的余额整体转出。
    DuoqianCloseExecute,
    /// 链下清算批次执行：允许普通付款账户作为批次 source。
    OffchainBatchDebit,
    /// 省储行手续费账户归集：从 `fee_account` 划回机构主账户。
    OffchainFeeSweepExecute,
    /// 国储会安全基金转账：从 `NRC_ANQUAN_ADDRESS` 向指定收款地址转账。
    NrcSafetyFundTransfer,
}

/// 机构账户资金白名单检查器。
///
/// 该接口只解决“哪些内部执行动作可以从哪些制度账户扣钱”。
/// 外部签名权限、提案投票权限、地址注册权限仍由各自模块负责。
pub trait InstitutionAssetGuard<AccountId> {
    fn can_spend(source: &AccountId, action: InstitutionAssetAction) -> bool;
}

impl<AccountId> InstitutionAssetGuard<AccountId> for () {
    fn can_spend(_source: &AccountId, _action: InstitutionAssetAction) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_guard_allows_all_actions() {
        let account = [7u8; 32];
        assert!(<() as InstitutionAssetGuard<[u8; 32]>>::can_spend(
            &account,
            InstitutionAssetAction::DuoqianTransferExecute,
        ));
        assert!(<() as InstitutionAssetGuard<[u8; 32]>>::can_spend(
            &account,
            InstitutionAssetAction::DuoqianCloseExecute,
        ));
        assert!(<() as InstitutionAssetGuard<[u8; 32]>>::can_spend(
            &account,
            InstitutionAssetAction::OffchainBatchDebit,
        ));
        assert!(<() as InstitutionAssetGuard<[u8; 32]>>::can_spend(
            &account,
            InstitutionAssetAction::OffchainFeeSweepExecute,
        ));
        assert!(<() as InstitutionAssetGuard<[u8; 32]>>::can_spend(
            &account,
            InstitutionAssetAction::NrcSafetyFundTransfer,
        ));
    }
}
