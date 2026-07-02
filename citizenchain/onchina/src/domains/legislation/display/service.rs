//! 名册 × 活跃提案 × 逐席投票的聚合装配。
//!
//! 中文注释:大屏看板的纯装配层——把链读结果拼成 `DisplayBoard`。计票/阶段判定不在此发生
//! (只搬运 `fetch_proposal_state` 的只读投影)。席位投票由本机构名册左连接逐席投票映射得到。

use std::collections::HashMap;

use crate::core::chain_runtime::{
    fetch_active_admin_profiles_onchain, NodeInstitutionIdentity, OnChainAdminProfileView,
};
use crate::domains::legislation::chain_read_proposal::{fetch_proposal_state, LegProposalState};

use super::chain_read::{fetch_active_proposal_ids, fetch_house_ballots};
use super::model::{ActiveProposalView, DisplayBoard, SeatView};

/// 立法提案种类判别式(对齐链端 votingengine `PROPOSAL_KIND_LEGISLATION`)。
const PROPOSAL_KIND_LEGISLATION: u8 = 2;

/// 装配本节点机构的大屏看板:名册 + 活跃立法提案(逐席投票)。
///
/// 中文注释:活跃提案来自 `ActiveProposalsBySubject[InstitutionCid(cid_number)]`;逐个取进度投影,
/// 非立法提案(无 `LegMeta` → `fetch_proposal_state` 返回 `None`)或已清理者跳过。
pub(crate) async fn build_display_board(
    identity: &NodeInstitutionIdentity,
    institution_code: String,
    cid_short_name: Option<String>,
    scope_label: String,
) -> Result<DisplayBoard, String> {
    let roster = fetch_active_admin_profiles_onchain(identity)
        .await?
        .unwrap_or_default();
    // FRG 等非立法机构:`main_account` 为 `[0u8;32]` 哨兵(身份走 frg_province_code 分流),
    // 无立法提案。跳过对省组键的无意义 `ActiveProposalsBySubject` 点查,活跃提案恒空。
    let active_ids = if identity.frg_province_code.is_some() {
        Vec::new()
    } else {
        let cid_number = identity
            .cid_number
            .as_deref()
            .ok_or_else(|| "node binding institution_cid_number is required".to_string())?;
        fetch_active_proposal_ids(cid_number).await?
    };

    let mut active_proposals = Vec::new();
    for proposal_id in active_ids {
        let Some(state) = fetch_proposal_state(proposal_id).await? else {
            continue;
        };
        if state.kind != PROPOSAL_KIND_LEGISLATION {
            continue;
        }
        let ballots = fetch_house_ballots(proposal_id).await?;
        active_proposals.push(build_active_proposal_view(state, &roster, &ballots));
    }

    Ok(DisplayBoard {
        institution_code,
        cid_short_name,
        scope_label,
        roster_total: roster.len() as u32,
        active_proposals,
    })
}

/// 名册左连接逐席投票 → 席位板 + 聚合计数(纯装配,可单测)。
fn build_active_proposal_view(
    state: LegProposalState,
    roster: &[OnChainAdminProfileView],
    ballots: &HashMap<String, bool>,
) -> ActiveProposalView {
    let seats: Vec<SeatView> = roster
        .iter()
        .map(|p| SeatView {
            admin_account: p.account_hex.clone(),
            name: p.name.clone(),
            title: p.title.clone(),
            vote: ballots.get(&p.account_hex).copied(),
        })
        .collect();
    let approved_count = seats.iter().filter(|s| s.vote == Some(true)).count() as u32;
    let rejected_count = seats.iter().filter(|s| s.vote == Some(false)).count() as u32;
    let pending_count = seats.iter().filter(|s| s.vote.is_none()).count() as u32;
    ActiveProposalView {
        state,
        seats,
        approved_count,
        rejected_count,
        pending_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::legislation::chain_read_proposal::VoteTally;

    fn profile(hex_tail: &str, name: &str) -> OnChainAdminProfileView {
        OnChainAdminProfileView {
            account_hex: format!("0x{hex_tail}"),
            admin_cid_number: String::new(),
            name: name.to_string(),
            admin_role: "委员".to_string(),
            title: "委员".to_string(),
            term_start: 0,
            term_end: 0,
            source: 255,
            source_label: String::new(),
        }
    }

    fn sample_state() -> LegProposalState {
        LegProposalState {
            proposal_id: 7,
            kind: 2,
            stage: 10,
            status: 0,
            vote_type: 2,
            current_house: 0,
            referendum_required: false,
            needs_guard: false,
            houses: vec![],
            start_block: 100,
            end_block: 200,
            house_tally: VoteTally { yes: 1, no: 1 },
            referendum_tally: VoteTally { yes: 0, no: 0 },
        }
    }

    #[test]
    fn seats_left_join_ballots_and_count_by_vote() {
        let roster = vec![
            profile("aa", "甲"),
            profile("bb", "乙"),
            profile("cc", "丙"),
        ];
        let mut ballots = HashMap::new();
        ballots.insert("0xaa".to_string(), true);
        ballots.insert("0xbb".to_string(), false);
        // 丙(0xcc)未投。

        let view = build_active_proposal_view(sample_state(), &roster, &ballots);

        assert_eq!(view.seats.len(), 3);
        assert_eq!(view.seats[0].vote, Some(true));
        assert_eq!(view.seats[1].vote, Some(false));
        assert_eq!(view.seats[2].vote, None);
        assert_eq!(view.approved_count, 1);
        assert_eq!(view.rejected_count, 1);
        assert_eq!(view.pending_count, 1);
        assert_eq!(view.state.proposal_id, 7);
    }

    #[test]
    fn empty_roster_yields_no_seats() {
        let view = build_active_proposal_view(sample_state(), &[], &HashMap::new());
        assert!(view.seats.is_empty());
        assert_eq!(view.approved_count, 0);
        assert_eq!(view.pending_count, 0);
    }
}
