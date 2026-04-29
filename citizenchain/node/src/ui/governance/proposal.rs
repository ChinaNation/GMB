// 提案查询：提案列表、详情、投票计数，通过 RPC 读取 VotingEngine 链上存储。

use crate::ui::shared::rpc;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

use super::{signing, storage_keys};

const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// MODULE_TAG 前缀（必须与对应 pallet 保持一致）。
const TAG_TRANSFER: &[u8] = b"dq-xfer";
const TAG_RUNTIME_UPGRADE: &[u8] = b"rt-upg";
const TAG_RESOLUTION_ISSUANCE: &[u8] = b"res-iss";
const TAG_RESOLUTION_DESTROY: &[u8] = b"res-dst";
/// 多签管理提案 TAG — 不属于治理提案，在治理列表中过滤掉。
const TAG_DUOQIAN_MANAGE: &[u8] = b"dq-mgmt";
use crate::ui::shared::constants::RPC_RESPONSE_LIMIT_SMALL;

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(
        method,
        params,
        RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_SMALL,
    )
}

/// 提案元数据（从 VotingEngine::Proposals 解码）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalMeta {
    pub proposal_id: u64,
    /// 0=内部投票, 1=联合投票。
    pub kind: u8,
    /// 0=内部阶段, 1=联合阶段, 2=公民阶段。
    pub stage: u8,
    /// 0=投票中, 1=通过, 2=否决, 3=已执行, 4=执行失败。
    pub status: u8,
    pub internal_org: Option<u8>,
    /// 机构 48 字节 hex（不含 0x）。
    pub institution_hex: Option<String>,
}

/// 转账提案详情（从 VotingEngine::ProposalData 解码）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProposalDetail {
    pub proposal_id: u64,
    /// 机构 48 字节 hex。
    pub institution_hex: String,
    /// 收款人公钥 hex。
    pub beneficiary_hex: String,
    /// 金额（分）。
    pub amount_fen: String,
    pub remark: String,
    /// 提案人公钥 hex。
    pub proposer_hex: String,
}

/// Runtime 升级提案详情（从 VotingEngine::ProposalData 解码）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUpgradeDetail {
    pub proposal_id: u64,
    pub proposer_hex: String,
    pub reason: String,
    pub code_hash_hex: String,
    /// 0=Voting, 1=Passed, 2=Rejected, 3=ExecutionFailed。
    pub status: u8,
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

/// 公民投票计数。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CitizenVoteTally {
    pub yes: u64,
    pub no: u64,
}

/// 提案完整信息（元数据 + 业务详情 + 投票进度）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalFullInfo {
    pub meta: ProposalMeta,
    pub transfer_detail: Option<TransferProposalDetail>,
    pub runtime_upgrade_detail: Option<RuntimeUpgradeDetail>,
    pub fee_rate_detail: Option<FeeRateProposalDetail>,
    pub safety_fund_detail: Option<SafetyFundProposalDetail>,
    pub sweep_detail: Option<SweepProposalDetail>,
    pub resolution_issuance_detail: Option<ResolutionIssuanceDetail>,
    pub resolution_destroy_detail: Option<ResolutionDestroyDetail>,
    pub internal_tally: Option<VoteTally>,
    pub joint_tally: Option<JointVoteTally>,
    pub citizen_tally: Option<CitizenVoteTally>,
    /// 关联机构名称（通过 institutionBytes 反查）。
    pub institution_name: Option<String>,
}

/// 安全基金转账提案详情。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyFundProposalDetail {
    pub proposal_id: u64,
    pub beneficiary_hex: String,
    pub amount_fen: String,
    pub remark: String,
}

/// 手续费划转提案详情。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepProposalDetail {
    pub proposal_id: u64,
    pub institution_hex: String,
    pub amount_fen: String,
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
    pub institution_name: Option<String>,
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
    Transfer(Box<TransferProposalDetail>),
    RuntimeUpgrade(Box<RuntimeUpgradeDetail>),
    ResolutionIssuance(Box<ResolutionIssuanceDetail>),
    ResolutionDestroy(Box<ResolutionDestroyDetail>),
    FeeRate(Box<FeeRateProposalDetail>),
    SafetyFund(Box<SafetyFundProposalDetail>),
    Sweep(Box<SweepProposalDetail>),
    /// 所有可能数据源都查过仍无命中,展示层回退为"无详情数据"。
    Unknown,
}

