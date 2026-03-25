// 提案查询：提案列表、详情、投票计数，通过 RPC 读取 VotingEngineSystem 链上存储。

use crate::shared::rpc;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

use super::storage_keys;

const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RPC_RESPONSE_BYTES: u64 = 1024 * 1024;

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(method, params, RPC_REQUEST_TIMEOUT, MAX_RPC_RESPONSE_BYTES)
}

/// 提案元数据（从 VotingEngineSystem::Proposals 解码）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalMeta {
    pub proposal_id: u64,
    /// 0=内部投票, 1=联合投票。
    pub kind: u8,
    /// 0=内部阶段, 1=联合阶段, 2=公民阶段。
    pub stage: u8,
    /// 0=投票中, 1=通过, 2=否决。
    pub status: u8,
    pub internal_org: Option<u8>,
    /// 机构 48 字节 hex（不含 0x）。
    pub institution_hex: Option<String>,
}

/// 转账提案详情（从 VotingEngineSystem::ProposalData 解码）。
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

/// Runtime 升级提案详情（从 VotingEngineSystem::ProposalData 解码）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUpgradeDetail {
    pub proposal_id: u64,
    pub proposer_hex: String,
    pub reason: String,
    pub code_hash_hex: String,
    pub has_code: bool,
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
    pub internal_tally: Option<VoteTally>,
    pub joint_tally: Option<JointVoteTally>,
    pub citizen_tally: Option<CitizenVoteTally>,
    /// 关联机构名称（通过 institutionBytes 反查）。
    pub institution_name: Option<String>,
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

// ──── 公开查询函数 ────

/// 查询 NextProposalId（VotingEngineSystem 全局递增 ID）。
pub fn fetch_next_proposal_id() -> Result<u64, String> {
    let key = storage_keys::value_key("VotingEngineSystem", "NextProposalId");
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(key)]),
    )?;
    match result {
        Value::Null => Ok(0),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.len() < 8 {
                return Ok(0);
            }
            Ok(u64::from_le_bytes(data[..8].try_into().unwrap()))
        }
        _ => Ok(0),
    }
}

/// 分页查询提案列表（从 start_id 往前 count 个，按 ID 倒序）。
pub fn fetch_proposal_page(start_id: u64, count: u32) -> Result<ProposalPageResult, String> {
    let mut items = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let min_id = start_id.saturating_sub(count as u64);
    let mut id = start_id;

    while id > min_id {
        match fetch_proposal_meta(id) {
            Ok(Some(meta)) => {
                // 获取业务详情用于 summary
                let summary = match fetch_proposal_summary(id, &meta) {
                    Ok(s) => s,
                    Err(_) => "（详情查询失败）".to_string(),
                };
                let institution_name = resolve_institution_name(meta.institution_hex.as_deref());

                items.push(ProposalListItem {
                    proposal_id: id,
                    display_id: format_proposal_id(id),
                    kind: meta.kind,
                    kind_label: kind_label(meta.kind).to_string(),
                    stage: meta.stage,
                    stage_label: stage_label(meta.stage).to_string(),
                    status: meta.status,
                    status_label: status_label(meta.status).to_string(),
                    institution_name,
                    summary,
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
pub fn fetch_proposal_full(proposal_id: u64) -> Result<ProposalFullInfo, String> {
    let meta = fetch_proposal_meta(proposal_id)?
        .ok_or_else(|| format!("提案 {proposal_id} 不存在"))?;

    let (transfer_detail, runtime_upgrade_detail) = fetch_proposal_details(proposal_id, &meta)?;

    // 根据提案阶段查询对应的投票计数
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
        internal_tally,
        joint_tally,
        citizen_tally,
        institution_name,
    })
}

/// 查询机构的活跃提案 ID 列表。
pub fn fetch_active_proposal_ids(shenfen_id: &str) -> Result<Vec<u64>, String> {
    let institution_id = storage_keys::shenfen_id_to_fixed48(shenfen_id);
    let key = storage_keys::map_key(
        "VotingEngineSystem",
        "ActiveProposalsByInstitution",
        &institution_id,
    );
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(key)]),
    )?;
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
        "VotingEngineSystem",
        "Proposals",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(key)]),
    )?;
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
        "VotingEngineSystem",
        "ProposalData",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(key)]),
    )?;
    match result {
        Value::Null => Ok(None),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            Ok(Some(data))
        }
        _ => Ok(None),
    }
}

