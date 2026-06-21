// 治理投票 QR 签名：构建 CITIZEN_QR_V1 签名请求、验证响应、提交 extrinsic。
//
// 协议流程：
// 1. 后端构建未签名 signing payload + QR 请求 JSON
// 2. 前端显示 QR 码 → 用户用 citizenwallet 离线设备扫码签名
// 3. 前端摄像头扫描响应 QR → 传回后端
// 4. 后端验证 payload_hash → 构建 signed extrinsic → 提交到链

use crate::shared::rpc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const PROTOCOL_VERSION: &str = "CITIZEN_QR_V1";
pub(crate) const DEFAULT_TTL_SECS: u64 = 90;
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
use crate::shared::constants::RPC_RESPONSE_LIMIT_SMALL;
/// SS58 前缀 2027。
const SS58_PREFIX: u16 = 2027;

fn institution_account_from_cid(cid_number: &str) -> Result<[u8; 32], String> {
    let entry = super::registry::find_institution(cid_number)
        .ok_or_else(|| format!("未知的治理机构 cidNumber: {cid_number}"))?;
    let clean = entry.main_account_hex();
    let bytes = hex::decode(&clean).map_err(|e| format!("机构 AccountId 解码失败: {e}"))?;
    bytes
        .try_into()
        .map_err(|_| "机构 AccountId 必须为 32 字节".to_string())
}

/// 金额格式化：带千分位逗号，保留 2 位小数。
pub(crate) fn format_amount(yuan: f64) -> String {
    let fixed = format!("{:.2}", yuan);
    let parts: Vec<&str> = fixed.split('.').collect();
    let int_part = parts[0];
    let dec_part = parts.get(1).unwrap_or(&"00");
    let negative = int_part.starts_with('-');
    let digits: &str = if negative { &int_part[1..] } else { int_part };
    let mut result = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    let formatted: String = result.chars().rev().collect();
    if negative {
        format!("-{}.{}", formatted, dec_part)
    } else {
        format!("{}.{}", formatted, dec_part)
    }
}

// ──── QR 协议数据结构 ────

/// 签名请求 body(节点桌面端 → 离线设备)。
///
/// 注:历史上含 `spec_version: u32` 字段供冷钱包 decoder 锁布局,已随 strict
/// 两色模式独家把关而删除(2026-05-07)。SCALE additional_signed 的 spec_version
/// 仍在 payload_hex 内部 4 字节编码,链端验签直接拿这个,不依赖 envelope 字段。
#[derive(Debug, Serialize)]
pub struct SignRequestBody {
    pub address: String,
    pub pubkey: String,
    pub sig_alg: String,
    pub payload_hex: String,
    pub display: serde_json::Value,
}

/// CITIZEN_QR_V1 sign_request envelope。
#[derive(Debug, Serialize)]
pub struct QrSignRequest {
    pub proto: String,
    pub kind: String,
    pub id: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub body: SignRequestBody,
}

/// 签名响应 body(离线设备 → 节点桌面端)。
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SignResponseBody {
    pub pubkey: String,
    pub sig_alg: String,
    pub signature: String,
    pub payload_hash: String,
    pub signed_at: u64,
}

/// CITIZEN_QR_V1 sign_response envelope。
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct QrSignResponse {
    pub proto: String,
    pub kind: String,
    pub id: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub body: SignResponseBody,
}

/// 构建投票签名请求的结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteSignRequestResult {
    /// 完整的 QR 签名请求 JSON 字符串。
    pub request_json: String,
    /// 后端构造的完整 call data hex（不含 0x）。
    pub call_data_hex: String,
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

