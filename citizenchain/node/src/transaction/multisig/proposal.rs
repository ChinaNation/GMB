//! 多签转账提案详情查询与展示适配。
//!
//! 本文件只承载 `MultisigTransfer` pallet 的提案详情结构、SCALE 解码、
//! 独立 storage 查询和列表摘要格式化；治理提案聚合层只调用这里的结果。

use crate::governance::{chain_query, storage_keys};
use serde::Serialize;

/// MODULE_TAG 前缀（必须与 runtime `multisig-transfer` pallet 保持一致）。
const TAG_TRANSFER: &[u8] = b"multisig-transfer";

/// 普通多签转账提案详情（从 `VotingEngine::ProposalData` 解码）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProposalDetail {
    /// 提案主键 ID。
    pub proposal_id: u64,
    /// 机构多签账户 AccountId hex。
    pub institution_hex: String,
    /// 收款人公钥 hex。
    pub beneficiary_hex: String,
    /// 金额（分）。
    pub amount_fen: String,
    /// 转账备注。
    pub remark: String,
    /// 提案人公钥 hex。
    pub proposer_hex: String,
}

/// 安全基金转账提案详情。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyFundProposalDetail {
    /// 提案主键 ID。
    pub proposal_id: u64,
    /// 收款人公钥 hex。
    pub beneficiary_hex: String,
    /// 金额（分）。
    pub amount_fen: String,
    /// 转账备注。
    pub remark: String,
}

/// 手续费划转提案详情。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepProposalDetail {
    /// 提案主键 ID。
    pub proposal_id: u64,
    /// 机构多签账户 AccountId hex。
    pub institution_hex: String,
    /// 金额（分）。
    pub amount_fen: String,
}

/// node 提案详情接口中由 multisig-transfer 模块导出的字段集合。
///
/// 使用 `flatten` 挂到治理聚合返回值上，保持前端当前 JSON 字段，
/// 同时避免 governance/proposal 继续定义多签转账详情结构。
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalDetails {
    /// 普通多签转账详情。
    pub transfer_detail: Option<TransferProposalDetail>,
    /// 安全基金转账详情。
    pub safety_fund_detail: Option<SafetyFundProposalDetail>,
    /// 手续费划转详情。
    pub sweep_detail: Option<SweepProposalDetail>,
}

/// multisig-transfer 业务动作解码结果。
#[derive(Debug, Clone)]
pub enum ProposalAction {
    /// 普通多签转账。
    Transfer(Box<TransferProposalDetail>),
    /// 安全基金转账。
    SafetyFund(Box<SafetyFundProposalDetail>),
    /// 手续费划转。
    Sweep(Box<SweepProposalDetail>),
}

impl ProposalAction {
    /// 转为提案详情接口字段集合。
    pub fn into_details(self) -> ProposalDetails {
        match self {
            ProposalAction::Transfer(detail) => ProposalDetails {
                transfer_detail: Some(*detail),
                ..ProposalDetails::default()
            },
            ProposalAction::SafetyFund(detail) => ProposalDetails {
                safety_fund_detail: Some(*detail),
                ..ProposalDetails::default()
            },
            ProposalAction::Sweep(detail) => ProposalDetails {
                sweep_detail: Some(*detail),
                ..ProposalDetails::default()
            },
        }
    }
}

/// 从 `VotingEngine::ProposalData` 的业务 payload 解码普通多签转账动作。
pub fn decode_proposal_data_action(proposal_id: u64, data: &[u8]) -> Option<ProposalAction> {
    decode_transfer_action(proposal_id, data)
        .map(|detail| ProposalAction::Transfer(Box::new(detail)))
}

/// 从 multisig-transfer 独立 storage 查询安全基金转账或手续费划转动作。
pub fn fetch_stored_action(proposal_id: u64) -> Result<Option<ProposalAction>, String> {
    if let Some(detail) = fetch_safety_fund_proposal_action(proposal_id)? {
        return Ok(Some(ProposalAction::SafetyFund(Box::new(detail))));
    }
    if let Some(detail) = fetch_sweep_proposal_action(proposal_id)? {
        return Ok(Some(ProposalAction::Sweep(Box::new(detail))));
    }
    Ok(None)
}