fn fetch_proposal_details(
    proposal_id: u64,
    meta: &ProposalMeta,
) -> Result<(Option<TransferProposalDetail>, Option<RuntimeUpgradeDetail>), String> {
    let raw = match fetch_proposal_data_raw(proposal_id)? {
        Some(r) => r,
        None => return Ok((None, None)),
    };
    if raw.is_empty() {
        return Ok((None, None));
    }

    // ProposalData 存储为 BoundedVec<u8>：Compact<len> + bytes
    let (vec_len, len_bytes) = read_compact_u32(&raw, 0)?;
    let offset = len_bytes;
    if offset + vec_len as usize > raw.len() {
        return Ok((None, None));
    }
    let data = &raw[offset..offset + vec_len as usize];

    if meta.kind == 1 {
        // 联合投票提案 → runtime 升级
        Ok((None, decode_runtime_upgrade_action(proposal_id, data)))
    } else {
        // 内部投票提案 → 转账
        Ok((decode_transfer_action(proposal_id, data), None))
    }
}

fn fetch_internal_tally(proposal_id: u64) -> Result<VoteTally, String> {
    let key = storage_keys::map_key(
        "VotingEngineSystem",
        "InternalTallies",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(key)]),
    )?;
    match result {
        Value::Null => Ok(VoteTally { yes: 0, no: 0 }),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.len() < 8 {
                return Ok(VoteTally { yes: 0, no: 0 });
            }
            let yes = u32::from_le_bytes(data[0..4].try_into().unwrap());
            let no = u32::from_le_bytes(data[4..8].try_into().unwrap());
            Ok(VoteTally { yes, no })
        }
        _ => Ok(VoteTally { yes: 0, no: 0 }),
    }
}

fn fetch_joint_tally(proposal_id: u64) -> Result<JointVoteTally, String> {
    let key = storage_keys::map_key(
        "VotingEngineSystem",
        "JointTallies",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(key)]),
    )?;
    match result {
        Value::Null => Ok(JointVoteTally { yes: 0, no: 0 }),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.len() < 8 {
                return Ok(JointVoteTally { yes: 0, no: 0 });
            }
            let yes = u32::from_le_bytes(data[0..4].try_into().unwrap());
            let no = u32::from_le_bytes(data[4..8].try_into().unwrap());
            Ok(JointVoteTally { yes, no })
        }
        _ => Ok(JointVoteTally { yes: 0, no: 0 }),
    }
}

