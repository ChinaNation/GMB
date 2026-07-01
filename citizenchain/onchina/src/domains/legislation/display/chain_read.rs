//! 大屏专属链读:本机构活跃提案 ID 列表 + 某提案的逐席院内投票映射。
//!
//! 中文注释:复用 chain_runtime 读链范式(subxt dynamic + 镜像 decode + `storage_key_suffix`)。
//! - 活跃提案:点查 `VotingEngine::ActiveProposalsByInstitution[主账户]` → `BoundedVec<u64>`(与 `Vec<u64>` 同编码)。
//! - 逐席投票:按 `proposal_id` **部分键**迭代双 Map `LegislationVote::LegHouseVotesByAdmin`,
//!   尾部 32 字节即账户(Blake2_128Concat 二级键=16 字节哈希 + 32 字节原始账户),value=bool。

use std::collections::HashMap;

use parity_scale_codec::Decode;
use subxt::{dynamic, OnlineClient, PolkadotConfig};

use crate::core::chain_runtime::storage_key_suffix;
use crate::core::chain_url;

/// 读取某机构的活跃提案 ID 列表(点查 `ActiveProposalsByInstitution`;键不存在=空)。
///
/// 中文注释:值为 `BoundedVec<u64, MaxActiveProposals>`(ValueQuery),点查缺省返回空。
/// 该列表混合该机构所有种类活跃提案,是否为立法案由上层按 `LegProposalState.kind` 过滤。
pub(crate) async fn fetch_active_proposal_ids(
    institution_account: [u8; 32],
) -> Result<Vec<u64>, String> {
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
        "ActiveProposalsByInstitution",
        vec![dynamic::Value::from_bytes(institution_account)],
    );
    let Some(thunk) = storage
        .fetch(&address)
        .await
        .map_err(|e| format!("fetch ActiveProposalsByInstitution failed: {e}"))?
    else {
        return Ok(Vec::new());
    };
    let mut raw = thunk.encoded();
    Vec::<u64>::decode(&mut raw)
        .map_err(|e| format!("decode ActiveProposalsByInstitution failed: {e}"))
}

/// 读取某提案的逐席院内投票(`LegHouseVotesByAdmin[proposal_id][account] → bool`)。
///
/// 中文注释:按 `proposal_id` 部分键迭代双 Map,尾部 32 字节为账户;返回 `账户 0x hex → 赞成/反对`。
/// 未在映射内的席位=未投(上层置 `None`)。两院议员账户互不重叠,同一提案两院票据可并存。
pub(crate) async fn fetch_house_ballots(proposal_id: u64) -> Result<HashMap<String, bool>, String> {
    let ws_url = chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for house ballots failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;
    // 部分键 = proposal_id(u64);迭代其下所有 (account → bool) 二级键。
    let query = dynamic::storage(
        "LegislationVote",
        "LegHouseVotesByAdmin",
        vec![dynamic::Value::u128(proposal_id as u128)],
    );
    let mut iter = storage
        .iter(query)
        .await
        .map_err(|e| format!("iterate LegHouseVotesByAdmin failed: {e}"))?;
    let mut ballots = HashMap::new();
    while let Some(entry) = iter.next().await {
        let kv = entry.map_err(|e| format!("read LegHouseVotesByAdmin failed: {e}"))?;
        // 尾部 32 字节 = 账户(二级键 Blake2_128Concat:16 哈希 + 32 原始账户)。
        let account = storage_key_suffix::<32>(&kv.key_bytes)?;
        let mut raw = kv.value.encoded();
        let approve = bool::decode(&mut raw)
            .map_err(|e| format!("decode LegHouseVotesByAdmin value failed: {e}"))?;
        ballots.insert(format!("0x{}", hex::encode(account)), approve);
    }
    Ok(ballots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_scale_codec::Encode;

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

    /// 逐席投票值 bool 解码(赞成/反对)。
    #[test]
    fn ballot_value_decodes_bool() {
        assert!(bool::decode(&mut &true.encode()[..]).expect("decode true"));
        assert!(!bool::decode(&mut &false.encode()[..]).expect("decode false"));
    }
}
