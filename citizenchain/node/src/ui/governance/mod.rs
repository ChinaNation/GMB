// 治理模块入口：注册 Tauri 命令，聚合机构数据。

pub mod activation;
pub(crate) mod balance_watch;
pub(crate) mod institution;
pub mod proposal;
pub(crate) mod registry;
pub mod sfid_api;
pub mod signing;
mod storage_keys;
pub mod types;

use crate::ui::home;
use registry::InstitutionRef;
use types::{GovernanceOverview, InstitutionBalanceUpdate, InstitutionDetail, OrgType};

use serde::Serialize;
use tauri::AppHandle;

fn internal_threshold(org_type: OrgType) -> u32 {
    match org_type {
        OrgType::Nrc => 13,
        OrgType::Prc | OrgType::Prb => 6,
    }
}

fn joint_vote_weight(org_type: OrgType) -> u32 {
    match org_type {
        OrgType::Nrc => 19,
        OrgType::Prc | OrgType::Prb => 1,
    }
}

#[derive(Default)]
struct InstitutionBalances {
    balance_fen: Option<String>,
    staking_balance_fen: Option<String>,
    fee_balance_fen: Option<String>,
    cb_fee_balance_fen: Option<String>,
    nrc_fee_balance_fen: Option<String>,
    nrc_anquan_balance_fen: Option<String>,
}

struct ChainQueryContext {
    running: bool,
    block_hash: Option<String>,
    warnings: Vec<String>,
}

fn join_warnings(warnings: Vec<String>) -> Option<String> {
    if warnings.is_empty() {
        None
    } else {
        Some(warnings.join("；"))
    }
}

fn build_chain_query_context(app: &AppHandle) -> Result<ChainQueryContext, String> {
    let status = home::current_status(app)?;
    if !status.running {
        return Ok(ChainQueryContext {
            running: false,
            block_hash: None,
            warnings: vec!["节点未运行，无法查询链上数据".to_string()],
        });
    }

    let mut warnings = Vec::new();
    let block_hash = match institution::fetch_finalized_head() {
        Ok(hash) => Some(hash),
        Err(e) => {
            warnings.push(format!("查询最新区块失败: {e}"));
            None
        }
    };
    Ok(ChainQueryContext {
        running: true,
        block_hash,
        warnings,
    })
}

fn load_balance_at_block(
    account_hex: &str,
    block_hash: Option<&str>,
    label: &str,
    warnings: &mut Vec<String>,
) -> Option<String> {
    let Some(hash) = block_hash else {
        return None;
    };

    match institution::fetch_balance_at(account_hex, Some(hash)) {
        Ok(balance) => balance.map(|value| value.to_string()),
        Err(e) => {
            warnings.push(format!("查询{label}失败: {e}"));
            None
        }
    }
}

fn collect_admins(
    shenfen_id: &str,
    block_hash: Option<&str>,
    warnings: &mut Vec<String>,
) -> Vec<types::AdminInfo> {
    let pubkeys = match institution::fetch_admins(shenfen_id) {
        Ok(items) => items,
        Err(e) => {
            warnings.push(format!("查询管理员失败: {e}"));
            Vec::new()
        }
    };

    pubkeys
        .into_iter()
        .map(|pubkey_hex| {
            let balance_fen = block_hash
                .and_then(|hash| institution::fetch_balance_at(&pubkey_hex, Some(hash)).ok())
                .flatten()
                .map(|value| value.to_string());
            types::AdminInfo {
                pubkey_hex,
                balance_fen,
            }
        })
        .collect()
}

