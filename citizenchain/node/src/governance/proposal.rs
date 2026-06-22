// 提案查询：提案列表、详情、投票计数，通过 RPC 读取 VotingEngine 链上存储。

use crate::shared::proposal_business;
use primitives::institution_code::InstitutionCode;
use serde::Serialize;

use super::{chain_query, signing, storage_keys};

/// 把机构码 [u8;4] 序列化为 4 字符展示串(末尾 `\0` 填充去掉),便于前端消费。
fn serialize_internal_code<S>(
    code: &Option<InstitutionCode>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match code {
        Some(code) => {
            let end = code.iter().position(|&b| b == 0).unwrap_or(code.len());
            serializer.serialize_some(&String::from_utf8_lossy(&code[..end]))
        }
        None => serializer.serialize_none(),
    }
}

const TAG_RUNTIME_UPGRADE: &[u8] = b"rt-upg";
const TAG_RESOLUTION_ISSUANCE: &[u8] = b"res-iss";
const TAG_RESOLUTION_DESTROY: &[u8] = b"res-dst";
/// 多签管理提案 TAG — 不属于治理提案，在治理列表中过滤掉。
const TAG_ORGANIZATION_MANAGE: &[u8] = b"org-mgmt";

fn institution_account_from_cid(cid_number: &str) -> Result<[u8; 32], String> {
    let entry = super::registry::find_institution(cid_number)
        .ok_or_else(|| format!("未知的治理机构 cidNumber: {cid_number}"))?;
    let clean = entry.main_account_hex();
    let bytes = hex::decode(&clean).map_err(|e| format!("机构 AccountId 解码失败: {e}"))?;
    bytes
        .try_into()
        .map_err(|_| "机构 AccountId 必须为 32 字节".to_string())
}

/// 提案展示号(双层 ID v1):`ProposalDisplayId[id]` 反查值。
///
/// 主键 `proposal_id` 是全局单调 u64;展示号通过本结构持有
/// `(year, seq_in_year)`,渲染层把它拼成 "2026000123" 风格。
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalDisplayMeta {
    pub year: u16,
    pub seq_in_year: u32,
}

/// 提案元数据（从 VotingEngine::Proposals 解码）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalMeta {
    pub proposal_id: u64,
    /// 0=内部投票, 1=联合投票。
    pub kind: u8,
    /// 0=内部投票, 1=联合投票内部投票阶段, 2=联合公投阶段。
    pub stage: u8,
    /// 0=投票中, 1=通过, 2=否决, 3=已执行, 4=执行失败。
    pub status: u8,
    /// 仅内部投票:机构码(CID institution_code),序列化为 4 字符展示串。
    #[serde(serialize_with = "serialize_internal_code")]
    pub internal_code: Option<InstitutionCode>,
    /// 机构多签 AccountId32 hex（不含 0x）。
    pub institution_hex: Option<String>,
}

/// 协议升级提案详情（从 VotingEngine::ProposalData 解码）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUpgradeDetail {
    pub proposal_id: u64,
    pub proposer_hex: String,
    pub reason: String,
    pub code_hash_hex: String,
}

/// 投票计数。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteTally {
    pub yes: u32,
    pub no: u32,
}

/// 联合投票计数（权重制）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JointVoteTally {
    pub yes: u32,
    pub no: u32,
}

/// 联合公投计数。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferendumVoteTally {
    pub yes: u64,
    pub no: u64,
}

/// 提案完整信息（元数据 + 业务详情 + 投票进度）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalFullInfo {
    pub meta: ProposalMeta,
    #[serde(flatten)]
    pub business_details: proposal_business::ProposalDetails,
    pub runtime_upgrade_detail: Option<RuntimeUpgradeDetail>,
    pub fee_rate_detail: Option<FeeRateProposalDetail>,
    pub resolution_issuance_detail: Option<ResolutionIssuanceDetail>,
    pub resolution_destroy_detail: Option<ResolutionDestroyDetail>,
    pub internal_tally: Option<VoteTally>,
    pub joint_tally: Option<JointVoteTally>,
    pub referendum_tally: Option<ReferendumVoteTally>,
    /// 关联机构名称（通过 institutionBytes 反查）。
    pub cid_full_name: Option<String>,
}

/// 费率提案详情。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeRateProposalDetail {
    pub proposal_id: u64,
    pub institution_hex: String,
    pub new_rate_bp: u32,
}

/// 决议发行提案详情。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionIssuanceDetail {
    pub proposal_id: u64,
    pub proposer_hex: String,
    pub reason: String,
    pub total_amount_fen: String,
    pub allocations: Vec<IssuanceAllocationItem>,
}

/// 决议发行分配项。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuanceAllocationItem {
    pub recipient_hex: String,
    pub amount_fen: String,
}

/// 决议销毁提案详情。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionDestroyDetail {
    pub proposal_id: u64,
    pub institution_hex: String,
    pub amount_fen: String,
}

/// 提案列表项（轻量，用于列表展示）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalListItem {
    pub proposal_id: u64,
    pub display_id: String,
    pub kind: u8,
    pub kind_label: String,
    pub stage: u8,
    pub stage_label: String,
    pub status: u8,
    pub status_label: String,
    pub cid_full_name: Option<String>,
    /// 简要描述（转账提案：金额+备注，升级提案：reason 前 50 字）。
    pub summary: String,
}

/// 提案分页结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalPageResult {
    pub items: Vec<ProposalListItem>,
    /// 是否还有更多。
    pub has_more: bool,
    pub warning: Option<String>,
}

struct ProposalDisplayInfo {
    summary: String,
    status: u8,
    status_label: String,
}

