// 交易模块：冷钱包管理 + 链上转账（Balances::transfer_keep_alive）。
//
// 冷钱包仅存储 SS58 地址，签名通过 WUMIN_QR_V1 协议由离线设备完成。
// 转账构建和提交复用 governance/signing.rs 中的通用基础设施。

pub(crate) mod wallet_store;

use crate::{
    governance::{institution, signing},
    settings::{device_password, fee_address},
    shared::{constants::RPC_RESPONSE_LIMIT_SMALL, rpc, security},
};
use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use wallet_store::{ColdWallet, WalletKind, WalletStore};

const LOCAL_MINER_WALLET_ID: &str = "local-miner-hot-wallet";
const TRANSFER_RPC_TIMEOUT: Duration = Duration::from_secs(45);
const EXISTENTIAL_DEPOSIT_FEN: u128 = 111; // 1.11 元

/// 转账签名请求结果（前端用于显示 QR 码）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferSignRequestResult {
    pub request_json: String,
    pub request_id: String,
    pub expected_payload_hash: String,
    pub sign_nonce: u32,
    pub sign_block_number: u64,
    /// call_data 的 hex 编码（提交时需要回传）。
    pub call_data_hex: String,
    /// 预估手续费（元）。
    pub fee_yuan: f64,
}

/// 转账提交结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferSubmitResult {
    pub tx_hash: String,
}

fn normalize_pubkey_hex(pubkey_hex: &str, field_name: &str) -> Result<String, String> {
    let clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if clean.len() != 64 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("{field_name}格式无效"));
    }
    Ok(clean)
}

fn amount_yuan_to_fen(amount_yuan: f64) -> Result<u128, String> {
    if !amount_yuan.is_finite() {
        return Err("转账金额格式无效".to_string());
    }
    if amount_yuan < 0.01 {
        return Err("转账金额不能小于 0.01 元".to_string());
    }
    let amount_fen = (amount_yuan * 100.0).round() as u128;
    if amount_fen == 0 {
        return Err("转账金额不能为零".to_string());
    }
    Ok(amount_fen)
}

fn calculate_transfer_fee(amount_fen: u128) -> u128 {
    onchain_transaction::calculate_onchain_fee(amount_fen)
}

fn ensure_spendable_balance(
    sender_clean: &str,
    amount_fen: u128,
    fee_fen: u128,
) -> Result<(), String> {
    let balance_fen =
        institution::fetch_balance(sender_clean)?.ok_or("发送方账户不存在或余额为零")?;
    let total_needed = amount_fen + fee_fen;
    if balance_fen < total_needed + EXISTENTIAL_DEPOSIT_FEN {
        let available = if balance_fen > EXISTENTIAL_DEPOSIT_FEN {
            (balance_fen - EXISTENTIAL_DEPOSIT_FEN) as f64 / 100.0
        } else {
            0.0
        };
        return Err(format!(
            "余额不足：可用 {} 元，需要 {} 元（含手续费 {} 元）",
            signing::format_amount(available),
            signing::format_amount(total_needed as f64 / 100.0),
            signing::format_amount(fee_fen as f64 / 100.0),
        ));
    }
    Ok(())
}

fn local_miner_wallet(app: &tauri::AppHandle) -> Result<Option<ColdWallet>, String> {
    let Some(miner_hex) = fee_address::local_powr_miner_account_hex(app)? else {
        return Ok(None);
    };
    let pubkey_hex = normalize_pubkey_hex(&miner_hex, "矿工公钥")?;
    let pubkey = hex::decode(&pubkey_hex).map_err(|e| format!("矿工公钥解码失败: {e}"))?;
    let address = signing::pubkey_to_ss58(&pubkey)?;

    Ok(Some(ColdWallet {
        id: LOCAL_MINER_WALLET_ID.to_string(),
        name: "矿工热钱包".to_string(),
        kind: WalletKind::MinerHot,
        deletable: false,
        address,
        pubkey_hex,
        created_at: 0,
    }))
}

fn normalize_cold_wallets(store: &mut WalletStore) {
    for wallet in &mut store.wallets {
        wallet.kind = WalletKind::Cold;
        wallet.deletable = true;
    }
}