fn collect_institution_balances(
    entry: InstitutionRef,
    block_hash: Option<&str>,
    warnings: &mut Vec<String>,
) -> InstitutionBalances {
    let main_address = entry.main_address_hex();
    let mut balances = InstitutionBalances {
        balance_fen: load_balance_at_block(&main_address, block_hash, "主账户余额", warnings),
        ..InstitutionBalances::default()
    };

    match entry {
        InstitutionRef::Nrc(_) => {
            let fee_address = entry.fee_address_hex();
            let anquan_address = entry
                .anquan_address_hex()
                .expect("国储会安全基金账户地址必须存在");
            balances.nrc_fee_balance_fen =
                load_balance_at_block(&fee_address, block_hash, "费用账户余额", warnings);
            balances.nrc_anquan_balance_fen =
                load_balance_at_block(&anquan_address, block_hash, "安全基金账户余额", warnings);
        }
        InstitutionRef::Prc(_) => {
            let fee_address = entry.fee_address_hex();
            balances.cb_fee_balance_fen =
                load_balance_at_block(&fee_address, block_hash, "费用账户余额", warnings);
        }
        InstitutionRef::Prb(_) => {
            let stake_address = entry
                .staking_address_hex()
                .expect("省储行永久质押账户地址必须存在");
            let fee_address = entry.fee_address_hex();
            balances.staking_balance_fen =
                load_balance_at_block(&stake_address, block_hash, "永久质押账户余额", warnings);
            balances.fee_balance_fen =
                load_balance_at_block(&fee_address, block_hash, "费用账户余额", warnings);
        }
    }

    balances
}

fn build_institution_detail_sync(
    app: &AppHandle,
    shenfen_id: &str,
) -> Result<InstitutionDetail, String> {
    let entry = registry::find_institution(shenfen_id)
        .ok_or_else(|| format!("未知的机构 shenfenId: {shenfen_id}"))?;
    let org_type = entry.org_type();
    let mut context = build_chain_query_context(app)?;
    let admins = if context.running {
        collect_admins(
            shenfen_id,
            context.block_hash.as_deref(),
            &mut context.warnings,
        )
    } else {
        Vec::new()
    };
    let balances =
        collect_institution_balances(entry, context.block_hash.as_deref(), &mut context.warnings);
    let (staking_address, fee_address, cb_fee_address, nrc_fee_address, nrc_anquan_address) =
        match entry {
            InstitutionRef::Nrc(_) => (
                None,
                None,
                None,
                Some(entry.fee_address_hex()),
                entry.anquan_address_hex(),
            ),
            InstitutionRef::Prc(_) => (None, None, Some(entry.fee_address_hex()), None, None),
            InstitutionRef::Prb(_) => (
                entry.staking_address_hex(),
                Some(entry.fee_address_hex()),
                None,
                None,
                None,
            ),
        };

    Ok(InstitutionDetail {
        name: entry.name().to_string(),
        shenfen_id: shenfen_id.to_string(),
        org_type: org_type as u8,
        org_type_label: org_type.label().to_string(),
        main_address: entry.main_address_hex(),
        balance_fen: balances.balance_fen,
        admins,
        internal_threshold: internal_threshold(org_type),
        joint_vote_weight: joint_vote_weight(org_type),
        staking_address,
        staking_balance_fen: balances.staking_balance_fen,
        fee_address,
        fee_balance_fen: balances.fee_balance_fen,
        cb_fee_address,
        cb_fee_balance_fen: balances.cb_fee_balance_fen,
        nrc_fee_address,
        nrc_fee_balance_fen: balances.nrc_fee_balance_fen,
        nrc_anquan_address,
        nrc_anquan_balance_fen: balances.nrc_anquan_balance_fen,
        warning: join_warnings(context.warnings),
    })
}

pub(super) fn build_institution_balance_update_sync(
    app: &AppHandle,
    shenfen_id: &str,
) -> Result<InstitutionBalanceUpdate, String> {
    let entry = registry::find_institution(shenfen_id)
        .ok_or_else(|| format!("未知的机构 shenfenId: {shenfen_id}"))?;
    let mut context = build_chain_query_context(app)?;
    let balances =
        collect_institution_balances(entry, context.block_hash.as_deref(), &mut context.warnings);

    Ok(InstitutionBalanceUpdate {
        shenfen_id: shenfen_id.to_string(),
        balance_fen: balances.balance_fen,
        staking_balance_fen: balances.staking_balance_fen,
        fee_balance_fen: balances.fee_balance_fen,
        cb_fee_balance_fen: balances.cb_fee_balance_fen,
        nrc_fee_balance_fen: balances.nrc_fee_balance_fen,
        nrc_anquan_balance_fen: balances.nrc_anquan_balance_fen,
        warning: join_warnings(context.warnings),
    })
}