/// 提案业务动作（统一解码结果）。
///
/// 列表 summary 与详情 `ProposalFullInfo` 的各个 `*_detail` 字段均由它派生,
/// 保证两条路径对同一提案看到一致内容,避免再次出现"列表说无详情、详情页却能显示"的漂移。
///
/// 每种变体用 `Box` 包裹是为了控制 enum 大小(`ResolutionIssuance` 含 Vec 较重)。
enum ProposalAction {
    Business(Box<proposal_business::ProposalAction>),
    RuntimeUpgrade(Box<RuntimeUpgradeDetail>),
    ResolutionIssuance(Box<ResolutionIssuanceDetail>),
    ResolutionDestroy(Box<ResolutionDestroyDetail>),
    /// 中文注释：费率提案详情展示结构已保留，链上查询接入前该动作分支暂不构造。
    #[allow(dead_code)]
    FeeRate(Box<FeeRateProposalDetail>),
    /// 所有可能数据源都查过仍无命中,展示层回退为"无详情数据"。
    Unknown,
}

// ──── 公开查询函数 ────

/// 查询 NextProposalId（VotingEngine 全局递增 ID）。
pub fn fetch_next_proposal_id() -> Result<u64, String> {
    let key = storage_keys::value_key("VotingEngine", "NextProposalId");
    // 中文注释(ADR-017):业务读取统一经 chain_query 钉 finalized,禁止 best。
    match chain_query::fetch_finalized_storage(&key)? {
        None => Ok(0),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.len() < 8 {
                return Ok(0);
            }
            Ok(u64::from_le_bytes(
                data[..8]
                    .try_into()
                    .map_err(|_| "SCALE 数据长度不足".to_string())?,
            ))
        }
    }
}

/// 检查提案是否为多签管理提案（创建/关闭多签账户），这类提案不在治理列表中显示。
fn is_organization_manage_proposal(proposal_id: u64) -> bool {
    let Ok(Some(raw)) = fetch_proposal_data_raw(proposal_id) else {
        return false;
    };
    if raw.is_empty() {
        return false;
    }
    // ProposalData 存储为 BoundedVec<u8>：Compact<len> + bytes
    let Ok((vec_len, len_bytes)) = read_compact_u32(&raw, 0) else {
        return false;
    };
    let offset = len_bytes;
    let tag = TAG_ORGANIZATION_MANAGE;
    offset + tag.len() <= raw.len()
        && (vec_len as usize) >= tag.len()
        && raw[offset..offset + tag.len()] == *tag
}

/// 分页查询提案列表(从 start_id 往前 count 个,按 ID 倒序)。
/// 自动过滤多签管理提案(创建/关闭多签账户),这些在多签账户详情页单独展示。
///
/// 双层 ID v1:主键 0 起单调累加,所以下界用 0(不再按年份切)。
pub fn fetch_proposal_page(start_id: u64, count: u32) -> Result<ProposalPageResult, String> {
    let mut items = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let min_id = start_id.saturating_sub(count as u64);
    let mut id = start_id;

    while id > min_id {
        match fetch_proposal_meta(id) {
            Ok(Some(meta)) => {
                // 中文注释:多签管理提案(创建/关闭多签账户)不在治理列表中显示。
                if is_organization_manage_proposal(id) {
                    if id == 0 {
                        break;
                    }
                    id -= 1;
                    continue;
                }
                // 中文注释:协议升级摘要只保存展示字段，真实状态统一读取 votingengine。
                let display = match fetch_proposal_display(id, &meta) {
                    Ok(v) => v,
                    Err(_) => ProposalDisplayInfo {
                        summary: "(详情查询失败)".to_string(),
                        status: meta.status,
                        status_label: status_label(meta.status).to_string(),
                    },
                };
                let cid_full_name = resolve_cid_full_name(meta.institution_hex.as_deref());
                // 双层 ID v1:展示号从 ProposalDisplayId 反查;查不到 fallback `#id`
                let display_meta = fetch_proposal_display_id(id).ok().flatten();

                items.push(ProposalListItem {
                    proposal_id: id,
                    display_id: format_proposal_id(id, display_meta.as_ref()),
                    kind: meta.kind,
                    kind_label: kind_label(meta.kind).to_string(),
                    stage: meta.stage,
                    stage_label: stage_label(meta.stage).to_string(),
                    status: display.status,
                    status_label: display.status_label,
                    cid_full_name,
                    summary: display.summary,
                });
            }
            Ok(None) => {} // 提案不存在,跳过
            Err(e) => {
                warnings.push(format!("查询提案 {id} 失败: {e}"));
            }
        }
        if id == 0 {
            break;
        }
        id -= 1;
    }

    // 双层 ID v1:主键单调,下界 0
    let has_more = min_id > 0;

    Ok(ProposalPageResult {
        items,
        has_more,
        warning: if warnings.is_empty() {
            None
        } else {
            Some(warnings.join(";"))
        },
    })
}

