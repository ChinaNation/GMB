//! 费率规则常量库 = fee_policy.rs
//!
//! 全链费率定义的**单一权威源**。所有链上交易计费 / 链下清算行扣费 / 投票治理统一价 /
//! 链上手续费分账比例,都以本文件常量为唯一参考。
//!
//! ## 设计铁律
//!
//! 1. **不可治理修改**:本文件定义的常量是编译期 `pub const`,**不允许**通过链上 storage 治理修改。
//!    改费率 = 改本文件 + 走 `runtime_upgrade::propose_runtime_upgrade` 联合投票升级 wasm。
//! 2. **单一权威源**:全仓库引用费率 / 阈值 / 分账比例时,**只允许**从 `primitives::fee_policy::*`
//!    路径导入。禁止自行定义重复常量,禁止散落多处 hardcode 数值。
//! 3. **链上费率与链下费率分离语义**:
//!    - `ONCHAIN_*` 系列由 `OnchainTxAmountExtractor` + `onchain-transaction` pallet 使用,
//!      用于链上 extrinsic 计费 + 80/10/10 分账。
//!    - `OFFCHAIN_*` 系列由 `offchain-transaction` pallet 使用,描述清算行 L2 链下账本扣费规则;
//!      清算行通过 `propose_l2_fee_rate` 投票设置个体费率,但必须落在 [MIN, MAX] 区间内。
//! 4. **货币单位**:本文件所有金额常量单位都是 `FEN`(分),`1 GMB = 100 FEN`。
//!
//! ## 4 类链上 extrinsic 计费规则(规则定义)
//!
//! | 类别 | 规则 | 实际收费 |
//! |---|---|---|
//! | 免费 | 不进费率公式 | 0 |
//! | 投票/治理 | 固定 `VOTE_FLAT_FEE` | 1 元 |
//! | 链上交易 | `max(amount × ONCHAIN_FEE_RATE, ONCHAIN_MIN_FEE)` | 0.1 元起 |
//! | 未识别 | 拒绝交易 | 不入块 |
//!
//! 具体每个 extrinsic 归哪一类由 `runtime/src/configs/mod.rs::OnchainTxAmountExtractor`
//! 决定;新增 extrinsic 必须在该 match 中显式归类。

use sp_runtime::Perbill;

// =====================================================================
// 链上交易手续费模型(Onchain Fee Model)
// =====================================================================

/// 链上交易费率:**0.1%**(amount × 1‰)。
///
/// 与 `ONCHAIN_MIN_FEE` 共同决定链上 extrinsic 的实际收费:
/// `fee = max(amount × ONCHAIN_FEE_RATE, ONCHAIN_MIN_FEE)`。
pub const ONCHAIN_FEE_RATE: Perbill = Perbill::from_parts(1_000_000);

/// 链上交易单笔最小手续费:**10 FEN = 0.1 元**。
///
/// 当 `amount × ONCHAIN_FEE_RATE` 不足 10 FEN 时,按本最低值收取。
pub const ONCHAIN_MIN_FEE: u128 = 10;

// =====================================================================
// 投票/治理类统一费用(Vote Flat Fee)
// =====================================================================

/// 投票 / 治理类 extrinsic 统一费用:**100 FEN = 1 元/次**。
///
/// 适用范围:
/// - VotingEngine: internal_vote / joint_vote / citizen_vote /
///   retry_passed_proposal / cancel_passed_proposal
/// - 各业务 pallet 不涉及金额的 propose_X / cleanup_X / register_X / 管理操作
///
/// 详见 `runtime/src/configs/mod.rs::OnchainTxAmountExtractor`。
pub const VOTE_FLAT_FEE: u128 = 100;

// =====================================================================
// 链上手续费分账(Onchain Fee Split)
// =====================================================================

/// 链上交易手续费铸块全节点分成:**80%**。
pub const ONCHAIN_FEE_FULLNODE_PERCENT: u32 = 80;

