// 治理投票 QR 签名：构建 WUMIN_SIGN_V1.0.0 签名请求、验证响应、提交 extrinsic。
//
// 协议流程：
// 1. 后端构建未签名 signing payload + QR 请求 JSON
// 2. 前端显示 QR 码 → 用户用 wumin 离线设备扫码签名
// 3. 前端摄像头扫描响应 QR → 传回后端
// 4. 后端验证 payload_hash → 构建 signed extrinsic → 提交到链

use crate::shared::rpc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PROTOCOL_VERSION: &str = "WUMIN_SIGN_V1.0.0";
const DEFAULT_TTL_SECS: u64 = 90;
const MORTAL_ERA_PERIOD: u64 = 64;
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RPC_RESPONSE_BYTES: u64 = 512 * 1024;
/// SS58 前缀 2027。
const SS58_PREFIX: u16 = 2027;

// ──── QR 协议数据结构 ────

/// 签名请求（nodeui → 离线设备）。
/// 字段名使用 snake_case，与 WUMIN_SIGN_V1.0.0 协议一致。
#[derive(Debug, Serialize)]
pub struct QrSignRequest {
    pub proto: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub request_id: String,
    pub account: String,
    pub pubkey: String,
    pub sig_alg: String,
    pub payload_hex: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub display: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_version: Option<u32>,
}

/// 签名响应（离线设备 → nodeui）。
/// 字段名使用 snake_case，与 WUMIN_SIGN_V1.0.0 协议一致。
#[derive(Debug, Deserialize)]
pub struct QrSignResponse {
    pub proto: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub request_id: String,
    pub pubkey: String,
    pub sig_alg: String,
    pub signature: String,
    pub payload_hash: String,
    pub signed_at: u64,
}

/// 构建投票签名请求的结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteSignRequestResult {
    /// 完整的 QR 签名请求 JSON 字符串。
    pub request_json: String,
    /// 请求 ID（用于后续验证匹配）。
    pub request_id: String,
    /// 签名 payload 的 SHA-256 哈希（用于验证响应）。
    pub expected_payload_hash: String,
    /// 签名时使用的 nonce（提交时必须复用）。
    pub sign_nonce: u32,
    /// 签名时使用的区块号（提交时必须复用以计算相同 era）。
    pub sign_block_number: u64,
}

/// 投票提交结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteSubmitResult {
    pub tx_hash: String,
}

// ──── 公开函数 ────

/// 构建 vote_transfer 签名请求。
///
/// 返回 QR 签名请求 JSON + 请求 ID + 预期 payload hash。
pub fn build_vote_sign_request(
    proposal_id: u64,
    pubkey_hex: &str,
    approve: bool,
) -> Result<VoteSignRequestResult, String> {
    // 验证公钥格式
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if pubkey_clean.len() != 64 || !pubkey_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效，应为 64 位十六进制".to_string());
    }
    let pubkey_bytes = hex::decode(&pubkey_clean)
        .map_err(|e| format!("公钥解码失败: {e}"))?;

    // 获取链上参数
    let (spec_version, tx_version) = fetch_runtime_version()?;
    let genesis_hash = fetch_genesis_hash()?;
    let (block_hash, block_number) = fetch_latest_block()?;
    let nonce = fetch_nonce(&pubkey_clean)?;

    // 构建 call data: [pallet=19][call=1][proposal_id: u64_le][approve: bool]
    let mut call_data = Vec::with_capacity(11);
    call_data.push(19u8); // DuoqianTransferPow pallet index
    call_data.push(1u8);  // vote_transfer call index
    call_data.extend_from_slice(&proposal_id.to_le_bytes());
    call_data.push(if approve { 1u8 } else { 0u8 });

    // 构建 signing payload
    let payload = build_signing_payload(
        &call_data, &genesis_hash, &block_hash, block_number,
        nonce, spec_version, tx_version,
    );

    // 计算 payload hash
    let payload_hash = sha256_hash(&payload);

    // 生成请求 ID
    let request_id = generate_request_id("vote");

    // SS58 编码账户地址
    let account_ss58 = pubkey_to_ss58(&pubkey_bytes)?;

    // display.fields 必须与 wumin PayloadDecoder 解码结果的 key/value 完全一致。
    // wumin 解码 vote_transfer 返回: proposal_id=数字字符串, approve="true"/"false"
    let display = serde_json::json!({
        "action": "vote_transfer",
        "action_label": "转账提案投票",
        "summary": format!("转账提案 #{proposal_id} 投票：{}", if approve { "赞成" } else { "反对" }),
        "fields": [
            { "key": "proposal_id", "label": "提案编号", "value": proposal_id.to_string() },
            { "key": "approve", "label": "投票", "value": approve.to_string() }
        ]
    });

    let now = now_secs();
    let request = QrSignRequest {
        proto: PROTOCOL_VERSION.to_string(),
        msg_type: "sign_request".to_string(),
        request_id: request_id.clone(),
        account: account_ss58,
        pubkey: format!("0x{pubkey_clean}"),
        sig_alg: "sr25519".to_string(),
        payload_hex: format!("0x{}", hex::encode(&payload)),
        issued_at: now,
        expires_at: now + DEFAULT_TTL_SECS,
        display,
        spec_version: Some(spec_version),
    };

    let request_json = serde_json::to_string(&request)
        .map_err(|e| format!("序列化签名请求失败: {e}"))?;

    Ok(VoteSignRequestResult {
        request_json,
        request_id,
        expected_payload_hash: format!("0x{}", hex::encode(&payload_hash)),
        sign_nonce: nonce,
        sign_block_number: block_number,
    })
}

