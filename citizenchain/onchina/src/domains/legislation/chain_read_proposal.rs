//! 提案进度只读投影:链上 votingengine `Proposal` + legislation-vote `LegMeta`/tally → `LegProposalState`。
//!
//! 中文注释:onchina 只读链装配只读投影,**绝不计票**(计票/状态推进全归投票引擎)。镜像字段顺序
//! 锁死链端:`votingengine::Proposal`(types.rs:231)、`legislation-vote::LegislationMeta`(lib.rs:88)、
//! `VoteCountU32/U64`(types.rs:250/258)。`referendum_scope`(Option<PopulationScope>)是 `LegislationMeta`
//! **末字段**且进度投影不需要,故用**前缀解码镜像**(SCALE decode 只读声明字段、忽略尾部字节),
//! 无需引入 `PopulationScope` 结构。
//!
//! 取数复用 chain_runtime 读链范式(iterate + `storage_key_suffix::<8>` 取 proposal_id + 镜像 decode);

use super::law::model::{house_ref, HouseRef};
use crate::core::chain_runtime::storage_key_suffix;
use crate::core::chain_url;
use parity_scale_codec::Decode;
use serde::Serialize;
use subxt::{dynamic, OnlineClient, PolkadotConfig};

/// votingengine `Proposal<BlockNumber, AccountId>` 解码镜像(BlockNumber=u32 / AccountId=[u8;32])。
// 中文注释:部分字段(internal_code/internal_institution/citizen_eligible_total)仅为 SCALE 布局对齐,
// LegProposalState 投影暂不读,保留以锁死解码字段序。
#[allow(dead_code)]
#[derive(Debug, Decode)]
pub struct OnChainProposal {
    /// 提案种类(2 = 立法,PROPOSAL_KIND_LEGISLATION)。
    pub kind: u8,
    /// 阶段(10 院内 / 11 公投 / 12 签署 / 13 会签 / 14 护宪)。
    pub stage: u8,
    /// 状态(投票中 / 通过 / 否决)。
    pub status: u8,
    pub internal_code: Option<[u8; 4]>,
    pub internal_institution: Option<[u8; 32]>,
    pub start: u32,
    pub end: u32,
    pub citizen_eligible_total: u64,
}

/// legislation-vote `LegislationMeta<T>` **前缀**解码镜像(略 `referendum_scope` 末字段,进度投影不需要)。
// 中文注释:executive/legislature 仅为 SCALE 布局对齐,LegProposalState 投影暂不读,保留以锁死解码字段序。
#[allow(dead_code)]
#[derive(Debug, Decode)]
pub struct OnChainLegMeta {
    pub vote_type: u8,
    pub houses: Vec<([u8; 4], [u8; 32])>,
    pub current_house: u32,
    pub referendum_required: bool,
    pub executive: ([u8; 4], [u8; 32]),
    pub legislature: Option<([u8; 4], [u8; 32])>,
    pub needs_guard: bool,
    // referendum_scope: Option<PopulationScope> —— 末字段,前缀解码略过(尾部字节被忽略)。
}

/// `VoteCountU32` 解码镜像(院内表决计票)。
#[derive(Debug, Decode, Default)]
pub struct OnChainVoteCount32 {
    pub yes: u32,
    pub no: u32,
}

/// `VoteCountU64` 解码镜像(公投计票)。
#[derive(Debug, Decode, Default)]
pub struct OnChainVoteCount64 {
    pub yes: u64,
    pub no: u64,
}

/// 计票只读投影(院内 u32 计数统一加宽为 u64 展示)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteTally {
    pub yes: u64,
    pub no: u64,
}