/// 获取治理首页机构分类列表（直接读取 runtime 常量，不依赖节点运行）。
#[tauri::command]
pub async fn get_governance_overview() -> Result<GovernanceOverview, String> {
    Ok(registry::governance_overview())
}

/// 获取指定机构的详细信息（地址来自 runtime 常量，金额来自链上 finalized 快照）。
#[tauri::command]
pub async fn get_institution_detail(
    app: AppHandle,
    shenfen_id: String,
) -> Result<InstitutionDetail, String> {
    tauri::async_runtime::spawn_blocking(move || build_institution_detail_sync(&app, &shenfen_id))
        .await
        .map_err(|e| format!("institution detail task failed: {e}"))?
}

/// 通过 shenfenId 查找机构名称（供 proposal 模块反查用）。
pub(crate) fn find_institution_name(shenfen_id: &str) -> Option<String> {
    registry::find_institution_name(shenfen_id).map(str::to_string)
}

/// 获取提案分页列表（需要节点运行）。
#[tauri::command]
pub async fn get_proposal_page(
    app: AppHandle,
    start_id: u64,
    count: u32,
) -> Result<proposal::ProposalPageResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || proposal::fetch_proposal_page(start_id, count))
        .await
        .map_err(|e| format!("proposal page task failed: {e}"))?
}

/// 获取单个提案完整信息（需要节点运行）。
#[tauri::command]
pub async fn get_proposal_detail(
    app: AppHandle,
    proposal_id: u64,
) -> Result<proposal::ProposalFullInfo, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || proposal::fetch_proposal_full(proposal_id))
        .await
        .map_err(|e| format!("proposal detail task failed: {e}"))?
}

/// 获取 NextProposalId（需要节点运行）。
#[tauri::command]
pub async fn get_next_proposal_id(app: AppHandle) -> Result<u64, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询提案 ID".to_string());
    }
    tauri::async_runtime::spawn_blocking(proposal::fetch_next_proposal_id)
        .await
        .map_err(|e| format!("next proposal id task failed: {e}"))?
}

/// 获取机构活跃提案 ID 列表（需要节点运行）。
#[tauri::command]
pub async fn get_institution_proposals(
    app: AppHandle,
    shenfen_id: String,
) -> Result<Vec<proposal::ProposalListItem>, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let ids = proposal::fetch_active_proposal_ids(&shenfen_id)?;
        let mut items = Vec::new();
        for id in ids.iter().rev() {
            match proposal::fetch_proposal_page(*id, 1) {
                Ok(page) => items.extend(page.items),
                Err(_) => {}
            }
        }
        Ok(items)
    })
    .await
    .map_err(|e| format!("institution proposals task failed: {e}"))?
}

/// 分页查询指定机构的所有提案（需要节点运行）。
///
/// 从 start_id 倒序遍历，过滤属于该机构的提案，返回分页结果。
#[tauri::command]
pub async fn get_institution_proposal_page(
    app: AppHandle,
    shenfen_id: String,
    start_id: u64,
    count: u32,
) -> Result<proposal::ProposalPageResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        proposal::fetch_institution_proposal_page(&shenfen_id, start_id, count)
    })
    .await
    .map_err(|e| format!("institution proposal page task failed: {e}"))?
}

/// 构建投票签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_vote_request(
    app: AppHandle,
    proposal_id: u64,
    pubkey_hex: String,
    approve: bool,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_vote_sign_request(proposal_id, &pubkey_hex, approve)
    })
    .await
    .map_err(|e| format!("build vote request task failed: {e}"))?
}

