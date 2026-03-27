//! 链上事件解析器。
//!
//! 使用 subxt 动态 API 解码区块事件，匹配所有余额变动事件，
//! 转换为 `TxRecordInsert` 写入数据库。

use chrono::{DateTime, TimeZone, Utc};
use sp_core::crypto::Ss58Codec;
use subxt::ext::scale_value::{At, Composite, Value};
use subxt::events::{EventDetails, Phase};
use subxt::PolkadotConfig;
use tracing::warn;

use super::db::TxRecordInsert;

/// citizenchain 的 SS58 地址前缀。
const SS58_PREFIX: u16 = 2027;

/// 将 32 字节 AccountId 编码为 SS58 地址。
fn account_to_ss58(bytes: &[u8; 32]) -> String {
    sp_core::sr25519::Public::from_raw(*bytes)
        .to_ss58check_with_version(sp_core::crypto::Ss58AddressFormat::custom(SS58_PREFIX))
}

/// 从 subxt scale_value::Value 提取 32 字节 AccountId。
///
/// AccountId 在 scale-value 中通常表示为一个包含 32 个 u8 primitive 的 unnamed composite。
fn extract_account_id<T>(val: &Value<T>) -> Option<[u8; 32]> {
    // 尝试从 composite 中提取 32 字节
    match &val.value {
        subxt::ext::scale_value::ValueDef::Composite(composite) => {
            extract_bytes_from_composite(composite)
        }
        _ => None,
    }
}

/// 从 Composite 提取 32 字节。
fn extract_bytes_from_composite<T>(composite: &Composite<T>) -> Option<[u8; 32]> {
    let mut bytes = [0u8; 32];
    let values: Vec<_> = composite.values().collect();
    if values.len() != 32 {
        return None;
    }
    for (i, val) in values.iter().enumerate() {
        bytes[i] = val.as_u128()? as u8;
    }
    Some(bytes)
}

/// 从 subxt Value 提取 u128 金额。
fn extract_balance<T>(val: &Value<T>) -> Option<u128> {
    val.as_u128()
}

/// 将 u128 余额（分）转为 i64。超过 i64::MAX 截断（实际不会发生）。
fn balance_to_i64(amount: u128) -> i64 {
    amount.min(i64::MAX as u128) as i64
}

/// 解析一个区块的所有事件，返回需要写入的交易记录。
pub(crate) fn parse_block_events(
    events: &subxt::events::Events<PolkadotConfig>,
    block_number: i64,
    block_timestamp_ms: Option<u64>,
) -> Vec<TxRecordInsert> {
    let block_ts = block_timestamp_ms
        .and_then(|ms| Utc.timestamp_millis_opt(ms as i64).single());

    let mut records = Vec::new();

    for (event_index, event_result) in events.iter().enumerate() {
        let event = match event_result {
            Ok(e) => e,
            Err(err) => {
                warn!(
                    block = block_number,
                    event_index, error = %err,
                    "failed to decode event, skipping"
                );
                continue;
            }
        };

        let pallet = event.pallet_name();
        let variant = event.variant_name();
        let ext_idx = match event.phase() {
            Phase::ApplyExtrinsic(i) => Some(i as i16),
            _ => None,
        };

        if let Some(mut rec) = match_event(pallet, variant, &event, block_number, ext_idx, block_ts)
        {
            rec.event_index = event_index as i16;
            records.push(rec);
        }
    }

    records
}