/// 提案进度只读投影(供操作端进度页与大屏消费;不含计票判定,只搬运链上事实)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegProposalState {
    pub proposal_id: u64,
    /// 提案种类(2 = 立法)。
    pub kind: u8,
    /// 阶段(10 院内 / 11 公投 / 12 签署 / 13 会签 / 14 护宪)。
    pub stage: u8,
    /// 状态(投票中 / 通过 / 否决)。
    pub status: u8,
    pub vote_type: u8,
    pub current_house: u32,
    pub referendum_required: bool,
    pub needs_guard: bool,
    /// 表决院序列(机构码 + 账户)。
    pub houses: Vec<HouseRef>,
    /// 阶段起止块。
    pub start_block: u32,
    pub end_block: u32,
    /// 院内表决计票。
    pub house_tally: VoteTally,
    /// 公投计票(非特别案为 0/0)。
    pub referendum_tally: VoteTally,
}

/// 链上 Proposal + LegMeta + tally → 只读投影 `LegProposalState`。
pub fn build_leg_proposal_state(
    proposal_id: u64,
    proposal: &OnChainProposal,
    meta: &OnChainLegMeta,
    house_tally: &OnChainVoteCount32,
    referendum_tally: &OnChainVoteCount64,
) -> LegProposalState {
    LegProposalState {
        proposal_id,
        kind: proposal.kind,
        stage: proposal.stage,
        status: proposal.status,
        vote_type: meta.vote_type,
        current_house: meta.current_house,
        referendum_required: meta.referendum_required,
        needs_guard: meta.needs_guard,
        houses: meta
            .houses
            .iter()
            .map(|(code, account)| house_ref(*code, *account))
            .collect(),
        start_block: proposal.start,
        end_block: proposal.end,
        house_tally: VoteTally {
            yes: house_tally.yes as u64,
            no: house_tally.no as u64,
        },
        referendum_tally: VoteTally {
            yes: referendum_tally.yes,
            no: referendum_tally.no,
        },
    }
}

/// 从某 pallet 的按 proposal_id(u64,Blake2_128Concat)存储项取单条 value 并镜像 decode。
///
/// 中文注释:iterate + `storage_key_suffix::<8>`(u64 key LE 尾部)匹配 proposal_id;proposal 数量少,
/// 整表扫描一次即可(符合 ADR-018 短键取一次)。
async fn fetch_value_by_proposal_id<V: Decode>(
    pallet: &str,
    item: &str,
    proposal_id: u64,
) -> Result<Option<V>, String> {
    let ws_url = chain_url::chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("connect chain ws for {pallet}::{item} failed: {e}"))?;
    let storage = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| format!("get latest chain storage failed: {e}"))?;
    let query = dynamic::storage(pallet, item, Vec::<dynamic::Value>::new());
    let mut iter = storage
        .iter(query)
        .await
        .map_err(|e| format!("iterate {pallet}::{item} failed: {e}"))?;
    while let Some(entry) = iter.next().await {
        let kv = entry.map_err(|e| format!("read {pallet}::{item} failed: {e}"))?;
        let suffix = storage_key_suffix::<8>(&kv.key_bytes)?;
        if u64::from_le_bytes(suffix) == proposal_id {
            let mut raw = kv.value.encoded();
            return Ok(Some(
                V::decode(&mut raw).map_err(|e| format!("decode {pallet}::{item} failed: {e}"))?,
            ));
        }
    }
    Ok(None)
}