/// 构建内部投票（`internal_vote`）签名请求。
///
/// Phase 3(2026-04-22)「投票引擎统一入口整改」:业务 pallet 的 vote_X 已物理删除。
/// 管理员一人一票统一走 `InternalVote::cast`(pallet=22, call=0,sub-pallet 拆分
/// 2026-05-05),由投票引擎按 ProposalData 前缀自动分派到对应 `InternalVoteExecutor`。
///
/// Call 编码: `[0x16][0x00][proposal_id:u64_le][approve:bool]` 共 11 字节。
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
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;

    // 获取链上参数
    let (spec_version, tx_version) = fetch_runtime_version()?;
    let genesis_hash = fetch_genesis_hash()?;
    let (block_hash, block_number) = fetch_latest_block()?;
    let nonce = fetch_nonce(&pubkey_clean)?;

    // 构建 call data: [pallet=22][call=0][proposal_id: u64_le][approve: bool]
    let mut call_data = Vec::with_capacity(11);
    call_data.push(22u8); // InternalVote sub-pallet index (sub-pallet split 2026-05-05)
    call_data.push(0u8); // cast call index
    call_data.extend_from_slice(&proposal_id.to_le_bytes());
    call_data.push(if approve { 1u8 } else { 0u8 });

    // 构建 signing payload
    let payload = build_signing_payload(
        &call_data,
        &genesis_hash,
        &block_hash,
        block_number,
        nonce,
        spec_version,
        tx_version,
    );

    // 计算 payload hash
    let payload_hash = sha256_hash(&payload);

    // 生成请求 ID
    let request_id = generate_request_id("vote");

    // SS58 编码账户地址
    let account_ss58 = pubkey_to_ss58(&pubkey_bytes)?;

    // display.fields 必须与 citizenwallet PayloadDecoder 解码结果的 key/value 完全一致。
    // citizenwallet 解码 internal_vote 返回: proposal_id=数字字符串, approve="true"/"false"
    let display = serde_json::json!({
        "action": "internal_vote",
        "summary": format!("管理员投票 提案 #{proposal_id}：{}", if approve { "赞成" } else { "反对" }),
        "fields": [
            { "key": "proposal_id", "label": "提案编号", "value": proposal_id.to_string() },
            { "key": "approve", "label": "投票", "value": approve.to_string() }
        ]
    });

    let now = now_secs()?;
    let request = QrSignRequest {
        proto: PROTOCOL_VERSION.to_string(),
        kind: "sign_request".to_string(),
        id: request_id.clone(),
        issued_at: now,
        expires_at: now + DEFAULT_TTL_SECS,
        body: SignRequestBody {
            address: account_ss58,
            pubkey: format!("0x{pubkey_clean}"),
            sig_alg: "sr25519".to_string(),
            payload_hex: format!("0x{}", hex::encode(&payload)),
            display,
        },
    };

    let request_json =
        serde_json::to_string(&request).map_err(|e| format!("序列化签名请求失败: {e}"))?;

    Ok(VoteSignRequestResult {
        request_json,
        call_data_hex: hex::encode(&call_data),
        request_id,
        expected_payload_hash: format!("0x{}", hex::encode(&payload_hash)),
        sign_nonce: nonce,
        sign_block_number: block_number,
    })
}