/// 构建 joint_vote 签名请求（联合投票：pallet=9, call=3）。
///
/// shenfen_id 用于构造 institution_id 48 字节参数。
pub fn build_joint_vote_sign_request(
    proposal_id: u64,
    pubkey_hex: &str,
    shenfen_id: &str,
    approve: bool,
) -> Result<VoteSignRequestResult, String> {
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if pubkey_clean.len() != 64 || !pubkey_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效，应为 64 位十六进制".to_string());
    }
    let pubkey_bytes = hex::decode(&pubkey_clean)
        .map_err(|e| format!("公钥解码失败: {e}"))?;

    let institution_id = super::storage_keys::shenfen_id_to_fixed48(shenfen_id);

    let (spec_version, tx_version) = fetch_runtime_version()?;
    let genesis_hash = fetch_genesis_hash()?;
    let (block_hash, block_number) = fetch_latest_block()?;
    let nonce = fetch_nonce(&pubkey_clean)?;

    // call data: [pallet=9][call=3][proposal_id: u64_le][institution_id: 48 bytes][approve: bool]
    let mut call_data = Vec::with_capacity(1 + 1 + 8 + 48 + 1);
    call_data.push(9u8);  // VotingEngineSystem pallet index
    call_data.push(3u8);  // joint_vote call index
    call_data.extend_from_slice(&proposal_id.to_le_bytes());
    call_data.extend_from_slice(&institution_id);
    call_data.push(if approve { 1u8 } else { 0u8 });

    let payload = build_signing_payload(
        &call_data, &genesis_hash, &block_hash, block_number,
        nonce, spec_version, tx_version,
    );
    let payload_hash = sha256_hash(&payload);
    let request_id = generate_request_id("jvote");
    let account_ss58 = pubkey_to_ss58(&pubkey_bytes)?;

    // display.fields 必须与 wumin PayloadDecoder 解码结果的 key/value 完全一致。
    // wumin 解码 joint_vote 返回: proposal_id=数字字符串, approve="true"/"false"
    let display = serde_json::json!({
        "action": "joint_vote",
        "action_label": "联合投票",
        "summary": format!("联合投票 提案 #{proposal_id}：{}", if approve { "赞成" } else { "反对" }),
        "fields": [
            { "key": "proposal_id", "label": "提案编号", "value": proposal_id.to_string() },
            { "key": "approve", "label": "投票", "value": approve.to_string() }
        ]
    });

    let now = now_secs();
    let request = QrSignRequest {
        proto: PROTOCOL_VERSION.to_string(),
        msg_type: "sign_request".to_string(),
        request_id: request_id.clone(),
        account: account_ss58,
        pubkey: format!("0x{pubkey_clean}"),
        sig_alg: "sr25519".to_string(),
        payload_hex: format!("0x{}", hex::encode(&payload)),
        issued_at: now,
        expires_at: now + DEFAULT_TTL_SECS,
        display,
        spec_version: Some(spec_version),
    };

    let request_json = serde_json::to_string(&request)
        .map_err(|e| format!("序列化签名请求失败: {e}"))?;

    Ok(VoteSignRequestResult {
        request_json,
        request_id,
        expected_payload_hash: format!("0x{}", hex::encode(&payload_hash)),
        sign_nonce: nonce,
        sign_block_number: block_number,
    })
}