/// 生成 multisig-transfer 列表摘要。
pub fn format_summary<F>(action: &ProposalAction, resolve_cid_full_name: F) -> String
where
    F: Fn(&str) -> Option<String>,
{
    match action {
        ProposalAction::Transfer(detail) => format_transfer_summary(detail),
        ProposalAction::SafetyFund(detail) => format_safety_fund_summary(detail),
        ProposalAction::Sweep(detail) => format_sweep_summary(detail, resolve_cid_full_name),
    }
}

fn decode_transfer_action(proposal_id: u64, data: &[u8]) -> Option<TransferProposalDetail> {
    // MODULE_TAG("multisig-transfer") + institution + beneficiary
    // + amount: u128(16) + remark: Vec<u8> + proposer: [u8;32]
    let tag = TAG_TRANSFER;
    if data.len() < tag.len() + 32 + 32 + 16 + 1 + 32 {
        return None;
    }
    if &data[..tag.len()] != tag {
        return None;
    }
    let mut offset = tag.len();

    let institution_hex = hex::encode(&data[offset..offset + 32]);
    offset += 32;

    let beneficiary_hex = hex::encode(&data[offset..offset + 32]);
    offset += 32;

    let amount_bytes: [u8; 16] = data[offset..offset + 16].try_into().ok()?;
    let amount_fen = u128::from_le_bytes(amount_bytes);
    offset += 16;

    let (remark_len, remark_len_size) = read_compact_u32(data, offset).ok()?;
    offset += remark_len_size;
    if offset + remark_len as usize > data.len() {
        return None;
    }
    let remark = String::from_utf8_lossy(&data[offset..offset + remark_len as usize]).to_string();
    offset += remark_len as usize;

    if offset + 32 > data.len() {
        return None;
    }
    let proposer_hex = hex::encode(&data[offset..offset + 32]);

    Some(TransferProposalDetail {
        proposal_id,
        institution_hex,
        beneficiary_hex,
        amount_fen: amount_fen.to_string(),
        remark,
        proposer_hex,
    })
}

fn fetch_safety_fund_proposal_action(
    proposal_id: u64,
) -> Result<Option<SafetyFundProposalDetail>, String> {
    let key = storage_keys::map_key(
        "MultisigTransfer",
        "SafetyFundProposalActions",
        &proposal_id.to_le_bytes(),
    );
    // 多签转账动作的金额展示以 finalized storage 为准(ADR-017 收口)。
    match chain_query::fetch_finalized_storage(&key)? {
        None => Ok(None),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            // SafetyFundAction: beneficiary(32) + amount(u128=16) + remark(compact+bytes) + proposer(32)
            if data.len() < 80 {
                return Ok(None);
            }
            let beneficiary_hex = hex::encode(&data[..32]);
            let amount_fen = {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&data[32..48]);
                u128::from_le_bytes(bytes)
            };

            let (remark_len, compact_size) = read_compact_u32(&data, 48)?;
            let remark_start = 48 + compact_size;
            let remark_end = remark_start + remark_len as usize;
            let remark = if remark_end <= data.len() {
                String::from_utf8_lossy(&data[remark_start..remark_end]).to_string()
            } else {
                String::new()
            };

            Ok(Some(SafetyFundProposalDetail {
                proposal_id,
                beneficiary_hex,
                amount_fen: amount_fen.to_string(),
                remark,
            }))
        }
    }
}

fn fetch_sweep_proposal_action(proposal_id: u64) -> Result<Option<SweepProposalDetail>, String> {
    let key = storage_keys::map_key(
        "MultisigTransfer",
        "SweepProposalActions",
        &proposal_id.to_le_bytes(),
    );
    // 多签转账动作的金额展示以 finalized storage 为准(ADR-017 收口)。
    match chain_query::fetch_finalized_storage(&key)? {
        None => Ok(None),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            // SweepAction: institution(AccountId32) + amount(u128=16) + proposer(AccountId32)
            if data.len() < 80 {
                return Ok(None);
            }
            let institution_hex = hex::encode(&data[..32]);
            let amount_fen = {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&data[32..48]);
                u128::from_le_bytes(bytes)
            };
            Ok(Some(SweepProposalDetail {
                proposal_id,
                institution_hex,
                amount_fen: amount_fen.to_string(),
            }))
        }
    }
}