/// 构建 joint_vote 签名请求（联合投票内部投票阶段：pallet=23, call=0）。
///
/// sub-pallet 拆分(2026-05-05):JointVote 独立成 pallet,`cast_admin` 在 23.0,
/// `cast_referendum` 在 23.1(联合公投阶段需 ADR-008 step3 双层凭证,本函数不覆盖)。
///
/// cid_number 用于查找机构多签 AccountId32 参数。
pub fn build_joint_vote_sign_request(
    proposal_id: u64,
    pubkey_hex: &str,
    cid_number: &str,
    approve: bool,
) -> Result<VoteSignRequestResult, String> {
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if pubkey_clean.len() != 64 || !pubkey_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效，应为 64 位十六进制".to_string());
    }
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;

    let institution_account = institution_account_from_cid(cid_number)?;

    let (spec_version, tx_version) = fetch_runtime_version()?;
    let genesis_hash = fetch_genesis_hash()?;
    let (block_hash, block_number) = fetch_latest_block()?;
    let nonce = fetch_nonce(&pubkey_clean)?;

    // call data: [pallet=23][call=0][proposal_id:u64_le][institution_account:AccountId32][approve:bool]
    let mut call_data = Vec::with_capacity(1 + 1 + 8 + 32 + 1);
    call_data.push(23u8); // JointVote sub-pallet index (sub-pallet split 2026-05-05)
    call_data.push(0u8); // cast_admin call index
    call_data.extend_from_slice(&proposal_id.to_le_bytes());
    call_data.extend_from_slice(&institution_account);
    call_data.push(if approve { 1u8 } else { 0u8 });

    let payload = build_signing_payload(
        &call_data,
        &genesis_hash,
        &block_hash,
        block_number,
        nonce,
        spec_version,
        tx_version,
    );
    let payload_hash = sha256_hash(&payload);
    let request_id = generate_request_id("jvote");
    let account_ss58 = pubkey_to_ss58(&pubkey_bytes)?;

    // display.fields 必须与 citizenwallet PayloadDecoder 解码结果的 key/value 完全一致。
    // citizenwallet 解码 joint_vote 返回: proposal_id=数字字符串, approve="true"/"false"
    let display = serde_json::json!({
        "action": "joint_vote",
        "summary": format!("联合投票 提案 #{proposal_id}：{}", if approve { "赞成" } else { "反对" }),
        "fields": [
            { "key": "proposal_id", "label": "提案编号", "value": proposal_id.to_string() },
            { "key": "approve", "label": "投票", "value": approve.to_string() }
        ]
    });

    let now = now_secs()?;
    let request = QrSignRequest {
        proto: PROTOCOL_VERSION.to_string(),
        kind: "sign_request".to_string(),
        id: request_id.clone(),
        issued_at: now,
        expires_at: now + DEFAULT_TTL_SECS,
        body: SignRequestBody {
            address: account_ss58,
            pubkey: format!("0x{pubkey_clean}"),
            sig_alg: "sr25519".to_string(),
            payload_hex: format!("0x{}", hex::encode(&payload)),
            display,
        },
    };

    let request_json =
        serde_json::to_string(&request).map_err(|e| format!("序列化签名请求失败: {e}"))?;

    Ok(VoteSignRequestResult {
        request_json,
        call_data_hex: hex::encode(&call_data),
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
    let (prefix, prefix_len) = crate::settings::address_utils::decode_ss58_prefix(&data)?;
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
    let response: QrSignResponse =
        serde_json::from_str(response_json).map_err(|e| format!("解析签名响应失败: {e}"))?;

    // 验证协议版本
    if response.proto != PROTOCOL_VERSION {
        return Err(format!(
            "协议版本不匹配：期望 {PROTOCOL_VERSION}，实际 {}",
            response.proto
        ));
    }

    // 验证请求 ID 匹配
    if response.id != request_id {
        return Err("请求 ID 不匹配,可能扫描了其他交易的签名".to_string());
    }

    // 验证公钥匹配
    let expected_pubkey = expected_pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(expected_pubkey_hex)
        .to_ascii_lowercase();
    let response_pubkey = response
        .body
        .pubkey
        .strip_prefix("0x")
        .unwrap_or(&response.body.pubkey)
        .to_ascii_lowercase();
    if response_pubkey != expected_pubkey {
        return Err("公钥不匹配".to_string());
    }

    // 验证 payload hash
    let expected_hash = expected_payload_hash
        .strip_prefix("0x")
        .unwrap_or(expected_payload_hash)
        .to_ascii_lowercase();
    let response_hash = response
        .body
        .payload_hash
        .strip_prefix("0x")
        .unwrap_or(&response.body.payload_hash)
        .to_ascii_lowercase();
    if response_hash != expected_hash {
        return Err("payload hash 不匹配,签名数据可能被篡改".to_string());
    }

    // 提取签名
    let sig_hex = response
        .body
        .signature
        .strip_prefix("0x")
        .unwrap_or(&response.body.signature);
    if sig_hex.len() != 128 {
        return Err(format!(
            "签名长度无效：期望 128 hex，实际 {}",
            sig_hex.len()
        ));
    }
    let signature_bytes = hex::decode(sig_hex).map_err(|e| format!("签名解码失败: {e}"))?;

    // 提取公钥
    let pubkey_hex_clean = expected_pubkey
        .strip_prefix("0x")
        .unwrap_or(&expected_pubkey);
    let pubkey_bytes = hex::decode(pubkey_hex_clean).map_err(|e| format!("公钥解码失败: {e}"))?;

    // 使用签名时保存的 nonce 和 block_number，必须与签名载荷一致
    eprintln!("[签名提交] sign_nonce={sign_nonce}, sign_block_number={sign_block_number}");
    // immortal era(单字节 0x00):PoW 链块速变化大 + 冷钱包签名流程数分钟,
    // mortal era=64 块经常导致 "ancient birth block",改 immortal 永不过期。
    // 防重放靠 nonce(链上一次性消费)。规则:feedback_cid_pow_chain_recipe.md
    let era_bytes = vec![0x00u8];
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
    // AuthorizeCall(0) + CheckNonZeroSender(0) + CheckNonStakeSender(0)
    // + CheckSpecVersion(0) + CheckTxVersion(0) + CheckGenesis(0)
    extrinsic_body.extend_from_slice(&era_bytes); // CheckEra
    extrinsic_body.extend_from_slice(&nonce_compact); // CheckNonce
                                                      // CheckWeight(0)
    extrinsic_body.extend_from_slice(&tip_compact); // ChargeTransactionPayment
    extrinsic_body.push(0x00u8); // CheckMetadataHash: mode=Disabled
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
    eprintln!(
        "[签名提交] extrinsic hex ({} bytes): {}",
        full_extrinsic.len(),
        &extrinsic_hex[..extrinsic_hex.len().min(200)]
    );
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
            // 中文注释：dry-run 已应答但结果无法解码/为空属于异常应答，此时放行
            // 提交等于放弃校验，必须拒绝；与下方"dry-run RPC 不可用"的可用性
            // 兜底是两回事。
            let result_bytes = hex::decode(result_hex)
                .map_err(|e| format!("dry-run 结果异常，拒绝提交: {e} (raw: {s})"))?;
            if result_bytes.is_empty() {
                return Err("dry-run 返回空结果，拒绝提交".to_string());
            }
            if result_bytes[0] != 0x00 {
                // 外层 Result = Err → TransactionValidityError。
                // 中文注释：Future/Stale 等交易提交后只会"看似成功永不上链"
                // （Future 进 future 队列且不向 peer 广播），一律拒绝并把
                // 原因抛给前端，绝不再"继续尝试提交"。
                let reason = classify_invalid_tx(&result_bytes);
                eprintln!("[签名提交] dry-run 拒绝: {reason} (hex: {s})");
                return Err(dry_run_reject_message(&result_bytes, s));
            }
            if result_bytes.len() > 1 && result_bytes[1] != 0x00 {
                // Ok(Err(DispatchError)) — 交易格式正确但执行会失败，阻止提交
                return Err(format!("交易执行会失败: DispatchError (hex: {s})"));
            }
            // 0x0000 = Ok(Ok(())) → 可以提交
        }
        Err(e) => {
            // 中文注释：dry-run RPC 本身不可用（节点未启用 system_dryRun 等）
            // 时保持可用性兜底继续提交，由交易池做最终校验。
            eprintln!("[签名提交] dry-run RPC 失败: {e}");
            eprintln!("[签名提交] 跳过 dry-run 检查，继续提交");
        }
    }

    // dry-run 通过后再正式提交
    let result = rpc_post(
        "author_submitExtrinsic",
        Value::Array(vec![Value::String(extrinsic_hex)]),
    )?;

    // 中文注释：提交结果必须是交易哈希字符串；其它形态说明节点应答异常，
    // 必须上抛而不是用占位值伪装成功。
    let tx_hash = result
        .as_str()
        .ok_or_else(|| format!("author_submitExtrinsic 返回非字符串: {result}"))?
        .to_string();

    // 中文注释：被交易池接受 ≠ 已上链（nonce 错位时交易进 future 队列，永不
    // 被打包且不广播）。后台延迟核对一次 nonce 是否被消费，只打日志不阻塞。
    spawn_post_submit_audit(pubkey_hex_clean.to_string(), sign_nonce, tx_hash.clone());

    Ok(VoteSubmitResult { tx_hash })
}