/// 查询单个提案完整信息。
///
/// 业务动作统一走 [`resolve_proposal_action`] 解析,再按变体填入对应 `*_detail` 字段;
/// 投票计数按 kind/stage 独立查询(保留原有条件)。
pub fn fetch_proposal_full(proposal_id: u64) -> Result<ProposalFullInfo, String> {
    let meta =
        fetch_proposal_meta(proposal_id)?.ok_or_else(|| format!("提案 {proposal_id} 不存在"))?;

    // 业务动作:一次性解析,按变体分发到 7 个 *_detail 字段。
    let action = resolve_proposal_action(proposal_id, &meta)?;
    let (
        business_details,
        runtime_upgrade_detail,
        resolution_issuance_detail,
        resolution_destroy_detail,
        fee_rate_detail,
    ) = split_action_into_details(action);

    // 根据提案阶段查询对应的投票计数。
    let internal_tally = if meta.kind == 0 || meta.stage == 0 {
        fetch_internal_tally(proposal_id).ok()
    } else {
        None
    };

    let joint_tally = if meta.kind == 1 && meta.stage >= 1 {
        fetch_joint_tally(proposal_id).ok()
    } else {
        None
    };

    let referendum_tally = if meta.kind == 1 && meta.stage >= 2 {
        fetch_referendum_tally(proposal_id).ok()
    } else {
        None
    };

    let cid_full_name = resolve_cid_full_name(meta.institution_hex.as_deref());

    Ok(ProposalFullInfo {
        meta,
        business_details,
        runtime_upgrade_detail,
        fee_rate_detail,
        resolution_issuance_detail,
        resolution_destroy_detail,
        internal_tally,
        joint_tally,
        referendum_tally,
        cid_full_name,
    })
}

/// 把 [`ProposalAction`] 展开成 `ProposalFullInfo` 的业务详情字段。
///
/// 业务模块详情由共享适配层承载，再 flatten 到接口返回值中；
/// 其它治理详情仍保留在本聚合层。
#[allow(clippy::type_complexity)]
fn split_action_into_details(
    action: ProposalAction,
) -> (
    proposal_business::ProposalDetails,
    Option<RuntimeUpgradeDetail>,
    Option<ResolutionIssuanceDetail>,
    Option<ResolutionDestroyDetail>,
    Option<FeeRateProposalDetail>,
) {
    match action {
        ProposalAction::Business(d) => ((*d).into_details(), None, None, None, None),
        ProposalAction::RuntimeUpgrade(d) => (
            proposal_business::ProposalDetails::default(),
            Some(*d),
            None,
            None,
            None,
        ),
        ProposalAction::ResolutionIssuance(d) => (
            proposal_business::ProposalDetails::default(),
            None,
            Some(*d),
            None,
            None,
        ),
        ProposalAction::ResolutionDestroy(d) => (
            proposal_business::ProposalDetails::default(),
            None,
            None,
            Some(*d),
            None,
        ),
        ProposalAction::FeeRate(d) => (
            proposal_business::ProposalDetails::default(),
            None,
            None,
            None,
            Some(*d),
        ),
        ProposalAction::Unknown => (
            proposal_business::ProposalDetails::default(),
            None,
            None,
            None,
            None,
        ),
    }
}

/// 分页查询指定机构的所有存在提案（从 start_id 往前，按 ID 倒序）。
///
/// 通过机构多签 AccountId 反向索引读取本机构提案。
/// 每页最多返回 count 条，has_more 表示是否还有更早的提案。
pub fn fetch_institution_proposal_page(
    cid_number: &str,
    start_id: u64,
    count: u32,
) -> Result<ProposalPageResult, String> {
    let mut items = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // 双层 ID v1:走 ProposalsByInstitution 反向索引,O(本机构提案数),
    // 不再扫主键 + 客户端过滤。
    let mut ids = fetch_proposals_by_institution(cid_number)?;
    ids.sort_by(|a, b| b.cmp(a)); // 降序(主键单调,降序即按时间倒序)

    // start_id 是上一次翻页返回的最后一个 id - 1。本次取 ids 中 ≤ start_id 的部分。
    let from_idx = ids
        .iter()
        .position(|id| *id <= start_id)
        .unwrap_or(ids.len());
    let take_ids: Vec<u64> = ids
        .iter()
        .skip(from_idx)
        .take(count as usize)
        .copied()
        .collect();
    let next_idx = from_idx + take_ids.len();

    for id in take_ids {
        if is_organization_manage_proposal(id) {
            // 防御性过滤：多签管理提案不属于治理机构列表，详情页不展示。
            continue;
        }
        match fetch_proposal_meta(id) {
            Ok(Some(meta)) => {
                let display = match fetch_proposal_display(id, &meta) {
                    Ok(v) => v,
                    Err(_) => ProposalDisplayInfo {
                        summary: "(详情查询失败)".to_string(),
                        status: meta.status,
                        status_label: status_label(meta.status).to_string(),
                    },
                };
                let cid_full_name = resolve_cid_full_name(meta.institution_hex.as_deref());
                let display_meta = fetch_proposal_display_id(id).ok().flatten();
                items.push(ProposalListItem {
                    proposal_id: id,
                    display_id: format_proposal_id(id, display_meta.as_ref()),
                    kind: meta.kind,
                    kind_label: kind_label(meta.kind).to_string(),
                    stage: meta.stage,
                    stage_label: stage_label(meta.stage).to_string(),
                    status: display.status,
                    status_label: display.status_label,
                    cid_full_name,
                    summary: display.summary,
                });
            }
            Ok(None) => {} // 提案不存在,跳过
            Err(e) => {
                warnings.push(format!("查询提案 {id} 失败: {e}"));
            }
        }
    }

    let has_more = next_idx < ids.len();

    Ok(ProposalPageResult {
        items,
        has_more,
        warning: if warnings.is_empty() {
            None
        } else {
            Some(warnings.join("；"))
        },
    })
}

/// 查询机构的活跃提案 ID 列表。
pub fn fetch_active_proposal_ids(cid_number: &str) -> Result<Vec<u64>, String> {
    let institution_account = institution_account_from_cid(cid_number)?;
    let key = storage_keys::map_key(
        "VotingEngine",
        "ActiveProposalsByInstitution",
        &institution_account,
    );
    // 中文注释(ADR-017):活跃提案索引按 finalized 口径读取,避免 best 漂移漏列。
    match chain_query::fetch_finalized_storage(&key)? {
        None => Ok(Vec::new()),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            decode_u64_vec(&data)
        }
    }
}