fn wallet_store_for_frontend(
    app: &tauri::AppHandle,
    mut store: WalletStore,
) -> Result<WalletStore, String> {
    // 中文注释：冷钱包文件只保存用户添加的钱包；矿工热钱包每次从 powr keystore 动态注入。
    normalize_cold_wallets(&mut store);

    let miner_wallet = local_miner_wallet(app)?;
    let has_miner_wallet = miner_wallet.is_some();
    let mut wallets = Vec::with_capacity(store.wallets.len() + usize::from(has_miner_wallet));
    if let Some(wallet) = miner_wallet {
        wallets.push(wallet);
    }
    wallets.extend(store.wallets.clone());

    let active_id = match store.active_id.as_deref() {
        Some(LOCAL_MINER_WALLET_ID) if has_miner_wallet => Some(LOCAL_MINER_WALLET_ID.to_string()),
        Some(id) if store.wallets.iter().any(|w| w.id == id) => Some(id.to_string()),
        _ => wallets.first().map(|w| w.id.clone()),
    };

    Ok(WalletStore { wallets, active_id })
}

// ──── 钱包管理命令 ────

#[tauri::command]
pub fn get_wallets(app: tauri::AppHandle) -> Result<WalletStore, String> {
    let store = wallet_store::load(&app)?;
    wallet_store_for_frontend(&app, store)
}

#[tauri::command]
pub fn add_wallet(
    app: tauri::AppHandle,
    name: String,
    address: String,
) -> Result<ColdWallet, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("钱包名称不能为空".to_string());
    }
    let address = address.trim().to_string();
    let pubkey_bytes = signing::decode_ss58_to_pubkey(&address)?;
    let pubkey_hex = hex::encode(pubkey_bytes);

    let mut store = wallet_store::load(&app)?;
    normalize_cold_wallets(&mut store);

    if let Some(miner_wallet) = local_miner_wallet(&app)? {
        if miner_wallet.pubkey_hex == pubkey_hex {
            return Err("矿工热钱包已在列表中，无需重复添加".to_string());
        }
    }

    // 查重：同一公钥不能重复添加
    if store.wallets.iter().any(|w| w.pubkey_hex == pubkey_hex) {
        return Err("该地址已存在".to_string());
    }

    let wallet = ColdWallet {
        id: generate_uuid(),
        name,
        kind: WalletKind::Cold,
        deletable: true,
        address,
        pubkey_hex,
        created_at: now_secs(),
    };

    store.wallets.push(wallet.clone());
    // 若本机尚无矿工热钱包且这是第一个钱包，保持旧行为自动激活。
    if store.active_id.is_none() && local_miner_wallet(&app)?.is_none() {
        store.active_id = Some(wallet.id.clone());
    }
    wallet_store::save(&app, &store)?;
    Ok(wallet)
}

#[tauri::command]
pub fn remove_wallet(app: tauri::AppHandle, wallet_id: String) -> Result<WalletStore, String> {
    if wallet_id == LOCAL_MINER_WALLET_ID {
        return Err("矿工热钱包不能删除".to_string());
    }
    let mut store = wallet_store::load(&app)?;
    normalize_cold_wallets(&mut store);
    let before_len = store.wallets.len();
    store.wallets.retain(|w| w.id != wallet_id);
    if store.wallets.len() == before_len {
        return Err("钱包不存在".to_string());
    }
    // 若删除的是激活钱包，清空激活状态
    if store.active_id.as_deref() == Some(&wallet_id) {
        store.active_id = if local_miner_wallet(&app)?.is_some() {
            Some(LOCAL_MINER_WALLET_ID.to_string())
        } else {
            store.wallets.first().map(|w| w.id.clone())
        };
    }
    wallet_store::save(&app, &store)?;
    wallet_store_for_frontend(&app, store)
}

#[tauri::command]
pub fn set_active_wallet(app: tauri::AppHandle, wallet_id: String) -> Result<WalletStore, String> {
    let mut store = wallet_store::load(&app)?;
    normalize_cold_wallets(&mut store);
    if wallet_id == LOCAL_MINER_WALLET_ID {
        local_miner_wallet(&app)?.ok_or("未找到矿工热钱包，请先启动节点生成矿工密钥")?;
        store.active_id = Some(LOCAL_MINER_WALLET_ID.to_string());
        wallet_store::save(&app, &store)?;
        return wallet_store_for_frontend(&app, store);
    }
    if !store.wallets.iter().any(|w| w.id == wallet_id) {
        return Err("钱包不存在".to_string());
    }
    store.active_id = Some(wallet_id);
    wallet_store::save(&app, &store)?;
    wallet_store_for_frontend(&app, store)
}

#[tauri::command]
pub fn get_wallet_balance(pubkey_hex: String) -> Result<Option<String>, String> {
    let clean = normalize_pubkey_hex(&pubkey_hex, "公钥")?;
    match institution::fetch_balance(&clean)? {
        Some(fen) => Ok(Some(fen.to_string())),
        None => Ok(None),
    }
}