/// 把 dry-run 拒绝结果转成抛给前端的报错文案。
///
/// 中文注释：Future（0x01 0x00 0x02）对用户而言就是"上一笔还没出块"——
/// 签名 nonce 排在池中上一笔之后，链上状态尚未消费——给人话提示，
/// 技术细节留在调用方日志；其余变体保留技术原因便于排查。
fn dry_run_reject_message(result_bytes: &[u8], raw_hex: &str) -> String {
    if result_bytes.starts_with(&[0x01, 0x00, 0x02]) {
        return "上一笔交易尚未出块，请稍候再试".to_string();
    }
    let reason = classify_invalid_tx(result_bytes);
    format!("交易校验失败，已拒绝提交: {reason} (hex: {raw_hex})")
}

/// 解析 dry-run 返回的 TransactionValidityError，给出可读原因。
///
/// SCALE 布局：外层 0x01 = Err；第二字节 0x00 = InvalidTransaction、
/// 0x01 = UnknownTransaction；第三字节为具体变体编号。
fn classify_invalid_tx(result_bytes: &[u8]) -> String {
    if result_bytes.len() > 1 && result_bytes[1] == 0x00 {
        let kind = match result_bytes.get(2).copied().unwrap_or(0xff) {
            0 => "Call(当前链状态下不可调度)",
            1 => "Payment(余额不足以支付手续费)",
            2 => "Future(nonce 超前，交易会卡在 future 队列永不出块)",
            3 => "Stale(nonce 已被消费，交易过期)",
            4 => "BadProof(签名校验失败)",
            5 => "AncientBirthBlock(签名时代过旧)",
            6 => "ExhaustsResources(资源超限，请稍后重试)",
            _ => "Unknown",
        };
        format!("InvalidTransaction::{kind}")
    } else {
        "UnknownTransaction".to_string()
    }
}