/// 构建 propose_transfer 签名请求（创建转账提案：pallet=19, call=0）。
pub fn build_propose_transfer_sign_request(
    pubkey_hex: &str,
    shenfen_id: &str,
    org_type: u8,
    beneficiary_address: &str,
    amount_yuan: f64,
    remark: &str,
) -> Result<VoteSignRequestResult, String> {
    // 验证公钥
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if pubkey_clean.len() != 64 || !pubkey_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效，应为 64 位十六进制".to_string());
    }
    let pubkey_bytes = hex::decode(&pubkey_clean)
        .map_err(|e| format!("公钥解码失败: {e}"))?;

    // 验证金额
    if amount_yuan < 1.11 {
        return Err("转账金额不能低于 1.11 元".to_string());
    }
    let amount_fen = (amount_yuan * 100.0).round() as u128;

    // 验证备注长度
    let remark_bytes = remark.as_bytes();
    if remark_bytes.len() > 256 {
        return Err(format!(
            "备注长度不能超过 256 字节，当前 {} 字节",
            remark_bytes.len()
        ));
    }

    // 解码收款地址
    let beneficiary_bytes = decode_ss58_to_pubkey(beneficiary_address)?;

    // 验证收款地址不等于本机构多签地址
    let entry = super::find_entry(shenfen_id)
        .ok_or_else(|| format!("未知的机构 shenfenId: {shenfen_id}"))?;
    let institution_duoqian = hex::decode(entry.duoqian_address)
        .map_err(|e| format!("机构多签地址解码失败: {e}"))?;
    if beneficiary_bytes[..] == institution_duoqian[..] {
        return Err("收款地址不能为本机构多签地址".to_string());
    }

    let institution_id = super::storage_keys::shenfen_id_to_fixed48(shenfen_id);

    let (spec_version, tx_version) = fetch_runtime_version()?;
    let genesis_hash = fetch_genesis_hash()?;
    let (block_hash, block_number) = fetch_latest_block()?;
    let nonce = fetch_nonce(&pubkey_clean)?;

    // call data: [0x13][0x00][org:u8][institution:48][beneficiary:32][amount:u128_le][remark:Vec<u8>]
    let remark_compact = encode_compact_u32(remark_bytes.len() as u32);
    let mut call_data = Vec::with_capacity(2 + 1 + 48 + 32 + 16 + remark_compact.len() + remark_bytes.len());
    call_data.push(19u8); // DuoqianTransferPow pallet
    call_data.push(0u8);  // propose_transfer call
    call_data.push(org_type);
    call_data.extend_from_slice(&institution_id);
    call_data.extend_from_slice(&beneficiary_bytes);
    call_data.extend_from_slice(&amount_fen.to_le_bytes());
    call_data.extend_from_slice(&remark_compact);
    call_data.extend_from_slice(remark_bytes);

    let payload = build_signing_payload(
        &call_data, &genesis_hash, &block_hash, block_number,
        nonce, spec_version, tx_version,
    );
    let payload_hash = sha256_hash(&payload);
    let request_id = generate_request_id("propose");
    let account_ss58 = pubkey_to_ss58(&pubkey_bytes)?;

    // display.fields 必须与 wumin PayloadDecoder 解码结果的 key/value 完全一致。
    // wumin 解码 propose_transfer 返回: org=机构名, beneficiary=SS58, amount_yuan="X.XX GMB", remark=文本
    let org_name = match org_type {
        0 => "国储会",
        1 => "省储会",
        2 => "省储行",
        _ => "未知",
    };
    let display = serde_json::json!({
        "action": "propose_transfer",
        "action_label": "发起转账提案",
        "summary": format!("{org_name} 提案转账 {:.2} GMB 给 {beneficiary_address}", amount_yuan),
        "fields": [
            { "key": "org", "label": "机构类型", "value": org_name },
            { "key": "beneficiary", "label": "收款地址", "value": beneficiary_address },
            { "key": "amount_yuan", "label": "金额", "value": format!("{:.2} GMB", amount_yuan) },
            { "key": "remark", "label": "备注", "value": remark }
        ]
    });

    let now = now_secs();
    let request = QrSignRequest {
        proto: PROTOCOL_VERSION.to_string(),
        msg_type: "sign_request".to_string(),
        request_id: request_id.clone(),
        account: account_ss58,
        pubkey: format!("0x{pubkey_clean}"),
        sig_alg: "sr25519".to_string(),
        payload_hex: format!("0x{}", hex::encode(&payload)),
        issued_at: now,
        expires_at: now + DEFAULT_TTL_SECS,
        display,
        spec_version: Some(spec_version),
    };

    let request_json = serde_json::to_string(&request)
        .map_err(|e| format!("序列化签名请求失败: {e}"))?;

    Ok(VoteSignRequestResult {
        request_json,
        request_id,
        expected_payload_hash: format!("0x{}", hex::encode(&payload_hash)),
        sign_nonce: nonce,
        sign_block_number: block_number,
    })
}