// ──── 转账命令 ────

/// 构建 Balances::transfer_keep_alive 签名请求。
///
/// 返回 QR 签名请求 JSON，前端显示 QR 码供离线设备扫码签名。
#[tauri::command]
pub fn build_transfer_request(
    pubkey_hex: String,
    to_address: String,
    amount_yuan: f64,
) -> Result<TransferSignRequestResult, String> {
    // 校验发送方公钥
    let sender_clean = normalize_pubkey_hex(&pubkey_hex, "发送方公钥")?;
    let sender_bytes =
        hex::decode(&sender_clean).map_err(|e| format!("发送方公钥解码失败: {e}"))?;

    // 校验收款地址
    let dest_pubkey = signing::decode_ss58_to_pubkey(&to_address)?;
    let dest_hex = hex::encode(dest_pubkey);
    if dest_hex == sender_clean {
        return Err("收款地址不能与发送方相同".to_string());
    }

    let amount_fen = amount_yuan_to_fen(amount_yuan)?;

    // 中文注释：前端预估费和链上实扣费统一复用 runtime 手续费公式。
    let fee_fen = calculate_transfer_fee(amount_fen);
    let fee_yuan = fee_fen as f64 / 100.0;

    // 校验余额
    ensure_spendable_balance(&sender_clean, amount_fen, fee_fen)?;

    // 构建 call_data: Balances::transfer_keep_alive
    // pallet_index=2, call_index=3, MultiAddress::Id(0x00) + 32 bytes, Compact<u128>(amount_fen)
    let mut call_data = Vec::with_capacity(70);
    call_data.push(2u8); // Balances pallet index
    call_data.push(3u8); // transfer_keep_alive call index
    call_data.push(0x00); // MultiAddress::Id variant
    call_data.extend_from_slice(&dest_pubkey);
    call_data.extend_from_slice(&encode_compact_u128(amount_fen));

    // 获取链上参数并构建签名载荷
    let result = signing::build_sign_request_from_call_data(
        &sender_clean,
        &sender_bytes,
        &call_data,
        "transfer",
        &format!(
            "转账 {} GMB 给 {}...{}",
            signing::format_amount(amount_yuan),
            &to_address[..8],
            &to_address[to_address.len() - 6..]
        ),
        &serde_json::json!([
            { "key": "to", "label": "收款地址", "value": to_address },
            { "key": "amount_yuan", "label": "金额", "value": format!("{} GMB", signing::format_amount(amount_yuan)) }
        ]),
    )?;

    Ok(TransferSignRequestResult {
        request_json: result.request_json,
        request_id: result.request_id,
        expected_payload_hash: result.expected_payload_hash,
        sign_nonce: result.sign_nonce,
        sign_block_number: result.sign_block_number,
        call_data_hex: format!("0x{}", hex::encode(&call_data)),
        fee_yuan,
    })
}

/// 提交已签名的转账交易。
#[tauri::command]
pub fn submit_transfer(
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    call_data_hex: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<TransferSubmitResult, String> {
    let call_data_clean = call_data_hex.strip_prefix("0x").unwrap_or(&call_data_hex);
    let call_data = hex::decode(call_data_clean).map_err(|e| format!("call_data 解码失败: {e}"))?;

    let result = signing::verify_and_submit(
        &request_id,
        &expected_pubkey_hex,
        &expected_payload_hash,
        &call_data,
        sign_nonce,
        sign_block_number,
        &response_json,
    )?;

    Ok(TransferSubmitResult {
        tx_hash: result.tx_hash,
    })
}

/// 使用本机矿工热钱包直接签名并提交转账。
#[tauri::command]
pub async fn submit_miner_transfer(
    app: tauri::AppHandle,
    to_address: String,
    amount_yuan: f64,
    unlock_password: String,
) -> Result<TransferSubmitResult, String> {
    if let Err(e) = security::append_audit_log(&app, "submit_miner_transfer", "attempt") {
        eprintln!("[审计] submit_miner_transfer attempt 日志写入失败: {e}");
    }

    let unlock = security::ensure_unlock_password(&unlock_password)?;
    device_password::verify_device_login_password(&app, unlock)?;
    drop(unlock_password);

    let app_for_task = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        submit_miner_transfer_inner(&app_for_task, to_address, amount_yuan)
    })
    .await
    .map_err(|e| format!("矿工热钱包签名任务失败: {e}"))?;

    match &result {
        Ok(_) => {
            if let Err(e) = security::append_audit_log(&app, "submit_miner_transfer", "success") {
                eprintln!("[审计] submit_miner_transfer success 日志写入失败: {e}");
            }
        }
        Err(err) => {
            if let Err(e) = security::append_audit_log(&app, "submit_miner_transfer", "failed") {
                eprintln!("[审计] submit_miner_transfer failed 日志写入失败: {e}");
            }
            eprintln!("[交易] 矿工热钱包签名提交失败: {err}");
        }
    }

    result
}

