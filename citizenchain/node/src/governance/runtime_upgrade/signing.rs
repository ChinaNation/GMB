use super::call_data;
use crate::governance::signing::{
    build_signing_payload, fetch_genesis_hash, fetch_latest_block, fetch_nonce,
    fetch_runtime_version, generate_request_id, now_secs, pubkey_to_ss58, sha256_hash,
    QrSignRequest, SignRequestBody, VoteSignRequestResult, DEFAULT_TTL_SECS, PROTOCOL_VERSION,
};

fn normalize_pubkey(pubkey_hex: &str) -> Result<(String, Vec<u8>), String> {
    let pubkey_clean = pubkey_hex
        .strip_prefix("0x")
        .unwrap_or(pubkey_hex)
        .to_ascii_lowercase();
    if pubkey_clean.len() != 64 || !pubkey_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("公钥格式无效，应为 64 位十六进制".to_string());
    }
    let pubkey_bytes = hex::decode(&pubkey_clean).map_err(|e| format!("公钥解码失败: {e}"))?;
    Ok((pubkey_clean, pubkey_bytes))
}

fn build_hashed_payload_request(
    request_prefix: &str,
    pubkey_clean: &str,
    pubkey_bytes: &[u8],
    call_data: &[u8],
    display: serde_json::Value,
) -> Result<VoteSignRequestResult, String> {
    let (spec_version, tx_version) = fetch_runtime_version()?;
    let genesis_hash = fetch_genesis_hash()?;
    let (block_hash, block_number) = fetch_latest_block()?;
    let nonce = fetch_nonce(pubkey_clean)?;

    let payload = build_signing_payload(
        call_data,
        &genesis_hash,
        &block_hash,
        block_number,
        nonce,
        spec_version,
        tx_version,
    );
    let request_id = generate_request_id(request_prefix);
    let account_ss58 = pubkey_to_ss58(pubkey_bytes)?;

    // Runtime WASM 交易 payload 远大于 QR 承载能力。Substrate sr25519 在 payload
    // 超过 256 字节时实际签 blake2_256(payload),所以这里把同一个 32 字节摘要交给冷钱包。
    let payload_for_qr = blake2b_simd::Params::new().hash_length(32).hash(&payload);
    // expected_payload_hash 必须对应 QR 中实际发送的 payload_hex。
    let payload_hash = sha256_hash(payload_for_qr.as_bytes());

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
            payload_hex: format!("0x{}", hex::encode(payload_for_qr.as_bytes())),
            display,
        },
    };

    let request_json =
        serde_json::to_string(&request).map_err(|e| format!("序列化签名请求失败: {e}"))?;

    Ok(VoteSignRequestResult {
        request_json,
        call_data_hex: hex::encode(call_data),
        request_id,
        expected_payload_hash: format!("0x{}", hex::encode(&payload_hash)),
        sign_nonce: nonce,
        sign_block_number: block_number,
    })
}

/// 构建开发期直接升级签名请求。
pub(crate) fn build_developer_upgrade_sign_request(
    pubkey_hex: &str,
    wasm_path: &str,
) -> Result<VoteSignRequestResult, String> {
    let (pubkey_clean, pubkey_bytes) = normalize_pubkey(pubkey_hex)?;
    let (wasm_code, wasm_size_mb) = call_data::read_wasm(wasm_path)?;
    let call_data = call_data::developer_direct_upgrade(&wasm_code);

    let display = serde_json::json!({
        "action": "developer_direct_upgrade",
        "summary": format!("开发期直接升级（{wasm_size_mb:.2} MB）"),
        "fields": [
            { "key": "wasm_size", "label": "WASM 大小", "value": format!("{wasm_size_mb:.2} MB") },
            { "key": "wasm_hash", "label": "代码哈希", "value": format!("0x{}", hex::encode(sha256_hash(&wasm_code))) }
        ]
    });

    build_hashed_payload_request("devupg", &pubkey_clean, &pubkey_bytes, &call_data, display)
}

/// 构建运行期协议升级提案签名请求。
pub(crate) fn build_propose_runtime_upgrade_sign_request(
    pubkey_hex: &str,
    wasm_path: &str,
    reason: &str,
) -> Result<VoteSignRequestResult, String> {
    let (pubkey_clean, pubkey_bytes) = normalize_pubkey(pubkey_hex)?;
    let (wasm_code, wasm_size_mb) = call_data::read_wasm(wasm_path)?;
    let call_data = call_data::propose_runtime_upgrade(&wasm_code, reason)?;

    let display = serde_json::json!({
        "action": "propose_runtime_upgrade",
        "summary": format!("提交协议升级提案（{wasm_size_mb:.2} MB）"),
        "fields": [
            { "key": "reason", "label": "升级理由", "value": reason },
            { "key": "wasm_size", "label": "WASM 大小", "value": format!("{wasm_size_mb:.2} MB") },
            { "key": "wasm_hash", "label": "代码哈希", "value": format!("0x{}", hex::encode(sha256_hash(&wasm_code))) }
        ]
    });

    build_hashed_payload_request("upgrade", &pubkey_clean, &pubkey_bytes, &call_data, display)
}