/// 提交后的后台核对：延迟一个出块周期后检查账户 nonce 是否前进。
///
/// 中文注释：`system_accountNextIndex` 包含就绪队列中的交易——nonce 未前进
/// 说明交易既不在就绪队列也未上链（丢失或卡 future 队列），打告警日志供排查；
/// 该核对纯观测，不影响提交结果，沿用"submit-only + 后台观察"的既定模式。
fn spawn_post_submit_audit(pubkey_hex: String, sign_nonce: u32, tx_hash: String) {
    std::thread::spawn(move || {
        // 创世期目标块时 30 秒，留 3 个周期余量再核对。
        std::thread::sleep(std::time::Duration::from_secs(90));
        match fetch_nonce(&pubkey_hex) {
            Ok(next) if next > sign_nonce => {
                eprintln!(
                    "[签名提交][后台核对] {tx_hash} nonce 已消费(next={next})，交易已上链或在就绪队列"
                );
            }
            Ok(next) => {
                eprintln!(
                    "[签名提交][后台核对] ⚠ {tx_hash} 提交 90 秒后 nonce 仍未消费(next={next}, 期望 >{sign_nonce})：交易已丢失或卡在 future 队列，不会上链，需要重新提交"
                );
            }
            Err(e) => {
                eprintln!("[签名提交][后台核对] {tx_hash} nonce 查询失败，无法核对: {e}");
            }
        }
    });
}

// ──── RPC 查询 ────

// 中文注释:chain_query(ADR-017 finalized 收口)复用本封装,放宽到 pub(crate)。
pub(crate) fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(
        method,
        params,
        RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_SMALL,
    )
}