// ──── 内部查询 ────

fn fetch_proposal_meta(proposal_id: u64) -> Result<Option<ProposalMeta>, String> {
    let key = storage_keys::map_key("VotingEngine", "Proposals", &proposal_id.to_le_bytes());
    // 中文注释(ADR-017):提案元数据(含 status)按 finalized 口径读取,禁止 best。
    match chain_query::fetch_finalized_storage(&key)? {
        None => Ok(None),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            Ok(decode_proposal_meta(proposal_id, &data))
        }
    }
}

fn fetch_proposal_data_raw(proposal_id: u64) -> Result<Option<Vec<u8>>, String> {
    let key = storage_keys::map_key("VotingEngine", "ProposalData", &proposal_id.to_le_bytes());
    // 中文注释:提案动作里包含金额字段,详情和摘要按 finalized 口径展示。
    match chain_query::fetch_finalized_storage(&key)? {
        None => Ok(None),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            Ok(Some(data))
        }
    }
}

/// 按优先级依次查询所有提案动作来源,返回命中的第一个业务动作。
///
/// 查找顺序(命中即返回,不重复查询):
/// 1. `VotingEngine::ProposalData`(业务模块/升级/发行/销毁,按 kind 分流)
/// 2. 业务模块独立 storage 动作
/// 3. 全部未命中 → [`ProposalAction::Unknown`]
///
/// 常见提案(业务模块/升级/销毁/发行)1 次 RPC 即可命中;少量业务 detail
/// 存在独立 pallet 存储,会多查 1~2 次,但这几类提案频率极低。
/// 原"省储行费率提案"(`RateProposalActions`)已在 Step 2b-iv-b 随老省储行
/// pallet Call 一起下线,此处不再枚举。
fn resolve_proposal_action(
    proposal_id: u64,
    meta: &ProposalMeta,
) -> Result<ProposalAction, String> {
    // ── Step 1:尝试从 VotingEngine::ProposalData 解码 ──
    if let Some(raw) = fetch_proposal_data_raw(proposal_id)? {
        if !raw.is_empty() {
            // ProposalData 存储为 BoundedVec<u8>:Compact<len> + bytes
            let (vec_len, len_bytes) = read_compact_u32(&raw, 0)?;
            let offset = len_bytes;
            if offset + vec_len as usize <= raw.len() {
                let data = &raw[offset..offset + vec_len as usize];
                if meta.kind == 1 {
                    // 联合投票:先 runtime 升级,再决议发行
                    if let Some(detail) = decode_runtime_upgrade_action(proposal_id, data) {
                        return Ok(ProposalAction::RuntimeUpgrade(Box::new(detail)));
                    }
                    if let Some(detail) = decode_resolution_issuance_action(proposal_id, data) {
                        return Ok(ProposalAction::ResolutionIssuance(Box::new(detail)));
                    }
                } else {
                    // 内部投票:先交给业务模块适配层识别,再识别治理侧销毁。
                    if let Some(action) =
                        proposal_business::decode_internal_proposal_data_action(proposal_id, data)
                    {
                        return Ok(ProposalAction::Business(Box::new(action)));
                    }
                    if let Some(detail) = decode_resolution_destroy_action(proposal_id, data) {
                        return Ok(ProposalAction::ResolutionDestroy(Box::new(detail)));
                    }
                }
            }
        }
    }

    // ── Step 2:业务模块独立 storage ──
    if let Some(action) = proposal_business::fetch_stored_action(proposal_id)? {
        return Ok(ProposalAction::Business(Box::new(action)));
    }

    // Step 3:全部未命中
    Ok(ProposalAction::Unknown)
}

fn fetch_internal_tally(proposal_id: u64) -> Result<VoteTally, String> {
    let key = storage_keys::map_key(
        "InternalVote",
        "InternalTallies",
        &proposal_id.to_le_bytes(),
    );
    // 中文注释(ADR-017):投票计数按 finalized 口径读取,禁止 best。
    match chain_query::fetch_finalized_storage(&key)? {
        None => Ok(VoteTally { yes: 0, no: 0 }),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.len() < 8 {
                return Ok(VoteTally { yes: 0, no: 0 });
            }
            let yes = u32::from_le_bytes(
                data[0..4]
                    .try_into()
                    .map_err(|_| "SCALE 数据长度不足".to_string())?,
            );
            let no = u32::from_le_bytes(
                data[4..8]
                    .try_into()
                    .map_err(|_| "SCALE 数据长度不足".to_string())?,
            );
            Ok(VoteTally { yes, no })
        }
    }
}

fn fetch_joint_tally(proposal_id: u64) -> Result<JointVoteTally, String> {
    let key = storage_keys::map_key("JointVote", "JointTallies", &proposal_id.to_le_bytes());
    // 中文注释(ADR-017):投票计数按 finalized 口径读取,禁止 best。
    match chain_query::fetch_finalized_storage(&key)? {
        None => Ok(JointVoteTally { yes: 0, no: 0 }),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.len() < 8 {
                return Ok(JointVoteTally { yes: 0, no: 0 });
            }
            let yes = u32::from_le_bytes(
                data[0..4]
                    .try_into()
                    .map_err(|_| "SCALE 数据长度不足".to_string())?,
            );
            let no = u32::from_le_bytes(
                data[4..8]
                    .try_into()
                    .map_err(|_| "SCALE 数据长度不足".to_string())?,
            );
            Ok(JointVoteTally { yes, no })
        }
    }
}