/// Compact<u32> 编码（公开版本，供 mod.rs 调用）。
pub fn encode_compact_u32_pub(value: u32) -> Vec<u8> {
    encode_compact_u32(value)
}

/// 从 SS58 地址解码 32 字节公钥。
pub fn decode_ss58_to_pubkey(address: &str) -> Result<[u8; 32], String> {
    let data = bs58::decode(address)
        .into_vec()
        .map_err(|_| "SS58 地址解码失败".to_string())?;
    let (prefix, prefix_len) = super::storage_keys::decode_ss58_prefix_raw(&data)?;
    if prefix != SS58_PREFIX {
        return Err(format!("SS58 地址前缀无效，期望 2027，实际 {prefix}"));
    }
    if data.len() < prefix_len + 32 + 2 {
        return Err("SS58 地址长度无效".to_string());
    }
    let (without_checksum, checksum) = data.split_at(data.len() - 2);
    let hash = blake2b_simd::Params::new()
        .hash_length(64)
        .to_state()
        .update(b"SS58PRE")
        .update(without_checksum)
        .finalize();
    if checksum != &hash.as_bytes()[..2] {
        return Err("SS58 地址校验和无效".to_string());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&data[prefix_len..prefix_len + 32]);
    Ok(out)
}

/// 验证签名响应并提交 extrinsic（通用，支持 vote_transfer 和 joint_vote）。
///
/// call_data 由调用方根据投票类型构建。
pub fn verify_and_submit(
    request_id: &str,
    expected_pubkey_hex: &str,
    expected_payload_hash: &str,
    call_data: &[u8],
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: &str,
) -> Result<VoteSubmitResult, String> {
    // 解析响应
    let response: QrSignResponse = serde_json::from_str(response_json)
        .map_err(|e| format!("解析签名响应失败: {e}"))?;

    // 验证协议版本
    if response.proto != PROTOCOL_VERSION {
        return Err(format!(
            "协议版本不匹配：期望 {PROTOCOL_VERSION}，实际 {}",
            response.proto
        ));
    }

    // 验证请求 ID 匹配
    if response.request_id != request_id {
        return Err("请求 ID 不匹配，可能扫描了其他交易的签名".to_string());
    }

    // 验证公钥匹配
    let expected_pubkey = expected_pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(expected_pubkey_hex)
        .to_ascii_lowercase();
    let response_pubkey = response
        .pubkey
        .strip_prefix("0x")
        .unwrap_or(&response.pubkey)
        .to_ascii_lowercase();
    if response_pubkey != expected_pubkey {
        return Err("公钥不匹配".to_string());
    }

    // 验证 payload hash
    // wumin 返回的 payload_hash 不含 0x 前缀，统一去掉前缀再比较
    let expected_hash = expected_payload_hash
        .strip_prefix("0x")
        .unwrap_or(expected_payload_hash)
        .to_ascii_lowercase();
    let response_hash = response
        .payload_hash
        .strip_prefix("0x")
        .unwrap_or(&response.payload_hash)
        .to_ascii_lowercase();
    if response_hash != expected_hash {
        return Err("payload hash 不匹配，签名数据可能被篡改".to_string());
    }

    // 提取签名
    let sig_hex = response
        .signature
        .strip_prefix("0x")
        .unwrap_or(&response.signature);
    if sig_hex.len() != 128 {
        return Err(format!("签名长度无效：期望 128 hex，实际 {}", sig_hex.len()));
    }
    let signature_bytes = hex::decode(sig_hex)
        .map_err(|e| format!("签名解码失败: {e}"))?;

    // 提取公钥
    let pubkey_hex_clean = expected_pubkey
        .strip_prefix("0x")
        .unwrap_or(&expected_pubkey);
    let pubkey_bytes = hex::decode(pubkey_hex_clean)
        .map_err(|e| format!("公钥解码失败: {e}"))?;

    // 使用签名时保存的 nonce 和 block_number，必须与签名载荷一致
    eprintln!("[签名提交] sign_nonce={sign_nonce}, sign_block_number={sign_block_number}");
    let era_bytes = encode_mortal_era(MORTAL_ERA_PERIOD, sign_block_number);
    eprintln!("[签名提交] era_bytes: {:?}", era_bytes);
    let nonce_compact = encode_compact_u32(sign_nonce);
    let tip_compact = encode_compact_u32(0);

    let mut extrinsic_body = Vec::new();
    // 版本字节：v4 legacy signed format = 0x84 (bit7=signed, bits0-6=version4)
    // polkadart 0.7.1 使用此格式，citizenchain runtime 同时支持 v4 和 v5
    extrinsic_body.push(0x84u8);
    // MultiAddress::Id = 0x00 + 32-byte account
    extrinsic_body.push(0x00u8);
    extrinsic_body.extend_from_slice(&pubkey_bytes);
    // SignatureType::Sr25519 = 0x01
    extrinsic_body.push(0x01u8);
    extrinsic_body.extend_from_slice(&signature_bytes);
    // extensions_signed（与 signing payload 中的 extensions_signed 完全相同）
    // AuthorizeCall(0) + CheckNonZeroSender(0) + CheckNonKeylessSender(0)
    // + CheckSpecVersion(0) + CheckTxVersion(0) + CheckGenesis(0)
    extrinsic_body.extend_from_slice(&era_bytes);       // CheckEra
    extrinsic_body.extend_from_slice(&nonce_compact);   // CheckNonce
    // CheckWeight(0)
    extrinsic_body.extend_from_slice(&tip_compact);     // ChargeTransactionPayment
    extrinsic_body.push(0x00u8);                        // CheckMetadataHash: mode=Disabled
    // WeightReclaim(0)
    // call data
    extrinsic_body.extend_from_slice(call_data);

    // Length-prefixed extrinsic
    let len_prefix = encode_compact_u32(extrinsic_body.len() as u32);
    let mut full_extrinsic = Vec::with_capacity(len_prefix.len() + extrinsic_body.len());
    full_extrinsic.extend_from_slice(&len_prefix);
    full_extrinsic.extend_from_slice(&extrinsic_body);

    // 先 dry-run 验证，避免提交错误交易导致链卡住
    let extrinsic_hex = format!("0x{}", hex::encode(&full_extrinsic));
    eprintln!("[签名提交] extrinsic hex ({} bytes): {}", full_extrinsic.len(), &extrinsic_hex[..extrinsic_hex.len().min(200)]);
    eprintln!("[签名提交] call_data hex: 0x{}", hex::encode(call_data));

    let dry_run_result = rpc_post(
        "system_dryRun",
        Value::Array(vec![Value::String(extrinsic_hex.clone())]),
    );
    match &dry_run_result {
        Ok(v) => {
            let s = v.as_str().unwrap_or("");
            eprintln!("[签名提交] dry-run 结果: {s}");
            // dry-run 返回 SCALE 编码的 ApplyExtrinsicResult:
            //   0x0000 = Ok(Ok(())) 成功
            //   0x00 01 xx = Ok(Err(DispatchError)) 可调度错误
            //   0x01 00 xx = Err(InvalidTransaction::xxx)
            //   0x01 01 xx = Err(UnknownTransaction::xxx)
            let result_hex = s.strip_prefix("0x").unwrap_or(s);
            let result_bytes = hex::decode(result_hex).unwrap_or_default();
            if result_bytes.is_empty() {
                eprintln!("[签名提交] dry-run 结果为空，跳过检查");
            } else if result_bytes[0] != 0x00 {
                // 外层 Result = Err → TransactionValidityError
                let err_kind = if result_bytes.len() > 1 && result_bytes[1] == 0x00 {
                    let code = result_bytes.get(2).copied().unwrap_or(0);
                    match code {
                        0 => "Call",
                        1 => "Payment",
                        2 => "Future",
                        3 => "Stale",
                        4 => "BadProof",
                        5 => "AncientBirthBlock",
                        6 => "ExhaustsResources",
                        _ => "Unknown",
                    }
                } else {
                    "UnknownTransaction"
                };
                eprintln!(
                    "[签名提交] dry-run 返回 InvalidTransaction::{err_kind}，继续尝试提交（依赖 pool 验证）"
                );
            } else if result_bytes.len() > 1 && result_bytes[1] != 0x00 {
                // Ok(Err(DispatchError)) — 交易格式正确但执行会失败，阻止提交
                return Err(format!(
                    "交易执行会失败: DispatchError (hex: {s})"
                ));
            }
            // 0x0000 = Ok(Ok(())) → 可以提交
        }
        Err(e) => {
            eprintln!("[签名提交] dry-run RPC 失败: {e}");
            // dry-run RPC 可能不可用，不阻止提交
            eprintln!("[签名提交] 跳过 dry-run 检查，继续提交");
        }
    }

    // dry-run 通过后再正式提交
    let result = rpc_post(
        "author_submitExtrinsic",
        Value::Array(vec![Value::String(extrinsic_hex)]),
    )?;

    let tx_hash = result
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    Ok(VoteSubmitResult { tx_hash })
}