/// 链上交易手续费国储会账户分成:**10%**。
pub const ONCHAIN_FEE_NRC_PERCENT: u32 = 10;

/// 链上交易手续费安全基金账户分成:**10%**。
pub const ONCHAIN_FEE_SAFETY_FUND_PERCENT: u32 = 10;

// =====================================================================
// 链下清算行 L2 手续费模型(Offchain Fee Model)
// =====================================================================

/// 链下交易单笔最小手续费:**1 FEN = 0.01 元**。
pub const OFFCHAIN_MIN_FEE: u128 = 1;

/// 链下清算行个体费率下限:**0.01%**。
///
/// 各清算行通过 `propose_l2_fee_rate` 投票设置自身费率,链上校验
/// `OFFCHAIN_FEE_RATE_MIN ≤ rate ≤ OFFCHAIN_FEE_RATE_MAX`。
pub const OFFCHAIN_FEE_RATE_MIN: Perbill = Perbill::from_parts(100_000);

/// 链下清算行个体费率上限:**0.1%**。
pub const OFFCHAIN_FEE_RATE_MAX: Perbill = Perbill::from_parts(1_000_000);

// =====================================================================
// 运营类费用乘数(Operational Fee Multiplier)
// =====================================================================

/// 运营类交易费乘数:**1**(不额外加价)。
///
/// 由 pallet-transaction-payment 引用,用于区分 `Operational` 与 `Normal` dispatch class。
pub const OPERATIONAL_FEE_MULTIPLIER: u8 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    /// 链上手续费分成铁律:三方比例之和必须 = 100%。
    #[test]
    fn onchain_fee_percents_sum_to_100() {
        assert_eq!(
            ONCHAIN_FEE_FULLNODE_PERCENT
                + ONCHAIN_FEE_NRC_PERCENT
                + ONCHAIN_FEE_SAFETY_FUND_PERCENT,
            100
        );
    }

    /// 投票统一价 = 1 元 = 100 FEN(精度 2 位)。
    #[test]
    fn vote_flat_fee_equals_one_yuan() {
        assert_eq!(VOTE_FLAT_FEE, 100);
    }

    /// 链下费率上下限合法。
    #[test]
    fn offchain_rate_bounds_consistent() {
        assert!(
            OFFCHAIN_FEE_RATE_MIN.deconstruct() <= OFFCHAIN_FEE_RATE_MAX.deconstruct(),
            "OFFCHAIN_FEE_RATE_MIN must not exceed OFFCHAIN_FEE_RATE_MAX"
        );
    }

    /// 最低费用必须 > 0,防止零费用攻击。
    #[test]
    fn min_fees_positive() {
        assert!(ONCHAIN_MIN_FEE > 0);
        assert!(OFFCHAIN_MIN_FEE > 0);
    }

    /// 链上费率 > 0,防止零费率绕过。
    #[test]
    fn onchain_rate_positive() {
        assert!(ONCHAIN_FEE_RATE.deconstruct() > 0);
    }

    /// 链下费率 Perbill / bp 单位换算自洽:
    /// `OFFCHAIN_FEE_RATE_MIN.deconstruct() / 100_000 = 1 bp`(0.01%),
    /// `OFFCHAIN_FEE_RATE_MAX.deconstruct() / 100_000 = 10 bp`(0.1%)。
    /// `offchain-transaction::fee_config::L2_FEE_RATE_BP_MIN/MAX` 从此处推导,
    /// 一旦本铁律破裂,链下清算行个体费率边界即与单一权威源脱钩。
    #[test]
    fn offchain_perbill_to_bp_conversion_stable() {
        assert_eq!(OFFCHAIN_FEE_RATE_MIN.deconstruct() / 100_000, 1);
        assert_eq!(OFFCHAIN_FEE_RATE_MAX.deconstruct() / 100_000, 10);
    }
}