// ──── 公开查询函数 ────

/// 查询 NextProposalId（VotingEngine 全局递增 ID）。
pub fn fetch_next_proposal_id() -> Result<u64, String> {
    let key = storage_keys::value_key("VotingEngine", "NextProposalId");
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(0),
        Value::String(hex_data) => {
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
        _ => Ok(0),
    }
}

/// 检查提案是否为多签管理提案（创建/关闭多签账户），这类提案不在治理列表中显示。
fn is_duoqian_manage_proposal(proposal_id: u64) -> bool {
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
    let tag = TAG_DUOQIAN_MANAGE;
    offset + tag.len() <= raw.len()
        && (vec_len as usize) >= tag.len()
        && raw[offset..offset + tag.len()] == *tag
}

/// 分页查询提案列表（从 start_id 往前 count 个，按 ID 倒序）。
/// 自动过滤多签管理提案（创建/关闭多签账户），这些在多签账户详情页单独展示。
pub fn fetch_proposal_page(start_id: u64, count: u32) -> Result<ProposalPageResult, String> {
    let mut items = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let min_id = start_id.saturating_sub(count as u64);
    let mut id = start_id;

    while id > min_id {
        match fetch_proposal_meta(id) {
            Ok(Some(meta)) => {
                // 中文注释：多签管理提案（创建/关闭多签账户）不在治理列表中显示。
                if is_duoqian_manage_proposal(id) {
                    if id == 0 {
                        break;
                    }
                    id -= 1;
                    continue;
                }
                // 中文注释：runtime-upgrade 的业务终态保存在 ProposalData，
                // 这里只把它折叠成列表展示状态，避免 UI 把”已否决/执行失败”误显示成”已执行”。
                let display = match fetch_proposal_display(id, &meta) {
                    Ok(v) => v,
                    Err(_) => ProposalDisplayInfo {
                        summary: "（详情查询失败）".to_string(),
                        status: meta.status,
                        status_label: status_label(meta.status).to_string(),
                    },
                };
                let institution_name = resolve_institution_name(meta.institution_hex.as_deref());

                items.push(ProposalListItem {
                    proposal_id: id,
                    display_id: format_proposal_id(id),
                    kind: meta.kind,
                    kind_label: kind_label(meta.kind).to_string(),
                    stage: meta.stage,
                    stage_label: stage_label(meta.stage).to_string(),
                    status: display.status,
                    status_label: display.status_label,
                    institution_name,
                    summary: display.summary,
                });
            }
            Ok(None) => {} // 提案不存在，跳过
            Err(e) => {
                warnings.push(format!("查询提案 {id} 失败: {e}"));
            }
        }
        if id == 0 {
            break;
        }
        id -= 1;
    }

    // 判断是否还有更多
    let has_more = min_id > 0 && {
        let year_start = (start_id / 1_000_000) * 1_000_000;
        min_id > year_start
    };

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
        transfer_detail,
        runtime_upgrade_detail,
        resolution_issuance_detail,
        resolution_destroy_detail,
        fee_rate_detail,
        safety_fund_detail,
        sweep_detail,
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

    let citizen_tally = if meta.kind == 1 && meta.stage >= 2 {
        fetch_citizen_tally(proposal_id).ok()
    } else {
        None
    };

    let institution_name = resolve_institution_name(meta.institution_hex.as_deref());

    Ok(ProposalFullInfo {
        meta,
        transfer_detail,
        runtime_upgrade_detail,
        fee_rate_detail,
        safety_fund_detail,
        sweep_detail,
        resolution_issuance_detail,
        resolution_destroy_detail,
        internal_tally,
        joint_tally,
        citizen_tally,
        institution_name,
    })
}