fn submit_miner_transfer_inner(
    app: &tauri::AppHandle,
    to_address: String,
    amount_yuan: f64,
) -> Result<TransferSubmitResult, String> {
    let miner_wallet =
        local_miner_wallet(app)?.ok_or("未找到矿工热钱包，请先启动节点生成矿工密钥")?;
    let to_address = to_address.trim().to_string();
    let dest_pubkey = signing::decode_ss58_to_pubkey(&to_address)?;
    let dest_hex = hex::encode(dest_pubkey);
    if dest_hex == miner_wallet.pubkey_hex {
        return Err("收款地址不能与矿工热钱包相同".to_string());
    }

    let amount_fen = amount_yuan_to_fen(amount_yuan)?;
    let fee_fen = calculate_transfer_fee(amount_fen);
    ensure_spendable_balance(&miner_wallet.pubkey_hex, amount_fen, fee_fen)?;

    // 中文注释：真正的私钥签名只发生在节点 RPC 内部；一次性令牌避免外部本机 RPC 直接花费矿工余额。
    let auth_token = crate::core::rpc::issue_miner_transfer_token()?;
    let result = rpc::rpc_post(
        "transaction_submitMinerTransfer",
        serde_json::json!([to_address, amount_fen.to_string(), auth_token.clone()]),
        TRANSFER_RPC_TIMEOUT,
        RPC_RESPONSE_LIMIT_SMALL,
    );
    if result.is_err() {
        crate::core::rpc::revoke_miner_transfer_token(&auth_token);
    }
    let result = result?;
    let tx_hash = result
        .as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or("节点未返回交易哈希")?
        .to_string();

    Ok(TransferSubmitResult { tx_hash })
}

// ──── 编码工具 ────

/// SCALE Compact<u128> 编码。
fn encode_compact_u128(value: u128) -> Vec<u8> {
    if value < 0x40 {
        vec![(value as u8) << 2]
    } else if value < 0x4000 {
        let v = ((value as u16) << 2) | 0x01;
        vec![v as u8, (v >> 8) as u8]
    } else if value < 0x4000_0000 {
        let v = ((value as u32) << 2) | 0x02;
        v.to_le_bytes().to_vec()
    } else {
        // big-integer mode: 上 6 位 = (byte_count - 4), 下 2 位 = 0b11
        let le_bytes = value.to_le_bytes();
        // 找到最后一个非零字节确定实际长度
        let byte_count = le_bytes
            .iter()
            .rposition(|&b| b != 0)
            .map(|i| i + 1)
            .unwrap_or(1);
        let header = (((byte_count as u8 - 4) << 2) | 0x03) as u8;
        let mut out = vec![header];
        out.extend_from_slice(&le_bytes[..byte_count]);
        out
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn generate_uuid() -> String {
    let bytes: [u8; 16] = rand::random();
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        u64::from_be_bytes([
            0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        ]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_u128_single_byte() {
        assert_eq!(encode_compact_u128(0), vec![0x00]);
        assert_eq!(encode_compact_u128(1), vec![0x04]);
        assert_eq!(encode_compact_u128(63), vec![0xfc]);
    }

    #[test]
    fn compact_u128_two_bytes() {
        assert_eq!(encode_compact_u128(64), vec![0x01, 0x01]);
        assert_eq!(encode_compact_u128(16383), vec![0xfd, 0xff]);
    }

    #[test]
    fn compact_u128_four_bytes() {
        assert_eq!(encode_compact_u128(16384), vec![0x02, 0x00, 0x01, 0x00]);
        assert_eq!(
            encode_compact_u128(1_073_741_823),
            vec![0xfe, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn compact_u128_big_integer() {
        // 10000 元 = 1_000_000 分
        let result = encode_compact_u128(1_000_000);
        assert_eq!(result[0] & 0x03, 0x03); // big-integer mode
    }

    #[test]
    fn compact_u128_large_value() {
        // 确保大金额不溢出
        let result = encode_compact_u128(100_000_000_000_u128);
        assert!(!result.is_empty());
        assert_eq!(result[0] & 0x03, 0x03);
    }
}
