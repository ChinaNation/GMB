//! 费率常量单一真源:链上交易、链下清算、投票治理与手续费分账。

use sp_runtime::Perbill;

// 链上交易费:`max(amount × 0.1%, ONCHAIN_MIN_FEE)`。
pub const ONCHAIN_FEE_RATE: Perbill = Perbill::from_parts(1_000_000);

/// 链上交易单笔最小手续费:10 FEN = 0.1 元。
pub const ONCHAIN_MIN_FEE: u128 = 10;

/// 投票/治理类统一费用:100 FEN = 1 元/次。
pub const VOTE_FLAT_FEE: u128 = 100;

/// 链上发行代币一次性创建费:100_000 FEN = 1000 元/次。
pub const ONCHAIN_ASSET_CREATE_FEE: u128 = 100_000;

// 链上交易手续费分账。
/// 铸块全节点分成:80%。
pub const ONCHAIN_FEE_FULLNODE_PERCENT: u32 = 80;

/// 国家储委会账户分成:10%。
pub const ONCHAIN_FEE_NRC_PERCENT: u32 = 10;

/// 安全基金账户分成:10%。
pub const ONCHAIN_FEE_SAFETY_FUND_PERCENT: u32 = 10;

// 链下清算行 L2 手续费。
/// 链下交易单笔最小手续费:1 FEN = 0.01 元。
pub const OFFCHAIN_MIN_FEE: u128 = 1;

/// 链下清算行个体费率下限:0.01%。
pub const OFFCHAIN_FEE_RATE_MIN: Perbill = Perbill::from_parts(100_000);

/// 链下清算行个体费率上限:0.1%。
pub const OFFCHAIN_FEE_RATE_MAX: Perbill = Perbill::from_parts(1_000_000);

/// 运营类交易费乘数:1,不额外加价。
pub const OPERATIONAL_FEE_MULTIPLIER: u8 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    /// 链上手续费三方分账必须等于 100%。
    #[test]
    fn onchain_fee_percents_sum_to_100() {
        assert_eq!(
            ONCHAIN_FEE_FULLNODE_PERCENT
                + ONCHAIN_FEE_NRC_PERCENT
                + ONCHAIN_FEE_SAFETY_FUND_PERCENT,
            100
        );
    }

    /// 投票统一价为 1 元。
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

    /// 最低费用必须大于 0。
    #[test]
    fn min_fees_positive() {
        assert!(ONCHAIN_MIN_FEE > 0);
        assert!(OFFCHAIN_MIN_FEE > 0);
    }

    /// 链上费率必须大于 0。
    #[test]
    fn onchain_rate_positive() {
        assert!(ONCHAIN_FEE_RATE.deconstruct() > 0);
    }

    /// 链下费率 Perbill 到 bp 的换算保持稳定。
    #[test]
    fn offchain_perbill_to_bp_conversion_stable() {
        assert_eq!(OFFCHAIN_FEE_RATE_MIN.deconstruct() / 100_000, 1);
        assert_eq!(OFFCHAIN_FEE_RATE_MAX.deconstruct() / 100_000, 10);
    }
}