// ──── RPC 查询 ────

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(method, params, RPC_REQUEST_TIMEOUT, MAX_RPC_RESPONSE_BYTES)
}

fn fetch_runtime_version() -> Result<(u32, u32), String> {
    let result = rpc_post("state_getRuntimeVersion", Value::Array(vec![]))?;
    let spec = result
        .get("specVersion")
        .and_then(|v| v.as_u64())
        .ok_or("缺少 specVersion")?;
    let tx = result
        .get("transactionVersion")
        .and_then(|v| v.as_u64())
        .ok_or("缺少 transactionVersion")?;
    Ok((spec as u32, tx as u32))
}

/// 构建 signing payload，严格按 citizenchain 的 TxExtension 顺序编码。
///
/// 签名载荷 = call_data + extensions_signed + extensions_implicit
///
/// extensions_signed（放在 extrinsic body 中、也是 signing payload 的一部分）:
///   AuthorizeCall: 0B, CheckNonZeroSender: 0B, CheckNonKeylessSender: 0B,
///   CheckSpecVersion: 0B, CheckTxVersion: 0B, CheckGenesis: 0B,
///   CheckEra: 2B (mortal era), CheckNonce: compact(nonce), CheckWeight: 0B,
///   ChargeTransactionPayment: compact(tip), CheckMetadataHash: 1B (mode=0), WeightReclaim: 0B
///
/// extensions_implicit（仅在 signing payload 中追加）:
///   AuthorizeCall: 0B, CheckNonZeroSender: 0B, CheckNonKeylessSender: 0B,
///   CheckSpecVersion: 4B (u32_le), CheckTxVersion: 4B (u32_le), CheckGenesis: 32B,
///   CheckEra: 32B (block_hash), CheckNonce: 0B, CheckWeight: 0B,
///   ChargeTransactionPayment: 0B, CheckMetadataHash: 1B (Option::None=0x00), WeightReclaim: 0B
fn build_signing_payload(
    call_data: &[u8],
    genesis_hash: &[u8; 32],
    block_hash: &[u8; 32],
    block_number: u64,
    nonce: u32,
    spec_version: u32,
    tx_version: u32,
) -> Vec<u8> {
    let era_bytes = encode_mortal_era(MORTAL_ERA_PERIOD, block_number);
    let nonce_compact = encode_compact_u32(nonce);
    let tip_compact = encode_compact_u32(0);

    let mut payload = Vec::new();
    // call data
    payload.extend_from_slice(call_data);
    // extensions_signed（与 extrinsic body 中的扩展字节相同）
    // AuthorizeCall(0) + CheckNonZeroSender(0) + CheckNonKeylessSender(0)
    // + CheckSpecVersion(0) + CheckTxVersion(0) + CheckGenesis(0)
    payload.extend_from_slice(&era_bytes);       // CheckEra: mortal era 2 bytes
    payload.extend_from_slice(&nonce_compact);   // CheckNonce: compact nonce
    // CheckWeight(0)
    payload.extend_from_slice(&tip_compact);     // ChargeTransactionPayment: compact tip
    payload.push(0x00u8);                        // CheckMetadataHash: mode=Disabled
    // WeightReclaim(0)

    // extensions_implicit（additional signed data）
    // AuthorizeCall(0) + CheckNonZeroSender(0) + CheckNonKeylessSender(0)
    payload.extend_from_slice(&spec_version.to_le_bytes());  // CheckSpecVersion: u32
    payload.extend_from_slice(&tx_version.to_le_bytes());    // CheckTxVersion: u32
    payload.extend_from_slice(genesis_hash);                 // CheckGenesis: H256
    payload.extend_from_slice(block_hash);                   // CheckEra: birth block hash H256
    // CheckNonce(0) + CheckWeight(0) + ChargeTransactionPayment(0)
    payload.push(0x00u8);                                    // CheckMetadataHash: Option::None
    // WeightReclaim(0)

    payload
}