/// 把 [`ProposalAction`] 展开成 `ProposalFullInfo` 的 7 个 `*_detail` Option。
///
/// 返回元组顺序:transfer / runtime_upgrade / resolution_issuance / resolution_destroy
/// / fee_rate / safety_fund / sweep。未命中的类型全部为 `None`;
/// `Unknown` 时 7 个字段都是 `None`。
#[allow(clippy::type_complexity)]
fn split_action_into_details(
    action: ProposalAction,
) -> (
    Option<TransferProposalDetail>,
    Option<RuntimeUpgradeDetail>,
    Option<ResolutionIssuanceDetail>,
    Option<ResolutionDestroyDetail>,
    Option<FeeRateProposalDetail>,
    Option<SafetyFundProposalDetail>,
    Option<SweepProposalDetail>,
) {
    match action {
        ProposalAction::Transfer(d) => (Some(*d), None, None, None, None, None, None),
        ProposalAction::RuntimeUpgrade(d) => (None, Some(*d), None, None, None, None, None),
        ProposalAction::ResolutionIssuance(d) => (None, None, Some(*d), None, None, None, None),
        ProposalAction::ResolutionDestroy(d) => (None, None, None, Some(*d), None, None, None),
        ProposalAction::FeeRate(d) => (None, None, None, None, Some(*d), None, None),
        ProposalAction::SafetyFund(d) => (None, None, None, None, None, Some(*d), None),
        ProposalAction::Sweep(d) => (None, None, None, None, None, None, Some(*d)),
        ProposalAction::Unknown => (None, None, None, None, None, None, None),
    }
}

/// 分页查询指定机构的所有存在提案（从 start_id 往前，按 ID 倒序）。
///
/// 遍历所有提案 ID，过滤 institution_hex 匹配的记录。
/// 每页最多返回 count 条，has_more 表示是否还有更早的提案。
pub fn fetch_institution_proposal_page(
    shenfen_id: &str,
    start_id: u64,
    count: u32,
) -> Result<ProposalPageResult, String> {
    let institution_hex = hex::encode(storage_keys::shenfen_id_to_fixed48(shenfen_id));
    let mut items = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut id = start_id;
    // 提案 ID 格式为 YYYY000000 起，年份起始下界（含）
    let year_start = (start_id / 1_000_000) * 1_000_000;

    while items.len() < count as usize && id >= year_start {
        match fetch_proposal_meta(id) {
            Ok(Some(meta)) => {
                // 过滤：只保留属于该机构的提案，且排除多签管理提案
                let matches = meta.institution_hex.as_deref() == Some(&institution_hex);
                if matches && !is_duoqian_manage_proposal(id) {
                    let display = match fetch_proposal_display(id, &meta) {
                        Ok(v) => v,
                        Err(_) => ProposalDisplayInfo {
                            summary: "（详情查询失败）".to_string(),
                            status: meta.status,
                            status_label: status_label(meta.status).to_string(),
                        },
                    };
                    let institution_name =
                        resolve_institution_name(meta.institution_hex.as_deref());
                    items.push(ProposalListItem {
                        proposal_id: id,
                        display_id: format_proposal_id(id),
                        kind: meta.kind,
                        kind_label: kind_label(meta.kind).to_string(),
                        stage: meta.stage,
                        stage_label: stage_label(meta.stage).to_string(),
                        status: display.status,
                        status_label: display.status_label,
                        institution_name,
                        summary: display.summary,
                    });
                }
            }
            Ok(None) => {} // 提案不存在，跳过
            Err(e) => {
                warnings.push(format!("查询提案 {id} 失败: {e}"));
            }
        }
        if id == year_start {
            break;
        }
        id -= 1;
    }

    let has_more = id > year_start;

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
pub fn fetch_active_proposal_ids(shenfen_id: &str) -> Result<Vec<u64>, String> {
    let institution_id = storage_keys::shenfen_id_to_fixed48(shenfen_id);
    let key = storage_keys::map_key(
        "VotingEngine",
        "ActiveProposalsByInstitution",
        &institution_id,
    );
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(Vec::new()),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            decode_u64_vec(&data)
        }
        _ => Ok(Vec::new()),
    }
}

// ──── 内部查询 ────

fn fetch_proposal_meta(proposal_id: u64) -> Result<Option<ProposalMeta>, String> {
    let key = storage_keys::map_key(
        "VotingEngine",
        "Proposals",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(None),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            Ok(decode_proposal_meta(proposal_id, &data))
        }
        _ => Ok(None),
    }
}