fn fetch_referendum_tally(proposal_id: u64) -> Result<ReferendumVoteTally, String> {
    let key = storage_keys::map_key("JointVote", "ReferendumTallies", &proposal_id.to_le_bytes());
    // 中文注释(ADR-017):投票计数按 finalized 口径读取,禁止 best。
    match chain_query::fetch_finalized_storage(&key)? {
        None => Ok(ReferendumVoteTally { yes: 0, no: 0 }),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.len() < 16 {
                return Ok(ReferendumVoteTally { yes: 0, no: 0 });
            }
            let yes = u64::from_le_bytes(
                data[0..8]
                    .try_into()
                    .map_err(|_| "SCALE 数据长度不足".to_string())?,
            );
            let no = u64::from_le_bytes(
                data[8..16]
                    .try_into()
                    .map_err(|_| "SCALE 数据长度不足".to_string())?,
            );
            Ok(ReferendumVoteTally { yes, no })
        }
    }
}

// ──── SCALE 解码 ────

fn decode_hex_storage(hex_str: &str) -> Result<Vec<u8>, String> {
    let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(clean).map_err(|e| format!("hex 解码失败: {e}"))
}

fn decode_proposal_meta(proposal_id: u64, data: &[u8]) -> Option<ProposalMeta> {
    if data.len() < 3 {
        return None;
    }
    let kind = data[0];
    let stage = data[1];
    let status = data[2];

    let mut offset = 3;

    // internal_code: Option<InstitutionCode>([u8;4])
    let internal_code = if offset < data.len() && data[offset] == 1 {
        offset += 1;
        if offset + 4 <= data.len() {
            let code: InstitutionCode = data[offset..offset + 4].try_into().ok()?;
            offset += 4;
            Some(code)
        } else {
            None
        }
    } else {
        offset += 1; // skip 0x00 (None)
        None
    };

    // internal_institution: Option<AccountId32>
    let institution_hex = if offset < data.len() && data[offset] == 1 {
        offset += 1;
        if offset + 32 <= data.len() {
            Some(hex::encode(&data[offset..offset + 32]))
        } else {
            None
        }
    } else {
        None
    };

    Some(ProposalMeta {
        proposal_id,
        kind,
        stage,
        status,
        internal_code,
        institution_hex,
    })
}

fn decode_runtime_upgrade_action(proposal_id: u64, data: &[u8]) -> Option<RuntimeUpgradeDetail> {
    // 跳过 MODULE_TAG("rt-upg":6)
    let tag = TAG_RUNTIME_UPGRADE;
    if data.len() < tag.len() || &data[..tag.len()] != tag {
        return None;
    }
    let mut offset = tag.len();

    // proposer: [u8;32]
    if offset + 32 > data.len() {
        return None;
    }
    let proposer_hex = hex::encode(&data[offset..offset + 32]);
    offset += 32;

    // reason: Vec<u8>
    let (reason_len, reason_len_size) = read_compact_u32(data, offset).ok()?;
    offset += reason_len_size;
    if offset + reason_len as usize > data.len() {
        return None;
    }
    let reason = String::from_utf8_lossy(&data[offset..offset + reason_len as usize]).to_string();
    offset += reason_len as usize;

    // code_hash: [u8;32]
    if offset + 32 > data.len() {
        return None;
    }
    let code_hash_hex = hex::encode(&data[offset..offset + 32]);
    offset += 32;

    // 中文注释：协议升级摘要不保存业务状态，真实状态只读 VotingEngine::Proposals.status。
    if offset != data.len() {
        return None;
    }

    Some(RuntimeUpgradeDetail {
        proposal_id,
        proposer_hex,
        reason,
        code_hash_hex,
    })
}

fn decode_resolution_issuance_action(
    proposal_id: u64,
    data: &[u8],
) -> Option<ResolutionIssuanceDetail> {
    // SCALE 布局：MODULE_TAG("res-iss":7) + proposer(32) + reason(Compact+bytes) + total_amount(u128:16)
    //            + allocations(Compact<count> + N × (recipient(32) + amount(u128:16)))
    let tag = TAG_RESOLUTION_ISSUANCE;
    if data.len() < tag.len() || &data[..tag.len()] != tag {
        return None;
    }
    let mut offset = tag.len();

    // proposer: [u8;32]
    if offset + 32 > data.len() {
        return None;
    }
    let proposer_hex = hex::encode(&data[offset..offset + 32]);
    offset += 32;

    // reason: Vec<u8>
    let (reason_len, reason_len_size) = read_compact_u32(data, offset).ok()?;
    offset += reason_len_size;
    if offset + reason_len as usize > data.len() {
        return None;
    }
    let reason = String::from_utf8_lossy(&data[offset..offset + reason_len as usize]).to_string();
    offset += reason_len as usize;

    // total_amount: u128 (16 bytes LE)
    if offset + 16 > data.len() {
        return None;
    }
    let total_amount = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
    offset += 16;

    // allocations: Vec<RecipientAmount>
    let (alloc_count, alloc_count_size) = read_compact_u32(data, offset).ok()?;
    offset += alloc_count_size;

    let mut allocations = Vec::new();
    for _ in 0..alloc_count {
        if offset + 32 + 16 > data.len() {
            return None;
        }
        let recipient_hex = hex::encode(&data[offset..offset + 32]);
        offset += 32;
        let amount = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);
        offset += 16;
        allocations.push(IssuanceAllocationItem {
            recipient_hex,
            amount_fen: amount.to_string(),
        });
    }

    Some(ResolutionIssuanceDetail {
        proposal_id,
        proposer_hex,
        reason,
        total_amount_fen: total_amount.to_string(),
        allocations,
    })
}