/// 构建联合投票签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_joint_vote_request(
    app: AppHandle,
    proposal_id: u64,
    pubkey_hex: String,
    shenfen_id: String,
    approve: bool,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_joint_vote_sign_request(proposal_id, &pubkey_hex, &shenfen_id, approve)
    })
    .await
    .map_err(|e| format!("build joint vote request task failed: {e}"))?
}

/// 验证签名响应并提交投票（通用，支持内部和联合投票）。
///
/// call_data_hex 为完整的 SCALE call data hex（不含 0x 前缀）。
#[tauri::command]
pub async fn submit_vote(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    call_data_hex: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交投票".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data =
            hex::decode(&call_data_hex).map_err(|e| format!("call_data 解码失败: {e}"))?;
        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit vote task failed: {e}"))?
}

/// 查询用户投票状态（需要节点运行）。
#[tauri::command]
pub async fn check_vote_status(
    app: AppHandle,
    proposal_id: u64,
    pubkey_hex: String,
    shenfen_id: Option<String>,
) -> Result<proposal::UserVoteStatus, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询投票状态".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        proposal::fetch_user_vote_status(proposal_id, &pubkey_hex, shenfen_id.as_deref())
    })
    .await
    .map_err(|e| format!("check vote status task failed: {e}"))?
}

/// 构建创建转账提案签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_propose_transfer_request(
    app: AppHandle,
    pubkey_hex: String,
    shenfen_id: String,
    org_type: u8,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_propose_transfer_sign_request(
            &pubkey_hex,
            &shenfen_id,
            org_type,
            &beneficiary_address,
            amount_yuan,
            &remark,
        )
    })
    .await
    .map_err(|e| format!("build propose transfer request task failed: {e}"))?
}

/// 构建开发期 runtime 直接升级签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_developer_upgrade_request(
    app: AppHandle,
    pubkey_hex: String,
    wasm_path: String,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_developer_upgrade_sign_request(&pubkey_hex, &wasm_path)
    })
    .await
    .map_err(|e| format!("build developer upgrade request task failed: {e}"))?
}

/// 验证签名响应并提交开发期 runtime 直接升级。
#[tauri::command]
pub async fn submit_developer_upgrade(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    wasm_path: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交升级".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = signing::build_developer_upgrade_call_data(&wasm_path)?;
        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit developer upgrade task failed: {e}"))?
}

/// 构建 propose_runtime_upgrade 签名请求的返回值（包含 SFID 快照数据）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposeUpgradeRequestResult {
    pub request_json: String,
    pub request_id: String,
    pub expected_payload_hash: String,
    pub sign_nonce: u32,
    pub sign_block_number: u64,
    pub eligible_total: u64,
    pub snapshot_nonce: String,
    pub snapshot_signature: String,
}

/// 构建 Runtime 升级提案签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_propose_upgrade_request(
    app: AppHandle,
    pubkey_hex: String,
    wasm_path: String,
    reason: String,
) -> Result<ProposeUpgradeRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let (sign_result, snapshot) =
            signing::build_propose_runtime_upgrade_sign_request(&pubkey_hex, &wasm_path, &reason)?;
        Ok(ProposeUpgradeRequestResult {
            request_json: sign_result.request_json,
            request_id: sign_result.request_id,
            expected_payload_hash: sign_result.expected_payload_hash,
            sign_nonce: sign_result.sign_nonce,
            sign_block_number: sign_result.sign_block_number,
            eligible_total: snapshot.eligible_total,
            snapshot_nonce: snapshot.snapshot_nonce,
            snapshot_signature: snapshot.signature,
        })
    })
    .await
    .map_err(|e| format!("build propose upgrade request task failed: {e}"))?
}

/// 验证签名响应并提交 Runtime 升级提案。
#[tauri::command]
pub async fn submit_propose_upgrade(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    wasm_path: String,
    reason: String,
    eligible_total: u64,
    snapshot_nonce: String,
    snapshot_signature: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = signing::build_propose_runtime_upgrade_call_data(
            &wasm_path,
            &reason,
            eligible_total,
            &snapshot_nonce,
            &snapshot_signature,
        )?;
        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit propose upgrade task failed: {e}"))?
}