fn fetch_genesis_hash() -> Result<[u8; 32], String> {
    let result = rpc_post(
        "chain_getBlockHash",
        Value::Array(vec![Value::Number(0.into())]),
    )?;
    let hash_str = result
        .as_str()
        .ok_or("genesis hash 格式无效")?;
    decode_hash32(hash_str)
}

fn fetch_latest_block() -> Result<([u8; 32], u64), String> {
    let header = rpc_post("chain_getHeader", Value::Array(vec![]))?;
    let hash_result = rpc_post("chain_getBlockHash", Value::Array(vec![]))?;

    let block_hash = decode_hash32(
        hash_result.as_str().ok_or("最新区块哈希格式无效")?,
    )?;

    let number_hex = header
        .get("number")
        .and_then(|v| v.as_str())
        .ok_or("缺少区块号")?;
    let number = u64::from_str_radix(
        number_hex.strip_prefix("0x").unwrap_or(number_hex),
        16,
    )
    .map_err(|e| format!("区块号解析失败: {e}"))?;

    Ok((block_hash, number))
}

fn fetch_nonce(pubkey_hex: &str) -> Result<u32, String> {
    let ss58 = pubkey_to_ss58(
        &hex::decode(pubkey_hex).map_err(|e| format!("公钥解码失败: {e}"))?,
    )?;
    let result = rpc_post(
        "system_accountNextIndex",
        Value::Array(vec![Value::String(ss58)]),
    )?;
    result
        .as_u64()
        .map(|v| v as u32)
        .ok_or_else(|| "nonce 格式无效".to_string())
}