fn decode_resolution_destroy_action(
    proposal_id: u64,
    data: &[u8],
) -> Option<ResolutionDestroyDetail> {
    // SCALE 布局：MODULE_TAG("res-dst":7) + institution(AccountId32) + amount(u128:16)
    let tag = TAG_RESOLUTION_DESTROY;
    if data.len() < tag.len() + 32 + 16 || &data[..tag.len()] != tag {
        return None;
    }
    let mut offset = tag.len();

    let institution_hex = hex::encode(&data[offset..offset + 32]);
    offset += 32;

    let amount = u128::from_le_bytes(data[offset..offset + 16].try_into().ok()?);

    Some(ResolutionDestroyDetail {
        proposal_id,
        institution_hex,
        amount_fen: amount.to_string(),
    })
}

fn decode_u64_vec(data: &[u8]) -> Result<Vec<u64>, String> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let (count, len_bytes) = read_compact_u32(data, 0)?;
    let mut offset = len_bytes;
    let mut ids = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if offset + 8 > data.len() {
            break;
        }
        let val = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| "SCALE 数据长度不足".to_string())?,
        );
        ids.push(val);
        offset += 8;
    }
    Ok(ids)
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

// ──── 工具函数 ────

/// 提案展示号格式化(双层 ID v1):
///   主键 `proposal_id` 是单调 u64,与展示号解耦。展示号通过链上
///   `ProposalDisplayId[id] = ProposalDisplayMeta { year, seq_in_year }` 反查。
///
/// 渲染格式:`2026000123`(年份 + 6 位补零序号);seq 突破 6 位时自动扩展。
/// `display_meta=None` 时(理论不该发生)fallback 到 `#<u64>` 形式避免空字符串。
fn format_proposal_id(id: u64, display_meta: Option<&ProposalDisplayMeta>) -> String {
    match display_meta {
        Some(meta) => format!("{}{:06}", meta.year, meta.seq_in_year),
        None => format!("#{id}"),
    }
}

/// 查询 `VotingEngine::ProposalDisplayId[id]` → `ProposalDisplayMeta { year:u16, seq_in_year:u32 }`。
/// 6 字节 SCALE:u16 LE + u32 LE。
pub fn fetch_proposal_display_id(proposal_id: u64) -> Result<Option<ProposalDisplayMeta>, String> {
    let key = storage_keys::map_key(
        "VotingEngine",
        "ProposalDisplayId",
        &proposal_id.to_le_bytes(),
    );
    // 中文注释(ADR-017):展示号反查按 finalized 口径读取,禁止 best。
    let hex_value = match chain_query::fetch_finalized_storage(&key)? {
        Some(s) => s,
        None => return Ok(None),
    };
    let raw = hex::decode(hex_value.trim_start_matches("0x"))
        .map_err(|e| format!("解析 ProposalDisplayId hex 失败: {e}"))?;
    if raw.len() < 6 {
        return Ok(None);
    }
    let year = u16::from_le_bytes([raw[0], raw[1]]);
    let seq_in_year = u32::from_le_bytes([raw[2], raw[3], raw[4], raw[5]]);
    Ok(Some(ProposalDisplayMeta { year, seq_in_year }))
}

/// 通用反向索引迭代器:列举 `StorageDoubleMap<_, Twox64Concat, K1, Twox64Concat, u64, ()>`
/// 在指定 K1 下的所有 proposal_id。每条 key 末 8 字节 = u64 LE = proposal_id。
fn fetch_proposal_ids_by_index(storage_name: &str, key1: &[u8]) -> Result<Vec<u64>, String> {
    let prefix = storage_keys::twox64_concat_prefix("VotingEngine", storage_name, key1);
    // 中文注释(ADR-017):反向索引列举按 finalized 口径,best 漂移会列出半新半旧 key 集。
    let keys = chain_query::fetch_finalized_keys_paged(&prefix, 1000, None)?;
    let mut ids = Vec::with_capacity(keys.len());
    for s in &keys {
        let bytes = match hex::decode(s.trim_start_matches("0x")) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if bytes.len() < 8 {
            continue;
        }
        let mut tail = [0u8; 8];
        tail.copy_from_slice(&bytes[bytes.len() - 8..]);
        ids.push(u64::from_le_bytes(tail));
    }
    Ok(ids)
}

/// 反向索引:`ProposalsByOrg[institution_code]` → 该机构码下所有 proposal_id。
///
/// 链上第一腿 key 已从 `org: u8`(1 字节)改为 `InstitutionCode`([u8;4]),
/// `institution_code` 入参是 4 字符机构码字符串(如 "NRC"/"CGOV")。
pub fn fetch_proposals_by_institution_code(institution_code: &str) -> Result<Vec<u64>, String> {
    let code = primitives::institution_code::code_bytes(institution_code.trim());
    fetch_proposal_ids_by_index("ProposalsByOrg", &code)
}

/// 反向索引:`ProposalsByInstitution[account]` → 本机构多签账户所有 proposal_id。
pub fn fetch_proposals_by_institution(cid_number: &str) -> Result<Vec<u64>, String> {
    let institution_account = institution_account_from_cid(cid_number)?;
    fetch_proposal_ids_by_index("ProposalsByInstitution", &institution_account)
}

/// 反向索引:`ProposalsByOwner[module_tag]` → 该业务模块所有 proposal_id。
/// `module_tag` 入参为 BoundedVec<u8> 的 SCALE 编码体(Compact<len> + bytes)。
pub fn fetch_proposals_by_owner(module_tag_scale: &[u8]) -> Result<Vec<u64>, String> {
    fetch_proposal_ids_by_index("ProposalsByOwner", module_tag_scale)
}

fn kind_label(kind: u8) -> &'static str {
    match kind {
        0 => "内部投票",
        1 => "联合投票",
        _ => "未知",
    }
}