fn fetch_proposal_data_raw(proposal_id: u64) -> Result<Option<Vec<u8>>, String> {
    let key = storage_keys::map_key(
        "VotingEngine",
        "ProposalData",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(None),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            Ok(Some(data))
        }
        _ => Ok(None),
    }
}

/// 按优先级依次查询所有提案动作来源,返回命中的第一个业务动作。
///
/// 查找顺序(命中即返回,不重复查询):
/// 1. `VotingEngine::ProposalData`(转账/升级/发行/销毁 4 种,按 kind 分流)
/// 2. `DuoqianTransferPow::SafetyFundProposalActions`
/// 3. `DuoqianTransferPow::SweepProposalActions`
/// 4. 全部未命中 → [`ProposalAction::Unknown`]
///
/// 常见提案(转账/升级/销毁/发行)1 次 RPC 即可命中;安全基金/手续费划转
/// 因业务 detail 存在独立 pallet 存储,会多查 1~2 次,但这几类提案频率极低。
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
                    // 内部投票:先转账,再销毁
                    if let Some(detail) = decode_transfer_action(proposal_id, data) {
                        return Ok(ProposalAction::Transfer(Box::new(detail)));
                    }
                    if let Some(detail) = decode_resolution_destroy_action(proposal_id, data) {
                        return Ok(ProposalAction::ResolutionDestroy(Box::new(detail)));
                    }
                }
            }
        }
    }

    // ── Step 2:SafetyFundProposalActions(安全基金) ──
    if let Ok(Some(detail)) = fetch_safety_fund_proposal_action(proposal_id) {
        return Ok(ProposalAction::SafetyFund(Box::new(detail)));
    }

    // ── Step 3:SweepProposalActions(手续费划转) ──
    if let Ok(Some(detail)) = fetch_sweep_proposal_action(proposal_id) {
        return Ok(ProposalAction::Sweep(Box::new(detail)));
    }

    // 4:全部未命中
    Ok(ProposalAction::Unknown)
}

