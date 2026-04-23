//! 核心常量=core_const.rs

use sp_runtime::Perbill;

/// 1. 货币基础参数（Economic Base）
pub const TOKEN_SYMBOL: &str = "GMB"; // 公民币符号
pub const TOKEN_DECIMALS: u32 = 2; // 精度：2 位（元 / 分制），1 GMB = 100 FEN
pub const TOKEN_MIN_UNIT: u128 = 1; // 最小计价单位（1 分）
pub const SS58_FORMAT: u16 = 2027; // 地址格式前缀（SS58）
pub const CHAIN_NAME: &str = "CitizenChain"; // 链显示名称
pub const CHAIN_ID: &str = "citizenchain"; // 链唯一 ID（chain spec id）
pub const SUPPORT_URL: &str = "https://www.crcfrcn.com"; // 官方支持网址
pub const BLOCK_HASH_COUNT: u32 = 2400; // 最近区块哈希保留数量
pub const NORMAL_DISPATCH_PERCENT: u32 = 75; // 普通交易可用区块权重比例
pub const MAX_BLOCK_BYTES: u32 = 100 * 1024 * 1024; // 单区块最大字节数：100MB

/// 2. 交易手续费模型（Fee Model）
pub const ONCHAIN_FEE_RATE: Perbill = Perbill::from_parts(1_000_000); // 链上交易费率：0.1%
pub const ONCHAIN_MIN_FEE: u128 = 10; // 链上交易单笔最小手续费：0.1 元
pub const ONCHAIN_FEE_FULLNODE_PERCENT: u32 = 80; // 链上交易费铸块全节点分成比例：80%
pub const ONCHAIN_FEE_NRC_PERCENT: u32 = 10; // 链上交易费国储会手续费账户分成比例：10%
pub const ONCHAIN_FEE_SAFETY_FUND_PERCENT: u32 = 10; // 链上交易费安全基金账户分成比例：10%
pub const OFFCHAIN_MIN_FEE: u128 = 1; // 链下交易单笔最小手续费：0.01 元
pub const OFFCHAIN_FEE_RATE_MIN: Perbill = Perbill::from_parts(100_000); // 链下交易费率下限：0.01%
pub const OFFCHAIN_FEE_RATE_MAX: Perbill = Perbill::from_parts(1_000_000); // 链下交易费率上限：0.1%
pub const OPERATIONAL_FEE_MULTIPLIER: u8 = 1; // 运营类交易费乘数（1=不额外加价）

/// 3. 省储行质押年利率模型 (Annual Interest Rate)
pub const SHENGBANK_INITIAL_INTEREST_BP: u32 = 100; // 省储行初始年利率（第一年）：1.00%
pub const SHENGBANK_INTEREST_DECREASE_BP: u32 = 1; // 年利率递减值：0.01%
pub const SHENGBANK_INTEREST_DURATION_YEARS: u32 = 100; // 利率递减年限（100 年后归零）
pub const ENABLE_SHENGBANK_INTEREST_DECAY: bool = true; // 是否启用逐年递减利率模型

// 4. 安全与反滥用参数（Security）
pub const ACCOUNT_EXISTENTIAL_DEPOSIT: u128 = 111; // 账户存在最低余额（Existential Deposit），余额 < 111 分 → 链上账户状态被删除，剩余余额销毁
pub const ALLOW_ZERO_BALANCE_ACCOUNT: bool = false; // 是否允许零余额账户存在（必须关闭）
pub const ENABLE_DUST_CLEANUP: bool = true; // 是否允许 Dust 回收（必须开启）
pub const ALLOW_LOCAL_ADDRESS_GENERATION: bool = true; // 是否允许无限地址本地生成（链下）

/// 5. 统一签名/派生域铁律（Unified Signature & Derivation Domain）
/// 全仓库地址派生（BLAKE2-256）+ 签名 payload（sr25519）统一使用 `DUOQIAN_DOMAIN` 前缀，后接 1 字节 `op_tag` 做子命名空间。
/// preimage = DUOQIAN_DOMAIN (10B) || op_tag (1B) || ss58_prefix_le (2B) || payload_bytes
/// 地址派生：`address = BLAKE2-256(preimage)` → 32 字节 AccountId
/// 签名 payload：`message = BLAKE2-256(SCALE.encode(tuple(domain_str, ..., payload_fields)))`
pub const DUOQIAN_DOMAIN: &[u8; 10] = b"DUOQIAN_V1";

// 地址派生 op_tag (0x00-0x0F)
// 每个 op_tag 单一派生公式，不得复用，OP_MAIN / OP_FEE 覆盖所有机构，保留名 "主账户"/"费用账户"
// 必须强制走这两个 tag，禁止落到 OP_INSTITUTION。OP_INSTITUTION 仅容纳 SFID 机构的自定义命名账户。
pub const OP_MAIN: u8 = 0x00; // 所有机构主账户 · input: ss58 || sfid_id
pub const OP_FEE: u8 = 0x01; // 所有机构费用账户 · input: ss58 || sfid_id
pub const OP_STAKE: u8 = 0x02; // 仅 PRB 质押账户 · input: ss58 || citizens_number_u64_le
pub const OP_AN: u8 = 0x03; // 仅 NRC 安全基金账户 · input: ss58 || NRC_shenfen_id
pub const OP_PERSONAL: u8 = 0x04; // 个人多签账户 · input: ss58 || creator_32 || account_name
pub const OP_INSTITUTION: u8 = 0x05; // SFID 机构自定义命名账户 · input: ss58 || sfid_id || account_name
                                     //（account_name 非空且不得为 "主账户"/"费用账户" 等保留角色名）

// 签名 payload op_tag (0x10-0x1F)
pub const OP_SIGN_BIND: u8 = 0x10; // 公民身份绑定
pub const OP_SIGN_VOTE: u8 = 0x11; // 公民投票
pub const OP_SIGN_POP: u8 = 0x12; // 人口快照
pub const OP_SIGN_INST: u8 = 0x13; // SFID 机构登记
// 注:0x14 ~ 0x17 原为多签/转账/安全基金/手续费划转的离线聚合签名 op_tag
// (Step 1 / Step 2 旧架构),已随 Phase 2"统一投票入口"整改全部删除。
// 所有治理投票一律走 `VotingEngineSystem::internal_vote` 公开 call,业务模块
// 不再持有 `finalize_X` / `vote_X` 聚合签名接口。
// 新业务从 0x18 起分配,签名域 op_tag 空间共 0x10-0x1F。

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_percents_sum_to_100() {
        // 中文注释：链上手续费分成比例必须恰好为 100%。
        assert_eq!(
            ONCHAIN_FEE_FULLNODE_PERCENT
                + ONCHAIN_FEE_NRC_PERCENT
                + ONCHAIN_FEE_SAFETY_FUND_PERCENT,
            100
        );
    }
}