fn stage_label(stage: u8) -> &'static str {
    match stage {
        0 => "内部阶段",
        1 => "联合阶段",
        2 => "联合公投阶段",
        _ => "未知",
    }
}

fn status_label(status: u8) -> &'static str {
    match status {
        0 => "投票中",
        1 => "已通过",
        2 => "已否决",
        3 => "已执行",
        4 => "执行失败",
        _ => "未知",
    }
}

/// 从机构多签 AccountId32 hex 反查机构名称。
fn resolve_cid_full_name(institution_hex: Option<&str>) -> Option<String> {
    let hex_str = institution_hex?;
    let bytes = hex::decode(hex_str).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    super::registry::find_institution_by_main_account(&bytes).map(|item| item.name().to_string())
}

/// 列表卡片展示信息:一次解析,按动作变体生成 summary + votingengine status。
///
/// 与 [`fetch_proposal_full`] 共用 [`resolve_proposal_action`],保证两条路径对同一提案看到一致内容。
fn fetch_proposal_display(
    proposal_id: u64,
    meta: &ProposalMeta,
) -> Result<ProposalDisplayInfo, String> {
    let action = resolve_proposal_action(proposal_id, meta)?;
    let (summary, status, status_label_s) = match action {
        ProposalAction::Business(action) => (
            proposal_business::format_summary(&action, |institution_hex| {
                resolve_cid_full_name(Some(institution_hex))
            }),
            meta.status,
            status_label(meta.status).to_string(),
        ),
        ProposalAction::RuntimeUpgrade(d) => (
            format_runtime_upgrade_summary(&d),
            meta.status,
            status_label(meta.status).to_string(),
        ),
        ProposalAction::ResolutionIssuance(d) => (
            format_issuance_summary(&d),
            meta.status,
            status_label(meta.status).to_string(),
        ),
        ProposalAction::ResolutionDestroy(d) => (
            format_destroy_summary(&d),
            meta.status,
            status_label(meta.status).to_string(),
        ),
        ProposalAction::FeeRate(d) => (
            format_fee_rate_summary(&d),
            meta.status,
            status_label(meta.status).to_string(),
        ),
        ProposalAction::Unknown => (
            "（无详情数据）".to_string(),
            meta.status,
            status_label(meta.status).to_string(),
        ),
    };
    Ok(ProposalDisplayInfo {
        summary,
        status,
        status_label: status_label_s,
    })
}

// ──── summary 格式化(纯函数,便于单测) ────

/// 安全截断 UTF-8 字符串(按 Unicode 字符数计,避免切中 multi-byte 字符)。
fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn format_runtime_upgrade_summary(d: &RuntimeUpgradeDetail) -> String {
    let reason_short = truncate_chars(&d.reason, 50);
    format!("协议升级：{reason_short}")
}

fn format_issuance_summary(d: &ResolutionIssuanceDetail) -> String {
    let total: u128 = d.total_amount_fen.parse().unwrap_or(0);
    let reason_short = truncate_chars(&d.reason, 30);
    format!(
        "决议发行 {}.{:02} 元（{}条分配）：{reason_short}",
        total / 100,
        total % 100,
        d.allocations.len()
    )
}

fn format_destroy_summary(d: &ResolutionDestroyDetail) -> String {
    let amount: u128 = d.amount_fen.parse().unwrap_or(0);
    let inst_name =
        resolve_cid_full_name(Some(&d.institution_hex)).unwrap_or_else(|| "未知机构".to_string());
    format!(
        "决议销毁 {} 元：{inst_name}",
        signing::format_amount(amount as f64 / 100.0)
    )
}

fn format_fee_rate_summary(d: &FeeRateProposalDetail) -> String {
    let rate_percent = format!("{:.2}%", d.new_rate_bp as f64 / 100.0);
    let inst_name =
        resolve_cid_full_name(Some(&d.institution_hex)).unwrap_or_else(|| "未知机构".to_string());
    format!("费率设置 {rate_percent}：{inst_name}")
}

// ──── 投票状态查询 ────

/// 用户投票状态。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserVoteStatus {
    /// 提案 ID。
    pub proposal_id: u64,
    /// 提案 kind（0=内部, 1=联合）。
    pub kind: u8,
    /// 提案当前阶段。
    pub stage: u8,
    /// 该用户是否已在内部阶段投票：null=未投票, true=赞成, false=反对。
    pub internal_vote: Option<bool>,
    /// 该用户是否已在联合阶段投票（通过机构）：null=未投票, true=赞成, false=反对。
    pub joint_vote: Option<bool>,
}

/// 查询指定用户（管理员公钥）对某提案的投票状态。
pub fn fetch_user_vote_status(
    proposal_id: u64,
    pubkey_hex: &str,
    cid_number: Option<&str>,
) -> Result<UserVoteStatus, String> {
    let meta =
        fetch_proposal_meta(proposal_id)?.ok_or_else(|| format!("提案 {proposal_id} 不存在"))?;

    let pubkey_bytes = hex::decode(pubkey_hex).map_err(|e| format!("公钥解码失败: {e}"))?;

    // 查询内部投票状态（InternalVotesByAccount: DoubleMap<u64, AccountId32> → bool）
    let internal_vote = {
        let key = storage_keys::double_map_key(
            "InternalVote",
            "InternalVotesByAccount",
            &proposal_id.to_le_bytes(),
            &pubkey_bytes,
        );
        fetch_option_bool(&key)?
    };

    // 查询联合投票状态（JointVotesByAdmin: DoubleMap<u64, (InstitutionAccount32 ++ AccountId32)> → bool）
    let joint_vote = if meta.kind == 1 && cid_number.is_some() {
        // cid_number.is_some() 已在上方 if 条件中守卫，此处 expect 不会 panic。
        let institution_account =
            institution_account_from_cid(cid_number.expect("guarded by is_some()"))?;
        let mut composite_key = Vec::with_capacity(32 + 32);
        composite_key.extend_from_slice(&institution_account);
        composite_key.extend_from_slice(&pubkey_bytes);
        let key = storage_keys::double_map_key(
            "JointVote",
            "JointVotesByAdmin",
            &proposal_id.to_le_bytes(),
            &composite_key,
        );
        fetch_option_bool(&key)?
    } else {
        None
    };

    Ok(UserVoteStatus {
        proposal_id,
        kind: meta.kind,
        stage: meta.stage,
        internal_vote,
        joint_vote,
    })
}