/// 匹配单个事件，返回 Some(TxRecordInsert) 如果是余额变动事件。
fn match_event(
    pallet: &str,
    variant: &str,
    event: &EventDetails<PolkadotConfig>,
    block_number: i64,
    extrinsic_index: Option<i16>,
    block_ts: Option<DateTime<Utc>>,
) -> Option<TxRecordInsert> {
    let fields = event.field_values().ok()?;

    match (pallet, variant) {
        // ─── pallet_balances (index 2) ──────────────────────────────
        ("Balances", "Transfer") => {
            let from = fields.at("from").and_then(extract_account_id)?;
            let to = fields.at("to").and_then(extract_account_id)?;
            let amount = fields.at("amount").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "transfer",
                from_address: Some(account_to_ss58(&from)),
                to_address: Some(account_to_ss58(&to)),
                amount_fen: balance_to_i64(amount),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }
        ("Balances", "Withdraw") => {
            let who = fields.at("who").and_then(extract_account_id)?;
            let amount = fields.at("amount").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "fee_withdraw",
                from_address: Some(account_to_ss58(&who)),
                to_address: None,
                amount_fen: balance_to_i64(amount),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }
        ("Balances", "Deposit") => {
            let who = fields.at("who").and_then(extract_account_id)?;
            let amount = fields.at("amount").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "fee_deposit",
                from_address: None,
                to_address: Some(account_to_ss58(&who)),
                amount_fen: balance_to_i64(amount),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }

        // ─── fullnode_pow_reward (index 6) ──────────────────────────
        ("FullnodePowReward", "PowRewardIssued") => {
            let wallet = fields.at("wallet").and_then(extract_account_id)?;
            let amount = fields.at("amount").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "block_reward",
                from_address: None,
                to_address: Some(account_to_ss58(&wallet)),
                amount_fen: balance_to_i64(amount),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }

        // ─── shengbank_stake_interest (index 5) ─────────────────────
        ("ShengBankStakeInterest", "ShengBankInterestMinted") => {
            let account = fields.at("account").and_then(extract_account_id)?;
            let amount = fields.at("amount").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "bank_interest",
                from_address: None,
                to_address: Some(account_to_ss58(&account)),
                amount_fen: balance_to_i64(amount),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }

        // ─── resolution_issuance_iss (index 7) ──────────────────────
        ("ResolutionIssuanceIss", "ResolutionIssuanceExecuted") => {
            let total = fields.at("total_amount").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "gov_issuance",
                from_address: None,
                to_address: None,
                amount_fen: balance_to_i64(total),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }

        // ─── citizen_lightnode_issuance (index 11) ───────────────────
        ("CitizenLightnodeIssuance", "CertificationRewardIssued") => {
            let who = fields.at("who").and_then(extract_account_id)?;
            let reward = fields.at("reward").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "lightnode_reward",
                from_address: None,
                to_address: Some(account_to_ss58(&who)),
                amount_fen: balance_to_i64(reward),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }

        // ─── duoqian_transfer_pow (index 19) ────────────────────────
        ("DuoqianTransferPow", "TransferExecuted") => {
            let beneficiary = fields.at("beneficiary").and_then(extract_account_id)?;
            let amount = fields.at("amount").and_then(extract_balance)?;
            let fee = fields.at("fee").and_then(extract_balance);
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "proposal_transfer",
                from_address: None,
                to_address: Some(account_to_ss58(&beneficiary)),
                amount_fen: balance_to_i64(amount),
                fee_fen: fee.map(balance_to_i64),
                block_timestamp: block_ts,
            })
        }

        // ─── duoqian_manage_pow (index 17) ──────────────────────────
        ("DuoqianManagePow", "DuoqianCreated") => {
            let duoqian = fields.at("duoqian_address").and_then(extract_account_id)?;
            let creator = fields.at("creator").and_then(extract_account_id)?;
            let amount = fields.at("amount").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "duoqian_create",
                from_address: Some(account_to_ss58(&creator)),
                to_address: Some(account_to_ss58(&duoqian)),
                amount_fen: balance_to_i64(amount),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }
        ("DuoqianManagePow", "DuoqianClosed") => {
            let duoqian = fields.at("duoqian_address").and_then(extract_account_id)?;
            let beneficiary = fields.at("beneficiary").and_then(extract_account_id)?;
            let amount = fields.at("amount").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "duoqian_close",
                from_address: Some(account_to_ss58(&duoqian)),
                to_address: Some(account_to_ss58(&beneficiary)),
                amount_fen: balance_to_i64(amount),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }

        // ─── resolution_destro_gov (index 14) ───────────────────────
        ("ResolutionDestroGov", "DestroyExecuted") => {
            let amount = fields.at("amount").and_then(extract_balance)?;
            Some(TxRecordInsert {
                block_number,
                extrinsic_index,
                event_index: 0,
                tx_type: "fund_destroy",
                from_address: None,
                to_address: None,
                amount_fen: balance_to_i64(amount),
                fee_fen: None,
                block_timestamp: block_ts,
            })
        }

        _ => None,
    }
}