fn decode_hex_storage(hex_str: &str) -> Result<Vec<u8>, String> {
    let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(clean).map_err(|e| format!("hex 解码失败: {e}"))
}

fn read_compact_u32(data: &[u8], offset: usize) -> Result<(u32, usize), String> {
    if offset >= data.len() {
        return Err("Compact<u32> 数据不足".to_string());
    }
    let first = data[offset];
    let mode = first & 0x03;
    match mode {
        0 => Ok(((first >> 2) as u32, 1)),
        1 => {
            if offset + 2 > data.len() {
                return Err("Compact<u32> two-byte 数据不足".to_string());
            }
            let value = (((data[offset + 1] as u32) << 8) | first as u32) >> 2;
            Ok((value, 2))
        }
        2 => {
            if offset + 4 > data.len() {
                return Err("Compact<u32> four-byte 数据不足".to_string());
            }
            let value = ((data[offset + 3] as u32) << 24)
                | ((data[offset + 2] as u32) << 16)
                | ((data[offset + 1] as u32) << 8)
                | (data[offset] as u32);
            Ok((value >> 2, 4))
        }
        _ => Err("Compact<u32> big-integer 模式暂不支持".to_string()),
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn format_amount_fen(amount_fen: &str) -> String {
    let amount: u128 = amount_fen.parse().unwrap_or(0);
    let yuan = amount / 100;
    let cents = amount % 100;
    let mut digits: Vec<char> = yuan.to_string().chars().rev().collect();
    let mut grouped = String::new();
    for (index, ch) in digits.drain(..).enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let yuan_grouped: String = grouped.chars().rev().collect();
    format!("{yuan_grouped}.{cents:02}")
}

fn format_transfer_summary(detail: &TransferProposalDetail) -> String {
    let remark_short = truncate_chars(&detail.remark, 30);
    format!(
        "转账 {} 元：{remark_short}",
        format_amount_fen(&detail.amount_fen)
    )
}

fn format_safety_fund_summary(detail: &SafetyFundProposalDetail) -> String {
    format!("安全基金转账 {} 元", format_amount_fen(&detail.amount_fen))
}

fn format_sweep_summary<F>(detail: &SweepProposalDetail, resolve_cid_full_name: F) -> String
where
    F: Fn(&str) -> Option<String>,
{
    let inst_name =
        resolve_cid_full_name(&detail.institution_hex).unwrap_or_else(|| "未知机构".to_string());
    format!(
        "手续费划转 {} 元：{inst_name}",
        format_amount_fen(&detail.amount_fen)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_summary_truncates_long_remark() {
        let detail = TransferProposalDetail {
            proposal_id: 1,
            institution_hex: String::new(),
            beneficiary_hex: String::new(),
            amount_fen: "12345".to_string(),
            remark: "一二三四五六七八九十".repeat(4),
            proposer_hex: String::new(),
        };
        let summary = format_transfer_summary(&detail);
        assert!(summary.starts_with("转账 123.45 元："));
        assert!(summary.contains('…'));
    }

    #[test]
    fn sweep_summary_uses_unknown_institution_fallback() {
        let detail = SweepProposalDetail {
            proposal_id: 2,
            institution_hex: "00".repeat(48),
            amount_fen: "999900".to_string(),
        };
        assert_eq!(
            format_sweep_summary(&detail, |_| None),
            "手续费划转 9,999.00 元：未知机构"
        );
    }

    #[test]
    fn action_into_details_maps_each_field() {
        let detail = SafetyFundProposalDetail {
            proposal_id: 3,
            beneficiary_hex: String::new(),
            amount_fen: "100".to_string(),
            remark: String::new(),
        };
        let details = ProposalAction::SafetyFund(Box::new(detail)).into_details();
        assert!(details.transfer_detail.is_none());
        assert!(details.safety_fund_detail.is_some());
        assert!(details.sweep_detail.is_none());
    }
}
