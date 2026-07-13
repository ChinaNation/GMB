use super::call_data;
use crate::governance::signing::{
    build_signing_payloads, chain_action_code, fetch_genesis_hash, fetch_nonce,
    fetch_runtime_version, generate_request_id, now_secs, payload_b64, pubkey_b64,
    remember_chain_sign_request_session, sha256_hash, QrSignRequest, SignRequestBody,
    VoteSignRequestResult, DEFAULT_TTL_SECS, IMMORTAL_SIGN_BLOCK_NUMBER, PROTOCOL_VERSION,
    QR_KIND_SIGN_REQUEST,
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
) -> Result<VoteSignRequestResult, String> {
    let (spec_version, tx_version) = fetch_runtime_version()?;
    let genesis_hash = fetch_genesis_hash()?;
    let nonce = fetch_nonce(pubkey_clean)?;

    let (full_payload, payload_for_qr) =
        build_signing_payloads(call_data, &genesis_hash, nonce, spec_version, tx_version)?;
    let request_id = generate_request_id(request_prefix);

    // Runtime WASM 交易 payload 远大于 QR 承载能力。Substrate sr25519 在 payload
    // 超过 256 字节时实际签 blake2_256(payload)，这里直接复用 runtime helper 返回的
    // signing_bytes，禁止 runtime-upgrade 另起一套哈希规则。
    // expected_payload_hash 必须对应 QR 中实际发送的 payload_hex。
    let payload_hash = sha256_hash(&payload_for_qr);
    let payload_hash_hex = hex::encode(payload_hash);
    let full_payload_hash_hex = hex::encode(sha256_hash(&full_payload));

    let now = now_secs()?;
    let expires_at = now + DEFAULT_TTL_SECS;
    let request = QrSignRequest {
        proto: PROTOCOL_VERSION.to_string(),
        kind: QR_KIND_SIGN_REQUEST,
        id: request_id.clone(),
        expires_at,
        body: SignRequestBody {
            action: chain_action_code(call_data)?,
            sig_alg: 1,
            pubkey: pubkey_b64(pubkey_bytes)?,
            payload: payload_b64(&payload_for_qr),
        },
    };

    let request_json =
        serde_json::to_string(&request).map_err(|e| format!("序列化签名请求失败: {e}"))?;
    remember_chain_sign_request_session(
        &request_id,
        pubkey_clean,
        call_data,
        &full_payload_hash_hex,
        &payload_hash_hex,
        nonce,
        expires_at,
    )?;

    Ok(VoteSignRequestResult {
        request_json,
        call_data_hex: hex::encode(call_data),
        request_id,
        expected_payload_hash: format!("0x{}", payload_hash_hex),
        sign_nonce: nonce,
        sign_block_number: IMMORTAL_SIGN_BLOCK_NUMBER,
    })
}

/// 构建开发期直接升级签名请求。
pub(crate) fn build_developer_upgrade_sign_request(
    pubkey_hex: &str,
    wasm_path: &str,
    pow_params: pow_difficulty::PowDifficultyParams,
) -> Result<VoteSignRequestResult, String> {
    let (pubkey_clean, pubkey_bytes) = normalize_pubkey(pubkey_hex)?;
    let (wasm_code, _wasm_size_mb) = call_data::read_wasm(wasm_path)?;
    let call_data = call_data::developer_direct_upgrade(&wasm_code, pow_params);

    build_hashed_payload_request("devupg", &pubkey_clean, &pubkey_bytes, &call_data)
}

/// 构建运行期协议升级提案签名请求。
pub(crate) fn build_propose_runtime_upgrade_sign_request(
    pubkey_hex: &str,
    wasm_path: &str,
    reason: &str,
    pow_params: pow_difficulty::PowDifficultyParams,
) -> Result<VoteSignRequestResult, String> {
    let (pubkey_clean, pubkey_bytes) = normalize_pubkey(pubkey_hex)?;
    let (wasm_code, _wasm_size_mb) = call_data::read_wasm(wasm_path)?;
    let call_data = call_data::propose_runtime_upgrade(&wasm_code, reason, pow_params)?;

    build_hashed_payload_request("upgrade", &pubkey_clean, &pubkey_bytes, &call_data)
}