fn fetch_citizen_tally(proposal_id: u64) -> Result<CitizenVoteTally, String> {
    let key = storage_keys::map_key(
        "VotingEngineSystem",
        "CitizenTallies",
        &proposal_id.to_le_bytes(),
    );
    let result = rpc_post(
        "state_getStorage",
        Value::Array(vec![Value::String(key)]),
    )?;
    match result {
        Value::Null => Ok(CitizenVoteTally { yes: 0, no: 0 }),
        Value::String(hex_data) => {
            let data = decode_hex_storage(&hex_data)?;
            if data.len() < 16 {
                return Ok(CitizenVoteTally { yes: 0, no: 0 });
            }
            let yes = u64::from_le_bytes(data[0..8].try_into().unwrap());
            let no = u64::from_le_bytes(data[8..16].try_into().unwrap());
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
    // institution: [u8;48] + beneficiary: [u8;32] + amount: u128(16)
    // + remark: Vec<u8> + proposer: [u8;32]
    if data.len() < 48 + 32 + 16 + 1 + 32 {
        return None;
    }
    let mut offset = 0;

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

fn decode_runtime_upgrade_action(
    proposal_id: u64,
    data: &[u8],
) -> Option<RuntimeUpgradeDetail> {
    let mut offset = 0;

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
    let reason =
        String::from_utf8_lossy(&data[offset..offset + reason_len as usize]).to_string();
    offset += reason_len as usize;

    // code_hash: [u8;32]
    if offset + 32 > data.len() {
        return None;
    }
    let code_hash_hex = hex::encode(&data[offset..offset + 32]);
    offset += 32;

    // has_code: bool
    if offset >= data.len() {
        return None;
    }
    let has_code = data[offset] != 0;
    offset += 1;

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
        has_code,
        status,
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
        let val = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
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
        3 => "执行失败",
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
    let end = bytes.iter().rposition(|&b| b != 0).map(|i| i + 1).unwrap_or(0);
    let shenfen_id = std::str::from_utf8(&bytes[..end]).ok()?;
    // 在静态数据中查找
    super::find_institution_name(shenfen_id)
}

/// 获取提案简要描述。
fn fetch_proposal_summary(proposal_id: u64, meta: &ProposalMeta) -> Result<String, String> {
    let raw = match fetch_proposal_data_raw(proposal_id)? {
        Some(r) => r,
        None => return Ok("（无详情数据）".to_string()),
    };
    if raw.is_empty() {
        return Ok("（无详情数据）".to_string());
    }

    let (vec_len, len_bytes) = read_compact_u32(&raw, 0)?;
    let offset = len_bytes;
    if offset + vec_len as usize > raw.len() {
        return Ok("（数据截断）".to_string());
    }
    let data = &raw[offset..offset + vec_len as usize];

    if meta.kind == 1 {
        // Runtime 升级提案
        if let Some(detail) = decode_runtime_upgrade_action(proposal_id, data) {
            let reason_short = if detail.reason.len() > 50 {
                format!("{}…", &detail.reason[..50])
            } else {
                detail.reason.clone()
            };
            return Ok(format!("运行时升级：{reason_short}"));
        }
    } else {
        // 转账提案
        if let Some(detail) = decode_transfer_action(proposal_id, data) {
            let amount: u128 = detail.amount_fen.parse().unwrap_or(0);
            let yuan = amount / 100;
            let fen = amount % 100;
            let remark_short = if detail.remark.len() > 30 {
                format!("{}…", &detail.remark[..30])
            } else {
                detail.remark.clone()
            };
            return Ok(format!("转账 {yuan}.{fen:02} 元：{remark_short}"));
        }
    }
    Ok("（无法解码详情）".to_string())
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
    let meta = fetch_proposal_meta(proposal_id)?
        .ok_or_else(|| format!("提案 {proposal_id} 不存在"))?;

    let pubkey_bytes = hex::decode(pubkey_hex)
        .map_err(|e| format!("公钥解码失败: {e}"))?;

    // 查询内部投票状态（InternalVotesByAccount: DoubleMap<u64, AccountId32> → bool）
    let internal_vote = {
        let key = storage_keys::double_map_key(
            "VotingEngineSystem",
            "InternalVotesByAccount",
            &proposal_id.to_le_bytes(),
            &pubkey_bytes,
        );
        fetch_option_bool(&key)?
    };

    // 查询联合投票状态（JointVotesByAdmin: DoubleMap<u64, (InstitutionId48 ++ AccountId32)> → bool）
    let joint_vote = if meta.kind == 1 && shenfen_id.is_some() {
        let institution_id = storage_keys::shenfen_id_to_fixed48(shenfen_id.unwrap());
        let mut composite_key = Vec::with_capacity(48 + 32);
        composite_key.extend_from_slice(&institution_id);
        composite_key.extend_from_slice(&pubkey_bytes);
        let key = storage_keys::double_map_key(
            "VotingEngineSystem",
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