/// 查询链上 Option<bool> 存储值。
fn fetch_option_bool(storage_key: &str) -> Result<Option<bool>, String> {
    // 中文注释(ADR-017):投票状态按 finalized 口径读取,禁止 best。
    match chain_query::fetch_finalized_storage(storage_key)? {
        None => Ok(None),
        Some(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.is_empty() {
                Ok(None)
            } else {
                Ok(Some(data[0] == 1))
            }
        }
    }
}

// ──── 单元测试 ────

#[cfg(test)]
mod format_summary_tests {
    use super::*;

    fn compact_bytes_for_test(bytes: &[u8]) -> Vec<u8> {
        assert!(bytes.len() < 64, "test helper only supports short vectors");
        let mut out = Vec::with_capacity(1 + bytes.len());
        out.push((bytes.len() as u8) << 2);
        out.extend_from_slice(bytes);
        out
    }

    #[test]
    fn truncate_chars_keeps_short_input_unchanged() {
        assert_eq!(truncate_chars("hello", 10), "hello");
    }

    #[test]
    fn truncate_chars_appends_ellipsis_when_over_limit() {
        assert_eq!(truncate_chars("0123456789ABCDEF", 8), "01234567…");
    }

    #[test]
    fn truncate_chars_safe_for_multibyte_utf8() {
        // 8 个汉字 ≤ 10 字符上限 → 不截断
        assert_eq!(truncate_chars("中华人民共和国家", 10), "中华人民共和国家");
        // 8 个汉字 > 4 字符上限 → 保留前 4 字符 + …
        assert_eq!(truncate_chars("中华人民共和国家", 4), "中华人民…");
    }

    #[test]
    fn format_runtime_upgrade_summary_truncates_long_reason() {
        let long = "a".repeat(80);
        let d = RuntimeUpgradeDetail {
            proposal_id: 2,
            proposer_hex: String::new(),
            reason: long,
            code_hash_hex: String::new(),
        };
        let summary = format_runtime_upgrade_summary(&d);
        assert!(summary.starts_with("协议升级："));
        assert!(summary.contains("…"));
    }

    #[test]
    fn decode_runtime_upgrade_action_uses_current_summary_layout() {
        let mut data = Vec::from(TAG_RUNTIME_UPGRADE);
        data.extend_from_slice(&[7u8; 32]);
        data.extend_from_slice(&compact_bytes_for_test("升级".as_bytes()));
        data.extend_from_slice(&[9u8; 32]);

        let detail =
            decode_runtime_upgrade_action(10, &data).expect("current layout should decode");
        assert_eq!(detail.proposal_id, 10);
        assert_eq!(detail.reason, "升级");
        assert_eq!(detail.code_hash_hex, "09".repeat(32));

        data.push(0);
        assert!(
            decode_runtime_upgrade_action(10, &data).is_none(),
            "old runtime-upgrade summary status field must not be accepted"
        );
    }

    #[test]
    fn format_issuance_summary_shows_count_and_amount() {
        let d = ResolutionIssuanceDetail {
            proposal_id: 3,
            proposer_hex: String::new(),
            reason: "春节福利".to_string(),
            total_amount_fen: "100000".to_string(), // 1000 元
            allocations: vec![
                IssuanceAllocationItem {
                    recipient_hex: "a".repeat(64),
                    amount_fen: "50000".to_string(),
                },
                IssuanceAllocationItem {
                    recipient_hex: "b".repeat(64),
                    amount_fen: "50000".to_string(),
                },
            ],
        };
        assert_eq!(
            format_issuance_summary(&d),
            "决议发行 1000.00 元（2条分配）：春节福利"
        );
    }

    #[test]
    fn format_destroy_summary_falls_back_to_unknown_institution() {
        let d = ResolutionDestroyDetail {
            proposal_id: 4,
            institution_hex: "00".repeat(32), // 全零 → 无法反查中文名
            amount_fen: "50000".to_string(),
        };
        assert_eq!(format_destroy_summary(&d), "决议销毁 500.00 元：未知机构");
    }

    #[test]
    fn format_fee_rate_summary_shows_percent() {
        let d = FeeRateProposalDetail {
            proposal_id: 5,
            institution_hex: "00".repeat(32),
            new_rate_bp: 150, // 1.50%
        };
        assert_eq!(format_fee_rate_summary(&d), "费率设置 1.50%：未知机构");
    }

    #[test]
    fn split_action_into_details_maps_each_variant() {
        let detail = RuntimeUpgradeDetail {
            proposal_id: 1,
            proposer_hex: String::new(),
            reason: "升级".to_string(),
            code_hash_hex: String::new(),
        };
        let (dq, ru, ri, rd, fr) =
            split_action_into_details(ProposalAction::RuntimeUpgrade(Box::new(detail)));
        let _ = dq;
        assert!(ru.is_some());
        assert!(ri.is_none() && rd.is_none() && fr.is_none());

        let (_, ru2, _, _, _) = split_action_into_details(ProposalAction::Unknown);
        assert!(ru2.is_none());
    }
}