fn fetch_internal_tally(proposal_id: u64) -> Result<VoteTally, String> {
    let key = storage_keys::map_key(
        "VotingEngine",
        "InternalTallies",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(VoteTally { yes: 0, no: 0 }),
        Value::String(hex_data) => {
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
        _ => Ok(VoteTally { yes: 0, no: 0 }),
    }
}

fn fetch_joint_tally(proposal_id: u64) -> Result<JointVoteTally, String> {
    let key = storage_keys::map_key(
        "VotingEngine",
        "JointTallies",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(JointVoteTally { yes: 0, no: 0 }),
        Value::String(hex_data) => {
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
        _ => Ok(JointVoteTally { yes: 0, no: 0 }),
    }
}

fn fetch_citizen_tally(proposal_id: u64) -> Result<CitizenVoteTally, String> {
    let key = storage_keys::map_key(
        "VotingEngine",
        "CitizenTallies",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(CitizenVoteTally { yes: 0, no: 0 }),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.len() < 16 {
                return Ok(CitizenVoteTally { yes: 0, no: 0 });
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
            Ok(CitizenVoteTally { yes, no })
        }
        _ => Ok(CitizenVoteTally { yes: 0, no: 0 }),
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

    // internal_org: Option<u8>
    let internal_org = if offset < data.len() && data[offset] == 1 {
        offset += 1;
        if offset < data.len() {
            let v = data[offset];
            offset += 1;
            Some(v)
        } else {
            None
        }
    } else {
        offset += 1; // skip 0x00 (None)
        None
    };

    // internal_institution: Option<[u8;48]>
    let institution_hex = if offset < data.len() && data[offset] == 1 {
        offset += 1;
        if offset + 48 <= data.len() {
            Some(hex::encode(&data[offset..offset + 48]))
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
        internal_org,
        institution_hex,
    })
}

fn decode_transfer_action(proposal_id: u64, data: &[u8]) -> Option<TransferProposalDetail> {
    // MODULE_TAG("dq-xfer":7) + institution: [u8;48] + beneficiary: [u8;32] + amount: u128(16)
    // + remark: Vec<u8> + proposer: [u8;32]
    let tag = TAG_TRANSFER;
    if data.len() < tag.len() + 48 + 32 + 16 + 1 + 32 {
        return None;
    }
    if &data[..tag.len()] != tag {
        return None;
    }
    let mut offset = tag.len();

    let institution_hex = hex::encode(&data[offset..offset + 48]);
    offset += 48;

    let beneficiary_hex = hex::encode(&data[offset..offset + 32]);
    offset += 32;

    let amount_bytes: [u8; 16] = data[offset..offset + 16].try_into().ok()?;
    let amount_fen = u128::from_le_bytes(amount_bytes);
    offset += 16;

    // remark: Vec<u8>
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

    // status: u8
    if offset >= data.len() {
        return None;
    }
    let status = data[offset];

    Some(RuntimeUpgradeDetail {
        proposal_id,
        proposer_hex,
        reason,
        code_hash_hex,
        status,
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
    // SCALE 布局：MODULE_TAG("res-dst":7) + institution(48) + amount(u128:16)
    let tag = TAG_RESOLUTION_DESTROY;
    if data.len() < tag.len() + 48 + 16 || &data[..tag.len()] != tag {
        return None;
    }
    let mut offset = tag.len();

    let institution_hex = hex::encode(&data[offset..offset + 48]);
    offset += 48;

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

/// 提案 ID 格式化：2026000001 → "2026#1"。
fn format_proposal_id(id: u64) -> String {
    let year = id / 1_000_000;
    let counter = id % 1_000_000;
    format!("{year}#{counter}")
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
        2 => "公民阶段",
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

/// 从 48 字节机构 hex 反查机构名称。
fn resolve_institution_name(institution_hex: Option<&str>) -> Option<String> {
    let hex_str = institution_hex?;
    let bytes = hex::decode(hex_str).ok()?;
    if bytes.len() != 48 {
        return None;
    }
    // 截取非零部分作为 UTF-8 字符串
    let end = bytes
        .iter()
        .rposition(|&b| b != 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    let shenfen_id = std::str::from_utf8(&bytes[..end]).ok()?;
    // 在静态数据中查找
    super::find_institution_name(shenfen_id)
}

/// 列表卡片展示信息:一次解析,按动作变体生成 summary + status(runtime 升级需折叠状态)。
///
/// 与 [`fetch_proposal_full`] 共用 [`resolve_proposal_action`],保证两条路径对同一提案看到一致内容。
fn fetch_proposal_display(
    proposal_id: u64,
    meta: &ProposalMeta,
) -> Result<ProposalDisplayInfo, String> {
    let action = resolve_proposal_action(proposal_id, meta)?;
    let (summary, status, status_label_s) = match action {
        ProposalAction::Transfer(d) => (
            format_transfer_summary(&d),
            meta.status,
            status_label(meta.status).to_string(),
        ),
        ProposalAction::RuntimeUpgrade(d) => {
            // runtime 升级的业务终态由 detail.status 决定,需折叠到列表展示状态,
            // 避免 UI 把"已否决/执行失败"误显示为"已执行"。
            let (display_status, display_label) =
                runtime_upgrade_display_status(d.status, meta.status);
            (
                format_runtime_upgrade_summary(&d),
                display_status,
                display_label.to_string(),
            )
        }
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
        ProposalAction::SafetyFund(d) => (
            format_safety_fund_summary(&d),
            meta.status,
            status_label(meta.status).to_string(),
        ),
        ProposalAction::Sweep(d) => (
            format_sweep_summary(&d),
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

fn format_transfer_summary(d: &TransferProposalDetail) -> String {
    let amount: u128 = d.amount_fen.parse().unwrap_or(0);
    let remark_short = truncate_chars(&d.remark, 30);
    format!(
        "转账 {} 元：{remark_short}",
        signing::format_amount(amount as f64 / 100.0)
    )
}

fn format_runtime_upgrade_summary(d: &RuntimeUpgradeDetail) -> String {
    let reason_short = truncate_chars(&d.reason, 50);
    format!("运行时升级：{reason_short}")
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
    let inst_name = resolve_institution_name(Some(&d.institution_hex))
        .unwrap_or_else(|| "未知机构".to_string());
    format!(
        "决议销毁 {} 元：{inst_name}",
        signing::format_amount(amount as f64 / 100.0)
    )
}

fn format_fee_rate_summary(d: &FeeRateProposalDetail) -> String {
    let rate_percent = format!("{:.2}%", d.new_rate_bp as f64 / 100.0);
    let inst_name = resolve_institution_name(Some(&d.institution_hex))
        .unwrap_or_else(|| "未知机构".to_string());
    format!("费率设置 {rate_percent}：{inst_name}")
}

fn format_safety_fund_summary(d: &SafetyFundProposalDetail) -> String {
    let amount: u128 = d.amount_fen.parse().unwrap_or(0);
    format!(
        "安全基金转账 {} 元",
        signing::format_amount(amount as f64 / 100.0)
    )
}

fn format_sweep_summary(d: &SweepProposalDetail) -> String {
    let amount: u128 = d.amount_fen.parse().unwrap_or(0);
    let inst_name = resolve_institution_name(Some(&d.institution_hex))
        .unwrap_or_else(|| "未知机构".to_string());
    format!(
        "手续费划转 {} 元：{inst_name}",
        signing::format_amount(amount as f64 / 100.0)
    )
}

fn runtime_upgrade_display_status(runtime_status: u8, fallback_status: u8) -> (u8, &'static str) {
    match runtime_status {
        0 => (0, status_label(0)),
        1 => (3, status_label(3)),
        2 => (2, status_label(2)),
        3 => (4, status_label(4)),
        _ => (fallback_status, status_label(fallback_status)),
    }
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
    shenfen_id: Option<&str>,
) -> Result<UserVoteStatus, String> {
    let meta =
        fetch_proposal_meta(proposal_id)?.ok_or_else(|| format!("提案 {proposal_id} 不存在"))?;

    let pubkey_bytes = hex::decode(pubkey_hex).map_err(|e| format!("公钥解码失败: {e}"))?;

    // 查询内部投票状态（InternalVotesByAccount: DoubleMap<u64, AccountId32> → bool）
    let internal_vote = {
        let key = storage_keys::double_map_key(
            "VotingEngine",
            "InternalVotesByAccount",
            &proposal_id.to_le_bytes(),
            &pubkey_bytes,
        );
        fetch_option_bool(&key)?
    };

    // 查询联合投票状态（JointVotesByAdmin: DoubleMap<u64, (InstitutionId48 ++ AccountId32)> → bool）
    let joint_vote = if meta.kind == 1 && shenfen_id.is_some() {
        // shenfen_id.is_some() 已在上方 if 条件中守卫，此处 expect 不会 panic。
        let institution_id =
            storage_keys::shenfen_id_to_fixed48(shenfen_id.expect("guarded by is_some()"));
        let mut composite_key = Vec::with_capacity(48 + 32);
        composite_key.extend_from_slice(&institution_id);
        composite_key.extend_from_slice(&pubkey_bytes);
        let key = storage_keys::double_map_key(
            "VotingEngine",
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
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(storage_key.to_string())]),
    )?;
    match result {
        Value::Null => Ok(None),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.is_empty() {
                Ok(None)
            } else {
                Ok(Some(data[0] == 1))
            }
        }
        _ => Ok(None),
    }
}

// ──── 费率提案 ────

/// 费率提案详情。
struct RateProposalActionDetail {
    institution_hex: String,
    new_rate_bp: u32,
}

/// 从链上 SafetyFundProposalActions 存储查询安全基金提案数据。
fn fetch_safety_fund_proposal_action(
    proposal_id: u64,
) -> Result<Option<SafetyFundProposalDetail>, String> {
    let key = storage_keys::map_key(
        "DuoqianTransferPow",
        "SafetyFundProposalActions",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(None),
        Value::String(hex_data) => {
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
            // remark: compact length + bytes
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
        _ => Ok(None),
    }
}

/// 从链上 DuoqianTransferPow::SweepProposalActions 存储查询手续费划转提案数据。
fn fetch_sweep_proposal_action(proposal_id: u64) -> Result<Option<SweepProposalDetail>, String> {
    let key = storage_keys::map_key(
        "DuoqianTransferPow",
        "SweepProposalActions",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(None),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            // SweepAction: institution([u8;48]) + amount(u128=16)
            if data.len() < 64 {
                return Ok(None);
            }
            let institution_hex = hex::encode(&data[..48]);
            let amount_fen = {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&data[48..64]);
                u128::from_le_bytes(bytes)
            };
            Ok(Some(SweepProposalDetail {
                proposal_id,
                institution_hex,
                amount_fen: amount_fen.to_string(),
            }))
        }
        _ => Ok(None),
    }
}

// ──── 单元测试 ────

#[cfg(test)]
mod format_summary_tests {
    use super::*;

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
    fn format_transfer_summary_basic() {
        let d = TransferProposalDetail {
            proposal_id: 1,
            institution_hex: String::new(),
            beneficiary_hex: String::new(),
            amount_fen: "23400".to_string(), // 234.00 元
            remark: "办公采购".to_string(),
            proposer_hex: String::new(),
        };
        assert_eq!(format_transfer_summary(&d), "转账 234.00 元：办公采购");
    }

    #[test]
    fn format_transfer_summary_truncates_long_remark() {
        let long = "一二三四五六七八九十".repeat(4); // 40 个汉字
        let d = TransferProposalDetail {
            proposal_id: 1,
            institution_hex: String::new(),
            beneficiary_hex: String::new(),
            amount_fen: "100".to_string(),
            remark: long,
            proposer_hex: String::new(),
        };
        let summary = format_transfer_summary(&d);
        assert!(summary.contains("…"), "超过 30 字符时应带省略号: {summary}");
    }

    #[test]
    fn format_runtime_upgrade_summary_truncates_long_reason() {
        let long = "a".repeat(80);
        let d = RuntimeUpgradeDetail {
            proposal_id: 2,
            proposer_hex: String::new(),
            reason: long,
            code_hash_hex: String::new(),
            status: 0,
        };
        let summary = format_runtime_upgrade_summary(&d);
        assert!(summary.starts_with("运行时升级："));
        assert!(summary.contains("…"));
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
            institution_hex: "00".repeat(48), // 全零 → 无法反查中文名
            amount_fen: "50000".to_string(),
        };
        assert_eq!(format_destroy_summary(&d), "决议销毁 500.00 元：未知机构");
    }

    #[test]
    fn format_fee_rate_summary_shows_percent() {
        let d = FeeRateProposalDetail {
            proposal_id: 5,
            institution_hex: "00".repeat(48),
            new_rate_bp: 150, // 1.50%
        };
        assert_eq!(format_fee_rate_summary(&d), "费率设置 1.50%：未知机构");
    }

    #[test]
    fn format_safety_fund_summary_shows_amount() {
        let d = SafetyFundProposalDetail {
            proposal_id: 6,
            beneficiary_hex: String::new(),
            amount_fen: "12345".to_string(), // 123.45 元
            remark: String::new(),
        };
        assert_eq!(format_safety_fund_summary(&d), "安全基金转账 123.45 元");
    }

    #[test]
    fn format_sweep_summary_shows_amount_and_unknown_institution() {
        let d = SweepProposalDetail {
            proposal_id: 7,
            institution_hex: "00".repeat(48),
            amount_fen: "999900".to_string(), // 9999 元
        };
        assert_eq!(format_sweep_summary(&d), "手续费划转 9,999.00 元：未知机构");
    }

    #[test]
    fn split_action_into_details_maps_each_variant() {
        let boxed = |a: u128| TransferProposalDetail {
            proposal_id: 1,
            institution_hex: String::new(),
            beneficiary_hex: String::new(),
            amount_fen: a.to_string(),
            remark: String::new(),
            proposer_hex: String::new(),
        };
        let (t, ru, ri, rd, fr, sf, sw) =
            split_action_into_details(ProposalAction::Transfer(Box::new(boxed(1))));
        assert!(t.is_some());
        assert!(ru.is_none() && ri.is_none() && rd.is_none());
        assert!(fr.is_none() && sf.is_none() && sw.is_none());

        let (t2, _, _, _, _, _, _) = split_action_into_details(ProposalAction::Unknown);
        assert!(t2.is_none());
    }
}