pub(crate) fn fetch_runtime_version() -> Result<(u32, u32), String> {
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
///   AuthorizeCall: 0B, CheckNonZeroSender: 0B, CheckNonStakeSender: 0B,
///   CheckSpecVersion: 0B, CheckTxVersion: 0B, CheckGenesis: 0B,
///   CheckEra: 2B (mortal era), CheckNonce: compact(nonce), CheckWeight: 0B,
///   ChargeTransactionPayment: compact(tip), CheckMetadataHash: 1B (mode=0), WeightReclaim: 0B
///
/// extensions_implicit（仅在 signing payload 中追加）:
///   AuthorizeCall: 0B, CheckNonZeroSender: 0B, CheckNonStakeSender: 0B,
///   CheckSpecVersion: 4B (u32_le), CheckTxVersion: 4B (u32_le), CheckGenesis: 32B,
///   CheckEra: 32B (block_hash), CheckNonce: 0B, CheckWeight: 0B,
///   ChargeTransactionPayment: 0B, CheckMetadataHash: 1B (Option::None=0x00), WeightReclaim: 0B
pub(crate) fn build_signing_payload(
    call_data: &[u8],
    genesis_hash: &[u8; 32],
    block_hash: &[u8; 32],
    block_number: u64,
    nonce: u32,
    spec_version: u32,
    tx_version: u32,
) -> Vec<u8> {
    // immortal era(单字节 0x00):必须与 verify_and_submit 路径完全一致,
    // 否则签名 payload 与最终 extrinsic body 的 era 字节不匹配,链上签名校验失败。
    // 使用 immortal 的原因见 verify_and_submit 中同款注释。
    let _ = block_number; // immortal 不需要 block_number,保留参数兼容签名
    let _ = block_hash; // immortal 不需要 birth block hash,保留参数兼容签名
    let era_bytes = vec![0x00u8];
    let nonce_compact = encode_compact_u32(nonce);
    let tip_compact = encode_compact_u32(0);

    let mut payload = Vec::new();
    // call data
    payload.extend_from_slice(call_data);
    // extensions_signed（与 extrinsic body 中的扩展字节相同）
    // AuthorizeCall(0) + CheckNonZeroSender(0) + CheckNonStakeSender(0)
    // + CheckSpecVersion(0) + CheckTxVersion(0) + CheckGenesis(0)
    payload.extend_from_slice(&era_bytes); // CheckEra: 1 byte 0x00 (immortal)
    payload.extend_from_slice(&nonce_compact); // CheckNonce: compact nonce
                                               // CheckWeight(0)
    payload.extend_from_slice(&tip_compact); // ChargeTransactionPayment: compact tip
    payload.push(0x00u8); // CheckMetadataHash: mode=Disabled
                          // WeightReclaim(0)

    // extensions_implicit（additional signed data）
    // AuthorizeCall(0) + CheckNonZeroSender(0) + CheckNonStakeSender(0)
    payload.extend_from_slice(&spec_version.to_le_bytes()); // CheckSpecVersion: u32
    payload.extend_from_slice(&tx_version.to_le_bytes()); // CheckTxVersion: u32
    payload.extend_from_slice(genesis_hash); // CheckGenesis: H256
                                             // CheckEra::additional_signed:
                                             //   immortal → block_hash(0) = genesis_hash
                                             //   mortal   → block_hash(birth_block_number)
                                             // 当前固定 immortal,所以这里也填 genesis_hash(链上 frame_system::CheckEra
                                             // 在 immortal 分支会用 birth=0 取 block_hash(0),与 genesis_hash 一致)。
                                             // 之前误填 fetch_latest_block 拿到的最新块 hash → 与链端重建 payload 不匹配
                                             // → blake2_256 不同 → "Transaction has a bad signature"。
    payload.extend_from_slice(genesis_hash); // CheckEra: birth block hash = genesis(immortal)
                                             // CheckNonce(0) + CheckWeight(0) + ChargeTransactionPayment(0)
    payload.push(0x00u8); // CheckMetadataHash: Option::None
                          // WeightReclaim(0)

    payload
}

pub(crate) fn fetch_genesis_hash() -> Result<[u8; 32], String> {
    let result = rpc_post(
        "chain_getBlockHash",
        Value::Array(vec![Value::Number(0.into())]),
    )?;
    let hash_str = result.as_str().ok_or("genesis hash 格式无效")?;
    decode_hash32(hash_str)
}

pub(crate) fn fetch_latest_block() -> Result<([u8; 32], u64), String> {
    let header = rpc_post("chain_getHeader", Value::Array(vec![]))?;
    let hash_result = rpc_post("chain_getBlockHash", Value::Array(vec![]))?;

    let block_hash = decode_hash32(hash_result.as_str().ok_or("最新区块哈希格式无效")?)?;

    let number_hex = header
        .get("number")
        .and_then(|v| v.as_str())
        .ok_or("缺少区块号")?;
    let number = u64::from_str_radix(number_hex.strip_prefix("0x").unwrap_or(number_hex), 16)
        .map_err(|e| format!("区块号解析失败: {e}"))?;

    Ok((block_hash, number))
}

pub(crate) fn fetch_nonce(pubkey_hex: &str) -> Result<u32, String> {
    let ss58 = pubkey_to_ss58(&hex::decode(pubkey_hex).map_err(|e| format!("公钥解码失败: {e}"))?)?;
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

/// Compact<u32> 编码（SCALE）。
pub(crate) fn encode_compact_u32(value: u32) -> Vec<u8> {
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
pub(crate) fn pubkey_to_ss58(pubkey: &[u8]) -> Result<String, String> {
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

pub(crate) fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// SHA-256 哈希（供 activation 模块调用）。
pub(crate) fn sha256_hash_public(data: &[u8]) -> [u8; 32] {
    sha256_hash(data)
}

/// 当前 Unix 秒。
///
/// 中文注释：系统时钟早于 epoch 属于环境故障；静默返回 0 会让 QR 请求一出生
/// 就过期（issued_at=0），冷钱包只报"协议过期"而毫无线索，必须显式失败。
pub(crate) fn now_secs() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| format!("系统时钟异常（早于 Unix epoch）: {e}"))
}

pub(crate) fn generate_request_id(prefix: &str) -> String {
    let random_bytes: [u8; 16] = rand::random();
    format!("{}-{}", prefix, hex::encode(random_bytes))
}

/// 生成请求 ID（供 activation 模块调用）。
pub(crate) fn generate_request_id_public(prefix: &str) -> String {
    generate_request_id(prefix)
}

/// 通用签名请求构建：给定 call_data + display 信息，返回完整的 QR 签名请求。
///
/// 供 transaction 模块等外部调用方使用，避免重复获取链上参数和构建 payload。
pub fn build_sign_request_from_call_data(
    pubkey_hex: &str,
    pubkey_bytes: &[u8],
    call_data: &[u8],
    action: &str,
    summary: &str,
    fields: &serde_json::Value,
) -> Result<VoteSignRequestResult, String> {
    let (spec_version, tx_version) = fetch_runtime_version()?;
    let genesis_hash = fetch_genesis_hash()?;
    let (block_hash, block_number) = fetch_latest_block()?;
    let nonce = fetch_nonce(pubkey_hex)?;

    let payload = build_signing_payload(
        call_data,
        &genesis_hash,
        &block_hash,
        block_number,
        nonce,
        spec_version,
        tx_version,
    );
    let payload_hash = sha256_hash(&payload);
    let request_id = generate_request_id(action);
    let account_ss58 = pubkey_to_ss58(pubkey_bytes)?;

    let display = serde_json::json!({
        "action": action,
        "summary": summary,
        "fields": fields
    });

    let now = now_secs()?;
    let request = QrSignRequest {
        proto: PROTOCOL_VERSION.to_string(),
        kind: "sign_request".to_string(),
        id: request_id.clone(),
        issued_at: now,
        expires_at: now + DEFAULT_TTL_SECS,
        body: SignRequestBody {
            address: account_ss58,
            pubkey: format!("0x{pubkey_hex}"),
            sig_alg: "sr25519".to_string(),
            payload_hex: format!("0x{}", hex::encode(&payload)),
            display,
        },
    };

    let request_json =
        serde_json::to_string(&request).map_err(|e| format!("序列化签名请求失败: {e}"))?;

    Ok(VoteSignRequestResult {
        request_json,
        call_data_hex: hex::encode(call_data),
        request_id,
        expected_payload_hash: format!("0x{}", hex::encode(payload_hash)),
        sign_nonce: nonce,
        sign_block_number: block_number,
    })
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

    #[test]
    fn classify_invalid_tx_known_variants() {
        // 0x01 00 xx = Err(InvalidTransaction::xxx)
        assert!(classify_invalid_tx(&[0x01, 0x00, 0x02]).contains("Future"));
        assert!(classify_invalid_tx(&[0x01, 0x00, 0x03]).contains("Stale"));
        assert!(classify_invalid_tx(&[0x01, 0x00, 0x04]).contains("BadProof"));
        assert!(classify_invalid_tx(&[0x01, 0x00, 0x01]).contains("Payment"));
    }

    #[test]
    fn classify_invalid_tx_unknown_transaction() {
        // 0x01 01 xx = Err(UnknownTransaction::xxx)
        assert_eq!(
            classify_invalid_tx(&[0x01, 0x01, 0x00]),
            "UnknownTransaction"
        );
    }

    #[test]
    fn classify_invalid_tx_unrecognized_code_does_not_panic() {
        // 越界/未知变体编号不得 panic，归入 Unknown
        assert!(classify_invalid_tx(&[0x01, 0x00, 0x63]).contains("Unknown"));
        assert_eq!(classify_invalid_tx(&[0x01]), "UnknownTransaction");
    }

    #[test]
    fn dry_run_reject_future_gives_user_hint() {
        // Future = 上一笔还没出块，前端文案必须是人话，不带技术细节
        assert_eq!(
            dry_run_reject_message(&[0x01, 0x00, 0x02], "0x010002"),
            "上一笔交易尚未出块，请稍候再试"
        );
    }

    #[test]
    fn dry_run_reject_other_variants_keep_technical_reason() {
        // Future 之外的变体保持原有技术报错格式（含 hex 便于排查）
        let stale = dry_run_reject_message(&[0x01, 0x00, 0x03], "0x010003");
        assert!(stale.contains("交易校验失败，已拒绝提交"));
        assert!(stale.contains("Stale"));
        assert!(stale.contains("0x010003"));

        let unknown_tx = dry_run_reject_message(&[0x01, 0x01, 0x00], "0x010100");
        assert!(unknown_tx.contains("UnknownTransaction"));
    }

    #[test]
    fn now_secs_returns_positive() {
        // 正常系统时钟下必须返回 epoch 之后的正数秒
        assert!(now_secs().unwrap() > 1_700_000_000);
    }
}
