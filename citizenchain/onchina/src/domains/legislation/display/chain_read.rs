//! 大屏专属链读:本机构活跃提案 ID 列表 + 某提案的逐席院内投票映射。
//!
//! 复用 chain_runtime 读链范式(subxt dynamic + 镜像 decode + `storage_key_suffix`)。
//! - 活跃提案:点查 `VotingEngine::ActiveProposalsBySubject[InstitutionCid(cid_number)]`
//!   → `BoundedVec<u64>`(与 `Vec<u64>` 同编码)。
//! - 逐席投票：按 `proposal_id` 迭代 `RepresentativeVotesByAccount`，再按当前代表机构索引过滤。

use std::collections::HashMap;

use codec::{Decode, Encode};
use subxt::{dynamic, OnlineClient, PolkadotConfig};

use crate::core::chain_runtime::storage_key_suffix;
use crate::core::chain_url;

/// 读取某机构的活跃提案 ID 列表(点查 `ActiveProposalsBySubject`;键不存在=空)。
///
/// 值为 `BoundedVec<u64, MaxActiveProposals>`(ValueQuery),点查缺省返回空。
/// 该列表混合该机构所有种类活跃提案,是否为立法案由上层按 `LegProposalState.kind` 过滤。
pub(crate) async fn fetch_active_proposal_ids(cid_number: &str) -> Result<Vec<u64>, String> {
    let subject_key = proposal_subject_institution_cid_key(cid_number)?;
    let ws_url = chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for active proposals failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;
    let address = dynamic::storage(
        "VotingEngine",
        "ActiveProposalsBySubject",
        vec![dynamic::Value::from_bytes(subject_key)],
    );
    let Some(thunk) = storage
        .fetch(&address)
        .await
        .map_err(|e| format!("fetch ActiveProposalsBySubject failed: {e}"))?
    else {
        return Ok(Vec::new());
    };
    let mut raw = thunk.encoded();
    Vec::<u64>::decode(&mut raw).map_err(|e| format!("decode ActiveProposalsBySubject failed: {e}"))
}

/// `ProposalSubject::InstitutionCid(CidNumber)` 的 SCALE 键编码。
///
/// 链端 enum 变体 0 = InstitutionCid,其后是 `BoundedVec<u8>` 的
/// Compact 长度 + UTF-8 CID 字节。不要把主账户 AccountId 当作提案归属键。
fn proposal_subject_institution_cid_key(cid_number: &str) -> Result<Vec<u8>, String> {
    let cid_number = cid_number.trim();
    if cid_number.is_empty() {
        return Err("institution cid_number is required".to_string());
    }
    let mut key = vec![0_u8];
    key.extend(cid_number.as_bytes().to_vec().encode());
    Ok(key)
}

/// 读取某提案当前代表机构的逐席投票。
///
/// 二级键是 `(body_index, account)`；同一钱包在多个机构有席位时，每个机构的票据独立保留。
pub(crate) async fn fetch_representative_ballots(
    proposal_id: u64,
    current_body: u32,
) -> Result<HashMap<String, bool>, String> {
    let ws_url = chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for representative ballots failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;
    // 部分键 = proposal_id(u64);迭代其下所有 (account → bool) 二级键。
    let query = dynamic::storage(
        "LegislationVote",
        "RepresentativeVotesByAccount",
        vec![dynamic::Value::u128(proposal_id as u128)],
    );
    let mut iter = storage
        .iter(query)
        .await
        .map_err(|e| format!("iterate RepresentativeVotesByAccount failed: {e}"))?;
    let mut ballots = HashMap::new();
    while let Some(entry) = iter.next().await {
        let kv = entry.map_err(|e| format!("read RepresentativeVotesByAccount failed: {e}"))?;
        // 元组原始键位于尾部 36 字节：body_index(u32 LE) + AccountId32。
        let key = storage_key_suffix::<36>(&kv.key_bytes)?;
        let body_index = u32::from_le_bytes(
            key[..4]
                .try_into()
                .map_err(|_| "invalid representative ballot body index".to_string())?,
        );
        if body_index != current_body {
            continue;
        }
        let account = &key[4..];
        let mut raw = kv.value.encoded();
        let approve = bool::decode(&mut raw)
            .map_err(|e| format!("decode RepresentativeVotesByAccount value failed: {e}"))?;
        ballots.insert(format!("0x{}", hex::encode(account)), approve);
    }
    Ok(ballots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;

    /// 活跃提案 ID 列表:`BoundedVec<u64>` 与 `Vec<u64>` 同编码,镜像 decode 一致。
    #[test]
    fn active_proposal_ids_decode_from_vec_u64_golden() {
        let ids: Vec<u64> = vec![7, 12, 99];
        let encoded = ids.encode();
        let decoded = Vec::<u64>::decode(&mut &encoded[..]).expect("decode Vec<u64>");
        assert_eq!(decoded, vec![7, 12, 99]);
    }

    /// 空列表(点查缺省)解码为空。
    #[test]
    fn active_proposal_ids_decode_empty() {
        let encoded = Vec::<u64>::new().encode();
        let decoded = Vec::<u64>::decode(&mut &encoded[..]).expect("decode empty");
        assert!(decoded.is_empty());
    }

    /// 活跃提案键必须是 ProposalSubject::InstitutionCid(CID),不是机构主账户。
    #[test]
    fn proposal_subject_institution_cid_key_encodes_scale_enum() {
        let key = proposal_subject_institution_cid_key("LN001-NRP0G-000000001-2026")
            .expect("encode subject key");
        let mut expected = vec![0_u8];
        expected.extend("LN001-NRP0G-000000001-2026".as_bytes().to_vec().encode());
        assert_eq!(key, expected);
    }

    /// 逐席投票值 bool 解码(赞成/反对)。
    #[test]
    fn ballot_value_decodes_bool() {
        assert!(bool::decode(&mut &true.encode()[..]).expect("decode true"));
        assert!(!bool::decode(&mut &false.encode()[..]).expect("decode false"));
    }
}