/// 读取某提案的完整进度投影(Proposal + LegMeta 必存,tally 缺省 0/0)。
pub async fn fetch_proposal_state(proposal_id: u64) -> Result<Option<LegProposalState>, String> {
    let Some(proposal) =
        fetch_value_by_proposal_id::<OnChainProposal>("VotingEngine", "Proposals", proposal_id)
            .await?
    else {
        return Ok(None);
    };
    let Some(meta) =
        fetch_value_by_proposal_id::<OnChainLegMeta>("LegislationVote", "LegMeta", proposal_id)
            .await?
    else {
        return Ok(None);
    };
    let house_tally = fetch_value_by_proposal_id::<OnChainVoteCount32>(
        "LegislationVote",
        "LegHouseTally",
        proposal_id,
    )
    .await?
    .unwrap_or_default();
    let referendum_tally = fetch_value_by_proposal_id::<OnChainVoteCount64>(
        "LegislationVote",
        "LegReferendumTally",
        proposal_id,
    )
    .await?
    .unwrap_or_default();
    Ok(Some(build_leg_proposal_state(
        proposal_id,
        &proposal,
        &meta,
        &house_tally,
        &referendum_tally,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_scale_codec::Encode;

    /// Proposal 镜像从链端字段序编码的 golden 逐字节解码一致。
    #[test]
    fn proposal_decodes_from_field_ordered_golden() {
        let mut golden = Vec::new();
        golden.extend(2u8.encode()); // kind = 立法
        golden.extend(10u8.encode()); // stage = 院内
        golden.extend(0u8.encode()); // status = 投票中
        golden.extend(Some(*b"NRP\0").encode()); // internal_code
        golden.extend(Option::<[u8; 32]>::None.encode()); // internal_institution
        golden.extend(100u32.encode()); // start
        golden.extend(200u32.encode()); // end
        golden.extend(0u64.encode()); // citizen_eligible_total

        let proposal = OnChainProposal::decode(&mut &golden[..]).expect("decode Proposal");
        assert_eq!(proposal.kind, 2);
        assert_eq!(proposal.stage, 10);
        assert_eq!(proposal.internal_code, Some(*b"NRP\0"));
        assert_eq!(proposal.start, 100);
    }

    /// LegMeta 前缀镜像:即使尾部有 referendum_scope 字节,也正确解码到 needs_guard 为止。
    #[test]
    fn leg_meta_prefix_mirror_ignores_trailing_referendum_scope() {
        let houses: Vec<([u8; 4], [u8; 32])> = vec![(*b"NRP\0", [1u8; 32]), (*b"NSN\0", [2u8; 32])];
        let mut golden = Vec::new();
        golden.extend(2u8.encode()); // vote_type
        golden.extend(houses.encode());
        golden.extend(0u32.encode()); // current_house
        golden.extend(false.encode()); // referendum_required
        golden.extend((*b"PRS\0", [3u8; 32]).encode()); // executive
        golden.extend(Some((*b"NLG\0", [4u8; 32])).encode()); // legislature
        golden.extend(false.encode()); // needs_guard
        golden.extend(Option::<()>::None.encode()); // referendum_scope=None(尾部,前缀镜像忽略)

        let meta = OnChainLegMeta::decode(&mut &golden[..]).expect("decode LegMeta prefix");
        assert_eq!(meta.vote_type, 2);
        assert_eq!(meta.houses.len(), 2);
        assert_eq!(meta.current_house, 0);
        assert!(!meta.referendum_required);
        assert_eq!(meta.executive.0, *b"PRS\0");
        assert_eq!(meta.legislature.as_ref().map(|l| l.0), Some(*b"NLG\0"));
        assert!(!meta.needs_guard);
    }

    /// VoteCount 镜像解码 + 组装 LegProposalState(计票加宽、houses→HouseRef)。
    #[test]
    fn build_state_projects_tally_and_houses() {
        let proposal = OnChainProposal {
            kind: 2,
            stage: 10,
            status: 0,
            internal_code: None,
            internal_institution: None,
            start: 100,
            end: 200,
            citizen_eligible_total: 0,
        };
        let meta = OnChainLegMeta {
            vote_type: 2,
            houses: vec![(*b"NRP\0", [1u8; 32]), (*b"NSN\0", [2u8; 32])],
            current_house: 1,
            referendum_required: false,
            executive: (*b"PRS\0", [3u8; 32]),
            legislature: Some((*b"NLG\0", [4u8; 32])),
            needs_guard: false,
        };
        let house_tally = OnChainVoteCount32 { yes: 220, no: 30 };
        let referendum_tally = OnChainVoteCount64::default();

        let state = build_leg_proposal_state(7, &proposal, &meta, &house_tally, &referendum_tally);
        assert_eq!(state.proposal_id, 7);
        assert_eq!(state.stage, 10);
        assert_eq!(state.current_house, 1);
        assert_eq!(state.houses.len(), 2);
        assert_eq!(state.houses[1].code, "NSN");
        assert_eq!(state.house_tally.yes, 220);
        assert_eq!(state.house_tally.no, 30);
        assert_eq!(state.referendum_tally.yes, 0);
    }
}