// ──── 编码工具 ────

fn decode_hash32(hex_str: &str) -> Result<[u8; 32], String> {
    let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(clean).map_err(|e| format!("哈希解码失败: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!("哈希长度无效：期望 32 字节，实际 {}", bytes.len()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// Mortal era 编码（与 Substrate 的 MortalEra::new(period, block_number) 一致）。
fn encode_mortal_era(period: u64, block_number: u64) -> Vec<u8> {
    let period = period.next_power_of_two().max(4).min(1 << 16);
    let phase = block_number % period;
    let quantize_factor = (period >> 12).max(1);
    let quantized_phase = phase / quantize_factor * quantize_factor;
    let encoded = period.trailing_zeros() as u16 - 1;
    let encoded = encoded.max(1).min(15);
    let era_val = ((quantized_phase as u16) << 4) | encoded;
    vec![era_val as u8, (era_val >> 8) as u8]
}

/// Compact<u32> 编码（SCALE）。
fn encode_compact_u32(value: u32) -> Vec<u8> {
    if value < 0x40 {
        vec![(value as u8) << 2]
    } else if value < 0x4000 {
        let v = ((value as u16) << 2) | 0x01;
        vec![v as u8, (v >> 8) as u8]
    } else if value < 0x4000_0000 {
        let v = (value << 2) | 0x02;
        v.to_le_bytes().to_vec()
    } else {
        let mut out = vec![0x03u8]; // big-integer mode
        out.extend_from_slice(&value.to_le_bytes());
        out
    }
}

/// 将 32 字节公钥编码为 SS58 地址（prefix=2027）。
fn pubkey_to_ss58(pubkey: &[u8]) -> Result<String, String> {
    if pubkey.len() != 32 {
        return Err("公钥长度必须为 32 字节".to_string());
    }
    // SS58 prefix 2027 的双字节编码：
    // byte0 = ((2027 & 0x00fc) >> 2) | 0x40 = ((2027 & 252) >> 2) | 64
    //        = (8 >> 2) | 64 = 2 | 64 = 66
    // Wait, 2027 in binary: 0b11111101011
    // For two-byte SS58: first_byte = ((prefix & 0xFC) >> 2) | 0x40
    //                     second_byte = (prefix >> 8) | ((prefix & 0x03) << 6)
    let prefix = SS58_PREFIX;
    let first = ((prefix & 0x00fc) >> 2) as u8 | 0x40;
    let second = ((prefix >> 8) as u8) | (((prefix & 0x03) << 6) as u8);

    let mut payload = Vec::with_capacity(2 + 32);
    payload.push(first);
    payload.push(second);
    payload.extend_from_slice(pubkey);

    // Blake2b-512 checksum
    let hash = blake2b_simd::Params::new()
        .hash_length(64)
        .to_state()
        .update(b"SS58PRE")
        .update(&payload)
        .finalize();

    payload.push(hash.as_bytes()[0]);
    payload.push(hash.as_bytes()[1]);

    Ok(bs58::encode(&payload).into_string())
}

fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn generate_request_id(prefix: &str) -> String {
    let random_bytes: [u8; 16] = rand::random();
    format!("{}-{}", prefix, hex::encode(random_bytes))
}

fn format_proposal_id(id: u64) -> String {
    let year = id / 1_000_000;
    let counter = id % 1_000_000;
    format!("{year}#{counter}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_compact_u32_single_byte() {
        assert_eq!(encode_compact_u32(0), vec![0x00]);
        assert_eq!(encode_compact_u32(1), vec![0x04]);
        assert_eq!(encode_compact_u32(63), vec![0xfc]);
    }

    #[test]
    fn encode_compact_u32_two_bytes() {
        assert_eq!(encode_compact_u32(64), vec![0x01, 0x01]);
    }

    #[test]
    fn encode_mortal_era_period64() {
        let era = encode_mortal_era(64, 100);
        assert_eq!(era.len(), 2);
        // period=64, phase=100%64=36, encoded=5
        // era_val = (36 << 4) | 5 = 576 | 5 = 581 = 0x0245
        assert_eq!(era, vec![0x45, 0x02]);
    }

    #[test]
    fn sha256_hash_deterministic() {
        let h1 = sha256_hash(b"hello");
        let h2 = sha256_hash(b"hello");
        assert_eq!(h1, h2);
        assert_ne!(h1, sha256_hash(b"world"));
    }

    #[test]
    fn pubkey_to_ss58_roundtrip() {
        let pubkey = [0xAAu8; 32];
        let ss58 = pubkey_to_ss58(&pubkey).unwrap();
        assert!(!ss58.is_empty());
        // 验证可以用 bs58 解码回来
        let decoded = bs58::decode(&ss58).into_vec().unwrap();
        assert_eq!(&decoded[2..34], &pubkey);
    }
}