/// 验证签名响应并提交转账提案（专用命令，后端构建 call data）。
#[tauri::command]
pub async fn submit_propose_transfer(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    shenfen_id: String,
    org_type: u8,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let amount_fen = (amount_yuan * 100.0).round() as u128;
        let institution_id = storage_keys::shenfen_id_to_fixed48(&shenfen_id);
        let beneficiary_bytes = signing::decode_ss58_to_pubkey(&beneficiary_address)?;
        let remark_bytes = remark.as_bytes();
        let remark_compact = signing::encode_compact_u32_pub(remark_bytes.len() as u32);

        let mut call_data = Vec::new();
        call_data.push(19u8);
        call_data.push(0u8);
        call_data.push(org_type);
        call_data.extend_from_slice(&institution_id);
        call_data.extend_from_slice(&beneficiary_bytes);
        call_data.extend_from_slice(&amount_fen.to_le_bytes());
        call_data.extend_from_slice(&remark_compact);
        call_data.extend_from_slice(remark_bytes);

        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit propose transfer task failed: {e}"))?
}

/// 构建安全基金转账提案签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_propose_safety_fund_request(
    app: AppHandle,
    pubkey_hex: String,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_propose_safety_fund_sign_request(
            &pubkey_hex,
            &beneficiary_address,
            amount_yuan,
            &remark,
        )
    })
    .await
    .map_err(|e| format!("build propose safety fund request task failed: {e}"))?
}

/// 验证签名响应并提交安全基金转账提案。
#[tauri::command]
pub async fn submit_propose_safety_fund(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let amount_fen = (amount_yuan * 100.0).round() as u128;
        let beneficiary_bytes = signing::decode_ss58_to_pubkey(&beneficiary_address)?;
        let remark_bytes = remark.as_bytes();
        let remark_compact = signing::encode_compact_u32_pub(remark_bytes.len() as u32);

        let mut call_data = Vec::new();
        call_data.push(19u8); // DuoqianTransferPow pallet
        call_data.push(1u8); // propose_safety_fund_transfer call (Phase 2 重排,原 3)
        call_data.extend_from_slice(&beneficiary_bytes);
        call_data.extend_from_slice(&amount_fen.to_le_bytes());
        call_data.extend_from_slice(&remark_compact);
        call_data.extend_from_slice(remark_bytes);

        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit propose safety fund task failed: {e}"))?
}

/// 构建安全基金投票签名请求 QR JSON（需要节点运行）。
// Phase 3(2026-04-22): `build_safety_fund_vote_request` 已删除,
// 所有管理员投票统一走 `build_vote_request`(internal_vote, 9.0)。

/// 构建手续费划转提案签名请求。
#[tauri::command]
pub async fn build_propose_sweep_request(
    app: AppHandle,
    pubkey_hex: String,
    shenfen_id: String,
    amount_yuan: f64,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_propose_sweep_sign_request(&pubkey_hex, &shenfen_id, amount_yuan)
    })
    .await
    .map_err(|e| format!("build propose sweep failed: {e}"))?
}

/// 验证签名并提交手续费划转提案。
#[tauri::command]
pub async fn submit_propose_sweep(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    shenfen_id: String,
    amount_yuan: f64,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let amount_fen = (amount_yuan * 100.0).round() as u128;
        let institution_id = storage_keys::shenfen_id_to_fixed48(&shenfen_id);
        let mut call_data = Vec::with_capacity(66);
        call_data.push(19u8); // DuoqianTransferPow pallet
        call_data.push(2u8); // propose_sweep_to_main call (Phase 2 重排,原 5)
        call_data.extend_from_slice(&institution_id);
        call_data.extend_from_slice(&amount_fen.to_le_bytes());
        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit propose sweep failed: {e}"))?
}

// Phase 3(2026-04-22): `build_sweep_vote_request` 已删除,
// 所有管理员投票统一走 `build_vote_request`(internal_vote, 9.0)。
